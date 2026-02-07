// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;
use crate::rpc::history::{TxRecord, TxDirection};
use crate::i18n::t;
use wallet_core::chains::get_chains;

#[component]
pub fn HistoryPage() -> impl IntoView {
    let wallet_state: ReadSignal<WalletState> = expect_context();
    let set_page: WriteSignal<AppPage> = expect_context();
    let testnet_mode: ReadSignal<bool> = expect_context();

    let (transactions, set_transactions) = signal(Vec::<TxRecord>::new());
    let (loading, set_loading) = signal(false);

    // Fetch history when page loads
    Effect::new(move |_| {
        let (is_unlocked, chain, address) = wallet_state.with(|s| {
            (s.is_unlocked, s.active_chain.clone(), s.current_address())
        });
        if !is_unlocked {
            return;
        }
        if address.is_empty() {
            return;
        }

        set_loading.set(true);

        let rpc_url = get_chains(testnet_mode.get()).into_iter()
            .find(|c| crate::rpc::chain_id_str(&c.id) == chain)
            .and_then(|c| c.rpc_urls.first().cloned())
            .unwrap_or_default();

        wasm_bindgen_futures::spawn_local(async move {
            let evm_chains = ["ethereum", "polygon", "bsc", "optimism", "base", "arbitrum"];
            let txs = if evm_chains.contains(&chain.as_str()) {
                crate::rpc::history::fetch_evm_history(&address, &chain, &rpc_url).await
            } else if chain == "solana" {
                crate::rpc::history::fetch_solana_history(&address, &rpc_url).await
            } else if chain == "ton" {
                crate::rpc::history::fetch_ton_history(&address, &rpc_url).await
            } else if chain == "cosmos" || chain == "osmosis" {
                crate::rpc::history::fetch_cosmos_history(&address, &rpc_url, &chain).await
            } else {
                Vec::new()
            };

            set_transactions.set(txs);
            set_loading.set(false);
        });
    });

    view! {
        <div class="p-4">
            <div class="flex items-center justify-between mb-4">
                <button class="btn btn-sm btn-secondary" on:click=move |_| set_page.set(AppPage::Dashboard)>
                    {move || t("history.back")}
                </button>
                <h2>{move || t("history.title")}</h2>
                <div style="width: 60px;" />
            </div>

            {move || {
                if loading.get() {
                    return view! { <p class="text-center text-muted">{t("history.loading")}</p> }.into_any();
                }

                let txs = transactions.get();
                if txs.is_empty() {
                    return view! { <p class="text-center text-muted">{t("history.no_transactions")}</p> }.into_any();
                }

                view! {
                    <div class="chain-list">
                        {txs.into_iter().map(|tx| {
                            let icon = if tx.direction == TxDirection::Sent { "↑" } else { "↓" };
                            let color = if tx.direction == TxDirection::Sent { "var(--danger, #f44336)" } else { "var(--success, #4caf50)" };
                            let label = if tx.direction == TxDirection::Sent { t("history.sent") } else { t("history.received") };
                            let hash_short = if tx.hash.len() > 16 {
                                format!("{}...{}", &tx.hash[..8], &tx.hash[tx.hash.len()-6..])
                            } else {
                                tx.hash.clone()
                            };
                            let explorer_url = tx.explorer_url.clone();

                            view! {
                                <div class="chain-item" style="cursor: pointer;"
                                    on:click=move |_| {
                                        if let Some(window) = web_sys::window() {
                                            let _ = window.open_with_url_and_target(&explorer_url, "_blank");
                                        }
                                    }
                                >
                                    <div class="chain-icon" style=format!("color: {};", color)>{icon}</div>
                                    <div class="chain-info">
                                        <div class="chain-name">{label}</div>
                                        <div class="text-sm text-muted" style="font-family: monospace;">{hash_short}</div>
                                    </div>
                                    <div style="text-align: right;">
                                        <div style=format!("color: {};", color)>
                                            {if tx.direction == TxDirection::Sent { "-" } else { "+" }}
                                            {tx.value.clone()}
                                        </div>
                                        <div class="text-sm text-muted">{tx.timestamp.clone()}</div>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}
