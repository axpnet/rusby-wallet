// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

mod app;
mod state;
mod pages;
mod components;
pub mod i18n;
pub mod theme;
pub mod rpc;
pub mod tx_send;
pub mod logging;

use leptos::prelude::*;
use app::App;

fn main() {
    components::error_boundary::install_panic_hook();
    mount_to_body(App);
}
