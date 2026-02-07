// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// tx/bitcoin: P2WPKH (Native SegWit) transaction construction and signing
//
// Implements BIP-143 signature hash for witness v0 transactions.
// Format: [version][marker][flag][inputs][outputs][witness][locktime]

use k256::ecdsa::{SigningKey, signature::hazmat::PrehashSigner};
use sha2::{Digest, Sha256};

use super::SignedTransaction;
use crate::chains::ChainId;

/// A Bitcoin UTXO (unspent transaction output)
#[derive(Debug, Clone)]
pub struct Utxo {
    pub txid: [u8; 32],
    pub vout: u32,
    pub value: u64, // in satoshi
    pub script_pubkey: Vec<u8>, // P2WPKH script: OP_0 <20-byte-hash>
}

/// A Bitcoin transaction output
#[derive(Debug, Clone)]
pub struct TxOutput {
    pub value: u64, // in satoshi
    pub script_pubkey: Vec<u8>,
}

/// A P2WPKH Bitcoin transaction
#[derive(Debug, Clone)]
pub struct BitcoinTransaction {
    pub inputs: Vec<Utxo>,
    pub outputs: Vec<TxOutput>,
    pub fee_rate: u64, // sat/vbyte (not used in signing, for reference)
}

impl BitcoinTransaction {
    /// Build a simple P2WPKH spend transaction
    /// `to_address_hash` is the 20-byte pubkey hash of the recipient
    /// `change_hash` is the 20-byte pubkey hash for change output
    pub fn build_p2wpkh(
        inputs: Vec<Utxo>,
        to_hash: &[u8; 20],
        amount: u64,
        change_hash: &[u8; 20],
        fee: u64,
    ) -> Result<Self, String> {
        let total_input: u64 = inputs.iter().map(|u| u.value).sum();
        if total_input < amount + fee {
            return Err(format!(
                "Fondi insufficienti: {} sat disponibili, {} sat richiesti (amount {} + fee {})",
                total_input, amount + fee, amount, fee
            ));
        }

        let mut outputs = vec![TxOutput {
            value: amount,
            script_pubkey: p2wpkh_script(to_hash),
        }];

        let change = total_input - amount - fee;
        if change > 546 { // dust threshold
            outputs.push(TxOutput {
                value: change,
                script_pubkey: p2wpkh_script(change_hash),
            });
        }

        Ok(Self {
            inputs,
            outputs,
            fee_rate: 0,
        })
    }

    /// Sign the transaction with a private key (all inputs signed with same key)
    /// Returns a fully serialized SegWit transaction
    pub fn sign(&self, private_key: &[u8; 32]) -> Result<SignedTransaction, String> {
        let signing_key = SigningKey::from_bytes(private_key.into())
            .map_err(|e| format!("Chiave non valida: {}", e))?;
        let verifying_key = signing_key.verifying_key();
        let pubkey = verifying_key.to_encoded_point(true);
        let pubkey_bytes = pubkey.as_bytes();

        // BIP-143 precomputed hashes
        let hash_prevouts = double_sha256(&self.serialize_prevouts());
        let hash_sequence = double_sha256(&self.serialize_sequence());
        let hash_outputs = double_sha256(&self.serialize_outputs());

        let mut witnesses: Vec<Vec<u8>> = Vec::new();

        for (i, input) in self.inputs.iter().enumerate() {
            // BIP-143 sighash for P2WPKH
            let script_code = p2pkh_script_code(&input.script_pubkey)?;
            let sighash = self.bip143_sighash(
                &hash_prevouts,
                &hash_sequence,
                i,
                &script_code,
                input.value,
                &hash_outputs,
            );

            let (signature, recovery_id) = signing_key
                .sign_prehash(&sighash)
                .map_err(|e| format!("Errore firma: {}", e))?;
            let _ = recovery_id;

            // DER encode signature + SIGHASH_ALL
            let sig_bytes = signature.to_bytes();
            let r = &sig_bytes[..32];
            let s = &sig_bytes[32..];
            let mut der_sig = der_encode_signature(r, s);
            der_sig.push(0x01); // SIGHASH_ALL

            // Witness: [sig, pubkey]
            let mut witness = Vec::new();
            witness.push(0x02); // 2 items
            push_var_bytes(&mut witness, &der_sig);
            push_var_bytes(&mut witness, pubkey_bytes);
            witnesses.push(witness);
        }

        // Serialize full SegWit transaction
        let raw = self.serialize_segwit(&witnesses);

        // txid = double_sha256 of non-witness serialization (legacy format)
        let legacy = self.serialize_legacy();
        let txid_bytes = double_sha256(&legacy);
        // Reverse for display (Bitcoin txid is little-endian)
        let mut txid_display = txid_bytes;
        txid_display.reverse();

        Ok(SignedTransaction {
            chain_id: ChainId::Bitcoin,
            raw_bytes: raw,
            tx_hash: hex::encode(txid_display),
        })
    }

    /// BIP-143 sighash computation for a single input
    fn bip143_sighash(
        &self,
        hash_prevouts: &[u8; 32],
        hash_sequence: &[u8; 32],
        input_index: usize,
        script_code: &[u8],
        value: u64,
        hash_outputs: &[u8; 32],
    ) -> [u8; 32] {
        let input = &self.inputs[input_index];
        let mut preimage = Vec::new();

        // 1. version
        preimage.extend_from_slice(&2u32.to_le_bytes());
        // 2. hashPrevouts
        preimage.extend_from_slice(hash_prevouts);
        // 3. hashSequence
        preimage.extend_from_slice(hash_sequence);
        // 4. outpoint (txid + vout)
        preimage.extend_from_slice(&input.txid);
        preimage.extend_from_slice(&input.vout.to_le_bytes());
        // 5. scriptCode
        push_var_bytes(&mut preimage, script_code);
        // 6. value
        preimage.extend_from_slice(&value.to_le_bytes());
        // 7. nSequence
        preimage.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());
        // 8. hashOutputs
        preimage.extend_from_slice(hash_outputs);
        // 9. nLockTime
        preimage.extend_from_slice(&0u32.to_le_bytes());
        // 10. sighash type (SIGHASH_ALL = 1)
        preimage.extend_from_slice(&1u32.to_le_bytes());

        double_sha256(&preimage)
    }

    fn serialize_prevouts(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        for input in &self.inputs {
            buf.extend_from_slice(&input.txid);
            buf.extend_from_slice(&input.vout.to_le_bytes());
        }
        buf
    }

    fn serialize_sequence(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        for _ in &self.inputs {
            buf.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());
        }
        buf
    }

    fn serialize_outputs(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        for output in &self.outputs {
            buf.extend_from_slice(&output.value.to_le_bytes());
            push_var_bytes(&mut buf, &output.script_pubkey);
        }
        buf
    }

    /// Serialize as SegWit format (with marker, flag, witness)
    fn serialize_segwit(&self, witnesses: &[Vec<u8>]) -> Vec<u8> {
        let mut buf = Vec::new();
        // Version
        buf.extend_from_slice(&2u32.to_le_bytes());
        // Marker + Flag
        buf.push(0x00);
        buf.push(0x01);
        // Input count
        push_varint(&mut buf, self.inputs.len() as u64);
        // Inputs
        for input in &self.inputs {
            buf.extend_from_slice(&input.txid);
            buf.extend_from_slice(&input.vout.to_le_bytes());
            buf.push(0x00); // empty scriptSig for SegWit
            buf.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());
        }
        // Output count
        push_varint(&mut buf, self.outputs.len() as u64);
        // Outputs
        for output in &self.outputs {
            buf.extend_from_slice(&output.value.to_le_bytes());
            push_var_bytes(&mut buf, &output.script_pubkey);
        }
        // Witnesses
        for witness in witnesses {
            buf.extend_from_slice(witness);
        }
        // Locktime
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf
    }

    /// Serialize legacy format (no witness) — used for txid calculation
    fn serialize_legacy(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&2u32.to_le_bytes());
        push_varint(&mut buf, self.inputs.len() as u64);
        for input in &self.inputs {
            buf.extend_from_slice(&input.txid);
            buf.extend_from_slice(&input.vout.to_le_bytes());
            buf.push(0x00); // empty scriptSig
            buf.extend_from_slice(&0xFFFFFFFFu32.to_le_bytes());
        }
        push_varint(&mut buf, self.outputs.len() as u64);
        for output in &self.outputs {
            buf.extend_from_slice(&output.value.to_le_bytes());
            push_var_bytes(&mut buf, &output.script_pubkey);
        }
        buf.extend_from_slice(&0u32.to_le_bytes());
        buf
    }
}

/// Parse BTC amount string to satoshi
pub fn parse_btc_to_satoshi(amount: &str) -> Result<u64, String> {
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
        let padded = format!("{:0<8}", decimal_part);
        let trimmed = &padded[..8];
        trimmed.parse().map_err(|_| "Parte decimale non valida")?
    };

    Ok(integer * 100_000_000 + decimals)
}

/// Create P2WPKH output script: OP_0 <20-byte-hash>
pub fn p2wpkh_script(pubkey_hash: &[u8; 20]) -> Vec<u8> {
    let mut script = Vec::with_capacity(22);
    script.push(0x00); // OP_0
    script.push(0x14); // push 20 bytes
    script.extend_from_slice(pubkey_hash);
    script
}

/// Extract script code for BIP-143 from P2WPKH scriptPubKey
/// P2WPKH scriptCode = OP_DUP OP_HASH160 <20-byte-hash> OP_EQUALVERIFY OP_CHECKSIG
fn p2pkh_script_code(script_pubkey: &[u8]) -> Result<Vec<u8>, String> {
    if script_pubkey.len() != 22 || script_pubkey[0] != 0x00 || script_pubkey[1] != 0x14 {
        return Err("Input non è P2WPKH".into());
    }
    let hash = &script_pubkey[2..22];
    let mut code = Vec::with_capacity(25);
    code.push(0x76); // OP_DUP
    code.push(0xa9); // OP_HASH160
    code.push(0x14); // push 20 bytes
    code.extend_from_slice(hash);
    code.push(0x88); // OP_EQUALVERIFY
    code.push(0xac); // OP_CHECKSIG
    Ok(code)
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
    use crate::chains::bitcoin;

    #[test]
    fn test_parse_btc_to_satoshi() {
        assert_eq!(parse_btc_to_satoshi("1").unwrap(), 100_000_000);
        assert_eq!(parse_btc_to_satoshi("0.001").unwrap(), 100_000);
        assert_eq!(parse_btc_to_satoshi("0.00000001").unwrap(), 1);
        assert_eq!(parse_btc_to_satoshi("21000000").unwrap(), 2_100_000_000_000_000);
    }

    #[test]
    fn test_p2wpkh_script() {
        let hash = [0xab; 20];
        let script = p2wpkh_script(&hash);
        assert_eq!(script.len(), 22);
        assert_eq!(script[0], 0x00); // OP_0
        assert_eq!(script[1], 0x14); // push 20
    }

    #[test]
    fn test_der_encode_signature() {
        let r = [0x01; 32];
        let s = [0x02; 32];
        let der = der_encode_signature(&r, &s);
        assert_eq!(der[0], 0x30); // SEQUENCE
        assert_eq!(der[2], 0x02); // first INTEGER
    }

    #[test]
    fn test_build_and_sign_tx() {
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let seed = bip39_utils::mnemonic_to_seed(phrase, "").unwrap();
        let privkey = bitcoin::get_private_key(&seed).unwrap();
        let pubkey = bitcoin::get_public_key(&seed).unwrap();

        // Create a fake UTXO
        let pubkey_hash = hash160_pubkey(&pubkey);
        let utxo = Utxo {
            txid: [0xaa; 32],
            vout: 0,
            value: 100_000, // 0.001 BTC
            script_pubkey: p2wpkh_script(&pubkey_hash),
        };

        let recipient_hash = [0xbb; 20];
        let tx = BitcoinTransaction::build_p2wpkh(
            vec![utxo],
            &recipient_hash,
            50_000,
            &pubkey_hash,
            1_000,
        ).unwrap();

        assert_eq!(tx.outputs.len(), 2); // amount + change

        let signed = tx.sign(&privkey).unwrap();
        assert!(!signed.raw_bytes.is_empty());
        assert_eq!(signed.tx_hash.len(), 64);
    }

    #[test]
    fn test_insufficient_funds() {
        let utxo = Utxo {
            txid: [0; 32],
            vout: 0,
            value: 1_000,
            script_pubkey: p2wpkh_script(&[0; 20]),
        };
        let result = BitcoinTransaction::build_p2wpkh(
            vec![utxo],
            &[0; 20],
            5_000,
            &[0; 20],
            1_000,
        );
        assert!(result.is_err());
    }

    /// Helper: RIPEMD160(SHA256(pubkey))
    fn hash160_pubkey(pubkey: &[u8; 33]) -> [u8; 20] {
        use ripemd::Ripemd160;
        let sha = Sha256::digest(pubkey);
        let hash = Ripemd160::digest(sha);
        let mut result = [0u8; 20];
        result.copy_from_slice(&hash);
        result
    }
}
