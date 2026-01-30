// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use super::post_json;

/// Get native ETH/token balance via eth_getBalance
/// Returns formatted balance string (e.g. "1.2345")
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getBalance",
        "params": [address, "latest"],
        "id": 1
    }).to_string();

    let json = post_json(rpc_url, &body).await?;
    let hex_balance = json["result"].as_str()
        .ok_or("Missing result in eth_getBalance response")?;

    let wei = parse_hex_u128(hex_balance)?;
    Ok(format_wei(wei, 18))
}

/// Get transaction count (nonce)
pub async fn get_nonce(address: &str, rpc_url: &str) -> Result<u64, String> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getTransactionCount",
        "params": [address, "latest"],
        "id": 1
    }).to_string();

    let json = post_json(rpc_url, &body).await?;
    let hex_nonce = json["result"].as_str()
        .ok_or("Missing result in eth_getTransactionCount")?;

    parse_hex_u64(hex_nonce)
}

/// Get current gas price
pub async fn get_gas_price(rpc_url: &str) -> Result<u128, String> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_gasPrice",
        "params": [],
        "id": 1
    }).to_string();

    let json = post_json(rpc_url, &body).await?;
    let hex_price = json["result"].as_str()
        .ok_or("Missing result in eth_gasPrice")?;

    parse_hex_u128(hex_price)
}

/// Get max priority fee per gas (EIP-1559)
pub async fn get_max_priority_fee(rpc_url: &str) -> Result<u128, String> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_maxPriorityFeePerGas",
        "params": [],
        "id": 1
    }).to_string();

    let json = post_json(rpc_url, &body).await?;
    let hex_fee = json["result"].as_str()
        .ok_or("Missing result in eth_maxPriorityFeePerGas")?;

    parse_hex_u128(hex_fee)
}

/// Broadcast a signed raw transaction
pub async fn send_raw_transaction(signed_hex: &str, rpc_url: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_sendRawTransaction",
        "params": [signed_hex],
        "id": 1
    }).to_string();

    let json = post_json(rpc_url, &body).await?;

    if let Some(error) = json.get("error") {
        return Err(format!("RPC error: {}", error));
    }

    json["result"].as_str()
        .map(|s| s.to_string())
        .ok_or("Missing result in eth_sendRawTransaction".into())
}

fn parse_hex_u128(hex: &str) -> Result<u128, String> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    u128::from_str_radix(hex, 16).map_err(|e| format!("Hex parse error: {}", e))
}

fn parse_hex_u64(hex: &str) -> Result<u64, String> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    u64::from_str_radix(hex, 16).map_err(|e| format!("Hex parse error: {}", e))
}

/// Format wei to human-readable (e.g. 1000000000000000000 → "1.0000")
fn format_wei(wei: u128, decimals: u32) -> String {
    let divisor = 10u128.pow(decimals);
    let integer = wei / divisor;
    let fraction = wei % divisor;
    let frac_str = format!("{:0>width$}", fraction, width = decimals as usize);
    // Show 4 decimal places
    format!("{}.{}", integer, &frac_str[..4])
}
