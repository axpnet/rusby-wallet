// Rusby Wallet — Lightweight WASM logging via console API
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Usage:
//   log_info!("Balance loaded: {}", balance);
//   log_warn!("RPC fallback for {}", chain);
//   log_error!("TX signing failed: {}", err);
//   log_debug!("Raw RPC response: {:?}", json);
//
// Debug logs are compiled out in release builds.

/// Log level for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub fn log(level: LogLevel, module: &str, message: &str) {
    let prefix = format!("[Rusby:{}]", module);
    let full = format!("{} {}", prefix, message);
    match level {
        LogLevel::Debug => web_sys::console::debug_1(&full.into()),
        LogLevel::Info => web_sys::console::log_1(&full.into()),
        LogLevel::Warn => web_sys::console::warn_1(&full.into()),
        LogLevel::Error => web_sys::console::error_1(&full.into()),
    }
}

macro_rules! log_info {
    ($($arg:tt)*) => {
        crate::logging::log(crate::logging::LogLevel::Info, module_path!(), &format!($($arg)*))
    };
}

macro_rules! log_warn {
    ($($arg:tt)*) => {
        crate::logging::log(crate::logging::LogLevel::Warn, module_path!(), &format!($($arg)*))
    };
}

macro_rules! log_error {
    ($($arg:tt)*) => {
        crate::logging::log(crate::logging::LogLevel::Error, module_path!(), &format!($($arg)*))
    };
}

macro_rules! log_debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        crate::logging::log(crate::logging::LogLevel::Debug, module_path!(), &format!($($arg)*))
    };
}

pub(crate) use log_info;
pub(crate) use log_warn;
pub(crate) use log_error;
pub(crate) use log_debug;
