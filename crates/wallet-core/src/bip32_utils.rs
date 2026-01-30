// Rusby Wallet — Pure Rust multi-chain crypto wallet
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// bip32_utils: BIP32/BIP44/SLIP-10 hierarchical deterministic key derivation
//
// Types:
//   DerivationPath                — BIP44 path components
// Functions:
//   derive_key_from_seed()        — secp256k1 key derivation (EVM, Bitcoin, Cosmos)
//   derive_ed25519_key_from_seed() — Ed25519 SLIP-10 derivation (Solana, TON)

use hmac::Hmac;
use sha2::Sha512;

type HmacSha512 = Hmac<Sha512>;

/// BIP44 derivation path components
/// m / purpose' / coin_type' / account' / change / address_index
#[derive(Debug, Clone)]
pub struct DerivationPath {
    pub purpose: u32,
    pub coin_type: u32,
    pub account: u32,
    pub change: u32,
    pub address_index: u32,
}

impl DerivationPath {
    /// Standard BIP44 path: m/44'/coin_type'/0'/0/0
    pub fn bip44(coin_type: u32) -> Self {
        Self {
            purpose: 44,
            coin_type,
            account: 0,
            change: 0,
            address_index: 0,
        }
    }

    /// Solana uses hardened path: m/44'/501'/0'/0'
    pub fn solana() -> Self {
        Self {
            purpose: 44,
            coin_type: 501,
            account: 0,
            change: 0,
            address_index: 0,
        }
    }

    /// Format as string
    pub fn to_string(&self) -> String {
        format!(
            "m/44'/{}'/{}'/{}/{}",
            self.coin_type, self.account, self.change, self.address_index
        )
    }
}

/// SLIP-10 / BIP32 key derivation from seed
/// Returns (private_key, chain_code)
pub fn derive_key_from_seed(seed: &[u8; 64], path: &DerivationPath) -> Result<([u8; 32], [u8; 32]), String> {
    use hmac::Mac;

    // Master key derivation: HMAC-SHA512 with key "Bitcoin seed"
    let mut mac = HmacSha512::new_from_slice(b"Bitcoin seed")
        .map_err(|e| format!("HMAC error: {}", e))?;
    mac.update(seed);
    let result = mac.finalize().into_bytes();

    let mut key = [0u8; 32];
    let mut chain_code = [0u8; 32];
    key.copy_from_slice(&result[..32]);
    chain_code.copy_from_slice(&result[32..]);

    // Derive through path: purpose' / coin_type' / account' / change / address_index
    let indices = [
        path.purpose | 0x80000000,     // hardened
        path.coin_type | 0x80000000,   // hardened
        path.account | 0x80000000,     // hardened
        path.change,
        path.address_index,
    ];

    for index in indices {
        let mut mac = HmacSha512::new_from_slice(&chain_code)
            .map_err(|e| format!("HMAC error: {}", e))?;

        if index & 0x80000000 != 0 {
            // Hardened: 0x00 || key || index
            mac.update(&[0x00]);
            mac.update(&key);
        } else {
            // Normal: pubkey || index
            // For secp256k1, compute compressed public key
            let pubkey = secp256k1_pubkey_from_private(&key)?;
            mac.update(&pubkey);
        }
        mac.update(&index.to_be_bytes());

        let result = mac.finalize().into_bytes();
        // For secp256k1 curves, we need to add parent and child keys mod n
        // For simplicity in hardened-only paths, just use the derived key directly
        let mut child_key = [0u8; 32];
        child_key.copy_from_slice(&result[..32]);

        // Add parent key to child key mod n (secp256k1 order)
        key = add_private_keys(&key, &child_key)?;
        chain_code.copy_from_slice(&result[32..]);
    }

    Ok((key, chain_code))
}

/// SLIP-10 Ed25519 key derivation (all hardened)
/// Used for Solana and TON
pub fn derive_ed25519_key_from_seed(seed: &[u8; 64], path: &DerivationPath) -> Result<([u8; 32], [u8; 32]), String> {
    use hmac::Mac;

    // Master key: HMAC-SHA512 with key "ed25519 seed"
    let mut mac = HmacSha512::new_from_slice(b"ed25519 seed")
        .map_err(|e| format!("HMAC error: {}", e))?;
    mac.update(seed);
    let result = mac.finalize().into_bytes();

    let mut key = [0u8; 32];
    let mut chain_code = [0u8; 32];
    key.copy_from_slice(&result[..32]);
    chain_code.copy_from_slice(&result[32..]);

    // All indices hardened for Ed25519
    let indices = [
        path.purpose | 0x80000000,
        path.coin_type | 0x80000000,
        path.account | 0x80000000,
        path.change | 0x80000000,
    ];

    for index in indices {
        let mut mac = HmacSha512::new_from_slice(&chain_code)
            .map_err(|e| format!("HMAC error: {}", e))?;
        mac.update(&[0x00]);
        mac.update(&key);
        mac.update(&index.to_be_bytes());

        let result = mac.finalize().into_bytes();
        key.copy_from_slice(&result[..32]);
        chain_code.copy_from_slice(&result[32..]);
    }

    Ok((key, chain_code))
}

/// Compute compressed secp256k1 public key from private key
fn secp256k1_pubkey_from_private(private_key: &[u8; 32]) -> Result<[u8; 33], String> {
    use k256::ecdsa::SigningKey;
    let signing_key = SigningKey::from_bytes(private_key.into())
        .map_err(|e| format!("Invalid private key: {}", e))?;
    let verifying_key = signing_key.verifying_key();
    let encoded = verifying_key.to_encoded_point(true);
    let bytes = encoded.as_bytes();
    let mut result = [0u8; 33];
    result.copy_from_slice(bytes);
    Ok(result)
}

/// Add two secp256k1 private keys mod n
fn add_private_keys(parent: &[u8; 32], child: &[u8; 32]) -> Result<[u8; 32], String> {
    use k256::elliptic_curve::ops::Reduce;
    use k256::{Scalar, U256};

    let parent_scalar = <Scalar as Reduce<U256>>::reduce_bytes(parent.into());
    let child_scalar = <Scalar as Reduce<U256>>::reduce_bytes(child.into());
    let sum = parent_scalar + child_scalar;

    let bytes = sum.to_bytes();
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bip39_utils;

    #[test]
    fn test_derivation_path_format() {
        let path = DerivationPath::bip44(60);
        assert_eq!(path.to_string(), "m/44'/60'/0'/0/0");
    }

    #[test]
    fn test_derive_key_deterministic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let path = DerivationPath::bip44(60); // Ethereum

        let (key1, _) = derive_key_from_seed(&seed, &path).unwrap();
        let (key2, _) = derive_key_from_seed(&seed, &path).unwrap();
        assert_eq!(key1, key2);
        assert_ne!(key1, [0u8; 32]);
    }

    #[test]
    fn test_derive_ed25519_deterministic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let path = DerivationPath::solana();

        let (key1, _) = derive_ed25519_key_from_seed(&seed, &path).unwrap();
        let (key2, _) = derive_ed25519_key_from_seed(&seed, &path).unwrap();
        assert_eq!(key1, key2);
        assert_ne!(key1, [0u8; 32]);
    }

    #[test]
    fn test_different_coin_types_different_keys() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();

        let (eth_key, _) = derive_key_from_seed(&seed, &DerivationPath::bip44(60)).unwrap();
        let (btc_key, _) = derive_key_from_seed(&seed, &DerivationPath::bip44(0)).unwrap();
        assert_ne!(eth_key, btc_key);
    }
}
