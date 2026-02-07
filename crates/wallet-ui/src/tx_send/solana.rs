// Rusby Wallet — Solana send logic
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use wallet_core::tx::solana::*;
use wallet_core::chains::solana as sol_chain;
use zeroize::Zeroize;

pub async fn send(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
    rpc_url: &str,
) -> Result<String, String> {
    let mut keypair = sol_chain::get_keypair(seed)?;
    let mut private_key: [u8; 32] = keypair[..32].try_into().unwrap();
    let from_pubkey: [u8; 32] = keypair[32..].try_into().unwrap();
    keypair.zeroize();
    let to_pubkey = parse_pubkey(to)?;
    let lamports = parse_sol_to_lamports(amount)?;

    let blockhash_b58 = crate::rpc::solana::get_latest_blockhash(rpc_url).await?;
    let blockhash_bytes = bs58::decode(&blockhash_b58).into_vec()
        .map_err(|e| format!("Invalid blockhash: {}", e))?;
    let mut recent_blockhash = [0u8; 32];
    if blockhash_bytes.len() != 32 {
        return Err("Invalid blockhash length".into());
    }
    recent_blockhash.copy_from_slice(&blockhash_bytes);

    let transfer = SolanaTransfer {
        from_pubkey,
        to_pubkey,
        lamports,
        recent_blockhash,
    };

    let signed = transfer.sign(&private_key)?;
    private_key.zeroize();
    let signed_b58 = bs58::encode(&signed.raw_bytes).into_string();

    crate::rpc::solana::send_transaction(&signed_b58, rpc_url).await
}
