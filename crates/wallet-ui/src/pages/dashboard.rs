// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;
use crate::components::chain_selector::ChainSelector;
use crate::components::address_display::AddressDisplay;

#[component]
pub fn Dashboard() -> impl IntoView {
    let wallet_state: ReadSignal<WalletState> = expect_context();
    let set_wallet_state: WriteSignal<WalletState> = expect_context();
    let set_page: WriteSignal<AppPage> = expect_context();

    let active_chain_info = move || {
        let state = wallet_state.get();
        let chains = chain_list();
        chains.into_iter()
            .find(|c| c.id == state.active_chain)
            .map(|c| (c.name, c.ticker))
            .unwrap_or(("Unknown".into(), "???".into()))
    };

    // Fetch balance when chain or address changes
    Effect::new(move |_| {
        let state = wallet_state.get();
        if !state.is_unlocked {
            return;
        }
        let chain = state.active_chain.clone();
        let address = state.current_address();
        if address.is_empty() {
            return;
        }

        // Mark as loading
        set_wallet_state.update(|s| s.balance_loading = true);

        wasm_bindgen_futures::spawn_local(async move {
            let result = crate::rpc::fetch_balance(&chain, &address).await;
            set_wallet_state.update(|s| {
                s.balance_loading = false;
                if let Ok(balance) = result {
                    s.balances.insert(chain.clone(), balance);
                }
            });
        });
    });

    // Auto-refresh every 30 seconds
    Effect::new(move |_| {
        let state = wallet_state.get();
        if !state.is_unlocked {
            return;
        }
        let chain = state.active_chain.clone();
        let address = state.current_address();
        if address.is_empty() {
            return;
        }

        let handle = gloo_timers::callback::Interval::new(30_000, move || {
            let chain = chain.clone();
            let address = address.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = crate::rpc::fetch_balance(&chain, &address).await;
                set_wallet_state.update(|s| {
                    if let Ok(balance) = result {
                        s.balances.insert(chain.clone(), balance);
                    }
                });
            });
        });
        handle.forget();
    });

    view! {
        <div>
            // Balance hero
            <div class="balance-hero">
                <p class="text-sm text-muted">{move || wallet_state.get().wallet_name.clone()}</p>
                <div class="balance-amount">
                    {move || {
                        if wallet_state.get().balance_loading {
                            "Loading...".to_string()
                        } else {
                            wallet_state.get().current_balance()
                        }
                    }}
                    " "
                    {move || active_chain_info().1}
                </div>
                <p class="balance-fiat">"$ 0.00 USD"</p>
            </div>

            // Action buttons
            <div class="action-row">
                <button class="action-btn" on:click=move |_| set_page.set(AppPage::Send)>
                    <span class="icon">"↑"</span>
                    "Send"
                </button>
                <button class="action-btn" on:click=move |_| set_page.set(AppPage::Receive)>
                    <span class="icon">"↓"</span>
                    "Receive"
                </button>
            </div>

            // Current address
            <div style="padding: 0 20px 16px;">
                <AddressDisplay />
            </div>

            // Chain selector
            <div style="padding: 0 20px;">
                <h3 class="text-sm text-muted mb-2">"Networks"</h3>
                <ChainSelector />
            </div>
        </div>
    }
}
