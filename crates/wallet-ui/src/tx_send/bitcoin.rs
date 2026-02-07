// Rusby Wallet — Bitcoin send logic
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use wallet_core::tx::bitcoin::*;
use wallet_core::chains::bitcoin as btc_chain;
use zeroize::Zeroize;

pub async fn send(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
) -> Result<String, String> {
    let mut private_key = btc_chain::get_private_key(seed)?;
    let pubkey = btc_chain::get_public_key(seed)?;

    let our_hash = btc_chain::hash160_pubkey(&pubkey);
    let our_script = p2wpkh_script(&our_hash);

    let from_address = btc_chain::derive_bitcoin_address(seed)?;
    let utxos_resp = crate::rpc::bitcoin::get_utxos(&from_address).await?;

    if utxos_resp.is_empty() {
        return Err("Nessun UTXO disponibile".into());
    }

    let utxos: Vec<Utxo> = utxos_resp.iter()
        .map(|u| crate::rpc::bitcoin::to_core_utxo(u, &our_script))
        .collect::<Result<Vec<_>, _>>()?;

    let amount_sat = parse_btc_to_satoshi(amount)?;

    let fees = crate::rpc::bitcoin::get_fee_estimates().await
        .unwrap_or(crate::rpc::bitcoin::FeeEstimate {
            fastest: 10, half_hour: 5, hour: 3, economy: 1,
        });

    let estimated_vsize = 141u64 * utxos.len().max(1) as u64;
    let fee = fees.half_hour * estimated_vsize;

    let to_hash = btc_chain::decode_bech32_address(to)?;

    let tx = BitcoinTransaction::build_p2wpkh(
        utxos,
        &to_hash,
        amount_sat,
        &our_hash,
        fee,
    )?;

    let signed = tx.sign(&private_key)?;
    private_key.zeroize();
    let tx_hex = hex::encode(&signed.raw_bytes);

    crate::rpc::bitcoin::broadcast_tx(&tx_hex).await
}
