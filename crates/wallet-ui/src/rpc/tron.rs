// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// rpc/tron: Balance query, TX creation, and broadcast for TRON
// Uses TronGrid API (https://api.trongrid.io / https://nile.trongrid.io)

use super::{get_json, post_json};

/// Get TRX balance in TRX (formatted string)
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String, String> {
    // TronGrid v1 API: GET /v1/accounts/{addr}
    let url = format!("{}/v1/accounts/{}", rpc_url, address);
    let json = get_json(&url).await?;

    let balance_sun = json["data"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|acc| acc["balance"].as_u64())
        .unwrap_or(0);

    Ok(wallet_core::tx::tron::format_sun(balance_sun))
}

/// Create a TRX transfer transaction via TronGrid API
/// Returns (txID, raw_data JSON, raw_data_hex)
pub async fn create_transaction(
    rpc_url: &str,
    from_hex: &str,
    to_hex: &str,
    amount_sun: u64,
) -> Result<(String, serde_json::Value, String), String> {
    let url = format!("{}/wallet/createtransaction", rpc_url);
    let body = serde_json::json!({
        "owner_address": from_hex,
        "to_address": to_hex,
        "amount": amount_sun,
        "visible": false
    }).to_string();

    let json = post_json(&url, &body).await?;

    // Check for errors (TronGrid sometimes returns hex-encoded error messages)
    if let Some(err) = json.get("Error") {
        let err_str = err.as_str().unwrap_or("Errore sconosciuto");
        return Err(format!("TronGrid error: {}", err_str));
    }

    let tx_id = json["txID"].as_str()
        .ok_or("Missing txID in response")?
        .to_string();

    let raw_data = json["raw_data"].clone();
    let raw_data_hex = json["raw_data_hex"].as_str()
        .ok_or("Missing raw_data_hex in response")?
        .to_string();

    Ok((tx_id, raw_data, raw_data_hex))
}

/// Broadcast a signed transaction
pub async fn broadcast_transaction(
    rpc_url: &str,
    tx_id: &str,
    raw_data: &serde_json::Value,
    raw_data_hex: &str,
    signature: &str,
) -> Result<String, String> {
    let url = format!("{}/wallet/broadcasttransaction", rpc_url);
    let body = serde_json::json!({
        "txID": tx_id,
        "raw_data": raw_data,
        "raw_data_hex": raw_data_hex,
        "signature": [signature],
        "visible": false
    }).to_string();

    let json = post_json(&url, &body).await?;

    if json["result"].as_bool() == Some(true) {
        Ok(tx_id.to_string())
    } else {
        // TronGrid error messages can be hex-encoded
        let msg = json["message"].as_str()
            .map(|m| {
                hex::decode(m)
                    .ok()
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                    .unwrap_or_else(|| m.to_string())
            })
            .unwrap_or_else(|| "Broadcast fallito".to_string());
        Err(msg)
    }
}
