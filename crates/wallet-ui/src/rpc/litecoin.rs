// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// rpc/litecoin: Balance query, UTXO fetch, fee estimation, and TX broadcast
// Uses litecoinspace.org REST API (mempool.space clone for Litecoin)

use super::get_json;
use serde::Deserialize;

const LITECOINSPACE_API: &str = "https://litecoinspace.org/api";
const LITECOINSPACE_TESTNET_API: &str = "https://litecoinspace.org/testnet/api";

/// Get litecoinspace API base URL for the given network
pub fn litecoinspace_base_url(testnet: bool) -> &'static str {
    if testnet { LITECOINSPACE_TESTNET_API } else { LITECOINSPACE_API }
}

/// Get Litecoin balance in LTC (formatted string) — mainnet
pub async fn get_balance(address: &str) -> Result<String, String> {
    get_balance_for_network(address, false).await
}

/// Get Litecoin balance with network selection
pub async fn get_balance_for_network(address: &str, testnet: bool) -> Result<String, String> {
    let base = litecoinspace_base_url(testnet);
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

    let balance_litoshi = (funded + mempool_funded).saturating_sub(spent + mempool_spent);
    Ok(format_litoshi(balance_litoshi))
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

pub async fn get_utxos(address: &str, testnet: bool) -> Result<Vec<UtxoResponse>, String> {
    let base = litecoinspace_base_url(testnet);
    let url = format!("{}/address/{}/utxo", base, address);
    let json = get_json(&url).await?;

    let utxos: Vec<UtxoResponse> = serde_json::from_value(json)
        .map_err(|e| format!("Errore parsing UTXO: {}", e))?;

    Ok(utxos)
}

/// Get recommended fee rates (sat/vB)
#[derive(Debug, Clone)]
pub struct FeeEstimate {
    pub fastest: u64,
    pub half_hour: u64,
    pub hour: u64,
    pub economy: u64,
}

pub async fn get_fee_estimates(testnet: bool) -> Result<FeeEstimate, String> {
    let base = litecoinspace_base_url(testnet);
    let url = format!("{}/v1/fees/recommended", base);
    let json = get_json(&url).await?;

    Ok(FeeEstimate {
        fastest: json["fastestFee"].as_u64().unwrap_or(10),
        half_hour: json["halfHourFee"].as_u64().unwrap_or(5),
        hour: json["hourFee"].as_u64().unwrap_or(3),
        economy: json["economyFee"].as_u64().unwrap_or(1),
    })
}

/// Broadcast a signed transaction (hex-encoded raw bytes)
pub async fn broadcast_tx(tx_hex: &str, testnet: bool) -> Result<String, String> {
    use gloo_net::http::Request;

    let base = litecoinspace_base_url(testnet);
    let url = format!("{}/tx", base);
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
pub fn to_core_utxo(utxo: &UtxoResponse, address_script: &[u8]) -> Result<wallet_core::tx::litecoin::Utxo, String> {
    let txid_bytes = hex::decode(&utxo.txid)
        .map_err(|e| format!("Errore decode txid: {}", e))?;
    if txid_bytes.len() != 32 {
        return Err("txid non valido".into());
    }
    let mut txid = [0u8; 32];
    // Litecoin txids are displayed in reversed byte order (same as Bitcoin)
    for (i, b) in txid_bytes.iter().enumerate() {
        txid[31 - i] = *b;
    }

    Ok(wallet_core::tx::litecoin::Utxo {
        txid,
        vout: utxo.vout,
        value: utxo.value,
        script_pubkey: address_script.to_vec(),
    })
}

/// Format litoshi to LTC string (e.g. 100000000 → "1.0000")
fn format_litoshi(litoshi: u64) -> String {
    let ltc = litoshi / 100_000_000;
    let frac = litoshi % 100_000_000;
    let frac_str = format!("{:08}", frac);
    format!("{}.{}", ltc, &frac_str[..4])
}
