// Rusby Wallet — Lightweight WASM logging via console API
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Usage:
//   log_info!("Balance loaded: {}", balance);
//   log_error!("TX signing failed: {}", err);

/// Log level for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Info,
    Error,
}

pub fn log(level: LogLevel, module: &str, message: &str) {
    let prefix = format!("[Rusby:{}]", module);
    let full = format!("{} {}", prefix, message);
    match level {
        LogLevel::Info => web_sys::console::log_1(&full.into()),
        LogLevel::Error => web_sys::console::error_1(&full.into()),
    }
}

macro_rules! log_info {
    ($($arg:tt)*) => {
        crate::logging::log(crate::logging::LogLevel::Info, module_path!(), &format!($($arg)*))
    };
}

macro_rules! log_error {
    ($($arg:tt)*) => {
        crate::logging::log(crate::logging::LogLevel::Error, module_path!(), &format!($($arg)*))
    };
}

pub(crate) use log_info;
pub(crate) use log_error;
