// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// rpc: Multi-chain RPC client for balance queries and transaction broadcasting

pub mod evm;
pub mod solana;
pub mod ton;
pub mod cosmos;
pub mod bitcoin;
pub mod litecoin;
pub mod stellar;
pub mod ripple;
pub mod dogecoin;
pub mod tron;
pub mod prices;
pub mod erc20;
pub mod spl;
pub mod history;
pub mod approvals;
pub mod simulate;
pub mod cw20;
pub mod jetton;
pub mod nft;
pub mod swap;

use wallet_core::chains::{ChainId, get_chains};

/// Fetch the native balance for any supported chain (mainnet)
pub async fn fetch_balance(chain_id: &str, address: &str) -> Result<String, String> {
    fetch_balance_for_network(chain_id, address, false).await
}

/// Fetch the native balance for any supported chain with network selection
pub async fn fetch_balance_for_network(chain_id: &str, address: &str, testnet: bool) -> Result<String, String> {
    let chains = get_chains(testnet);
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
        ChainId::Bitcoin => {
            bitcoin::get_balance_for_network(address, testnet).await
        }
        ChainId::Litecoin => {
            litecoin::get_balance_for_network(address, testnet).await
        }
        ChainId::Stellar => {
            stellar::get_balance(address, rpc_url).await
        }
        ChainId::Ripple => {
            ripple::get_balance(address, rpc_url).await
        }
        ChainId::Dogecoin => {
            dogecoin::get_balance_for_network(address, testnet).await
        }
        ChainId::Tron => {
            tron::get_balance(address, rpc_url).await
        }
    }
}

/// Helper to post JSON-RPC requests (with 30s timeout and status check)
pub async fn post_json(url: &str, body: &str) -> Result<serde_json::Value, String> {
    use gloo_net::http::Request;
    use wasm_bindgen::JsCast;

    // Create AbortController for timeout
    let abort_controller = web_sys::AbortController::new()
        .map_err(|_| "Failed to create AbortController".to_string())?;
    let signal = abort_controller.signal();

    // Set 30-second timeout
    let controller_clone = abort_controller.clone();
    let timeout_id = web_sys::window()
        .and_then(|w| {
            let cb = wasm_bindgen::closure::Closure::once(move || {
                controller_clone.abort();
            });
            let id = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(), 30_000
            ).ok();
            cb.forget();
            id
        });

    let result = Request::post(url)
        .header("Content-Type", "application/json")
        .abort_signal(Some(&signal))
        .body(body)
        .map_err(|e| format!("Request build error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error (timeout?): {}", e));

    // Clear timeout
    if let Some(id) = timeout_id {
        if let Some(w) = web_sys::window() {
            w.clear_timeout_with_handle(id);
        }
    }

    let response = result?;

    if response.status() >= 400 {
        return Err(format!("HTTP error: status {}", response.status()));
    }

    let text = response.text().await
        .map_err(|e| format!("Response read error: {}", e))?;

    serde_json::from_str(&text)
        .map_err(|e| format!("JSON parse error: {}", e))
}

/// Helper to GET a URL and parse as JSON (with 30s timeout and status check)
pub async fn get_json(url: &str) -> Result<serde_json::Value, String> {
    use gloo_net::http::Request;
    use wasm_bindgen::JsCast;

    let abort_controller = web_sys::AbortController::new()
        .map_err(|_| "Failed to create AbortController".to_string())?;
    let signal = abort_controller.signal();

    let controller_clone = abort_controller.clone();
    let timeout_id = web_sys::window()
        .and_then(|w| {
            let cb = wasm_bindgen::closure::Closure::once(move || {
                controller_clone.abort();
            });
            let id = w.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(), 30_000
            ).ok();
            cb.forget();
            id
        });

    let result = Request::get(url)
        .abort_signal(Some(&signal))
        .send()
        .await
        .map_err(|e| format!("Network error (timeout?): {}", e));

    if let Some(id) = timeout_id {
        if let Some(w) = web_sys::window() {
            w.clear_timeout_with_handle(id);
        }
    }

    let response = result?;

    if response.status() >= 400 {
        return Err(format!("HTTP error: status {}", response.status()));
    }

    let text = response.text().await
        .map_err(|e| format!("Response read error: {}", e))?;

    serde_json::from_str(&text)
        .map_err(|e| format!("JSON parse error: {}", e))
}

pub fn chain_id_str(id: &ChainId) -> &'static str {
    chain_id_to_string(id)
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
        ChainId::Litecoin => "litecoin",
        ChainId::Stellar => "stellar",
        ChainId::Ripple => "ripple",
        ChainId::Dogecoin => "dogecoin",
        ChainId::Tron => "tron",
    }
}
