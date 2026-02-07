// Rusby Wallet — CAIP-2 chain identifier mapping
// Maps between CAIP-2 namespace:reference format and internal ChainId

use crate::chains::ChainId;

/// Convert a CAIP-2 identifier (e.g. "eip155:1") to internal ChainId
pub fn caip2_to_chain_id(caip2: &str) -> Option<ChainId> {
    match caip2 {
        "eip155:1" => Some(ChainId::Ethereum),
        "eip155:137" => Some(ChainId::Polygon),
        "eip155:56" => Some(ChainId::Bsc),
        "eip155:10" => Some(ChainId::Optimism),
        "eip155:8453" => Some(ChainId::Base),
        "eip155:42161" => Some(ChainId::Arbitrum),
        "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp" => Some(ChainId::Solana),
        "cosmos:cosmoshub-4" => Some(ChainId::CosmosHub),
        "cosmos:osmosis-1" => Some(ChainId::Osmosis),
        "bip122:000000000019d6689c085ae165831e93" => Some(ChainId::Bitcoin),
        "ton:mainnet" => Some(ChainId::Ton),
        "bip122:12a765e31ffd4059bada1e25190f6e98" => Some(ChainId::Litecoin),
        "stellar:pubnet" => Some(ChainId::Stellar),
        "xrpl:0" => Some(ChainId::Ripple),
        "bip122:1a91e3dace36e2be3bf030a65679fe821aa1d6ef92e7c9902eb318182c355691" => Some(ChainId::Dogecoin),
        "tron:mainnet" => Some(ChainId::Tron),
        _ => None,
    }
}

/// Convert internal ChainId to CAIP-2 identifier
pub fn chain_id_to_caip2(chain_id: &ChainId) -> &'static str {
    match chain_id {
        ChainId::Ethereum => "eip155:1",
        ChainId::Polygon => "eip155:137",
        ChainId::Bsc => "eip155:56",
        ChainId::Optimism => "eip155:10",
        ChainId::Base => "eip155:8453",
        ChainId::Arbitrum => "eip155:42161",
        ChainId::Solana => "solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp",
        ChainId::CosmosHub => "cosmos:cosmoshub-4",
        ChainId::Osmosis => "cosmos:osmosis-1",
        ChainId::Bitcoin => "bip122:000000000019d6689c085ae165831e93",
        ChainId::Ton => "ton:mainnet",
        ChainId::Litecoin => "bip122:12a765e31ffd4059bada1e25190f6e98",
        ChainId::Stellar => "stellar:pubnet",
        ChainId::Ripple => "xrpl:0",
        ChainId::Dogecoin => "bip122:1a91e3dace36e2be3bf030a65679fe821aa1d6ef92e7c9902eb318182c355691",
        ChainId::Tron => "tron:mainnet",
    }
}

/// Get CAIP-2 namespace for a ChainId
pub fn chain_id_to_namespace(chain_id: &ChainId) -> &'static str {
    match chain_id {
        ChainId::Ethereum | ChainId::Polygon | ChainId::Bsc |
        ChainId::Optimism | ChainId::Base | ChainId::Arbitrum => "eip155",
        ChainId::Solana => "solana",
        ChainId::CosmosHub | ChainId::Osmosis => "cosmos",
        ChainId::Bitcoin | ChainId::Litecoin | ChainId::Dogecoin => "bip122",
        ChainId::Ton => "ton",
        ChainId::Stellar => "stellar",
        ChainId::Ripple => "xrpl",
        ChainId::Tron => "tron",
    }
}

/// Build WalletConnect supported namespaces from addresses
/// Returns a map of namespace -> { chains, methods, events, accounts }
pub fn supported_namespaces(
    addresses: &[(ChainId, String)],
) -> std::collections::HashMap<String, NamespaceConfig> {
    use std::collections::{HashMap, HashSet};

    let mut namespaces: HashMap<String, NamespaceConfig> = HashMap::new();

    for (chain_id, address) in addresses {
        let caip2 = chain_id_to_caip2(chain_id);
        let ns = chain_id_to_namespace(chain_id);
        let account = format!("{}:{}", caip2, address);

        let entry = namespaces.entry(ns.to_string()).or_insert_with(|| {
            NamespaceConfig {
                chains: HashSet::new(),
                methods: namespace_methods(ns),
                events: namespace_events(ns),
                accounts: Vec::new(),
            }
        });
        entry.chains.insert(caip2.to_string());
        entry.accounts.push(account);
    }

    namespaces
}

/// WalletConnect namespace configuration
#[derive(Debug, Clone)]
pub struct NamespaceConfig {
    pub chains: std::collections::HashSet<String>,
    pub methods: Vec<String>,
    pub events: Vec<String>,
    pub accounts: Vec<String>,
}

fn namespace_methods(ns: &str) -> Vec<String> {
    match ns {
        "eip155" => vec![
            "eth_sendTransaction".into(),
            "personal_sign".into(),
            "eth_signTypedData_v4".into(),
        ],
        "solana" => vec![
            "solana_signTransaction".into(),
            "solana_signMessage".into(),
        ],
        "cosmos" => vec![
            "cosmos_signAmino".into(),
            "cosmos_signDirect".into(),
        ],
        _ => vec![],
    }
}

fn namespace_events(ns: &str) -> Vec<String> {
    match ns {
        "eip155" => vec![
            "chainChanged".into(),
            "accountsChanged".into(),
        ],
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caip2_roundtrip_evm() {
        assert_eq!(caip2_to_chain_id("eip155:1"), Some(ChainId::Ethereum));
        assert_eq!(chain_id_to_caip2(&ChainId::Ethereum), "eip155:1");
    }

    #[test]
    fn test_caip2_roundtrip_solana() {
        let caip = chain_id_to_caip2(&ChainId::Solana);
        assert_eq!(caip2_to_chain_id(caip), Some(ChainId::Solana));
    }

    #[test]
    fn test_caip2_roundtrip_all() {
        let all = vec![
            ChainId::Ethereum, ChainId::Polygon, ChainId::Bsc,
            ChainId::Optimism, ChainId::Base, ChainId::Arbitrum,
            ChainId::Solana, ChainId::CosmosHub, ChainId::Osmosis,
            ChainId::Bitcoin, ChainId::Ton,
            ChainId::Litecoin, ChainId::Stellar, ChainId::Ripple,
            ChainId::Dogecoin, ChainId::Tron,
        ];
        for id in all {
            let caip = chain_id_to_caip2(&id);
            assert_eq!(caip2_to_chain_id(caip), Some(id));
        }
    }

    #[test]
    fn test_caip2_unknown() {
        assert_eq!(caip2_to_chain_id("eip155:999999"), None);
        assert_eq!(caip2_to_chain_id(""), None);
    }

    #[test]
    fn test_namespace() {
        assert_eq!(chain_id_to_namespace(&ChainId::Ethereum), "eip155");
        assert_eq!(chain_id_to_namespace(&ChainId::Polygon), "eip155");
        assert_eq!(chain_id_to_namespace(&ChainId::Solana), "solana");
        assert_eq!(chain_id_to_namespace(&ChainId::CosmosHub), "cosmos");
        assert_eq!(chain_id_to_namespace(&ChainId::Bitcoin), "bip122");
    }

    #[test]
    fn test_supported_namespaces() {
        let addresses = vec![
            (ChainId::Ethereum, "0xabc123".into()),
            (ChainId::Polygon, "0xabc123".into()),
            (ChainId::Solana, "So1abc".into()),
        ];
        let ns = supported_namespaces(&addresses);
        assert!(ns.contains_key("eip155"));
        assert!(ns.contains_key("solana"));
        assert_eq!(ns["eip155"].chains.len(), 2);
        assert_eq!(ns["eip155"].accounts.len(), 2);
        assert_eq!(ns["solana"].accounts.len(), 1);
    }

    #[test]
    fn test_eip155_methods() {
        let methods = namespace_methods("eip155");
        assert!(methods.contains(&"personal_sign".to_string()));
        assert!(methods.contains(&"eth_sendTransaction".to_string()));
    }
}
