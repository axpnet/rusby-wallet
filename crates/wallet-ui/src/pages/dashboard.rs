// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;
use crate::components::chain_selector::ChainSelector;
use crate::components::address_display::AddressDisplay;

#[component]
pub fn Dashboard() -> impl IntoView {
    let wallet_state: ReadSignal<WalletState> = expect_context();
    let set_page: WriteSignal<AppPage> = expect_context();

    let active_chain_info = move || {
        let state = wallet_state.get();
        let chains = chain_list();
        chains.into_iter()
            .find(|c| c.id == state.active_chain)
            .map(|c| (c.name, c.ticker))
            .unwrap_or(("Unknown".into(), "???".into()))
    };

    view! {
        <div>
            // Balance hero
            <div class="balance-hero">
                <p class="text-sm text-muted">{move || wallet_state.get().wallet_name.clone()}</p>
                <div class="balance-amount">
                    {move || wallet_state.get().current_balance()}
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
