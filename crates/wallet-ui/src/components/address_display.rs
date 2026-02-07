// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;
use crate::i18n::t;

#[component]
pub fn AddressDisplay() -> impl IntoView {
    let wallet_state: ReadSignal<WalletState> = expect_context();
    let (copied, set_copied) = signal(false);

    let address = move || wallet_state.with(|s| s.current_address());

    let short_address = move || {
        let addr = address();
        if addr.len() > 16 {
            format!("{}...{}", &addr[..8], &addr[addr.len()-6..])
        } else {
            addr
        }
    };

    let copy = move |_| {
        let addr = address();
        if let Some(window) = web_sys::window() {
            let clipboard = window.navigator().clipboard();
            let _ = clipboard.write_text(&addr);
            set_copied.set(true);
            gloo_timers::callback::Timeout::new(2000, move || {
                set_copied.set(false);
            }).forget();
        }
    };

    view! {
        <div class="address-display">
            <span>{short_address}</span>
            <button class="copy-btn" on:click=copy>
                {move || if copied.get() { t("common.copied") } else { t("common.copy") }}
            </button>
        </div>
    }
}
