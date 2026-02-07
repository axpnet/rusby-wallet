// Rusby Wallet — TX send logic, separated by chain
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod evm;
pub mod solana;
pub mod ton;
pub mod cosmos;
pub mod bitcoin;

use wallet_core::chains::ChainId;
use crate::logging::{log_info, log_error};

pub fn chain_id_to_string(id: &ChainId) -> String {
    match id {
        ChainId::Ethereum => "ethereum",
        ChainId::Polygon => "polygon",
        ChainId::Bsc => "bsc",
        ChainId::Optimism => "optimism",
        ChainId::Base => "base",
        ChainId::Arbitrum => "arbitrum",
        ChainId::Solana => "solana",
        ChainId::Ton => "ton",
        ChainId::Bitcoin => "bitcoin",
        ChainId::CosmosHub => "cosmos",
        ChainId::Osmosis => "osmosis",
    }.to_string()
}

/// Decrypt seed from wallet store
fn decrypt_seed(password: &str) -> Result<[u8; 64], String> {
    use zeroize::Zeroize;
    let store_json = crate::state::load_from_storage("wallet_store")
        .ok_or("No wallet found")?;
    let store: wallet_core::wallet::WalletStore = serde_json::from_str(&store_json)
        .map_err(|e| format!("Invalid wallet data: {}", e))?;

    let entry = store.wallets.get(store.active_index)
        .ok_or("No active wallet")?;
    let mut seed_bytes = wallet_core::crypto::decrypt(&entry.encrypted_seed, password)?;
    if seed_bytes.len() != 64 {
        seed_bytes.zeroize();
        return Err("Invalid seed".into());
    }
    let mut seed = [0u8; 64];
    seed.copy_from_slice(&seed_bytes);
    seed_bytes.zeroize();
    Ok(seed)
}

/// Execute send for any chain — main dispatch
pub async fn execute_send(chain: &str, to: &str, amount: &str, password: &str, token_address: &str) -> Result<String, String> {
    execute_send_for_network(chain, to, amount, password, token_address, false).await
}

/// Execute send with network selection (mainnet/testnet)
pub async fn execute_send_for_network(chain: &str, to: &str, amount: &str, password: &str, token_address: &str, testnet: bool) -> Result<String, String> {
    use zeroize::Zeroize;
    log_info!("TX send: chain={}, to={}...{}, amount={}, testnet={}", chain, &to[..6.min(to.len())], &to[to.len().saturating_sub(4)..], amount, testnet);
    let mut seed = decrypt_seed(password)?;

    let chains = wallet_core::chains::get_chains(testnet);
    let config = chains.iter()
        .find(|c| chain_id_to_string(&c.id) == chain)
        .ok_or("Unknown chain")?;
    let rpc_url = config.rpc_urls.first()
        .ok_or("No RPC URL")?;

    let result = match chain {
        "ethereum" | "polygon" | "bsc" | "optimism" | "base" | "arbitrum" => {
            if token_address.is_empty() {
                evm::send_native(&seed, to, amount, rpc_url, config).await
            } else {
                evm::send_erc20(&seed, to, amount, token_address, rpc_url, config).await
            }
        }
        "solana" => solana::send(&seed, to, amount, rpc_url).await,
        "ton" => {
            if token_address.is_empty() {
                ton::send(&seed, to, amount, rpc_url).await
            } else {
                ton::send_jetton(&seed, to, amount, token_address, rpc_url).await
            }
        }
        "cosmos" => {
            let cid = if testnet { "theta-testnet-001" } else { "cosmoshub-4" };
            if token_address.is_empty() {
                cosmos::send(&seed, to, amount, rpc_url, "uatom", cid, ChainId::CosmosHub).await
            } else {
                cosmos::send_cw20(&seed, to, amount, token_address, rpc_url, "uatom", cid, ChainId::CosmosHub).await
            }
        }
        "osmosis" => {
            let cid = if testnet { "osmo-test-5" } else { "osmosis-1" };
            if token_address.is_empty() {
                cosmos::send(&seed, to, amount, rpc_url, "uosmo", cid, ChainId::Osmosis).await
            } else {
                cosmos::send_cw20(&seed, to, amount, token_address, rpc_url, "uosmo", cid, ChainId::Osmosis).await
            }
        }
        "bitcoin" => bitcoin::send(&seed, to, amount).await,
        _ => Err(format!("Sending not supported for {}", chain)),
    };
    seed.zeroize();
    match &result {
        Ok(hash) => log_info!("TX success: {}", hash),
        Err(e) => log_error!("TX failed: {}", e),
    }
    result
}

pub fn base64_simple_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(CHARS[((n >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(CHARS[(n & 0x3F) as usize] as char); } else { result.push('='); }
    }
    result
}
