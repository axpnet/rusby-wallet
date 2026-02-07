// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use super::get_json;

/// Get balance from Cosmos REST API
/// Note: rpc_url should be a REST/LCD endpoint, not Tendermint RPC
pub async fn get_balance(address: &str, rpc_url: &str, denom: &str, decimals: u32) -> Result<String, String> {
    // Convert RPC URL to REST if needed (common pattern)
    let rest_url = rpc_url_to_rest(rpc_url);
    let url = format!("{}/cosmos/bank/v1beta1/balances/{}", rest_url, address);

    let json = get_json(&url).await?;

    let balances = json["balances"].as_array()
        .ok_or("Missing balances array")?;

    let amount = balances.iter()
        .find(|b| b["denom"].as_str() == Some(denom))
        .and_then(|b| b["amount"].as_str())
        .unwrap_or("0");

    let raw: u64 = amount.parse().unwrap_or(0);
    Ok(format_micro(raw, decimals))
}

/// Get account number and sequence for transaction signing
pub async fn get_account_info(address: &str, rpc_url: &str) -> Result<(u64, u64), String> {
    let rest_url = rpc_url_to_rest(rpc_url);
    let url = format!("{}/cosmos/auth/v1beta1/accounts/{}", rest_url, address);

    let json = get_json(&url).await?;

    let account = &json["account"];
    let account_number: u64 = account["account_number"].as_str()
        .unwrap_or("0")
        .parse().unwrap_or(0);
    let sequence: u64 = account["sequence"].as_str()
        .unwrap_or("0")
        .parse().unwrap_or(0);

    Ok((account_number, sequence))
}

/// Broadcast Amino JSON transaction
pub async fn broadcast_tx(tx_json: &str, rpc_url: &str) -> Result<String, String> {
    use gloo_net::http::Request;

    let rest_url = rpc_url_to_rest(rpc_url);
    let url = format!("{}/cosmos/tx/v1beta1/txs", rest_url);

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .body(tx_json)
        .map_err(|e| format!("Request error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let text = response.text().await
        .map_err(|e| format!("Response error: {}", e))?;

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("JSON error: {}", e))?;

    if let Some(code) = json["tx_response"]["code"].as_u64() {
        if code != 0 {
            let log = json["tx_response"]["raw_log"].as_str().unwrap_or("Unknown error");
            return Err(format!("TX failed (code {}): {}", code, log));
        }
    }

    json["tx_response"]["txhash"].as_str()
        .map(|s| s.to_string())
        .ok_or("Missing txhash in response".into())
}

/// Convert Tendermint RPC URL to REST/LCD endpoint
/// e.g. cosmos-rpc.polkachu.com → cosmos-rest.polkachu.com
pub fn rpc_url_to_rest(rpc_url: &str) -> String {
    if rpc_url.contains("-rpc.") {
        rpc_url.replace("-rpc.", "-rest.")
    } else if rpc_url.contains("rpc.cosmos") {
        rpc_url.replace("rpc.cosmos", "rest.cosmos")
    } else {
        // Fallback: use as-is
        rpc_url.to_string()
    }
}

/// Get all non-native balances (IBC tokens) from Cosmos bank module
pub async fn get_all_balances(address: &str, rpc_url: &str, chain_id: &str) -> Result<Vec<wallet_core::tokens::TokenBalance>, String> {
    use wallet_core::tokens::{Token, TokenBalance};

    let rest_url = rpc_url_to_rest(rpc_url);
    let url = format!("{}/cosmos/bank/v1beta1/balances/{}", rest_url, address);

    let json = get_json(&url).await?;

    let balances = json["balances"].as_array()
        .ok_or("Missing balances array")?;

    // Native denoms to skip
    let native_denoms = match chain_id {
        "cosmos" => vec!["uatom"],
        "osmosis" => vec!["uosmo"],
        _ => vec![],
    };

    let mut result = Vec::new();
    for b in balances {
        let denom = match b["denom"].as_str() {
            Some(d) => d,
            None => continue,
        };
        if native_denoms.contains(&denom) {
            continue;
        }
        let amount_str = b["amount"].as_str().unwrap_or("0");
        let raw: u128 = amount_str.parse().unwrap_or(0);
        if raw == 0 {
            continue;
        }

        // IBC tokens: show truncated denom hash
        let symbol = if denom.starts_with("ibc/") {
            let hash = &denom[4..];
            format!("IBC/{}", &hash[..6.min(hash.len())])
        } else {
            denom.to_string()
        };

        // Assume 6 decimals for IBC tokens (standard for Cosmos)
        let decimals: u8 = 6;
        let formatted = format_micro(raw as u64, decimals as u32);

        result.push(TokenBalance {
            token: Token {
                address: denom.to_string(),
                symbol,
                name: denom.to_string(),
                decimals,
                chain_id: chain_id.to_string(),
            },
            balance: formatted,
            balance_usd: 0.0,
        });
    }

    Ok(result)
}

fn format_micro(amount: u64, decimals: u32) -> String {
    let divisor = 10u64.pow(decimals);
    let integer = amount / divisor;
    let fraction = amount % divisor;
    let frac_str = format!("{:0>width$}", fraction, width = decimals as usize);
    format!("{}.{}", integer, &frac_str[..4])
}
