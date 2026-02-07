// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// tx/tron: TRON transaction signing (API-assisted)
//
// Flow:
// 1. POST /wallet/createtransaction -> {txID, raw_data, raw_data_hex}
// 2. SHA256(hex::decode(raw_data_hex)) -> digest
// 3. secp256k1 sign(digest) -> 65 bytes (r || s || recovery_id)
// 4. POST /wallet/broadcasttransaction with TX + signature

use k256::ecdsa::{SigningKey, signature::hazmat::PrehashSigner};
use sha2::{Digest, Sha256};

/// Sign the raw_data_hex of a TRON transaction
/// Returns the signature as 65-byte hex string (r || s || v)
pub fn sign_tron_tx(private_key: &[u8; 32], raw_data_hex: &str) -> Result<String, String> {
    let raw_bytes = hex::decode(raw_data_hex)
        .map_err(|e| format!("Hex decode error: {}", e))?;

    // SHA256 of raw_data bytes
    let digest = Sha256::digest(&raw_bytes);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&digest);

    let signing_key = SigningKey::from_bytes(private_key.into())
        .map_err(|e| format!("Invalid key: {}", e))?;

    let (signature, recovery_id) = signing_key
        .sign_prehash(&hash)
        .map_err(|e| format!("Sign error: {}", e))?;

    let sig_bytes = signature.to_bytes();
    // TRON signature: r(32) || s(32) || v(1) where v = recovery_id (0 or 1)
    let mut sig_65 = Vec::with_capacity(65);
    sig_65.extend_from_slice(&sig_bytes[..64]);
    sig_65.push(recovery_id.to_byte());

    Ok(hex::encode(sig_65))
}

/// Verify that a TX created by the node contains the correct parameters
/// SECURITY: Prevents a malicious node from substituting the recipient or amount
pub fn verify_tron_tx_params(
    raw_data_json: &serde_json::Value,
    expected_to_hex: &str,
    expected_amount: u64,
) -> Result<(), String> {
    let contract = raw_data_json["contract"]
        .as_array()
        .and_then(|a| a.first())
        .ok_or("Missing contract in raw_data")?;

    let params = &contract["parameter"]["value"];

    let to = params["to_address"].as_str()
        .ok_or("Missing to_address in TX")?;
    if to != expected_to_hex {
        return Err(format!(
            "to_address mismatch: expected {}, got {}",
            expected_to_hex, to
        ));
    }

    let amount = params["amount"].as_u64()
        .ok_or("Missing amount in TX")?;
    if amount != expected_amount {
        return Err(format!(
            "amount mismatch: expected {}, got {}",
            expected_amount, amount
        ));
    }

    Ok(())
}

/// Convert TRX to sun (1 TRX = 1,000,000 sun)
pub fn parse_trx_to_sun(amount: &str) -> Result<u64, String> {
    let parts: Vec<&str> = amount.split('.').collect();
    let (integer_part, decimal_part) = match parts.len() {
        1 => (parts[0], ""),
        2 => (parts[0], parts[1]),
        _ => return Err("Formato importo non valido".into()),
    };

    let integer: u64 = if integer_part.is_empty() {
        0
    } else {
        integer_part.parse().map_err(|_| "Parte intera non valida")?
    };

    let decimals: u64 = if decimal_part.is_empty() {
        0
    } else {
        let padded = format!("{:0<6}", decimal_part);
        let trimmed = &padded[..6];
        trimmed.parse().map_err(|_| "Parte decimale non valida")?
    };

    integer.checked_mul(1_000_000)
        .and_then(|v| v.checked_add(decimals))
        .ok_or_else(|| "Importo overflow".into())
}

/// Format sun to TRX string (e.g. 1000000 -> "1.0000")
pub fn format_sun(sun: u64) -> String {
    let trx = sun / 1_000_000;
    let frac = sun % 1_000_000;
    let frac_str = format!("{:06}", frac);
    format!("{}.{}", trx, &frac_str[..4])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bip39_utils;
    use crate::chains::tron;

    #[test]
    fn test_parse_trx_to_sun() {
        assert_eq!(parse_trx_to_sun("1").unwrap(), 1_000_000);
        assert_eq!(parse_trx_to_sun("0.5").unwrap(), 500_000);
        assert_eq!(parse_trx_to_sun("100").unwrap(), 100_000_000);
        assert_eq!(parse_trx_to_sun("0.000001").unwrap(), 1);
    }

    #[test]
    fn test_format_sun() {
        assert_eq!(format_sun(1_000_000), "1.0000");
        assert_eq!(format_sun(0), "0.0000");
        assert_eq!(format_sun(1_500_000), "1.5000");
    }

    #[test]
    fn test_sign_tron_tx() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let privkey = tron::get_private_key(&seed).unwrap();

        // Mock raw_data_hex (any hex bytes will do for testing signing)
        let raw_data_hex = "0a02abcd2208aabbccdd1122334540e0a7c3b6f0315a65080112610a2d747970652e676f6f676c65617069732e636f6d2f70726f746f636f6c2e5472616e73666572436f6e747261637412300a1541aabbccddee00112233445566778899aabbccddee12154100112233445566778899aabbccddeeff0011223318c0843d70d0c4bfb6f031";
        let sig = sign_tron_tx(&privkey, raw_data_hex).unwrap();
        // Signature should be 65 bytes = 130 hex chars
        assert_eq!(sig.len(), 130, "Sig length: {}", sig.len());
    }

    #[test]
    fn test_verify_tron_tx_params_ok() {
        let raw_data: serde_json::Value = serde_json::json!({
            "contract": [{
                "parameter": {
                    "value": {
                        "to_address": "41aabbccdd",
                        "amount": 1000000
                    }
                }
            }]
        });
        assert!(verify_tron_tx_params(&raw_data, "41aabbccdd", 1_000_000).is_ok());
    }

    #[test]
    fn test_verify_tron_tx_params_mismatch() {
        let raw_data: serde_json::Value = serde_json::json!({
            "contract": [{
                "parameter": {
                    "value": {
                        "to_address": "41aabbccdd",
                        "amount": 999999
                    }
                }
            }]
        });
        assert!(verify_tron_tx_params(&raw_data, "41aabbccdd", 1_000_000).is_err());
    }
}
