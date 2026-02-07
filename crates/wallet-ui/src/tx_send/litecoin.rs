// Rusby Wallet — Litecoin send logic
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use wallet_core::tx::litecoin::*;
use wallet_core::chains::litecoin as ltc_chain;
use zeroize::Zeroize;

pub async fn send(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
) -> Result<String, String> {
    let mut private_key = ltc_chain::get_private_key(seed)?;
    let pubkey = ltc_chain::get_public_key(seed)?;

    let our_hash = ltc_chain::hash160_pubkey(&pubkey);
    let our_script = p2wpkh_script(&our_hash);

    let from_address = ltc_chain::derive_litecoin_address(seed)?;
    let utxos_resp = crate::rpc::litecoin::get_utxos(&from_address, false).await?;

    if utxos_resp.is_empty() {
        return Err("Nessun UTXO disponibile".into());
    }

    let utxos: Vec<Utxo> = utxos_resp.iter()
        .map(|u| crate::rpc::litecoin::to_core_utxo(u, &our_script))
        .collect::<Result<Vec<_>, _>>()?;

    let amount_litoshi = parse_ltc_to_litoshi(amount)?;

    let fees = crate::rpc::litecoin::get_fee_estimates(false).await
        .unwrap_or(crate::rpc::litecoin::FeeEstimate {
            fastest: 10, half_hour: 5, hour: 3, economy: 1,
        });

    let estimated_vsize = 141u64 * utxos.len().max(1) as u64;
    let fee = fees.half_hour * estimated_vsize;

    let to_hash = ltc_chain::decode_bech32_address(to)?;

    let tx = BitcoinTransaction::build_p2wpkh(
        utxos,
        &to_hash,
        amount_litoshi,
        &our_hash,
        fee,
    )?;

    let signed = tx.sign_for_chain(&private_key, wallet_core::chains::ChainId::Litecoin)?;
    private_key.zeroize();
    let tx_hex = hex::encode(&signed.raw_bytes);

    crate::rpc::litecoin::broadcast_tx(&tx_hex, false).await
}
