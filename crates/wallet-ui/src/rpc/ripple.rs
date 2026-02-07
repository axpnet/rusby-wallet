// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// rpc/ripple: Balance query, account info, and TX submission for XRP Ledger
// Uses rippled JSON-RPC API

use super::post_json;

/// Get XRP balance in XRP (formatted string)
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "method": "account_info",
        "params": [{
            "account": address,
            "ledger_index": "validated"
        }]
    }).to_string();

    let json = post_json(rpc_url, &body).await;

    match json {
        Ok(json) => {
            if let Some(balance_str) = json["result"]["account_data"]["Balance"].as_str() {
                if let Ok(drops) = balance_str.parse::<u64>() {
                    return Ok(format_drops(drops));
                }
            }
            // Account not found or error
            if json["result"]["error"].as_str() == Some("actNotFound") {
                return Ok("0.0000".into());
            }
            Ok("0.0000".into())
        }
        Err(_) => Ok("0.0000".into()),
    }
}

/// Get account sequence number (needed for TX construction)
pub async fn get_account_sequence(address: &str, rpc_url: &str) -> Result<u32, String> {
    let body = serde_json::json!({
        "method": "account_info",
        "params": [{
            "account": address,
            "ledger_index": "current"
        }]
    }).to_string();

    let json = post_json(rpc_url, &body).await?;

    json["result"]["account_data"]["Sequence"]
        .as_u64()
        .map(|s| s as u32)
        .ok_or_else(|| {
            if json["result"]["error"].as_str() == Some("actNotFound") {
                "Account non attivato (serve almeno 10 XRP)".into()
            } else {
                "Impossibile leggere sequence number".into()
            }
        })
}

/// Submit a signed transaction (hex-encoded tx_blob)
pub async fn submit_tx(tx_hex: &str, rpc_url: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "method": "submit",
        "params": [{
            "tx_blob": tx_hex
        }]
    }).to_string();

    let json = post_json(rpc_url, &body).await?;

    let engine_result = json["result"]["engine_result"]
        .as_str()
        .unwrap_or("unknown");

    if engine_result == "tesSUCCESS" || engine_result.starts_with("tes") {
        // Success — return tx hash
        let hash = json["result"]["tx_json"]["hash"]
            .as_str()
            .unwrap_or("");
        Ok(hash.to_string())
    } else {
        let msg = json["result"]["engine_result_message"]
            .as_str()
            .unwrap_or("Errore sconosciuto");
        Err(format!("Submit fallito: {} — {}", engine_result, msg))
    }
}

/// Get current fee (in drops)
pub async fn get_fee(rpc_url: &str) -> Result<u64, String> {
    let body = serde_json::json!({
        "method": "fee",
        "params": [{}]
    }).to_string();

    let json = post_json(rpc_url, &body).await?;

    json["result"]["drops"]["open_ledger_fee"]
        .as_str()
        .and_then(|s| s.parse::<u64>().ok())
        .or_else(|| {
            json["result"]["drops"]["base_fee"]
                .as_str()
                .and_then(|s| s.parse::<u64>().ok())
        })
        .ok_or_else(|| "Impossibile leggere fee".into())
}

/// Format drops to XRP string (e.g. 1000000 → "1.0000")
fn format_drops(drops: u64) -> String {
    let xrp = drops / 1_000_000;
    let frac = drops % 1_000_000;
    let frac_str = format!("{:06}", frac);
    format!("{}.{}", xrp, &frac_str[..4])
}
