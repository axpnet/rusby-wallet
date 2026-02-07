// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// chains/ripple: XRP Ledger address derivation (secp256k1 + base58check Ripple)
//
// Functions:
//   derive_ripple_address() — seed → BIP44 m/44'/144'/0'/0/0 → secp256k1 → Hash160 → base58check (r...)
//   get_private_key()       — Extract private key for signing
//   get_public_key()        — Extract compressed public key

use crate::bip32_utils::{self, DerivationPath};
use k256::ecdsa::SigningKey;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

use super::{Chain, ChainId};

pub struct RippleChain;

impl Chain for RippleChain {
    fn derive_address(&self, seed: &[u8; 64]) -> Result<String, String> {
        derive_ripple_address(seed)
    }

    fn name(&self) -> &str {
        "XRP Ledger"
    }

    fn ticker(&self) -> &str {
        "XRP"
    }

    fn chain_id(&self) -> ChainId {
        ChainId::Ripple
    }
}

/// BIP44 derivation path for XRP: m/44'/144'/0'/0/0
fn bip44_path() -> DerivationPath {
    DerivationPath {
        purpose: 44,
        coin_type: 144,
        account: 0,
        change: 0,
        address_index: 0,
    }
}

/// Derive XRP address from seed
/// Path: m/44'/144'/0'/0/0 (BIP44)
/// Address = base58check_ripple(0x00 + RIPEMD160(SHA256(compressed_pubkey)))
pub fn derive_ripple_address(seed: &[u8; 64]) -> Result<String, String> {
    let path = bip44_path();
    let (private_key, _) = bip32_utils::derive_key_from_seed(seed, &path)?;

    let signing_key = SigningKey::from_bytes((&private_key).into())
        .map_err(|e| format!("Invalid key: {}", e))?;
    let verifying_key = signing_key.verifying_key();

    // Compressed public key (33 bytes)
    let pubkey_compressed = verifying_key.to_encoded_point(true);
    let pubkey_bytes = pubkey_compressed.as_bytes();

    pubkey_to_address(pubkey_bytes)
}

/// Convert a compressed public key to an XRP address
fn pubkey_to_address(pubkey_bytes: &[u8]) -> Result<String, String> {
    // Hash160: RIPEMD160(SHA256(pubkey))
    let sha256_hash = Sha256::digest(pubkey_bytes);
    let account_id = Ripemd160::digest(sha256_hash);

    // Base58check encode with version byte 0x00
    let mut payload = Vec::with_capacity(21);
    payload.push(0x00); // version byte for XRP mainnet
    payload.extend_from_slice(&account_id);

    Ok(base58check_encode_ripple(&payload))
}

/// Get private key for signing
pub fn get_private_key(seed: &[u8; 64]) -> Result<[u8; 32], String> {
    let path = bip44_path();
    let (private_key, _) = bip32_utils::derive_key_from_seed(seed, &path)?;
    Ok(private_key)
}

/// Get compressed public key (33 bytes)
pub fn get_public_key(seed: &[u8; 64]) -> Result<[u8; 33], String> {
    let path = bip44_path();
    let (private_key, _) = bip32_utils::derive_key_from_seed(seed, &path)?;

    let signing_key = SigningKey::from_bytes((&private_key).into())
        .map_err(|e| format!("Invalid key: {}", e))?;
    let verifying_key = signing_key.verifying_key();
    let encoded = verifying_key.to_encoded_point(true);
    let bytes = encoded.as_bytes();
    let mut result = [0u8; 33];
    result.copy_from_slice(bytes);
    Ok(result)
}

/// Get the 20-byte account ID (Hash160 of compressed pubkey)
pub fn get_account_id(seed: &[u8; 64]) -> Result<[u8; 20], String> {
    let pubkey = get_public_key(seed)?;
    let sha = Sha256::digest(&pubkey);
    let hash = Ripemd160::digest(sha);
    let mut result = [0u8; 20];
    result.copy_from_slice(&hash);
    Ok(result)
}

/// Decode an XRP address to its 20-byte account ID
pub fn decode_address(address: &str) -> Result<[u8; 20], String> {
    let decoded = base58check_decode_ripple(address)?;
    if decoded.len() != 21 || decoded[0] != 0x00 {
        return Err("Indirizzo XRP non valido".into());
    }
    let mut result = [0u8; 20];
    result.copy_from_slice(&decoded[1..21]);
    Ok(result)
}

/// Base58check encode using the Ripple alphabet
fn base58check_encode_ripple(payload: &[u8]) -> String {
    // Checksum = first 4 bytes of SHA256d(payload)
    let hash1 = Sha256::digest(payload);
    let hash2 = Sha256::digest(hash1);
    let checksum = &hash2[..4];

    let mut data = Vec::with_capacity(payload.len() + 4);
    data.extend_from_slice(payload);
    data.extend_from_slice(checksum);

    bs58::encode(data)
        .with_alphabet(bs58::Alphabet::RIPPLE)
        .into_string()
}

/// Base58check decode using the Ripple alphabet
fn base58check_decode_ripple(encoded: &str) -> Result<Vec<u8>, String> {
    let data = bs58::decode(encoded)
        .with_alphabet(bs58::Alphabet::RIPPLE)
        .into_vec()
        .map_err(|e| format!("Base58 decode error: {}", e))?;

    if data.len() < 5 {
        return Err("Indirizzo troppo corto".into());
    }

    let payload = &data[..data.len() - 4];
    let checksum = &data[data.len() - 4..];

    // Verify checksum
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
    fn test_derive_ripple_address() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_ripple_address(&seed).unwrap();

        // XRP addresses start with 'r'
        assert!(address.starts_with('r'), "Got: {}", address);
        // Typical XRP address length is 25-35 chars
        assert!(address.len() >= 25 && address.len() <= 35, "Length: {}", address.len());
    }

    #[test]
    fn test_ripple_deterministic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let addr1 = derive_ripple_address(&seed).unwrap();
        let addr2 = derive_ripple_address(&seed).unwrap();
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_ripple_different_from_bitcoin() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let xrp = derive_ripple_address(&seed).unwrap();
        let btc = crate::chains::bitcoin::derive_bitcoin_address(&seed).unwrap();
        assert_ne!(xrp, btc);
        assert!(xrp.starts_with('r'));
        assert!(btc.starts_with("bc1"));
    }

    #[test]
    fn test_address_roundtrip() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_ripple_address(&seed).unwrap();
        let account_id = decode_address(&address).unwrap();
        let expected_id = get_account_id(&seed).unwrap();
        assert_eq!(account_id, expected_id);
    }

    #[test]
    fn test_get_private_key() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let key = get_private_key(&seed).unwrap();
        assert_ne!(key, [0u8; 32]);
    }

    #[test]
    fn test_get_public_key() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let pubkey = get_public_key(&seed).unwrap();
        assert!(pubkey[0] == 0x02 || pubkey[0] == 0x03);
    }

    #[test]
    fn test_base58check_roundtrip() {
        let payload = vec![0x00, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67,
                           0x89, 0xAB, 0xCD, 0xEF, 0x01, 0x23, 0x45, 0x67,
                           0x89, 0xAB, 0xCD, 0xEF, 0x01];
        let encoded = base58check_encode_ripple(&payload);
        let decoded = base58check_decode_ripple(&encoded).unwrap();
        assert_eq!(payload, decoded);
    }
}
