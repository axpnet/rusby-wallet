// Rusby Wallet — NFT gallery page
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;
use crate::i18n::t;
use wallet_core::nft::NftItem;

#[component]
pub fn NftPage() -> impl IntoView {
    let wallet_state: ReadSignal<WalletState> = expect_context();
    let set_wallet_state: WriteSignal<WalletState> = expect_context();
    let set_page: WriteSignal<AppPage> = expect_context();

    let (loading, set_loading) = signal(false);
    let (selected_nft, set_selected_nft) = signal::<Option<NftItem>>(None);

    // Fetch NFTs on mount
    Effect::new(move |_| {
        let (is_unlocked, chain, address) = wallet_state.with(|s| {
            (s.is_unlocked, s.active_chain.clone(), s.current_address())
        });
        if !is_unlocked {
            return;
        }
        if address.is_empty() {
            return;
        }

        set_loading.set(true);
        wasm_bindgen_futures::spawn_local(async move {
            let nfts = crate::rpc::nft::fetch_nfts(&address, &chain).await;
            set_wallet_state.update(|s| s.nfts = nfts);
            set_loading.set(false);
        });
    });

    view! {
        <div class="p-4">
            // Header
            <div class="flex items-center justify-between mb-4">
                <button class="btn btn-sm btn-secondary" on:click=move |_| set_page.set(AppPage::Dashboard)>
                    {move || t("send.back")}
                </button>
                <h2>{move || t("nft.title")}</h2>
                <div style="width: 60px;" />
            </div>

            // Loading state
            {move || {
                if loading.get() {
                    Some(view! {
                        <div class="text-center p-4">
                            <p class="text-muted">{t("nft.loading")}</p>
                        </div>
                    })
                } else {
                    None
                }
            }}

            // NFT grid
            {move || {
                if loading.get() {
                    return None;
                }
                let nfts = wallet_state.with(|s| s.nfts.clone());
                if nfts.is_empty() {
                    // Empty state
                    Some(view! {
                        <div class="card text-center" style="padding: 40px 20px;">
                            <div style="font-size: 48px; margin-bottom: 12px; color: var(--text-muted);">"◆"</div>
                            <p class="text-muted">{t("nft.empty")}</p>
                            <p class="text-sm text-muted" style="margin-top: 8px;">
                                {t("nft.empty_hint")}
                            </p>
                            <button
                                class="btn btn-sm btn-secondary"
                                style="margin-top: 16px;"
                                on:click=move |_| set_page.set(AppPage::Settings)
                            >
                                {move || t("nav.settings")}
                            </button>
                        </div>
                    }.into_any())
                } else {
                    Some(view! {
                        <div class="nft-grid">
                            {nfts.into_iter().map(|nft| {
                                let nft_clone = nft.clone();
                                let has_image = !nft.image_url.is_empty();
                                let image_url = nft.image_url.clone();
                                let name = if nft.name.is_empty() {
                                    format!("#{}", nft.token_id)
                                } else {
                                    nft.name.clone()
                                };
                                let collection = nft.collection_name.clone();
                                view! {
                                    <div
                                        class="nft-card"
                                        on:click=move |_| set_selected_nft.set(Some(nft_clone.clone()))
                                    >
                                        {if has_image {
                                            view! {
                                                <img
                                                    src=image_url
                                                    alt=name.clone()
                                                    loading="lazy"
                                                />
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div style="width: 100%; aspect-ratio: 1; background: var(--bg-secondary); display: flex; align-items: center; justify-content: center; font-size: 32px; color: var(--text-muted);">
                                                    "◆"
                                                </div>
                                            }.into_any()
                                        }}
                                        <div class="nft-card-info">
                                            <div class="nft-card-name">{name}</div>
                                            <div class="nft-card-collection">{collection}</div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any())
                }
            }}

            // Detail modal
            {move || {
                selected_nft.get().map(|nft| {
                    let has_image = !nft.image_url.is_empty();
                    let image_url = nft.image_url.clone();
                    let name = if nft.name.is_empty() {
                        format!("#{}", nft.token_id)
                    } else {
                        nft.name.clone()
                    };
                    let description = nft.description.clone();
                    let collection = nft.collection_name.clone();
                    let contract = nft.contract_address.clone();
                    let token_id = nft.token_id.clone();
                    let chain = nft.chain_id.clone();
                    let standard = nft.token_standard.clone();

                    let contract_short = if contract.len() > 16 {
                        format!("{}...{}", &contract[..8], &contract[contract.len()-6..])
                    } else {
                        contract.clone()
                    };

                    view! {
                        <div
                            class="nft-detail-overlay"
                            on:click=move |_| set_selected_nft.set(None)
                        >
                            <div
                                class="nft-detail"
                                on:click=move |ev| ev.stop_propagation()
                            >
                                {if has_image {
                                    view! {
                                        <img src=image_url alt=name.clone() />
                                    }.into_any()
                                } else {
                                    view! {
                                        <div style="width: 100%; aspect-ratio: 1; background: var(--bg-card); border-radius: var(--radius-sm); display: flex; align-items: center; justify-content: center; font-size: 48px; color: var(--text-muted); margin-bottom: 12px;">
                                            "◆"
                                        </div>
                                    }.into_any()
                                }}

                                <h3 style="margin-bottom: 8px;">{name}</h3>

                                {if !description.is_empty() {
                                    Some(view! {
                                        <p class="text-sm text-muted" style="margin-bottom: 12px;">{description}</p>
                                    })
                                } else {
                                    None
                                }}

                                <div class="card" style="font-size: 12px;">
                                    <div class="flex justify-between mb-2">
                                        <span class="text-muted">{t("nft.collection")}</span>
                                        <span>{collection}</span>
                                    </div>
                                    <div class="flex justify-between mb-2">
                                        <span class="text-muted">{t("nft.contract")}</span>
                                        <span style="font-family: monospace;">{contract_short}</span>
                                    </div>
                                    <div class="flex justify-between mb-2">
                                        <span class="text-muted">{t("nft.token_id")}</span>
                                        <span style="font-family: monospace;">{token_id}</span>
                                    </div>
                                    <div class="flex justify-between mb-2">
                                        <span class="text-muted">{t("nft.chain")}</span>
                                        <span>{chain}</span>
                                    </div>
                                    <div class="flex justify-between">
                                        <span class="text-muted">"Standard"</span>
                                        <span>{standard}</span>
                                    </div>
                                </div>

                                <button
                                    class="btn btn-secondary btn-block"
                                    style="margin-top: 12px;"
                                    on:click=move |_| set_selected_nft.set(None)
                                >
                                    {move || t("nft.close")}
                                </button>
                            </div>
                        </div>
                    }
                })
            }}
        </div>
    }
}
