// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;
use wallet_core::bip39_utils::{self, WordCount};
use wallet_core::wallet::{self, WalletStore, PasswordStrength, validate_password_strength};
use wallet_core::crypto;
use zeroize::Zeroize;

use crate::state::*;
use crate::i18n::t;
use crate::components::SPINNER_SVG;

/// Chain info for the selection UI
struct ChainInfo {
    id: &'static str,
    name: &'static str,
    icon_path: &'static str,
}

const AVAILABLE_CHAINS: &[ChainInfo] = &[
    ChainInfo { id: "ethereum", name: "Ethereum", icon_path: "chain-icons/ethereum.png" },
    ChainInfo { id: "polygon", name: "Polygon", icon_path: "chain-icons/polygon.png" },
    ChainInfo { id: "bsc", name: "BNB Chain", icon_path: "chain-icons/bsc.png" },
    ChainInfo { id: "arbitrum", name: "Arbitrum", icon_path: "chain-icons/arbitrum.png" },
    ChainInfo { id: "optimism", name: "Optimism", icon_path: "chain-icons/optimism.png" },
    ChainInfo { id: "base", name: "Base", icon_path: "chain-icons/base.png" },
    ChainInfo { id: "solana", name: "Solana", icon_path: "chain-icons/solana.png" },
    ChainInfo { id: "bitcoin", name: "Bitcoin", icon_path: "chain-icons/bitcoin.png" },
    ChainInfo { id: "ton", name: "TON", icon_path: "chain-icons/ton.png" },
    ChainInfo { id: "cosmos", name: "Cosmos Hub", icon_path: "chain-icons/cosmos.png" },
    ChainInfo { id: "osmosis", name: "Osmosis", icon_path: "chain-icons/osmosis.png" },
    ChainInfo { id: "litecoin", name: "Litecoin", icon_path: "chain-icons/litecoin.png" },
    ChainInfo { id: "stellar", name: "Stellar", icon_path: "chain-icons/stellar.png" },
    ChainInfo { id: "ripple", name: "XRP Ledger", icon_path: "chain-icons/ripple.png" },
    ChainInfo { id: "dogecoin", name: "Dogecoin", icon_path: "chain-icons/dogecoin.png" },
    ChainInfo { id: "tron", name: "TRON", icon_path: "chain-icons/tron.png" },
];

/// Default enabled chains (fast to derive)
const DEFAULT_CHAINS: &[&str] = &["ethereum"];

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
    let (loading, set_loading) = signal(false);
    let (show_password, set_show_password) = signal(false);
    let (show_confirm, set_show_confirm) = signal(false);
    let (loading_text, set_loading_text) = signal(String::new());
    let (enabled_chains, set_enabled_chains) = signal(
        DEFAULT_CHAINS.iter().map(|s| s.to_string()).collect::<Vec<String>>()
    );

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

    let proceed_to_chains = move |_| {
        if import_mode.get() {
            let phrase = import_phrase.get();
            if !bip39_utils::validate_mnemonic(&phrase) {
                set_error_msg.set(t("onboarding.invalid_mnemonic"));
                return;
            }
            set_mnemonic.set(phrase);
        }
        set_error_msg.set(String::new());
        set_step.set(2);
    };

    let proceed_to_password = move |_| {
        if enabled_chains.get().is_empty() {
            set_error_msg.set(t("onboarding.select_at_least_one"));
            return;
        }
        set_error_msg.set(String::new());
        set_step.set(3);
    };

    let toggle_chain = move |chain_id: String| {
        set_enabled_chains.update(|chains| {
            if chains.contains(&chain_id) {
                chains.retain(|c| c != &chain_id);
            } else {
                chains.push(chain_id);
            }
        });
    };

    let select_all = move |_| {
        set_enabled_chains.set(
            AVAILABLE_CHAINS.iter().map(|c| c.id.to_string()).collect()
        );
    };

    let select_none = move |_| {
        set_enabled_chains.set(Vec::new());
    };

    let create_wallet = move |_| {
        if loading.get() { return; }

        let pass = password.get();
        let confirm = confirm_password.get();

        let (strength, msg) = validate_password_strength(&pass);
        if strength == PasswordStrength::Weak {
            set_error_msg.set(msg.to_string());
            return;
        }
        if pass != confirm {
            set_error_msg.set(t("onboarding.passwords_mismatch"));
            return;
        }

        let name = wallet_name.get();
        let phrase = mnemonic.get();
        let chains = enabled_chains.get();

        // Three-phase creation to avoid blocking the main thread for >5s:
        // Phase 1 (Timeout 50ms): BIP39 mnemonic → seed (~0.5-2s)
        // Phase 2 (Timeout 0ms):  PBKDF2 encrypt seed (~1-3s)
        // Phase 3 (Timeout 0ms):  Derive chain addresses (~1-3s)
        // Each phase yields to the browser event loop, preventing "page not responding".
        set_loading.set(true);
        set_loading_text.set(t("loading.generating_seed"));
        set_error_msg.set(String::new());

        gloo_timers::callback::Timeout::new(50, move || {
            // Phase 1: BIP39 mnemonic → seed
            if !bip39_utils::validate_mnemonic(&phrase) {
                set_loading.set(false);
                set_error_msg.set("Invalid mnemonic".to_string());
                return;
            }
            let seed = match bip39_utils::mnemonic_to_seed(&phrase, "") {
                Ok(s) => s,
                Err(e) => {
                    set_loading.set(false);
                    set_error_msg.set(format!("Error: {}", e));
                    return;
                }
            };

            // Phase 2: PBKDF2 encrypt (deferred)
            set_loading_text.set(t("loading.pbkdf2_encrypt"));
            gloo_timers::callback::Timeout::new(0, move || {
                let encrypted = match crypto::encrypt(&seed, &pass) {
                    Ok(e) => e,
                    Err(e) => {
                        set_loading.set(false);
                        set_error_msg.set(format!("Error: {}", e));
                        return;
                    }
                };

                // Store immediately
                let mut store = WalletStore::new();
                store.store_encrypted(&name, encrypted);
                if let Ok(json) = serde_json::to_string(&store) {
                    save_to_storage("wallet_store", &json);
                }
                if let Ok(chains_json) = serde_json::to_string(&chains) {
                    save_to_storage("enabled_chains", &chains_json);
                }

                // Phase 3: derive addresses (deferred)
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
            }).forget();
        }).forget();
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
                <div class=move || step_class(3) />
            </div>

            {move || match step.get() {
                // Step 0: Welcome
                0 => view! {
                    <div class="text-center p-4">
                        <h2 class="mb-4">{move || t("onboarding.welcome")}</h2>
                        <p class="text-muted mb-4">{move || t("onboarding.subtitle")}</p>

                        <div class="input-group">
                            <label>{move || t("onboarding.wallet_name")}</label>
                            <input
                                type="text"
                                prop:value=move || wallet_name.get()
                                on:input=move |ev| set_wallet_name.set(event_target_value(&ev))
                            />
                        </div>

                        <button class="btn btn-primary btn-block mb-2" on:click=generate_seed>
                            {move || t("onboarding.create_new")}
                        </button>
                        <button class="btn btn-secondary btn-block" on:click=start_import>
                            {move || t("onboarding.import_existing")}
                        </button>
                    </div>
                }.into_any(),

                // Step 1: Seed / Import
                1 => view! {
                    <div class="p-4">
                        {move || {
                            if import_mode.get() {
                                view! {
                                    <div>
                                        <h2 class="mb-2">{move || t("onboarding.import_title")}</h2>
                                        <p class="text-sm text-muted mb-4">{move || t("onboarding.import_hint")}</p>
                                        <textarea
                                            placeholder=t("onboarding.import_placeholder")
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
                                        <h2 class="mb-2">{move || t("onboarding.recovery_title")}</h2>
                                        <p class="text-sm text-muted mb-4">{move || t("onboarding.recovery_warning")}</p>
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

                        <button class="btn btn-primary btn-block mt-4" on:click=proceed_to_chains>
                            {move || t("onboarding.continue")}
                        </button>
                        <button class="btn btn-secondary btn-block mt-2" on:click=move |_| set_step.set(0)>
                            {move || t("onboarding.back")}
                        </button>
                    </div>
                }.into_any(),

                // Step 2: Chain Selection
                2 => view! {
                    <div class="p-4">
                        <h2 class="mb-2">{move || t("onboarding.select_chains")}</h2>
                        <p class="text-sm text-muted mb-4">{move || t("onboarding.select_chains_hint")}</p>

                        <div style="display: flex; gap: 8px; margin-bottom: 12px;">
                            <button
                                class="btn btn-secondary"
                                style="flex: 1; padding: 6px; font-size: 13px;"
                                on:click=select_all
                            >
                                {move || t("onboarding.select_all")}
                            </button>
                            <button
                                class="btn btn-secondary"
                                style="flex: 1; padding: 6px; font-size: 13px;"
                                on:click=select_none
                            >
                                {move || t("onboarding.select_none")}
                            </button>
                        </div>

                        <div class="chain-list" style="max-height: 320px; overflow-y: auto;">
                            {AVAILABLE_CHAINS.iter().map(|chain| {
                                let chain_id = chain.id.to_string();
                                let chain_id2 = chain_id.clone();
                                let name = chain.name;
                                let icon_path = chain.icon_path;
                                view! {
                                    <div
                                        class="chain-item"
                                        style="cursor: pointer;"
                                        on:click=move |_| toggle_chain(chain_id.clone())
                                    >
                                        <div class="chain-icon">
                                            <img src=icon_path alt=name class="chain-icon-img" />
                                        </div>
                                        <div class="chain-info" style="flex: 1;">
                                            <div class="chain-name">{name}</div>
                                        </div>
                                        <div style="font-size: 20px;">
                                            {move || {
                                                if enabled_chains.get().contains(&chain_id2) {
                                                    "\u{2705}"
                                                } else {
                                                    "\u{2B1C}"
                                                }
                                            }}
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>

                        <p class="text-sm text-muted mt-2" style="text-align: center;">
                            {move || {
                                let count = enabled_chains.get().len();
                                format!("{} / {} chain", count, AVAILABLE_CHAINS.len())
                            }}
                        </p>

                        {error_view}

                        <button class="btn btn-primary btn-block mt-4" on:click=proceed_to_password>
                            {move || t("onboarding.continue")}
                        </button>
                        <button class="btn btn-secondary btn-block mt-2" on:click=move |_| set_step.set(1)>
                            {move || t("onboarding.back")}
                        </button>
                    </div>
                }.into_any(),

                // Step 3: Password
                _ => view! {
                    <div class="p-4">
                        <h2 class="mb-2">{move || t("onboarding.set_password")}</h2>
                        <p class="text-sm text-muted mb-4">{move || t("onboarding.password_hint")}</p>

                        <div class="input-group">
                            <label>{move || t("onboarding.password_label")}</label>
                            <div style="position: relative;">
                                <input
                                    type=move || if show_password.get() { "text" } else { "password" }
                                    prop:value=move || password.get()
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
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
                            let pass = password.get();
                            if pass.is_empty() {
                                return view! { <div /> }.into_any();
                            }
                            let (strength, msg) = validate_password_strength(&pass);
                            let (color, label) = match strength {
                                PasswordStrength::Weak => ("var(--danger)", t("onboarding.strength_weak")),
                                PasswordStrength::Fair => ("#f0ad4e", t("onboarding.strength_fair")),
                                PasswordStrength::Strong => ("var(--success)", t("onboarding.strength_strong")),
                            };
                            let bar_width = match strength {
                                PasswordStrength::Weak => "33%",
                                PasswordStrength::Fair => "66%",
                                PasswordStrength::Strong => "100%",
                            };
                            view! {
                                <div style="margin: 4px 0 12px 0;">
                                    <div style="height: 4px; background: var(--bg-secondary); border-radius: 2px; overflow: hidden;">
                                        <div style=format!("height: 100%; width: {}; background: {}; transition: all 0.3s;", bar_width, color) />
                                    </div>
                                    <p style=format!("color: {}; font-size: 12px; margin-top: 4px;", color)>
                                        {format!("{} — {}", label, msg)}
                                    </p>
                                </div>
                            }.into_any()
                        }}

                        <div class="input-group">
                            <label>{move || t("onboarding.confirm_password")}</label>
                            <div style="position: relative;">
                                <input
                                    type=move || if show_confirm.get() { "text" } else { "password" }
                                    prop:value=move || confirm_password.get()
                                    on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                                    style="padding-right: 40px;"
                                />
                                <button
                                    type="button"
                                    tabindex="-1"
                                    style="position: absolute; right: 8px; top: 50%; transform: translateY(-50%); background: none; border: none; color: var(--text-muted); cursor: pointer; font-size: 18px; padding: 4px;"
                                    on:click=move |_| set_show_confirm.set(!show_confirm.get())
                                >
                                    {move || if show_confirm.get() { "\u{1F648}" } else { "\u{1F441}" }}
                                </button>
                            </div>
                        </div>

                        {error_view}

                        <button
                            class="btn btn-primary btn-block"
                            on:click=create_wallet
                            disabled=move || loading.get()
                            style="display: flex; align-items: center; justify-content: center; gap: 8px;"
                        >
                            {move || loading.get().then(|| view! { <span inner_html=SPINNER_SVG /> })}
                            {move || if loading.get() {
                                loading_text.get()
                            } else {
                                t("onboarding.create_wallet")
                            }}
                        </button>
                        <button class="btn btn-secondary btn-block mt-2" on:click=move |_| set_step.set(2)
                            disabled=move || loading.get()
                        >
                            {move || t("onboarding.back")}
                        </button>

                        // Security badge
                        <div style="margin-top: 20px; padding: 14px 16px; background: var(--bg-secondary); border: 1px solid var(--border); border-radius: var(--radius); opacity: 0.9;">
                            <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 8px;">
                                <span style="font-size: 20px;">"\u{1F6E1}"</span>
                                <span style="font-weight: 600; font-size: 13px; color: var(--text-primary);">
                                    {move || t("security.title")}
                                </span>
                            </div>
                            <div style="display: flex; flex-direction: column; gap: 4px; font-size: 12px; color: var(--text-muted);">
                                <div style="display: flex; align-items: center; gap: 6px;">
                                    <span style="color: var(--success);">"\u{2705}"</span>
                                    {move || t("security.encryption")}
                                </div>
                                <div style="display: flex; align-items: center; gap: 6px;">
                                    <span style="color: var(--success);">"\u{2705}"</span>
                                    {move || t("security.key_derivation")}
                                </div>
                                <div style="display: flex; align-items: center; gap: 6px;">
                                    <span style="color: var(--success);">"\u{2705}"</span>
                                    {move || t("security.local_only")}
                                </div>
                            </div>
                        </div>
                    </div>
                }.into_any(),
            }}
        </div>
    }
}
