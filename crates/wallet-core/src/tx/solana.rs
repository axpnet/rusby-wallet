// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// tx/solana: Solana SystemProgram transfer construction and Ed25519 signing

use ed25519_dalek::{Signer, SigningKey};

use super::SignedTransaction;
use crate::chains::ChainId;

/// Solana native SOL transfer (SystemProgram.transfer)
#[derive(Debug, Clone)]
pub struct SolanaTransfer {
    pub from_pubkey: [u8; 32],
    pub to_pubkey: [u8; 32],
    pub lamports: u64,
    pub recent_blockhash: [u8; 32],
}

impl SolanaTransfer {
    /// Build the compact Solana transaction message (v0 legacy)
    fn build_message(&self) -> Vec<u8> {
        // SystemProgram ID = [0u8; 32]
        let system_program = [0u8; 32];

        // Message header: [num_required_signatures, num_readonly_signed, num_readonly_unsigned]
        let header = [1u8, 0, 1]; // 1 signer (from), 0 readonly signed, 1 readonly unsigned (system program)

        // Account keys: [from, to, system_program]
        let num_accounts: u8 = 3;

        // Instructions: 1 instruction
        // SystemProgram.Transfer = instruction index 2
        // Data: transfer instruction (index 2 as u32 LE + lamports as u64 LE)
        let mut instruction_data = Vec::with_capacity(12);
        instruction_data.extend_from_slice(&2u32.to_le_bytes()); // Transfer instruction
        instruction_data.extend_from_slice(&self.lamports.to_le_bytes());

        let mut message = Vec::new();
        // Header
        message.extend_from_slice(&header);
        // Compact array: num accounts
        message.push(num_accounts);
        // Account keys
        message.extend_from_slice(&self.from_pubkey);
        message.extend_from_slice(&self.to_pubkey);
        message.extend_from_slice(&system_program);
        // Recent blockhash
        message.extend_from_slice(&self.recent_blockhash);
        // Compact array: num instructions = 1
        message.push(1);
        // Instruction: program_id_index, accounts, data
        message.push(2); // program_id_index = 2 (system program)
        message.push(2); // num account indices
        message.push(0); // from account index
        message.push(1); // to account index
        // Instruction data length + data
        message.push(instruction_data.len() as u8);
        message.extend_from_slice(&instruction_data);

        message
    }

    /// Sign the transfer with an Ed25519 keypair
    pub fn sign(&self, private_key: &[u8; 32]) -> Result<SignedTransaction, String> {
        let signing_key = SigningKey::from_bytes(private_key);
        let message = self.build_message();

        let signature = signing_key.sign(&message);
        let sig_bytes = signature.to_bytes();

        // Full transaction: [num_signatures(compact), signature(64), message]
        let mut tx = Vec::new();
        tx.push(1u8); // 1 signature
        tx.extend_from_slice(&sig_bytes);
        tx.extend_from_slice(&message);

        // Tx hash = first signature as base58
        let tx_hash = bs58::encode(&sig_bytes).into_string();

        Ok(SignedTransaction {
            chain_id: ChainId::Solana,
            raw_bytes: tx,
            tx_hash,
        })
    }
}

/// Parse a base58 Solana address to 32 bytes
pub fn parse_pubkey(addr: &str) -> Result<[u8; 32], String> {
    let bytes = bs58::decode(addr)
        .into_vec()
        .map_err(|e| format!("Invalid base58: {}", e))?;
    if bytes.len() != 32 {
        return Err(format!("Invalid pubkey length: {} (expected 32)", bytes.len()));
    }
    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    Ok(result)
}

/// Parse SOL amount to lamports
pub fn parse_sol_to_lamports(amount: &str) -> Result<u64, String> {
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
    fn test_parse_sol_to_lamports() {
        assert_eq!(parse_sol_to_lamports("1").unwrap(), 1_000_000_000);
        assert_eq!(parse_sol_to_lamports("0.5").unwrap(), 500_000_000);
        assert_eq!(parse_sol_to_lamports("0.000000001").unwrap(), 1);
    }

    #[test]
    fn test_sign_solana_transfer() {
        let transfer = SolanaTransfer {
            from_pubkey: [1u8; 32],
            to_pubkey: [2u8; 32],
            lamports: 1_000_000_000,
            recent_blockhash: [3u8; 32],
        };
        let key = [4u8; 32];
        let signed = transfer.sign(&key).unwrap();
        assert!(!signed.tx_hash.is_empty());
        assert!(!signed.raw_bytes.is_empty());
        // First byte = 1 (num signatures)
        assert_eq!(signed.raw_bytes[0], 1);
    }
}
