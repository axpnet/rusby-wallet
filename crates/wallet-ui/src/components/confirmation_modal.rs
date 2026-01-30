// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

#[component]
pub fn ConfirmationModal(
    recipient: String,
    amount: String,
    fee: String,
    chain: String,
    ticker: String,
    on_confirm: Callback<String>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal(String::new());

    let confirm = move |_| {
        let pass = password.get();
        if pass.is_empty() {
            set_error.set("Enter your password".into());
            return;
        }
        on_confirm.run(pass);
    };

    let cancel = move |_| {
        on_cancel.run(());
    };

    view! {
        <div class="modal-overlay">
            <div class="modal-content">
                <h3>"Confirm Transaction"</h3>

                <div class="card text-sm">
                    <div class="flex justify-between">
                        <span class="text-muted">"Network"</span>
                        <span>{chain}</span>
                    </div>
                    <div class="flex justify-between mt-2">
                        <span class="text-muted">"To"</span>
                        <span style="font-family: monospace; font-size: 11px; max-width: 200px; overflow: hidden; text-overflow: ellipsis;">
                            {recipient}
                        </span>
                    </div>
                    <div class="flex justify-between mt-2">
                        <span class="text-muted">"Amount"</span>
                        <span>{amount.clone()} " " {ticker.clone()}</span>
                    </div>
                    <div class="flex justify-between mt-2">
                        <span class="text-muted">"Est. Fee"</span>
                        <span>{fee} " " {ticker}</span>
                    </div>
                </div>

                <div class="input-group">
                    <label>"Password (to sign)"</label>
                    <input
                        type="password"
                        placeholder="Enter wallet password"
                        prop:value=move || password.get()
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                    />
                </div>

                {move || {
                    let e = error.get();
                    if e.is_empty() { None } else {
                        Some(view! { <p class="text-sm" style="color: var(--danger);">{e}</p> })
                    }
                }}

                <div class="flex gap-2 mt-4">
                    <button class="btn btn-secondary" style="flex: 1;" on:click=cancel>
                        "Cancel"
                    </button>
                    <button class="btn btn-primary" style="flex: 1;" on:click=confirm>
                        "Confirm & Sign"
                    </button>
                </div>
            </div>
        </div>
    }
}
