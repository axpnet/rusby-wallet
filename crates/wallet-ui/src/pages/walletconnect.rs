// Rusby Wallet — WalletConnect management page
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;
use crate::state::*;
use crate::i18n::t;

#[component]
pub fn WalletConnectPage() -> impl IntoView {
    let set_page: WriteSignal<AppPage> = expect_context();

    let (uri, set_uri) = signal(String::new());
    let (status, set_status) = signal(String::new());
    let (connecting, set_connecting) = signal(false);
    let (sessions, set_sessions) = signal::<Vec<WcSessionInfo>>(vec![]);
    let (project_id, set_project_id) = signal(
        load_from_storage("wc_project_id").unwrap_or_default()
    );
    let (show_config, set_show_config) = signal(false);

    // Fetch active sessions on mount
    Effect::new(move |_| {
        wasm_bindgen_futures::spawn_local(async move {
            let data = serde_json::json!({});
            if let Some(response) = send_to_background("__rusby_wc_get_sessions", &data).await {
                if let Some(sessions_obj) = response.get("sessions").and_then(|s| s.as_object()) {
                    let list: Vec<WcSessionInfo> = sessions_obj.iter().map(|(topic, info)| {
                        let peer = info.get("peer").and_then(|p| p.get("metadata"));
                        WcSessionInfo {
                            topic: topic.clone(),
                            name: peer.and_then(|m| m.get("name")).and_then(|n| n.as_str()).unwrap_or("Unknown").to_string(),
                            url: peer.and_then(|m| m.get("url")).and_then(|u| u.as_str()).unwrap_or("").to_string(),
                        }
                    }).collect();
                    set_sessions.set(list);
                }
            }
        });
    });

    let pair = move |_| {
        let uri_val = uri.get_untracked();
        if uri_val.is_empty() || !uri_val.starts_with("wc:") {
            set_status.set(t("wc.invalid_uri"));
            return;
        }
        set_connecting.set(true);
        set_status.set(t("wc.connecting"));
        wasm_bindgen_futures::spawn_local(async move {
            let data = serde_json::json!({"uri": uri_val});
            match send_to_background("__rusby_wc_pair", &data).await {
                Some(resp) if resp.get("ok").is_some() => {
                    set_status.set(t("wc.pairing_started"));
                    set_uri.set(String::new());
                }
                Some(resp) => {
                    let err = resp.get("error").and_then(|e| e.as_str()).map(|e| e.to_string()).unwrap_or_else(|| t("wc.unknown_error"));
                    set_status.set(format!("{} {}", t("wc.error"), err));
                }
                None => {
                    set_status.set(t("wc.bg_error"));
                }
            }
            set_connecting.set(false);
        });
    };

    let save_project_id = move |_| {
        let pid = project_id.get_untracked();
        save_to_storage("wc_project_id", &pid);
        wasm_bindgen_futures::spawn_local(async move {
            let data = serde_json::json!({"projectId": pid});
            let _ = send_to_background("__rusby_wc_set_project_id", &data).await;
        });
        set_status.set(t("wc.project_id_saved"));
        set_show_config.set(false);
    };

    view! {
        <div>
            <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 16px;">
                <button class="btn btn-secondary" style="padding: 4px 8px;" on:click=move |_| set_page.set(AppPage::Settings)>
                    {move || t("wc.back")}
                </button>
                <h2>{move || t("wc.title")}</h2>
            </div>

            // Config section
            <div class="card">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <p class="text-sm text-muted">{move || t("wc.config")}</p>
                    <button class="btn btn-secondary" style="font-size: 0.7rem; padding: 2px 8px;" on:click=move |_| set_show_config.update(|v| *v = !*v)>
                        {move || if show_config.get() { t("wc.close") } else { t("wc.edit") }}
                    </button>
                </div>
                {move || {
                    if show_config.get() {
                        Some(view! {
                            <div style="margin-top: 8px;">
                                <label class="text-sm">{t("wc.project_id")}</label>
                                <input
                                    type="text"
                                    placeholder=t("wc.project_id_placeholder")
                                    style="width: 100%; padding: 8px; border-radius: 8px; background: var(--bg-secondary); color: var(--text-primary); border: 1px solid var(--border); margin-top: 4px;"
                                    prop:value=move || project_id.get()
                                    on:input=move |ev| set_project_id.set(event_target_value(&ev))
                                />
                                <button class="btn btn-primary btn-block" style="margin-top: 8px;" on:click=save_project_id>
                                    {move || t("wc.save")}
                                </button>
                                <p class="text-sm text-muted" style="margin-top: 4px;">{move || t("wc.project_id_hint")}</p>
                            </div>
                        })
                    } else {
                        None
                    }
                }}
            </div>

            // Pair section
            <div class="card">
                <p class="text-sm text-muted mb-2">{move || t("wc.connect_dapp")}</p>
                <input
                    type="text"
                    placeholder="wc:..."
                    style="width: 100%; padding: 8px; border-radius: 8px; background: var(--bg-secondary); color: var(--text-primary); border: 1px solid var(--border);"
                    prop:value=move || uri.get()
                    on:input=move |ev| set_uri.set(event_target_value(&ev))
                    on:keydown=move |ev: web_sys::KeyboardEvent| {
                        if ev.key() == "Enter" && !connecting.get() {
                            let uri_val = uri.get_untracked();
                            if !uri_val.is_empty() {
                                set_connecting.set(true);
                                set_status.set(t("wc.connecting"));
                                wasm_bindgen_futures::spawn_local(async move {
                                    let data = serde_json::json!({"uri": uri_val});
                                    match send_to_background("__rusby_wc_pair", &data).await {
                                        Some(resp) if resp.get("ok").is_some() => {
                                            set_status.set(t("wc.pairing_started_short"));
                                            set_uri.set(String::new());
                                        }
                                        _ => set_status.set(t("wc.pairing_error")),
                                    }
                                    set_connecting.set(false);
                                });
                            }
                        }
                    }
                />
                <button
                    class="btn btn-primary btn-block"
                    style="margin-top: 8px;"
                    on:click=pair
                    disabled=move || connecting.get()
                >
                    {move || if connecting.get() { t("wc.connecting_btn") } else { t("wc.connect") }}
                </button>
            </div>

            // Status
            {move || {
                let s = status.get();
                if s.is_empty() {
                    None
                } else {
                    Some(view! { <p class="text-sm" style="padding: 8px; text-align: center;">{s}</p> })
                }
            }}

            // Active sessions
            <div class="card">
                <p class="text-sm text-muted mb-2">{move || t("wc.active_sessions")}</p>
                {move || {
                    let list = sessions.get();
                    if list.is_empty() {
                        Some(view! { <p class="text-sm">{t("wc.no_sessions")}</p> }.into_any())
                    } else {
                        Some(view! {
                            <div>
                                {list.into_iter().map(|session| {
                                    let topic = session.topic.clone();
                                    let name = session.name.clone();
                                    let url = session.url.clone();
                                    view! {
                                        <div style="display: flex; justify-content: space-between; align-items: center; padding: 8px 0; border-bottom: 1px solid var(--border);">
                                            <div>
                                                <p style="font-weight: bold; font-size: 0.9rem;">{name}</p>
                                                <p class="text-sm text-muted" style="word-break: break-all;">{url}</p>
                                            </div>
                                            <button
                                                class="btn btn-secondary"
                                                style="font-size: 0.7rem; padding: 2px 8px; min-width: auto;"
                                                on:click=move |_| {
                                                    let t = topic.clone();
                                                    wasm_bindgen_futures::spawn_local(async move {
                                                        let data = serde_json::json!({"topic": t});
                                                        let _ = send_to_background("__rusby_wc_disconnect", &data).await;
                                                    });
                                                    set_sessions.update(|list| list.retain(|s| s.topic != topic));
                                                }
                                            >
                                                {t("wc.disconnect")}
                                            </button>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any())
                    }
                }}
            </div>
        </div>
    }
}

#[derive(Debug, Clone)]
struct WcSessionInfo {
    topic: String,
    name: String,
    url: String,
}
