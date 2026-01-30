// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;
use wallet_core::bip39_utils::{self, WordCount};
use wallet_core::wallet::WalletStore;

use crate::state::*;

#[component]
pub fn Onboarding() -> impl IntoView {
    let (step, set_step) = signal(0u8);
    let (mnemonic, set_mnemonic) = signal(String::new());
    let (wallet_name, set_wallet_name) = signal("My Wallet".to_string());
    let (password, set_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (import_mode, set_import_mode) = signal(false);
    let (import_phrase, set_import_phrase) = signal(String::new());
    let (error_msg, set_error_msg) = signal(String::new());

    let set_page: WriteSignal<AppPage> = expect_context();
    let set_wallet_state: WriteSignal<WalletState> = expect_context();

    let generate_seed = move |_| {
        let phrase = bip39_utils::generate_mnemonic(WordCount::W12);
        set_mnemonic.set(phrase);
        set_import_mode.set(false);
        set_step.set(1);
    };

    let start_import = move |_| {
        set_import_mode.set(true);
        set_step.set(1);
    };

    let proceed_to_password = move |_| {
        if import_mode.get() {
            let phrase = import_phrase.get();
            if !bip39_utils::validate_mnemonic(&phrase) {
                set_error_msg.set("Invalid mnemonic phrase".into());
                return;
            }
            set_mnemonic.set(phrase);
        }
        set_error_msg.set(String::new());
        set_step.set(2);
    };

    let create_wallet = move |_| {
        let pass = password.get();
        let confirm = confirm_password.get();

        if pass.len() < 8 {
            set_error_msg.set("Password must be at least 8 characters".into());
            return;
        }
        if pass != confirm {
            set_error_msg.set("Passwords do not match".into());
            return;
        }

        let name = wallet_name.get();
        let phrase = mnemonic.get();

        let mut store = WalletStore::new();
        match store.create_wallet(&name, &phrase, &pass) {
            Ok(wallet) => {
                if let Ok(json) = serde_json::to_string(&store) {
                    save_to_storage("wallet_store", &json);
                }

                set_wallet_state.set(WalletState {
                    is_unlocked: true,
                    wallet_name: wallet.name,
                    addresses: wallet.addresses,
                    active_chain: "ethereum".into(),
                    balances: std::collections::HashMap::new(),
                });

                set_page.set(AppPage::Dashboard);
            }
            Err(e) => {
                set_error_msg.set(format!("Error: {}", e));
            }
        }
    };

    let step_class = move |s: u8| {
        let current = step.get();
        if current == s { "step-dot active" }
        else if current > s { "step-dot completed" }
        else { "step-dot" }
    };

    let error_view = move || {
        let err = error_msg.get();
        if err.is_empty() { None } else {
            Some(view! { <p style="color: var(--danger); margin: 8px 0;">{err}</p> })
        }
    };

    view! {
        <div>
            <div class="wizard-steps">
                <div class=move || step_class(0) />
                <div class=move || step_class(1) />
                <div class=move || step_class(2) />
            </div>

            {move || match step.get() {
                0 => view! {
                    <div class="text-center p-4">
                        <h2 class="mb-4">"Welcome to Rusby"</h2>
                        <p class="text-muted mb-4">"Secure multi-chain wallet powered by Rust"</p>

                        <div class="input-group">
                            <label>"Wallet Name"</label>
                            <input
                                type="text"
                                prop:value=move || wallet_name.get()
                                on:input=move |ev| set_wallet_name.set(event_target_value(&ev))
                            />
                        </div>

                        <button class="btn btn-primary btn-block mb-2" on:click=generate_seed>
                            "Create New Wallet"
                        </button>
                        <button class="btn btn-secondary btn-block" on:click=start_import>
                            "Import Existing Wallet"
                        </button>
                    </div>
                }.into_any(),

                1 => view! {
                    <div class="p-4">
                        {move || {
                            if import_mode.get() {
                                view! {
                                    <div>
                                        <h2 class="mb-2">"Import Wallet"</h2>
                                        <p class="text-sm text-muted mb-4">"Enter your 12 or 24 word recovery phrase"</p>
                                        <textarea
                                            placeholder="Enter your mnemonic phrase..."
                                            prop:value=move || import_phrase.get()
                                            on:input=move |ev| set_import_phrase.set(event_target_value(&ev))
                                        />
                                    </div>
                                }.into_any()
                            } else {
                                let words: Vec<(usize, String)> = mnemonic.get()
                                    .split_whitespace()
                                    .enumerate()
                                    .map(|(i, w)| (i + 1, w.to_string()))
                                    .collect();
                                view! {
                                    <div>
                                        <h2 class="mb-2">"Your Recovery Phrase"</h2>
                                        <p class="text-sm text-muted mb-4">"Write down these words in order. Never share them."</p>
                                        <div class="seed-grid">
                                            {words.into_iter().map(|(idx, word)| {
                                                view! {
                                                    <div class="seed-word">
                                                        <span class="index">{idx.to_string()}</span>
                                                        " " {word}
                                                    </div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }}

                        {error_view}

                        <button class="btn btn-primary btn-block mt-4" on:click=proceed_to_password>
                            "Continue"
                        </button>
                        <button class="btn btn-secondary btn-block mt-2" on:click=move |_| set_step.set(0)>
                            "Back"
                        </button>
                    </div>
                }.into_any(),

                _ => view! {
                    <div class="p-4">
                        <h2 class="mb-2">"Set Password"</h2>
                        <p class="text-sm text-muted mb-4">"This password encrypts your wallet locally"</p>

                        <div class="input-group">
                            <label>"Password (min 8 characters)"</label>
                            <input
                                type="password"
                                prop:value=move || password.get()
                                on:input=move |ev| set_password.set(event_target_value(&ev))
                            />
                        </div>

                        <div class="input-group">
                            <label>"Confirm Password"</label>
                            <input
                                type="password"
                                prop:value=move || confirm_password.get()
                                on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                            />
                        </div>

                        {error_view}

                        <button class="btn btn-primary btn-block" on:click=create_wallet>
                            "Create Wallet"
                        </button>
                        <button class="btn btn-secondary btn-block mt-2" on:click=move |_| set_step.set(1)>
                            "Back"
                        </button>
                    </div>
                }.into_any(),
            }}
        </div>
    }
}
