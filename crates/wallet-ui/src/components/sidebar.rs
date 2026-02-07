// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::AppPage;
use crate::i18n::t;

#[component]
pub fn Sidebar() -> impl IntoView {
    let page: ReadSignal<AppPage> = expect_context();
    let set_page: WriteSignal<AppPage> = expect_context();

    let items: Vec<(&str, &str, AppPage)> = vec![
        ("\u{1f3e0}", "nav.home", AppPage::Dashboard),
        ("\u{2197}", "nav.send", AppPage::Send),
        ("\u{2199}", "nav.receive", AppPage::Receive),
        ("\u{2630}", "nav.history", AppPage::History),
        ("\u{2699}", "nav.settings", AppPage::Settings),
    ];

    view! {
        <aside class="sidebar">
            {items.into_iter().map(|(icon, key, target)| {
                let target_clone = target.clone();
                let key = key.to_string();
                let is_active = move || {
                    if page.get() == target_clone { "nav-item active" } else { "nav-item" }
                };
                view! {
                    <button
                        class=is_active
                        on:click=move |_| set_page.set(target.clone())
                    >
                        <span class="nav-icon">{icon}</span>
                        {let k = key.clone(); move || t(&k)}
                    </button>
                }
            }).collect::<Vec<_>>()}
        </aside>
    }
}
