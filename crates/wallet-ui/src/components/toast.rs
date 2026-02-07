// Rusby Wallet — Toast notification component
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

/// Toast message type — variants used by ToastContainer for CSS class mapping
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

/// A single toast notification
#[derive(Debug, Clone)]
pub struct ToastMessage {
    pub message: String,
    pub toast_type: ToastType,
}

/// Toast container component — renders all active toasts
#[component]
pub fn ToastContainer() -> impl IntoView {
    let toasts: ReadSignal<Vec<ToastMessage>> = expect_context();

    view! {
        <div class="toast-container">
            {move || {
                toasts.get().into_iter().map(|toast| {
                    let class = match toast.toast_type {
                        ToastType::Success => "toast toast-success",
                        ToastType::Error => "toast toast-error",
                        ToastType::Warning => "toast toast-warning",
                        ToastType::Info => "toast toast-info",
                    };
                    view! {
                        <div class=class>
                            {toast.message}
                        </div>
                    }
                }).collect::<Vec<_>>()
            }}
        </div>
    }
}
