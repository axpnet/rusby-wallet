// Rusby Wallet — Pure Rust multi-chain crypto wallet
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// tokens/erc20: ERC-20 ABI encoding and default token list

use super::Token;

/// Default ERC-20 tokens per chain
pub fn default_tokens() -> Vec<Token> {
    vec![
        // Ethereum
        Token { address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".into(), symbol: "USDT".into(), name: "Tether USD".into(), decimals: 6, chain_id: "ethereum".into() },
        Token { address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".into(), symbol: "USDC".into(), name: "USD Coin".into(), decimals: 6, chain_id: "ethereum".into() },
        Token { address: "0x6B175474E89094C44Da98b954EedeAC495271d0F".into(), symbol: "DAI".into(), name: "Dai Stablecoin".into(), decimals: 18, chain_id: "ethereum".into() },
        Token { address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".into(), symbol: "WETH".into(), name: "Wrapped Ether".into(), decimals: 18, chain_id: "ethereum".into() },
        Token { address: "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599".into(), symbol: "WBTC".into(), name: "Wrapped Bitcoin".into(), decimals: 8, chain_id: "ethereum".into() },
        // Polygon
        Token { address: "0xc2132D05D31c914a87C6611C10748AEb04B58e8F".into(), symbol: "USDT".into(), name: "Tether USD".into(), decimals: 6, chain_id: "polygon".into() },
        Token { address: "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359".into(), symbol: "USDC".into(), name: "USD Coin".into(), decimals: 6, chain_id: "polygon".into() },
        // BSC
        Token { address: "0x55d398326f99059fF775485246999027B3197955".into(), symbol: "USDT".into(), name: "Tether USD".into(), decimals: 18, chain_id: "bsc".into() },
        Token { address: "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d".into(), symbol: "USDC".into(), name: "USD Coin".into(), decimals: 18, chain_id: "bsc".into() },
        // Arbitrum
        Token { address: "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9".into(), symbol: "USDT".into(), name: "Tether USD".into(), decimals: 6, chain_id: "arbitrum".into() },
        Token { address: "0xaf88d065e77c8cC2239327C5EDb3A432268e5831".into(), symbol: "USDC".into(), name: "USD Coin".into(), decimals: 6, chain_id: "arbitrum".into() },
        // Base
        Token { address: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".into(), symbol: "USDC".into(), name: "USD Coin".into(), decimals: 6, chain_id: "base".into() },
        // Optimism
        Token { address: "0x0b2C639c533813f4Aa9D7837CAf62653d097Ff85".into(), symbol: "USDC".into(), name: "USD Coin".into(), decimals: 6, chain_id: "optimism".into() },
        Token { address: "0x94b008aA00579c1307B0EF2c499aD98a8ce58e58".into(), symbol: "USDT".into(), name: "Tether USD".into(), decimals: 6, chain_id: "optimism".into() },
    ]
}

/// Get default tokens for a specific chain
pub fn tokens_for_chain(chain_id: &str) -> Vec<Token> {
    default_tokens().into_iter().filter(|t| t.chain_id == chain_id).collect()
}

/// Build `balanceOf(address)` calldata
/// Selector: 0x70a08231
pub fn encode_balance_of(owner: &str) -> Result<Vec<u8>, String> {
    let owner_bytes = parse_address(owner)?;
    let mut data = vec![0x70, 0xa0, 0x82, 0x31]; // selector
    // Pad address to 32 bytes (left-padded with zeros)
    data.extend_from_slice(&[0u8; 12]);
    data.extend_from_slice(&owner_bytes);
    Ok(data)
}

/// Build `transfer(address,uint256)` calldata
/// Selector: 0xa9059cbb
pub fn encode_transfer(to: &str, amount: &str, decimals: u8) -> Result<Vec<u8>, String> {
    let to_bytes = parse_address(to)?;
    let amount_raw = parse_token_amount(amount, decimals)?;

    let mut data = vec![0xa9, 0x05, 0x9c, 0xbb]; // selector
    // Pad 'to' address to 32 bytes
    data.extend_from_slice(&[0u8; 12]);
    data.extend_from_slice(&to_bytes);
    // Pad amount to 32 bytes (big-endian)
    let amount_bytes = amount_raw.to_be_bytes();
    data.extend_from_slice(&[0u8; 16]); // 32 - 16 = 16 leading zeros for u128
    data.extend_from_slice(&amount_bytes);
    Ok(data)
}

/// Build `allowance(address,address)` calldata
/// Selector: 0xdd62ed3e
pub fn encode_allowance(owner: &str, spender: &str) -> Result<Vec<u8>, String> {
    let owner_bytes = parse_address(owner)?;
    let spender_bytes = parse_address(spender)?;
    let mut data = vec![0xdd, 0x62, 0xed, 0x3e]; // selector
    data.extend_from_slice(&[0u8; 12]);
    data.extend_from_slice(&owner_bytes);
    data.extend_from_slice(&[0u8; 12]);
    data.extend_from_slice(&spender_bytes);
    Ok(data)
}

/// Build `approve(address,uint256)` calldata with amount=0 (revoke)
/// Selector: 0x095ea7b3
pub fn encode_revoke(spender: &str) -> Result<Vec<u8>, String> {
    let spender_bytes = parse_address(spender)?;
    let mut data = vec![0x09, 0x5e, 0xa7, 0xb3]; // selector
    data.extend_from_slice(&[0u8; 12]);
    data.extend_from_slice(&spender_bytes);
    data.extend_from_slice(&[0u8; 32]); // amount = 0
    Ok(data)
}

/// Decode a uint256 hex result to a formatted balance string
pub fn decode_balance(hex_result: &str, decimals: u8) -> String {
    let hex = hex_result.trim_start_matches("0x");
    let raw = u128::from_str_radix(hex, 16).unwrap_or(0);
    format_token_amount(raw, decimals)
}

/// Parse a decimal amount string to raw token units (u128)
pub fn parse_token_amount(amount: &str, decimals: u8) -> Result<u128, String> {
    let parts: Vec<&str> = amount.split('.').collect();
    let integer_part: u128 = parts[0].parse().map_err(|_| "Invalid amount")?;
    let decimal_part: u128 = if parts.len() > 1 {
        let dec_str = parts[1];
        let padded = format!("{:0<width$}", dec_str, width = decimals as usize);
        padded[..decimals as usize].parse().map_err(|_| "Invalid decimals")?
    } else {
        0
    };
    let multiplier = 10u128.pow(decimals as u32);
    let total = integer_part.checked_mul(multiplier)
        .ok_or("Amount overflow: value too large")?;
    total.checked_add(decimal_part)
        .ok_or_else(|| "Amount overflow: value too large".to_string())
}

/// Format raw token units to decimal string
fn format_token_amount(raw: u128, decimals: u8) -> String {
    let multiplier = 10u128.pow(decimals as u32);
    let integer = raw / multiplier;
    let fraction = raw % multiplier;
    let dec_str = format!("{:0>width$}", fraction, width = decimals as usize);
    // Show up to 4 decimal places
    let show = std::cmp::min(4, decimals as usize);
    format!("{}.{}", integer, &dec_str[..show])
}

fn parse_address(addr: &str) -> Result<[u8; 20], String> {
    let hex = addr.trim_start_matches("0x");
    if hex.len() != 40 {
        return Err("Invalid address length".into());
    }
    let bytes = hex::decode(hex).map_err(|_| "Invalid hex address")?;
    let mut result = [0u8; 20];
    result.copy_from_slice(&bytes);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_balance_of() {
        let data = encode_balance_of("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        assert_eq!(data.len(), 36); // 4 selector + 32 padded address
        assert_eq!(&data[..4], &[0x70, 0xa0, 0x82, 0x31]);
    }

    #[test]
    fn test_encode_transfer() {
        let data = encode_transfer("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", "100.5", 6).unwrap();
        assert_eq!(data.len(), 68); // 4 + 32 + 32
        assert_eq!(&data[..4], &[0xa9, 0x05, 0x9c, 0xbb]);
    }

    #[test]
    fn test_parse_token_amount() {
        assert_eq!(parse_token_amount("1.0", 6).unwrap(), 1_000_000);
        assert_eq!(parse_token_amount("100.5", 6).unwrap(), 100_500_000);
        assert_eq!(parse_token_amount("1.0", 18).unwrap(), 1_000_000_000_000_000_000);
    }

    #[test]
    fn test_encode_allowance() {
        let data = encode_allowance(
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            "0x1111111111111111111111111111111111111111",
        ).unwrap();
        assert_eq!(data.len(), 68); // 4 + 32 + 32
        assert_eq!(&data[..4], &[0xdd, 0x62, 0xed, 0x3e]);
    }

    #[test]
    fn test_encode_revoke() {
        let data = encode_revoke("0x1111111111111111111111111111111111111111").unwrap();
        assert_eq!(data.len(), 68); // 4 + 32 + 32
        assert_eq!(&data[..4], &[0x09, 0x5e, 0xa7, 0xb3]);
        // Last 32 bytes should be all zeros (amount = 0)
        assert!(data[36..68].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_decode_balance() {
        // 1 USDC = 1000000 (6 decimals) = 0xF4240
        let result = decode_balance("0x00000000000000000000000000000000000000000000000000000000000f4240", 6);
        assert_eq!(result, "1.0000");
    }
}
