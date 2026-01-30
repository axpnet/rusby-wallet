// Rusby Wallet — Pure Rust multi-chain crypto wallet
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// chains/evm: EVM address derivation (ETH, Polygon, BSC, OP, Base, Arbitrum)
//
// Functions:
//   derive_evm_address() — seed → BIP44 m/44'/60'/0'/0/0 → secp256k1 → keccak256 → EIP-55
//   get_private_key()    — Extract raw private key for tx signing
//   eip55_checksum()     — Mixed-case checksum encoding

use crate::bip32_utils::{self, DerivationPath};
use k256::ecdsa::SigningKey;
use tiny_keccak::{Hasher, Keccak};

use super::{Chain, ChainId};

/// EVM-compatible chain (Ethereum, Polygon, BSC, etc.)
/// All EVM chains share the same derivation logic: secp256k1 + keccak256
pub struct EvmChain {
    pub id: ChainId,
    pub name: String,
    pub ticker: String,
    pub evm_chain_id: u64,
}

impl EvmChain {
    pub fn ethereum() -> Self {
        Self {
            id: ChainId::Ethereum,
            name: "Ethereum".into(),
            ticker: "ETH".into(),
            evm_chain_id: 1,
        }
    }

    pub fn polygon() -> Self {
        Self {
            id: ChainId::Polygon,
            name: "Polygon".into(),
            ticker: "POL".into(),
            evm_chain_id: 137,
        }
    }

    pub fn bsc() -> Self {
        Self {
            id: ChainId::Bsc,
            name: "BNB Smart Chain".into(),
            ticker: "BNB".into(),
            evm_chain_id: 56,
        }
    }

    pub fn optimism() -> Self {
        Self {
            id: ChainId::Optimism,
            name: "Optimism".into(),
            ticker: "ETH".into(),
            evm_chain_id: 10,
        }
    }

    pub fn base() -> Self {
        Self {
            id: ChainId::Base,
            name: "Base".into(),
            ticker: "ETH".into(),
            evm_chain_id: 8453,
        }
    }

    pub fn arbitrum() -> Self {
        Self {
            id: ChainId::Arbitrum,
            name: "Arbitrum".into(),
            ticker: "ETH".into(),
            evm_chain_id: 42161,
        }
    }
}

impl Chain for EvmChain {
    fn derive_address(&self, seed: &[u8; 64]) -> Result<String, String> {
        derive_evm_address(seed)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn ticker(&self) -> &str {
        &self.ticker
    }

    fn chain_id(&self) -> ChainId {
        self.id.clone()
    }
}

/// Derive an EVM address from a BIP39 seed
/// Path: m/44'/60'/0'/0/0
/// Process: seed → BIP32 private key → secp256k1 public key → keccak256 → last 20 bytes
pub fn derive_evm_address(seed: &[u8; 64]) -> Result<String, String> {
    let path = DerivationPath::bip44(60);
    let (private_key, _) = bip32_utils::derive_key_from_seed(seed, &path)?;

    let signing_key = SigningKey::from_bytes((&private_key).into())
        .map_err(|e| format!("Invalid key: {}", e))?;
    let verifying_key = signing_key.verifying_key();

    // Get uncompressed public key (65 bytes: 0x04 + x + y), skip the 0x04 prefix
    let encoded = verifying_key.to_encoded_point(false);
    let pubkey_bytes = &encoded.as_bytes()[1..]; // skip 0x04

    // Keccak256 hash of the public key
    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(pubkey_bytes);
    hasher.finalize(&mut hash);

    // Take last 20 bytes as address
    let address = &hash[12..32];

    // EIP-55 checksum encoding
    Ok(eip55_checksum(address))
}

/// Get the private key bytes for signing
pub fn get_private_key(seed: &[u8; 64]) -> Result<[u8; 32], String> {
    let path = DerivationPath::bip44(60);
    let (private_key, _) = bip32_utils::derive_key_from_seed(seed, &path)?;
    Ok(private_key)
}

/// EIP-55 mixed-case checksum encoding
fn eip55_checksum(address: &[u8]) -> String {
    let hex_addr = hex::encode(address);

    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(hex_addr.as_bytes());
    hasher.finalize(&mut hash);
    let hash_hex = hex::encode(hash);

    let mut result = String::with_capacity(42);
    result.push_str("0x");

    for (i, c) in hex_addr.chars().enumerate() {
        let hash_char = hash_hex.chars().nth(i).unwrap();
        if hash_char >= '8' {
            result.push(c.to_ascii_uppercase());
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bip39_utils;

    #[test]
    fn test_derive_evm_address_format() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_evm_address(&seed).unwrap();

        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);
    }

    #[test]
    fn test_derive_evm_deterministic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let addr1 = derive_evm_address(&seed).unwrap();
        let addr2 = derive_evm_address(&seed).unwrap();
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_eip55_checksum() {
        // Known address bytes
        let addr_bytes = hex::decode("d8da6bf26964af9d7eed9e03e53415d37aa96045").unwrap();
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&addr_bytes);
        let checksummed = eip55_checksum(&bytes);
        assert_eq!(checksummed, "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
    }
}
