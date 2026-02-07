// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::i18n::t;

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
            set_error.set(t("confirm.enter_password"));
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
                <h3>{move || t("confirm.title")}</h3>

                <div class="card text-sm">
                    <div class="flex justify-between">
                        <span class="text-muted">{move || t("confirm.network")}</span>
                        <span>{chain}</span>
                    </div>
                    <div class="flex justify-between mt-2">
                        <span class="text-muted">{move || t("confirm.to")}</span>
                        <span style="font-family: monospace; font-size: 11px; max-width: 200px; overflow: hidden; text-overflow: ellipsis;">
                            {recipient}
                        </span>
                    </div>
                    <div class="flex justify-between mt-2">
                        <span class="text-muted">{move || t("confirm.amount")}</span>
                        <span>{amount.clone()} " " {ticker.clone()}</span>
                    </div>
                    <div class="flex justify-between mt-2">
                        <span class="text-muted">{move || t("confirm.est_fee")}</span>
                        <span>{fee} " " {ticker}</span>
                    </div>
                </div>

                <div class="input-group">
                    <label>{move || t("confirm.password_label")}</label>
                    <input
                        type="password"
                        placeholder={t("confirm.password_placeholder")}
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
                        {move || t("confirm.cancel")}
                    </button>
                    <button class="btn btn-primary" style="flex: 1;" on:click=confirm>
                        {move || t("confirm.confirm_sign")}
                    </button>
                </div>
            </div>
        </div>
    }
}
