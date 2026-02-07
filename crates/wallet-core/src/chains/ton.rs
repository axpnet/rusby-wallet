// Rusby Wallet — Pure Rust multi-chain crypto wallet
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// chains/ton: TON address derivation (Ed25519 + wallet v4r2)
//
// Functions:
//   derive_ton_address()            — seed → SLIP-10 m/44'/607'/0' → Ed25519 → v4r2 address
//   decode_ton_friendly_address()   — base64url address → raw bytes (tag + workchain + hash)
//   crc16_xmodem()                  — CRC16-XMODEM for TON address checksum
//
// NOTE: This implementation uses the proper wallet v4r2 state_init hash for
// address derivation. The code BOC is parsed at derivation time to compute
// the code cell hash.

use crate::bip32_utils::{self, DerivationPath};
use ed25519_dalek::SigningKey;
use sha2::{Digest, Sha256};

use super::{Chain, ChainId};

/// Standard wallet v4r2 code BOC (base64, standard TON format without CRC)
/// Source: tonwhales/ton — WalletContractV4.ts
const WALLET_V4R2_CODE_BOC_B64: &str = "\
te6ccgECFAEAAtQAART/APSkE/S88sgLAQIBIAIDAgFIBAUE+PKDCNcYINMf0x/THwL4I7vyZO1E\
0NMf0x/T//QE0VFDuvKhUVG68qIF+QFUEGT5EPKj+AAkpMjLH1JAyx9SMMv/UhD0AMntVPgPAdMH\
IcAAn2xRkyDXSpbTB9QC+wDoMOAhwAHjACHAAuMAAcADkTDjDQOkyMsfEssfy/8QERITAubQAdDT\
AyFxsJJfBOAi10nBIJJfBOAC0x8hghBwbHVnvSKCEGRzdHK9sJJfBeAD+kAwIPpEAcjKB8v/ydDt\
RNCBAUDXIfQEMFyBAQj0Cm+hMbOSXwfgBdM/yCWCEHBsdWe6kjgw4w0DghBkc3RyupJfBuMNBgcC\
ASAICQB4AfoA9AQw+CdvIjBQCqEhvvLgUIIQcGx1Z4MesXCAGFAEywUmzxZY+gIZ9ADLaRfLH1Jg\
yz8gyYBA+wAGAIpQBIEBCPRZMO1E0IEBQNcgyAHPFvQAye1UAXKwjiOCEGRzdHKDHrFwgBhQBcsF\
UAPPFiP6AhPLassfyz/JgED7AJJfA+ICASAKCwBZvSQrb2omhAgKBrkPoCGEcNQICEekk30pkQzm\
kD6f+YN4EoAbeBAUiYcVnzGEAgFYDA0AEbjJftRNDXCx+AA9sp37UTQgQFA1yH0BDACyMoHy//J0\
AGBAQj0Cm+hMYAIBIA4PABmtznaiaEAga5Drhf/AABmvHfaiaEAQa5DrhY/AAG7SB/oA1NQi+QAF\
yMoHFcv/ydB3dIAYyMsFywIizxZQBfoCFMtrEszMyXP7AMhAFIEBCPRR8qcCAHCBAQjXGPoA0z/I\
VCBHgQEI9FHyp4IQbm90ZXB0gBjIywXLAlAGzxZQBPoCFMtqEssfyz/Jc/sAAgBsgQEI1xj6ANM/\
MFIkgQEI9Fnyp4IQZHN0cnB0gBjIywXLAlAFzxZQA/oCE8tqyx8Syz/Jc/sAAAr0AMntVA==";

/// Default wallet v4r2 sub-wallet ID
const WALLET_V4R2_ID: u32 = 698983191; // 0x29A9A317

pub struct TonChain;

impl Chain for TonChain {
    fn derive_address(&self, seed: &[u8; 64]) -> Result<String, String> {
        derive_ton_address(seed)
    }

    fn name(&self) -> &str {
        "TON"
    }

    fn ticker(&self) -> &str {
        "TON"
    }

    fn chain_id(&self) -> ChainId {
        ChainId::Ton
    }
}

/// Derive TON address from seed
/// Path: m/44'/607'/0' (SLIP-10 Ed25519)
/// Address: wallet v4r2 bounceable base64url
pub fn derive_ton_address(seed: &[u8; 64]) -> Result<String, String> {
    let path = DerivationPath::bip44(607);
    let (private_key, _) = bip32_utils::derive_ed25519_key_from_seed(seed, &path)?;

    let signing_key = SigningKey::from_bytes(&private_key);
    let public_key = signing_key.verifying_key();
    let pubkey_bytes = public_key.as_bytes();

    build_wallet_v4r2_address(pubkey_bytes)
}

/// Build TON wallet v4r2 address from public key using proper state_init hash
fn build_wallet_v4r2_address(pubkey: &[u8; 32]) -> Result<String, String> {
    // 1. Parse wallet v4r2 code BOC to get code cell hash and depth
    let boc_bytes = base64_decode(WALLET_V4R2_CODE_BOC_B64)?;
    let (cells, roots) = parse_boc(&boc_bytes)?;
    if roots.is_empty() {
        return Err("BOC has no root cells".into());
    }
    let hashed = compute_cell_hashes(&cells);
    let root_idx = roots[0];
    let code = &hashed[root_idx];

    // 2. Compute data cell hash
    let data = build_v4r2_data_cell_hash(pubkey);

    // 3. Compute state_init hash
    let state_init_hash = compute_state_init_hash(
        &code.hash, code.depth,
        &data.hash, data.depth,
    );

    // 4. Build user-friendly address: tag(1) + workchain(1) + hash(32) + crc16(2) = 36 bytes
    let tag: u8 = 0x11; // bounceable
    let workchain: u8 = 0x00; // basechain

    let mut addr_data = Vec::with_capacity(36);
    addr_data.push(tag);
    addr_data.push(workchain);
    addr_data.extend_from_slice(&state_init_hash);

    let crc = crc16_xmodem(&addr_data);
    addr_data.push((crc >> 8) as u8);
    addr_data.push((crc & 0xff) as u8);

    Ok(base64url_encode(&addr_data))
}

/// Decode a TON user-friendly address (base64url) to raw 34 bytes (tag + workchain + hash)
/// Verifies CRC16 checksum
pub fn decode_ton_friendly_address(addr: &str) -> Result<Vec<u8>, String> {
    let bytes = base64url_decode(addr)?;
    if bytes.len() != 36 {
        return Err(format!(
            "Invalid TON address length: {} (expected 36 bytes from 48 base64 chars)",
            bytes.len()
        ));
    }

    // Verify CRC16
    let crc_data = &bytes[..34];
    let expected_crc = ((bytes[34] as u16) << 8) | (bytes[35] as u16);
    let actual_crc = crc16_xmodem(crc_data);
    if expected_crc != actual_crc {
        return Err("Invalid TON address checksum".into());
    }

    Ok(bytes[..34].to_vec())
}

// ======== TON Cell hashing ========

struct CellHashInfo {
    hash: [u8; 32],
    depth: u16,
}

struct BocCell {
    d1: u8,
    d2: u8,
    data: Vec<u8>,
    refs: Vec<usize>,
}

/// Build data cell hash for wallet v4r2 initial state
/// Data: seqno(32bit) + wallet_id(32bit) + pubkey(256bit) + empty_plugins(1bit) = 321 bits
fn build_v4r2_data_cell_hash(pubkey: &[u8; 32]) -> CellHashInfo {
    let mut data = Vec::with_capacity(41);
    data.extend_from_slice(&0u32.to_be_bytes());           // seqno = 0
    data.extend_from_slice(&WALLET_V4R2_ID.to_be_bytes()); // wallet_id v4r2
    data.extend_from_slice(pubkey);                         // public key (256 bits)
    // empty plugins dict (0 bit) + padding marker (1 bit) + zeros = 0b01000000
    data.push(0x40);

    // Cell representation for hashing: d1(1) + d2(1) + data(41)
    // d1 = 0x00 (0 refs, ordinary cell)
    // d2 = floor(321/8) + ceil(321/8) = 40 + 41 = 81 = 0x51
    let mut repr = Vec::with_capacity(43);
    repr.push(0x00); // d1
    repr.push(0x51); // d2
    repr.extend_from_slice(&data);

    CellHashInfo { hash: sha256_hash(&repr), depth: 0 }
}

/// Compute state_init cell hash from code and data cell hashes
/// State init (TL-B): split_depth(0) special(0) code(1) data(1) library(0) = 5 bits
fn compute_state_init_hash(
    code_hash: &[u8; 32], code_depth: u16,
    data_hash: &[u8; 32], data_depth: u16,
) -> [u8; 32] {
    // d1 = 0x02 (2 refs: code + data, ordinary cell)
    // d2 = floor(5/8) + ceil(5/8) = 0 + 1 = 1 = 0x01
    // data = 0b00110100 = 0x34 (bits: 0 0 1 1 0 + padding 1 0 0)
    let mut repr = Vec::with_capacity(71);
    repr.push(0x02); // d1
    repr.push(0x01); // d2
    repr.push(0x34); // data
    // Depths (2 bytes each, big-endian): code then data
    repr.push((code_depth >> 8) as u8);
    repr.push((code_depth & 0xff) as u8);
    repr.push((data_depth >> 8) as u8);
    repr.push((data_depth & 0xff) as u8);
    // Hashes: code then data
    repr.extend_from_slice(code_hash);
    repr.extend_from_slice(data_hash);

    sha256_hash(&repr)
}

fn sha256_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut h = [0u8; 32];
    h.copy_from_slice(&result);
    h
}

// ======== Minimal BOC parser ========

fn read_be(data: &[u8], n: usize) -> usize {
    let mut result: usize = 0;
    for &byte in data.iter().take(n) {
        result = (result << 8) | byte as usize;
    }
    result
}

/// Parse a BOC (Bag of Cells) and return cells + root indices
fn parse_boc(boc: &[u8]) -> Result<(Vec<BocCell>, Vec<usize>), String> {
    if boc.len() < 6 {
        return Err("BOC too short".into());
    }
    if boc[0..4] != [0xb5, 0xee, 0x9c, 0x72] {
        return Err("Invalid BOC magic".into());
    }

    let flags_byte = boc[4];
    let has_idx = (flags_byte >> 7) & 1 == 1;
    let ref_sz = (flags_byte & 0x07) as usize;
    let off_sz = boc[5] as usize;

    let mut pos = 6;
    let cells_count = read_be(&boc[pos..], ref_sz); pos += ref_sz;
    let roots_count = read_be(&boc[pos..], ref_sz); pos += ref_sz;
    let _absent     = read_be(&boc[pos..], ref_sz); pos += ref_sz;
    let _total_size = read_be(&boc[pos..], off_sz); pos += off_sz;

    let mut root_indices = Vec::with_capacity(roots_count);
    for _ in 0..roots_count {
        root_indices.push(read_be(&boc[pos..], ref_sz));
        pos += ref_sz;
    }

    // Skip index table if present
    if has_idx {
        pos += cells_count * off_sz;
    }

    let mut cells = Vec::with_capacity(cells_count);
    for _ in 0..cells_count {
        if pos + 2 > boc.len() {
            return Err("BOC truncated at cell descriptor".into());
        }
        let d1 = boc[pos];
        let d2 = boc[pos + 1];
        pos += 2;

        let data_len = ((d2 as usize) + 1) / 2;
        let refs_count = (d1 & 0x07) as usize;

        if pos + data_len + refs_count * ref_sz > boc.len() {
            return Err("BOC truncated at cell data/refs".into());
        }

        let data = boc[pos..pos + data_len].to_vec();
        pos += data_len;

        let mut refs = Vec::with_capacity(refs_count);
        for _ in 0..refs_count {
            refs.push(read_be(&boc[pos..], ref_sz));
            pos += ref_sz;
        }

        cells.push(BocCell { d1, d2, data, refs });
    }

    Ok((cells, root_indices))
}

/// Compute hashes for all cells in a BOC (bottom-up: leaves first)
/// Cells in BOC are stored with children having higher indices than parents.
fn compute_cell_hashes(cells: &[BocCell]) -> Vec<CellHashInfo> {
    let n = cells.len();
    let mut result: Vec<Option<CellHashInfo>> = (0..n).map(|_| None).collect();

    for i in (0..n).rev() {
        let cell = &cells[i];
        let data_len = ((cell.d2 as usize) + 1) / 2;

        // Depth: 0 for leaves, 1 + max(children) for internal nodes
        let mut depth: u16 = 0;
        for r in &cell.refs {
            if let Some(ref info) = result[*r] {
                depth = depth.max(info.depth + 1);
            }
        }

        // Cell representation: d1 + d2 + data + depth(refs)... + hash(refs)...
        let mut repr = Vec::new();
        repr.push(cell.d1);
        repr.push(cell.d2);
        repr.extend_from_slice(&cell.data[..data_len.min(cell.data.len())]);

        // Append child depths (2 bytes each, big-endian)
        for r in &cell.refs {
            if let Some(ref info) = result[*r] {
                repr.push((info.depth >> 8) as u8);
                repr.push((info.depth & 0xff) as u8);
            }
        }
        // Append child hashes (32 bytes each)
        for r in &cell.refs {
            if let Some(ref info) = result[*r] {
                repr.extend_from_slice(&info.hash);
            }
        }

        result[i] = Some(CellHashInfo { hash: sha256_hash(&repr), depth });
    }

    result.into_iter()
        .map(|o| o.unwrap_or(CellHashInfo { hash: [0; 32], depth: 0 }))
        .collect()
}

// ======== Base64 encoding/decoding ========

/// CRC16-XMODEM used by TON addresses
fn crc16_xmodem(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for byte in data {
        crc ^= (*byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// Base64url encoding without padding
fn base64url_encode(data: &[u8]) -> String {
    let encoded = base64_encode(data);
    encoded.replace('+', "-").replace('/', "_").trim_end_matches('=').to_string()
}

/// Simple base64 encoding
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    let chunks = data.chunks(3);

    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };

        let n = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((n >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARS[((n >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

/// Standard base64 decoding
fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let input = input.trim_end_matches('=');
    let mut output = Vec::new();
    let mut buf: u32 = 0;
    let mut bits: u32 = 0;

    for c in input.chars() {
        if c.is_whitespace() {
            continue; // skip whitespace in multi-line constants
        }
        let val = base64_char_value(c)?;
        buf = (buf << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push((buf >> bits) as u8);
            buf &= (1 << bits) - 1;
        }
    }

    Ok(output)
}

/// Base64url decoding (converts base64url alphabet to standard base64 first)
fn base64url_decode(input: &str) -> Result<Vec<u8>, String> {
    let standard: String = input.chars().map(|c| match c {
        '-' => '+',
        '_' => '/',
        _ => c,
    }).collect();
    base64_decode(&standard)
}

fn base64_char_value(c: char) -> Result<u8, String> {
    match c {
        'A'..='Z' => Ok(c as u8 - b'A'),
        'a'..='z' => Ok(c as u8 - b'a' + 26),
        '0'..='9' => Ok(c as u8 - b'0' + 52),
        '+' => Ok(62),
        '/' => Ok(63),
        _ => Err(format!("Invalid base64 character: {}", c)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bip39_utils;

    #[test]
    fn test_derive_ton_address() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_ton_address(&seed).unwrap();
        assert!(!address.is_empty());
        // v4r2 bounceable workchain-0 address starts with 'E' (tag 0x11)
        assert!(address.starts_with('E'), "Address should start with E, got: {}", address);
    }

    #[test]
    fn test_ton_deterministic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let addr1 = derive_ton_address(&seed).unwrap();
        let addr2 = derive_ton_address(&seed).unwrap();
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_address_decode_roundtrip() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_ton_address(&seed).unwrap();
        // Decode should succeed and return 34 bytes (tag + workchain + hash)
        let raw = decode_ton_friendly_address(&address).unwrap();
        assert_eq!(raw.len(), 34);
        assert_eq!(raw[0], 0x11); // bounceable tag
        assert_eq!(raw[1], 0x00); // workchain 0
    }

    #[test]
    fn test_crc16() {
        let data = [0x11, 0x00, 0x01, 0x02, 0x03];
        let crc = crc16_xmodem(&data);
        assert_ne!(crc, 0);
    }

    #[test]
    fn test_boc_parse() {
        let boc_bytes = base64_decode(WALLET_V4R2_CODE_BOC_B64).unwrap();
        let (cells, roots) = parse_boc(&boc_bytes).unwrap();
        assert!(!cells.is_empty(), "BOC should contain cells");
        assert_eq!(roots.len(), 1, "BOC should have exactly 1 root");
        assert_eq!(roots[0], 0, "Root should be cell 0");
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = vec![0x11, 0x00, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let encoded = base64_encode(&data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_base64url_roundtrip() {
        let data = vec![0x11, 0x00, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let encoded = base64url_encode(&data);
        let decoded = base64url_decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_invalid_address_checksum() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_ton_address(&seed).unwrap();
        // Corrupt one character to break CRC
        let mut chars: Vec<char> = address.chars().collect();
        if chars.len() > 5 {
            chars[5] = if chars[5] == 'A' { 'B' } else { 'A' };
        }
        let corrupted: String = chars.into_iter().collect();
        assert!(decode_ton_friendly_address(&corrupted).is_err());
    }
}
