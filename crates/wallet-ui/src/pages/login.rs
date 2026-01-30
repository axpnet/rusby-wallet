// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;
use wallet_core::wallet::WalletStore;

use crate::state::*;

#[component]
pub fn Login() -> impl IntoView {
    let (password, set_password) = signal(String::new());
    let (error_msg, set_error_msg) = signal(String::new());
    let (selected_index, set_selected_index) = signal(0usize);

    let set_page: WriteSignal<AppPage> = expect_context();
    let set_wallet_state: WriteSignal<WalletState> = expect_context();

    let load_store = move || -> Option<WalletStore> {
        let json = load_from_storage("wallet_store")?;
        serde_json::from_str(&json).ok()
    };

    let do_unlock = move || {
        let Some(s) = load_store() else {
            set_error_msg.set("No wallet data found".into());
            return;
        };

        match s.unlock_wallet(selected_index.get(), &password.get()) {
            Ok(wallet) => {
                set_wallet_state.set(WalletState {
                    is_unlocked: true,
                    wallet_name: wallet.name,
                    addresses: wallet.addresses,
                    active_chain: "ethereum".into(),
                    balances: std::collections::HashMap::new(),
                    balance_loading: false,
                });
                set_page.set(AppPage::Dashboard);
            }
            Err(_) => {
                set_error_msg.set("Wrong password".into());
            }
        }
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
            <h2 class="text-center mb-4">"Unlock Wallet"</h2>

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
                <label>"Password"</label>
                <input
                    type="password"
                    placeholder="Enter your password..."
                    prop:value=move || password.get()
                    on:input=move |ev| {
                        set_password.set(event_target_value(&ev));
                        set_error_msg.set(String::new());
                    }
                    on:keydown=unlock_enter
                />
            </div>

            {move || {
                let err = error_msg.get();
                if err.is_empty() { None } else {
                    Some(view! { <p style="color: var(--danger); margin: 8px 0;">{err}</p> })
                }
            }}

            <button class="btn btn-primary btn-block" on:click=unlock_click>
                "Unlock"
            </button>

            <button class="btn btn-secondary btn-block mt-4" on:click=go_to_create>
                "Create New Wallet"
            </button>
        </div>
    }
}
