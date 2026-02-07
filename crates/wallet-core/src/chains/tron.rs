// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// chains/tron: TRON address derivation (secp256k1 + Keccak256 + base58check)
//
// BIP44 path: m/44'/195'/0'/0/0
// Address: uncompressed pubkey → Keccak256 → last 20 bytes → 0x41 prefix → base58check → 'T...'
// Testnet (Nile): byte 0xa0 → addresses start with '27'

use crate::bip32_utils::{self, DerivationPath};
use k256::ecdsa::SigningKey;
use tiny_keccak::{Hasher, Keccak};
use sha2::{Digest, Sha256};

use super::{Chain, ChainId};

pub struct TronChain;

impl Chain for TronChain {
    fn derive_address(&self, seed: &[u8; 64]) -> Result<String, String> {
        derive_tron_address(seed)
    }

    fn name(&self) -> &str {
        "TRON"
    }

    fn ticker(&self) -> &str {
        "TRX"
    }

    fn chain_id(&self) -> ChainId {
        ChainId::Tron
    }
}

/// BIP44 derivation path for TRON: m/44'/195'/0'/0/0
fn bip44_path() -> DerivationPath {
    DerivationPath {
        purpose: 44,
        coin_type: 195,
        account: 0,
        change: 0,
        address_index: 0,
    }
}

/// Derive TRON address from seed (mainnet)
pub fn derive_tron_address(seed: &[u8; 64]) -> Result<String, String> {
    derive_tron_address_for_network(seed, false)
}

/// Derive TRON address with network selection
pub fn derive_tron_address_for_network(seed: &[u8; 64], testnet: bool) -> Result<String, String> {
    let path = bip44_path();
    let (private_key, _) = bip32_utils::derive_key_from_seed(seed, &path)?;

    let signing_key = SigningKey::from_bytes((&private_key).into())
        .map_err(|e| format!("Invalid key: {}", e))?;
    let verifying_key = signing_key.verifying_key();

    // Uncompressed public key (65 bytes: 0x04 + x + y), skip the 0x04 prefix
    let encoded = verifying_key.to_encoded_point(false);
    let pubkey_bytes = &encoded.as_bytes()[1..]; // 64 bytes

    // Keccak256 hash of the public key
    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(pubkey_bytes);
    hasher.finalize(&mut hash);

    // Take last 20 bytes as address
    let addr_bytes = &hash[12..32];

    // Prepend version byte: 0x41 mainnet, 0xa0 testnet
    let version = if testnet { 0xa0u8 } else { 0x41u8 };
    let mut payload = Vec::with_capacity(21);
    payload.push(version);
    payload.extend_from_slice(addr_bytes);

    Ok(base58check_encode(&payload))
}

/// Get private key for signing
pub fn get_private_key(seed: &[u8; 64]) -> Result<[u8; 32], String> {
    let path = bip44_path();
    let (private_key, _) = bip32_utils::derive_key_from_seed(seed, &path)?;
    Ok(private_key)
}

/// Convert base58check TRON address to hex string (41... format)
pub fn address_to_hex(address: &str) -> Result<String, String> {
    let decoded = base58check_decode(address)?;
    Ok(hex::encode(&decoded))
}

/// Decode TRON address to 20 pure bytes (without version byte)
pub fn decode_address(address: &str) -> Result<[u8; 20], String> {
    let decoded = base58check_decode(address)?;
    if decoded.len() != 21 {
        return Err("Indirizzo TRON non valido (lunghezza errata)".into());
    }
    let mut result = [0u8; 20];
    result.copy_from_slice(&decoded[1..21]);
    Ok(result)
}

/// Base58check encode using standard Bitcoin alphabet
fn base58check_encode(payload: &[u8]) -> String {
    let hash1 = Sha256::digest(payload);
    let hash2 = Sha256::digest(hash1);
    let checksum = &hash2[..4];

    let mut data = Vec::with_capacity(payload.len() + 4);
    data.extend_from_slice(payload);
    data.extend_from_slice(checksum);

    bs58::encode(data)
        .with_alphabet(bs58::Alphabet::BITCOIN)
        .into_string()
}

/// Base58check decode using standard Bitcoin alphabet
fn base58check_decode(encoded: &str) -> Result<Vec<u8>, String> {
    let data = bs58::decode(encoded)
        .with_alphabet(bs58::Alphabet::BITCOIN)
        .into_vec()
        .map_err(|e| format!("Base58 decode error: {}", e))?;

    if data.len() < 5 {
        return Err("Indirizzo troppo corto".into());
    }

    let payload = &data[..data.len() - 4];
    let checksum = &data[data.len() - 4..];

    let hash1 = Sha256::digest(payload);
    let hash2 = Sha256::digest(hash1);

    if checksum != &hash2[..4] {
        return Err("Checksum non valido".into());
    }

    Ok(payload.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bip39_utils;

    #[test]
    fn test_derive_tron_address() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_tron_address(&seed).unwrap();

        assert!(address.starts_with('T'), "Got: {}", address);
        assert!(address.len() >= 25 && address.len() <= 35, "Length: {}", address.len());
    }

    #[test]
    fn test_tron_deterministic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let addr1 = derive_tron_address(&seed).unwrap();
        let addr2 = derive_tron_address(&seed).unwrap();
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_tron_different_from_evm() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let tron = derive_tron_address(&seed).unwrap();
        let evm = crate::chains::evm::derive_evm_address(&seed).unwrap();
        // Different because coin_type 195 vs 60, plus different encoding
        assert_ne!(tron, evm);
        assert!(tron.starts_with('T'));
        assert!(evm.starts_with("0x"));
    }

    #[test]
    fn test_address_to_hex_roundtrip() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_tron_address(&seed).unwrap();
        let hex_addr = address_to_hex(&address).unwrap();
        // Hex should start with 41 (mainnet version byte)
        assert!(hex_addr.starts_with("41"), "Got: {}", hex_addr);
        assert_eq!(hex_addr.len(), 42); // 21 bytes = 42 hex chars
    }

    #[test]
    fn test_decode_address() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_tron_address(&seed).unwrap();
        let decoded = decode_address(&address).unwrap();
        assert_eq!(decoded.len(), 20);
    }

    #[test]
    fn test_testnet_address() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_tron_address_for_network(&seed, true).unwrap();
        let hex_addr = address_to_hex(&address).unwrap();
        assert!(hex_addr.starts_with("a0"), "Got: {}", hex_addr);
    }

    #[test]
    fn test_get_private_key() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let key = get_private_key(&seed).unwrap();
        assert_ne!(key, [0u8; 32]);
    }
}
