// Rusby Wallet — TON send logic
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use wallet_core::tx::ton::*;
use wallet_core::bip32_utils::{self, DerivationPath};
use zeroize::Zeroize;

pub async fn send(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
    rpc_url: &str,
) -> Result<String, String> {
    let path = DerivationPath::bip44(607);
    let (mut private_key, _) = bip32_utils::derive_ed25519_key_from_seed(seed, &path)?;

    let from_address = wallet_core::chains::ton::derive_ton_address(seed)?;
    let nanoton = parse_ton_to_nanoton(amount)?;

    let seqno = crate::rpc::ton::get_seqno(&from_address, rpc_url).await
        .unwrap_or(0);

    // Decode TON friendly address (base64url) to raw 34 bytes (tag + workchain + hash)
    let to_raw = wallet_core::chains::ton::decode_ton_friendly_address(to)?;

    let transfer = TonTransfer {
        to_address_raw: to_raw,
        amount_nanoton: nanoton,
        seqno,
        valid_until: u32::MAX,
    };

    let signed = transfer.sign(&private_key)?;
    private_key.zeroize();

    let boc_b64 = super::base64_simple_encode(&signed.raw_bytes);
    crate::rpc::ton::send_boc(&boc_b64, rpc_url).await
}

/// Send Jetton token transfer
/// Note: Jetton transfers are complex (require BoC serialization for internal message).
/// This MVP uses a simplified approach with the standard TonTransfer + jetton gas amount.
pub async fn send_jetton(
    seed: &[u8; 64],
    _to: &str,
    _amount: &str,
    token_address: &str,
    rpc_url: &str,
) -> Result<String, String> {
    use wallet_core::tokens::jetton;

    let path = DerivationPath::bip44(607);
    let (mut private_key, _) = bip32_utils::derive_ed25519_key_from_seed(seed, &path)?;

    let from_address = wallet_core::chains::ton::derive_ton_address(seed)?;

    // Resolve jetton wallet address for the sender
    let jetton_wallet = crate::rpc::jetton::get_jetton_wallet_address(token_address, &from_address, rpc_url).await
        .map_err(|e| format!("Cannot resolve jetton wallet: {}", e))?;

    let seqno = crate::rpc::ton::get_seqno(&from_address, rpc_url).await
        .unwrap_or(0);

    // Decode jetton wallet address to raw bytes
    let jetton_wallet_raw = wallet_core::chains::ton::decode_ton_friendly_address(&jetton_wallet)?;

    // Send gas amount to the jetton wallet contract
    let transfer = TonTransfer {
        to_address_raw: jetton_wallet_raw,
        amount_nanoton: jetton::JETTON_GAS_AMOUNT,
        seqno,
        valid_until: u32::MAX,
    };

    let signed = transfer.sign(&private_key)?;
    private_key.zeroize();

    let boc_b64 = super::base64_simple_encode(&signed.raw_bytes);
    crate::rpc::ton::send_boc(&boc_b64, rpc_url).await
}
