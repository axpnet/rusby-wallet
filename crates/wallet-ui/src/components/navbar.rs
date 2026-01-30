// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::AppPage;

#[component]
pub fn BottomNav() -> impl IntoView {
    let page: ReadSignal<AppPage> = expect_context();
    let set_page: WriteSignal<AppPage> = expect_context();

    let nav_items = vec![
        ("Home", AppPage::Dashboard),
        ("Send", AppPage::Send),
        ("Receive", AppPage::Receive),
        ("Settings", AppPage::Settings),
    ];

    view! {
        <nav class="bottom-nav">
            {nav_items.into_iter().map(|(label, target)| {
                let target_clone = target.clone();
                let is_active = move || page.get() == target_clone;
                view! {
                    <button
                        class="nav-item"
                        class:active=is_active
                        on:click=move |_| set_page.set(target.clone())
                    >
                        {label}
                    </button>
                }
            }).collect::<Vec<_>>()}
        </nav>
    }
}
