// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::AppPage;

#[component]
pub fn Sidebar() -> impl IntoView {
    let page: ReadSignal<AppPage> = expect_context();
    let set_page: WriteSignal<AppPage> = expect_context();

    let items = vec![
        ("🏠", "Home", AppPage::Dashboard),
        ("↗", "Send", AppPage::Send),
        ("↙", "Receive", AppPage::Receive),
        ("⚙", "Settings", AppPage::Settings),
    ];

    view! {
        <aside class="sidebar">
            {items.into_iter().map(|(icon, label, target)| {
                let target_clone = target.clone();
                let is_active = move || {
                    if page.get() == target_clone { "nav-item active" } else { "nav-item" }
                };
                view! {
                    <button
                        class=is_active
                        on:click=move |_| set_page.set(target.clone())
                    >
                        <span class="nav-icon">{icon}</span>
                        {label}
                    </button>
                }
            }).collect::<Vec<_>>()}
        </aside>
    }
}
