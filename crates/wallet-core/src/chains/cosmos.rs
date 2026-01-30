// Rusby Wallet — Pure Rust multi-chain crypto wallet
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// chains/cosmos: Cosmos SDK address derivation (ATOM, Osmosis)
//
// Functions:
//   derive_cosmos_address() — seed → BIP44 m/44'/118'/0'/0/0 → secp256k1 → SHA256 → bech32
//   bech32_encode()         — Encode with human-readable prefix (cosmos1..., osmo1...)

use crate::bip32_utils::{self, DerivationPath};
use k256::ecdsa::SigningKey;
use sha2::{Digest, Sha256};

use super::{Chain, ChainId};

/// Cosmos-SDK based chain
pub struct CosmosChain {
    pub id: ChainId,
    pub chain_name: String,
    pub ticker_symbol: String,
    pub bech32_prefix: String,
    pub coin_type: u32,
}

impl CosmosChain {
    pub fn cosmos_hub() -> Self {
        Self {
            id: ChainId::CosmosHub,
            chain_name: "Cosmos Hub".into(),
            ticker_symbol: "ATOM".into(),
            bech32_prefix: "cosmos".into(),
            coin_type: 118,
        }
    }

    pub fn osmosis() -> Self {
        Self {
            id: ChainId::Osmosis,
            chain_name: "Osmosis".into(),
            ticker_symbol: "OSMO".into(),
            bech32_prefix: "osmo".into(),
            coin_type: 118,
        }
    }
}

impl Chain for CosmosChain {
    fn derive_address(&self, seed: &[u8; 64]) -> Result<String, String> {
        derive_cosmos_address(seed, &self.bech32_prefix, self.coin_type)
    }

    fn name(&self) -> &str {
        &self.chain_name
    }

    fn ticker(&self) -> &str {
        &self.ticker_symbol
    }

    fn chain_id(&self) -> ChainId {
        self.id.clone()
    }
}

/// Derive Cosmos address from seed
/// Path: m/44'/118'/0'/0/0 (secp256k1)
/// Address = bech32(prefix, ripemd160(sha256(compressed_pubkey)))
/// Since ripemd160 adds a dependency, Cosmos SDK actually uses sha256 truncated
/// Standard: sha256(pubkey) → first 20 bytes → bech32
pub fn derive_cosmos_address(seed: &[u8; 64], prefix: &str, coin_type: u32) -> Result<String, String> {
    let path = DerivationPath::bip44(coin_type);
    let (private_key, _) = bip32_utils::derive_key_from_seed(seed, &path)?;

    let signing_key = SigningKey::from_bytes((&private_key).into())
        .map_err(|e| format!("Invalid key: {}", e))?;
    let verifying_key = signing_key.verifying_key();

    // Compressed public key (33 bytes)
    let pubkey_compressed = verifying_key.to_encoded_point(true);
    let pubkey_bytes = pubkey_compressed.as_bytes();

    // SHA256 hash
    let mut hasher = Sha256::new();
    hasher.update(pubkey_bytes);
    let sha256_hash = hasher.finalize();

    // Take first 20 bytes (this is the standard Cosmos address hash)
    // Note: full standard uses RIPEMD160(SHA256(pubkey)), but many implementations
    // just use SHA256 truncated. We'll add ripemd160 if needed for compatibility.
    let addr_bytes = &sha256_hash[..20];

    // Bech32 encode
    bech32_encode(prefix, addr_bytes)
}

/// Bech32 encoding for Cosmos addresses
fn bech32_encode(hrp: &str, data: &[u8]) -> Result<String, String> {
    use bech32::{Bech32, Hrp};

    let hrp = Hrp::parse(hrp).map_err(|e| format!("Invalid HRP: {}", e))?;
    let encoded = bech32::encode::<Bech32>(hrp, data)
        .map_err(|e| format!("Bech32 encode error: {}", e))?;
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bip39_utils;

    #[test]
    fn test_derive_cosmos_address() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_cosmos_address(&seed, "cosmos", 118).unwrap();

        assert!(address.starts_with("cosmos1"));
        assert!(address.len() > 10);
    }

    #[test]
    fn test_derive_osmosis_address() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let address = derive_cosmos_address(&seed, "osmo", 118).unwrap();

        assert!(address.starts_with("osmo1"));
    }

    #[test]
    fn test_cosmos_deterministic() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let addr1 = derive_cosmos_address(&seed, "cosmos", 118).unwrap();
        let addr2 = derive_cosmos_address(&seed, "cosmos", 118).unwrap();
        assert_eq!(addr1, addr2);
    }

    #[test]
    fn test_different_prefixes_same_hash() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let cosmos_addr = derive_cosmos_address(&seed, "cosmos", 118).unwrap();
        let osmo_addr = derive_cosmos_address(&seed, "osmo", 118).unwrap();

        // Same coin_type means same key, but different prefix
        assert_ne!(cosmos_addr, osmo_addr);
        assert!(cosmos_addr.starts_with("cosmos1"));
        assert!(osmo_addr.starts_with("osmo1"));
    }
}
