// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;

#[component]
pub fn ReceivePage() -> impl IntoView {
    let wallet_state: ReadSignal<WalletState> = expect_context();
    let set_page: WriteSignal<AppPage> = expect_context();
    let (copied, set_copied) = signal(false);

    let address = move || wallet_state.get().current_address();

    let active_info = move || {
        let state = wallet_state.get();
        chain_list().into_iter()
            .find(|c| c.id == state.active_chain)
            .map(|c| (c.name, c.ticker))
            .unwrap_or(("Unknown".into(), "???".into()))
    };

    let copy_address = move |_| {
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
        <div class="p-4">
            <div class="flex items-center justify-between mb-4">
                <button class="btn btn-sm btn-secondary" on:click=move |_| set_page.set(AppPage::Dashboard)>
                    "< Back"
                </button>
                <h2>"Receive " {move || active_info().1}</h2>
                <div style="width: 60px;" />
            </div>

            <div class="qr-container">
                <div class="qr-code">
                    <p class="text-muted text-sm">"QR Code"</p>
                </div>

                <p class="text-sm text-muted">{move || active_info().0} " Address"</p>

                <div class="address-display">
                    <span>{address}</span>
                    <button class="copy-btn" on:click=copy_address>
                        {move || if copied.get() { "Copied!" } else { "Copy" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
