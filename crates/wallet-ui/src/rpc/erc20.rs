// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// rpc/erc20: Fetch ERC-20 token balances via eth_call

use wallet_core::tokens::erc20;
use wallet_core::tokens::{Token, TokenBalance};

/// Fetch balance of a single ERC-20 token
pub async fn get_token_balance(
    owner: &str,
    token: &Token,
    rpc_url: &str,
) -> Result<String, String> {
    let calldata = erc20::encode_balance_of(owner)?;
    let calldata_hex = format!("0x{}", hex::encode(&calldata));

    let body = format!(
        r#"{{"jsonrpc":"2.0","method":"eth_call","params":[{{"to":"{}","data":"{}"}},"latest"],"id":1}}"#,
        token.address, calldata_hex
    );

    let json = super::post_json(rpc_url, &body).await?;

    let result = json.get("result")
        .and_then(|v| v.as_str())
        .unwrap_or("0x0");

    Ok(erc20::decode_balance(result, token.decimals))
}

/// Fetch balances for all default tokens on a chain
pub async fn get_all_token_balances(
    owner: &str,
    chain_id: &str,
    rpc_url: &str,
) -> Vec<TokenBalance> {
    let tokens = erc20::tokens_for_chain(chain_id);
    let mut results = Vec::new();

    for token in tokens {
        let balance = match get_token_balance(owner, &token, rpc_url).await {
            Ok(b) => b,
            Err(_) => "0.0000".to_string(),
        };

        // Skip zero balances
        let balance_f: f64 = balance.parse().unwrap_or(0.0);
        if balance_f > 0.0 {
            results.push(TokenBalance {
                token,
                balance,
                balance_usd: 0.0,
            });
        }
    }

    results
}
