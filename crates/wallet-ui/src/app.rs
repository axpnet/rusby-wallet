// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::state::*;
use crate::i18n::{Locale, t};
use crate::theme::ThemeId;
use crate::pages::onboarding::Onboarding;
use crate::pages::login::Login;
use crate::pages::dashboard::Dashboard;
use crate::pages::send::SendPage;
use crate::pages::receive::ReceivePage;
use crate::pages::history::HistoryPage;
use crate::pages::approve::ApprovePage;
use crate::pages::walletconnect::WalletConnectPage;
use crate::pages::wc_proposal::WcProposalPage;
use crate::pages::approvals::ApprovalsPage;
use crate::pages::address_book::AddressBookPage;
use crate::pages::nft::NftPage;
use crate::pages::swap::SwapPage;
use crate::components::navbar::BottomNav;
use crate::components::sidebar::Sidebar;
use crate::components::top_nav::TopNav;
use crate::components::chain_sidebar::ChainSidebar;
use crate::components::toast::{ToastContainer, ToastMessage};

#[component]
pub fn App() -> impl IntoView {
    // Sync chrome.storage → localStorage on extension startup
    sync_chrome_storage_to_local();

    // Detect if running as extension popup or full-page
    let is_popup = is_extension_popup();
    let (fullpage, set_fullpage) = signal(!is_popup);

    // Global signals
    let (page, set_page) = signal({
        // Check if opened for dApp approval
        if get_url_param("approve").is_some() {
            AppPage::DappApproval
        } else if get_url_param("wc_proposal").is_some() {
            AppPage::WcProposal
        } else {
            initial_page()
        }
    });
    let (wallet_state, set_wallet_state) = signal(WalletState::default());
    let (theme, set_theme) = signal(
        ThemeId::from_code(&load_from_storage("theme").unwrap_or_else(|| "default".into()))
    );

    // Auto-lock settings (default OFF)
    let auto_lock_enabled = load_from_storage("auto_lock_enabled").map(|v| v == "true").unwrap_or(false);
    let auto_lock_timeout = load_from_storage("auto_lock_timeout").and_then(|v| v.parse::<u32>().ok()).unwrap_or(300);
    let (auto_lock_on, set_auto_lock_on) = signal(auto_lock_enabled);
    let (auto_lock_secs, set_auto_lock_secs) = signal(auto_lock_timeout);

    // i18n locale (default: detect browser or fallback to EN)
    let saved_locale = load_from_storage("locale")
        .map(|code| Locale::from_code(&code))
        .unwrap_or(Locale::En);
    let (locale, set_locale) = signal(saved_locale);

    // Testnet mode (default: OFF)
    let testnet_saved = load_from_storage("testnet_mode").map(|v| v == "true").unwrap_or(false);
    let (testnet_mode, set_testnet_mode) = signal(testnet_saved);

    // Toast notifications
    let (toasts, set_toasts) = signal::<Vec<ToastMessage>>(vec![]);

    // Provide context to all children
    provide_context(toasts);
    provide_context(set_toasts);
    provide_context(page);
    provide_context(set_page);
    provide_context(wallet_state);
    provide_context(set_wallet_state);
    provide_context(FullpageMode(fullpage));
    provide_context(auto_lock_on);
    provide_context(set_auto_lock_on);
    provide_context(auto_lock_secs);
    provide_context(set_auto_lock_secs);
    provide_context(locale);
    provide_context(set_locale);
    provide_context(testnet_mode);
    provide_context(set_testnet_mode);
    provide_context(theme);
    provide_context(set_theme);

    // Theme effect: apply CSS variables on change
    Effect::new(move |_| {
        let tid = theme.get();
        crate::theme::apply_theme(&tid);
        save_to_storage("theme", tid.code());
    });

    // Auto-lock: interval-based inactivity check.
    // Uses ONE-TIME closure setup to avoid leaking externref table entries.
    // Previous approach used Effect + Closure::forget() on each re-run,
    // which leaked closures and caused WASM "table grow failure".
    {
        use std::rc::Rc;
        use std::cell::Cell;

        let last_activity = Rc::new(Cell::new(js_sys::Date::now()));

        // One-time: track user activity via click/keydown
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            let la = last_activity.clone();
            let activity_cb = Closure::wrap(Box::new(move || {
                la.set(js_sys::Date::now());
            }) as Box<dyn Fn()>);
            let _ = doc.add_event_listener_with_callback("click", activity_cb.as_ref().unchecked_ref());
            let _ = doc.add_event_listener_with_callback("keydown", activity_cb.as_ref().unchecked_ref());
            activity_cb.forget(); // Leaked ONCE (app lifetime)
        }

        // One-time: check inactivity every 10 seconds
        let la = last_activity;
        let check_cb = Closure::wrap(Box::new(move || {
            let enabled = auto_lock_on.get_untracked();
            let is_unlocked = wallet_state.with_untracked(|ws| ws.is_unlocked);
            let timeout = auto_lock_secs.get_untracked();

            if !enabled || !is_unlocked { return; }

            let elapsed_secs = (js_sys::Date::now() - la.get()) / 1000.0;
            if elapsed_secs >= timeout as f64 {
                set_wallet_state.set(WalletState::default());
                set_page.set(AppPage::Login);
                la.set(js_sys::Date::now());
            }
        }) as Box<dyn Fn()>);

        if let Some(window) = web_sys::window() {
            let _ = window.set_interval_with_callback_and_timeout_and_arguments_0(
                check_cb.as_ref().unchecked_ref(), 10_000,
            );
        }
        check_cb.forget(); // Leaked ONCE (app lifetime)
    }

    // Fullpage body class effect
    Effect::new(move |_| {
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Some(body) = doc.body() {
                let class_list = body.class_list();
                if fullpage.get() {
                    let _ = class_list.add_1("fullpage");
                } else {
                    let _ = class_list.remove_1("fullpage");
                }
            }
        }
    });

    let toggle_theme = move |_| {
        set_theme.update(|t| {
            *t = if *t == ThemeId::Light { ThemeId::Default } else { ThemeId::Light };
        });
    };

    // Expand to full-page tab (like MetaMask/Rabby)
    let expand_to_tab = move |_| {
        if let Some(window) = web_sys::window() {
            // Get current URL and open in new tab
            if let Ok(href) = window.location().href() {
                let fullpage_url = if href.contains('?') {
                    format!("{}&fullpage=1", href)
                } else {
                    format!("{}?fullpage=1", href)
                };
                let _ = window.open_with_url_and_target(&fullpage_url, "_blank");
            }
        }
    };

    let container_class = move || {
        if fullpage.get() { "app-container fullpage" } else { "app-container" }
    };

    let show_nav = move || {
        let p = page.get();
        matches!(p, AppPage::Dashboard | AppPage::Send | AppPage::Receive | AppPage::History | AppPage::Settings)
    };

    // Notify background of lock state changes.
    // Use Memo to only react when is_unlocked actually changes,
    // not on every WalletState mutation (balances, nfts, etc.).
    let is_unlocked_memo = Memo::new(move |_| wallet_state.with(|ws| ws.is_unlocked));
    Effect::new(move |_| {
        let unlocked = is_unlocked_memo.get();
        wasm_bindgen_futures::spawn_local(async move {
            let data = serde_json::json!({"locked": !unlocked});
            let _ = send_to_background("__rusby_lock_state", &data).await;
        });
    });

    view! {
        <div class=container_class>
            <ToastContainer />
            <header class="header">
                <h1>"Rusby"
                    {move || {
                        if testnet_mode.get() {
                            Some(view! { <span style="font-size: 0.5em; color: var(--warning, #ff9800); margin-left: 6px; vertical-align: middle;">"TESTNET"</span> })
                        } else {
                            None
                        }
                    }}
                </h1>
                {move || {
                    if fullpage.get() && show_nav() {
                        Some(view! { <TopNav /> })
                    } else {
                        None
                    }
                }}
                <div class="header-actions">
                    {move || {
                        if !fullpage.get() {
                            Some(view! {
                                <button class="btn-icon" on:click=expand_to_tab title="Open in full page">
                                    "⛶"
                                </button>
                            })
                        } else {
                            None
                        }
                    }}
                    <button class="btn-icon" on:click=toggle_theme title="Toggle theme">
                        {move || if theme.get().is_dark() { "☀" } else { "🌙" }}
                    </button>
                </div>
            </header>

            {move || {
                if fullpage.get() && show_nav() {
                    // Full-page Talisman-like: chain sidebar + main content
                    Some(view! {
                        <div class="app-layout-fp">
                            <ChainSidebar />
                            <div class="main-content">
                                <div class="content">
                                    {move || render_page(page.get())}
                                </div>
                            </div>
                        </div>
                    }.into_any())
                } else {
                    // Popup: content + bottom nav
                    Some(view! {
                        <div>
                            <div class="content">
                                {move || render_page(page.get())}
                            </div>
                            {move || {
                                if show_nav() {
                                    Some(view! { <BottomNav /> })
                                } else {
                                    None
                                }
                            }}
                        </div>
                    }.into_any())
                }
            }}
        </div>
    }
}

fn render_page(page: AppPage) -> impl IntoView {
    match page {
        AppPage::Onboarding => view! { <Onboarding /> }.into_any(),
        AppPage::Login => view! { <Login /> }.into_any(),
        AppPage::Dashboard => view! { <Dashboard /> }.into_any(),
        AppPage::Send => view! { <SendPage /> }.into_any(),
        AppPage::Receive => view! { <ReceivePage /> }.into_any(),
        AppPage::History => view! { <HistoryPage /> }.into_any(),
        AppPage::Settings => view! { <SettingsPanel /> }.into_any(),
        AppPage::DappApproval => view! { <ApprovePage /> }.into_any(),
        AppPage::WalletConnect => view! { <WalletConnectPage /> }.into_any(),
        AppPage::WcProposal => view! { <WcProposalPage /> }.into_any(),
        AppPage::Approvals => view! { <ApprovalsPage /> }.into_any(),
        AppPage::AddressBook => view! { <AddressBookPage /> }.into_any(),
        AppPage::Nft => view! { <NftPage /> }.into_any(),
        AppPage::Swap => view! { <SwapPage /> }.into_any(),
    }
}

#[component]
fn SettingsPanel() -> impl IntoView {
    let set_page: WriteSignal<AppPage> = expect_context();
    let set_wallet_state: WriteSignal<WalletState> = expect_context();
    let auto_lock_on: ReadSignal<bool> = expect_context();
    let set_auto_lock_on: WriteSignal<bool> = expect_context();
    let auto_lock_secs: ReadSignal<u32> = expect_context();
    let set_auto_lock_secs: WriteSignal<u32> = expect_context();
    let set_locale: WriteSignal<Locale> = expect_context();
    let testnet_mode: ReadSignal<bool> = expect_context();
    let set_testnet_mode: WriteSignal<bool> = expect_context();

    // Backup state
    let (backup_status, set_backup_status) = signal(String::new());
    let (backup_password, set_backup_password) = signal(String::new());

    // Connected dApps
    let (connected_dapps, set_connected_dapps) = signal::<Vec<String>>(vec![]);

    // Fetch approved origins from background on mount
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let data = serde_json::json!({});
            if let Some(response) = send_to_background("__rusby_get_approved_origins", &data).await {
                if let Some(origins) = response.get("origins").and_then(|o| o.as_object()) {
                    let list: Vec<String> = origins.keys().cloned().collect();
                    set_connected_dapps.set(list);
                }
            }
        });
    });

    let logout = move |_| {
        set_wallet_state.set(WalletState::default());
        set_page.set(AppPage::Login);
    };

    let toggle_auto_lock = move |_| {
        let new_val = !auto_lock_on.get_untracked();
        set_auto_lock_on.set(new_val);
        save_to_storage("auto_lock_enabled", if new_val { "true" } else { "false" });
    };

    let on_timeout_change = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev);
        if let Ok(secs) = val.parse::<u32>() {
            set_auto_lock_secs.set(secs);
            save_to_storage("auto_lock_timeout", &secs.to_string());
        }
    };

    let on_locale_change = move |ev: leptos::ev::Event| {
        let code = event_target_value(&ev);
        let new_locale = Locale::from_code(&code);
        set_locale.set(new_locale);
        save_to_storage("locale", new_locale.code());
    };

    view! {
        <div>
            <h2 class="mb-4">{move || t("settings.title")}</h2>
            <div class="card">
                <p class="text-sm text-muted mb-2">{move || t("settings.language")}</p>
                <select
                    prop:value=move || { let l: ReadSignal<Locale> = expect_context(); l.get().code().to_string() }
                    on:change=on_locale_change
                    style="width: 100%; padding: 8px; border-radius: 8px; background: var(--bg-secondary); color: var(--text-primary); border: 1px solid var(--border);"
                >
                    {Locale::all().iter().map(|loc| {
                        let code = loc.code();
                        let name = loc.name();
                        view! { <option value=code>{name}</option> }
                    }).collect::<Vec<_>>()}
                </select>
            </div>
            <crate::theme::ThemeSelector />
            <div class="card">
                <p class="text-sm text-muted mb-2">{move || t("settings.wallet_version")}</p>
                <p>"v0.5.0 - Rusby (Rust + Leptos)"</p>
            </div>
            <div class="card">
                <p class="text-sm text-muted mb-2">{move || t("settings.security")}</p>
                <p>{move || t("settings.aes_encryption")}</p>
                <p class="text-sm text-muted">{move || t("settings.pbkdf2")}</p>
            </div>
            <div class="card">
                <p class="text-sm text-muted mb-2">{move || t("settings.auto_lock")}</p>
                <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 8px;">
                    <button
                        class="btn btn-secondary"
                        style="min-width: 60px;"
                        on:click=toggle_auto_lock
                    >
                        {move || if auto_lock_on.get() { t("settings.on") } else { t("settings.off") }}
                    </button>
                    <span class="text-sm">{move || if auto_lock_on.get() { t("settings.auto_lock_enabled") } else { t("settings.auto_lock_disabled") }}</span>
                </div>
                {move || {
                    if auto_lock_on.get() {
                        Some(view! {
                            <div>
                                <label class="text-sm">{move || t("settings.timeout")}</label>
                                <select
                                    prop:value=move || auto_lock_secs.get().to_string()
                                    on:change=on_timeout_change
                                    style="width: 100%; padding: 8px; border-radius: 8px; background: var(--bg-secondary); color: var(--text-primary); border: 1px solid var(--border);"
                                >
                                    <option value="60">{move || t("settings.1_minute")}</option>
                                    <option value="300">{move || t("settings.5_minutes")}</option>
                                    <option value="900">{move || t("settings.15_minutes")}</option>
                                    <option value="1800">{move || t("settings.30_minutes")}</option>
                                </select>
                            </div>
                        })
                    } else {
                        None
                    }
                }}
            </div>
            <div class="card">
                <p class="text-sm text-muted mb-2">{move || t("settings.connected_dapps")}</p>
                {move || {
                    let dapps = connected_dapps.get();
                    if dapps.is_empty() {
                        Some(view! { <p class="text-sm">{move || t("settings.no_dapps")}</p> }.into_any())
                    } else {
                        Some(view! {
                            <div>
                                {dapps.into_iter().map(|origin| {
                                    let origin_clone = origin.clone();
                                    let origin_display = origin.clone();
                                    view! {
                                        <div style="display: flex; justify-content: space-between; align-items: center; padding: 4px 0; border-bottom: 1px solid var(--border);">
                                            <span class="text-sm" style="word-break: break-all;">{origin_display}</span>
                                            <button
                                                class="btn btn-secondary"
                                                style="font-size: 0.7rem; padding: 2px 8px; min-width: auto;"
                                                on:click=move |_| {
                                                    let o = origin_clone.clone();
                                                    wasm_bindgen_futures::spawn_local(async move {
                                                        let data = serde_json::json!({"origin": o});
                                                        let _ = send_to_background("__rusby_revoke_origin", &data).await;
                                                    });
                                                    set_connected_dapps.update(|list| list.retain(|x| x != &origin));
                                                }
                                            >
                                                {move || t("settings.revoke")}
                                            </button>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any())
                    }
                }}
            </div>
            <div class="card">
                <p class="text-sm text-muted mb-2">{move || t("settings.token_approvals")}</p>
                <button
                    class="btn btn-primary btn-block"
                    on:click=move |_| set_page.set(AppPage::Approvals)
                >
                    {move || t("settings.manage_approvals")}
                </button>
            </div>
            <div class="card">
                <p class="text-sm text-muted mb-2">{move || t("settings.walletconnect")}</p>
                <button
                    class="btn btn-primary btn-block"
                    on:click=move |_| set_page.set(AppPage::WalletConnect)
                >
                    {move || t("settings.manage_wc")}
                </button>
            </div>
            // Testnet Toggle
            <div class="card">
                <p class="text-sm text-muted mb-2">{move || t("settings.testnet_toggle")}</p>
                <div style="display: flex; align-items: center; gap: 8px;">
                    <button
                        class="btn btn-secondary"
                        style="min-width: 100px;"
                        on:click=move |_| {
                            let new_val = !testnet_mode.get_untracked();
                            set_testnet_mode.set(new_val);
                            save_to_storage("testnet_mode", if new_val { "true" } else { "false" });
                        }
                    >
                        {move || if testnet_mode.get() { t("settings.testnet_on") } else { t("settings.testnet_off") }}
                    </button>
                    {move || {
                        if testnet_mode.get() {
                            Some(view! { <span class="text-sm" style="color: var(--warning, #ff9800);">{t("settings.testnet_badge")}</span> })
                        } else {
                            None
                        }
                    }}
                </div>
            </div>
            // Address Book
            <div class="card">
                <p class="text-sm text-muted mb-2">{move || t("addressbook.title")}</p>
                <button
                    class="btn btn-primary btn-block"
                    on:click=move |_| set_page.set(AppPage::AddressBook)
                >
                    {move || t("addressbook.title")}
                </button>
            </div>
            // API Keys (NFT + Swap)
            <div class="card">
                <p class="text-sm text-muted mb-2">{move || t("settings.api_keys")}</p>
                <p class="text-sm text-muted" style="margin-bottom: 8px;">{move || t("settings.api_keys_hint")}</p>
                <div style="margin-bottom: 8px;">
                    <label class="text-sm">{move || t("settings.alchemy_key")}</label>
                    <input
                        type="text"
                        placeholder="Alchemy API Key"
                        prop:value=move || load_from_storage("alchemy_api_key").unwrap_or_default()
                        on:change=move |ev| {
                            save_to_storage("alchemy_api_key", &event_target_value(&ev));
                        }
                        style="width: 100%; padding: 8px; border-radius: 8px; background: var(--bg-input); color: var(--text-primary); border: 1px solid var(--border); font-size: 12px; margin-top: 4px;"
                    />
                </div>
                <div style="margin-bottom: 8px;">
                    <label class="text-sm">{move || t("settings.helius_key")}</label>
                    <input
                        type="text"
                        placeholder="Helius API Key"
                        prop:value=move || load_from_storage("helius_api_key").unwrap_or_default()
                        on:change=move |ev| {
                            save_to_storage("helius_api_key", &event_target_value(&ev));
                        }
                        style="width: 100%; padding: 8px; border-radius: 8px; background: var(--bg-input); color: var(--text-primary); border: 1px solid var(--border); font-size: 12px; margin-top: 4px;"
                    />
                </div>
                <div>
                    <label class="text-sm">{move || t("settings.zeroex_key")}</label>
                    <input
                        type="text"
                        placeholder="0x API Key"
                        prop:value=move || load_from_storage("zeroex_api_key").unwrap_or_default()
                        on:change=move |ev| {
                            save_to_storage("zeroex_api_key", &event_target_value(&ev));
                        }
                        style="width: 100%; padding: 8px; border-radius: 8px; background: var(--bg-input); color: var(--text-primary); border: 1px solid var(--border); font-size: 12px; margin-top: 4px;"
                    />
                </div>
            </div>
            // Export/Import Backup
            <div class="card">
                <p class="text-sm text-muted mb-2">{move || t("settings.export_backup")}{" / "}{move || t("settings.import_backup")}</p>
                <div style="margin-bottom: 8px;">
                    <input
                        type="password"
                        placeholder={move || t("backup.password_placeholder")}
                        prop:value=move || backup_password.get()
                        on:input=move |ev| {
                            set_backup_password.set(event_target_value(&ev));
                        }
                        style="width: 100%; padding: 8px; border-radius: 8px; background: var(--bg-input); color: var(--text-primary); border: 1px solid var(--border); font-size: 12px;"
                    />
                </div>
                <div class="flex gap-2 mb-2">
                    <button
                        class="btn btn-primary flex-1"
                        on:click=move |_| {
                            let pwd = backup_password.get();
                            if pwd.is_empty() {
                                set_backup_status.set(t("backup.password_required"));
                                return;
                            }
                            let wallet_json = load_from_storage("wallet_store").unwrap_or_default();
                            if wallet_json.is_empty() {
                                set_backup_status.set(t("backup.invalid_file"));
                                return;
                            }
                            set_backup_status.set(t("backup.exporting"));
                            match wallet_core::backup::export_backup(&wallet_json, &pwd) {
                                Ok(backup_json) => {
                                    trigger_download("rusby-backup.rusby", &backup_json);
                                    set_backup_status.set(t("backup.success_export"));
                                    set_backup_password.set(String::new());
                                }
                                Err(e) => {
                                    set_backup_status.set(format!("{} {}", t("common.error"), e));
                                }
                            }
                        }
                    >
                        {move || t("settings.export_backup")}
                    </button>
                    <button
                        class="btn btn-secondary flex-1"
                        on:click=move |_| {
                            let pwd = backup_password.get();
                            if pwd.is_empty() {
                                set_backup_status.set(t("backup.password_required"));
                                return;
                            }
                            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                                if let Some(el) = doc.get_element_by_id("backup-file-input") {
                                    if let Some(input) = el.dyn_ref::<web_sys::HtmlInputElement>() {
                                        input.click();
                                    }
                                }
                            }
                        }
                    >
                        {move || t("settings.import_backup")}
                    </button>
                </div>
                <input
                    type="file"
                    id="backup-file-input"
                    accept=".rusby"
                    style="display: none;"
                    on:change=move |ev| {
                        let target = event_target::<web_sys::HtmlInputElement>(&ev);
                        if let Some(files) = target.files() {
                            if let Some(file) = files.get(0) {
                                let pwd = backup_password.get();
                                set_backup_status.set(t("backup.importing"));
                                let reader = web_sys::FileReader::new().unwrap();
                                let reader_clone = reader.clone();
                                let onload = Closure::wrap(Box::new(move || {
                                    if let Ok(result) = reader_clone.result() {
                                        if let Some(text) = result.as_string() {
                                            match wallet_core::backup::import_backup(&text, &pwd) {
                                                Ok(wallet_json) => {
                                                    save_to_storage("wallet_store", &wallet_json);
                                                    set_backup_status.set(t("backup.success_import"));
                                                    set_page.set(AppPage::Login);
                                                }
                                                Err(e) => {
                                                    set_backup_status.set(format!("{} {}", t("common.error"), e));
                                                }
                                            }
                                        }
                                    }
                                }) as Box<dyn Fn()>);
                                reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                                onload.forget();
                                let _ = reader.read_as_text(&file);
                            }
                        }
                    }
                />
                {move || {
                    let status = backup_status.get();
                    if status.is_empty() {
                        None
                    } else {
                        Some(view! { <p class="text-sm mt-2">{status}</p> })
                    }
                }}
            </div>
            <button class="btn btn-danger btn-block mt-4" on:click=logout>
                {move || t("settings.lock_wallet")}
            </button>
        </div>
    }
}

fn initial_page() -> AppPage {
    if load_from_storage("wallet_store").is_some() {
        AppPage::Login
    } else {
        AppPage::Onboarding
    }
}

/// Trigger a file download in the browser
fn trigger_download(filename: &str, content: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            if let Ok(a) = doc.create_element("a") {
                let href = format!(
                    "data:application/json;charset=utf-8,{}",
                    js_sys::encode_uri_component(content)
                );
                let _ = a.set_attribute("href", &href);
                let _ = a.set_attribute("download", filename);
                let _ = a.set_attribute("style", "display:none");
                if let Some(body) = doc.body() {
                    let _ = body.append_child(&a);
                    if let Some(el) = a.dyn_ref::<web_sys::HtmlElement>() {
                        el.click();
                    }
                    let _ = body.remove_child(&a);
                }
            }
        }
    }
}

/// Detect if running inside an extension popup (small viewport or chrome.extension context)
fn is_extension_popup() -> bool {
    if let Some(window) = web_sys::window() {
        // Check URL param
        if let Ok(href) = window.location().href() {
            if href.contains("fullpage=1") {
                return false;
            }
        }
        // Check viewport width — popup is typically 400px
        if let Ok(width) = js_sys::Reflect::get(&window, &"innerWidth".into()) {
            if let Some(w) = width.as_f64() {
                return w <= 500.0;
            }
        }
    }
    false
}
