// Rusby Wallet — Jetton token definitions for TON
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use super::Token;

/// Jetton transfer operation code
pub const JETTON_TRANSFER_OP: u32 = 0x0f8a7ea5;

/// Amount of TON to forward with jetton transfer (0.05 TON)
pub const FORWARD_TON_AMOUNT: u64 = 50_000_000;

/// Gas amount for jetton transfer (0.1 TON)
pub const JETTON_GAS_AMOUNT: u64 = 100_000_000;

/// Default Jetton tokens on TON mainnet
pub fn default_tokens() -> Vec<Token> {
    vec![
        Token {
            address: "EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs".into(),
            symbol: "USDT".into(),
            name: "Tether USD".into(),
            decimals: 6,
            chain_id: "ton".into(),
        },
        Token {
            address: "EQAvlWFDxGF2lXm67y4yzC17wYKD9A0guwPkMs1gOsM__NOT".into(),
            symbol: "NOT".into(),
            name: "Notcoin".into(),
            decimals: 9,
            chain_id: "ton".into(),
        },
        Token {
            address: "EQCvxJy4eG8hyHBFsZ7eePxrRsUQSFE_jpptRAYBmcG_DOGS".into(),
            symbol: "DOGS".into(),
            name: "Dogs".into(),
            decimals: 9,
            chain_id: "ton".into(),
        },
        Token {
            address: "EQBlqsm144Dq6SjbPI4jjZvlmHDr2y_R56Ny0i0gSfP_SCALE".into(),
            symbol: "SCALE".into(),
            name: "Scale".into(),
            decimals: 9,
            chain_id: "ton".into(),
        },
    ]
}

/// Find a jetton token by its master contract address
pub fn find_by_address(address: &str) -> Option<Token> {
    default_tokens().into_iter().find(|t| t.address == address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tokens() {
        let tokens = default_tokens();
        assert!(tokens.len() >= 4);
        assert!(tokens.iter().all(|t| t.chain_id == "ton"));
        assert!(tokens.iter().any(|t| t.symbol == "USDT"));
        assert!(tokens.iter().any(|t| t.symbol == "NOT"));
    }

    #[test]
    fn test_find_by_address() {
        let usdt = find_by_address("EQCxE6mUtQJKFnGfaROTKOt1lZbDiiX1kCixRv7Nw2Id_sDs");
        assert!(usdt.is_some());
        assert_eq!(usdt.unwrap().symbol, "USDT");

        let unknown = find_by_address("EQUnknown");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_constants() {
        assert_eq!(JETTON_TRANSFER_OP, 0x0f8a7ea5);
        assert_eq!(FORWARD_TON_AMOUNT, 50_000_000);
        assert_eq!(JETTON_GAS_AMOUNT, 100_000_000);
    }
}
