// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// chains/stellar: Stellar address derivation (Ed25519 + StrKey base32)
//
// Functions:
//   derive_stellar_address() — seed → SLIP-10 m/44'/148'/0' → Ed25519 → StrKey (G...)
//   get_keypair()            — Extract Ed25519 keypair (private + public)

use crate::bip32_utils::{self, DerivationPath};
use ed25519_dalek::SigningKey;

use super::{Chain, ChainId};

pub struct StellarChain;

impl Chain for StellarChain {
    fn derive_address(&self, seed: &[u8; 64]) -> Result<String, String> {
        derive_stellar_address(seed)
    }

    fn name(&self) -> &str {
        "Stellar"
    }

    fn ticker(&self) -> &str {
        "XLM"
    }

    fn chain_id(&self) -> ChainId {
        ChainId::Stellar
    }
}

/// StrKey version byte for public key (ED25519): 6 << 3 = 48
const STRKEY_PUBLIC_VERSION: u8 = 6 << 3;

/// Derive Stellar address from seed
/// Path: m/44'/148'/0' (SLIP-10 Ed25519, all hardened)
/// Address = StrKey(version_byte=48 + pubkey_32_bytes + CRC16-XMODEM) → base32
pub fn derive_stellar_address(seed: &[u8; 64]) -> Result<String, String> {
    let path = DerivationPath::stellar();
    let (private_key, _) = bip32_utils::derive_ed25519_key_from_seed(seed, &path)?;

    let signing_key = SigningKey::from_bytes(&private_key);
    let public_key = signing_key.verifying_key();

    strkey_encode(STRKEY_PUBLIC_VERSION, public_key.as_bytes())
}

/// Get the Ed25519 keypair bytes (64 bytes: private + public)
pub fn get_keypair(seed: &[u8; 64]) -> Result<[u8; 64], String> {
    let path = DerivationPath::stellar();
    let (private_key, _) = bip32_utils::derive_ed25519_key_from_seed(seed, &path)?;

    let signing_key = SigningKey::from_bytes(&private_key);
    let public_key = signing_key.verifying_key();

    let mut keypair = [0u8; 64];
    keypair[..32].copy_from_slice(&private_key);
    keypair[32..].copy_from_slice(public_key.as_bytes());
    Ok(keypair)
}

/// Get the raw Ed25519 public key (32 bytes)
pub fn get_public_key(seed: &[u8; 64]) -> Result<[u8; 32], String> {
    let path = DerivationPath::stellar();
    let (private_key, _) = bip32_utils::derive_ed25519_key_from_seed(seed, &path)?;
    let signing_key = SigningKey::from_bytes(&private_key);
    let public_key = signing_key.verifying_key();
    let mut result = [0u8; 32];
    result.copy_from_slice(public_key.as_bytes());
    Ok(result)
}

/// Encode a Stellar StrKey address
/// Format: base32(version_byte + payload + crc16_xmodem(version_byte + payload))
pub fn strkey_encode(version: u8, payload: &[u8]) -> Result<String, String> {
    let mut data = Vec::with_capacity(1 + payload.len());
    data.push(version);
    data.extend_from_slice(payload);

    let crc = crc16_xmodem(&data);
    data.push((crc & 0xFF) as u8);       // CRC low byte first (little-endian)
    data.push(((crc >> 8) & 0xFF) as u8); // CRC high byte

    Ok(base32_encode(&data))
}

/// Decode a Stellar StrKey address, returning (version_byte, payload)
pub fn strkey_decode(encoded: &str) -> Result<(u8, Vec<u8>), String> {
    let data = base32_decode(encoded)?;
    if data.len() < 3 {
        return Err("StrKey troppo corto".into());
    }

    let payload_end = data.len() - 2;
    let version = data[0];
    let payload = &data[1..payload_end];
    let crc_bytes = &data[payload_end..];

    let expected_crc = crc16_xmodem(&data[..payload_end]);
    let actual_crc = (crc_bytes[0] as u16) | ((crc_bytes[1] as u16) << 8);

    if expected_crc != actual_crc {
        return Err("StrKey CRC non valido".into());
    }

    Ok((version, payload.to_vec()))
}

/// CRC16-XMODEM (polynomial 0x1021)
fn crc16_xmodem(data: &[u8]) -> u16 {
    let mut crc: u16 = 0x0000;
    for &byte in data {
        crc ^= (byte as u16) << 8;
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

/// Base32 encoding (RFC 4648, no padding)
fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = String::new();
    let mut bits = 0u32;
    let mut n_bits = 0u32;

    for &byte in data {
        bits = (bits << 8) | byte as u32;
        n_bits += 8;
        while n_bits >= 5 {
            n_bits -= 5;
            result.push(ALPHABET[((bits >> n_bits) & 0x1F) as usize] as char);
        }
    }
    if n_bits > 0 {
        result.push(ALPHABET[((bits << (5 - n_bits)) & 0x1F) as usize] as char);
    }
    result
}

/// Base32 decoding (RFC 4648, no padding)
fn base32_decode(encoded: &str) -> Result<Vec<u8>, String> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = Vec::new();
    let mut bits = 0u32;
    let mut n_bits = 0u32;

    for ch in encoded.chars() {
        let val = ALPHABET.iter().position(|&c| c == ch as u8)
            .ok_or_else(|| format!("Carattere base32 non valido: {}", ch))?;
        bits = (bits << 5) | val as u32;
        n_bits += 5;
        if n_bits >= 8 {
            n_bits -= 8;
            result.push(((bits >> n_bits) & 0xFF) as u8);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bip39_utils;

    #[test]
    fn test_derive_stellar_address() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_stellar_address(&seed).unwrap();

        // Stellar addresses start with G and are 56 chars
        assert!(address.starts_with('G'), "Got: {}", address);
        assert_eq!(address.len(), 56, "Length: {}", address.len());
    }

    #[test]
    fn test_stellar_deterministic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let addr1 = derive_stellar_address(&seed).unwrap();
        let addr2 = derive_stellar_address(&seed).unwrap();
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_stellar_different_from_solana() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let xlm = derive_stellar_address(&seed).unwrap();
        let sol = crate::chains::solana::derive_solana_address(&seed).unwrap();
        // Different coin types mean different keys
        assert_ne!(xlm, sol);
    }

    #[test]
    fn test_strkey_roundtrip() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_stellar_address(&seed).unwrap();

        let (version, payload) = strkey_decode(&address).unwrap();
        assert_eq!(version, STRKEY_PUBLIC_VERSION);
        assert_eq!(payload.len(), 32);

        let re_encoded = strkey_encode(version, &payload).unwrap();
        assert_eq!(address, re_encoded);
    }

    #[test]
    fn test_crc16_xmodem() {
        // Known test vector: CRC16-XMODEM of empty data = 0x0000
        assert_eq!(crc16_xmodem(&[]), 0x0000);
        // CRC16-XMODEM of "123456789" = 0x31C3
        assert_eq!(crc16_xmodem(b"123456789"), 0x31C3);
    }

    #[test]
    fn test_get_keypair() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let keypair = get_keypair(&seed).unwrap();
        // Private key should not be all zeros
        assert_ne!(&keypair[..32], &[0u8; 32]);
        // Public key should not be all zeros
        assert_ne!(&keypair[32..], &[0u8; 32]);
    }
}
