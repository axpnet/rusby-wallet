// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;
use wallet_core::wallet::{self, WalletStore};
use zeroize::Zeroize;

use crate::state::*;
use crate::i18n::t;
use crate::components::SPINNER_SVG;

#[component]
pub fn Login() -> impl IntoView {
    let (password, set_password) = signal(String::new());
    let (error_msg, set_error_msg) = signal(String::new());
    let (selected_index, set_selected_index) = signal(0usize);
    let (loading, set_loading) = signal(false);
    let (show_password, set_show_password) = signal(false);
    let (loading_text, set_loading_text) = signal(String::new());

    let set_page: WriteSignal<AppPage> = expect_context();
    let set_wallet_state: WriteSignal<WalletState> = expect_context();

    let load_store = move || -> Option<WalletStore> {
        let json = load_from_storage("wallet_store")?;
        serde_json::from_str(&json).ok()
    };

    let load_enabled_chains = move || -> Vec<String> {
        load_from_storage("enabled_chains")
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_else(|| vec!["ethereum".to_string()])
    };

    let do_unlock = move || {
        if loading.get() { return; }

        let Some(s) = load_store() else {
            set_error_msg.set(t("login.no_wallet"));
            return;
        };

        let pass = password.get();
        let idx = selected_index.get();
        let chains = load_enabled_chains();

        // Show loading, then defer heavy PBKDF2 computation to let UI render
        set_loading.set(true);
        set_loading_text.set(t("loading.pbkdf2_decrypt"));
        set_error_msg.set(String::new());

        // Two-phase unlock to avoid blocking main thread for >5s:
        // Phase 1 (Timeout 50ms): PBKDF2 decrypt seed (~1-3s)
        // Phase 2 (Timeout 0ms):  Derive chain addresses (~1-3s)
        gloo_timers::callback::Timeout::new(50, move || {
            // Phase 1: decrypt seed (PBKDF2)
            match s.decrypt_seed(idx, &pass) {
                Ok((name, created_at, seed)) => {
                    // Phase 2: derive addresses (deferred to next event loop tick)
                    set_loading_text.set(t("loading.deriving_keys"));
                    gloo_timers::callback::Timeout::new(0, move || {
                        let chain_strs: Vec<&str> = chains.iter().map(|s| s.as_str()).collect();
                        let mut seed_copy = seed;
                        match wallet::derive_addresses_filtered(&seed_copy, false, Some(&chain_strs)) {
                            Ok(addresses) => {
                                seed_copy.zeroize();
                                let active = chains.first()
                                    .map(|s| s.as_str())
                                    .unwrap_or("ethereum")
                                    .to_string();

                                set_wallet_state.set(WalletState {
                                    is_unlocked: true,
                                    wallet_name: name,
                                    addresses,
                                    active_chain: active,
                                    balances: std::collections::HashMap::new(),
                                    balance_loading: false,
                                    prices: std::collections::HashMap::new(),
                                    token_balances: std::collections::HashMap::new(),
                                    nfts: Vec::new(),
                                });
                                set_page.set(AppPage::Dashboard);
                            }
                            Err(e) => {
                                seed_copy.zeroize();
                                set_loading.set(false);
                                set_error_msg.set(format!("Error: {}", e));
                            }
                        }
                    }).forget();
                }
                Err(_) => {
                    set_loading.set(false);
                    set_error_msg.set(t("login.wrong_password"));
                }
            }
        }).forget();
    };

    let unlock_click = move |_| do_unlock();

    let unlock_enter = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Enter" {
            do_unlock();
        }
    };

    let go_to_create = move |_| {
        set_page.set(AppPage::Onboarding);
    };

    view! {
        <div class="p-4">
            <h2 class="text-center mb-4">{move || t("login.title")}</h2>

            <div class="chain-list mb-4">
                {move || {
                    let names = load_store()
                        .map(|s| s.wallet_names())
                        .unwrap_or_default();
                    names.into_iter().enumerate().map(|(i, name)| {
                        let first_char = name.chars().next().unwrap_or('W').to_string();
                        let active_class = move || {
                            if selected_index.get() == i { "chain-item active" } else { "chain-item" }
                        };
                        view! {
                            <div
                                class=active_class
                                on:click=move |_| set_selected_index.set(i)
                            >
                                <div class="chain-icon">{first_char.clone()}</div>
                                <div class="chain-info">
                                    <div class="chain-name">{name}</div>
                                </div>
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>

            <div class="input-group">
                <label>{move || t("login.password")}</label>
                <div style="position: relative;">
                    <input
                        type=move || if show_password.get() { "text" } else { "password" }
                        placeholder=t("login.password_placeholder")
                        prop:value=move || password.get()
                        on:input=move |ev| {
                            set_password.set(event_target_value(&ev));
                            set_error_msg.set(String::new());
                        }
                        on:keydown=unlock_enter
                        style="padding-right: 40px;"
                    />
                    <button
                        type="button"
                        tabindex="-1"
                        style="position: absolute; right: 8px; top: 50%; transform: translateY(-50%); background: none; border: none; color: var(--text-muted); cursor: pointer; font-size: 18px; padding: 4px;"
                        on:click=move |_| set_show_password.set(!show_password.get())
                    >
                        {move || if show_password.get() { "\u{1F648}" } else { "\u{1F441}" }}
                    </button>
                </div>
            </div>

            {move || {
                let err = error_msg.get();
                if err.is_empty() { None } else {
                    Some(view! { <p style="color: var(--danger); margin: 8px 0;">{err}</p> })
                }
            }}

            <button
                class="btn btn-primary btn-block"
                on:click=unlock_click
                disabled=move || loading.get()
                style="display: flex; align-items: center; justify-content: center; gap: 8px;"
            >
                {move || loading.get().then(|| view! { <span inner_html=SPINNER_SVG /> })}
                {move || if loading.get() {
                    loading_text.get()
                } else {
                    t("login.unlock")
                }}
            </button>

            <button class="btn btn-secondary btn-block mt-4" on:click=go_to_create
                disabled=move || loading.get()
            >
                {move || t("login.create_new")}
            </button>
        </div>
    }
}
