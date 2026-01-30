// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// rpc: Multi-chain RPC client for balance queries and transaction broadcasting

pub mod evm;
pub mod solana;
pub mod ton;
pub mod cosmos;

use wallet_core::chains::{ChainId, supported_chains};

/// Fetch the native balance for any supported chain
pub async fn fetch_balance(chain_id: &str, address: &str) -> Result<String, String> {
    let chains = supported_chains();
    let config = chains.iter()
        .find(|c| chain_id_to_string(&c.id) == chain_id)
        .ok_or_else(|| format!("Unknown chain: {}", chain_id))?;

    let rpc_url = config.rpc_urls.first()
        .ok_or_else(|| format!("No RPC URL for {}", chain_id))?;

    match &config.id {
        ChainId::Ethereum | ChainId::Polygon | ChainId::Bsc |
        ChainId::Optimism | ChainId::Base | ChainId::Arbitrum => {
            evm::get_balance(address, rpc_url).await
        }
        ChainId::Solana => {
            solana::get_balance(address, rpc_url).await
        }
        ChainId::Ton => {
            ton::get_balance(address, rpc_url).await
        }
        ChainId::CosmosHub => {
            cosmos::get_balance(address, rpc_url, "uatom", 6).await
        }
        ChainId::Osmosis => {
            cosmos::get_balance(address, rpc_url, "uosmo", 6).await
        }
        _ => Ok("0.0000".into()),
    }
}

/// Helper to post JSON-RPC requests
pub async fn post_json(url: &str, body: &str) -> Result<serde_json::Value, String> {
    use gloo_net::http::Request;

    let response = Request::post(url)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|e| format!("Request build error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let text = response.text().await
        .map_err(|e| format!("Response read error: {}", e))?;

    serde_json::from_str(&text)
        .map_err(|e| format!("JSON parse error: {}", e))
}

/// Helper to GET a URL and parse as JSON
pub async fn get_json(url: &str) -> Result<serde_json::Value, String> {
    use gloo_net::http::Request;

    let response = Request::get(url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let text = response.text().await
        .map_err(|e| format!("Response read error: {}", e))?;

    serde_json::from_str(&text)
        .map_err(|e| format!("JSON parse error: {}", e))
}

fn chain_id_to_string(id: &ChainId) -> &'static str {
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
    }
}
