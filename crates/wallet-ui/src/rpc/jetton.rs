// Rusby Wallet — Jetton RPC client for TON
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use super::get_json;
use wallet_core::tokens::{Token, TokenBalance};
use wallet_core::tokens::jetton;

/// Get all Jetton balances using toncenter v3 REST API
/// Primary approach: /v3/jetton/wallets?owner_address=...
/// Fallback: iterate default tokens with runGetMethod
pub async fn get_all_token_balances(owner: &str, rpc_url: &str) -> Vec<TokenBalance> {
    // Try v3 REST API first
    let base_url = rpc_url.trim_end_matches("/jsonRPC");
    let v3_url = format!("{}/v3/jetton/wallets?owner_address={}&limit=50", base_url, owner);

    if let Ok(json) = get_json(&v3_url).await {
        if let Some(wallets) = json["jetton_wallets"].as_array() {
            let known_tokens = jetton::default_tokens();
            let mut result = Vec::new();

            for wallet in wallets {
                let balance_raw = wallet["balance"].as_str().unwrap_or("0");
                let raw: u128 = balance_raw.parse().unwrap_or(0);
                if raw == 0 {
                    continue;
                }

                let master = wallet["jetton"].as_str()
                    .or_else(|| wallet["jetton_master"].as_str())
                    .unwrap_or("");

                // Try to match with known tokens
                let (symbol, name, decimals) = if let Some(known) = known_tokens.iter().find(|t| t.address == master) {
                    (known.symbol.clone(), known.name.clone(), known.decimals)
                } else {
                    // Unknown jetton: use truncated address
                    let short = if master.len() > 10 {
                        format!("{}...", &master[..8])
                    } else {
                        master.to_string()
                    };
                    (short, "Unknown Jetton".to_string(), 9)
                };

                let formatted = format_jetton_balance(raw, decimals);

                result.push(TokenBalance {
                    token: Token {
                        address: master.to_string(),
                        symbol,
                        name,
                        decimals,
                        chain_id: "ton".into(),
                    },
                    balance: formatted,
                    balance_usd: 0.0,
                });
            }

            if !result.is_empty() {
                return result;
            }
        }
    }

    // Fallback: query each default token individually
    get_default_token_balances(owner, rpc_url).await
}

/// Fallback: query each default token via runGetMethod
async fn get_default_token_balances(owner: &str, rpc_url: &str) -> Vec<TokenBalance> {
    let tokens = jetton::default_tokens();
    let mut result = Vec::new();

    for token in tokens {
        // Resolve jetton wallet address
        let wallet_addr = match get_jetton_wallet_address(&token.address, owner, rpc_url).await {
            Ok(addr) => addr,
            Err(_) => continue,
        };

        // Get balance from wallet
        match get_jetton_balance(&wallet_addr, rpc_url, token.decimals).await {
            Ok(balance) => {
                if balance.starts_with("0.0000") {
                    continue;
                }
                result.push(TokenBalance {
                    token,
                    balance,
                    balance_usd: 0.0,
                });
            }
            Err(_) => continue,
        }
    }

    result
}

/// Resolve jetton wallet address via runGetMethod on the master contract
pub async fn get_jetton_wallet_address(master: &str, owner: &str, rpc_url: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "runGetMethod",
        "params": {
            "address": master,
            "method": "get_wallet_address",
            "stack": [
                ["tvm.Slice", owner]
            ]
        }
    }).to_string();

    let json = super::post_json(rpc_url, &body).await?;

    let stack = json["result"]["stack"].as_array()
        .ok_or("Missing stack in get_wallet_address response")?;

    if let Some(first) = stack.first() {
        if let Some(val) = first.get(1).and_then(|v| v.as_str()) {
            return Ok(val.to_string());
        }
    }

    Err("Could not resolve jetton wallet address".into())
}

/// Get jetton balance from a jetton wallet contract via runGetMethod
pub async fn get_jetton_balance(wallet_addr: &str, rpc_url: &str, decimals: u8) -> Result<String, String> {
    let body = serde_json::json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "runGetMethod",
        "params": {
            "address": wallet_addr,
            "method": "get_wallet_data",
            "stack": []
        }
    }).to_string();

    let json = super::post_json(rpc_url, &body).await?;

    let stack = json["result"]["stack"].as_array()
        .ok_or("Missing stack in get_wallet_data response")?;

    // stack[0] is the balance
    if let Some(first) = stack.first() {
        if let Some(val) = first.get(1).and_then(|v| v.as_str()) {
            let hex = val.strip_prefix("0x").unwrap_or(val);
            let raw = u128::from_str_radix(hex, 16)
                .map_err(|e| format!("Parse balance error: {}", e))?;
            return Ok(format_jetton_balance(raw, decimals));
        }
    }

    Ok("0.0000".into())
}

fn format_jetton_balance(raw: u128, decimals: u8) -> String {
    let divisor = 10u128.pow(decimals as u32);
    let integer = raw / divisor;
    let fraction = raw % divisor;
    let frac_str = format!("{:0>width$}", fraction, width = decimals as usize);
    let display_decimals = 4.min(decimals as usize);
    format!("{}.{}", integer, &frac_str[..display_decimals])
}
