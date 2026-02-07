// Rusby Wallet — Cosmos send logic
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use wallet_core::tx::cosmos::*;
use wallet_core::bip32_utils::{self, DerivationPath};
use wallet_core::chains::ChainId;
use zeroize::Zeroize;

pub async fn send(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
    rpc_url: &str,
    denom: &str,
    chain_id_str: &str,
    chain_id: ChainId,
) -> Result<String, String> {
    let path = DerivationPath::bip44(118);
    let (mut private_key, _) = bip32_utils::derive_key_from_seed(seed, &path)?;

    let prefix = if denom == "uatom" { "cosmos" } else { "osmo" };
    let from_address = wallet_core::chains::cosmos::derive_cosmos_address(seed, prefix, 118)?;

    let uamount = parse_atom_to_uatom(amount)?;

    let (account_number, sequence) = crate::rpc::cosmos::get_account_info(&from_address, rpc_url).await
        .unwrap_or((0, 0));

    let msg = CosmosMsgSend {
        from_address,
        to_address: to.to_string(),
        amount: uamount,
        denom: denom.to_string(),
        chain_id_str: chain_id_str.to_string(),
        account_number,
        sequence,
        gas_limit: 200000,
        fee_amount: 5000,
        fee_denom: denom.to_string(),
    };

    let signed = msg.sign(&private_key, chain_id)?;
    private_key.zeroize();
    let tx_json = String::from_utf8(signed.raw_bytes)
        .map_err(|_| "Invalid TX bytes")?;

    crate::rpc::cosmos::broadcast_tx(&tx_json, rpc_url).await
}

/// Send CW-20 token via MsgExecuteContract
pub async fn send_cw20(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
    token_address: &str,
    rpc_url: &str,
    denom: &str,
    chain_id_str: &str,
    chain_id: ChainId,
) -> Result<String, String> {
    use wallet_core::tx::cosmos::CosmosMsgExecuteContract;
    use wallet_core::tokens::cw20;

    let path = DerivationPath::bip44(118);
    let (mut private_key, _) = bip32_utils::derive_key_from_seed(seed, &path)?;

    let prefix = if denom == "uatom" { "cosmos" } else { "osmo" };
    let from_address = wallet_core::chains::cosmos::derive_cosmos_address(seed, prefix, 118)?;

    // Find token decimals from default list
    let chain_str = super::chain_id_to_string(&chain_id);
    let tokens = cw20::tokens_for_chain(&chain_str);
    let decimals = tokens.iter()
        .find(|t| t.address == token_address)
        .map(|t| t.decimals)
        .unwrap_or(6);

    let msg_json = cw20::encode_transfer_msg(to, amount, decimals)?;

    let (account_number, sequence) = crate::rpc::cosmos::get_account_info(&from_address, rpc_url).await
        .unwrap_or((0, 0));

    let msg = CosmosMsgExecuteContract {
        sender: from_address,
        contract: token_address.to_string(),
        msg_json,
        chain_id_str: chain_id_str.to_string(),
        account_number,
        sequence,
        gas_limit: 300000,
        fee_amount: 7500,
        fee_denom: denom.to_string(),
    };

    let signed = msg.sign(&private_key, chain_id)?;
    private_key.zeroize();
    let tx_json = String::from_utf8(signed.raw_bytes)
        .map_err(|_| "Invalid TX bytes")?;

    crate::rpc::cosmos::broadcast_tx(&tx_json, rpc_url).await
}
