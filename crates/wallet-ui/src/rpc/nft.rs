// Rusby Wallet — NFT RPC client (Alchemy + Helius)
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use wallet_core::nft::{NftItem, sanitize_image_url};
use crate::state::load_from_storage;

/// Alchemy base URL per EVM chain
fn alchemy_base_url(chain_id: &str) -> Option<&'static str> {
    match chain_id {
        "ethereum" => Some("https://eth-mainnet.g.alchemy.com"),
        "polygon" => Some("https://polygon-mainnet.g.alchemy.com"),
        "base" => Some("https://base-mainnet.g.alchemy.com"),
        "arbitrum" => Some("https://arb-mainnet.g.alchemy.com"),
        "optimism" => Some("https://opt-mainnet.g.alchemy.com"),
        _ => None,
    }
}

/// Fetch NFTs for any supported chain
pub async fn fetch_nfts(owner: &str, chain_id: &str) -> Vec<NftItem> {
    let evm_chains = ["ethereum", "polygon", "base", "arbitrum", "optimism"];
    if evm_chains.contains(&chain_id) {
        let api_key = load_from_storage("alchemy_api_key").unwrap_or_default();
        if api_key.is_empty() {
            return Vec::new();
        }
        fetch_nfts_evm(owner, chain_id, &api_key).await
    } else if chain_id == "solana" {
        let api_key = load_from_storage("helius_api_key").unwrap_or_default();
        if api_key.is_empty() {
            return Vec::new();
        }
        fetch_nfts_solana(owner, &api_key).await
    } else {
        Vec::new()
    }
}

/// Fetch NFTs via Alchemy NFT API v3
async fn fetch_nfts_evm(owner: &str, chain_id: &str, api_key: &str) -> Vec<NftItem> {
    let base = match alchemy_base_url(chain_id) {
        Some(b) => b,
        None => return Vec::new(),
    };

    let url = format!(
        "{}/nft/v3/{}/getNFTsForOwner?owner={}&withMetadata=true&pageSize=50",
        base, api_key, owner
    );

    let json = match super::get_json(&url).await {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    let nfts = match json.get("ownedNfts").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    nfts.iter().filter_map(|nft| {
        let contract = nft.get("contract")
            .and_then(|c| c.get("address"))
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        let token_id = nft.get("tokenId")
            .and_then(|v| v.as_str())
            .unwrap_or("0");

        let name = nft.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unnamed NFT");

        let description = nft.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let image_url = nft.get("image")
            .and_then(|img| img.get("cachedUrl").or_else(|| img.get("originalUrl")))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let collection_name = nft.get("contract")
            .and_then(|c| c.get("openSeaMetadata"))
            .and_then(|m| m.get("collectionName"))
            .and_then(|v| v.as_str())
            .or_else(|| nft.get("contract").and_then(|c| c.get("name")).and_then(|v| v.as_str()))
            .unwrap_or("Unknown Collection");

        let token_type = nft.get("tokenType")
            .and_then(|v| v.as_str())
            .unwrap_or("ERC-721");

        if contract.is_empty() {
            return None;
        }

        Some(NftItem {
            contract_address: contract.to_string(),
            token_id: token_id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            image_url: sanitize_image_url(image_url),
            collection_name: collection_name.to_string(),
            chain_id: chain_id.to_string(),
            token_standard: token_type.to_string(),
        })
    }).collect()
}

/// Fetch NFTs via Helius DAS API (Solana)
async fn fetch_nfts_solana(owner: &str, api_key: &str) -> Vec<NftItem> {
    let url = format!("https://mainnet.helius-rpc.com/?api-key={}", api_key);

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAssetsByOwner",
        "params": {
            "ownerAddress": owner,
            "page": 1,
            "limit": 50,
            "displayOptions": {
                "showCollectionMetadata": true
            }
        }
    });

    let json = match super::post_json(&url, &body.to_string()).await {
        Ok(j) => j,
        Err(_) => return Vec::new(),
    };

    let items = match json.get("result")
        .and_then(|r| r.get("items"))
        .and_then(|v| v.as_array())
    {
        Some(arr) => arr,
        None => return Vec::new(),
    };

    items.iter().filter_map(|item| {
        // Only include NFTs and compressed NFTs
        let interface = item.get("interface")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if interface != "V1_NFT" && interface != "ProgrammableNFT" {
            return None;
        }

        let id = item.get("id").and_then(|v| v.as_str()).unwrap_or_default();

        let content = item.get("content")?;
        let metadata = content.get("metadata")?;

        let name = metadata.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unnamed NFT");

        let description = metadata.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let image_url = content.get("links")
            .and_then(|l| l.get("image"))
            .and_then(|v| v.as_str())
            .or_else(|| content.get("json_uri").and_then(|v| v.as_str()))
            .unwrap_or("");

        let collection_name = item.get("grouping")
            .and_then(|g| g.as_array())
            .and_then(|arr| arr.iter().find(|g| {
                g.get("group_key").and_then(|k| k.as_str()) == Some("collection")
            }))
            .and_then(|g| g.get("collection_metadata"))
            .and_then(|m| m.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Collection");

        let contract_address = item.get("grouping")
            .and_then(|g| g.as_array())
            .and_then(|arr| arr.iter().find(|g| {
                g.get("group_key").and_then(|k| k.as_str()) == Some("collection")
            }))
            .and_then(|g| g.get("group_value"))
            .and_then(|v| v.as_str())
            .unwrap_or(id);

        Some(NftItem {
            contract_address: contract_address.to_string(),
            token_id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            image_url: sanitize_image_url(image_url),
            collection_name: collection_name.to_string(),
            chain_id: "solana".to_string(),
            token_standard: "Metaplex".to_string(),
        })
    }).collect()
}
