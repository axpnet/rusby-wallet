// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// rpc/bitcoin: Balance query, UTXO fetch, fee estimation, and TX broadcast
// Uses mempool.space REST API (no API key required)

use super::get_json;
use serde::Deserialize;

const MEMPOOL_API: &str = "https://mempool.space/api";
const MEMPOOL_SIGNET_API: &str = "https://mempool.space/signet/api";

/// Get mempool API base URL for the given network
pub fn mempool_base_url(testnet: bool) -> &'static str {
    if testnet { MEMPOOL_SIGNET_API } else { MEMPOOL_API }
}

/// Get Bitcoin balance in BTC (formatted string) — mainnet
pub async fn get_balance(address: &str) -> Result<String, String> {
    get_balance_for_network(address, false).await
}

/// Get Bitcoin balance with network selection
pub async fn get_balance_for_network(address: &str, testnet: bool) -> Result<String, String> {
    let base = mempool_base_url(testnet);
    let url = format!("{}/address/{}", base, address);
    let json = get_json(&url).await?;

    let funded: u64 = json["chain_stats"]["funded_txo_sum"]
        .as_u64()
        .unwrap_or(0);
    let spent: u64 = json["chain_stats"]["spent_txo_sum"]
        .as_u64()
        .unwrap_or(0);
    let mempool_funded: u64 = json["mempool_stats"]["funded_txo_sum"]
        .as_u64()
        .unwrap_or(0);
    let mempool_spent: u64 = json["mempool_stats"]["spent_txo_sum"]
        .as_u64()
        .unwrap_or(0);

    let balance_sat = (funded + mempool_funded).saturating_sub(spent + mempool_spent);
    Ok(format_satoshi(balance_sat))
}

/// Fetch UTXOs for an address
#[derive(Debug, Clone, Deserialize)]
pub struct UtxoResponse {
    pub txid: String,
    pub vout: u32,
    pub value: u64,
    pub status: UtxoStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UtxoStatus {
    pub confirmed: bool,
}

pub async fn get_utxos(address: &str) -> Result<Vec<UtxoResponse>, String> {
    let url = format!("{}/address/{}/utxo", MEMPOOL_API, address);
    let json = get_json(&url).await?;

    let utxos: Vec<UtxoResponse> = serde_json::from_value(json)
        .map_err(|e| format!("Errore parsing UTXO: {}", e))?;

    Ok(utxos)
}

/// Get recommended fee rates (sat/vB)
#[derive(Debug, Clone)]
pub struct FeeEstimate {
    pub fastest: u64,  // ~10 min
    pub half_hour: u64,
    pub hour: u64,
    pub economy: u64,
}

pub async fn get_fee_estimates() -> Result<FeeEstimate, String> {
    let url = format!("{}/v1/fees/recommended", MEMPOOL_API);
    let json = get_json(&url).await?;

    Ok(FeeEstimate {
        fastest: json["fastestFee"].as_u64().unwrap_or(10),
        half_hour: json["halfHourFee"].as_u64().unwrap_or(5),
        hour: json["hourFee"].as_u64().unwrap_or(3),
        economy: json["economyFee"].as_u64().unwrap_or(1),
    })
}

/// Broadcast a signed transaction (hex-encoded raw bytes)
pub async fn broadcast_tx(tx_hex: &str) -> Result<String, String> {
    use gloo_net::http::Request;

    let url = format!("{}/tx", MEMPOOL_API);
    let response = Request::post(&url)
        .header("Content-Type", "text/plain")
        .body(tx_hex)
        .map_err(|e| format!("Errore richiesta: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Errore rete: {}", e))?;

    let text = response.text().await
        .map_err(|e| format!("Errore lettura risposta: {}", e))?;

    if response.status() == 200 {
        Ok(text) // returns txid
    } else {
        Err(format!("Broadcast fallito: {}", text))
    }
}

/// Convert UTXO response to wallet-core Utxo struct
pub fn to_core_utxo(utxo: &UtxoResponse, address_script: &[u8]) -> Result<wallet_core::tx::bitcoin::Utxo, String> {
    let txid_bytes = hex::decode(&utxo.txid)
        .map_err(|e| format!("Errore decode txid: {}", e))?;
    if txid_bytes.len() != 32 {
        return Err("txid non valido".into());
    }
    let mut txid = [0u8; 32];
    // Bitcoin txids are displayed in reversed byte order
    for (i, b) in txid_bytes.iter().enumerate() {
        txid[31 - i] = *b;
    }

    Ok(wallet_core::tx::bitcoin::Utxo {
        txid,
        vout: utxo.vout,
        value: utxo.value,
        script_pubkey: address_script.to_vec(),
    })
}

/// Format satoshi to BTC string (e.g. 100000000 → "1.0000")
fn format_satoshi(sat: u64) -> String {
    let btc = sat / 100_000_000;
    let frac = sat % 100_000_000;
    let frac_str = format!("{:08}", frac);
    format!("{}.{}", btc, &frac_str[..4])
}
