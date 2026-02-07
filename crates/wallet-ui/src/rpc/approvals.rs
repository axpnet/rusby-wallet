// Rusby Wallet — Token approval RPC queries
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use super::post_json;
use wallet_core::tokens::erc20;

/// Known spender contracts (DEX routers, etc.)
pub struct KnownSpender {
    pub address: String,
    pub name: String,
    pub chain: String,
}

pub fn known_spenders() -> Vec<KnownSpender> {
    vec![
        // Ethereum
        KnownSpender { address: "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45".into(), name: "Uniswap V3 Router".into(), chain: "ethereum".into() },
        KnownSpender { address: "0xEf1c6E67703c7BD7107eed8303Fbe6EC2554BF6B".into(), name: "Uniswap Universal Router".into(), chain: "ethereum".into() },
        KnownSpender { address: "0x1111111254EEB25477B68fb85Ed929f73A960582".into(), name: "1inch V5 Router".into(), chain: "ethereum".into() },
        KnownSpender { address: "0xDef1C0ded9bec7F1a1670819833240f027b25EfF".into(), name: "0x Exchange Proxy".into(), chain: "ethereum".into() },
        KnownSpender { address: "0xd9e1cE17f2641f24aE83637ab66a2cca9C378B9F".into(), name: "SushiSwap Router".into(), chain: "ethereum".into() },
        // Polygon
        KnownSpender { address: "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45".into(), name: "Uniswap V3 Router".into(), chain: "polygon".into() },
        KnownSpender { address: "0x1111111254EEB25477B68fb85Ed929f73A960582".into(), name: "1inch V5 Router".into(), chain: "polygon".into() },
        // BSC
        KnownSpender { address: "0x10ED43C718714eb63d5aA57B78B54704E256024E".into(), name: "PancakeSwap Router".into(), chain: "bsc".into() },
        KnownSpender { address: "0x1111111254EEB25477B68fb85Ed929f73A960582".into(), name: "1inch V5 Router".into(), chain: "bsc".into() },
        // Arbitrum
        KnownSpender { address: "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45".into(), name: "Uniswap V3 Router".into(), chain: "arbitrum".into() },
        // Base
        KnownSpender { address: "0x2626664c2603336E57B271c5C0b26F421741e481".into(), name: "Uniswap V3 Router".into(), chain: "base".into() },
    ]
}

/// Approval info for display
#[derive(Debug, Clone)]
pub struct ApprovalInfo {
    pub token_address: String,
    pub token_symbol: String,
    pub spender_address: String,
    pub spender_name: String,
    pub allowance: String,
}

/// Get token allowance for a specific owner/spender pair
pub async fn get_token_allowance(
    rpc_url: &str,
    token_address: &str,
    owner: &str,
    spender: &str,
) -> Result<String, String> {
    let calldata = erc20::encode_allowance(owner, spender)?;
    let hex_data = format!("0x{}", hex::encode(&calldata));

    let body = format!(
        r#"{{"jsonrpc":"2.0","method":"eth_call","params":[{{"to":"{}","data":"{}"}},"latest"],"id":1}}"#,
        token_address, hex_data
    );

    let json = post_json(rpc_url, &body).await?;

    if let Some(result) = json.get("result").and_then(|r| r.as_str()) {
        Ok(result.to_string())
    } else {
        Err("Failed to get allowance".into())
    }
}

/// Check all known spenders for a token and return active approvals
pub async fn check_approvals_for_token(
    rpc_url: &str,
    token_address: &str,
    token_symbol: &str,
    owner: &str,
    chain: &str,
) -> Vec<ApprovalInfo> {
    let spenders: Vec<_> = known_spenders().into_iter()
        .filter(|s| s.chain == chain)
        .collect();

    let mut approvals = Vec::new();

    for spender in spenders {
        if let Ok(allowance_hex) = get_token_allowance(rpc_url, token_address, owner, &spender.address).await {
            let hex = allowance_hex.trim_start_matches("0x");
            let value = u128::from_str_radix(hex, 16).unwrap_or(0);
            if value > 0 {
                let allowance_display = if value >= u128::MAX / 2 {
                    "Unlimited".to_string()
                } else {
                    format!("{}", value)
                };
                approvals.push(ApprovalInfo {
                    token_address: token_address.to_string(),
                    token_symbol: token_symbol.to_string(),
                    spender_address: spender.address,
                    spender_name: spender.name,
                    allowance: allowance_display,
                });
            }
        }
    }

    approvals
}
