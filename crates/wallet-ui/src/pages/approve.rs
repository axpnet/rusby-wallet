// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// dApp request approval page — handles EIP-1193 and WalletConnect requests

use leptos::prelude::*;
use crate::state::*;
use crate::i18n::t;
use crate::components::security_warning::{SecurityWarning, Severity};

#[component]
pub fn ApprovePage() -> impl IntoView {
    let set_page: WriteSignal<AppPage> = expect_context();
    let wallet_state: ReadSignal<WalletState> = expect_context();
    let (request, set_request) = signal::<Option<DappRequest>>(None);
    let (loading, set_loading) = signal(true);
    let (status_msg, set_status_msg) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (needs_password, set_needs_password) = signal(false);
    let (signing, set_signing) = signal(false);

    // On mount: fetch pending request from background
    Effect::new(move |_| {
        let request_id = get_url_param("approve");
        if request_id.is_none() {
            set_loading.set(false);
            set_status_msg.set(t("approve.no_request"));
            return;
        }
        let request_id = request_id.unwrap();

        wasm_bindgen_futures::spawn_local(async move {
            let data = serde_json::json!({});
            if let Some(response) = send_to_background("__rusby_get_pending", &data).await {
                if let Some(requests) = response.get("requests").and_then(|r| r.as_array()) {
                    for req in requests {
                        if req.get("requestId").and_then(|r| r.as_str()) == Some(&request_id) {
                            set_request.set(Some(DappRequest {
                                request_id: request_id.clone(),
                                origin: req.get("origin").and_then(|o| o.as_str()).unwrap_or("").to_string(),
                                method: req.get("method").and_then(|m| m.as_str()).unwrap_or("").to_string(),
                                params: req.get("params").map(|p| p.to_string()).unwrap_or_default(),
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
        let req = request.get_untracked();
        let addresses = wallet_state.with_untracked(|s| s.addresses.clone());
        if let Some(req) = req {
            match req.method.as_str() {
                "eth_requestAccounts" => {
                    // No password needed — just return address
                    set_status_msg.set(t("approve.approving"));
                    let addr = addresses.get("ethereum").cloned().unwrap_or_default();
                    let result = serde_json::json!({"requestId": req.request_id, "result": [addr]});
                    wasm_bindgen_futures::spawn_local(async move {
                        if send_to_background("__rusby_approve", &result).await.is_some() {
                            close_after_delay();
                        }
                    });
                }
                "personal_sign" | "eth_signTypedData_v4" | "eth_sendTransaction" => {
                    // Needs password to decrypt seed
                    set_needs_password.set(true);
                }
                _ => {
                    let result = serde_json::json!({"requestId": req.request_id, "result": null});
                    wasm_bindgen_futures::spawn_local(async move {
                        send_to_background("__rusby_approve", &result).await;
                        close_after_delay();
                    });
                }
            }
        }
    };

    let trigger_sign = move || {
        let req = request.get_untracked();
        let pwd = password.get_untracked();
        if pwd.is_empty() {
            set_status_msg.set(t("approve.enter_password"));
            return;
        }
        let Some(req) = req else { return };
        set_signing.set(true);
        set_status_msg.set(t("approve.signing"));

        wasm_bindgen_futures::spawn_local(async move {
            let sign_result = execute_sign(&req.method, &req.params, &pwd);
            match sign_result {
                Ok(sig_hex) => {
                    let result = serde_json::json!({
                        "requestId": req.request_id,
                        "result": sig_hex
                    });
                    if send_to_background("__rusby_approve", &result).await.is_some() {
                        set_status_msg.set(t("approve.signed"));
                        close_after_delay();
                    }
                }
                Err(e) => {
                    set_signing.set(false);
                    set_status_msg.set(e);
                }
            }
        });
    };

    let do_sign = move |_: web_sys::MouseEvent| {
        trigger_sign();
    };

    let reject = move |_| {
        let req = request.get_untracked();
        if let Some(req) = req {
            set_status_msg.set(t("approve.rejecting"));
            wasm_bindgen_futures::spawn_local(async move {
                let data = serde_json::json!({"requestId": req.request_id});
                send_to_background("__rusby_reject", &data).await;
                close_after_delay();
            });
        }
    };

    let method_label = move || {
        request.get().map(|r| match r.method.as_str() {
            "eth_requestAccounts" => t("approve.wallet_connection"),
            "personal_sign" => t("approve.sign_message"),
            "eth_signTypedData_v4" => t("approve.sign_typed"),
            "eth_sendTransaction" => t("approve.send_tx"),
            other => format!("{} {}", t("approve.request"), other),
        }).unwrap_or_default()
    };

    // Extract readable message for personal_sign
    let display_message = move || {
        let req = request.get()?;
        if req.method != "personal_sign" { return None; }
        // personal_sign params: [message_hex, address]
        let params: serde_json::Value = serde_json::from_str(&req.params).ok()?;
        let msg_hex = params.get(0)?.as_str()?;
        let hex_str = msg_hex.strip_prefix("0x").unwrap_or(msg_hex);
        let bytes = hex::decode(hex_str).ok()?;
        String::from_utf8(bytes).ok()
    };

    view! {
        <div>
            <h2 class="mb-4">{move || t("approve.title")}</h2>
            {move || {
                if loading.get() {
                    Some(view! { <div class="card"><p>{t("approve.loading")}</p></div> }.into_any())
                } else if let Some(req) = request.get() {
                    Some(view! {
                        <div class="card">
                            <p class="text-sm text-muted mb-2">{move || t("approve.origin")}</p>
                            <p style="font-weight: bold; word-break: break-all;">
                                {if req.origin.starts_with("walletconnect:") {
                                    t("approve.walletconnect")
                                } else {
                                    req.origin.clone()
                                }}
                            </p>
                        </div>
                        // Phishing domain warning
                        {
                            let origin = req.origin.clone();
                            let phishing_warning = wallet_core::security::phishing::check_suspicious_domain(&origin);
                            if let Some(reason) = phishing_warning {
                                Some(view! {
                                    <SecurityWarning
                                        severity=Severity::High
                                        title=t("approve.suspicious_domain")
                                        message=reason
                                        dismissable=false
                                    />
                                })
                            } else {
                                None
                            }
                        }
                        <div class="card">
                            <p class="text-sm text-muted mb-2">{move || t("approve.request_type")}</p>
                            <p style="font-weight: bold;">{method_label()}</p>
                        </div>
                        // Show readable message for personal_sign
                        {move || {
                            display_message().map(|msg| view! {
                                <div class="card">
                                    <p class="text-sm text-muted mb-2">{t("approve.message")}</p>
                                    <pre style="font-size: 0.8rem; white-space: pre-wrap; word-break: break-all; max-height: 120px; overflow-y: auto;">{msg}</pre>
                                </div>
                            })
                        }}
                        // Show raw params if not personal_sign or no readable message
                        {
                            let params = req.params.clone();
                            let method = req.method.clone();
                            if method != "personal_sign" && !params.is_empty() && params != "[]" && params != "null" {
                                Some(view! {
                                    <div class="card">
                                        <p class="text-sm text-muted mb-2">{t("approve.params")}</p>
                                        <pre style="font-size: 0.75rem; overflow-x: auto; max-height: 120px; white-space: pre-wrap; word-break: break-all;">{params}</pre>
                                    </div>
                                })
                            } else {
                                None
                            }
                        }
                        // Password input for signing operations
                        {move || {
                            if needs_password.get() {
                                Some(view! {
                                    <div class="card">
                                        <p class="text-sm text-muted mb-2">{t("approve.password_to_sign")}</p>
                                        <input
                                            type="password"
                                            placeholder=t("approve.enter_password_placeholder")
                                            style="width: 100%; padding: 8px; border-radius: 8px; background: var(--bg-secondary); color: var(--text-primary); border: 1px solid var(--border);"
                                            prop:value=move || password.get()
                                            on:input=move |ev| set_password.set(event_target_value(&ev))
                                            on:keydown=move |ev: web_sys::KeyboardEvent| {
                                                if ev.key() == "Enter" && !signing.get() {
                                                    trigger_sign();
                                                }
                                            }
                                        />
                                        <button
                                            class="btn btn-primary btn-block"
                                            style="margin-top: 8px;"
                                            on:click=do_sign
                                            disabled=move || signing.get()
                                        >
                                            {move || if signing.get() { t("approve.signing") } else { t("approve.sign_approve") }}
                                        </button>
                                    </div>
                                })
                            } else {
                                None
                            }
                        }}
                        // Approve/Reject buttons (hide approve if password mode active)
                        <div style="display: flex; gap: 8px; margin-top: 16px;">
                            {move || {
                                if !needs_password.get() {
                                    Some(view! {
                                        <button class="btn btn-primary" style="flex: 1;" on:click=approve>
                                            {move || t("approve.approve")}
                                        </button>
                                    })
                                } else {
                                    None
                                }
                            }}
                            <button class="btn btn-danger" style="flex: 1;" on:click=reject>
                                {move || t("approve.reject")}
                            </button>
                        </div>
                    }.into_any())
                } else {
                    Some(view! {
                        <div class="card">
                            <p>{t("approve.no_request")}</p>
                        </div>
                        <button class="btn btn-secondary btn-block mt-4" on:click=move |_| set_page.set(AppPage::Dashboard)>
                            {move || t("approve.back_dashboard")}
                        </button>
                    }.into_any())
                }
            }}
            {move || {
                let msg = status_msg.get();
                if !msg.is_empty() {
                    Some(view! { <p class="text-sm text-muted" style="text-align: center; margin-top: 8px;">{msg}</p> })
                } else {
                    None
                }
            }}
        </div>
    }
}

/// Execute signing operation — decrypt seed and sign
fn execute_sign(method: &str, params_json: &str, password: &str) -> Result<String, String> {
    // Load wallet store
    let store_json = load_from_storage("wallet_store")
        .ok_or(t("approve.wallet_not_found"))?;
    let store: wallet_core::wallet::WalletStore = serde_json::from_str(&store_json)
        .map_err(|e| format!("{} {}", t("approve.error_parsing_wallet"), e))?;
    let entry = store.wallets.get(store.active_index)
        .ok_or(t("approve.active_wallet_not_found"))?;

    // Decrypt seed
    let seed_bytes = wallet_core::crypto::decrypt(&entry.encrypted_seed, password)?;
    if seed_bytes.len() != 64 {
        return Err(t("approve.seed_invalid"));
    }
    let mut seed = [0u8; 64];
    seed.copy_from_slice(&seed_bytes);

    // Get EVM private key
    let private_key = wallet_core::chains::evm::get_private_key(&seed)
        .map_err(|e| { seed.fill(0); e })?;
    seed.fill(0); // zeroize

    let result = match method {
        "personal_sign" => {
            sign_personal_message(params_json, &private_key)
        }
        "eth_signTypedData_v4" => {
            sign_typed_data(params_json, &private_key)
        }
        "eth_sendTransaction" => {
            // For sendTransaction, we return the signed tx hash
            // Full implementation would build, sign and broadcast
            Err(t("approve.sendtx_unsupported"))
        }
        _ => Err(format!("{} {}", t("approve.method_unsupported"), method)),
    };

    result
}

/// EIP-191 personal_sign
fn sign_personal_message(params_json: &str, private_key: &[u8; 32]) -> Result<String, String> {
    let params: serde_json::Value = serde_json::from_str(params_json)
        .map_err(|e| format!("{} {}", t("approve.error_parsing_params"), e))?;

    // personal_sign params: [message_hex, address]
    let msg_hex = params.get(0)
        .and_then(|v| v.as_str())
        .ok_or(t("approve.missing_message_param"))?;

    let hex_str = msg_hex.strip_prefix("0x").unwrap_or(msg_hex);
    let message = hex::decode(hex_str)
        .map_err(|e| format!("{} {}", t("approve.invalid_hex"), e))?;

    let signature = wallet_core::signing::personal_sign::personal_sign(&message, private_key)?;
    Ok(format!("0x{}", hex::encode(signature)))
}

/// EIP-712 signTypedData_v4
fn sign_typed_data(params_json: &str, private_key: &[u8; 32]) -> Result<String, String> {
    let params: serde_json::Value = serde_json::from_str(params_json)
        .map_err(|e| format!("{} {}", t("approve.error_parsing_params"), e))?;

    // eth_signTypedData_v4 params: [address, typed_data_json]
    let typed_data_str = params.get(1)
        .and_then(|v| v.as_str())
        .ok_or(t("approve.missing_typed_data"))?;

    let typed_data: serde_json::Value = serde_json::from_str(typed_data_str)
        .map_err(|e| format!("{} {}", t("approve.error_parsing_params"), e))?;

    // Extract domain
    let domain = typed_data.get("domain")
        .ok_or(t("approve.missing_domain"))?;
    let name = domain.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let version = domain.get("version").and_then(|v| v.as_str()).unwrap_or("1");
    let chain_id = domain.get("chainId")
        .and_then(|v| v.as_u64())
        .unwrap_or(1);
    let contract_hex = domain.get("verifyingContract")
        .and_then(|v| v.as_str())
        .unwrap_or("0x0000000000000000000000000000000000000000");
    let contract_bytes = wallet_core::tx::evm::parse_address(contract_hex)?;

    let domain_separator = wallet_core::signing::eip712::hash_eip712_domain(
        name, version, chain_id, &contract_bytes
    );

    // For the struct hash, we need the primary type and its encoding
    // This is a simplified implementation — full EIP-712 recursive hashing
    // would require parsing the types field completely
    let message = typed_data.get("message")
        .ok_or(t("approve.missing_message_field"))?;
    let primary_type = typed_data.get("primaryType")
        .and_then(|v| v.as_str())
        .ok_or(t("approve.missing_primary_type"))?;
    let types = typed_data.get("types")
        .ok_or(t("approve.missing_types"))?;

    let struct_hash = compute_struct_hash(primary_type, message, types)?;

    let signature = wallet_core::signing::eip712::sign_typed_data_hash(
        &domain_separator, &struct_hash, private_key
    )?;
    Ok(format!("0x{}", hex::encode(signature)))
}

/// Compute EIP-712 struct hash (simplified — handles common cases)
fn compute_struct_hash(
    primary_type: &str,
    data: &serde_json::Value,
    types: &serde_json::Value,
) -> Result<[u8; 32], String> {
    use wallet_core::signing::keccak256;

    // Build type string: "PrimaryType(type1 name1,type2 name2,...)"
    let type_fields = types.get(primary_type)
        .and_then(|v| v.as_array())
        .ok_or(format!("{} {}", t("approve.type_not_found"), primary_type))?;

    let mut type_str = format!("{}(", primary_type);
    for (i, field) in type_fields.iter().enumerate() {
        if i > 0 { type_str.push(','); }
        let ftype = field.get("type").and_then(|v| v.as_str()).unwrap_or("uint256");
        let fname = field.get("name").and_then(|v| v.as_str()).unwrap_or("");
        type_str.push_str(&format!("{} {}", ftype, fname));
    }
    type_str.push(')');

    let type_hash = keccak256(type_str.as_bytes());

    // Encode data fields
    let mut encoded = Vec::with_capacity(32 + type_fields.len() * 32);
    encoded.extend_from_slice(&type_hash);

    for field in type_fields {
        let fname = field.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let ftype = field.get("type").and_then(|v| v.as_str()).unwrap_or("uint256");
        let value = data.get(fname);

        let encoded_value = encode_eip712_value(ftype, value)?;
        encoded.extend_from_slice(&encoded_value);
    }

    Ok(keccak256(&encoded))
}

/// Encode a single EIP-712 value to 32 bytes
fn encode_eip712_value(type_name: &str, value: Option<&serde_json::Value>) -> Result<[u8; 32], String> {
    use wallet_core::signing::keccak256;

    let mut result = [0u8; 32];

    match type_name {
        "address" => {
            let addr_str = value.and_then(|v| v.as_str()).unwrap_or("0x0000000000000000000000000000000000000000");
            let addr = wallet_core::tx::evm::parse_address(addr_str)?;
            result[12..].copy_from_slice(&addr);
        }
        "string" => {
            let s = value.and_then(|v| v.as_str()).unwrap_or("");
            result = keccak256(s.as_bytes());
        }
        "bytes" => {
            let hex_str = value.and_then(|v| v.as_str()).unwrap_or("0x");
            let clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
            let bytes = hex::decode(clean).unwrap_or_default();
            result = keccak256(&bytes);
        }
        "bool" => {
            if value.and_then(|v| v.as_bool()).unwrap_or(false) {
                result[31] = 1;
            }
        }
        t if t.starts_with("uint") || t.starts_with("int") => {
            if let Some(v) = value {
                if let Some(n) = v.as_u64() {
                    result[24..].copy_from_slice(&n.to_be_bytes());
                } else if let Some(s) = v.as_str() {
                    // Handle large numbers as decimal strings
                    if let Ok(n) = s.parse::<u128>() {
                        result[16..].copy_from_slice(&n.to_be_bytes());
                    }
                }
            }
        }
        t if t.starts_with("bytes") && t.len() > 5 => {
            // bytesN (fixed-size)
            let hex_str = value.and_then(|v| v.as_str()).unwrap_or("0x");
            let clean = hex_str.strip_prefix("0x").unwrap_or(hex_str);
            let bytes = hex::decode(clean).unwrap_or_default();
            let len = bytes.len().min(32);
            result[..len].copy_from_slice(&bytes[..len]);
        }
        _ => {
            // Unknown type — hash the JSON representation
            let json = value.map(|v| v.to_string()).unwrap_or_default();
            result = keccak256(json.as_bytes());
        }
    }

    Ok(result)
}

fn close_after_delay() {
    gloo_timers::callback::Timeout::new(500, move || {
        if let Some(window) = web_sys::window() {
            let _ = window.close();
        }
    }).forget();
}
