// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// tx/cosmos: Cosmos SDK MsgSend construction and secp256k1 signing (Amino JSON)

use k256::ecdsa::{SigningKey, signature::hazmat::PrehashSigner};
use sha2::{Digest, Sha256};
use super::SignedTransaction;
use crate::chains::ChainId;

/// Cosmos bank send transaction
#[derive(Debug, Clone)]
pub struct CosmosMsgSend {
    pub from_address: String,
    pub to_address: String,
    pub amount: u64,
    pub denom: String,
    pub chain_id_str: String,
    pub account_number: u64,
    pub sequence: u64,
    pub gas_limit: u64,
    pub fee_amount: u64,
    pub fee_denom: String,
}

impl CosmosMsgSend {
    /// Build the Amino JSON sign doc
    fn build_sign_doc(&self) -> String {
        // Amino JSON canonical signing format
        serde_json::json!({
            "account_number": self.account_number.to_string(),
            "chain_id": self.chain_id_str,
            "fee": {
                "amount": [{
                    "amount": self.fee_amount.to_string(),
                    "denom": &self.fee_denom,
                }],
                "gas": self.gas_limit.to_string(),
            },
            "memo": "",
            "msgs": [{
                "type": "cosmos-sdk/MsgSend",
                "value": {
                    "amount": [{
                        "amount": self.amount.to_string(),
                        "denom": &self.denom,
                    }],
                    "from_address": &self.from_address,
                    "to_address": &self.to_address,
                }
            }],
            "sequence": self.sequence.to_string(),
        }).to_string()
    }

    /// Sign with secp256k1 private key
    pub fn sign(&self, private_key: &[u8; 32], chain_id: ChainId) -> Result<SignedTransaction, String> {
        let sign_doc = self.build_sign_doc();

        // SHA256 hash of the sign doc
        let mut hasher = Sha256::new();
        hasher.update(sign_doc.as_bytes());
        let hash = hasher.finalize();
        let hash_bytes: [u8; 32] = hash.into();

        // Sign with secp256k1
        let signing_key = SigningKey::from_bytes(private_key.into())
            .map_err(|e| format!("Invalid key: {}", e))?;
        let (signature, _) = signing_key
            .sign_prehash(&hash_bytes)
            .map_err(|e| format!("Signing error: {}", e))?;

        let sig_bytes = signature.to_bytes();

        // Build broadcast-ready Amino JSON tx
        let public_key = signing_key.verifying_key();
        let pubkey_compressed = public_key.to_encoded_point(true);
        let pubkey_b64 = base64_encode(pubkey_compressed.as_bytes());
        let sig_b64 = base64_encode(&sig_bytes);

        let tx_json = serde_json::json!({
            "tx": {
                "msg": [{
                    "type": "cosmos-sdk/MsgSend",
                    "value": {
                        "amount": [{
                            "amount": self.amount.to_string(),
                            "denom": &self.denom,
                        }],
                        "from_address": &self.from_address,
                        "to_address": &self.to_address,
                    }
                }],
                "fee": {
                    "amount": [{
                        "amount": self.fee_amount.to_string(),
                        "denom": &self.fee_denom,
                    }],
                    "gas": self.gas_limit.to_string(),
                },
                "signatures": [{
                    "pub_key": {
                        "type": "tendermint/PubKeySecp256k1",
                        "value": pubkey_b64,
                    },
                    "signature": sig_b64,
                }],
                "memo": "",
            },
            "mode": "sync",
        });

        let tx_bytes = tx_json.to_string().into_bytes();

        // Tx hash = SHA256 of the sign doc
        let tx_hash = format!("0x{}", hex::encode(&hash_bytes));

        Ok(SignedTransaction {
            chain_id,
            raw_bytes: tx_bytes,
            tx_hash,
        })
    }
}

/// Cosmos wasm MsgExecuteContract transaction (CW-20 transfers)
#[derive(Debug, Clone)]
pub struct CosmosMsgExecuteContract {
    pub sender: String,
    pub contract: String,
    pub msg_json: String,
    pub chain_id_str: String,
    pub account_number: u64,
    pub sequence: u64,
    pub gas_limit: u64,
    pub fee_amount: u64,
    pub fee_denom: String,
}

impl CosmosMsgExecuteContract {
    /// Build the Amino JSON sign doc for MsgExecuteContract
    fn build_sign_doc(&self) -> String {
        // Parse msg_json to include as nested JSON
        let msg_value: serde_json::Value = serde_json::from_str(&self.msg_json)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        serde_json::json!({
            "account_number": self.account_number.to_string(),
            "chain_id": self.chain_id_str,
            "fee": {
                "amount": [{
                    "amount": self.fee_amount.to_string(),
                    "denom": &self.fee_denom,
                }],
                "gas": self.gas_limit.to_string(),
            },
            "memo": "",
            "msgs": [{
                "type": "wasm/MsgExecuteContract",
                "value": {
                    "contract": &self.contract,
                    "funds": [],
                    "msg": msg_value,
                    "sender": &self.sender,
                }
            }],
            "sequence": self.sequence.to_string(),
        }).to_string()
    }

    /// Sign with secp256k1 private key
    pub fn sign(&self, private_key: &[u8; 32], chain_id: ChainId) -> Result<SignedTransaction, String> {
        let sign_doc = self.build_sign_doc();

        let msg_value: serde_json::Value = serde_json::from_str(&self.msg_json)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        let mut hasher = Sha256::new();
        hasher.update(sign_doc.as_bytes());
        let hash = hasher.finalize();
        let hash_bytes: [u8; 32] = hash.into();

        let signing_key = SigningKey::from_bytes(private_key.into())
            .map_err(|e| format!("Invalid key: {}", e))?;
        let (signature, _) = signing_key
            .sign_prehash(&hash_bytes)
            .map_err(|e| format!("Signing error: {}", e))?;

        let sig_bytes = signature.to_bytes();

        let public_key = signing_key.verifying_key();
        let pubkey_compressed = public_key.to_encoded_point(true);
        let pubkey_b64 = base64_encode(pubkey_compressed.as_bytes());
        let sig_b64 = base64_encode(&sig_bytes);

        let tx_json = serde_json::json!({
            "tx": {
                "msg": [{
                    "type": "wasm/MsgExecuteContract",
                    "value": {
                        "contract": &self.contract,
                        "funds": [],
                        "msg": msg_value,
                        "sender": &self.sender,
                    }
                }],
                "fee": {
                    "amount": [{
                        "amount": self.fee_amount.to_string(),
                        "denom": &self.fee_denom,
                    }],
                    "gas": self.gas_limit.to_string(),
                },
                "signatures": [{
                    "pub_key": {
                        "type": "tendermint/PubKeySecp256k1",
                        "value": pubkey_b64,
                    },
                    "signature": sig_b64,
                }],
                "memo": "",
            },
            "mode": "sync",
        });

        let tx_bytes = tx_json.to_string().into_bytes();
        let tx_hash = format!("0x{}", hex::encode(&hash_bytes));

        Ok(SignedTransaction {
            chain_id,
            raw_bytes: tx_bytes,
            tx_hash,
        })
    }
}

/// Parse uatom/uosmo from decimal amount (6 decimals)
pub fn parse_atom_to_uatom(amount: &str) -> Result<u64, String> {
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
        let padded = format!("{:0<6}", decimal_part);
        let trimmed = &padded[..6];
        trimmed.parse().map_err(|_| "Invalid decimal part")?
    };

    Ok(integer * 1_000_000 + decimals)
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
    fn test_parse_atom_to_uatom() {
        assert_eq!(parse_atom_to_uatom("1").unwrap(), 1_000_000);
        assert_eq!(parse_atom_to_uatom("0.5").unwrap(), 500_000);
        assert_eq!(parse_atom_to_uatom("0.000001").unwrap(), 1);
    }

    #[test]
    fn test_sign_cosmos_msg_send() {
        let msg = CosmosMsgSend {
            from_address: "cosmos1test".into(),
            to_address: "cosmos1dest".into(),
            amount: 1_000_000,
            denom: "uatom".into(),
            chain_id_str: "cosmoshub-4".into(),
            account_number: 0,
            sequence: 0,
            gas_limit: 200000,
            fee_amount: 5000,
            fee_denom: "uatom".into(),
        };
        let key = [6u8; 32];
        let signed = msg.sign(&key, ChainId::CosmosHub).unwrap();
        assert!(!signed.raw_bytes.is_empty());
        assert!(signed.tx_hash.starts_with("0x"));
    }
}
