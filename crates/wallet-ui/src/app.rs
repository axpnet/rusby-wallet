// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;
use crate::pages::onboarding::Onboarding;
use crate::pages::login::Login;
use crate::pages::dashboard::Dashboard;
use crate::pages::send::SendPage;
use crate::pages::receive::ReceivePage;
use crate::components::navbar::BottomNav;
use crate::components::sidebar::Sidebar;

#[component]
pub fn App() -> impl IntoView {
    // Sync chrome.storage → localStorage on extension startup
    sync_chrome_storage_to_local();

    // Detect if running as extension popup or full-page
    let is_popup = is_extension_popup();
    let (fullpage, set_fullpage) = signal(!is_popup);

    // Global signals
    let (page, set_page) = signal(initial_page());
    let (wallet_state, set_wallet_state) = signal(WalletState::default());
    let (theme, set_theme) = signal(load_from_storage("theme").unwrap_or_else(|| "dark".into()));

    // Provide context to all children
    provide_context(page);
    provide_context(set_page);
    provide_context(wallet_state);
    provide_context(set_wallet_state);
    provide_context(fullpage);

    // Theme effect
    Effect::new(move |_| {
        let t = theme.get();
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Some(el) = doc.document_element() {
                let _ = el.set_attribute("data-theme", &t);
            }
        }
        save_to_storage("theme", &t);
    });

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
            *t = if t == "dark" { "light".to_string() } else { "dark".to_string() };
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
        matches!(p, AppPage::Dashboard | AppPage::Send | AppPage::Receive | AppPage::Settings)
    };

    view! {
        <div class=container_class>
            <header class="header">
                <h1>"Rusby"</h1>
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
                        {move || if theme.get() == "dark" { "☀" } else { "🌙" }}
                    </button>
                </div>
            </header>

            {move || {
                if fullpage.get() && show_nav() {
                    // Full-page: sidebar + main content
                    Some(view! {
                        <div class="app-layout">
                            <Sidebar />
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
        AppPage::Settings => view! { <SettingsPanel /> }.into_any(),
    }
}

#[component]
fn SettingsPanel() -> impl IntoView {
    let set_page: WriteSignal<AppPage> = expect_context();
    let set_wallet_state: WriteSignal<WalletState> = expect_context();

    let logout = move |_| {
        set_wallet_state.set(WalletState::default());
        set_page.set(AppPage::Login);
    };

    view! {
        <div>
            <h2 class="mb-4">"Settings"</h2>
            <div class="card">
                <p class="text-sm text-muted mb-2">"Wallet Version"</p>
                <p>"v0.0.1 - Rusby (Rust + Leptos)"</p>
            </div>
            <div class="card">
                <p class="text-sm text-muted mb-2">"Security"</p>
                <p>"AES-256-GCM encryption"</p>
                <p class="text-sm text-muted">"PBKDF2 100k iterations"</p>
            </div>
            <button class="btn btn-danger btn-block mt-4" on:click=logout>
                "Lock Wallet"
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
