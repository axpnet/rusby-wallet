// Rusby Wallet — EVM send logic (native + ERC-20)
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use wallet_core::tx::evm::*;
use wallet_core::chains::evm;
use super::chain_id_to_string;
use zeroize::Zeroize;

pub async fn send_native(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
    rpc_url: &str,
    config: &wallet_core::chains::ChainConfig,
) -> Result<String, String> {
    let mut private_key = evm::get_private_key(seed)?;
    let from_address = evm::derive_evm_address(seed)?;

    let to_bytes = parse_address(to)?;
    let value = parse_ether_to_wei(amount)?;

    let nonce = crate::rpc::evm::get_nonce(&from_address, rpc_url).await?;
    let gas_price = crate::rpc::evm::get_gas_price(rpc_url).await?;
    let priority_fee = crate::rpc::evm::get_max_priority_fee(rpc_url).await
        .unwrap_or(1_500_000_000);

    let evm_chain_id = config.evm_chain_id.ok_or("Missing EVM chain ID")?;

    let tx = EvmTransaction {
        chain_id_num: evm_chain_id,
        nonce,
        max_priority_fee_per_gas: priority_fee,
        max_fee_per_gas: gas_price.saturating_mul(2),
        gas_limit: 21000,
        to: to_bytes,
        value,
        data: vec![],
    };

    let signed = tx.sign(&private_key, config.id.clone())?;
    private_key.zeroize();
    let raw_hex = format!("0x{}", hex::encode(&signed.raw_bytes));

    crate::rpc::evm::send_raw_transaction(&raw_hex, rpc_url).await
}

pub async fn send_erc20(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
    token_address: &str,
    rpc_url: &str,
    config: &wallet_core::chains::ChainConfig,
) -> Result<String, String> {
    use wallet_core::tokens::erc20;

    let mut private_key = evm::get_private_key(seed)?;
    let from_address = evm::derive_evm_address(seed)?;

    let tokens = erc20::tokens_for_chain(&chain_id_to_string(&config.id));
    let token = tokens.iter()
        .find(|t| t.address.to_lowercase() == token_address.to_lowercase())
        .ok_or("Token not found")?;

    let data = erc20::encode_transfer(to, amount, token.decimals)?;
    let contract_address = parse_address(token_address)?;

    let nonce = crate::rpc::evm::get_nonce(&from_address, rpc_url).await?;
    let gas_price = crate::rpc::evm::get_gas_price(rpc_url).await?;
    let priority_fee = crate::rpc::evm::get_max_priority_fee(rpc_url).await
        .unwrap_or(1_500_000_000);

    let evm_chain_id = config.evm_chain_id.ok_or("Missing EVM chain ID")?;

    let tx = EvmTransaction {
        chain_id_num: evm_chain_id,
        nonce,
        max_priority_fee_per_gas: priority_fee,
        max_fee_per_gas: gas_price.saturating_mul(2),
        gas_limit: 65000,
        to: contract_address,
        value: 0,
        data,
    };

    let signed = tx.sign(&private_key, config.id.clone())?;
    private_key.zeroize();
    let raw_hex = format!("0x{}", hex::encode(&signed.raw_bytes));

    crate::rpc::evm::send_raw_transaction(&raw_hex, rpc_url).await
}

/// Send a swap transaction with arbitrary calldata (from 0x API quote)
pub async fn send_swap_tx(
    seed: &[u8; 64],
    to: &str,
    value: &str,
    data: &str,
    gas_limit: u64,
    rpc_url: &str,
    config: &wallet_core::chains::ChainConfig,
) -> Result<String, String> {
    let mut private_key = evm::get_private_key(seed)?;
    let from_address = evm::derive_evm_address(seed)?;

    let to_bytes = parse_address(to)?;

    // Parse value (decimal string or hex)
    let value_u128 = if value.starts_with("0x") {
        u128::from_str_radix(value.trim_start_matches("0x"), 16)
            .map_err(|_| "Invalid hex value")?
    } else {
        value.parse::<u128>().unwrap_or(0)
    };

    // Parse data hex to bytes
    let data_bytes = if data.starts_with("0x") {
        hex::decode(data.trim_start_matches("0x"))
            .map_err(|_| "Invalid calldata hex")?
    } else if !data.is_empty() {
        hex::decode(data).map_err(|_| "Invalid calldata hex")?
    } else {
        vec![]
    };

    let nonce = crate::rpc::evm::get_nonce(&from_address, rpc_url).await?;
    let gas_price = crate::rpc::evm::get_gas_price(rpc_url).await?;
    let priority_fee = crate::rpc::evm::get_max_priority_fee(rpc_url).await
        .unwrap_or(1_500_000_000);

    let evm_chain_id = config.evm_chain_id.ok_or("Missing EVM chain ID")?;

    let tx = EvmTransaction {
        chain_id_num: evm_chain_id,
        nonce,
        max_priority_fee_per_gas: priority_fee,
        max_fee_per_gas: gas_price.saturating_mul(2),
        gas_limit,
        to: to_bytes,
        value: value_u128,
        data: data_bytes,
    };

    let signed = tx.sign(&private_key, config.id.clone())?;
    private_key.zeroize();
    let raw_hex = format!("0x{}", hex::encode(&signed.raw_bytes));

    crate::rpc::evm::send_raw_transaction(&raw_hex, rpc_url).await
}
