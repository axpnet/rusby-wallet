// Rusby Wallet — Pure Rust multi-chain crypto wallet
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// chains: Address derivation for all supported blockchains (16 chains)
//
// Submodules: evm, solana, ton, cosmos, bitcoin, litecoin, stellar, ripple, dogecoin, tron
// Trait: Chain — derive_address(), name(), ticker(), chain_id()
// Functions: supported_chains() — Config for all chains with RPC URLs

pub mod evm;
pub mod solana;
pub mod ton;
pub mod cosmos;
pub mod bitcoin;
pub mod litecoin;
pub mod stellar;
pub mod ripple;
pub mod dogecoin;
pub mod tron;

use serde::{Deserialize, Serialize};

/// Unified chain trait
pub trait Chain {
    /// Derive address from BIP39 seed
    fn derive_address(&self, seed: &[u8; 64]) -> Result<String, String>;

    /// Get chain display name
    fn name(&self) -> &str;

    /// Get chain ticker symbol
    fn ticker(&self) -> &str;

    /// Get chain identifier
    fn chain_id(&self) -> ChainId;
}

/// Chain identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainId {
    Ethereum,
    Polygon,
    Bsc,
    Optimism,
    Base,
    Arbitrum,
    Solana,
    Ton,
    Bitcoin,
    CosmosHub,
    Osmosis,
    Litecoin,
    Stellar,
    Ripple,
    Dogecoin,
    Tron,
}

/// Supported chain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub id: ChainId,
    pub name: String,
    pub ticker: String,
    pub evm_chain_id: Option<u64>,
    pub coin_type: u32,
    pub rpc_urls: Vec<String>,
}

/// Get all supported chain configs (mainnet)
pub fn supported_chains() -> Vec<ChainConfig> {
    get_chains(false)
}

/// Get all supported chain configs for testnet
pub fn supported_chains_testnet() -> Vec<ChainConfig> {
    get_chains(true)
}

/// Get chains for mainnet or testnet
pub fn get_chains(testnet: bool) -> Vec<ChainConfig> {
    if testnet {
        vec![
            ChainConfig {
                id: ChainId::Ethereum,
                name: "Sepolia".into(),
                ticker: "ETH".into(),
                evm_chain_id: Some(11155111),
                coin_type: 60,
                rpc_urls: vec![
                    "https://rpc.sepolia.org".into(),
                    "https://rpc.ankr.com/eth_sepolia".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Polygon,
                name: "Polygon Amoy".into(),
                ticker: "POL".into(),
                evm_chain_id: Some(80002),
                coin_type: 60,
                rpc_urls: vec![
                    "https://rpc-amoy.polygon.technology".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Bsc,
                name: "BSC Testnet".into(),
                ticker: "tBNB".into(),
                evm_chain_id: Some(97),
                coin_type: 60,
                rpc_urls: vec![
                    "https://data-seed-prebsc-1-s1.binance.org:8545".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Optimism,
                name: "OP Sepolia".into(),
                ticker: "ETH".into(),
                evm_chain_id: Some(11155420),
                coin_type: 60,
                rpc_urls: vec![
                    "https://sepolia.optimism.io".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Base,
                name: "Base Sepolia".into(),
                ticker: "ETH".into(),
                evm_chain_id: Some(84532),
                coin_type: 60,
                rpc_urls: vec![
                    "https://sepolia.base.org".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Arbitrum,
                name: "Arbitrum Sepolia".into(),
                ticker: "ETH".into(),
                evm_chain_id: Some(421614),
                coin_type: 60,
                rpc_urls: vec![
                    "https://sepolia-rollup.arbitrum.io/rpc".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Solana,
                name: "Solana Devnet".into(),
                ticker: "SOL".into(),
                evm_chain_id: None,
                coin_type: 501,
                rpc_urls: vec![
                    "https://api.devnet.solana.com".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Ton,
                name: "TON Testnet".into(),
                ticker: "TON".into(),
                evm_chain_id: None,
                coin_type: 607,
                rpc_urls: vec![
                    "https://testnet.toncenter.com/api/v2/jsonRPC".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Bitcoin,
                name: "Bitcoin Signet".into(),
                ticker: "tBTC".into(),
                evm_chain_id: None,
                coin_type: 0,
                rpc_urls: vec![
                    "https://mempool.space/signet/api".into(),
                ],
            },
            ChainConfig {
                id: ChainId::CosmosHub,
                name: "Cosmos Testnet".into(),
                ticker: "ATOM".into(),
                evm_chain_id: None,
                coin_type: 118,
                rpc_urls: vec![
                    "https://rpc.sentry-01.theta-testnet.polypore.xyz".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Osmosis,
                name: "Osmosis Testnet".into(),
                ticker: "OSMO".into(),
                evm_chain_id: None,
                coin_type: 118,
                rpc_urls: vec![
                    "https://rpc.testnet.osmosis.zone".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Litecoin,
                name: "Litecoin Testnet".into(),
                ticker: "tLTC".into(),
                evm_chain_id: None,
                coin_type: 2,
                rpc_urls: vec![
                    "https://litecoinspace.org/testnet/api".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Stellar,
                name: "Stellar Testnet".into(),
                ticker: "XLM".into(),
                evm_chain_id: None,
                coin_type: 148,
                rpc_urls: vec![
                    "https://horizon-testnet.stellar.org".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Ripple,
                name: "XRP Testnet".into(),
                ticker: "XRP".into(),
                evm_chain_id: None,
                coin_type: 144,
                rpc_urls: vec![
                    "https://s.altnet.rippletest.net:51234".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Dogecoin,
                name: "Dogecoin Testnet".into(),
                ticker: "tDOGE".into(),
                evm_chain_id: None,
                coin_type: 3,
                rpc_urls: vec![
                    "https://doge1.trezor.io/api/v2".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Tron,
                name: "TRON Nile".into(),
                ticker: "TRX".into(),
                evm_chain_id: None,
                coin_type: 195,
                rpc_urls: vec![
                    "https://nile.trongrid.io".into(),
                ],
            },
        ]
    } else {
        vec![
            ChainConfig {
                id: ChainId::Ethereum,
                name: "Ethereum".into(),
                ticker: "ETH".into(),
                evm_chain_id: Some(1),
                coin_type: 60,
                rpc_urls: vec![
                    "https://eth.llamarpc.com".into(),
                    "https://rpc.ankr.com/eth".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Polygon,
                name: "Polygon".into(),
                ticker: "POL".into(),
                evm_chain_id: Some(137),
                coin_type: 60,
                rpc_urls: vec![
                    "https://polygon-rpc.com".into(),
                    "https://rpc.ankr.com/polygon".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Bsc,
                name: "BNB Smart Chain".into(),
                ticker: "BNB".into(),
                evm_chain_id: Some(56),
                coin_type: 60,
                rpc_urls: vec![
                    "https://bsc-dataseed.binance.org".into(),
                    "https://rpc.ankr.com/bsc".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Optimism,
                name: "Optimism".into(),
                ticker: "ETH".into(),
                evm_chain_id: Some(10),
                coin_type: 60,
                rpc_urls: vec![
                    "https://mainnet.optimism.io".into(),
                    "https://rpc.ankr.com/optimism".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Base,
                name: "Base".into(),
                ticker: "ETH".into(),
                evm_chain_id: Some(8453),
                coin_type: 60,
                rpc_urls: vec![
                    "https://mainnet.base.org".into(),
                    "https://rpc.ankr.com/base".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Arbitrum,
                name: "Arbitrum".into(),
                ticker: "ETH".into(),
                evm_chain_id: Some(42161),
                coin_type: 60,
                rpc_urls: vec![
                    "https://arb1.arbitrum.io/rpc".into(),
                    "https://rpc.ankr.com/arbitrum".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Solana,
                name: "Solana".into(),
                ticker: "SOL".into(),
                evm_chain_id: None,
                coin_type: 501,
                rpc_urls: vec![
                    "https://api.mainnet-beta.solana.com".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Ton,
                name: "TON".into(),
                ticker: "TON".into(),
                evm_chain_id: None,
                coin_type: 607,
                rpc_urls: vec![
                    "https://toncenter.com/api/v2/jsonRPC".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Bitcoin,
                name: "Bitcoin".into(),
                ticker: "BTC".into(),
                evm_chain_id: None,
                coin_type: 0,
                rpc_urls: vec![
                    "https://mempool.space/api".into(),
                ],
            },
            ChainConfig {
                id: ChainId::CosmosHub,
                name: "Cosmos Hub".into(),
                ticker: "ATOM".into(),
                evm_chain_id: None,
                coin_type: 118,
                rpc_urls: vec![
                    "https://cosmos-rpc.polkachu.com".into(),
                    "https://rpc.cosmos.network".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Osmosis,
                name: "Osmosis".into(),
                ticker: "OSMO".into(),
                evm_chain_id: None,
                coin_type: 118,
                rpc_urls: vec![
                    "https://osmosis-rpc.polkachu.com".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Litecoin,
                name: "Litecoin".into(),
                ticker: "LTC".into(),
                evm_chain_id: None,
                coin_type: 2,
                rpc_urls: vec![
                    "https://litecoinspace.org/api".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Stellar,
                name: "Stellar".into(),
                ticker: "XLM".into(),
                evm_chain_id: None,
                coin_type: 148,
                rpc_urls: vec![
                    "https://horizon.stellar.org".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Ripple,
                name: "XRP Ledger".into(),
                ticker: "XRP".into(),
                evm_chain_id: None,
                coin_type: 144,
                rpc_urls: vec![
                    "https://xrplcluster.com".into(),
                    "https://s1.ripple.com:51234".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Dogecoin,
                name: "Dogecoin".into(),
                ticker: "DOGE".into(),
                evm_chain_id: None,
                coin_type: 3,
                rpc_urls: vec![
                    "https://doge1.trezor.io/api/v2".into(),
                ],
            },
            ChainConfig {
                id: ChainId::Tron,
                name: "TRON".into(),
                ticker: "TRX".into(),
                evm_chain_id: None,
                coin_type: 195,
                rpc_urls: vec![
                    "https://api.trongrid.io".into(),
                ],
            },
        ]
    }
}
