// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// rpc/history: Fetch transaction history from block explorer APIs

use serde::{Deserialize, Serialize};

/// A transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxRecord {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub timestamp: String,
    pub direction: TxDirection,
    pub chain_id: String,
    pub explorer_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TxDirection {
    Sent,
    Received,
}

/// Fetch transaction history for EVM chains using public APIs
pub async fn fetch_evm_history(
    address: &str,
    chain_id: &str,
    _rpc_url: &str,
) -> Vec<TxRecord> {
    // Use free block explorer APIs
    let (api_url, explorer_base) = match chain_id {
        "ethereum" => ("https://api.etherscan.io/api", "https://etherscan.io/tx/"),
        "polygon" => ("https://api.polygonscan.com/api", "https://polygonscan.com/tx/"),
        "bsc" => ("https://api.bscscan.com/api", "https://bscscan.com/tx/"),
        "arbitrum" => ("https://api.arbiscan.io/api", "https://arbiscan.io/tx/"),
        "optimism" => ("https://api-optimistic.etherscan.io/api", "https://optimistic.etherscan.io/tx/"),
        "base" => ("https://api.basescan.org/api", "https://basescan.org/tx/"),
        _ => return Vec::new(),
    };

    let url = format!(
        "{}?module=account&action=txlist&address={}&startblock=0&endblock=99999999&page=1&offset=20&sort=desc",
        api_url, address
    );

    let json = match super::get_json(&url).await {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    let addr_lower = address.to_lowercase();
    let mut results = Vec::new();

    if let Some(txs) = json.get("result").and_then(|r| r.as_array()) {
        for tx in txs.iter().take(20) {
            let hash = tx.get("hash").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let from = tx.get("from").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let to = tx.get("to").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let value_wei = tx.get("value").and_then(|v| v.as_str()).unwrap_or("0");
            let timestamp = tx.get("timeStamp").and_then(|v| v.as_str()).unwrap_or("0").to_string();

            let direction = if from.to_lowercase() == addr_lower {
                TxDirection::Sent
            } else {
                TxDirection::Received
            };

            // Convert wei to ETH (simplified)
            let wei: u128 = value_wei.parse().unwrap_or(0);
            let eth = wei as f64 / 1e18;
            let value = format!("{:.4}", eth);

            // Convert unix timestamp to readable
            let ts: u64 = timestamp.parse().unwrap_or(0);
            let time_str = format_timestamp(ts);

            results.push(TxRecord {
                hash: hash.clone(),
                from,
                to,
                value,
                timestamp: time_str,
                direction,
                chain_id: chain_id.to_string(),
                explorer_url: format!("{}{}", explorer_base, hash),
            });
        }
    }

    results
}

/// Fetch Solana transaction history
pub async fn fetch_solana_history(
    address: &str,
    rpc_url: &str,
) -> Vec<TxRecord> {
    let body = format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"getSignaturesForAddress","params":["{}",{{"limit":20}}]}}"#,
        address
    );

    let json = match super::post_json(rpc_url, &body).await {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();

    if let Some(sigs) = json.get("result").and_then(|r| r.as_array()) {
        for sig in sigs.iter().take(20) {
            let hash = sig.get("signature").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let ts = sig.get("blockTime").and_then(|v| v.as_u64()).unwrap_or(0);
            let err = sig.get("err");

            results.push(TxRecord {
                hash: hash.clone(),
                from: address.to_string(),
                to: String::new(),
                value: if err.is_some() && !err.unwrap().is_null() { "Failed".into() } else { "—".into() },
                timestamp: format_timestamp(ts),
                direction: TxDirection::Sent,
                chain_id: "solana".to_string(),
                explorer_url: format!("https://solscan.io/tx/{}", hash),
            });
        }
    }

    results
}

/// Fetch TON transaction history
pub async fn fetch_ton_history(
    address: &str,
    rpc_url: &str,
) -> Vec<TxRecord> {
    let url = format!(
        "https://toncenter.com/api/v2/getTransactions?address={}&limit=20",
        address
    );

    let json = match super::get_json(&url).await {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();

    if let Some(txs) = json.get("result").and_then(|r| r.as_array()) {
        for tx in txs.iter().take(20) {
            let hash = tx.get("transaction_id").and_then(|t| t.get("hash")).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let utime = tx.get("utime").and_then(|v| v.as_u64()).unwrap_or(0);

            // Extract in_msg value
            let value_nano = tx.get("in_msg")
                .and_then(|m| m.get("value"))
                .and_then(|v| v.as_str().or_else(|| v.as_u64().map(|_| "").or(None)))
                .unwrap_or("0");
            let nano: u128 = value_nano.parse().unwrap_or(0);
            let ton_val = nano as f64 / 1e9;

            results.push(TxRecord {
                hash: hash.clone(),
                from: String::new(),
                to: String::new(),
                value: format!("{:.4} TON", ton_val),
                timestamp: format_timestamp(utime),
                direction: TxDirection::Received,
                chain_id: "ton".to_string(),
                explorer_url: format!("https://tonscan.org/tx/{}", hash),
            });
        }
    }

    results
}

/// Fetch Cosmos transaction history via LCD
pub async fn fetch_cosmos_history(
    address: &str,
    rpc_url: &str,
    chain_id: &str,
) -> Vec<TxRecord> {
    let rest_url = rpc_url.replace("-rpc.", "-rest.");
    let url = format!(
        "{}/cosmos/tx/v1beta1/txs?events=transfer.sender%3D'{}'&pagination.limit=20&order_by=ORDER_BY_DESC",
        rest_url, address
    );

    let json = match super::get_json(&url).await {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    let explorer_base = match chain_id {
        "cosmos" => "https://www.mintscan.io/cosmos/tx/",
        "osmosis" => "https://www.mintscan.io/osmosis/tx/",
        _ => "https://www.mintscan.io/cosmos/tx/",
    };

    let mut results = Vec::new();

    if let Some(txs) = json.get("tx_responses").and_then(|r| r.as_array()) {
        for tx in txs.iter().take(20) {
            let hash = tx.get("txhash").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let timestamp = tx.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string();

            results.push(TxRecord {
                hash: hash.clone(),
                from: address.to_string(),
                to: String::new(),
                value: "—".into(),
                timestamp,
                direction: TxDirection::Sent,
                chain_id: chain_id.to_string(),
                explorer_url: format!("{}{}", explorer_base, hash),
            });
        }
    }

    results
}

fn format_timestamp(ts: u64) -> String {
    if ts == 0 {
        return "—".to_string();
    }
    // Simple relative time
    let now = js_sys::Date::now() as u64 / 1000;
    let diff = now.saturating_sub(ts);

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}
