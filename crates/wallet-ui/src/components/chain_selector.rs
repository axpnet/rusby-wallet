// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;

#[component]
pub fn ChainSelector() -> impl IntoView {
    let wallet_state: ReadSignal<WalletState> = expect_context();
    let set_wallet_state: WriteSignal<WalletState> = expect_context();
    let testnet_mode: ReadSignal<bool> = expect_context();

    let chains = chain_list_for(testnet_mode.get_untracked());

    view! {
        <div class="chain-list">
            {chains.into_iter().map(|chain| {
                let chain_id = chain.id.clone();
                let chain_id_for_active = chain_id.clone();
                let chain_id_for_select = chain_id.clone();
                let chain_id_for_balance = chain_id.clone();
                let name = chain.name.clone();
                let ticker = chain.ticker.clone();
                let ticker_balance = ticker.clone();
                let icon = chain.icon.clone();

                let is_active = move || wallet_state.with(|s| s.active_chain == chain_id_for_active);

                let select = move |_| {
                    let id = chain_id_for_select.clone();
                    set_wallet_state.update(|s| {
                        s.active_chain = id;
                    });
                };

                let balance = {
                    let cid = chain_id_for_balance.clone();
                    move || {
                        wallet_state.with(|s| {
                            s.balances
                                .get(&cid)
                                .cloned()
                                .unwrap_or_else(|| "0.0000".into())
                        })
                    }
                };

                view! {
                    <div
                        class="chain-item"
                        class:active=is_active
                        on:click=select
                    >
                        <div class="chain-icon">{icon}</div>
                        <div class="chain-info">
                            <div class="chain-name">{name}</div>
                            <div class="chain-ticker">{ticker}</div>
                        </div>
                        <div class="chain-balance">
                            {balance} " " {ticker_balance}
                        </div>
                    </div>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
