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
//   derive_all_addresses()  — Derive addresses for 13 chains from seed

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zeroize::Zeroize;

use crate::bip39_utils;
use crate::chains::evm::derive_evm_address;
use crate::chains::solana::derive_solana_address;
use crate::chains::ton::derive_ton_address;
use crate::chains::cosmos::derive_cosmos_address;
use crate::chains::bitcoin::derive_bitcoin_address_for_network;
use crate::chains::litecoin::derive_litecoin_address_for_network;
use crate::chains::stellar::derive_stellar_address;
use crate::chains::ripple::derive_ripple_address;
use crate::chains::dogecoin::derive_dogecoin_address_for_network;
use crate::chains::tron::derive_tron_address_for_network;
use crate::crypto;

/// Password strength levels
#[derive(Debug, Clone, PartialEq)]
pub enum PasswordStrength {
    Weak,
    Fair,
    Strong,
}

/// Validate password strength
/// Returns (strength, message)
pub fn validate_password_strength(password: &str) -> (PasswordStrength, &'static str) {
    if password.len() < 8 {
        return (PasswordStrength::Weak, "Minimo 8 caratteri");
    }

    let has_upper = password.chars().any(|c| c.is_uppercase());
    let has_lower = password.chars().any(|c| c.is_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());
    let score = has_upper as u8 + has_lower as u8 + has_digit as u8 + has_symbol as u8;

    if password.len() >= 12 && score >= 3 {
        (PasswordStrength::Strong, "Password forte")
    } else if password.len() >= 8 && score >= 2 {
        (PasswordStrength::Fair, "Password discreta — aggiungi simboli o numeri")
    } else {
        (PasswordStrength::Weak, "Troppo debole — usa maiuscole, numeri e simboli")
    }
}

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
        self.create_wallet_with_chains(name, mnemonic, password, None)
    }

    /// Create a new wallet from mnemonic phrase, deriving only selected chains
    pub fn create_wallet_with_chains(
        &mut self,
        name: &str,
        mnemonic: &str,
        password: &str,
        enabled_chains: Option<&[&str]>,
    ) -> Result<Wallet, String> {
        if !bip39_utils::validate_mnemonic(mnemonic) {
            return Err("Invalid mnemonic phrase".into());
        }

        let mut seed = bip39_utils::mnemonic_to_seed(mnemonic, "")?;
        let encrypted = crypto::encrypt(&seed, password)?;

        let entry = WalletEntry {
            name: name.to_string(),
            encrypted_seed: encrypted,
            created_at: current_timestamp(),
        };

        self.wallets.push(entry);
        self.active_index = self.wallets.len() - 1;

        let result = derive_addresses_filtered(&seed, false, enabled_chains)
            .map(|addresses| Wallet {
                name: name.to_string(),
                addresses,
                created_at: current_timestamp(),
            });
        seed.zeroize();
        result
    }

    /// Unlock a wallet by index with password
    pub fn unlock_wallet(&self, index: usize, password: &str) -> Result<Wallet, String> {
        self.unlock_wallet_for_network(index, password, false)
    }

    /// Unlock a wallet by index with password, with network selection
    pub fn unlock_wallet_for_network(&self, index: usize, password: &str, testnet: bool) -> Result<Wallet, String> {
        self.unlock_wallet_with_chains(index, password, testnet, None)
    }

    /// Unlock a wallet, deriving only selected chains
    pub fn unlock_wallet_with_chains(&self, index: usize, password: &str, testnet: bool, enabled_chains: Option<&[&str]>) -> Result<Wallet, String> {
        let entry = self.wallets.get(index)
            .ok_or("Wallet not found")?;

        let mut seed_bytes = crypto::decrypt(&entry.encrypted_seed, password)?;
        if seed_bytes.len() != 64 {
            seed_bytes.zeroize();
            return Err("Invalid seed data".into());
        }

        let mut seed = [0u8; 64];
        seed.copy_from_slice(&seed_bytes);
        seed_bytes.zeroize();

        let result = derive_addresses_filtered(&seed, testnet, enabled_chains)
            .map(|addresses| Wallet {
                name: entry.name.clone(),
                addresses,
                created_at: entry.created_at,
            });
        seed.zeroize();
        result
    }

    /// Phase 1: Validate mnemonic, encrypt seed, store entry. Returns raw seed for phase 2.
    /// Use this for non-blocking UI: call this in one event loop tick, then
    /// call derive_addresses_filtered() in the next tick.
    pub fn encrypt_and_store(
        &mut self,
        name: &str,
        mnemonic: &str,
        password: &str,
    ) -> Result<[u8; 64], String> {
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
        Ok(seed)
    }

    /// Phase 1 (unlock): Decrypt seed only. Returns raw seed for phase 2.
    pub fn decrypt_seed(&self, index: usize, password: &str) -> Result<(String, u64, [u8; 64]), String> {
        let entry = self.wallets.get(index)
            .ok_or("Wallet not found")?;
        let mut seed_bytes = crypto::decrypt(&entry.encrypted_seed, password)?;
        if seed_bytes.len() != 64 {
            seed_bytes.zeroize();
            return Err("Invalid seed data".into());
        }
        let mut seed = [0u8; 64];
        seed.copy_from_slice(&seed_bytes);
        seed_bytes.zeroize();
        Ok((entry.name.clone(), entry.created_at, seed))
    }

    /// Store a pre-encrypted wallet entry. Used for 3-phase non-blocking UI:
    /// Phase 1: mnemonic_to_seed, Phase 2: crypto::encrypt, Phase 3: derive_addresses.
    pub fn store_encrypted(&mut self, name: &str, encrypted_seed: crypto::EncryptedData) {
        let entry = WalletEntry {
            name: name.to_string(),
            encrypted_seed,
            created_at: current_timestamp(),
        };
        self.wallets.push(entry);
        self.active_index = self.wallets.len() - 1;
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

/// Derive addresses for all supported chains with network selection
pub fn derive_all_addresses_for_network(seed: &[u8; 64], testnet: bool) -> Result<HashMap<String, String>, String> {
    derive_addresses_filtered(seed, testnet, None)
}

/// All supported chain IDs
pub const ALL_CHAIN_IDS: &[&str] = &[
    "ethereum", "polygon", "bsc", "optimism", "base", "arbitrum",
    "solana", "ton", "cosmos", "osmosis", "bitcoin", "litecoin", "stellar", "ripple",
    "dogecoin", "tron",
];

/// EVM chain IDs (share same address)
pub const EVM_CHAIN_IDS: &[&str] = &[
    "ethereum", "polygon", "bsc", "optimism", "base", "arbitrum",
];

/// Derive addresses for selected chains (None = all chains)
pub fn derive_addresses_filtered(
    seed: &[u8; 64],
    testnet: bool,
    enabled: Option<&[&str]>,
) -> Result<HashMap<String, String>, String> {
    let mut addresses = HashMap::new();

    let is_enabled = |chain: &str| -> bool {
        match &enabled {
            None => true,
            Some(list) => list.contains(&chain),
        }
    };

    // EVM chains (all share same address — derive once if any EVM chain is enabled)
    let any_evm = EVM_CHAIN_IDS.iter().any(|c| is_enabled(c));
    if any_evm {
        let evm_addr = derive_evm_address(seed)?;
        for chain_id in EVM_CHAIN_IDS {
            if is_enabled(chain_id) {
                addresses.insert(chain_id.to_string(), evm_addr.clone());
            }
        }
    }

    // Solana
    if is_enabled("solana") {
        addresses.insert("solana".to_string(), derive_solana_address(seed)?);
    }

    // TON
    if is_enabled("ton") {
        addresses.insert("ton".to_string(), derive_ton_address(seed)?);
    }

    // Cosmos chains
    if is_enabled("cosmos") {
        addresses.insert("cosmos".to_string(), derive_cosmos_address(seed, "cosmos", 118)?);
    }
    if is_enabled("osmosis") {
        addresses.insert("osmosis".to_string(), derive_cosmos_address(seed, "osmo", 118)?);
    }

    // Bitcoin
    if is_enabled("bitcoin") {
        addresses.insert("bitcoin".to_string(), derive_bitcoin_address_for_network(seed, testnet)?);
    }

    // Litecoin
    if is_enabled("litecoin") {
        addresses.insert("litecoin".to_string(), derive_litecoin_address_for_network(seed, testnet)?);
    }

    // Stellar
    if is_enabled("stellar") {
        addresses.insert("stellar".to_string(), derive_stellar_address(seed)?);
    }

    // Ripple (XRP)
    if is_enabled("ripple") {
        addresses.insert("ripple".to_string(), derive_ripple_address(seed)?);
    }

    // Dogecoin (P2PKH)
    if is_enabled("dogecoin") {
        addresses.insert("dogecoin".to_string(), derive_dogecoin_address_for_network(seed, testnet)?);
    }

    // TRON
    if is_enabled("tron") {
        addresses.insert("tron".to_string(), derive_tron_address_for_network(seed, testnet)?);
    }

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
