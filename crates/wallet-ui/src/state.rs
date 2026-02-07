// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use wallet_core::tokens::TokenBalance;
use wallet_core::chains::get_chains;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Newtype wrapper to disambiguate fullpage ReadSignal<bool> from testnet_mode
#[derive(Clone, Copy)]
pub struct FullpageMode(pub ReadSignal<bool>);

/// Application state
#[derive(Debug, Clone, PartialEq)]
pub enum AppPage {
    Onboarding,
    Login,
    Dashboard,
    Send,
    Receive,
    History,
    Settings,
    DappApproval,
    WalletConnect,
    WcProposal,
    Approvals,
    AddressBook,
    Nft,
    Swap,
}

/// Wallet state shared across components
#[derive(Debug, Clone)]
pub struct WalletState {
    pub is_unlocked: bool,
    pub wallet_name: String,
    pub addresses: HashMap<String, String>,
    pub active_chain: String,
    pub balances: HashMap<String, String>,
    pub balance_loading: bool,
    pub prices: HashMap<String, f64>,
    pub token_balances: HashMap<String, Vec<TokenBalance>>,
    pub nfts: Vec<wallet_core::nft::NftItem>,
}

impl Default for WalletState {
    fn default() -> Self {
        Self {
            is_unlocked: false,
            wallet_name: String::new(),
            addresses: HashMap::new(),
            active_chain: "ethereum".to_string(),
            balances: HashMap::new(),
            balance_loading: false,
            prices: HashMap::new(),
            token_balances: HashMap::new(),
            nfts: Vec::new(),
        }
    }
}

impl WalletState {
    pub fn current_address(&self) -> String {
        self.addresses
            .get(&self.active_chain)
            .cloned()
            .unwrap_or_default()
    }

    pub fn current_balance(&self) -> String {
        self.balances
            .get(&self.active_chain)
            .cloned()
            .unwrap_or_else(|| "0.0000".to_string())
    }
}

/// Chain display info
pub struct ChainDisplay {
    pub id: String,
    pub name: String,
    pub ticker: String,
    pub icon: String,
}

pub fn chain_list() -> Vec<ChainDisplay> {
    chain_list_for(false)
}

/// Get chain display list for mainnet or testnet (reads from wallet-core config)
pub fn chain_list_for(testnet: bool) -> Vec<ChainDisplay> {
    use wallet_core::chains::ChainId;
    let chains = get_chains(testnet);
    chains.iter().map(|c| {
        let (id, icon) = match &c.id {
            ChainId::Ethereum => ("ethereum", "E"),
            ChainId::Polygon => ("polygon", "P"),
            ChainId::Bsc => ("bsc", "B"),
            ChainId::Optimism => ("optimism", "O"),
            ChainId::Base => ("base", "Ba"),
            ChainId::Arbitrum => ("arbitrum", "A"),
            ChainId::Solana => ("solana", "S"),
            ChainId::Ton => ("ton", "T"),
            ChainId::CosmosHub => ("cosmos", "C"),
            ChainId::Osmosis => ("osmosis", "Os"),
            ChainId::Bitcoin => ("bitcoin", "₿"),
        };
        ChainDisplay {
            id: id.into(),
            name: c.name.clone(),
            ticker: c.ticker.clone(),
            icon: icon.into(),
        }
    }).collect()
}

/// Address book contact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub address: String,
    pub chain_hint: Option<String>,
    pub notes: Option<String>,
}

/// Save address book to storage
pub fn save_address_book(contacts: &[Contact]) {
    if let Ok(json) = serde_json::to_string(contacts) {
        save_to_storage("address_book", &json);
    }
}

/// Load address book from storage
pub fn load_address_book() -> Vec<Contact> {
    load_from_storage("address_book")
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

/// Detect if chrome.storage.local is available (extension context)
pub fn has_chrome_storage() -> bool {
    if let Some(window) = web_sys::window() {
        if let Ok(chrome) = js_sys::Reflect::get(&window, &"chrome".into()) {
            if !chrome.is_undefined() && !chrome.is_null() {
                if let Ok(storage) = js_sys::Reflect::get(&chrome, &"storage".into()) {
                    return !storage.is_undefined() && !storage.is_null();
                }
            }
        }
    }
    false
}

/// Save to storage — uses localStorage (sync) and also chrome.storage.local if available
pub fn save_to_storage(key: &str, value: &str) {
    // Always save to localStorage for immediate sync reads
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(key, value);
        }
    }
    // Also persist to chrome.storage.local for cross-context sync
    if has_chrome_storage() {
        let key = key.to_string();
        let value = value.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            chrome_storage_set(&key, &value).await;
        });
    }
}

pub fn load_from_storage(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item(key).ok()?
}

pub fn remove_from_storage(key: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item(key);
        }
    }
    if has_chrome_storage() {
        let key = key.to_string();
        wasm_bindgen_futures::spawn_local(async move {
            chrome_storage_remove(&key).await;
        });
    }
}

/// Sync chrome.storage.local → localStorage on extension startup
pub fn sync_chrome_storage_to_local() {
    if !has_chrome_storage() {
        return;
    }
    wasm_bindgen_futures::spawn_local(async {
        // Get all keys from chrome.storage.local and mirror to localStorage
        if let Some(data) = chrome_storage_get_all().await {
            if let Some(window) = web_sys::window() {
                if let Ok(Some(storage)) = window.local_storage() {
                    let keys = js_sys::Object::keys(&data);
                    for i in 0..keys.length() {
                        if let Some(key) = keys.get(i).as_string() {
                            if let Ok(val) = js_sys::Reflect::get(&data, &key.clone().into()) {
                                if let Some(s) = val.as_string() {
                                    let _ = storage.set_item(&key, &s);
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}

async fn chrome_storage_set(key: &str, value: &str) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let chrome = match js_sys::Reflect::get(&window, &"chrome".into()) {
        Ok(c) if !c.is_undefined() => c,
        _ => return,
    };
    let storage = match js_sys::Reflect::get(&chrome, &"storage".into()) {
        Ok(s) if !s.is_undefined() => s,
        _ => return,
    };
    let local = match js_sys::Reflect::get(&storage, &"local".into()) {
        Ok(l) if !l.is_undefined() => l,
        _ => return,
    };
    let obj = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&obj, &key.into(), &value.into());
    if let Ok(set_fn) = js_sys::Reflect::get(&local, &"set".into()) {
        if let Some(set_fn) = set_fn.dyn_ref::<js_sys::Function>() {
            if let Ok(promise) = set_fn.call1(&local, &obj) {
                let _ = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await;
            }
        }
    }
}

async fn chrome_storage_remove(key: &str) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let chrome = match js_sys::Reflect::get(&window, &"chrome".into()) {
        Ok(c) if !c.is_undefined() => c,
        _ => return,
    };
    let storage = match js_sys::Reflect::get(&chrome, &"storage".into()) {
        Ok(s) if !s.is_undefined() => s,
        _ => return,
    };
    let local = match js_sys::Reflect::get(&storage, &"local".into()) {
        Ok(l) if !l.is_undefined() => l,
        _ => return,
    };
    if let Ok(remove_fn) = js_sys::Reflect::get(&local, &"remove".into()) {
        if let Some(remove_fn) = remove_fn.dyn_ref::<js_sys::Function>() {
            if let Ok(promise) = remove_fn.call1(&local, &key.into()) {
                let _ = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await;
            }
        }
    }
}

/// dApp request pending approval
#[derive(Debug, Clone)]
pub struct DappRequest {
    pub request_id: String,
    pub origin: String,
    pub method: String,
    pub params: String,
}

/// Send a message to the background service worker via chrome.runtime.sendMessage
pub async fn send_to_background(method: &str, data: &serde_json::Value) -> Option<serde_json::Value> {
    let window = web_sys::window()?;
    let chrome = js_sys::Reflect::get(&window, &"chrome".into()).ok()?;
    if chrome.is_undefined() { return None; }
    let runtime = js_sys::Reflect::get(&chrome, &"runtime".into()).ok()?;
    if runtime.is_undefined() { return None; }
    let send_fn = js_sys::Reflect::get(&runtime, &"sendMessage".into()).ok()?;
    let send_fn = send_fn.dyn_ref::<js_sys::Function>()?;

    let msg = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&msg, &"target".into(), &"rusby-background".into());
    let _ = js_sys::Reflect::set(&msg, &"method".into(), &method.into());

    // Merge additional data fields into the message
    if let Some(obj) = data.as_object() {
        for (k, v) in obj {
            let js_val = match v {
                serde_json::Value::String(s) => wasm_bindgen::JsValue::from_str(s),
                serde_json::Value::Bool(b) => wasm_bindgen::JsValue::from_bool(*b),
                serde_json::Value::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        wasm_bindgen::JsValue::from_f64(f)
                    } else {
                        wasm_bindgen::JsValue::from_str(&n.to_string())
                    }
                }
                _ => wasm_bindgen::JsValue::from_str(&v.to_string()),
            };
            let _ = js_sys::Reflect::set(&msg, &k.as_str().into(), &js_val);
        }
    }

    let promise = send_fn.call1(&runtime, &msg).ok()?;
    let result = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await.ok()?;

    // Parse JsValue to serde_json::Value
    let json_str = js_sys::JSON::stringify(&result).ok()?.as_string()?;
    serde_json::from_str(&json_str).ok()
}

/// Get URL query parameter
pub fn get_url_param(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let href = window.location().href().ok()?;
    let url = web_sys::Url::new(&href).ok()?;
    let params = url.search_params();
    params.get(key)
}

async fn chrome_storage_get_all() -> Option<js_sys::Object> {
    let window = web_sys::window()?;
    let chrome = js_sys::Reflect::get(&window, &"chrome".into()).ok()?;
    if chrome.is_undefined() { return None; }
    let storage = js_sys::Reflect::get(&chrome, &"storage".into()).ok()?;
    if storage.is_undefined() { return None; }
    let local = js_sys::Reflect::get(&storage, &"local".into()).ok()?;
    if local.is_undefined() { return None; }
    let get_fn = js_sys::Reflect::get(&local, &"get".into()).ok()?;
    let get_fn = get_fn.dyn_ref::<js_sys::Function>()?;
    let promise = get_fn.call1(&local, &wasm_bindgen::JsValue::NULL).ok()?;
    let result = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise)).await.ok()?;
    result.dyn_into::<js_sys::Object>().ok()
}
