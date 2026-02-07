// Rusby Wallet — Swap page
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;
use crate::i18n::t;
use wallet_core::swap::{self, SwapQuote, SwapParams, evm_chain_id};

#[component]
pub fn SwapPage() -> impl IntoView {
    let wallet_state: ReadSignal<WalletState> = expect_context();
    let set_page: WriteSignal<AppPage> = expect_context();
    let testnet_mode: ReadSignal<bool> = expect_context();

    let (sell_token, set_sell_token) = signal(String::new());
    let (buy_token, set_buy_token) = signal(String::new());
    let (sell_amount, set_sell_amount) = signal(String::new());
    let (buy_amount, set_buy_amount) = signal(String::new());
    let (quote, set_quote) = signal::<Option<SwapQuote>>(None);
    let (loading, set_loading) = signal(false);
    let (status, set_status) = signal(String::new());
    let (status_type, set_status_type) = signal("warning");
    let (slippage, set_slippage) = signal(50u16); // 0.5%
    let (show_password, set_show_password) = signal(false);
    let (password, set_password) = signal(String::new());
    let (executing, set_executing) = signal(false);

    let active_chain = move || wallet_state.with(|s| s.active_chain.clone());

    let is_evm = move || {
        matches!(active_chain().as_str(), "ethereum" | "polygon" | "bsc" | "optimism" | "base" | "arbitrum")
    };

    let swap_tokens = move || {
        swap::common_swap_tokens(&active_chain())
    };

    // Initialize sell token to native
    Effect::new(move |_| {
        let tokens = swap::common_swap_tokens(&active_chain());
        if let Some(first) = tokens.first() {
            if sell_token.get().is_empty() {
                set_sell_token.set(first.address.clone());
            }
        }
    });

    let on_flip = move |_| {
        let sell = sell_token.get();
        let buy = buy_token.get();
        set_sell_token.set(buy);
        set_buy_token.set(sell);
        set_buy_amount.set(String::new());
        set_quote.set(None);
    };

    let on_get_quote = move |_| {
        let chain = active_chain();
        let chain_numeric = match evm_chain_id(&chain) {
            Some(id) => id,
            None => {
                set_status.set(t("swap.evm_only"));
                set_status_type.set("warning");
                return;
            }
        };

        let api_key = match crate::rpc::swap::get_api_key() {
            Some(k) => k,
            None => {
                set_status.set(t("swap.no_api_key"));
                set_status_type.set("warning");
                return;
            }
        };

        let sell = sell_token.get();
        let buy = buy_token.get();
        let amount = sell_amount.get();

        if sell.is_empty() || buy.is_empty() {
            set_status.set(t("swap.select_token"));
            set_status_type.set("warning");
            return;
        }
        if amount.is_empty() || amount.parse::<f64>().is_err() {
            set_status.set(t("send.enter_amount"));
            set_status_type.set("warning");
            return;
        }
        if sell == buy {
            set_status.set("Cannot swap same token".into());
            set_status_type.set("warning");
            return;
        }

        // Find decimals for sell token
        let tokens = swap::common_swap_tokens(&chain);
        let sell_decimals = tokens.iter()
            .find(|t| t.address == sell)
            .map(|t| t.decimals)
            .unwrap_or(18);

        let raw_amount = match swap::parse_swap_amount(&amount, sell_decimals) {
            Ok(r) => r,
            Err(e) => {
                set_status.set(e);
                set_status_type.set("danger");
                return;
            }
        };

        let taker = wallet_state.with(|s| s.current_address());
        let slippage_bps = slippage.get();

        let params = SwapParams {
            sell_token: sell,
            buy_token: buy,
            sell_amount: raw_amount,
            taker_address: taker,
            slippage_bps,
            chain_id: chain_numeric,
        };

        set_loading.set(true);
        set_status.set(t("swap.fetching_quote"));
        set_status_type.set("warning");

        wasm_bindgen_futures::spawn_local(async move {
            match crate::rpc::swap::get_swap_price(&params, &api_key).await {
                Ok(q) => {
                    // Format buy amount for display
                    let buy_tokens_list = swap::common_swap_tokens(&chain);
                    let buy_decimals = buy_tokens_list.iter()
                        .find(|t| t.address == params.buy_token)
                        .map(|t| t.decimals)
                        .unwrap_or(18);
                    let formatted = swap::format_swap_amount(&q.buy_amount, buy_decimals);
                    set_buy_amount.set(formatted);
                    set_quote.set(Some(q));
                    set_status.set(String::new());
                }
                Err(e) => {
                    set_status.set(format!("Quote error: {}", e));
                    set_status_type.set("danger");
                    set_quote.set(None);
                }
            }
            set_loading.set(false);
        });
    };

    let on_swap_click = move |_| {
        if quote.get().is_none() {
            return;
        }
        set_show_password.set(true);
    };

    let on_execute_swap = move |_| {
        let chain = active_chain();
        let chain_numeric = match evm_chain_id(&chain) {
            Some(id) => id,
            None => return,
        };
        let api_key = match crate::rpc::swap::get_api_key() {
            Some(k) => k,
            None => return,
        };
        let pwd = password.get();
        if pwd.is_empty() {
            return;
        }

        let sell = sell_token.get();
        let buy = buy_token.get();
        let tokens = swap::common_swap_tokens(&chain);
        let sell_decimals = tokens.iter()
            .find(|t| t.address == sell)
            .map(|t| t.decimals)
            .unwrap_or(18);

        let raw_amount = match swap::parse_swap_amount(&sell_amount.get(), sell_decimals) {
            Ok(r) => r,
            Err(_) => return,
        };

        let taker = wallet_state.with(|s| s.current_address());

        let params = SwapParams {
            sell_token: sell,
            buy_token: buy,
            sell_amount: raw_amount,
            taker_address: taker,
            slippage_bps: slippage.get(),
            chain_id: chain_numeric,
        };

        set_show_password.set(false);
        set_executing.set(true);
        set_status.set(t("send.signing"));
        set_status_type.set("warning");

        let testnet = testnet_mode.get();
        wasm_bindgen_futures::spawn_local(async move {
            match crate::rpc::swap::get_swap_quote(&params, &api_key).await {
                Ok((_quote, tx_data)) => {
                    // Execute the swap TX
                    let result = execute_swap_tx(&chain, &pwd, &tx_data, testnet).await;
                    set_executing.set(false);
                    match result {
                        Ok(hash) => {
                            set_status.set(format!("{} {}", t("send.tx_sent"), hash));
                            set_status_type.set("success");
                        }
                        Err(e) => {
                            set_status.set(format!("{} {}", t("send.error"), e));
                            set_status_type.set("danger");
                        }
                    }
                }
                Err(e) => {
                    set_executing.set(false);
                    set_status.set(format!("Swap error: {}", e));
                    set_status_type.set("danger");
                }
            }
        });
    };

    view! {
        <div class="p-4">
            // Header
            <div class="flex items-center justify-between mb-4">
                <button class="btn btn-sm btn-secondary" on:click=move |_| set_page.set(AppPage::Dashboard)>
                    {move || t("send.back")}
                </button>
                <h2>{move || t("swap.title")}</h2>
                <div style="width: 60px;" />
            </div>

            // EVM-only guard
            {move || {
                if !is_evm() {
                    Some(view! {
                        <div class="card text-center" style="padding: 40px 20px;">
                            <div style="font-size: 48px; margin-bottom: 12px; color: var(--text-muted);">"↔"</div>
                            <p class="text-muted">{t("swap.evm_only")}</p>
                        </div>
                    })
                } else {
                    None
                }
            }}

            // Swap form (only for EVM)
            {move || {
                if !is_evm() {
                    return None;
                }

                let tokens = swap_tokens();
                let tokens_buy = swap_tokens();

                Some(view! {
                    <div>
                        // Sell section
                        <div class="card" style="margin-bottom: 4px;">
                            <label class="text-sm text-muted">{t("swap.sell")}</label>
                            <div style="display: flex; gap: 8px; margin-top: 8px;">
                                <select
                                    style="flex: 1; padding: 10px; border-radius: 8px; background: var(--bg-input); color: var(--text-primary); border: 1px solid var(--border);"
                                    prop:value=move || sell_token.get()
                                    on:change=move |ev| {
                                        set_sell_token.set(event_target_value(&ev));
                                        set_quote.set(None);
                                        set_buy_amount.set(String::new());
                                    }
                                >
                                    {tokens.into_iter().map(|t| {
                                        let addr = t.address.clone();
                                        let label = format!("{} {}", t.logo_char, t.symbol);
                                        view! { <option value=addr>{label}</option> }
                                    }).collect::<Vec<_>>()}
                                </select>
                                <input
                                    type="text"
                                    placeholder="0.0"
                                    style="flex: 1; padding: 10px; border-radius: 8px; background: var(--bg-input); color: var(--text-primary); border: 1px solid var(--border);"
                                    prop:value=move || sell_amount.get()
                                    on:input=move |ev| set_sell_amount.set(event_target_value(&ev))
                                />
                            </div>
                        </div>

                        // Flip button
                        <div style="text-align: center; margin: 4px 0;">
                            <button
                                class="btn btn-sm btn-icon"
                                style="font-size: 18px;"
                                on:click=on_flip
                            >
                                "↕"
                            </button>
                        </div>

                        // Buy section
                        <div class="card" style="margin-bottom: 12px;">
                            <label class="text-sm text-muted">{t("swap.buy")}</label>
                            <div style="display: flex; gap: 8px; margin-top: 8px;">
                                <select
                                    style="flex: 1; padding: 10px; border-radius: 8px; background: var(--bg-input); color: var(--text-primary); border: 1px solid var(--border);"
                                    prop:value=move || buy_token.get()
                                    on:change=move |ev| {
                                        set_buy_token.set(event_target_value(&ev));
                                        set_quote.set(None);
                                        set_buy_amount.set(String::new());
                                    }
                                >
                                    <option value="">{t("swap.select_token")}</option>
                                    {tokens_buy.into_iter().map(|t| {
                                        let addr = t.address.clone();
                                        let label = format!("{} {}", t.logo_char, t.symbol);
                                        view! { <option value=addr>{label}</option> }
                                    }).collect::<Vec<_>>()}
                                </select>
                                <input
                                    type="text"
                                    placeholder="0.0"
                                    style="flex: 1; padding: 10px; border-radius: 8px; background: var(--bg-input); color: var(--text-primary); border: 1px solid var(--border); opacity: 0.7;"
                                    prop:value=move || buy_amount.get()
                                    readonly=true
                                />
                            </div>
                        </div>

                        // Slippage setting
                        <div class="card" style="margin-bottom: 12px;">
                            <div class="flex justify-between items-center">
                                <span class="text-sm text-muted">{t("swap.slippage")}</span>
                                <div style="display: flex; gap: 4px;">
                                    {[30u16, 50, 100, 300].into_iter().map(|bps| {
                                        let label = format!("{}%", bps as f64 / 100.0);
                                        let is_active = move || slippage.get() == bps;
                                        view! {
                                            <button
                                                class="btn btn-sm"
                                                class:btn-primary=is_active
                                                class:btn-secondary=move || !is_active()
                                                on:click=move |_| set_slippage.set(bps)
                                            >
                                                {label}
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </div>
                        </div>

                        // Quote details
                        {move || {
                            quote.get().map(|q| {
                                let rate = q.price.clone();
                                let gas = q.estimated_gas;
                                let sources: Vec<String> = q.sources.iter()
                                    .map(|s| s.name.clone())
                                    .collect();
                                let sources_str = if sources.is_empty() {
                                    "Direct".to_string()
                                } else {
                                    sources.join(", ")
                                };

                                view! {
                                    <div class="card" style="margin-bottom: 12px; font-size: 13px;">
                                        <div class="flex justify-between mb-2">
                                            <span class="text-muted">{t("swap.rate")}</span>
                                            <span>{rate}</span>
                                        </div>
                                        <div class="flex justify-between mb-2">
                                            <span class="text-muted">{t("swap.gas_estimate")}</span>
                                            <span>{format!("{}", gas)}</span>
                                        </div>
                                        <div class="flex justify-between">
                                            <span class="text-muted">{t("swap.sources")}</span>
                                            <span style="text-align: right; max-width: 200px; overflow: hidden; text-overflow: ellipsis;">{sources_str}</span>
                                        </div>
                                    </div>
                                }
                            })
                        }}

                        // Status
                        {move || {
                            let s = status.get();
                            if s.is_empty() { None } else {
                                let color = match status_type.get() {
                                    "success" => "var(--success, #4caf50)",
                                    "danger" => "var(--danger, #f44336)",
                                    _ => "var(--warning, #ff9800)",
                                };
                                Some(view! {
                                    <p class="text-sm" style=format!("color: {}; word-break: break-all; margin-bottom: 8px;", color)>{s}</p>
                                })
                            }
                        }}

                        // Buttons
                        {move || {
                            if quote.get().is_some() {
                                view! {
                                    <button
                                        class="btn btn-primary btn-block"
                                        on:click=on_swap_click
                                        disabled=move || executing.get()
                                    >
                                        {move || if executing.get() { t("send.sending") } else { t("swap.execute") }}
                                    </button>
                                }.into_any()
                            } else {
                                view! {
                                    <button
                                        class="btn btn-primary btn-block"
                                        on:click=on_get_quote
                                        disabled=move || loading.get()
                                    >
                                        {move || if loading.get() { t("swap.fetching_quote") } else { t("swap.get_quote") }}
                                    </button>
                                }.into_any()
                            }
                        }}

                        // Password modal
                        {move || {
                            if show_password.get() {
                                Some(view! {
                                    <div
                                        class="nft-detail-overlay"
                                        on:click=move |_| set_show_password.set(false)
                                    >
                                        <div
                                            class="nft-detail"
                                            on:click=move |ev| ev.stop_propagation()
                                        >
                                            <h3 style="margin-bottom: 12px;">{t("swap.confirm_swap")}</h3>
                                            <div class="input-group">
                                                <label>{t("confirm.password")}</label>
                                                <input
                                                    type="password"
                                                    prop:value=move || password.get()
                                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                                />
                                            </div>
                                            <button
                                                class="btn btn-primary btn-block"
                                                style="margin-top: 12px;"
                                                on:click=on_execute_swap
                                            >
                                                {move || t("swap.execute")}
                                            </button>
                                            <button
                                                class="btn btn-secondary btn-block"
                                                style="margin-top: 8px;"
                                                on:click=move |_| set_show_password.set(false)
                                            >
                                                {move || t("confirm.cancel")}
                                            </button>
                                        </div>
                                    </div>
                                })
                            } else {
                                None
                            }
                        }}
                    </div>
                })
            }}
        </div>
    }
}

/// Execute the swap transaction via existing EVM TX infrastructure
async fn execute_swap_tx(
    chain: &str,
    password: &str,
    tx_data: &crate::rpc::swap::SwapTxData,
    testnet: bool,
) -> Result<String, String> {
    use crate::tx_send;

    let store_json = load_from_storage("wallet_store")
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

    let chains = wallet_core::chains::get_chains(testnet);
    let config = chains.iter()
        .find(|c| tx_send::chain_id_to_string(&c.id) == chain)
        .ok_or("Unknown chain")?;
    let rpc_url = config.rpc_urls.first()
        .ok_or("No RPC URL")?;

    tx_send::evm::send_swap_tx(
        &seed,
        &tx_data.to,
        &tx_data.value,
        &tx_data.data,
        tx_data.gas_limit,
        rpc_url,
        config,
    ).await
}
