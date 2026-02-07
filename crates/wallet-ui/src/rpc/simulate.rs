// Rusby Wallet — TX simulation via eth_call
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use super::post_json;

/// Result of a TX simulation
#[derive(Debug, Clone)]
pub struct SimulationResult {
    pub success: bool,
    pub return_data: String,
    pub error: Option<String>,
}

/// Simulate an EVM transaction using eth_call
pub async fn simulate_evm_tx(
    rpc_url: &str,
    from: &str,
    to: &str,
    value: &str,
    data: &str,
) -> Result<SimulationResult, String> {
    // Build params using serde_json to prevent JSON injection
    let mut call_obj = serde_json::Map::new();
    call_obj.insert("from".to_string(), serde_json::Value::String(from.to_string()));
    call_obj.insert("to".to_string(), serde_json::Value::String(to.to_string()));

    if !value.is_empty() && value != "0" && value != "0x0" {
        call_obj.insert("value".to_string(), serde_json::Value::String(value.to_string()));
    }

    if !data.is_empty() {
        call_obj.insert("data".to_string(), serde_json::Value::String(data.to_string()));
    }

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [call_obj, "latest"],
        "id": 1
    }).to_string();

    let json = post_json(rpc_url, &body).await?;

    if let Some(result) = json.get("result").and_then(|r| r.as_str()) {
        Ok(SimulationResult {
            success: true,
            return_data: result.to_string(),
            error: None,
        })
    } else if let Some(error) = json.get("error") {
        let message = error.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown error");
        Ok(SimulationResult {
            success: false,
            return_data: String::new(),
            error: Some(message.to_string()),
        })
    } else {
        Ok(SimulationResult {
            success: false,
            return_data: String::new(),
            error: Some("Risposta RPC non valida".into()),
        })
    }
}

/// Decode common revert reasons from return data
pub fn decode_revert_reason(data: &str) -> Option<String> {
    let hex = data.strip_prefix("0x")?;
    if hex.len() < 8 { return None; }

    // Error(string) selector: 0x08c379a0
    if hex.starts_with("08c379a0") && hex.len() > 136 {
        // offset (32) + length (32) + string data
        let len_hex = &hex[72..136];
        let len = usize::from_str_radix(len_hex, 16).ok()?;
        let str_hex = &hex[136..136 + len * 2];
        let bytes = hex::decode(str_hex).ok()?;
        return String::from_utf8(bytes).ok();
    }

    None
}
