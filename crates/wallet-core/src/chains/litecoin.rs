// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// chains/litecoin: P2WPKH (Native SegWit) address derivation for Litecoin
//
// Functions:
//   derive_litecoin_address() — seed → BIP84 m/84'/2'/0'/0/0 → secp256k1 → RIPEMD160(SHA256) → bech32 (ltc1q...)
//   get_private_key()         — Extract private key for signing
//   get_public_key()          — Extract compressed public key

use crate::bip32_utils::{self, DerivationPath};
use k256::ecdsa::SigningKey;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

use super::{Chain, ChainId};

pub struct LitecoinChain;

impl Chain for LitecoinChain {
    fn derive_address(&self, seed: &[u8; 64]) -> Result<String, String> {
        derive_litecoin_address(seed)
    }

    fn name(&self) -> &str {
        "Litecoin"
    }

    fn ticker(&self) -> &str {
        "LTC"
    }

    fn chain_id(&self) -> ChainId {
        ChainId::Litecoin
    }
}

/// BIP84 derivation path for Litecoin P2WPKH: m/84'/2'/0'/0/0
fn bip84_path() -> DerivationPath {
    DerivationPath {
        purpose: 84,
        coin_type: 2,
        account: 0,
        change: 0,
        address_index: 0,
    }
}

/// Derive native SegWit (P2WPKH) Litecoin address from seed (mainnet)
/// Path: m/84'/2'/0'/0/0 (BIP84)
/// Address = bech32(ltc, 0, RIPEMD160(SHA256(compressed_pubkey)))
pub fn derive_litecoin_address(seed: &[u8; 64]) -> Result<String, String> {
    derive_litecoin_address_for_network(seed, false)
}

/// Derive native SegWit (P2WPKH) Litecoin address with network selection
/// Mainnet: prefix "ltc" → ltc1q...
/// Testnet: prefix "tltc" → tltc1q...
pub fn derive_litecoin_address_for_network(seed: &[u8; 64], testnet: bool) -> Result<String, String> {
    let path = bip84_path();
    let (private_key, _) = bip32_utils::derive_key_from_seed(seed, &path)?;

    let signing_key = SigningKey::from_bytes((&private_key).into())
        .map_err(|e| format!("Invalid key: {}", e))?;
    let verifying_key = signing_key.verifying_key();

    // Compressed public key (33 bytes)
    let pubkey_compressed = verifying_key.to_encoded_point(true);
    let pubkey_bytes = pubkey_compressed.as_bytes();

    // Hash160: RIPEMD160(SHA256(pubkey))
    let sha256_hash = Sha256::digest(pubkey_bytes);
    let hash160 = Ripemd160::digest(sha256_hash);

    // Bech32 encode with witness version 0
    let hrp = if testnet { "tltc" } else { "ltc" };
    bech32_segwit_encode(hrp, 0, &hash160)
}

/// Get private key for signing
pub fn get_private_key(seed: &[u8; 64]) -> Result<[u8; 32], String> {
    let path = bip84_path();
    let (private_key, _) = bip32_utils::derive_key_from_seed(seed, &path)?;
    Ok(private_key)
}

/// Get compressed public key
pub fn get_public_key(seed: &[u8; 64]) -> Result<[u8; 33], String> {
    let path = bip84_path();
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

/// Compute Hash160 (RIPEMD160(SHA256(data))) of a compressed public key
pub fn hash160_pubkey(pubkey: &[u8; 33]) -> [u8; 20] {
    let sha = Sha256::digest(pubkey);
    let hash = Ripemd160::digest(sha);
    let mut result = [0u8; 20];
    result.copy_from_slice(&hash);
    result
}

/// Decode a bech32 P2WPKH Litecoin address (ltc1q...) to its 20-byte witness program
pub fn decode_bech32_address(address: &str) -> Result<[u8; 20], String> {
    let (_hrp, _version, data) = bech32::segwit::decode(address)
        .map_err(|e| format!("Indirizzo Litecoin non valido: {}", e))?;

    if data.len() != 20 {
        return Err("Indirizzo P2WPKH non valido (attesi 20 byte)".into());
    }

    let mut result = [0u8; 20];
    result.copy_from_slice(&data);
    Ok(result)
}

/// Bech32 SegWit encoding (witness version 0 + program)
fn bech32_segwit_encode(hrp: &str, _witness_version: u8, program: &[u8]) -> Result<String, String> {
    use bech32::Hrp;

    let hrp = Hrp::parse(hrp).map_err(|e| format!("Invalid HRP: {}", e))?;
    bech32::segwit::encode_v0(hrp, program)
        .map_err(|e| format!("Bech32 segwit encode error: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bip39_utils;

    #[test]
    fn test_derive_litecoin_address() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_litecoin_address(&seed).unwrap();

        // P2WPKH Litecoin addresses start with ltc1q
        assert!(address.starts_with("ltc1q"), "Got: {}", address);
        // Bech32 P2WPKH addresses are typically 43-63 chars
        assert!(address.len() >= 42 && address.len() <= 63, "Length: {}", address.len());
    }

    #[test]
    fn test_litecoin_deterministic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let addr1 = derive_litecoin_address(&seed).unwrap();
        let addr2 = derive_litecoin_address(&seed).unwrap();
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_litecoin_testnet_address() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_litecoin_address_for_network(&seed, true).unwrap();
        assert!(address.starts_with("tltc1q"), "Got: {}", address);
    }

    #[test]
    fn test_litecoin_different_from_bitcoin() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let ltc = derive_litecoin_address(&seed).unwrap();
        let btc = crate::chains::bitcoin::derive_bitcoin_address(&seed).unwrap();
        assert_ne!(ltc, btc);
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
        // Compressed pubkey starts with 0x02 or 0x03
        assert!(pubkey[0] == 0x02 || pubkey[0] == 0x03);
    }
}
