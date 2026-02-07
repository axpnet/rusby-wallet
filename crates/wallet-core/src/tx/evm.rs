// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// tx/evm: EIP-1559 transaction construction, RLP encoding, secp256k1 signing

use k256::ecdsa::{SigningKey, signature::hazmat::PrehashSigner};
use tiny_keccak::{Hasher, Keccak};

use super::SignedTransaction;
use crate::chains::ChainId;

/// EIP-1559 (Type 2) transaction
#[derive(Debug, Clone)]
pub struct EvmTransaction {
    pub chain_id_num: u64,
    pub nonce: u64,
    pub max_priority_fee_per_gas: u128,
    pub max_fee_per_gas: u128,
    pub gas_limit: u64,
    pub to: [u8; 20],
    pub value: u128,
    pub data: Vec<u8>,
}

impl EvmTransaction {
    /// Encode for signing (without signature fields)
    fn encode_unsigned(&self) -> Vec<u8> {
        // Build RLP payload manually to correctly encode access_list as empty list (0xc0)
        // instead of empty byte string (0x80)
        let mut payload = Vec::new();
        payload.extend_from_slice(&rlp_encode_bytes(&rlp_encode_u64(self.chain_id_num)));
        payload.extend_from_slice(&rlp_encode_bytes(&rlp_encode_u64(self.nonce)));
        payload.extend_from_slice(&rlp_encode_bytes(&rlp_encode_u128(self.max_priority_fee_per_gas)));
        payload.extend_from_slice(&rlp_encode_bytes(&rlp_encode_u128(self.max_fee_per_gas)));
        payload.extend_from_slice(&rlp_encode_bytes(&rlp_encode_u64(self.gas_limit)));
        payload.extend_from_slice(&rlp_encode_bytes(&self.to.to_vec()));
        payload.extend_from_slice(&rlp_encode_bytes(&rlp_encode_u128(self.value)));
        payload.extend_from_slice(&rlp_encode_bytes(&self.data));
        payload.push(0xc0); // access_list: RLP empty list (NOT byte string 0x80)

        let rlp = rlp_wrap_list_payload(&payload);
        // EIP-1559: 0x02 || RLP(...)
        let mut result = Vec::with_capacity(1 + rlp.len());
        result.push(0x02);
        result.extend_from_slice(&rlp);
        result
    }

    /// Sign the transaction with a private key
    pub fn sign(&self, private_key: &[u8; 32], chain_id: ChainId) -> Result<SignedTransaction, String> {
        let unsigned = self.encode_unsigned();

        // Hash: keccak256(0x02 || RLP(unsigned))
        let mut hasher = Keccak::v256();
        let mut hash = [0u8; 32];
        hasher.update(&unsigned);
        hasher.finalize(&mut hash);

        // Sign with secp256k1
        let signing_key = SigningKey::from_bytes(private_key.into())
            .map_err(|e| format!("Invalid key: {}", e))?;
        let (signature, recovery_id) = signing_key
            .sign_prehash(&hash)
            .map_err(|e| format!("Signing error: {}", e))?;

        let sig_bytes = signature.to_bytes();
        let r = &sig_bytes[..32];
        let s = &sig_bytes[32..];
        let v = recovery_id.to_byte();

        // Encode signed: 0x02 || RLP([chain_id, nonce, max_priority_fee, max_fee, gas_limit, to, value, data, access_list, v, r, s])
        let mut payload = Vec::new();
        payload.extend_from_slice(&rlp_encode_bytes(&rlp_encode_u64(self.chain_id_num)));
        payload.extend_from_slice(&rlp_encode_bytes(&rlp_encode_u64(self.nonce)));
        payload.extend_from_slice(&rlp_encode_bytes(&rlp_encode_u128(self.max_priority_fee_per_gas)));
        payload.extend_from_slice(&rlp_encode_bytes(&rlp_encode_u128(self.max_fee_per_gas)));
        payload.extend_from_slice(&rlp_encode_bytes(&rlp_encode_u64(self.gas_limit)));
        payload.extend_from_slice(&rlp_encode_bytes(&self.to.to_vec()));
        payload.extend_from_slice(&rlp_encode_bytes(&rlp_encode_u128(self.value)));
        payload.extend_from_slice(&rlp_encode_bytes(&self.data));
        payload.push(0xc0); // access_list: RLP empty list
        payload.extend_from_slice(&rlp_encode_bytes(&vec![v]));
        payload.extend_from_slice(&rlp_encode_bytes(&r.to_vec()));
        payload.extend_from_slice(&rlp_encode_bytes(&s.to_vec()));

        let rlp_signed = rlp_wrap_list_payload(&payload);
        let mut raw = Vec::with_capacity(1 + rlp_signed.len());
        raw.push(0x02);
        raw.extend_from_slice(&rlp_signed);

        // Tx hash = keccak256(signed raw)
        let mut tx_hasher = Keccak::v256();
        let mut tx_hash = [0u8; 32];
        tx_hasher.update(&raw);
        tx_hasher.finalize(&mut tx_hash);

        Ok(SignedTransaction {
            chain_id,
            raw_bytes: raw,
            tx_hash: format!("0x{}", hex::encode(tx_hash)),
        })
    }
}

/// Parse an EVM address string (0x...) to 20 bytes
pub fn parse_address(addr: &str) -> Result<[u8; 20], String> {
    let hex_str = addr.strip_prefix("0x").unwrap_or(addr);
    if hex_str.len() != 40 {
        return Err("Invalid EVM address length".into());
    }
    let bytes = hex::decode(hex_str).map_err(|e| format!("Invalid hex: {}", e))?;
    let mut result = [0u8; 20];
    result.copy_from_slice(&bytes);
    Ok(result)
}

/// Parse a decimal amount string to wei (u128)
pub fn parse_ether_to_wei(amount: &str) -> Result<u128, String> {
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

    let decimals = if decimal_part.is_empty() {
        0u128
    } else {
        let padded = format!("{:0<18}", decimal_part);
        let trimmed = &padded[..18];
        trimmed.parse().map_err(|_| "Invalid decimal part")?
    };

    integer.checked_mul(1_000_000_000_000_000_000u128)
        .and_then(|v| v.checked_add(decimals))
        .ok_or_else(|| "Amount overflow".to_string())
}

// --- RLP encoding helpers ---

fn rlp_encode_u64(val: u64) -> Vec<u8> {
    if val == 0 {
        return vec![];
    }
    let bytes = val.to_be_bytes();
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(8);
    bytes[start..].to_vec()
}

fn rlp_encode_u128(val: u128) -> Vec<u8> {
    if val == 0 {
        return vec![];
    }
    let bytes = val.to_be_bytes();
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(16);
    bytes[start..].to_vec()
}

fn rlp_encode_bytes(data: &[u8]) -> Vec<u8> {
    if data.len() == 1 && data[0] < 0x80 {
        return data.to_vec();
    }
    if data.is_empty() {
        return vec![0x80];
    }
    if data.len() <= 55 {
        let mut result = vec![0x80 + data.len() as u8];
        result.extend_from_slice(data);
        result
    } else {
        let len_bytes = rlp_encode_u64(data.len() as u64);
        let mut result = vec![0xb7 + len_bytes.len() as u8];
        result.extend_from_slice(&len_bytes);
        result.extend_from_slice(data);
        result
    }
}

/// Wrap pre-encoded RLP payload bytes in a list envelope
/// Used when payload contains pre-encoded items (e.g., access_list as 0xc0)
fn rlp_wrap_list_payload(payload: &[u8]) -> Vec<u8> {
    if payload.len() <= 55 {
        let mut result = vec![0xc0 + payload.len() as u8];
        result.extend_from_slice(payload);
        result
    } else {
        let len_bytes = rlp_encode_u64(payload.len() as u64);
        let mut result = vec![0xf7 + len_bytes.len() as u8];
        result.extend_from_slice(&len_bytes);
        result.extend_from_slice(payload);
        result
    }
}

fn rlp_encode_list(items: &[Vec<u8>]) -> Vec<u8> {
    let mut payload = Vec::new();
    for item in items {
        payload.extend_from_slice(&rlp_encode_bytes(item));
    }
    if payload.len() <= 55 {
        let mut result = vec![0xc0 + payload.len() as u8];
        result.extend_from_slice(&payload);
        result
    } else {
        let len_bytes = rlp_encode_u64(payload.len() as u64);
        let mut result = vec![0xf7 + len_bytes.len() as u8];
        result.extend_from_slice(&len_bytes);
        result.extend_from_slice(&payload);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        let addr = parse_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
        assert_eq!(addr[0], 0xd8);
        assert_eq!(addr[19], 0x45);
    }

    #[test]
    fn test_parse_ether_to_wei() {
        assert_eq!(parse_ether_to_wei("1").unwrap(), 1_000_000_000_000_000_000u128);
        assert_eq!(parse_ether_to_wei("0.1").unwrap(), 100_000_000_000_000_000u128);
        assert_eq!(parse_ether_to_wei("0.001").unwrap(), 1_000_000_000_000_000u128);
    }

    #[test]
    fn test_rlp_encode_bytes() {
        assert_eq!(rlp_encode_bytes(&[]), vec![0x80]);
        assert_eq!(rlp_encode_bytes(&[0x01]), vec![0x01]);
        assert_eq!(rlp_encode_bytes(&[0x80]), vec![0x81, 0x80]);
    }

    #[test]
    fn test_sign_evm_tx() {
        let tx = EvmTransaction {
            chain_id_num: 1,
            nonce: 0,
            max_priority_fee_per_gas: 1_000_000_000,
            max_fee_per_gas: 20_000_000_000,
            gas_limit: 21000,
            to: [0u8; 20],
            value: 1_000_000_000_000_000_000, // 1 ETH
            data: vec![],
        };
        // Use a test private key
        let key = [1u8; 32];
        let signed = tx.sign(&key, ChainId::Ethereum).unwrap();
        assert!(signed.tx_hash.starts_with("0x"));
        assert!(!signed.raw_bytes.is_empty());
    }
}
