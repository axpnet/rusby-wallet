// Rusby Wallet — Security warning component
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
}

impl Severity {
    pub fn color(&self) -> &'static str {
        match self {
            Severity::Low => "#f39c12",      // yellow
            Severity::Medium => "#e67e22",   // orange
            Severity::High => "#e74c3c",     // red
        }
    }

    pub fn bg_color(&self) -> &'static str {
        match self {
            Severity::Low => "rgba(243, 156, 18, 0.1)",
            Severity::Medium => "rgba(230, 126, 34, 0.1)",
            Severity::High => "rgba(231, 76, 60, 0.15)",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Severity::Low => "i",
            Severity::Medium => "!",
            Severity::High => "!!",
        }
    }
}

#[component]
pub fn SecurityWarning(
    severity: Severity,
    title: String,
    message: String,
    #[prop(default = false)]
    dismissable: bool,
    #[prop(optional)]
    on_dismiss: Option<Callback<()>>,
) -> impl IntoView {
    let (dismissed, set_dismissed) = signal(false);

    let border_color = severity.color().to_string();
    let bg = severity.bg_color().to_string();
    let icon = severity.icon().to_string();
    let color = severity.color().to_string();

    view! {
        {move || {
            if dismissed.get() { return None; }
            let style = format!(
                "border: 1px solid {}; background: {}; border-radius: 8px; padding: 12px; margin: 8px 0;",
                border_color, bg
            );
            Some(view! {
                <div style=style>
                    <div style="display: flex; align-items: flex-start; gap: 8px;">
                        <span style=format!("color: {}; font-weight: bold; font-size: 1.1rem; min-width: 24px; text-align: center;", color)>
                            {icon.clone()}
                        </span>
                        <div style="flex: 1;">
                            <p style=format!("color: {}; font-weight: bold; font-size: 0.9rem; margin: 0;", color)>
                                {title.clone()}
                            </p>
                            <p style="font-size: 0.8rem; margin: 4px 0 0 0; opacity: 0.9;">
                                {message.clone()}
                            </p>
                        </div>
                        {if dismissable {
                            let on_dismiss = on_dismiss.clone();
                            Some(view! {
                                <button
                                    style="background: none; border: none; cursor: pointer; color: var(--text-muted); font-size: 1rem; padding: 0;"
                                    on:click=move |_| {
                                        set_dismissed.set(true);
                                        if let Some(cb) = &on_dismiss {
                                            cb.run(());
                                        }
                                    }
                                >
                                    "x"
                                </button>
                            })
                        } else {
                            None
                        }}
                    </div>
                </div>
            })
        }}
    }
}
