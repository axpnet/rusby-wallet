// Rusby Wallet — Pure Rust multi-chain crypto wallet
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// wallet: Multi-wallet manager — create, encrypt, unlock, derive addresses
//
// Types:
//   Wallet       — Unlocked wallet with derived addresses for all chains
//   WalletStore  — Persistent store of encrypted wallet entries
//   WalletEntry  — Single encrypted wallet (name + encrypted seed + timestamp)
// Functions:
//   create_wallet()         — Generate wallet from mnemonic, encrypt seed
//   unlock_wallet()         — Decrypt seed and derive all addresses
//   derive_all_addresses()  — Derive addresses for 11 chains from seed

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::bip39_utils;
use crate::chains;
use crate::chains::evm::derive_evm_address;
use crate::chains::solana::derive_solana_address;
use crate::chains::ton::derive_ton_address;
use crate::chains::cosmos::derive_cosmos_address;
use crate::crypto;

/// A wallet instance with derived addresses for all chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub name: String,
    pub addresses: HashMap<String, String>,
    pub created_at: u64,
}

/// Wallet manager handles multiple wallets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStore {
    pub wallets: Vec<WalletEntry>,
    pub active_index: usize,
}

/// A stored wallet entry (encrypted seed + metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletEntry {
    pub name: String,
    pub encrypted_seed: crypto::EncryptedData,
    pub created_at: u64,
}

impl WalletStore {
    pub fn new() -> Self {
        Self {
            wallets: Vec::new(),
            active_index: 0,
        }
    }

    /// Create a new wallet from mnemonic phrase
    pub fn create_wallet(
        &mut self,
        name: &str,
        mnemonic: &str,
        password: &str,
    ) -> Result<Wallet, String> {
        if !bip39_utils::validate_mnemonic(mnemonic) {
            return Err("Invalid mnemonic phrase".into());
        }

        let seed = bip39_utils::mnemonic_to_seed(mnemonic, "")?;
        let encrypted = crypto::encrypt(&seed, password)?;

        let entry = WalletEntry {
            name: name.to_string(),
            encrypted_seed: encrypted,
            created_at: current_timestamp(),
        };

        self.wallets.push(entry);
        self.active_index = self.wallets.len() - 1;

        derive_all_addresses(&seed)
            .map(|addresses| Wallet {
                name: name.to_string(),
                addresses,
                created_at: current_timestamp(),
            })
    }

    /// Unlock a wallet by index with password
    pub fn unlock_wallet(&self, index: usize, password: &str) -> Result<Wallet, String> {
        let entry = self.wallets.get(index)
            .ok_or("Wallet not found")?;

        let seed_bytes = crypto::decrypt(&entry.encrypted_seed, password)?;
        if seed_bytes.len() != 64 {
            return Err("Invalid seed data".into());
        }

        let mut seed = [0u8; 64];
        seed.copy_from_slice(&seed_bytes);

        derive_all_addresses(&seed)
            .map(|addresses| Wallet {
                name: entry.name.clone(),
                addresses,
                created_at: entry.created_at,
            })
    }

    /// Get wallet count
    pub fn count(&self) -> usize {
        self.wallets.len()
    }

    /// Get wallet names
    pub fn wallet_names(&self) -> Vec<String> {
        self.wallets.iter().map(|w| w.name.clone()).collect()
    }
}

/// Derive addresses for all supported chains
fn derive_all_addresses(seed: &[u8; 64]) -> Result<HashMap<String, String>, String> {
    let mut addresses = HashMap::new();

    // EVM chains (all share same address)
    let evm_addr = derive_evm_address(seed)?;
    for chain_id in &["ethereum", "polygon", "bsc", "optimism", "base", "arbitrum"] {
        addresses.insert(chain_id.to_string(), evm_addr.clone());
    }

    // Solana
    addresses.insert("solana".to_string(), derive_solana_address(seed)?);

    // TON
    addresses.insert("ton".to_string(), derive_ton_address(seed)?);

    // Cosmos chains
    addresses.insert("cosmos".to_string(), derive_cosmos_address(seed, "cosmos", 118)?);
    addresses.insert("osmosis".to_string(), derive_cosmos_address(seed, "osmo", 118)?);

    // Bitcoin - placeholder until bdk integration
    addresses.insert("bitcoin".to_string(), "btc_address_pending".into());

    Ok(addresses)
}

fn current_timestamp() -> u64 {
    // In WASM, we'd use js_sys::Date::now()
    // For native, use std::time
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    #[cfg(target_arch = "wasm32")]
    {
        0 // Will use js_sys in WASM builds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bip39_utils::{generate_mnemonic, WordCount};

    #[test]
    fn test_create_and_unlock_wallet() {
        let mut store = WalletStore::new();
        let mnemonic = generate_mnemonic(WordCount::W12);

        let wallet = store.create_wallet("Test", &mnemonic, "password123").unwrap();
        assert_eq!(wallet.name, "Test");
        assert!(wallet.addresses.contains_key("ethereum"));
        assert!(wallet.addresses.contains_key("solana"));
        assert!(wallet.addresses.contains_key("cosmos"));

        let unlocked = store.unlock_wallet(0, "password123").unwrap();
        assert_eq!(unlocked.addresses["ethereum"], wallet.addresses["ethereum"]);
    }

    #[test]
    fn test_wrong_password() {
        let mut store = WalletStore::new();
        let mnemonic = generate_mnemonic(WordCount::W12);
        store.create_wallet("Test", &mnemonic, "correct").unwrap();

        let result = store.unlock_wallet(0, "wrong");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_wallets() {
        let mut store = WalletStore::new();
        let m1 = generate_mnemonic(WordCount::W12);
        let m2 = generate_mnemonic(WordCount::W12);

        store.create_wallet("Wallet 1", &m1, "pass1").unwrap();
        store.create_wallet("Wallet 2", &m2, "pass2").unwrap();

        assert_eq!(store.count(), 2);
        assert_eq!(store.wallet_names(), vec!["Wallet 1", "Wallet 2"]);
    }
}
