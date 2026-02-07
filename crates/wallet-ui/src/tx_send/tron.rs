// Rusby Wallet — TRON send logic (API-assisted)
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use wallet_core::chains::tron as tron_chain;
use wallet_core::tx::tron;
use zeroize::Zeroize;

pub async fn send(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
    rpc_url: &str,
) -> Result<String, String> {
    let mut private_key = tron_chain::get_private_key(seed)?;

    // Convert addresses to hex format (41... for mainnet)
    let from_address = tron_chain::derive_tron_address(seed)?;
    let from_hex = tron_chain::address_to_hex(&from_address)?;
    let to_hex = tron_chain::address_to_hex(to)?;

    // Parse amount to sun (1 TRX = 1,000,000 sun)
    let amount_sun = tron::parse_trx_to_sun(amount)?;

    // 1. Create transaction via TronGrid API
    let (tx_id, raw_data, raw_data_hex) = crate::rpc::tron::create_transaction(
        rpc_url, &from_hex, &to_hex, amount_sun,
    ).await?;

    // 2. SECURITY: Verify TX params before signing
    tron::verify_tron_tx_params(&raw_data, &to_hex, amount_sun)?;

    // 3. Sign the transaction locally
    let signature = tron::sign_tron_tx(&private_key, &raw_data_hex)?;
    private_key.zeroize();

    // 4. Broadcast the signed transaction
    crate::rpc::tron::broadcast_transaction(
        rpc_url, &tx_id, &raw_data, &raw_data_hex, &signature,
    ).await
}
