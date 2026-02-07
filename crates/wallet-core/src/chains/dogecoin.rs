// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// chains/dogecoin: P2PKH (legacy) address derivation
//
// BIP44 path: m/44'/3'/0'/0/0
// Address = base58check(version_byte + RIPEMD160(SHA256(compressed_pubkey)))
// Mainnet version byte: 0x1E (30) -> addresses starting with 'D'
// Testnet version byte: 0x71 (113) -> addresses starting with 'n'

use crate::bip32_utils::{self, DerivationPath};
use k256::ecdsa::SigningKey;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

use super::{Chain, ChainId};

pub struct DogecoinChain;

impl Chain for DogecoinChain {
    fn derive_address(&self, seed: &[u8; 64]) -> Result<String, String> {
        derive_dogecoin_address(seed)
    }

    fn name(&self) -> &str {
        "Dogecoin"
    }

    fn ticker(&self) -> &str {
        "DOGE"
    }

    fn chain_id(&self) -> ChainId {
        ChainId::Dogecoin
    }
}

/// BIP44 derivation path for Dogecoin: m/44'/3'/0'/0/0
fn bip44_path() -> DerivationPath {
    DerivationPath {
        purpose: 44,
        coin_type: 3,
        account: 0,
        change: 0,
        address_index: 0,
    }
}

/// Derive Dogecoin P2PKH address from seed (mainnet)
pub fn derive_dogecoin_address(seed: &[u8; 64]) -> Result<String, String> {
    derive_dogecoin_address_for_network(seed, false)
}

/// Derive Dogecoin P2PKH address with network selection
pub fn derive_dogecoin_address_for_network(seed: &[u8; 64], testnet: bool) -> Result<String, String> {
    let path = bip44_path();
    let (private_key, _) = bip32_utils::derive_key_from_seed(seed, &path)?;

    let signing_key = SigningKey::from_bytes((&private_key).into())
        .map_err(|e| format!("Invalid key: {}", e))?;
    let verifying_key = signing_key.verifying_key();

    // Compressed public key (33 bytes)
    let pubkey_compressed = verifying_key.to_encoded_point(true);
    let pubkey_bytes = pubkey_compressed.as_bytes();
    let mut pubkey = [0u8; 33];
    pubkey.copy_from_slice(pubkey_bytes);

    let hash = hash160_pubkey(&pubkey);

    // Version byte: 0x1E mainnet, 0x71 testnet
    let version = if testnet { 0x71u8 } else { 0x1Eu8 };
    let mut payload = Vec::with_capacity(21);
    payload.push(version);
    payload.extend_from_slice(&hash);

    Ok(base58check_encode(&payload))
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

/// RIPEMD160(SHA256(pubkey))
pub fn hash160_pubkey(pubkey: &[u8; 33]) -> [u8; 20] {
    let sha = Sha256::digest(pubkey);
    let hash = Ripemd160::digest(sha);
    let mut result = [0u8; 20];
    result.copy_from_slice(&hash);
    result
}

/// Decode a P2PKH Dogecoin address to its 20-byte pubkey hash and version byte
pub fn decode_p2pkh_address(address: &str) -> Result<([u8; 20], u8), String> {
    let decoded = base58check_decode(address)?;
    if decoded.len() != 21 {
        return Err("Indirizzo Dogecoin non valido (lunghezza errata)".into());
    }
    let version = decoded[0];
    if version != 0x1E && version != 0x71 {
        return Err(format!("Version byte non valido: 0x{:02x}", version));
    }
    let mut hash = [0u8; 20];
    hash.copy_from_slice(&decoded[1..21]);
    Ok((hash, version))
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
    fn test_derive_dogecoin_address() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_dogecoin_address(&seed).unwrap();

        assert!(address.starts_with('D'), "Got: {}", address);
        assert!(address.len() >= 25 && address.len() <= 35, "Length: {}", address.len());
    }

    #[test]
    fn test_dogecoin_deterministic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let addr1 = derive_dogecoin_address(&seed).unwrap();
        let addr2 = derive_dogecoin_address(&seed).unwrap();
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_dogecoin_testnet() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_dogecoin_address_for_network(&seed, true).unwrap();
        assert!(address.starts_with('n'), "Got: {}", address);
    }

    #[test]
    fn test_dogecoin_different_from_bitcoin() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let doge = derive_dogecoin_address(&seed).unwrap();
        let btc = crate::chains::bitcoin::derive_bitcoin_address(&seed).unwrap();
        assert_ne!(doge, btc);
    }

    #[test]
    fn test_address_roundtrip() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_dogecoin_address(&seed).unwrap();
        let (hash, version) = decode_p2pkh_address(&address).unwrap();
        assert_eq!(version, 0x1E);
        let pubkey = get_public_key(&seed).unwrap();
        let expected_hash = hash160_pubkey(&pubkey);
        assert_eq!(hash, expected_hash);
    }

    #[test]
    fn test_get_keys() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let key = get_private_key(&seed).unwrap();
        assert_ne!(key, [0u8; 32]);
        let pubkey = get_public_key(&seed).unwrap();
        assert!(pubkey[0] == 0x02 || pubkey[0] == 0x03);
    }
}
