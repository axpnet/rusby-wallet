// Rusby Wallet — Pure Rust multi-chain crypto wallet
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// tokens: Token definitions and ABI encoding for ERC-20, SPL, etc.

pub mod erc20;
pub mod spl;
pub mod cw20;
pub mod jetton;

use serde::{Deserialize, Serialize};

/// A token definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub chain_id: String,
}

/// Token balance (token + formatted balance)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub token: Token,
    pub balance: String,
    pub balance_usd: f64,
}
