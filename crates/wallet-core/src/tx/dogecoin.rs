// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// tx/dogecoin: P2PKH (legacy) transaction construction and signing
//
// Implements legacy sighash for P2PKH transactions.
// Format: [version][inputs with scriptSig][outputs][locktime]
// NO witness data, NO segwit marker.

use k256::ecdsa::{SigningKey, signature::hazmat::PrehashSigner};
use sha2::{Digest, Sha256};

use super::SignedTransaction;
use crate::chains::ChainId;

/// A Dogecoin UTXO (unspent transaction output)
#[derive(Debug, Clone)]
pub struct DogecoinUtxo {
    pub txid: [u8; 32],
    pub vout: u32,
    pub value: u64,            // in satoshi (1 DOGE = 10^8)
    pub script_pubkey: Vec<u8>, // P2PKH: 76 a9 14 <hash160> 88 ac
}

/// A Dogecoin transaction output
#[derive(Debug, Clone)]
pub struct DogecoinTxOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

/// A P2PKH Dogecoin transaction
pub struct DogecoinTransaction {
    pub inputs: Vec<DogecoinUtxo>,
    pub outputs: Vec<DogecoinTxOutput>,
}

/// Create P2PKH output script: OP_DUP OP_HASH160 <20-byte-hash> OP_EQUALVERIFY OP_CHECKSIG
pub fn p2pkh_script(pubkey_hash: &[u8; 20]) -> Vec<u8> {
    let mut script = Vec::with_capacity(25);
    script.push(0x76); // OP_DUP
    script.push(0xa9); // OP_HASH160
    script.push(0x14); // push 20 bytes
    script.extend_from_slice(pubkey_hash);
    script.push(0x88); // OP_EQUALVERIFY
    script.push(0xac); // OP_CHECKSIG
    script
}

impl DogecoinTransaction {
    /// Build a simple P2PKH spend transaction
    pub fn build_p2pkh(
        inputs: Vec<DogecoinUtxo>,
        to_hash: &[u8; 20],
        amount: u64,
        change_hash: &[u8; 20],
        fee: u64,
    ) -> Result<Self, String> {
        let total_input: u64 = inputs.iter().map(|u| u.value).sum();
        let required = amount.checked_add(fee)
            .ok_or("Importo overflow")?;
        if total_input < required {
            return Err(format!(
                "Fondi insufficienti: {} sat disponibili, {} sat richiesti (amount {} + fee {})",
                total_input, required, amount, fee
            ));
        }

        let mut outputs = vec![DogecoinTxOutput {
            value: amount,
            script_pubkey: p2pkh_script(to_hash),
        }];

        let change = total_input - amount - fee;
        if change > 100_000 { // dust threshold: 0.001 DOGE
            outputs.push(DogecoinTxOutput {
                value: change,
                script_pubkey: p2pkh_script(change_hash),
            });
        }

        Ok(Self { inputs, outputs })
    }

    /// Sign the transaction with a private key (all inputs signed with same key)
    pub fn sign(&self, private_key: &[u8; 32]) -> Result<SignedTransaction, String> {
        let signing_key = SigningKey::from_bytes(private_key.into())
            .map_err(|e| format!("Chiave non valida: {}", e))?;
        let verifying_key = signing_key.verifying_key();
        let pubkey = verifying_key.to_encoded_point(true);
        let pubkey_bytes = pubkey.as_bytes(); // 33 bytes compressed

        let mut script_sigs: Vec<Vec<u8>> = Vec::new();

        for i in 0..self.inputs.len() {
            // Legacy SIGHASH_ALL
            let sighash = self.legacy_sighash(i);

            // Sign with secp256k1
            let (signature, _recovery_id) = signing_key
                .sign_prehash(&sighash)
                .map_err(|e| format!("Errore firma: {}", e))?;

            // DER encode signature + SIGHASH_ALL byte
            let sig_bytes = signature.to_bytes();
            let r = &sig_bytes[..32];
            let s = &sig_bytes[32..];
            let mut der_sig = der_encode_signature(r, s);
            der_sig.push(0x01); // SIGHASH_ALL byte

            // Build scriptSig: <push der_sig> <push pubkey>
            let mut script_sig = Vec::new();
            push_var_bytes(&mut script_sig, &der_sig);
            push_var_bytes(&mut script_sig, pubkey_bytes);
            script_sigs.push(script_sig);
        }

        // Serialize final transaction with signed scriptSigs
        let raw = self.serialize_with_scripts(&script_sigs);

        // txid = reverse(double_sha256(serialized))
        let txid_bytes = double_sha256(&raw);
        let mut txid_display = txid_bytes;
        txid_display.reverse();

        Ok(SignedTransaction {
            chain_id: ChainId::Dogecoin,
            raw_bytes: raw,
            tx_hash: hex::encode(txid_display),
        })
    }

    /// Compute legacy sighash for input at `index`
    /// 1. Serialize TX with scriptSig[index] = UTXO scriptPubKey, others = empty
    /// 2. Append SIGHASH_ALL (01000000 LE)
    /// 3. double_sha256(blob)
    fn legacy_sighash(&self, index: usize) -> [u8; 32] {
        let mut buf = Vec::new();

        // Version = 1 for Dogecoin legacy
        buf.extend_from_slice(&1u32.to_le_bytes());

        // Input count
        push_varint(&mut buf, self.inputs.len() as u64);

        // Inputs
        for (i, input) in self.inputs.iter().enumerate() {
            buf.extend_from_slice(&input.txid);
            buf.extend_from_slice(&input.vout.to_le_bytes());
            if i == index {
                // The input being signed: scriptSig = scriptPubKey of the UTXO
                push_var_bytes(&mut buf, &input.script_pubkey);
            } else {
                // Other inputs: empty scriptSig
                buf.push(0x00);
            }
            buf.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes()); // nSequence
        }

        // Output count
        push_varint(&mut buf, self.outputs.len() as u64);

        // Outputs
        for output in &self.outputs {
            buf.extend_from_slice(&output.value.to_le_bytes());
            push_var_bytes(&mut buf, &output.script_pubkey);
        }

        // Locktime
        buf.extend_from_slice(&0u32.to_le_bytes());

        // SIGHASH_ALL type (4 bytes LE)
        buf.extend_from_slice(&1u32.to_le_bytes());

        // Double SHA256
        double_sha256(&buf)
    }

    /// Serialize TX with signed scriptSigs (legacy format, no witness)
    fn serialize_with_scripts(&self, script_sigs: &[Vec<u8>]) -> Vec<u8> {
        let mut buf = Vec::new();

        // Version 1
        buf.extend_from_slice(&1u32.to_le_bytes());

        // Input count
        push_varint(&mut buf, self.inputs.len() as u64);

        // Inputs with signed scriptSigs
        for (i, input) in self.inputs.iter().enumerate() {
            buf.extend_from_slice(&input.txid);
            buf.extend_from_slice(&input.vout.to_le_bytes());
            push_var_bytes(&mut buf, &script_sigs[i]); // signed scriptSig
            buf.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());
        }

        // Output count
        push_varint(&mut buf, self.outputs.len() as u64);

        // Outputs
        for output in &self.outputs {
            buf.extend_from_slice(&output.value.to_le_bytes());
            push_var_bytes(&mut buf, &output.script_pubkey);
        }

        // Locktime
        buf.extend_from_slice(&0u32.to_le_bytes());

        buf
    }
}

/// Parse DOGE amount string to satoshi (1 DOGE = 10^8 satoshi)
pub fn parse_doge_to_satoshi(amount: &str) -> Result<u64, String> {
    // Same precision as BTC
    super::bitcoin::parse_btc_to_satoshi(amount)
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

fn double_sha256(data: &[u8]) -> [u8; 32] {
    let first = Sha256::digest(data);
    let second = Sha256::digest(first);
    let mut result = [0u8; 32];
    result.copy_from_slice(&second);
    result
}

fn push_varint(buf: &mut Vec<u8>, val: u64) {
    if val < 0xFD {
        buf.push(val as u8);
    } else if val <= 0xFFFF {
        buf.push(0xFD);
        buf.extend_from_slice(&(val as u16).to_le_bytes());
    } else if val <= 0xFFFFFFFF {
        buf.push(0xFE);
        buf.extend_from_slice(&(val as u32).to_le_bytes());
    } else {
        buf.push(0xFF);
        buf.extend_from_slice(&val.to_le_bytes());
    }
}

fn push_var_bytes(buf: &mut Vec<u8>, data: &[u8]) {
    push_varint(buf, data.len() as u64);
    buf.extend_from_slice(data);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bip39_utils;
    use crate::chains::dogecoin;

    #[test]
    fn test_p2pkh_script() {
        let hash = [0xab; 20];
        let script = p2pkh_script(&hash);
        assert_eq!(script.len(), 25);
        assert_eq!(script[0], 0x76); // OP_DUP
        assert_eq!(script[1], 0xa9); // OP_HASH160
        assert_eq!(script[2], 0x14); // push 20
        assert_eq!(script[23], 0x88); // OP_EQUALVERIFY
        assert_eq!(script[24], 0xac); // OP_CHECKSIG
    }

    #[test]
    fn test_parse_doge_to_satoshi() {
        assert_eq!(parse_doge_to_satoshi("1").unwrap(), 100_000_000);
        assert_eq!(parse_doge_to_satoshi("0.001").unwrap(), 100_000);
        assert_eq!(parse_doge_to_satoshi("100").unwrap(), 10_000_000_000);
    }

    #[test]
    fn test_build_and_sign_tx() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let privkey = dogecoin::get_private_key(&seed).unwrap();
        let pubkey = dogecoin::get_public_key(&seed).unwrap();
        let our_hash = dogecoin::hash160_pubkey(&pubkey);

        let utxo = DogecoinUtxo {
            txid: [0xaa; 32],
            vout: 0,
            value: 10_000_000_000, // 100 DOGE
            script_pubkey: p2pkh_script(&our_hash),
        };

        let recipient_hash = [0xbb; 20];
        let tx = DogecoinTransaction::build_p2pkh(
            vec![utxo],
            &recipient_hash,
            5_000_000_000, // 50 DOGE
            &our_hash,
            1_000_000, // 0.01 DOGE fee
        ).unwrap();

        assert_eq!(tx.outputs.len(), 2); // amount + change

        let signed = tx.sign(&privkey).unwrap();
        assert!(!signed.raw_bytes.is_empty());
        assert_eq!(signed.chain_id, ChainId::Dogecoin);
        assert_eq!(signed.tx_hash.len(), 64); // hex txid
    }

    #[test]
    fn test_insufficient_funds() {
        let utxo = DogecoinUtxo {
            txid: [0; 32],
            vout: 0,
            value: 1_000_000, // 0.01 DOGE
            script_pubkey: p2pkh_script(&[0; 20]),
        };
        let result = DogecoinTransaction::build_p2pkh(
            vec![utxo],
            &[0; 20],
            500_000_000, // 5 DOGE
            &[0; 20],
            1_000_000, // 0.01 DOGE fee
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_no_change_below_dust() {
        let our_hash = [0xcc; 20];
        let utxo = DogecoinUtxo {
            txid: [0xaa; 32],
            vout: 0,
            value: 2_000_000, // 0.02 DOGE
            script_pubkey: p2pkh_script(&our_hash),
        };
        let tx = DogecoinTransaction::build_p2pkh(
            vec![utxo],
            &[0xbb; 20],
            1_000_000, // 0.01 DOGE
            &our_hash,
            1_000_000, // 0.01 DOGE fee — leaves 0 change
        ).unwrap();
        // No change output since change = 0
        assert_eq!(tx.outputs.len(), 1);
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
