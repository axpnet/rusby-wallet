// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// rpc/dogecoin: Balance query, UTXO fetch, and TX broadcast
// Uses Trezor Blockbook API (https://doge1.trezor.io/api/v2)

use super::get_json;
use serde::Deserialize;

const BLOCKBOOK_API: &str = "https://doge1.trezor.io/api/v2";

/// Get Dogecoin balance in DOGE (formatted string)
pub async fn get_balance(address: &str) -> Result<String, String> {
    get_balance_for_network(address, false).await
}

/// Get Dogecoin balance with network selection
pub async fn get_balance_for_network(address: &str, _testnet: bool) -> Result<String, String> {
    // Blockbook API: GET /address/{addr}
    let url = format!("{}/address/{}", BLOCKBOOK_API, address);
    let json = get_json(&url).await?;

    // balance is in satoshi as string
    let balance_str = json["balance"].as_str().unwrap_or("0");
    let balance_sat: u64 = balance_str.parse().unwrap_or(0);
    Ok(format_dogecoin(balance_sat))
}

/// Fetch UTXOs for an address
#[derive(Debug, Clone, Deserialize)]
pub struct UtxoResponse {
    pub txid: String,
    pub vout: u32,
    pub value: String, // satoshi as string
    #[serde(default)]
    pub confirmations: u64,
}

pub async fn get_utxos(address: &str) -> Result<Vec<UtxoResponse>, String> {
    let url = format!("{}/utxo/{}", BLOCKBOOK_API, address);
    let json = get_json(&url).await?;

    let utxos: Vec<UtxoResponse> = serde_json::from_value(json)
        .map_err(|e| format!("Errore parsing UTXO: {}", e))?;

    Ok(utxos)
}

/// Broadcast a signed transaction (hex-encoded raw bytes)
pub async fn broadcast_tx(tx_hex: &str) -> Result<String, String> {
    let url = format!("{}/sendtx/{}", BLOCKBOOK_API, tx_hex);
    let json = get_json(&url).await?;

    if let Some(txid) = json["result"].as_str() {
        Ok(txid.to_string())
    } else if let Some(err) = json["error"].as_str() {
        Err(format!("Broadcast fallito: {}", err))
    } else {
        Err("Risposta broadcast non valida".into())
    }
}

/// Convert UTXO response to wallet-core DogecoinUtxo struct
pub fn to_core_utxo(
    utxo: &UtxoResponse,
    address_script: &[u8],
) -> Result<wallet_core::tx::dogecoin::DogecoinUtxo, String> {
    let txid_bytes = hex::decode(&utxo.txid)
        .map_err(|e| format!("Errore decode txid: {}", e))?;
    if txid_bytes.len() != 32 {
        return Err("txid non valido".into());
    }
    let mut txid = [0u8; 32];
    // Dogecoin txids are displayed in reversed byte order (same as Bitcoin)
    for (i, b) in txid_bytes.iter().enumerate() {
        txid[31 - i] = *b;
    }

    let value: u64 = utxo.value.parse()
        .map_err(|_| "Valore UTXO non valido")?;

    Ok(wallet_core::tx::dogecoin::DogecoinUtxo {
        txid,
        vout: utxo.vout,
        value,
        script_pubkey: address_script.to_vec(),
    })
}

/// Format satoshi to DOGE string (e.g. 100000000 -> "1.0000")
fn format_dogecoin(satoshi: u64) -> String {
    let doge = satoshi / 100_000_000;
    let frac = satoshi % 100_000_000;
    let frac_str = format!("{:08}", frac);
    format!("{}.{}", doge, &frac_str[..4])
}
