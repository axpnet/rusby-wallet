// Rusby Wallet — CW-20 token RPC client for Cosmos/Osmosis
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use super::get_json;
use wallet_core::tokens::{Token, TokenBalance};
use wallet_core::tokens::cw20;

/// Get CW-20 token balance via CosmWasm smart query
pub async fn get_token_balance(owner: &str, token: &Token, rest_url: &str) -> Result<String, String> {
    let query_b64 = cw20::encode_balance_query(owner)?;
    let url = format!(
        "{}/cosmwasm/wasm/v1/contract/{}/smart/{}",
        rest_url, token.address, query_b64
    );

    let json = get_json(&url).await?;

    let balance = json["data"]["balance"].as_str()
        .ok_or("Missing balance in CW-20 response")?;

    let raw: u128 = balance.parse().unwrap_or(0);
    Ok(cw20::format_token_amount(raw, token.decimals))
}

/// Get all CW-20 token balances for a chain
pub async fn get_all_token_balances(owner: &str, chain_id: &str, rpc_url: &str) -> Vec<TokenBalance> {
    let rest_url = super::cosmos::rpc_url_to_rest(rpc_url);
    let tokens = cw20::tokens_for_chain(chain_id);
    let mut result = Vec::new();

    for token in tokens {
        match get_token_balance(owner, &token, &rest_url).await {
            Ok(balance) => {
                // Skip zero balances
                if balance.starts_with("0.0000") {
                    continue;
                }
                result.push(TokenBalance {
                    token,
                    balance,
                    balance_usd: 0.0,
                });
            }
            Err(_) => {
                // Skip tokens that fail to query (contract may not exist)
                continue;
            }
        }
    }

    result
}
