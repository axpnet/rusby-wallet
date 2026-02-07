// Rusby Wallet — NFT types and helpers
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

/// Single NFT item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftItem {
    pub contract_address: String,
    pub token_id: String,
    pub name: String,
    pub description: String,
    pub image_url: String,
    pub collection_name: String,
    pub chain_id: String,
    pub token_standard: String,
}

/// NFT collection grouping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftCollection {
    pub name: String,
    pub contract_address: String,
    pub chain_id: String,
    pub items: Vec<NftItem>,
}

/// Convert IPFS and other protocol URLs to HTTPS gateway URLs
pub fn sanitize_image_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // ipfs:// protocol
    if let Some(cid) = trimmed.strip_prefix("ipfs://") {
        return format!("https://ipfs.io/ipfs/{}", cid);
    }

    // ar:// protocol (Arweave)
    if let Some(id) = trimmed.strip_prefix("ar://") {
        return format!("https://arweave.net/{}", id);
    }

    // Already HTTPS or data URI — keep as-is
    if trimmed.starts_with("https://") || trimmed.starts_with("data:") {
        return trimmed.to_string();
    }

    // HTTP → upgrade to HTTPS
    if let Some(rest) = trimmed.strip_prefix("http://") {
        return format!("https://{}", rest);
    }

    // Bare CID (starts with Qm or bafy)
    if trimmed.starts_with("Qm") || trimmed.starts_with("bafy") {
        return format!("https://ipfs.io/ipfs/{}", trimmed);
    }

    // Unknown/unsupported scheme — reject to prevent XSS vectors (javascript:, blob:, etc.)
    String::new()
}

/// Group NFT items by collection
pub fn group_by_collection(items: &[NftItem]) -> Vec<NftCollection> {
    let mut collections: std::collections::HashMap<String, NftCollection> =
        std::collections::HashMap::new();

    for item in items {
        let key = format!("{}:{}", item.chain_id, item.contract_address);
        collections
            .entry(key)
            .or_insert_with(|| NftCollection {
                name: item.collection_name.clone(),
                contract_address: item.contract_address.clone(),
                chain_id: item.chain_id.clone(),
                items: Vec::new(),
            })
            .items
            .push(item.clone());
    }

    let mut result: Vec<NftCollection> = collections.into_values().collect();
    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_ipfs_url() {
        assert_eq!(
            sanitize_image_url("ipfs://QmTest123/image.png"),
            "https://ipfs.io/ipfs/QmTest123/image.png"
        );
    }

    #[test]
    fn test_sanitize_arweave_url() {
        assert_eq!(
            sanitize_image_url("ar://abc123"),
            "https://arweave.net/abc123"
        );
    }

    #[test]
    fn test_sanitize_https_passthrough() {
        let url = "https://example.com/image.png";
        assert_eq!(sanitize_image_url(url), url);
    }

    #[test]
    fn test_sanitize_http_upgrade() {
        assert_eq!(
            sanitize_image_url("http://example.com/img.png"),
            "https://example.com/img.png"
        );
    }

    #[test]
    fn test_sanitize_bare_cid() {
        assert_eq!(
            sanitize_image_url("QmTestCID123"),
            "https://ipfs.io/ipfs/QmTestCID123"
        );
        assert_eq!(
            sanitize_image_url("bafyTestCID"),
            "https://ipfs.io/ipfs/bafyTestCID"
        );
    }

    #[test]
    fn test_sanitize_empty() {
        assert_eq!(sanitize_image_url(""), "");
        assert_eq!(sanitize_image_url("  "), "");
    }

    #[test]
    fn test_sanitize_data_uri() {
        let uri = "data:image/png;base64,abc123";
        assert_eq!(sanitize_image_url(uri), uri);
    }

    #[test]
    fn test_group_by_collection() {
        let items = vec![
            NftItem {
                contract_address: "0xAAA".into(),
                token_id: "1".into(),
                name: "Ape #1".into(),
                description: "".into(),
                image_url: "".into(),
                collection_name: "BAYC".into(),
                chain_id: "ethereum".into(),
                token_standard: "ERC-721".into(),
            },
            NftItem {
                contract_address: "0xAAA".into(),
                token_id: "2".into(),
                name: "Ape #2".into(),
                description: "".into(),
                image_url: "".into(),
                collection_name: "BAYC".into(),
                chain_id: "ethereum".into(),
                token_standard: "ERC-721".into(),
            },
            NftItem {
                contract_address: "0xBBB".into(),
                token_id: "5".into(),
                name: "Punk #5".into(),
                description: "".into(),
                image_url: "".into(),
                collection_name: "CryptoPunks".into(),
                chain_id: "ethereum".into(),
                token_standard: "ERC-721".into(),
            },
        ];

        let groups = group_by_collection(&items);
        assert_eq!(groups.len(), 2);
        // BAYC has 2 items
        let bayc = groups.iter().find(|g| g.name == "BAYC").unwrap();
        assert_eq!(bayc.items.len(), 2);
        // CryptoPunks has 1 item
        let punks = groups.iter().find(|g| g.name == "CryptoPunks").unwrap();
        assert_eq!(punks.items.len(), 1);
    }
}
