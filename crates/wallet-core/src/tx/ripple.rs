// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// tx/ripple: XRP Ledger Payment transaction construction and signing
//
// Implements XRP binary serialization for Payment transactions.
// Signs with secp256k1 ECDSA on SHA512-Half(prefix + tx_blob).

use k256::ecdsa::{SigningKey, signature::hazmat::PrehashSigner};
use sha2::{Digest, Sha512};

use super::SignedTransaction;
use crate::chains::ChainId;

/// An XRP Payment transaction
#[derive(Debug, Clone)]
pub struct RippleTransaction {
    pub account: [u8; 20],       // sender account ID (Hash160)
    pub destination: [u8; 20],   // recipient account ID (Hash160)
    pub amount_drops: u64,       // amount in drops (1 XRP = 1,000,000 drops)
    pub fee_drops: u64,          // fee in drops
    pub sequence: u32,           // account sequence number
    pub signing_pubkey: [u8; 33], // compressed secp256k1 public key
}

impl RippleTransaction {
    /// Sign the transaction with a secp256k1 private key
    pub fn sign(&self, private_key: &[u8; 32]) -> Result<SignedTransaction, String> {
        let signing_key = SigningKey::from_bytes(private_key.into())
            .map_err(|e| format!("Chiave non valida: {}", e))?;

        // Serialize for signing (with SigningPubKey, without TxnSignature)
        let signing_blob = self.serialize_for_signing();

        // Hash: SHA512-Half of "STX\0" + signing_blob
        let mut hasher = Sha512::new();
        hasher.update(b"STX\0"); // Transaction signing prefix
        hasher.update(&signing_blob);
        let hash_full = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&hash_full[..32]); // SHA512-Half = first 32 bytes

        // Sign with secp256k1
        let (signature, _recovery_id) = signing_key
            .sign_prehash(&hash)
            .map_err(|e| format!("Errore firma: {}", e))?;

        let sig_bytes = signature.to_bytes();
        let r = &sig_bytes[..32];
        let s = &sig_bytes[32..];
        let der_sig = der_encode_signature(r, s);

        // Serialize complete transaction (with TxnSignature)
        let tx_blob = self.serialize_with_signature(&der_sig);

        // Compute transaction hash: SHA512-Half of "TXN\0" + tx_blob
        let mut hasher2 = Sha512::new();
        hasher2.update(b"TXN\0"); // Transaction ID prefix
        hasher2.update(&tx_blob);
        let hash_full2 = hasher2.finalize();
        let tx_hash = hex::encode(&hash_full2[..32]);

        Ok(SignedTransaction {
            chain_id: ChainId::Ripple,
            raw_bytes: tx_blob,
            tx_hash,
        })
    }

    /// Serialize transaction for signing (no TxnSignature field)
    fn serialize_for_signing(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Prefix for signing: "53545800" (STX\0) is added externally by the hash function

        // Fields must be serialized in canonical order (by type code, then field code)

        // TransactionType (UINT16, type=1, field=2) → field_id = 0x12
        buf.push(0x12);
        buf.extend_from_slice(&0u16.to_be_bytes()); // Payment = 0

        // Flags (UINT32, type=2, field=2) → field_id = 0x22
        buf.push(0x22);
        buf.extend_from_slice(&0u32.to_be_bytes()); // tfFullyCanonicalSig = 0x80000000 not needed with canonical sig

        // Sequence (UINT32, type=2, field=4) → field_id = 0x24
        buf.push(0x24);
        buf.extend_from_slice(&self.sequence.to_be_bytes());

        // Amount (AMOUNT, type=6, field=1) → field_id = 0x61
        buf.push(0x61);
        buf.extend_from_slice(&encode_xrp_amount(self.amount_drops));

        // Fee (AMOUNT, type=6, field=8) → field_id = 0x68
        buf.push(0x68);
        buf.extend_from_slice(&encode_xrp_amount(self.fee_drops));

        // SigningPubKey (VL, type=7, field=3) → field_id = 0x73
        buf.push(0x73);
        buf.push(self.signing_pubkey.len() as u8); // length prefix
        buf.extend_from_slice(&self.signing_pubkey);

        // Account (ACCOUNT_ID, type=8, field=1) → field_id = 0x81
        buf.push(0x81);
        buf.push(20); // length
        buf.extend_from_slice(&self.account);

        // Destination (ACCOUNT_ID, type=8, field=3) → field_id = 0x83
        buf.push(0x83);
        buf.push(20); // length
        buf.extend_from_slice(&self.destination);

        buf
    }

    /// Serialize complete transaction (with TxnSignature)
    fn serialize_with_signature(&self, der_signature: &[u8]) -> Vec<u8> {
        let mut buf = Vec::new();

        // TransactionType
        buf.push(0x12);
        buf.extend_from_slice(&0u16.to_be_bytes());

        // Flags
        buf.push(0x22);
        buf.extend_from_slice(&0u32.to_be_bytes());

        // Sequence
        buf.push(0x24);
        buf.extend_from_slice(&self.sequence.to_be_bytes());

        // Amount
        buf.push(0x61);
        buf.extend_from_slice(&encode_xrp_amount(self.amount_drops));

        // Fee
        buf.push(0x68);
        buf.extend_from_slice(&encode_xrp_amount(self.fee_drops));

        // SigningPubKey
        buf.push(0x73);
        buf.push(self.signing_pubkey.len() as u8);
        buf.extend_from_slice(&self.signing_pubkey);

        // TxnSignature (VL, type=7, field=4) → field_id = 0x74
        buf.push(0x74);
        encode_vl_length(&mut buf, der_signature.len());
        buf.extend_from_slice(der_signature);

        // Account
        buf.push(0x81);
        buf.push(20);
        buf.extend_from_slice(&self.account);

        // Destination
        buf.push(0x83);
        buf.push(20);
        buf.extend_from_slice(&self.destination);

        buf
    }
}

/// Encode an XRP amount in drops (native currency)
/// XRP amounts use a special 64-bit encoding:
/// bit 63 = 1 (not-negative indicator for serialization)
/// bit 62 = 0 (XRP native, not IOU)
/// bits 0-61 = amount in drops
fn encode_xrp_amount(drops: u64) -> [u8; 8] {
    let encoded = 0x4000_0000_0000_0000u64 | drops;
    encoded.to_be_bytes()
}

/// Encode variable-length prefix for XRP serialization
fn encode_vl_length(buf: &mut Vec<u8>, len: usize) {
    if len <= 192 {
        buf.push(len as u8);
    } else if len <= 12480 {
        let adjusted = len - 193;
        buf.push((adjusted >> 8) as u8 + 193);
        buf.push((adjusted & 0xFF) as u8);
    } else {
        let adjusted = len - 12481;
        buf.push((adjusted >> 16) as u8 + 241);
        buf.push(((adjusted >> 8) & 0xFF) as u8);
        buf.push((adjusted & 0xFF) as u8);
    }
}

/// DER encode an ECDSA signature (r, s)
fn der_encode_signature(r: &[u8], s: &[u8]) -> Vec<u8> {
    fn encode_int(val: &[u8]) -> Vec<u8> {
        let stripped = match val.iter().position(|&b| b != 0) {
            Some(pos) => &val[pos..],
            None => &[0u8],
        };
        let needs_pad = stripped[0] & 0x80 != 0;
        let mut out = Vec::with_capacity(2 + stripped.len() + needs_pad as usize);
        out.push(0x02); // INTEGER
        out.push((stripped.len() + needs_pad as usize) as u8);
        if needs_pad {
            out.push(0x00);
        }
        out.extend_from_slice(stripped);
        out
    }

    let r_enc = encode_int(r);
    let s_enc = encode_int(s);
    let mut der = Vec::with_capacity(2 + r_enc.len() + s_enc.len());
    der.push(0x30); // SEQUENCE
    der.push((r_enc.len() + s_enc.len()) as u8);
    der.extend_from_slice(&r_enc);
    der.extend_from_slice(&s_enc);
    der
}

/// Parse XRP amount string to drops (1 XRP = 1,000,000 drops)
pub fn parse_xrp_to_drops(amount: &str) -> Result<u64, String> {
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

/// Format drops to XRP string (e.g. 1000000 → "1.0000")
pub fn format_drops(drops: u64) -> String {
    let xrp = drops / 1_000_000;
    let frac = drops % 1_000_000;
    let frac_str = format!("{:06}", frac);
    format!("{}.{}", xrp, &frac_str[..4])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bip39_utils;
    use crate::chains::ripple;

    #[test]
    fn test_parse_xrp_to_drops() {
        assert_eq!(parse_xrp_to_drops("1").unwrap(), 1_000_000);
        assert_eq!(parse_xrp_to_drops("0.001").unwrap(), 1_000);
        assert_eq!(parse_xrp_to_drops("0.000001").unwrap(), 1);
        assert_eq!(parse_xrp_to_drops("100").unwrap(), 100_000_000);
    }

    #[test]
    fn test_format_drops() {
        assert_eq!(format_drops(1_000_000), "1.0000");
        assert_eq!(format_drops(0), "0.0000");
        assert_eq!(format_drops(1_500_000), "1.5000");
    }

    #[test]
    fn test_encode_xrp_amount() {
        let encoded = encode_xrp_amount(1_000_000);
        // Bit 63=1, bit 62=0, rest = 1000000
        assert_eq!(encoded[0] & 0xC0, 0x40); // 01xxxxxx
    }

    #[test]
    fn test_sign_ripple_tx() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let privkey = ripple::get_private_key(&seed).unwrap();
        let pubkey = ripple::get_public_key(&seed).unwrap();
        let account_id = ripple::get_account_id(&seed).unwrap();

        let tx = RippleTransaction {
            account: account_id,
            destination: [0xBB; 20],
            amount_drops: 1_000_000, // 1 XRP
            fee_drops: 12,           // standard fee
            sequence: 1,
            signing_pubkey: pubkey,
        };

        let signed = tx.sign(&privkey).unwrap();
        assert!(!signed.raw_bytes.is_empty());
        assert_eq!(signed.chain_id, ChainId::Ripple);
        assert_eq!(signed.tx_hash.len(), 64); // hex SHA512-Half
    }

    #[test]
    fn test_der_encode() {
        let r = [0x01; 32];
        let s = [0x02; 32];
        let der = der_encode_signature(&r, &s);
        assert_eq!(der[0], 0x30); // SEQUENCE
        assert_eq!(der[2], 0x02); // first INTEGER
    }
}
