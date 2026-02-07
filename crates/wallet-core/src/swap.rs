// Rusby Wallet — Swap types and helpers
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

/// Swap quote returned by aggregator API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuote {
    pub sell_token: String,
    pub buy_token: String,
    pub sell_amount: String,
    pub buy_amount: String,
    pub price: String,
    pub estimated_gas: u64,
    pub sources: Vec<SwapSource>,
    pub allowance_target: String,
}

/// DEX source contributing to a swap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapSource {
    pub name: String,
    pub proportion: String,
}

/// Parameters for requesting a swap quote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapParams {
    pub sell_token: String,
    pub buy_token: String,
    pub sell_amount: String,
    pub taker_address: String,
    pub slippage_bps: u16,
    pub chain_id: u64,
}

/// Common token for swap UI selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapToken {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub logo_char: char,
}

/// Native token address placeholder used by 0x API
pub const NATIVE_TOKEN_ADDRESS: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";

/// EVM chain numeric ID for 0x API
pub fn evm_chain_id(chain_id: &str) -> Option<u64> {
    match chain_id {
        "ethereum" => Some(1),
        "polygon" => Some(137),
        "bsc" => Some(56),
        "arbitrum" => Some(42161),
        "base" => Some(8453),
        "optimism" => Some(10),
        _ => None,
    }
}

/// 0x API base URL per chain
pub fn zeroex_base_url(chain_id: &str) -> Option<&'static str> {
    match chain_id {
        "ethereum" => Some("https://api.0x.org"),
        "polygon" => Some("https://polygon.api.0x.org"),
        "bsc" => Some("https://bsc.api.0x.org"),
        "arbitrum" => Some("https://arbitrum.api.0x.org"),
        "base" => Some("https://base.api.0x.org"),
        "optimism" => Some("https://optimism.api.0x.org"),
        _ => None,
    }
}

/// Common swap tokens per chain (native + popular ERC-20)
pub fn common_swap_tokens(chain_id: &str) -> Vec<SwapToken> {
    match chain_id {
        "ethereum" => vec![
            SwapToken { address: NATIVE_TOKEN_ADDRESS.into(), symbol: "ETH".into(), name: "Ether".into(), decimals: 18, logo_char: 'E' },
            SwapToken { address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".into(), symbol: "USDC".into(), name: "USD Coin".into(), decimals: 6, logo_char: '$' },
            SwapToken { address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".into(), symbol: "USDT".into(), name: "Tether USD".into(), decimals: 6, logo_char: '$' },
            SwapToken { address: "0x6B175474E89094C44Da98b954EedeAC495271d0F".into(), symbol: "DAI".into(), name: "Dai Stablecoin".into(), decimals: 18, logo_char: 'D' },
            SwapToken { address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".into(), symbol: "WETH".into(), name: "Wrapped Ether".into(), decimals: 18, logo_char: 'W' },
            SwapToken { address: "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".into(), symbol: "WBTC".into(), name: "Wrapped BTC".into(), decimals: 8, logo_char: 'B' },
        ],
        "polygon" => vec![
            SwapToken { address: NATIVE_TOKEN_ADDRESS.into(), symbol: "MATIC".into(), name: "Polygon".into(), decimals: 18, logo_char: 'P' },
            SwapToken { address: "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359".into(), symbol: "USDC".into(), name: "USD Coin".into(), decimals: 6, logo_char: '$' },
            SwapToken { address: "0xc2132D05D31c914a87C6611C10748AEb04B58e8F".into(), symbol: "USDT".into(), name: "Tether USD".into(), decimals: 6, logo_char: '$' },
            SwapToken { address: "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270".into(), symbol: "WMATIC".into(), name: "Wrapped MATIC".into(), decimals: 18, logo_char: 'W' },
            SwapToken { address: "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619".into(), symbol: "WETH".into(), name: "Wrapped Ether".into(), decimals: 18, logo_char: 'E' },
        ],
        "bsc" => vec![
            SwapToken { address: NATIVE_TOKEN_ADDRESS.into(), symbol: "BNB".into(), name: "BNB".into(), decimals: 18, logo_char: 'B' },
            SwapToken { address: "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d".into(), symbol: "USDC".into(), name: "USD Coin".into(), decimals: 18, logo_char: '$' },
            SwapToken { address: "0x55d398326f99059fF775485246999027B3197955".into(), symbol: "USDT".into(), name: "Tether USD".into(), decimals: 18, logo_char: '$' },
            SwapToken { address: "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c".into(), symbol: "WBNB".into(), name: "Wrapped BNB".into(), decimals: 18, logo_char: 'W' },
        ],
        "arbitrum" => vec![
            SwapToken { address: NATIVE_TOKEN_ADDRESS.into(), symbol: "ETH".into(), name: "Ether".into(), decimals: 18, logo_char: 'E' },
            SwapToken { address: "0xaf88d065e77c8cC2239327C5EDb3A432268e5831".into(), symbol: "USDC".into(), name: "USD Coin".into(), decimals: 6, logo_char: '$' },
            SwapToken { address: "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9".into(), symbol: "USDT".into(), name: "Tether USD".into(), decimals: 6, logo_char: '$' },
            SwapToken { address: "0x912CE59144191C1204E64559FE8253a0e49E6548".into(), symbol: "ARB".into(), name: "Arbitrum".into(), decimals: 18, logo_char: 'A' },
        ],
        "base" => vec![
            SwapToken { address: NATIVE_TOKEN_ADDRESS.into(), symbol: "ETH".into(), name: "Ether".into(), decimals: 18, logo_char: 'E' },
            SwapToken { address: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".into(), symbol: "USDC".into(), name: "USD Coin".into(), decimals: 6, logo_char: '$' },
            SwapToken { address: "0x4200000000000000000000000000000000000006".into(), symbol: "WETH".into(), name: "Wrapped Ether".into(), decimals: 18, logo_char: 'W' },
        ],
        "optimism" => vec![
            SwapToken { address: NATIVE_TOKEN_ADDRESS.into(), symbol: "ETH".into(), name: "Ether".into(), decimals: 18, logo_char: 'E' },
            SwapToken { address: "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85".into(), symbol: "USDC".into(), name: "USD Coin".into(), decimals: 6, logo_char: '$' },
            SwapToken { address: "0x4200000000000000000000000000000000000042".into(), symbol: "OP".into(), name: "Optimism".into(), decimals: 18, logo_char: 'O' },
        ],
        _ => vec![],
    }
}

/// Format swap amount from raw to human-readable
pub fn format_swap_amount(raw: &str, decimals: u8) -> String {
    if raw.is_empty() || raw == "0" {
        return "0".to_string();
    }

    let raw_padded = if raw.len() <= decimals as usize {
        let pad = decimals as usize - raw.len() + 1;
        format!("{}{}", "0".repeat(pad), raw)
    } else {
        raw.to_string()
    };

    let decimal_pos = raw_padded.len() - decimals as usize;
    let integer = &raw_padded[..decimal_pos];
    let fraction = &raw_padded[decimal_pos..];

    // Trim trailing zeros, keep at least 4 decimals
    let frac_trimmed = fraction.trim_end_matches('0');
    let frac_display = if frac_trimmed.len() < 4 {
        &fraction[..4.min(fraction.len())]
    } else {
        &fraction[..frac_trimmed.len().min(8)]
    };

    let int_display = if integer.is_empty() { "0" } else { integer };
    format!("{}.{}", int_display, frac_display)
}

/// Parse human-readable amount to raw (no decimals)
pub fn parse_swap_amount(amount: &str, decimals: u8) -> Result<String, String> {
    let amount = amount.trim();
    if amount.is_empty() {
        return Err("Empty amount".into());
    }

    let parts: Vec<&str> = amount.split('.').collect();
    if parts.len() > 2 {
        return Err("Invalid amount format".into());
    }

    let integer = parts[0];
    let fraction = if parts.len() == 2 { parts[1] } else { "" };

    if fraction.len() > decimals as usize {
        return Err(format!("Too many decimals (max {})", decimals));
    }

    let padded_fraction = format!("{:0<width$}", fraction, width = decimals as usize);
    let raw = format!("{}{}", integer, padded_fraction);

    // Remove leading zeros
    let trimmed = raw.trim_start_matches('0');
    if trimmed.is_empty() {
        Ok("0".into())
    } else {
        Ok(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evm_chain_id() {
        assert_eq!(evm_chain_id("ethereum"), Some(1));
        assert_eq!(evm_chain_id("polygon"), Some(137));
        assert_eq!(evm_chain_id("bsc"), Some(56));
        assert_eq!(evm_chain_id("solana"), None);
        assert_eq!(evm_chain_id("ton"), None);
    }

    #[test]
    fn test_zeroex_base_url() {
        assert!(zeroex_base_url("ethereum").is_some());
        assert!(zeroex_base_url("polygon").is_some());
        assert!(zeroex_base_url("solana").is_none());
    }

    #[test]
    fn test_common_swap_tokens_ethereum() {
        let tokens = common_swap_tokens("ethereum");
        assert!(tokens.len() >= 4);
        assert_eq!(tokens[0].symbol, "ETH");
        assert!(tokens.iter().any(|t| t.symbol == "USDC"));
    }

    #[test]
    fn test_common_swap_tokens_polygon() {
        let tokens = common_swap_tokens("polygon");
        assert!(!tokens.is_empty());
        assert_eq!(tokens[0].symbol, "MATIC");
    }

    #[test]
    fn test_common_swap_tokens_unknown() {
        let tokens = common_swap_tokens("cosmos");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_format_swap_amount() {
        assert_eq!(format_swap_amount("1000000", 6), "1.0000");
        assert_eq!(format_swap_amount("1500000000000000000", 18), "1.5000");
        assert_eq!(format_swap_amount("0", 6), "0");
    }

    #[test]
    fn test_parse_swap_amount() {
        assert_eq!(parse_swap_amount("1.5", 18).unwrap(), "1500000000000000000");
        assert_eq!(parse_swap_amount("1.0", 6).unwrap(), "1000000");
        assert_eq!(parse_swap_amount("100", 6).unwrap(), "100000000");
        assert!(parse_swap_amount("1.1234567", 6).is_err());
    }
}
