// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

mod app;
mod state;
mod pages;
mod components;

use leptos::prelude::*;
use app::App;

fn main() {
    mount_to_body(App);
}
