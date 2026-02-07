// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;
use crate::i18n::t;

/// Chain selector sidebar for fullpage mode.
/// Shows all enabled chains with name, icon and balance.
#[component]
pub fn ChainSidebar() -> impl IntoView {
    let wallet_state: ReadSignal<WalletState> = expect_context();
    let set_wallet_state: WriteSignal<WalletState> = expect_context();
    let testnet_mode: ReadSignal<bool> = expect_context();

    let chains = chain_list_for(testnet_mode.get_untracked());

    view! {
        <aside class="chain-sidebar">
            <div class="chain-sidebar-header">
                <h3 class="chain-sidebar-title">{move || t("dashboard.networks")}</h3>
            </div>
            <div class="chain-sidebar-list">
                {chains.into_iter().map(|chain| {
                    let chain_id_active = chain.id.clone();
                    let chain_id_select = chain.id.clone();
                    let chain_id_balance = chain.id.clone();
                    let name = chain.name.clone();
                    let ticker = chain.ticker.clone();
                    let icon_path = chain.icon_path;

                    let is_active = move || wallet_state.with(|s| s.active_chain == chain_id_active);

                    let select = move |_| {
                        let id = chain_id_select.clone();
                        set_wallet_state.update(|s| s.active_chain = id);
                    };

                    let balance_text = {
                        let cid = chain_id_balance;
                        let tick = ticker;
                        move || {
                            let bal = wallet_state.with(|s| {
                                s.balances.get(&cid).cloned().unwrap_or_else(|| "0.0000".into())
                            });
                            format!("{} {}", bal, tick)
                        }
                    };

                    view! {
                        <button
                            class="chain-sidebar-item"
                            class:active=is_active
                            on:click=select
                        >
                            <div class="chain-sidebar-icon">
                                <img src=icon_path alt=name.clone() class="chain-icon-img" />
                            </div>
                            <div class="chain-sidebar-info">
                                <div class="chain-sidebar-name">{name}</div>
                                <div class="chain-sidebar-balance">{balance_text}</div>
                            </div>
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </aside>
    }
}
