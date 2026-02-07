// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// rpc/spl: Fetch SPL token balances via Solana JSON-RPC

use wallet_core::tokens::{Token, TokenBalance};
use wallet_core::tokens::spl;

/// Fetch all SPL token balances for a wallet
pub async fn get_all_token_balances(
    owner: &str,
    rpc_url: &str,
) -> Vec<TokenBalance> {
    let body = format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"getTokenAccountsByOwner","params":["{}",{{"programId":"{}"}},{{"encoding":"jsonParsed"}}]}}"#,
        owner, spl::TOKEN_PROGRAM_ID
    );

    let json = match super::post_json(rpc_url, &body).await {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();
    let known_tokens = spl::default_tokens();

    if let Some(value) = json.get("result").and_then(|r| r.get("value")).and_then(|v| v.as_array()) {
        for account in value {
            let parsed = match account
                .get("account")
                .and_then(|a| a.get("data"))
                .and_then(|d| d.get("parsed"))
                .and_then(|p| p.get("info"))
            {
                Some(info) => info,
                None => continue,
            };

            let mint = parsed.get("mint").and_then(|m| m.as_str()).unwrap_or("");
            let amount_str = parsed
                .get("tokenAmount")
                .and_then(|ta| ta.get("uiAmountString"))
                .and_then(|a| a.as_str())
                .unwrap_or("0");
            let decimals = parsed
                .get("tokenAmount")
                .and_then(|ta| ta.get("decimals"))
                .and_then(|d| d.as_u64())
                .unwrap_or(0) as u8;

            let balance_f: f64 = amount_str.parse().unwrap_or(0.0);
            if balance_f == 0.0 {
                continue;
            }

            // Match against known tokens, or use mint as symbol
            let token = known_tokens.iter()
                .find(|t| t.address == mint)
                .cloned()
                .unwrap_or(Token {
                    address: mint.to_string(),
                    symbol: format!("{}...", &mint[..6]),
                    name: "Unknown Token".to_string(),
                    decimals,
                    chain_id: "solana".to_string(),
                });

            results.push(TokenBalance {
                token,
                balance: format!("{:.4}", balance_f),
                balance_usd: 0.0,
            });
        }
    }

    results
}
