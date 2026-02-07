// Rusby Wallet — Pure Rust multi-chain crypto wallet
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// tokens/spl: SPL Token definitions and encoding for Solana

use super::Token;

/// Default SPL tokens
pub fn default_tokens() -> Vec<Token> {
    vec![
        Token { address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".into(), symbol: "USDC".into(), name: "USD Coin".into(), decimals: 6, chain_id: "solana".into() },
        Token { address: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB".into(), symbol: "USDT".into(), name: "Tether USD".into(), decimals: 6, chain_id: "solana".into() },
        Token { address: "So11111111111111111111111111111111111111112".into(), symbol: "WSOL".into(), name: "Wrapped SOL".into(), decimals: 9, chain_id: "solana".into() },
        Token { address: "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN".into(), symbol: "JUP".into(), name: "Jupiter".into(), decimals: 6, chain_id: "solana".into() },
    ]
}

/// SPL Token Program ID
pub const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

/// Associated Token Account Program ID
pub const ATA_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
