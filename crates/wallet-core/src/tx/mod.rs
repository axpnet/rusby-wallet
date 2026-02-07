// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// tx: Transaction construction and signing for all supported chains

pub mod evm;
pub mod solana;
pub mod ton;
pub mod cosmos;
pub mod bitcoin;

use crate::chains::ChainId;
use serde::{Deserialize, Serialize};

/// A signed transaction ready for broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub chain_id: ChainId,
    pub raw_bytes: Vec<u8>,
    pub tx_hash: String,
}
