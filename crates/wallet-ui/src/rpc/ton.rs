// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{get_json, post_json};

/// Get TON balance via toncenter API
pub async fn get_balance(address: &str, rpc_url: &str) -> Result<String, String> {
    // toncenter REST endpoint
    let base_url = rpc_url.trim_end_matches("/jsonRPC");
    let url = format!("{}/getAddressBalance?address={}", base_url, address);

    let json = get_json(&url).await?;

    let balance_str = json["result"].as_str()
        .ok_or("Missing result in getAddressBalance")?;

    let nanoton: u64 = balance_str.parse()
        .map_err(|_| "Invalid balance number")?;

    Ok(format_nanoton(nanoton))
}

/// Get wallet seqno
pub async fn get_seqno(address: &str, rpc_url: &str) -> Result<u32, String> {
    let body = serde_json::json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "runGetMethod",
        "params": {
            "address": address,
            "method": "seqno",
            "stack": []
        }
    }).to_string();

    let json = post_json(rpc_url, &body).await?;

    let stack = json["result"]["stack"].as_array()
        .ok_or("Missing stack in seqno response")?;

    if let Some(first) = stack.first() {
        if let Some(val) = first.get(1).and_then(|v| v.as_str()) {
            let hex = val.strip_prefix("0x").unwrap_or(val);
            return u32::from_str_radix(hex, 16)
                .map_err(|e| format!("Parse seqno error: {}", e));
        }
    }

    Ok(0)
}

/// Send BOC (Bag of Cells) to toncenter
pub async fn send_boc(boc_b64: &str, rpc_url: &str) -> Result<String, String> {
    let body = serde_json::json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "sendBoc",
        "params": {
            "boc": boc_b64
        }
    }).to_string();

    let json = post_json(rpc_url, &body).await?;

    if let Some(error) = json.get("error") {
        return Err(format!("RPC error: {}", error));
    }

    Ok(json["result"].to_string())
}

fn format_nanoton(nanoton: u64) -> String {
    let ton = nanoton / 1_000_000_000;
    let frac = nanoton % 1_000_000_000;
    let frac_str = format!("{:09}", frac);
    format!("{}.{}", ton, &frac_str[..4])
}
