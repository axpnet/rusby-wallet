// Rusby Wallet — CW-20 token definitions and encoding for Cosmos/Osmosis
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use super::Token;

/// Default CW-20 tokens for Cosmos Hub and Osmosis
pub fn default_tokens() -> Vec<Token> {
    vec![
        // Osmosis CW-20 tokens
        Token {
            address: "osmo1z0sh4s80u99l6y9d3vfy582p8jejeeu6tcucs2wz5pf0t7vd67cqxqux0s".into(),
            symbol: "MARS".into(),
            name: "Mars Protocol".into(),
            decimals: 6,
            chain_id: "osmosis".into(),
        },
        Token {
            address: "osmo1g7grcg3gzmx36mtqy7p0fy9jqzrlk9xpgl60g2p7n2s7q8dj5wqgmj5ch".into(),
            symbol: "ION".into(),
            name: "Ion".into(),
            decimals: 6,
            chain_id: "osmosis".into(),
        },
    ]
}

/// Get CW-20 tokens for a specific chain
pub fn tokens_for_chain(chain_id: &str) -> Vec<Token> {
    default_tokens().into_iter()
        .filter(|t| t.chain_id == chain_id)
        .collect()
}

/// Validate that a string is safe for JSON inclusion (no injection characters)
fn validate_json_safe(s: &str) -> Result<(), String> {
    if s.contains('"') || s.contains('\\') || s.contains('\n') || s.contains('\r') {
        return Err("Input contains invalid characters".into());
    }
    Ok(())
}

/// Encode a CW-20 balance query as base64
/// Query: {"balance":{"address":"cosmos1..."}}
pub fn encode_balance_query(owner_address: &str) -> Result<String, String> {
    validate_json_safe(owner_address)?;
    let query = format!(r#"{{"balance":{{"address":"{}"}}}}"#, owner_address);
    Ok(base64_encode(query.as_bytes()))
}

/// Encode a CW-20 transfer message as JSON string
/// Returns: {"transfer":{"recipient":"cosmos1...","amount":"1000000"}}
pub fn encode_transfer_msg(recipient: &str, amount: &str, decimals: u8) -> Result<String, String> {
    validate_json_safe(recipient)?;
    let raw_amount = parse_token_amount(amount, decimals)?;
    Ok(format!(
        r#"{{"transfer":{{"recipient":"{}","amount":"{}"}}}}"#,
        recipient, raw_amount
    ))
}

/// Parse a human-readable token amount to raw integer
/// e.g. "1.5" with 6 decimals -> 1500000
pub fn parse_token_amount(amount: &str, decimals: u8) -> Result<u128, String> {
    let parts: Vec<&str> = amount.split('.').collect();
    let (integer_part, decimal_part) = match parts.len() {
        1 => (parts[0], ""),
        2 => (parts[0], parts[1]),
        _ => return Err("Invalid amount format".into()),
    };

    let integer: u128 = if integer_part.is_empty() {
        0
    } else {
        integer_part.parse().map_err(|_| "Invalid integer part")?
    };

    let dec = decimals as usize;
    let decimal: u128 = if decimal_part.is_empty() {
        0
    } else {
        let padded = format!("{:0<width$}", decimal_part, width = dec);
        let trimmed = &padded[..dec];
        trimmed.parse().map_err(|_| "Invalid decimal part")?
    };

    let multiplier = 10u128.pow(decimals as u32);
    let total = integer.checked_mul(multiplier)
        .ok_or("Amount overflow: value too large")?;
    total.checked_add(decimal)
        .ok_or_else(|| "Amount overflow: value too large".to_string())
}

/// Format raw token amount to human-readable
pub fn format_token_amount(raw: u128, decimals: u8) -> String {
    let divisor = 10u128.pow(decimals as u32);
    let integer = raw / divisor;
    let fraction = raw % divisor;
    let frac_str = format!("{:0>width$}", fraction, width = decimals as usize);
    let display_decimals = 4.min(decimals as usize);
    format!("{}.{}", integer, &frac_str[..display_decimals])
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((n >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_balance_query() {
        let query = encode_balance_query("cosmos1abc123").unwrap();
        assert!(!query.is_empty());
        assert!(query.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='));
    }

    #[test]
    fn test_parse_token_amount() {
        assert_eq!(parse_token_amount("1", 6).unwrap(), 1_000_000);
        assert_eq!(parse_token_amount("0.5", 6).unwrap(), 500_000);
        assert_eq!(parse_token_amount("1.5", 6).unwrap(), 1_500_000);
        assert_eq!(parse_token_amount("0.000001", 6).unwrap(), 1);
        assert_eq!(parse_token_amount("100", 6).unwrap(), 100_000_000);
    }

    #[test]
    fn test_format_token_amount() {
        assert_eq!(format_token_amount(1_000_000, 6), "1.0000");
        assert_eq!(format_token_amount(1_500_000, 6), "1.5000");
        assert_eq!(format_token_amount(1, 6), "0.0000");
        assert_eq!(format_token_amount(500_000, 6), "0.5000");
    }

    #[test]
    fn test_encode_transfer_msg() {
        let msg = encode_transfer_msg("cosmos1dest", "1.5", 6).unwrap();
        assert!(msg.contains("\"transfer\""));
        assert!(msg.contains("\"recipient\":\"cosmos1dest\""));
        assert!(msg.contains("\"amount\":\"1500000\""));
    }

    #[test]
    fn test_tokens_for_chain() {
        let osmosis = tokens_for_chain("osmosis");
        assert!(!osmosis.is_empty());
        assert!(osmosis.iter().all(|t| t.chain_id == "osmosis"));

        let cosmos = tokens_for_chain("cosmos");
        // May be empty since most Cosmos Hub tokens are IBC, not CW-20
        assert!(cosmos.iter().all(|t| t.chain_id == "cosmos"));
    }
}
