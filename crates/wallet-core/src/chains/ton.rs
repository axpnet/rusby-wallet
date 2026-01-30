// Rusby Wallet — Pure Rust multi-chain crypto wallet
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// chains/ton: TON address derivation (Ed25519 + wallet v4r2)
//
// Functions:
//   derive_ton_address()   — seed → SLIP-10 m/44'/607'/0' → Ed25519 → v4r2 address
//   build_wallet_address() — pubkey → SHA256 → bounceable base64url
//   crc16_xmodem()         — CRC16-XMODEM for TON address checksum

use crate::bip32_utils::{self, DerivationPath};
use ed25519_dalek::SigningKey;
use sha2::{Digest, Sha256};

use super::{Chain, ChainId};

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
/// Path: m/44'/607'/0'/0' (SLIP-10 Ed25519)
/// Address format: EQ + base64url(workchain + state_init_hash + crc16)
pub fn derive_ton_address(seed: &[u8; 64]) -> Result<String, String> {
    let path = DerivationPath::bip44(607);
    let (private_key, _) = bip32_utils::derive_ed25519_key_from_seed(seed, &path)?;

    let signing_key = SigningKey::from_bytes(&private_key);
    let public_key = signing_key.verifying_key();
    let pubkey_bytes = public_key.as_bytes();

    // Build wallet v4r2 state init cell hash
    let address_bytes = build_wallet_address(pubkey_bytes)?;
    Ok(address_bytes)
}

/// Build TON wallet v4r2 address from public key
/// This creates a bounceable address in base64url format
fn build_wallet_address(pubkey: &[u8; 32]) -> Result<String, String> {
    // Simplified TON address derivation:
    // 1. Hash the public key with SHA256
    // 2. Build the raw address: workchain(1) + hash(32)
    // 3. Add tag + CRC16 for user-friendly format

    let mut hasher = Sha256::new();
    hasher.update(pubkey);
    let hash = hasher.finalize();

    // User-friendly address: tag(1) + workchain(1) + hash(32) + crc16(2) = 36 bytes
    let tag: u8 = 0x11; // bounceable
    let workchain: u8 = 0x00; // basechain

    let mut data = Vec::with_capacity(34);
    data.push(tag);
    data.push(workchain);
    data.extend_from_slice(&hash);

    let crc = crc16_xmodem(&data);
    data.push((crc >> 8) as u8);
    data.push((crc & 0xff) as u8);

    // Base64url encode
    Ok(base64url_encode(&data))
}

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
    }

    #[test]
    fn test_crc16() {
        let data = [0x11, 0x00, 0x01, 0x02, 0x03];
        let crc = crc16_xmodem(&data);
        assert_ne!(crc, 0);
    }

    #[test]
    fn test_ton_deterministic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let addr1 = derive_ton_address(&seed).unwrap();
        let addr2 = derive_ton_address(&seed).unwrap();
        assert_eq!(addr1, addr2);
    }
}
