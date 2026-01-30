// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use wasm_bindgen::JsCast;

/// Application state
#[derive(Debug, Clone, PartialEq)]
pub enum AppPage {
    Onboarding,
    Login,
    Dashboard,
    Send,
    Receive,
    Settings,
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
    vec![
        ChainDisplay { id: "ethereum".into(), name: "Ethereum".into(), ticker: "ETH".into(), icon: "E".into() },
        ChainDisplay { id: "polygon".into(), name: "Polygon".into(), ticker: "POL".into(), icon: "P".into() },
        ChainDisplay { id: "bsc".into(), name: "BNB Chain".into(), ticker: "BNB".into(), icon: "B".into() },
        ChainDisplay { id: "optimism".into(), name: "Optimism".into(), ticker: "ETH".into(), icon: "O".into() },
        ChainDisplay { id: "base".into(), name: "Base".into(), ticker: "ETH".into(), icon: "Ba".into() },
        ChainDisplay { id: "arbitrum".into(), name: "Arbitrum".into(), ticker: "ETH".into(), icon: "A".into() },
        ChainDisplay { id: "solana".into(), name: "Solana".into(), ticker: "SOL".into(), icon: "S".into() },
        ChainDisplay { id: "ton".into(), name: "TON".into(), ticker: "TON".into(), icon: "T".into() },
        ChainDisplay { id: "cosmos".into(), name: "Cosmos Hub".into(), ticker: "ATOM".into(), icon: "C".into() },
        ChainDisplay { id: "osmosis".into(), name: "Osmosis".into(), ticker: "OSMO".into(), icon: "Os".into() },
        ChainDisplay { id: "bitcoin".into(), name: "Bitcoin".into(), ticker: "BTC".into(), icon: "B".into() },
    ]
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
