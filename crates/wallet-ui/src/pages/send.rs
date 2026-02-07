// Rusby Wallet — Send page (UI only — TX logic in tx_send/)
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;
use crate::components::confirmation_modal::ConfirmationModal;
use crate::components::security_warning::{SecurityWarning, Severity};
use crate::tx_send;
use crate::i18n::t;

#[component]
pub fn SendPage() -> impl IntoView {
    let wallet_state: ReadSignal<WalletState> = expect_context();
    let set_page: WriteSignal<AppPage> = expect_context();
    let testnet_mode: ReadSignal<bool> = expect_context();

    let (recipient, set_recipient) = signal(String::new());
    let (amount, set_amount) = signal(String::new());
    let (status, set_status) = signal(String::new());
    let (status_type, set_status_type) = signal("warning"); // "warning" | "success" | "danger"
    let (sending, set_sending) = signal(false);
    let (show_confirm, set_show_confirm) = signal(false);
    let (estimated_fee, set_estimated_fee) = signal("0.0000".to_string());
    let (selected_token, set_selected_token) = signal(String::new());
    let (scam_warning, set_scam_warning) = signal::<Option<String>>(None);
    let (sim_warning, set_sim_warning) = signal::<Option<String>>(None);

    let active_chain = move || wallet_state.with(|s| s.active_chain.clone());

    let active_ticker = move || {
        let chain = wallet_state.with(|s| s.active_chain.clone());
        chain_list().into_iter()
            .find(|c| c.id == chain)
            .map(|c| c.ticker)
            .unwrap_or("???".into())
    };

    let active_chain_name = move || {
        let chain = wallet_state.with(|s| s.active_chain.clone());
        chain_list().into_iter()
            .find(|c| c.id == chain)
            .map(|c| c.name)
            .unwrap_or("Unknown".into())
    };

    let is_evm = move || {
        matches!(active_chain().as_str(), "ethereum" | "polygon" | "bsc" | "optimism" | "base" | "arbitrum")
    };

    let estimate = move |_| {
        let to = recipient.get();
        let amt = amount.get();

        if to.is_empty() {
            set_status.set(t("send.enter_recipient"));
            set_status_type.set("warning");
            return;
        }
        if amt.is_empty() || amt.parse::<f64>().is_err() {
            set_status.set(t("send.enter_amount"));
            set_status_type.set("warning");
            return;
        }

        set_status.set(String::new());

        if is_evm() {
            let chain = active_chain();
            let sim_to = to.clone();
            let sim_from = wallet_state.with(|s| s.current_address());
            set_estimated_fee.set(t("send.estimating"));
            set_sim_warning.set(None);
            let testnet = testnet_mode.get();
            wasm_bindgen_futures::spawn_local(async move {
                let chains = wallet_core::chains::get_chains(testnet);
                if let Some(config) = chains.iter().find(|c| tx_send::chain_id_to_string(&c.id) == chain) {
                    if let Some(rpc_url) = config.rpc_urls.first() {
                        match crate::rpc::evm::get_gas_price(rpc_url).await {
                            Ok(gas_price) => {
                                let fee_wei = gas_price * 21000;
                                let fee_gwei = fee_wei / 1_000_000_000;
                                set_estimated_fee.set(format!("~{} Gwei", fee_gwei));
                            }
                            Err(_) => {
                                set_estimated_fee.set("~21000 Gwei".into());
                            }
                        }
                        if let Ok(sim) = crate::rpc::simulate::simulate_evm_tx(
                            rpc_url, &sim_from, &sim_to, "0x0", ""
                        ).await {
                            if !sim.success {
                                let reason = sim.error.unwrap_or_else(|| t("send.tx_would_fail"));
                                set_sim_warning.set(Some(reason));
                            }
                        }
                    }
                }
            });
        } else if active_chain() == "bitcoin" {
            set_estimated_fee.set(t("send.estimating"));
            wasm_bindgen_futures::spawn_local(async move {
                match crate::rpc::bitcoin::get_fee_estimates().await {
                    Ok(fees) => {
                        let fee_sat = fees.half_hour * 141;
                        let fee_btc = fee_sat as f64 / 100_000_000.0;
                        set_estimated_fee.set(format!("~{:.8} BTC ({} sat/vB)", fee_btc, fees.half_hour));
                    }
                    Err(_) => {
                        set_estimated_fee.set("~0.00001 BTC".into());
                    }
                }
            });
        } else if active_chain() == "solana" {
            set_estimated_fee.set("~0.000005 SOL".into());
        } else if active_chain() == "ton" {
            if selected_token.get().is_empty() {
                set_estimated_fee.set("~0.01 TON".into());
            } else {
                set_estimated_fee.set("~0.1 TON".into());
            }
        } else if active_chain() == "litecoin" {
            set_estimated_fee.set(t("send.estimating"));
            wasm_bindgen_futures::spawn_local(async move {
                match crate::rpc::litecoin::get_fee_estimates(false).await {
                    Ok(fees) => {
                        let fee_litoshi = fees.half_hour * 141;
                        let fee_ltc = fee_litoshi as f64 / 100_000_000.0;
                        set_estimated_fee.set(format!("~{:.8} LTC ({} sat/vB)", fee_ltc, fees.half_hour));
                    }
                    Err(_) => {
                        set_estimated_fee.set("~0.00001 LTC".into());
                    }
                }
            });
        } else if active_chain() == "stellar" {
            set_estimated_fee.set("~0.00001 XLM (100 stroops)".into());
        } else if active_chain() == "ripple" {
            set_estimated_fee.set("~0.000012 XRP (12 drops)".into());
        } else if active_chain() == "dogecoin" {
            set_estimated_fee.set("~0.01 DOGE".into());
        } else if active_chain() == "tron" {
            set_estimated_fee.set("~1 TRX (bandwidth)".into());
        } else if active_chain() == "cosmos" || active_chain() == "osmosis" {
            if selected_token.get().is_empty() {
                set_estimated_fee.set("~0.005".into());
            } else {
                set_estimated_fee.set("~0.0075".into());
            }
        } else {
            set_estimated_fee.set("~0.005".into());
        }

        set_show_confirm.set(true);
    };

    let on_confirm = Callback::new(move |password: String| {
        set_show_confirm.set(false);
        set_sending.set(true);
        set_status.set(t("send.signing"));
        set_status_type.set("warning");

        let chain = active_chain();
        let to = recipient.get();
        let amt = amount.get();
        let token_addr = selected_token.get();
        let testnet = testnet_mode.get();

        wasm_bindgen_futures::spawn_local(async move {
            let result = tx_send::execute_send_for_network(&chain, &to, &amt, &password, &token_addr, testnet).await;
            set_sending.set(false);
            match result {
                Ok(tx_hash) => {
                    set_status.set(format!("{} {}", t("send.tx_sent"), tx_hash));
                    set_status_type.set("success");
                }
                Err(e) => {
                    set_status.set(format!("{} {}", t("send.error"), e));
                    set_status_type.set("danger");
                }
            }
        });
    });

    let on_cancel = Callback::new(move |_: ()| {
        set_show_confirm.set(false);
    });

    view! {
        <div class="p-4">
            <div class="flex items-center justify-between mb-4">
                <button class="btn btn-sm btn-secondary" on:click=move |_| set_page.set(AppPage::Dashboard)>
                    {move || t("send.back")}
                </button>
                <h2>{move || t("send.title")} " " {active_ticker}</h2>
                <div style="width: 60px;" />
            </div>

            // Token selector (EVM, Cosmos, Osmosis, TON)
            {move || {
                let supports_tokens = is_evm()
                    || active_chain() == "cosmos"
                    || active_chain() == "osmosis"
                    || active_chain() == "ton";
                if !supports_tokens {
                    return None;
                }
                let tokens = wallet_state.with(|s| s.token_balances.get(&s.active_chain).cloned().unwrap_or_default());
                if tokens.is_empty() {
                    return None;
                }
                Some(view! {
                    <div class="input-group">
                        <label>{t("send.asset")}</label>
                        <select
                            prop:value=move || selected_token.get()
                            on:change=move |ev| set_selected_token.set(event_target_value(&ev))
                            style="width: 100%; padding: 10px; border-radius: 8px; background: var(--bg-secondary); color: var(--text-primary); border: 1px solid var(--border);"
                        >
                            <option value="">{format!("{} ({})", active_ticker(), t("send.native"))}</option>
                            {tokens.into_iter().map(|tb| {
                                let addr = tb.token.address.clone();
                                let label = format!("{} — {}", tb.token.symbol, tb.balance);
                                view! { <option value=addr>{label}</option> }
                            }).collect::<Vec<_>>()}
                        </select>
                    </div>
                })
            }}

            <div class="input-group">
                <label>{t("send.recipient")}</label>
                <input
                    type="text"
                    placeholder=t("send.recipient_placeholder")
                    prop:value=move || recipient.get()
                    on:input=move |ev| {
                        let val = event_target_value(&ev);
                        set_recipient.set(val.clone());
                        let from = wallet_state.with(|s| s.current_address());
                        let (risk, reason) = wallet_core::security::scam_addresses::assess_address_risk(&val, &from);
                        use wallet_core::security::scam_addresses::RiskLevel;
                        match risk {
                            RiskLevel::High | RiskLevel::Medium | RiskLevel::Low => {
                                set_scam_warning.set(reason);
                            }
                            RiskLevel::Safe => set_scam_warning.set(None),
                        }
                    }
                />
            </div>

            // Scam address warning
            {move || {
                scam_warning.get().map(|msg| view! {
                    <SecurityWarning
                        severity=Severity::High
                        title=t("send.suspicious_address")
                        message=msg
                        dismissable=true
                    />
                })
            }}

            // TX simulation warning
            {move || {
                sim_warning.get().map(|msg| view! {
                    <SecurityWarning
                        severity=Severity::Medium
                        title=t("send.tx_simulation")
                        message=msg
                        dismissable=true
                    />
                })
            }}

            <div class="input-group">
                <label>{t("send.amount")}</label>
                <input
                    type="text"
                    placeholder="0.0"
                    prop:value=move || amount.get()
                    on:input=move |ev| set_amount.set(event_target_value(&ev))
                />
            </div>

            <div class="card text-sm">
                <div class="flex justify-between">
                    <span class="text-muted">{move || t("send.from")}</span>
                    <span style="font-family: monospace; font-size: 12px;">
                        {move || {
                            let addr = wallet_state.with(|s| s.current_address());
                            if addr.len() > 16 {
                                format!("{}...{}", &addr[..8], &addr[addr.len()-6..])
                            } else {
                                addr
                            }
                        }}
                    </span>
                </div>
                <div class="flex justify-between mt-2">
                    <span class="text-muted">{move || t("send.network")}</span>
                    <span>{active_chain_name}</span>
                </div>
                <div class="flex justify-between mt-2">
                    <span class="text-muted">{move || t("send.balance")}</span>
                    <span>{move || wallet_state.with(|s| s.current_balance())} " " {active_ticker}</span>
                </div>
            </div>

            {move || {
                let s = status.get();
                if s.is_empty() { None } else {
                    let color = match status_type.get() {
                        "success" => "var(--success, #4caf50)",
                        "danger" => "var(--danger, #f44336)",
                        _ => "var(--warning, #ff9800)",
                    };
                    Some(view! {
                        <p class="text-sm mt-2" style=format!("color: {}; word-break: break-all;", color)>{s}</p>
                    })
                }
            }}

            <button
                class="btn btn-primary btn-block mt-4"
                on:click=estimate
                disabled=move || sending.get()
            >
                {move || if sending.get() { t("send.sending") } else { t("send.send_tx") }}
            </button>

            {move || {
                if show_confirm.get() {
                    Some(view! {
                        <ConfirmationModal
                            recipient=recipient.get()
                            amount=amount.get()
                            fee=estimated_fee.get()
                            chain=active_chain_name()
                            ticker=active_ticker()
                            on_confirm=on_confirm
                            on_cancel=on_cancel
                        />
                    })
                } else {
                    None
                }
            }}
        </div>
    }
}
