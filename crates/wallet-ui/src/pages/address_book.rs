// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use leptos::prelude::*;

use crate::state::*;
use crate::i18n::t;

#[component]
pub fn AddressBookPage() -> impl IntoView {
    let set_page: WriteSignal<AppPage> = expect_context();

    let (contacts, set_contacts) = signal(load_address_book());
    let (search, set_search) = signal(String::new());
    let (editing, set_editing) = signal(Option::<usize>::None);

    // Form fields
    let (form_name, set_form_name) = signal(String::new());
    let (form_address, set_form_address) = signal(String::new());
    let (form_chain, set_form_chain) = signal(String::new());
    let (form_notes, set_form_notes) = signal(String::new());
    let (show_form, set_show_form) = signal(false);

    let clear_form = move || {
        set_form_name.set(String::new());
        set_form_address.set(String::new());
        set_form_chain.set(String::new());
        set_form_notes.set(String::new());
        set_editing.set(None);
        set_show_form.set(false);
    };

    let save_contact = move |_| {
        let name = form_name.get();
        let address = form_address.get();
        if name.is_empty() || address.is_empty() {
            return;
        }
        let contact = Contact {
            name,
            address,
            chain_hint: {
                let c = form_chain.get();
                if c.is_empty() { None } else { Some(c) }
            },
            notes: {
                let n = form_notes.get();
                if n.is_empty() { None } else { Some(n) }
            },
        };

        let mut list = contacts.get();
        if let Some(idx) = editing.get() {
            if idx < list.len() {
                list[idx] = contact;
            }
        } else {
            list.push(contact);
        }
        save_address_book(&list);
        set_contacts.set(list);
        clear_form();
    };

    let start_edit = move |idx: usize| {
        let list = contacts.get();
        if let Some(c) = list.get(idx) {
            set_form_name.set(c.name.clone());
            set_form_address.set(c.address.clone());
            set_form_chain.set(c.chain_hint.clone().unwrap_or_default());
            set_form_notes.set(c.notes.clone().unwrap_or_default());
            set_editing.set(Some(idx));
            set_show_form.set(true);
        }
    };

    let delete_contact = move |idx: usize| {
        let mut list = contacts.get();
        if idx < list.len() {
            list.remove(idx);
            save_address_book(&list);
            set_contacts.set(list);
        }
    };

    view! {
        <div class="p-4">
            <div class="flex items-center justify-between mb-4">
                <button class="btn btn-sm btn-secondary" on:click=move |_| set_page.set(AppPage::Settings)>
                    {move || t("common.back")}
                </button>
                <h2>{move || t("addressbook.title")}</h2>
                <button class="btn btn-sm btn-primary" on:click=move |_| {
                    clear_form();
                    set_show_form.set(true);
                }>
                    {move || t("addressbook.add")}
                </button>
            </div>

            // Search
            <div class="mb-3">
                <input
                    type="text"
                    class="input"
                    placeholder={t("addressbook.search")}
                    prop:value=move || search.get()
                    on:input=move |ev| set_search.set(event_target_value(&ev))
                />
            </div>

            // Add/Edit Form
            {move || {
                if !show_form.get() {
                    return view! { <div /> }.into_any();
                }
                view! {
                    <div class="card mb-3 p-3">
                        <div class="mb-2">
                            <label class="text-sm text-muted">{t("addressbook.name")}</label>
                            <input
                                type="text"
                                class="input"
                                prop:value=move || form_name.get()
                                on:input=move |ev| set_form_name.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="mb-2">
                            <label class="text-sm text-muted">{t("addressbook.address")}</label>
                            <input
                                type="text"
                                class="input"
                                prop:value=move || form_address.get()
                                on:input=move |ev| set_form_address.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="mb-2">
                            <label class="text-sm text-muted">{t("addressbook.chain")}</label>
                            <input
                                type="text"
                                class="input"
                                placeholder="ethereum, solana, bitcoin..."
                                prop:value=move || form_chain.get()
                                on:input=move |ev| set_form_chain.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="mb-2">
                            <label class="text-sm text-muted">{t("addressbook.notes")}</label>
                            <input
                                type="text"
                                class="input"
                                prop:value=move || form_notes.get()
                                on:input=move |ev| set_form_notes.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="flex gap-2">
                            <button class="btn btn-primary flex-1" on:click=save_contact>
                                {move || t("addressbook.save")}
                            </button>
                            <button class="btn btn-secondary flex-1" on:click=move |_| clear_form()>
                                {move || t("common.cancel")}
                            </button>
                        </div>
                    </div>
                }.into_any()
            }}

            // Contact List
            {move || {
                let list = contacts.get();
                let query = search.get().to_lowercase();

                let filtered: Vec<(usize, Contact)> = list.into_iter().enumerate()
                    .filter(|(_, c)| {
                        if query.is_empty() { return true; }
                        c.name.to_lowercase().contains(&query)
                            || c.address.to_lowercase().contains(&query)
                            || c.notes.as_deref().unwrap_or("").to_lowercase().contains(&query)
                    })
                    .collect();

                if filtered.is_empty() {
                    return view! { <p class="text-center text-muted">{t("addressbook.empty")}</p> }.into_any();
                }

                view! {
                    <div class="chain-list">
                        {filtered.into_iter().map(|(idx, contact)| {
                            let addr_short = if contact.address.len() > 20 {
                                format!("{}...{}", &contact.address[..10], &contact.address[contact.address.len()-8..])
                            } else {
                                contact.address.clone()
                            };
                            let chain_tag = contact.chain_hint.clone().unwrap_or_default();
                            let notes_display = contact.notes.clone().unwrap_or_default();

                            view! {
                                <div class="chain-item">
                                    <div class="chain-info" style="flex: 1;">
                                        <div class="chain-name">{contact.name.clone()}</div>
                                        <div class="text-sm text-muted" style="font-family: monospace;">
                                            {addr_short}
                                        </div>
                                        {if !chain_tag.is_empty() {
                                            view! { <span class="badge" style="font-size: 0.7em;">{chain_tag}</span> }.into_any()
                                        } else {
                                            view! { <span /> }.into_any()
                                        }}
                                        {if !notes_display.is_empty() {
                                            view! { <div class="text-sm text-muted" style="font-style: italic;">{notes_display}</div> }.into_any()
                                        } else {
                                            view! { <span /> }.into_any()
                                        }}
                                    </div>
                                    <div class="flex gap-1">
                                        <button class="btn btn-sm btn-secondary"
                                            on:click=move |_| start_edit(idx)
                                        >
                                            {t("addressbook.edit")}
                                        </button>
                                        <button class="btn btn-sm btn-danger"
                                            on:click=move |_| delete_contact(idx)
                                        >
                                            {t("addressbook.delete")}
                                        </button>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }.into_any()
            }}
        </div>
    }
}
