// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// rpc/prices: Fetch USD prices from CoinGecko API

use std::collections::HashMap;

const COINGECKO_API: &str = "https://api.coingecko.com/api/v3/simple/price";

/// Map chain_id → CoinGecko coin ID
fn chain_to_coingecko_id(chain_id: &str) -> Option<&'static str> {
    match chain_id {
        "ethereum" => Some("ethereum"),
        "polygon" => Some("matic-network"),
        "bsc" => Some("binancecoin"),
        "optimism" | "base" | "arbitrum" => Some("ethereum"),
        "solana" => Some("solana"),
        "ton" => Some("the-open-network"),
        "cosmos" => Some("cosmos"),
        "osmosis" => Some("osmosis"),
        "bitcoin" => Some("bitcoin"),
        "litecoin" => Some("litecoin"),
        "stellar" => Some("stellar"),
        "ripple" => Some("ripple"),
        "dogecoin" => Some("dogecoin"),
        "tron" => Some("tron"),
        _ => None,
    }
}

/// Fetch USD prices for all supported chains
/// Returns HashMap<chain_id, usd_price>
pub async fn fetch_prices() -> Result<HashMap<String, f64>, String> {
    let coingecko_ids = "ethereum,matic-network,binancecoin,solana,the-open-network,cosmos,osmosis,bitcoin,litecoin,stellar,ripple,dogecoin,tron";
    let url = format!("{}?ids={}&vs_currencies=usd", COINGECKO_API, coingecko_ids);

    let json = super::get_json(&url).await?;

    let mut prices: HashMap<String, f64> = HashMap::new();

    let chain_ids = [
        "ethereum", "polygon", "bsc", "optimism", "base", "arbitrum",
        "solana", "ton", "cosmos", "osmosis", "bitcoin", "litecoin", "stellar", "ripple",
        "dogecoin", "tron",
    ];

    for chain_id in chain_ids {
        if let Some(cg_id) = chain_to_coingecko_id(chain_id) {
            if let Some(usd) = json.get(cg_id).and_then(|v| v.get("usd")).and_then(|v| v.as_f64()) {
                prices.insert(chain_id.to_string(), usd);
            }
        }
    }

    Ok(prices)
}
