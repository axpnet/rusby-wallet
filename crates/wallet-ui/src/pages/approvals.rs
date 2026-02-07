// Rusby Wallet — Token Approvals management page
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;
use crate::i18n::t;
use crate::rpc::approvals::{check_approvals_for_token, ApprovalInfo};
use crate::components::security_warning::{SecurityWarning, Severity};

#[component]
pub fn ApprovalsPage() -> impl IntoView {
    let wallet_state: ReadSignal<WalletState> = expect_context();
    let set_page: WriteSignal<AppPage> = expect_context();

    let (approvals, set_approvals) = signal::<Vec<ApprovalInfo>>(vec![]);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (revoking, set_revoking) = signal::<Option<String>>(None); // spender being revoked

    // Scan approvals on mount
    Effect::new(move |_| {
        let (is_unlocked, chain, owner, tokens) = wallet_state.with(|s| (
            s.is_unlocked,
            s.active_chain.clone(),
            s.current_address(),
            s.token_balances.get(&s.active_chain).cloned().unwrap_or_default(),
        ));
        if !is_unlocked { return; }
        if owner.is_empty() { return; }

        // Only EVM chains support ERC-20 approvals
        let is_evm = matches!(chain.as_str(), "ethereum" | "polygon" | "bsc" | "optimism" | "base" | "arbitrum");
        if !is_evm { return; }

        let rpc_url = wallet_core::chains::supported_chains()
            .into_iter()
            .find(|c| crate::rpc::chain_id_str(&c.id) == chain)
            .and_then(|c| c.rpc_urls.first().cloned());

        let rpc_url = match rpc_url {
            Some(u) => u,
            None => return,
        };

        set_loading.set(true);
        set_error.set(None);

        wasm_bindgen_futures::spawn_local(async move {
            let mut all_approvals = Vec::new();

            for token in &tokens {
                let results = check_approvals_for_token(
                    &rpc_url,
                    &token.token.address,
                    &token.token.symbol,
                    &owner,
                    &chain,
                ).await;
                all_approvals.extend(results);
            }

            set_approvals.set(all_approvals);
            set_loading.set(false);
        });
    });

    let go_back = move |_| set_page.set(AppPage::Settings);

    let revoke_approval = move |token_addr: String, spender_addr: String| {
        let (chain, from) = wallet_state.with(|s| (s.active_chain.clone(), s.current_address()));

        let rpc_url = wallet_core::chains::supported_chains()
            .into_iter()
            .find(|c| crate::rpc::chain_id_str(&c.id) == chain)
            .and_then(|c| c.rpc_urls.first().cloned());

        let _rpc_url = match rpc_url {
            Some(u) => u,
            None => {
                set_error.set(Some(t("approvals.no_rpc")));
                return;
            }
        };

        set_revoking.set(Some(spender_addr.clone()));

        wasm_bindgen_futures::spawn_local(async move {
            // Encode approve(spender, 0)
            match wallet_core::tokens::erc20::encode_revoke(&spender_addr) {
                Ok(calldata) => {
                    let hex_data = format!("0x{}", hex::encode(&calldata));
                    // Build TX and send via background
                    let data = serde_json::json!({
                        "from": from,
                        "to": token_addr,
                        "value": "0x0",
                        "data": hex_data,
                        "chain": chain,
                    });
                    let _ = send_to_background("__rusby_send_tx", &data).await;
                    // Remove from list optimistically
                    set_approvals.update(|list| {
                        list.retain(|a| !(a.token_address == token_addr && a.spender_address == spender_addr));
                    });
                }
                Err(e) => {
                    set_error.set(Some(format!("{} {}", t("approvals.revoke_error"), e)));
                }
            }
            set_revoking.set(None);
        });
    };

    view! {
        <div>
            <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 16px;">
                <button class="btn btn-secondary" style="padding: 4px 12px;" on:click=go_back>
                    "←"
                </button>
                <h2 style="margin: 0;">{move || t("approvals.title")}</h2>
            </div>

            <SecurityWarning
                severity=Severity::Low
                title=t("approvals.management")
                message=t("approvals.management_desc")
                dismissable=true
            />

            {move || {
                let chain = wallet_state.with(|s| s.active_chain.clone());
                let is_evm = matches!(chain.as_str(), "ethereum" | "polygon" | "bsc" | "optimism" | "base" | "arbitrum");
                if !is_evm {
                    return Some(view! {
                        <div class="card">
                            <p class="text-sm text-muted">{t("approvals.evm_only")}</p>
                        </div>
                    }.into_any());
                }
                None
            }}

            {move || {
                if loading.get() {
                    Some(view! {
                        <div class="card" style="text-align: center;">
                            <p>{t("approvals.scanning")}</p>
                        </div>
                    })
                } else {
                    None
                }
            }}

            {move || {
                if let Some(err) = error.get() {
                    Some(view! {
                        <div class="card" style="border: 1px solid #e74c3c;">
                            <p style="color: #e74c3c;">{err}</p>
                        </div>
                    })
                } else {
                    None
                }
            }}

            {move || {
                let list = approvals.get();
                if !loading.get() && list.is_empty() {
                    return Some(view! {
                        <div class="card">
                            <p class="text-sm text-muted">{t("approvals.no_approvals")}</p>
                        </div>
                    }.into_any());
                }

                if list.is_empty() {
                    return None;
                }

                Some(view! {
                    <div>
                        {list.into_iter().map(|approval| {
                            let token_addr = approval.token_address.clone();
                            let is_unlimited = approval.allowance == "Unlimited";
                            let token_addr_c = token_addr.clone();
                            let spender_addr_c = approval.spender_address.clone();
                            let spender_for_btn = approval.spender_address.clone();
                            let spender_for_label = approval.spender_address.clone();
                            let revoke = revoke_approval.clone();

                            view! {
                                <div class="card" style="margin-bottom: 8px;">
                                    <div style="display: flex; justify-content: space-between; align-items: center;">
                                        <div>
                                            <p style="font-weight: bold; margin: 0;">
                                                {approval.token_symbol.clone()}
                                            </p>
                                            <p class="text-sm text-muted" style="margin: 2px 0;">
                                                {format!("Spender: {}", approval.spender_name)}
                                            </p>
                                            <p class="text-sm" style="margin: 2px 0; font-family: monospace; font-size: 0.7rem;">
                                                {format!("{}...{}", &approval.spender_address[..6], &approval.spender_address[approval.spender_address.len()-4..])}
                                            </p>
                                            <p class="text-sm" style=format!("margin: 2px 0; color: {};", if is_unlimited { "#e74c3c" } else { "inherit" })>
                                                {format!("Allowance: {}", approval.allowance)}
                                            </p>
                                        </div>
                                        <button
                                            class="btn btn-danger"
                                            style="min-width: 80px;"
                                            disabled=move || revoking.get().as_deref() == Some(spender_for_btn.as_str())
                                            on:click=move |_| {
                                                revoke(token_addr_c.clone(), spender_addr_c.clone());
                                            }
                                        >
                                            {move || if revoking.get().as_deref() == Some(spender_for_label.as_str()) { "...".to_string() } else { t("approvals.revoke") }}
                                        </button>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any())
            }}
        </div>
    }
}
