// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use super::post_json;

/// Get SOL balance in human-readable format
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "getBalance",
        "params": [address],
        "id": 1
    }).to_string();

    let json = post_json(rpc_url, &body).await?;
    let lamports = json["result"]["value"].as_u64()
        .ok_or("Missing value in getBalance response")?;

    Ok(format_lamports(lamports))
}

/// Get recent blockhash for transaction signing
pub async fn get_latest_blockhash(rpc_url: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "getLatestBlockhash",
        "params": [{"commitment": "finalized"}],
        "id": 1
    }).to_string();

    let json = post_json(rpc_url, &body).await?;
    json["result"]["value"]["blockhash"].as_str()
        .map(|s| s.to_string())
        .ok_or("Missing blockhash in response".into())
}

/// Send a signed transaction (base58 encoded)
pub async fn send_transaction(signed_b58: &str, rpc_url: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "sendTransaction",
        "params": [signed_b58, {"encoding": "base58"}],
        "id": 1
    }).to_string();

    let json = post_json(rpc_url, &body).await?;

    if let Some(error) = json.get("error") {
        return Err(format!("RPC error: {}", error));
    }

    json["result"].as_str()
        .map(|s| s.to_string())
        .ok_or("Missing result in sendTransaction".into())
}

fn format_lamports(lamports: u64) -> String {
    let sol = lamports / 1_000_000_000;
    let frac = lamports % 1_000_000_000;
    let frac_str = format!("{:09}", frac);
    format!("{}.{}", sol, &frac_str[..4])
}
