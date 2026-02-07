// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// tx/ton: TON internal message construction and Ed25519 signing
//
// Simplified TON transfer: builds a wallet v4r2 external message
// containing an internal transfer message.

use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};

use super::SignedTransaction;
use crate::chains::ChainId;

/// TON native transfer
#[derive(Debug, Clone)]
pub struct TonTransfer {
    pub to_address_raw: Vec<u8>, // 34 bytes: tag + workchain + hash
    pub amount_nanoton: u64,
    pub seqno: u32,
    pub valid_until: u32,
}

impl TonTransfer {
    /// Build and sign the external message for wallet v4r2
    pub fn sign(&self, private_key: &[u8; 32]) -> Result<SignedTransaction, String> {
        let signing_key = SigningKey::from_bytes(private_key);

        // Build the internal message body:
        // wallet_id(4) + valid_until(4) + seqno(4) + op(1) + mode(1) + internal_msg
        let mut body = Vec::new();

        // Wallet ID (v4r2 default: 698983191 = 0x29A9A317)
        body.extend_from_slice(&698983191u32.to_be_bytes());
        // Valid until
        body.extend_from_slice(&self.valid_until.to_be_bytes());
        // Seqno
        body.extend_from_slice(&self.seqno.to_be_bytes());
        // Simple send op = 0
        body.push(0);
        // Send mode = 3 (pay fees separately + ignore errors)
        body.push(3);

        // Simplified internal message ref:
        // flags(1) + destination(34) + amount(8) + empty state_init + empty body
        let mut internal = Vec::new();
        internal.push(0x00); // flags: no bounce
        internal.extend_from_slice(&self.to_address_raw);
        internal.extend_from_slice(&self.amount_nanoton.to_be_bytes());
        // Extra currency, IHR fee, fwd fee, created_lt, created_at: all 0
        internal.extend_from_slice(&[0u8; 4]); // zero fields
        body.extend_from_slice(&internal);

        // Sign the body
        let signature = signing_key.sign(&body);
        let sig_bytes = signature.to_bytes();

        // External message = signature(64) + body
        let mut external = Vec::with_capacity(64 + body.len());
        external.extend_from_slice(&sig_bytes);
        external.extend_from_slice(&body);

        // Tx hash
        let mut hasher = Sha256::new();
        hasher.update(&external);
        let hash = hasher.finalize();

        Ok(SignedTransaction {
            chain_id: ChainId::Ton,
            raw_bytes: external,
            tx_hash: format!("0x{}", hex::encode(&hash)),
        })
    }
}

/// Parse nanoton from TON amount string (9 decimals)
pub fn parse_ton_to_nanoton(amount: &str) -> Result<u64, String> {
    let parts: Vec<&str> = amount.split('.').collect();
    let (integer_part, decimal_part) = match parts.len() {
        1 => (parts[0], ""),
        2 => (parts[0], parts[1]),
        _ => return Err("Invalid amount format".into()),
    };

    let integer: u64 = if integer_part.is_empty() {
        0
    } else {
        integer_part.parse().map_err(|_| "Invalid integer part")?
    };

    let decimals: u64 = if decimal_part.is_empty() {
        0
    } else {
        let padded = format!("{:0<9}", decimal_part);
        let trimmed = &padded[..9];
        trimmed.parse().map_err(|_| "Invalid decimal part")?
    };

    integer.checked_mul(1_000_000_000)
        .and_then(|v| v.checked_add(decimals))
        .ok_or_else(|| "Amount overflow".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ton_to_nanoton() {
        assert_eq!(parse_ton_to_nanoton("1").unwrap(), 1_000_000_000);
        assert_eq!(parse_ton_to_nanoton("0.5").unwrap(), 500_000_000);
    }

    #[test]
    fn test_sign_ton_transfer() {
        let transfer = TonTransfer {
            to_address_raw: {
                let mut v = vec![0x11, 0x00];
                v.extend_from_slice(&[0u8; 32]);
                v
            },
            amount_nanoton: 1_000_000_000,
            seqno: 1,
            valid_until: u32::MAX,
        };
        let key = [5u8; 32];
        let signed = transfer.sign(&key).unwrap();
        assert!(!signed.raw_bytes.is_empty());
        assert!(signed.tx_hash.starts_with("0x"));
    }
}
