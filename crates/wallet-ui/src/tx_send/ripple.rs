// Rusby Wallet — XRP send logic
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use wallet_core::tx::ripple::*;
use wallet_core::chains::ripple as xrp_chain;
use zeroize::Zeroize;

pub async fn send(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
    rpc_url: &str,
) -> Result<String, String> {
    let mut private_key = xrp_chain::get_private_key(seed)?;
    let pubkey = xrp_chain::get_public_key(seed)?;
    let account_id = xrp_chain::get_account_id(seed)?;

    // Decode destination address
    let destination = xrp_chain::decode_address(to)?;

    // Get account sequence
    let from_address = xrp_chain::derive_ripple_address(seed)?;
    let sequence = crate::rpc::ripple::get_account_sequence(&from_address, rpc_url).await?;

    // Get current fee
    let fee_drops = crate::rpc::ripple::get_fee(rpc_url).await.unwrap_or(12);

    let amount_drops = parse_xrp_to_drops(amount)?;

    let tx = RippleTransaction {
        account: account_id,
        destination,
        amount_drops,
        fee_drops,
        sequence,
        signing_pubkey: pubkey,
    };

    let signed = tx.sign(&private_key)?;
    private_key.zeroize();

    let tx_hex = hex::encode(&signed.raw_bytes);
    crate::rpc::ripple::submit_tx(&tx_hex, rpc_url).await
}
