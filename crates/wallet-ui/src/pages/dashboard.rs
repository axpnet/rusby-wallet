// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;
use crate::i18n::t;
use crate::components::chain_selector::ChainSelector;
use crate::components::address_display::AddressDisplay;
use wallet_core::chains::get_chains;

#[component]
pub fn Dashboard() -> impl IntoView {
    let wallet_state: ReadSignal<WalletState> = expect_context();
    let set_wallet_state: WriteSignal<WalletState> = expect_context();
    let set_page: WriteSignal<AppPage> = expect_context();
    let testnet_mode: ReadSignal<bool> = expect_context();
    let FullpageMode(fullpage) = expect_context::<FullpageMode>();

    // Memos break the reactive cascade: Effects depend on these instead of
    // wallet_state directly. When balance/price updates mutate wallet_state,
    // memos re-evaluate but DON'T notify because active_chain/address/is_unlocked
    // haven't changed. This prevents Effects from re-triggering each other.
    let is_unlocked = Memo::new(move |_| wallet_state.with(|s| s.is_unlocked));
    let active_chain = Memo::new(move |_| wallet_state.with(|s| s.active_chain.clone()));
    let current_address = Memo::new(move |_| wallet_state.with(|s| s.current_address()));

    let active_chain_info = move || {
        let chain = active_chain.get();
        let chains = chain_list();
        chains.into_iter()
            .find(|c| c.id == chain)
            .map(|c| (c.name, c.ticker, c.icon_path))
            .unwrap_or(("Unknown".into(), "???".into(), String::new()))
    };

    // Fetch balance when chain or address changes
    Effect::new(move |_| {
        if !is_unlocked.get() { return; }
        let chain = active_chain.get();
        let address = current_address.get();
        if address.is_empty() { return; }

        set_wallet_state.update(|s| s.balance_loading = true);

        let testnet = testnet_mode.get();
        wasm_bindgen_futures::spawn_local(async move {
            let result = crate::rpc::fetch_balance_for_network(&chain, &address, testnet).await;
            set_wallet_state.update(|s| {
                s.balance_loading = false;
                if let Ok(balance) = result {
                    s.balances.insert(chain.clone(), balance);
                }
            });
        });
    });

    // Fetch ERC-20 token balances for EVM chains (mainnet only)
    Effect::new(move |_| {
        if !is_unlocked.get() || testnet_mode.get() { return; }
        let chain = active_chain.get();
        let address = current_address.get();
        if address.is_empty() { return; }

        let evm_chains = ["ethereum", "polygon", "bsc", "optimism", "base", "arbitrum"];
        if !evm_chains.contains(&chain.as_str()) { return; }

        let rpc_url = get_chains(false).into_iter()
            .find(|c| crate::rpc::chain_id_str(&c.id) == chain)
            .and_then(|c| c.rpc_urls.first().cloned());

        if let Some(rpc_url) = rpc_url {
            wasm_bindgen_futures::spawn_local(async move {
                let tokens = crate::rpc::erc20::get_all_token_balances(&address, &chain, &rpc_url).await;
                set_wallet_state.update(|s| {
                    s.token_balances.insert(chain.clone(), tokens);
                });
            });
        }
    });

    // Fetch SPL token balances for Solana (mainnet only)
    Effect::new(move |_| {
        if !is_unlocked.get() || testnet_mode.get() { return; }
        let chain = active_chain.get();
        let address = current_address.get();
        if chain != "solana" || address.is_empty() { return; }

        let rpc_url = get_chains(false).into_iter()
            .find(|c| crate::rpc::chain_id_str(&c.id) == "solana")
            .and_then(|c| c.rpc_urls.first().cloned());

        if let Some(rpc_url) = rpc_url {
            wasm_bindgen_futures::spawn_local(async move {
                let tokens = crate::rpc::spl::get_all_token_balances(&address, &rpc_url).await;
                set_wallet_state.update(|s| {
                    s.token_balances.insert("solana".to_string(), tokens);
                });
            });
        }
    });

    // Fetch CW-20 token balances for Cosmos/Osmosis (mainnet only)
    Effect::new(move |_| {
        if !is_unlocked.get() || testnet_mode.get() { return; }
        let chain = active_chain.get();
        let address = current_address.get();
        if address.is_empty() { return; }
        if chain != "cosmos" && chain != "osmosis" { return; }

        let rpc_url = get_chains(false).into_iter()
            .find(|c| crate::rpc::chain_id_str(&c.id) == chain)
            .and_then(|c| c.rpc_urls.first().cloned());

        if let Some(rpc_url) = rpc_url {
            let chain_clone = chain.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let cw20_tokens = crate::rpc::cw20::get_all_token_balances(&address, &chain_clone, &rpc_url).await;
                let ibc_tokens = crate::rpc::cosmos::get_all_balances(&address, &rpc_url, &chain_clone).await.unwrap_or_default();

                set_wallet_state.update(|s| {
                    let mut all = cw20_tokens;
                    all.extend(ibc_tokens);
                    if !all.is_empty() {
                        let existing = s.token_balances.entry(chain_clone.clone()).or_insert_with(Vec::new);
                        for token in all {
                            if !existing.iter().any(|t| t.token.address == token.token.address) {
                                existing.push(token);
                            }
                        }
                    }
                });
            });
        }
    });

    // Fetch Jetton balances for TON (mainnet only)
    Effect::new(move |_| {
        if !is_unlocked.get() || testnet_mode.get() { return; }
        let chain = active_chain.get();
        let address = current_address.get();
        if chain != "ton" || address.is_empty() { return; }

        let rpc_url = get_chains(false).into_iter()
            .find(|c| crate::rpc::chain_id_str(&c.id) == "ton")
            .and_then(|c| c.rpc_urls.first().cloned());

        if let Some(rpc_url) = rpc_url {
            wasm_bindgen_futures::spawn_local(async move {
                let tokens = crate::rpc::jetton::get_all_token_balances(&address, &rpc_url).await;
                set_wallet_state.update(|s| {
                    s.token_balances.insert("ton".to_string(), tokens);
                });
            });
        }
    });

    // Fetch USD prices on load (depends only on is_unlocked memo)
    Effect::new(move |_| {
        if !is_unlocked.get() { return; }
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(prices) = crate::rpc::prices::fetch_prices().await {
                set_wallet_state.update(|s| s.prices = prices);
            }
        });
    });

    // Auto-refresh every 30 seconds — ONE-TIME setup (NOT inside an Effect!).
    // Previous approach created a new Interval inside an Effect on every
    // wallet_state change, leaking unbounded intervals that never get canceled.
    // Now: single Interval that reads current state with with_untracked().
    {
        let handle = gloo_timers::callback::Interval::new(30_000, move || {
            let (unlocked, chain, address) = wallet_state.with_untracked(|s| {
                (s.is_unlocked, s.active_chain.clone(), s.current_address())
            });
            if !unlocked || address.is_empty() { return; }
            let testnet = testnet_mode.get_untracked();

            let chain2 = chain.clone();
            let address2 = address.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = crate::rpc::fetch_balance_for_network(&chain2, &address2, testnet).await;
                set_wallet_state.update(|s| {
                    if let Ok(balance) = result {
                        s.balances.insert(chain2.clone(), balance);
                    }
                });
            });
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(prices) = crate::rpc::prices::fetch_prices().await {
                    set_wallet_state.update(|s| s.prices = prices);
                }
            });
        });
        // Note: handle.forget() leaks the interval, but gloo_timers::Interval
        // is not Send+Sync so on_cleanup() can't be used in Leptos 0.7.
        // With with_untracked() the cost per tick is minimal (no full state clone).
        handle.forget();
    }

    view! {
        <div>
            // Balance hero
            <div class="balance-hero">
                <div style="display: flex; align-items: center; justify-content: center; gap: 8px; margin-bottom: 4px;">
                    <img
                        src=move || active_chain_info().2
                        alt=""
                        style="width: 24px; height: 24px; border-radius: 50%;"
                    />
                    <p class="text-sm text-muted" style="margin: 0;">{move || wallet_state.with(|s| s.wallet_name.clone())}</p>
                </div>
                <div class="balance-amount">
                    {move || {
                        let (loading, balance) = wallet_state.with(|s| (s.balance_loading, s.current_balance()));
                        if loading {
                            t("dashboard.loading")
                        } else {
                            balance
                        }
                    }}
                    " "
                    {move || active_chain_info().1}
                </div>
                <p class="balance-fiat">
                    {move || {
                        wallet_state.with(|s| {
                            let balance_str = s.current_balance();
                            let price = s.prices.get(&s.active_chain).copied().unwrap_or(0.0);
                            let balance: f64 = balance_str.parse().unwrap_or(0.0);
                            let usd = balance * price;
                            format!("$ {:.2} USD", usd)
                        })
                    }}
                </p>
            </div>

            // Action buttons
            <div class="action-row">
                <button class="action-btn" on:click=move |_| set_page.set(AppPage::Send)>
                    <span class="icon">"↑"</span>
                    {move || t("dashboard.send")}
                </button>
                <button class="action-btn" on:click=move |_| set_page.set(AppPage::Receive)>
                    <span class="icon">"↓"</span>
                    {move || t("dashboard.receive")}
                </button>
                <button class="action-btn" on:click=move |_| set_page.set(AppPage::Swap)>
                    <span class="icon">"↔"</span>
                    {move || t("dashboard.swap")}
                </button>
                <button class="action-btn" on:click=move |_| set_page.set(AppPage::Nft)>
                    <span class="icon">"◆"</span>
                    {move || t("dashboard.nft")}
                </button>
            </div>

            // Current address
            <div style="padding: 0 20px 16px;">
                <AddressDisplay />
            </div>

            // Token balances
            {move || {
                let tokens = wallet_state.with(|s| {
                    s.token_balances.get(&s.active_chain).cloned().unwrap_or_default()
                });
                if tokens.is_empty() {
                    None
                } else {
                    Some(view! {
                        <div style="padding: 0 20px 16px;">
                            <h3 class="text-sm text-muted mb-2">{move || t("dashboard.tokens")}</h3>
                            <div class="chain-list">
                                {tokens.into_iter().map(|tb| {
                                    view! {
                                        <div class="chain-item">
                                            <div class="chain-icon">{tb.token.symbol.chars().next().unwrap_or('?').to_string()}</div>
                                            <div class="chain-info">
                                                <div class="chain-name">{tb.token.symbol.clone()}</div>
                                                <div class="text-sm text-muted">{tb.token.name.clone()}</div>
                                            </div>
                                            <div style="text-align: right;">
                                                <div>{tb.balance.clone()}</div>
                                            </div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    })
                }
            }}

            // Chain selector — only in popup mode (fullpage has ChainSidebar)
            {move || (!fullpage.get()).then(|| view! {
                <div style="padding: 0 20px;">
                    <h3 class="text-sm text-muted mb-2">{move || t("dashboard.networks")}</h3>
                    <ChainSelector />
                </div>
            })}
        </div>
    }
}
