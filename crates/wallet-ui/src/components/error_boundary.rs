// Rusby Wallet — Error boundary component + WASM panic hook
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

/// Install a custom panic hook that shows errors in the DOM
/// instead of silently crashing the WASM module
pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Errore sconosciuto".to_string()
        };

        let location = info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_default();

        // Log to console
        web_sys::console::error_1(&format!("PANIC: {} at {}", msg, location).into());

        // Show error overlay in the DOM
        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document() {
                if let Ok(overlay) = doc.create_element("div") {
                    overlay.set_id("rusby-error-overlay");
                    let _ = overlay.set_attribute("style", concat!(
                        "position:fixed;top:0;left:0;right:0;bottom:0;z-index:99999;",
                        "background:rgba(0,0,0,0.9);color:#fff;padding:24px;",
                        "font-family:monospace;font-size:14px;overflow:auto;",
                        "display:flex;flex-direction:column;align-items:center;justify-content:center;"
                    ));
                    overlay.set_inner_html(&format!(
                        "<div style='max-width:400px;text-align:center;'>\
                            <h2 style='color:#e74c3c;margin-bottom:16px;'>Errore Critico</h2>\
                            <p style='margin-bottom:8px;'>Il wallet ha riscontrato un errore inatteso.</p>\
                            <p style='color:#888;font-size:12px;margin-bottom:16px;word-break:break-all;'>{}</p>\
                            <p style='color:#666;font-size:11px;margin-bottom:24px;'>{}</p>\
                            <button onclick='location.reload()' \
                                style='padding:10px 24px;border-radius:8px;border:none;\
                                background:#3498db;color:#fff;cursor:pointer;font-size:14px;'>\
                                Ricarica Wallet\
                            </button>\
                        </div>",
                        html_escape(&msg),
                        html_escape(&location),
                    ));
                    if let Some(body) = doc.body() {
                        let _ = body.append_child(&overlay);
                    }
                }
            }
        }
    }));
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

/// Leptos error fallback component — wraps children and catches render errors
#[allow(unused)] // Leptos component macro generates struct field from prop
#[component]
pub fn AppErrorFallback(
    #[prop(into)] errors: ArcRwSignal<Errors>,
) -> impl IntoView {
    view! {
        <div style="padding: 16px; text-align: center;">
            <div style="border: 1px solid #e74c3c; border-radius: 8px; padding: 16px; background: rgba(231,76,60,0.1);">
                <h3 style="color: #e74c3c; margin: 0 0 8px 0;">"Errore di rendering"</h3>
                <p style="font-size: 0.85rem; color: #888; margin: 0 0 12px 0;">
                    {move || {
                        let errs = errors.read();
                        errs.iter().map(|(_, e)| format!("{}", e)).collect::<Vec<_>>().join(", ")
                    }}
                </p>
                <button
                    class="btn btn-primary"
                    on:click=move |_| {
                        if let Some(window) = web_sys::window() {
                            let _ = window.location().reload();
                        }
                    }
                >
                    "Ricarica"
                </button>
            </div>
        </div>
    }
}
