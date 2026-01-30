// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;

#[component]
pub fn SendPage() -> impl IntoView {
    let wallet_state: ReadSignal<WalletState> = expect_context();
    let set_page: WriteSignal<AppPage> = expect_context();

    let (recipient, set_recipient) = signal(String::new());
    let (amount, set_amount) = signal(String::new());
    let (status, set_status) = signal(String::new());

    let active_ticker = move || {
        let state = wallet_state.get();
        chain_list().into_iter()
            .find(|c| c.id == state.active_chain)
            .map(|c| c.ticker)
            .unwrap_or("???".into())
    };

    let send_tx = move |_| {
        let to = recipient.get();
        let amt = amount.get();

        if to.is_empty() {
            set_status.set("Enter recipient address".into());
            return;
        }
        if amt.is_empty() || amt.parse::<f64>().is_err() {
            set_status.set("Enter a valid amount".into());
            return;
        }

        // TODO: Real transaction signing and broadcasting
        set_status.set(format!("Transaction signing not yet implemented. Would send {} {} to {}", amt, active_ticker(), to));
    };

    view! {
        <div class="p-4">
            <div class="flex items-center justify-between mb-4">
                <button class="btn btn-sm btn-secondary" on:click=move |_| set_page.set(AppPage::Dashboard)>
                    "< Back"
                </button>
                <h2>"Send " {active_ticker}</h2>
                <div style="width: 60px;" />
            </div>

            <div class="input-group">
                <label>"Recipient Address"</label>
                <input
                    type="text"
                    placeholder="0x... or address"
                    prop:value=move || recipient.get()
                    on:input=move |ev| set_recipient.set(event_target_value(&ev))
                />
            </div>

            <div class="input-group">
                <label>"Amount"</label>
                <input
                    type="text"
                    placeholder="0.0"
                    prop:value=move || amount.get()
                    on:input=move |ev| set_amount.set(event_target_value(&ev))
                />
            </div>

            <div class="card text-sm">
                <div class="flex justify-between">
                    <span class="text-muted">"From"</span>
                    <span style="font-family: monospace; font-size: 12px;">
                        {move || {
                            let addr = wallet_state.get().current_address();
                            if addr.len() > 16 {
                                format!("{}...{}", &addr[..8], &addr[addr.len()-6..])
                            } else {
                                addr
                            }
                        }}
                    </span>
                </div>
                <div class="flex justify-between mt-2">
                    <span class="text-muted">"Network"</span>
                    <span>{move || wallet_state.get().active_chain.clone()}</span>
                </div>
            </div>

            {move || {
                let s = status.get();
                if s.is_empty() { None } else {
                    Some(view! { <p class="text-sm mt-2" style="color: var(--warning);">{s}</p> })
                }
            }}

            <button class="btn btn-primary btn-block mt-4" on:click=send_tx>
                "Send Transaction"
            </button>
        </div>
    }
}
