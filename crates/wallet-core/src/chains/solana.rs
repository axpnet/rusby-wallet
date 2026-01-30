// Rusby Wallet — Pure Rust multi-chain crypto wallet
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// chains/solana: Solana address derivation (Ed25519 + base58)
//
// Functions:
//   derive_solana_address() — seed → SLIP-10 m/44'/501'/0'/0' → Ed25519 → base58
//   get_keypair()           — Extract 64-byte keypair (private + public)

use crate::bip32_utils::{self, DerivationPath};
use ed25519_dalek::SigningKey;

use super::{Chain, ChainId};

pub struct SolanaChain;

impl Chain for SolanaChain {
    fn derive_address(&self, seed: &[u8; 64]) -> Result<String, String> {
        derive_solana_address(seed)
    }

    fn name(&self) -> &str {
        "Solana"
    }

    fn ticker(&self) -> &str {
        "SOL"
    }

    fn chain_id(&self) -> ChainId {
        ChainId::Solana
    }
}

/// Derive Solana address from seed
/// Path: m/44'/501'/0'/0' (SLIP-10 Ed25519, all hardened)
/// Address = base58(public_key)
pub fn derive_solana_address(seed: &[u8; 64]) -> Result<String, String> {
    let path = DerivationPath::solana();
    let (private_key, _) = bip32_utils::derive_ed25519_key_from_seed(seed, &path)?;

    let signing_key = SigningKey::from_bytes(&private_key);
    let public_key = signing_key.verifying_key();

    Ok(bs58::encode(public_key.as_bytes()).into_string())
}

/// Get the Ed25519 keypair bytes (64 bytes: private + public)
pub fn get_keypair(seed: &[u8; 64]) -> Result<[u8; 64], String> {
    let path = DerivationPath::solana();
    let (private_key, _) = bip32_utils::derive_ed25519_key_from_seed(seed, &path)?;

    let signing_key = SigningKey::from_bytes(&private_key);
    let public_key = signing_key.verifying_key();

    let mut keypair = [0u8; 64];
    keypair[..32].copy_from_slice(&private_key);
    keypair[32..].copy_from_slice(public_key.as_bytes());
    Ok(keypair)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bip39_utils;

    #[test]
    fn test_derive_solana_address() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_solana_address(&seed).unwrap();

        // Solana addresses are base58-encoded 32-byte public keys (32-44 chars)
        assert!(address.len() >= 32 && address.len() <= 44);
        // Should be valid base58
        assert!(bs58::decode(&address).into_vec().is_ok());
    }

    #[test]
    fn test_solana_deterministic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let addr1 = derive_solana_address(&seed).unwrap();
        let addr2 = derive_solana_address(&seed).unwrap();
        assert_eq!(addr1, addr2);
    }
}
