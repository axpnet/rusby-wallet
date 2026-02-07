// Rusby Wallet — Dogecoin send logic (P2PKH legacy)
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use wallet_core::tx::dogecoin::*;
use wallet_core::chains::dogecoin as doge_chain;
use zeroize::Zeroize;

/// Default fee: 0.01 DOGE = 1,000,000 satoshi
const DEFAULT_FEE: u64 = 1_000_000;

pub async fn send(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
) -> Result<String, String> {
    let mut private_key = doge_chain::get_private_key(seed)?;
    let pubkey = doge_chain::get_public_key(seed)?;

    let our_hash = doge_chain::hash160_pubkey(&pubkey);
    let our_script = p2pkh_script(&our_hash);

    let from_address = doge_chain::derive_dogecoin_address(seed)?;
    let utxos_resp = crate::rpc::dogecoin::get_utxos(&from_address).await?;

    if utxos_resp.is_empty() {
        return Err("Nessun UTXO disponibile".into());
    }

    let utxos: Vec<DogecoinUtxo> = utxos_resp.iter()
        .map(|u| crate::rpc::dogecoin::to_core_utxo(u, &our_script))
        .collect::<Result<Vec<_>, _>>()?;

    let amount_sat = parse_doge_to_satoshi(amount)?;

    // Decode recipient P2PKH address
    let (to_hash, _version) = doge_chain::decode_p2pkh_address(to)?;

    let tx = DogecoinTransaction::build_p2pkh(
        utxos,
        &to_hash,
        amount_sat,
        &our_hash,
        DEFAULT_FEE,
    )?;

    let signed = tx.sign(&private_key)?;
    private_key.zeroize();
    let tx_hex = hex::encode(&signed.raw_bytes);

    crate::rpc::dogecoin::broadcast_tx(&tx_hex).await
}
