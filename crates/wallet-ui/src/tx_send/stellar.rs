// Rusby Wallet — Stellar send logic
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use wallet_core::tx::stellar::*;
use wallet_core::chains::stellar as xlm_chain;
use zeroize::Zeroize;

pub async fn send(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
    rpc_url: &str,
    testnet: bool,
) -> Result<String, String> {
    let mut keypair = xlm_chain::get_keypair(seed)?;
    let pubkey = xlm_chain::get_public_key(seed)?;

    // Decode destination address (StrKey → raw 32-byte pubkey)
    let (version, dest_pubkey_vec) = xlm_chain::strkey_decode(to)?;
    if version != (6 << 3) || dest_pubkey_vec.len() != 32 {
        return Err("Indirizzo Stellar destinatario non valido".into());
    }
    let mut dest_pubkey = [0u8; 32];
    dest_pubkey.copy_from_slice(&dest_pubkey_vec);

    // Get account sequence number (increment by 1 for new TX)
    let sequence = crate::rpc::stellar::get_account_sequence(
        &xlm_chain::derive_stellar_address(seed)?,
        rpc_url,
    ).await?;

    let amount_stroops = parse_xlm_to_stroops(amount)?;

    let passphrase = if testnet {
        TESTNET_PASSPHRASE
    } else {
        MAINNET_PASSPHRASE
    };

    let tx = StellarTransaction {
        source_pubkey: pubkey,
        destination_pubkey: dest_pubkey,
        amount_stroops,
        sequence: sequence + 1, // Next sequence number
        fee: 100, // Base fee: 100 stroops = 0.00001 XLM
        network_passphrase: passphrase.into(),
    };

    let mut private_key = [0u8; 32];
    private_key.copy_from_slice(&keypair[..32]);
    let signed = tx.sign(&private_key)?;
    private_key.zeroize();
    keypair.zeroize();

    // Base64 encode the envelope for submission
    let tx_xdr_base64 = crate::tx_send::base64_simple_encode(&signed.raw_bytes);

    crate::rpc::stellar::submit_tx(&tx_xdr_base64, rpc_url).await
}
