// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;
use crate::components::confirmation_modal::ConfirmationModal;

#[component]
pub fn SendPage() -> impl IntoView {
    let wallet_state: ReadSignal<WalletState> = expect_context();
    let set_page: WriteSignal<AppPage> = expect_context();

    let (recipient, set_recipient) = signal(String::new());
    let (amount, set_amount) = signal(String::new());
    let (status, set_status) = signal(String::new());
    let (status_type, set_status_type) = signal("warning"); // "warning" | "success" | "danger"
    let (sending, set_sending) = signal(false);
    let (show_confirm, set_show_confirm) = signal(false);
    let (estimated_fee, set_estimated_fee) = signal("0.0000".to_string());

    let active_chain = move || wallet_state.get().active_chain.clone();

    let active_ticker = move || {
        let state = wallet_state.get();
        chain_list().into_iter()
            .find(|c| c.id == state.active_chain)
            .map(|c| c.ticker)
            .unwrap_or("???".into())
    };

    let active_chain_name = move || {
        let state = wallet_state.get();
        chain_list().into_iter()
            .find(|c| c.id == state.active_chain)
            .map(|c| c.name)
            .unwrap_or("Unknown".into())
    };

    let is_evm = move || {
        matches!(active_chain().as_str(), "ethereum" | "polygon" | "bsc" | "optimism" | "base" | "arbitrum")
    };

    // Estimate gas when inputs change
    let estimate = move |_| {
        let to = recipient.get();
        let amt = amount.get();

        if to.is_empty() {
            set_status.set("Enter recipient address".into());
            set_status_type.set("warning");
            return;
        }
        if amt.is_empty() || amt.parse::<f64>().is_err() {
            set_status.set("Enter a valid amount".into());
            set_status_type.set("warning");
            return;
        }

        set_status.set(String::new());

        if is_evm() {
            // For EVM: estimate gas as 21000 * gas_price
            let chain = active_chain();
            set_estimated_fee.set("Estimating...".into());
            wasm_bindgen_futures::spawn_local(async move {
                let chains = wallet_core::chains::supported_chains();
                if let Some(config) = chains.iter().find(|c| chain_id_to_string(&c.id) == chain) {
                    if let Some(rpc_url) = config.rpc_urls.first() {
                        match crate::rpc::evm::get_gas_price(rpc_url).await {
                            Ok(gas_price) => {
                                let fee_wei = gas_price * 21000;
                                let fee_gwei = fee_wei / 1_000_000_000;
                                set_estimated_fee.set(format!("~{} Gwei", fee_gwei));
                            }
                            Err(_) => {
                                set_estimated_fee.set("~21000 Gwei".into());
                            }
                        }
                    }
                }
            });
        } else if active_chain() == "solana" {
            set_estimated_fee.set("~0.000005 SOL".into());
        } else if active_chain() == "ton" {
            set_estimated_fee.set("~0.01 TON".into());
        } else {
            set_estimated_fee.set("~0.005".into());
        }

        set_show_confirm.set(true);
    };

    let on_confirm = Callback::new(move |password: String| {
        set_show_confirm.set(false);
        set_sending.set(true);
        set_status.set("Signing and broadcasting...".into());
        set_status_type.set("warning");

        let chain = active_chain();
        let to = recipient.get();
        let amt = amount.get();

        wasm_bindgen_futures::spawn_local(async move {
            let result = execute_send(&chain, &to, &amt, &password).await;
            set_sending.set(false);
            match result {
                Ok(tx_hash) => {
                    set_status.set(format!("TX sent! Hash: {}", tx_hash));
                    set_status_type.set("success");
                }
                Err(e) => {
                    set_status.set(format!("Error: {}", e));
                    set_status_type.set("danger");
                }
            }
        });
    });

    let on_cancel = Callback::new(move |_: ()| {
        set_show_confirm.set(false);
    });

    view! {
        <div class="p-4">
            <div class="flex items-center justify-between mb-4">
                <button class="btn btn-sm btn-secondary" on:click=move |_| set_page.set(AppPage::Dashboard)>
                    "< Back"
                </button>
                <h2>"Send " {active_ticker}</h2>
                <div style="width: 60px;" />
            </div>

            <div class="input-group">
                <label>"Recipient Address"</label>
                <input
                    type="text"
                    placeholder="0x... or address"
                    prop:value=move || recipient.get()
                    on:input=move |ev| set_recipient.set(event_target_value(&ev))
                />
            </div>

            <div class="input-group">
                <label>"Amount"</label>
                <input
                    type="text"
                    placeholder="0.0"
                    prop:value=move || amount.get()
                    on:input=move |ev| set_amount.set(event_target_value(&ev))
                />
            </div>

            <div class="card text-sm">
                <div class="flex justify-between">
                    <span class="text-muted">"From"</span>
                    <span style="font-family: monospace; font-size: 12px;">
                        {move || {
                            let addr = wallet_state.get().current_address();
                            if addr.len() > 16 {
                                format!("{}...{}", &addr[..8], &addr[addr.len()-6..])
                            } else {
                                addr
                            }
                        }}
                    </span>
                </div>
                <div class="flex justify-between mt-2">
                    <span class="text-muted">"Network"</span>
                    <span>{active_chain_name}</span>
                </div>
                <div class="flex justify-between mt-2">
                    <span class="text-muted">"Balance"</span>
                    <span>{move || wallet_state.get().current_balance()} " " {active_ticker}</span>
                </div>
            </div>

            {move || {
                let s = status.get();
                if s.is_empty() { None } else {
                    let color = match status_type.get() {
                        "success" => "var(--success, #4caf50)",
                        "danger" => "var(--danger, #f44336)",
                        _ => "var(--warning, #ff9800)",
                    };
                    Some(view! {
                        <p class="text-sm mt-2" style=format!("color: {}; word-break: break-all;", color)>{s}</p>
                    })
                }
            }}

            <button
                class="btn btn-primary btn-block mt-4"
                on:click=estimate
                disabled=move || sending.get()
            >
                {move || if sending.get() { "Sending..." } else { "Send Transaction" }}
            </button>

            {move || {
                if show_confirm.get() {
                    Some(view! {
                        <ConfirmationModal
                            recipient=recipient.get()
                            amount=amount.get()
                            fee=estimated_fee.get()
                            chain=active_chain_name()
                            ticker=active_ticker()
                            on_confirm=on_confirm
                            on_cancel=on_cancel
                        />
                    })
                } else {
                    None
                }
            }}
        </div>
    }
}

fn chain_id_to_string(id: &wallet_core::chains::ChainId) -> String {
    match id {
        wallet_core::chains::ChainId::Ethereum => "ethereum",
        wallet_core::chains::ChainId::Polygon => "polygon",
        wallet_core::chains::ChainId::Bsc => "bsc",
        wallet_core::chains::ChainId::Optimism => "optimism",
        wallet_core::chains::ChainId::Base => "base",
        wallet_core::chains::ChainId::Arbitrum => "arbitrum",
        wallet_core::chains::ChainId::Solana => "solana",
        wallet_core::chains::ChainId::Ton => "ton",
        wallet_core::chains::ChainId::Bitcoin => "bitcoin",
        wallet_core::chains::ChainId::CosmosHub => "cosmos",
        wallet_core::chains::ChainId::Osmosis => "osmosis",
    }.to_string()
}

async fn execute_send(chain: &str, to: &str, amount: &str, password: &str) -> Result<String, String> {
    // Load wallet store and decrypt seed
    let store_json = crate::state::load_from_storage("wallet_store")
        .ok_or("No wallet found")?;
    let store: wallet_core::wallet::WalletStore = serde_json::from_str(&store_json)
        .map_err(|e| format!("Invalid wallet data: {}", e))?;

    let entry = store.wallets.get(store.active_index)
        .ok_or("No active wallet")?;
    let seed_bytes = wallet_core::crypto::decrypt(&entry.encrypted_seed, password)?;
    if seed_bytes.len() != 64 {
        return Err("Invalid seed".into());
    }
    let mut seed = [0u8; 64];
    seed.copy_from_slice(&seed_bytes);

    let chains = wallet_core::chains::supported_chains();
    let config = chains.iter()
        .find(|c| chain_id_to_string(&c.id) == chain)
        .ok_or("Unknown chain")?;
    let rpc_url = config.rpc_urls.first()
        .ok_or("No RPC URL")?;

    match chain {
        "ethereum" | "polygon" | "bsc" | "optimism" | "base" | "arbitrum" => {
            send_evm(&seed, to, amount, rpc_url, config).await
        }
        "solana" => {
            send_solana(&seed, to, amount, rpc_url).await
        }
        "ton" => {
            send_ton(&seed, to, amount, rpc_url).await
        }
        "cosmos" => {
            send_cosmos(&seed, to, amount, rpc_url, "uatom", "cosmoshub-4", wallet_core::chains::ChainId::CosmosHub).await
        }
        "osmosis" => {
            send_cosmos(&seed, to, amount, rpc_url, "uosmo", "osmosis-1", wallet_core::chains::ChainId::Osmosis).await
        }
        _ => Err(format!("Sending not supported for {}", chain)),
    }
}

async fn send_evm(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
    rpc_url: &str,
    config: &wallet_core::chains::ChainConfig,
) -> Result<String, String> {
    use wallet_core::tx::evm::*;
    use wallet_core::chains::evm;

    let private_key = evm::get_private_key(seed)?;
    let from_address = evm::derive_evm_address(seed)?;

    let to_bytes = parse_address(to)?;
    let value = parse_ether_to_wei(amount)?;

    // Fetch nonce and gas
    let nonce = crate::rpc::evm::get_nonce(&from_address, rpc_url).await?;
    let gas_price = crate::rpc::evm::get_gas_price(rpc_url).await?;
    let priority_fee = crate::rpc::evm::get_max_priority_fee(rpc_url).await
        .unwrap_or(1_500_000_000); // fallback 1.5 Gwei

    let evm_chain_id = config.evm_chain_id.ok_or("Missing EVM chain ID")?;

    let tx = EvmTransaction {
        chain_id_num: evm_chain_id,
        nonce,
        max_priority_fee_per_gas: priority_fee,
        max_fee_per_gas: gas_price * 2, // 2x base fee as buffer
        gas_limit: 21000,
        to: to_bytes,
        value,
        data: vec![],
    };

    let signed = tx.sign(&private_key, config.id.clone())?;
    let raw_hex = format!("0x{}", hex::encode(&signed.raw_bytes));

    crate::rpc::evm::send_raw_transaction(&raw_hex, rpc_url).await
}

async fn send_solana(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
    rpc_url: &str,
) -> Result<String, String> {
    use wallet_core::tx::solana::*;
    use wallet_core::chains::solana as sol_chain;

    let keypair = sol_chain::get_keypair(seed)?;
    let private_key: [u8; 32] = keypair[..32].try_into().unwrap();
    let from_pubkey: [u8; 32] = keypair[32..].try_into().unwrap();
    let to_pubkey = parse_pubkey(to)?;
    let lamports = parse_sol_to_lamports(amount)?;

    // Fetch recent blockhash
    let blockhash_b58 = crate::rpc::solana::get_latest_blockhash(rpc_url).await?;
    let blockhash_bytes = bs58::decode(&blockhash_b58).into_vec()
        .map_err(|e| format!("Invalid blockhash: {}", e))?;
    let mut recent_blockhash = [0u8; 32];
    if blockhash_bytes.len() != 32 {
        return Err("Invalid blockhash length".into());
    }
    recent_blockhash.copy_from_slice(&blockhash_bytes);

    let transfer = SolanaTransfer {
        from_pubkey,
        to_pubkey,
        lamports,
        recent_blockhash,
    };

    let signed = transfer.sign(&private_key)?;
    let signed_b58 = bs58::encode(&signed.raw_bytes).into_string();

    crate::rpc::solana::send_transaction(&signed_b58, rpc_url).await
}

async fn send_ton(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
    rpc_url: &str,
) -> Result<String, String> {
    use wallet_core::tx::ton::*;
    use wallet_core::bip32_utils::{self, DerivationPath};

    let path = DerivationPath::bip44(607);
    let (private_key, _) = bip32_utils::derive_ed25519_key_from_seed(seed, &path)?;

    let from_address = wallet_core::chains::ton::derive_ton_address(seed)?;
    let nanoton = parse_ton_to_nanoton(amount)?;

    // Fetch seqno
    let seqno = crate::rpc::ton::get_seqno(&from_address, rpc_url).await
        .unwrap_or(0);

    // Decode destination address to raw bytes (simplified)
    let to_raw = to.as_bytes().to_vec();

    let transfer = TonTransfer {
        to_address_raw: to_raw,
        amount_nanoton: nanoton,
        seqno,
        valid_until: u32::MAX,
    };

    let signed = transfer.sign(&private_key)?;

    // Base64 encode for sendBoc
    let boc_b64 = base64_simple_encode(&signed.raw_bytes);
    crate::rpc::ton::send_boc(&boc_b64, rpc_url).await
}

async fn send_cosmos(
    seed: &[u8; 64],
    to: &str,
    amount: &str,
    rpc_url: &str,
    denom: &str,
    chain_id_str: &str,
    chain_id: wallet_core::chains::ChainId,
) -> Result<String, String> {
    use wallet_core::tx::cosmos::*;
    use wallet_core::bip32_utils::{self, DerivationPath};

    let path = DerivationPath::bip44(118);
    let (private_key, _) = bip32_utils::derive_key_from_seed(seed, &path)?;

    let prefix = if denom == "uatom" { "cosmos" } else { "osmo" };
    let from_address = wallet_core::chains::cosmos::derive_cosmos_address(seed, prefix, 118)?;

    let uamount = parse_atom_to_uatom(amount)?;

    // Fetch account info
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
    let tx_json = String::from_utf8(signed.raw_bytes)
        .map_err(|_| "Invalid TX bytes")?;

    crate::rpc::cosmos::broadcast_tx(&tx_json, rpc_url).await
}

fn base64_simple_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(CHARS[((n >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(CHARS[(n & 0x3F) as usize] as char); } else { result.push('='); }
    }
    result
}
