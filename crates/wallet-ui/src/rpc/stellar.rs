// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// rpc/stellar: Balance query, account info, and TX submission
// Uses Horizon REST API

use super::get_json;

/// Get Stellar balance in XLM (formatted string)
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String, String> {
    let url = format!("{}/accounts/{}", rpc_url, address);
    let json = get_json(&url).await;

    match json {
        Ok(json) => {
            // Find native balance in balances array
            if let Some(balances) = json["balances"].as_array() {
                for bal in balances {
                    if bal["asset_type"].as_str() == Some("native") {
                        if let Some(balance_str) = bal["balance"].as_str() {
                            // Horizon returns balance as a string like "100.0000000"
                            return Ok(format_xlm_balance(balance_str));
                        }
                    }
                }
            }
            Ok("0.0000".into())
        }
        Err(_) => {
            // Account not found (not funded) returns 404
            Ok("0.0000".into())
        }
    }
}

/// Get account sequence number (needed for TX construction)
pub async fn get_account_sequence(address: &str, rpc_url: &str) -> Result<i64, String> {
    let url = format!("{}/accounts/{}", rpc_url, address);
    let json = get_json(&url).await?;

    json["sequence"]
        .as_str()
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| "Impossibile leggere sequence number".into())
}

/// Submit a signed transaction (XDR base64 encoded)
pub async fn submit_tx(tx_xdr_base64: &str, rpc_url: &str) -> Result<String, String> {
    use gloo_net::http::Request;

    let url = format!("{}/transactions", rpc_url);
    let body = format!("tx={}", url_encode(tx_xdr_base64));

    let response = Request::post(&url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(&body)
        .map_err(|e| format!("Errore richiesta: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Errore rete: {}", e))?;

    let text = response.text().await
        .map_err(|e| format!("Errore lettura risposta: {}", e))?;

    if response.status() == 200 {
        // Parse response to extract hash
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(hash) = json["hash"].as_str() {
                return Ok(hash.to_string());
            }
        }
        Ok(text)
    } else {
        Err(format!("Submit fallito: {}", text))
    }
}

/// Format XLM balance string from Horizon (e.g. "100.0000000" → "100.0000")
fn format_xlm_balance(balance: &str) -> String {
    // Horizon returns up to 7 decimal places, we show 4
    if let Some(dot) = balance.find('.') {
        let integer = &balance[..dot];
        let decimals = &balance[dot + 1..];
        let padded = format!("{:0<4}", decimals);
        format!("{}.{}", integer, &padded[..4])
    } else {
        format!("{}.0000", balance)
    }
}

/// Simple URL encoding for base64 (only encodes +, /, =)
fn url_encode(s: &str) -> String {
    s.replace('+', "%2B")
        .replace('/', "%2F")
        .replace('=', "%3D")
}
