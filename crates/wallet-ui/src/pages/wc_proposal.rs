// Rusby Wallet — WalletConnect session proposal approval page
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use crate::state::*;
use crate::i18n::t;

#[component]
pub fn WcProposalPage() -> impl IntoView {
    let wallet_state: ReadSignal<WalletState> = expect_context();

    let (loading, set_loading) = signal(true);
    let (proposal_info, set_proposal_info) = signal::<Option<ProposalDisplay>>(None);
    let (status, set_status) = signal(String::new());
    let (processing, set_processing) = signal(false);

    // Fetch proposal from background on mount
    Effect::new(move |_| {
        let proposal_id = get_url_param("wc_proposal");
        if proposal_id.is_none() {
            set_loading.set(false);
            set_status.set(t("wc_proposal.no_proposal"));
            return;
        }
        let proposal_id = proposal_id.unwrap();

        wasm_bindgen_futures::spawn_local(async move {
            let data = serde_json::json!({});
            if let Some(response) = send_to_background("__rusby_wc_get_proposals", &data).await {
                if let Some(proposals) = response.get("proposals").and_then(|p| p.as_array()) {
                    for p in proposals {
                        let id = p.get("id").and_then(|i| i.as_u64()).unwrap_or(0);
                        if id.to_string() == proposal_id {
                            let metadata = p.get("params")
                                .and_then(|params| params.get("proposer"))
                                .and_then(|prop| prop.get("metadata"));

                            let name = metadata.and_then(|m| m.get("name")).and_then(|n| n.as_str()).map(|s| s.to_string()).unwrap_or_else(|| t("wc_proposal.unknown_dapp"));
                            let url = metadata.and_then(|m| m.get("url")).and_then(|u| u.as_str()).unwrap_or("").to_string();
                            let description = metadata.and_then(|m| m.get("description")).and_then(|d| d.as_str()).unwrap_or("").to_string();

                            // Extract required chains
                            let mut chains = Vec::new();
                            if let Some(ns) = p.get("params").and_then(|params| params.get("requiredNamespaces")).and_then(|n| n.as_object()) {
                                for (_namespace, config) in ns {
                                    if let Some(chain_list) = config.get("chains").and_then(|c| c.as_array()) {
                                        for c in chain_list {
                                            if let Some(s) = c.as_str() {
                                                chains.push(s.to_string());
                                            }
                                        }
                                    }
                                }
                            }

                            set_proposal_info.set(Some(ProposalDisplay {
                                id,
                                name,
                                url,
                                description,
                                chains,
                            }));
                            break;
                        }
                    }
                }
            }
            set_loading.set(false);
        });
    });

    let approve = move |_| {
        let Some(proposal) = proposal_info.get_untracked() else { return };
        set_processing.set(true);
        set_status.set(t("wc_proposal.approving"));

        let addresses = wallet_state.with_untracked(|s| s.addresses.clone());

        wasm_bindgen_futures::spawn_local(async move {
            // Build namespaces response with our addresses
            let mut namespaces = serde_json::Map::new();

            // Group chains by namespace
            let mut eip155_chains = Vec::new();
            let mut eip155_accounts = Vec::new();

            let evm_address = addresses.get("ethereum").cloned().unwrap_or_default();

            for chain in &proposal.chains {
                if chain.starts_with("eip155:") {
                    eip155_chains.push(serde_json::Value::String(chain.clone()));
                    eip155_accounts.push(serde_json::Value::String(format!("{}:{}", chain, evm_address)));
                }
            }

            if !eip155_chains.is_empty() {
                namespaces.insert("eip155".into(), serde_json::json!({
                    "chains": eip155_chains,
                    "methods": ["eth_sendTransaction", "personal_sign", "eth_signTypedData_v4"],
                    "events": ["chainChanged", "accountsChanged"],
                    "accounts": eip155_accounts,
                }));
            }

            let data = serde_json::json!({
                "proposalId": proposal.id,
                "namespaces": namespaces,
            });

            match send_to_background("__rusby_wc_approve_proposal", &data).await {
                Some(resp) if resp.get("ok").is_some() => {
                    set_status.set(t("wc_proposal.approved"));
                    close_after_delay();
                }
                Some(resp) => {
                    let err = resp.get("error").and_then(|e| e.as_str()).map(|e| e.to_string()).unwrap_or_else(|| t("wc.error"));
                    set_status.set(format!("{} {}", t("wc.error"), err));
                    set_processing.set(false);
                }
                None => {
                    set_status.set(t("wc_proposal.comm_error"));
                    set_processing.set(false);
                }
            }
        });
    };

    let reject = move |_| {
        let Some(proposal) = proposal_info.get_untracked() else { return };
        set_processing.set(true);
        set_status.set(t("wc_proposal.rejecting"));

        wasm_bindgen_futures::spawn_local(async move {
            let data = serde_json::json!({"proposalId": proposal.id});
            let _ = send_to_background("__rusby_wc_reject_proposal", &data).await;
            set_status.set(t("wc_proposal.rejected"));
            close_after_delay();
        });
    };

    view! {
        <div>
            <h2 class="mb-4">{move || t("wc_proposal.title")}</h2>
            {move || {
                if loading.get() {
                    Some(view! { <div class="card"><p>{t("wc_proposal.loading")}</p></div> }.into_any())
                } else if let Some(proposal) = proposal_info.get() {
                    Some(view! {
                        <div class="card">
                            <p class="text-sm text-muted mb-2">{t("wc_proposal.dapp")}</p>
                            <p style="font-weight: bold;">{proposal.name.clone()}</p>
                            {if !proposal.url.is_empty() {
                                Some(view! { <p class="text-sm text-muted" style="word-break: break-all;">{proposal.url.clone()}</p> })
                            } else { None }}
                            {if !proposal.description.is_empty() {
                                Some(view! { <p class="text-sm" style="margin-top: 4px;">{proposal.description.clone()}</p> })
                            } else { None }}
                        </div>
                        <div class="card">
                            <p class="text-sm text-muted mb-2">{t("wc_proposal.required_chains")}</p>
                            {proposal.chains.iter().map(|c| {
                                let chain_label = caip_to_label(c);
                                view! { <p class="text-sm">{chain_label}</p> }
                            }).collect::<Vec<_>>()}
                        </div>
                        <div style="display: flex; gap: 8px; margin-top: 16px;">
                            <button
                                class="btn btn-danger"
                                style="flex: 1;"
                                on:click=reject
                                disabled=move || processing.get()
                            >
                                {move || t("wc_proposal.reject")}
                            </button>
                            <button
                                class="btn btn-primary"
                                style="flex: 1;"
                                on:click=approve
                                disabled=move || processing.get()
                            >
                                {move || t("wc_proposal.approve")}
                            </button>
                        </div>
                    }.into_any())
                } else {
                    Some(view! { <div class="card"><p>{t("wc_proposal.not_found")}</p></div> }.into_any())
                }
            }}
            {move || {
                let s = status.get();
                if s.is_empty() { None }
                else { Some(view! { <p class="text-sm" style="padding: 8px; text-align: center;">{s}</p> }) }
            }}
        </div>
    }
}

#[derive(Debug, Clone)]
struct ProposalDisplay {
    id: u64,
    name: String,
    url: String,
    description: String,
    chains: Vec<String>,
}

fn caip_to_label(caip: &str) -> String {
    match caip {
        "eip155:1" => "Ethereum Mainnet".into(),
        "eip155:137" => "Polygon".into(),
        "eip155:56" => "BNB Smart Chain".into(),
        "eip155:10" => "Optimism".into(),
        "eip155:8453" => "Base".into(),
        "eip155:42161" => "Arbitrum".into(),
        _ => caip.to_string(),
    }
}

fn close_after_delay() {
    if let Some(window) = web_sys::window() {
        let cb = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            if let Some(w) = web_sys::window() {
                let _ = w.close();
            }
        }) as Box<dyn Fn()>);
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(), 500,
        );
        cb.forget();
    }
}
