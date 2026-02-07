// Rusby Wallet — Toast notification component
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

/// Toast message type
#[derive(Debug, Clone, PartialEq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

/// A single toast notification
#[derive(Debug, Clone)]
pub struct ToastMessage {
    pub id: u32,
    pub message: String,
    pub toast_type: ToastType,
}

static TOAST_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Show a toast notification (adds to context signal)
pub fn show_toast(set_toasts: WriteSignal<Vec<ToastMessage>>, message: String, toast_type: ToastType) {
    let id = TOAST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let msg = ToastMessage { id, message, toast_type };

    set_toasts.update(|toasts| {
        toasts.push(msg);
    });

    // Auto-dismiss after 5 seconds
    let dismiss_id = id;
    gloo_timers::callback::Timeout::new(5_000, move || {
        set_toasts.update(|toasts| {
            toasts.retain(|t| t.id != dismiss_id);
        });
    }).forget();
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
