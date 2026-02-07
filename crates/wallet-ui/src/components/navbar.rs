// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::AppPage;
use crate::i18n::t;

#[component]
pub fn BottomNav() -> impl IntoView {
    let page: ReadSignal<AppPage> = expect_context();
    let set_page: WriteSignal<AppPage> = expect_context();

    let nav_items: Vec<(&str, AppPage)> = vec![
        ("nav.home", AppPage::Dashboard),
        ("nav.send", AppPage::Send),
        ("nav.receive", AppPage::Receive),
        ("nav.history", AppPage::History),
        ("nav.settings", AppPage::Settings),
    ];

    view! {
        <nav class="bottom-nav">
            {nav_items.into_iter().map(|(key, target)| {
                let target_clone = target.clone();
                let key = key.to_string();
                let is_active = move || page.get() == target_clone;
                view! {
                    <button
                        class="nav-item"
                        class:active=is_active
                        on:click=move |_| set_page.set(target.clone())
                    >
                        {let k = key.clone(); move || t(&k)}
                    </button>
                }
            }).collect::<Vec<_>>()}
        </nav>
    }
}
