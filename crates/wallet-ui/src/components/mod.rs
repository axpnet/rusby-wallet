// Rusby Wallet — Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod navbar;
pub mod sidebar;
pub mod chain_selector;
pub mod address_display;
pub mod confirmation_modal;
pub mod security_warning;
pub mod error_boundary;
pub mod toast;
pub mod top_nav;
pub mod chain_sidebar;

/// Triple-arc animated spinner SVG for inline use in buttons (16×16px)
pub const SPINNER_SVG: &str = r#"<svg width="16" height="16" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path d="M12 2C6.48 2 2 6.48 2 12" stroke="currentColor" stroke-width="3" stroke-linecap="round"><animateTransform attributeName="transform" type="rotate" from="0 12 12" to="360 12 12" dur="0.7s" repeatCount="indefinite"/></path><path d="M12 2C6.48 2 2 6.48 2 12" stroke="currentColor" stroke-width="3" stroke-linecap="round" opacity="0.4"><animateTransform attributeName="transform" type="rotate" from="120 12 12" to="480 12 12" dur="0.7s" repeatCount="indefinite"/></path><path d="M12 2C6.48 2 2 6.48 2 12" stroke="currentColor" stroke-width="3" stroke-linecap="round" opacity="0.15"><animateTransform attributeName="transform" type="rotate" from="240 12 12" to="600 12 12" dur="0.7s" repeatCount="indefinite"/></path></svg>"#;
