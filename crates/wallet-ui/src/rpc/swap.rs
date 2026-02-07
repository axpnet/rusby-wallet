// Rusby Wallet — Swap RPC client (0x API)
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use wallet_core::swap::{SwapQuote, SwapSource, SwapParams, zeroex_base_url};
use crate::state::load_from_storage;

/// Transaction data returned by 0x quote endpoint
#[derive(Debug, Clone)]
pub struct SwapTxData {
    pub to: String,
    pub data: String,
    pub value: String,
    pub gas_limit: u64,
}

/// Fetch swap price estimate (no TX data, cheaper call)
pub async fn get_swap_price(params: &SwapParams, api_key: &str) -> Result<SwapQuote, String> {
    let base_url = zeroex_base_url_for_chain(params.chain_id)?;

    let url = format!(
        "{}/swap/v1/price?sellToken={}&buyToken={}&sellAmount={}&takerAddress={}&slippagePercentage={}",
        base_url,
        params.sell_token,
        params.buy_token,
        params.sell_amount,
        params.taker_address,
        params.slippage_bps as f64 / 10000.0,
    );

    let json = get_json_with_key(&url, api_key).await?;
    parse_quote_from_json(&json)
}

/// Fetch swap quote with TX data (for execution)
pub async fn get_swap_quote(params: &SwapParams, api_key: &str) -> Result<(SwapQuote, SwapTxData), String> {
    let base_url = zeroex_base_url_for_chain(params.chain_id)?;

    let url = format!(
        "{}/swap/v1/quote?sellToken={}&buyToken={}&sellAmount={}&takerAddress={}&slippagePercentage={}",
        base_url,
        params.sell_token,
        params.buy_token,
        params.sell_amount,
        params.taker_address,
        params.slippage_bps as f64 / 10000.0,
    );

    let json = get_json_with_key(&url, api_key).await?;

    let quote = parse_quote_from_json(&json)?;

    let to = json.get("to")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'to' in quote response")?
        .to_string();

    let data = json.get("data")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'data' in quote response")?
        .to_string();

    let value = json.get("value")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string();

    let gas_limit = json.get("gas")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u64>().ok())
        .or_else(|| json.get("estimatedGas")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok()))
        .unwrap_or(300_000);

    let tx_data = SwapTxData {
        to,
        data,
        value,
        gas_limit,
    };

    Ok((quote, tx_data))
}

/// Load API key from storage
pub fn get_api_key() -> Option<String> {
    let key = load_from_storage("zeroex_api_key")?;
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

/// Resolve base URL from numeric chain ID
fn zeroex_base_url_for_chain(chain_id: u64) -> Result<&'static str, String> {
    let chain_str = match chain_id {
        1 => "ethereum",
        137 => "polygon",
        56 => "bsc",
        42161 => "arbitrum",
        8453 => "base",
        10 => "optimism",
        _ => return Err(format!("Unsupported chain ID for swap: {}", chain_id)),
    };
    zeroex_base_url(chain_str)
        .ok_or_else(|| format!("No 0x API URL for chain {}", chain_str))
}

/// GET request with 0x API key header
async fn get_json_with_key(url: &str, api_key: &str) -> Result<serde_json::Value, String> {
    let resp = gloo_net::http::Request::get(url)
        .header("0x-api-key", api_key)
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if resp.status() != 200 {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("0x API error {}: {}", status, body));
    }

    let text = resp.text().await
        .map_err(|e| format!("Response read error: {}", e))?;

    serde_json::from_str(&text)
        .map_err(|e| format!("JSON parse error: {}", e))
}

/// Parse SwapQuote from 0x API JSON response
fn parse_quote_from_json(json: &serde_json::Value) -> Result<SwapQuote, String> {
    let sell_token = json.get("sellTokenAddress")
        .or_else(|| json.get("sellToken"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let buy_token = json.get("buyTokenAddress")
        .or_else(|| json.get("buyToken"))
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let sell_amount = json.get("sellAmount")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string();

    let buy_amount = json.get("buyAmount")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string();

    let price = json.get("price")
        .and_then(|v| v.as_str())
        .unwrap_or("0")
        .to_string();

    let estimated_gas = json.get("estimatedGas")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<u64>().ok())
        .or_else(|| json.get("gas")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<u64>().ok()))
        .unwrap_or(0);

    let allowance_target = json.get("allowanceTarget")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    // Parse sources
    let sources = json.get("sources")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter().filter_map(|s| {
                let name = s.get("name").and_then(|v| v.as_str())?;
                let proportion = s.get("proportion")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0");
                // Skip zero-proportion sources
                if proportion == "0" {
                    return None;
                }
                Some(SwapSource {
                    name: name.to_string(),
                    proportion: proportion.to_string(),
                })
            }).collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(SwapQuote {
        sell_token,
        buy_token,
        sell_amount,
        buy_amount,
        price,
        estimated_gas,
        sources,
        allowance_target,
    })
}
