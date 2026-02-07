// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// tx/stellar: Stellar transaction construction and signing (XDR format)
//
// Implements XDR serialization for native XLM payment transactions.
// Signs with Ed25519 on SHA256(network_id + [0,0,0,2] + tx_body_xdr).

use ed25519_dalek::{SigningKey, Signer};
use sha2::{Digest, Sha256};

use super::SignedTransaction;
use crate::chains::ChainId;

/// Network passphrases
pub const MAINNET_PASSPHRASE: &str = "Public Global Stellar Network ; September 2015";
pub const TESTNET_PASSPHRASE: &str = "Test SDF Network ; September 2015";

/// A Stellar payment transaction (native XLM only)
#[derive(Debug, Clone)]
pub struct StellarTransaction {
    pub source_pubkey: [u8; 32],
    pub destination_pubkey: [u8; 32],
    pub amount_stroops: i64,
    pub sequence: i64,
    pub fee: u32,
    pub network_passphrase: String,
}

impl StellarTransaction {
    /// Sign the transaction with an Ed25519 private key
    pub fn sign(&self, private_key: &[u8; 32]) -> Result<SignedTransaction, String> {
        let signing_key = SigningKey::from_bytes(private_key);

        // Serialize transaction body as XDR
        let tx_body = self.serialize_tx_body();

        // Compute transaction hash:
        // SHA256(network_id + ENVELOPE_TYPE_TX [0,0,0,2] + tx_body_xdr)
        let network_id = Sha256::digest(self.network_passphrase.as_bytes());
        let mut preimage = Vec::new();
        preimage.extend_from_slice(&network_id);
        preimage.extend_from_slice(&[0, 0, 0, 2]); // ENVELOPE_TYPE_TX
        preimage.extend_from_slice(&tx_body);
        let tx_hash = Sha256::digest(&preimage);

        // Sign
        let signature = signing_key.sign(&tx_hash);

        // Build TransactionEnvelope (ENVELOPE_TYPE_TX = 2)
        let mut envelope = Vec::new();
        // Envelope type
        envelope.extend_from_slice(&2i32.to_be_bytes());
        // Transaction body
        envelope.extend_from_slice(&tx_body);
        // Decorated signatures array (1 element)
        envelope.extend_from_slice(&1u32.to_be_bytes()); // length of signatures array
        // Signature hint (last 4 bytes of public key)
        envelope.extend_from_slice(&self.source_pubkey[28..32]);
        // Signature (variable length opaque: length prefix + data)
        let sig_bytes = signature.to_bytes();
        envelope.extend_from_slice(&(sig_bytes.len() as u32).to_be_bytes());
        envelope.extend_from_slice(&sig_bytes);

        // tx_hash as hex string
        let tx_hash_hex = hex::encode(tx_hash);

        Ok(SignedTransaction {
            chain_id: ChainId::Stellar,
            raw_bytes: envelope,
            tx_hash: tx_hash_hex,
        })
    }

    /// Serialize the Transaction body as XDR (big-endian binary)
    fn serialize_tx_body(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Source account: MuxedAccount (KEY_TYPE_ED25519 = 0)
        buf.extend_from_slice(&0u32.to_be_bytes()); // KEY_TYPE_ED25519
        buf.extend_from_slice(&self.source_pubkey);

        // Fee
        buf.extend_from_slice(&self.fee.to_be_bytes());

        // Sequence number (int64)
        buf.extend_from_slice(&self.sequence.to_be_bytes());

        // Time bounds preconditions: PRECOND_NONE = 0
        buf.extend_from_slice(&0u32.to_be_bytes());

        // Memo: MEMO_NONE = 0
        buf.extend_from_slice(&0u32.to_be_bytes());

        // Operations array (1 operation)
        buf.extend_from_slice(&1u32.to_be_bytes()); // num operations

        // Operation: no source account override (bool false = 0)
        buf.extend_from_slice(&0u32.to_be_bytes());

        // Operation body: PAYMENT = 1
        buf.extend_from_slice(&1u32.to_be_bytes());

        // Payment destination: MuxedAccount (KEY_TYPE_ED25519 = 0)
        buf.extend_from_slice(&0u32.to_be_bytes());
        buf.extend_from_slice(&self.destination_pubkey);

        // Asset: ASSET_TYPE_NATIVE = 0
        buf.extend_from_slice(&0u32.to_be_bytes());

        // Amount (int64, in stroops)
        buf.extend_from_slice(&self.amount_stroops.to_be_bytes());

        // Transaction ext: 0
        buf.extend_from_slice(&0u32.to_be_bytes());

        buf
    }
}

/// Parse XLM amount string to stroops (1 XLM = 10^7 stroops)
pub fn parse_xlm_to_stroops(amount: &str) -> Result<i64, String> {
    let parts: Vec<&str> = amount.split('.').collect();
    let (integer_part, decimal_part) = match parts.len() {
        1 => (parts[0], ""),
        2 => (parts[0], parts[1]),
        _ => return Err("Formato importo non valido".into()),
    };

    let integer: i64 = if integer_part.is_empty() {
        0
    } else {
        integer_part.parse().map_err(|_| "Parte intera non valida")?
    };

    let decimals: i64 = if decimal_part.is_empty() {
        0
    } else {
        let padded = format!("{:0<7}", decimal_part);
        let trimmed = &padded[..7];
        trimmed.parse().map_err(|_| "Parte decimale non valida")?
    };

    integer.checked_mul(10_000_000)
        .and_then(|v| v.checked_add(decimals))
        .ok_or_else(|| "Importo overflow".into())
}

/// Format stroops to XLM string (e.g. 10000000 → "1.0000")
pub fn format_stroops(stroops: i64) -> String {
    let xlm = stroops / 10_000_000;
    let frac = (stroops % 10_000_000).unsigned_abs();
    let frac_str = format!("{:07}", frac);
    format!("{}.{}", xlm, &frac_str[..4])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bip39_utils;
    use crate::chains::stellar;

    #[test]
    fn test_parse_xlm_to_stroops() {
        assert_eq!(parse_xlm_to_stroops("1").unwrap(), 10_000_000);
        assert_eq!(parse_xlm_to_stroops("0.001").unwrap(), 10_000);
        assert_eq!(parse_xlm_to_stroops("0.0000001").unwrap(), 1);
        assert_eq!(parse_xlm_to_stroops("100").unwrap(), 1_000_000_000);
    }

    #[test]
    fn test_format_stroops() {
        assert_eq!(format_stroops(10_000_000), "1.0000");
        assert_eq!(format_stroops(0), "0.0000");
        assert_eq!(format_stroops(15_000_000), "1.5000");
    }

    #[test]
    fn test_sign_stellar_tx() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let keypair = stellar::get_keypair(&seed).unwrap();
        let pubkey = stellar::get_public_key(&seed).unwrap();

        let tx = StellarTransaction {
            source_pubkey: pubkey,
            destination_pubkey: [0xAA; 32],
            amount_stroops: 10_000_000, // 1 XLM
            sequence: 1,
            fee: 100,
            network_passphrase: TESTNET_PASSPHRASE.into(),
        };

        let private_key: [u8; 32] = keypair[..32].try_into().unwrap();
        let signed = tx.sign(&private_key).unwrap();
        assert!(!signed.raw_bytes.is_empty());
        assert_eq!(signed.chain_id, ChainId::Stellar);
        assert_eq!(signed.tx_hash.len(), 64); // hex SHA256
    }
}
