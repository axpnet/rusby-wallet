// Rusby Wallet — Theme system
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// Adding a new theme:
//   1. Create `crates/wallet-ui/src/theme/nome.rs` with `pub const THEME: &[(&str, &str)]`
//   2. Add `pub mod nome;` below and a new variant to `ThemeId`
//   3. Add i18n key `("theme.nome", "Display Name")` in all 9 locale files

pub mod default;
pub mod light;
pub mod midnight;
pub mod ocean;
pub mod forest;
pub mod atelier;
pub mod professional;

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::i18n::t;
use crate::state::{load_from_storage, save_to_storage};

/// All CSS custom properties managed by the theme system.
const THEME_VARS: &[&str] = &[
    "bg-primary", "bg-secondary", "bg-card", "bg-input",
    "text-primary", "text-secondary", "text-muted",
    "accent", "accent-hover", "accent-light",
    "success", "danger", "warning",
    "border", "shadow", "radius", "radius-sm",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ThemeId {
    Default,
    Light,
    Midnight,
    Ocean,
    Forest,
    Atelier,
    Professional,
    Custom,
}

impl ThemeId {
    pub fn code(&self) -> &'static str {
        match self {
            ThemeId::Default  => "default",
            ThemeId::Light    => "light",
            ThemeId::Midnight => "midnight",
            ThemeId::Ocean    => "ocean",
            ThemeId::Forest   => "forest",
            ThemeId::Atelier  => "atelier",
            ThemeId::Professional => "professional",
            ThemeId::Custom   => "custom",
        }
    }

    pub fn label_key(&self) -> &'static str {
        match self {
            ThemeId::Default  => "theme.default",
            ThemeId::Light    => "theme.light",
            ThemeId::Midnight => "theme.midnight",
            ThemeId::Ocean    => "theme.ocean",
            ThemeId::Forest   => "theme.forest",
            ThemeId::Atelier  => "theme.atelier",
            ThemeId::Professional => "theme.professional",
            ThemeId::Custom   => "theme.custom",
        }
    }

    /// Parse from localStorage value, with migration for old "dark"/"light" values.
    pub fn from_code(code: &str) -> Self {
        match code {
            "default" | "dark" => ThemeId::Default,
            "light"            => ThemeId::Light,
            "midnight"         => ThemeId::Midnight,
            "ocean"            => ThemeId::Ocean,
            "forest"           => ThemeId::Forest,
            "atelier"          => ThemeId::Atelier,
            "professional"     => ThemeId::Professional,
            "custom"           => ThemeId::Custom,
            _                  => ThemeId::Default,
        }
    }

    pub fn all() -> &'static [ThemeId] {
        &[
            ThemeId::Default, ThemeId::Light, ThemeId::Midnight,
            ThemeId::Ocean, ThemeId::Forest, ThemeId::Atelier,
            ThemeId::Professional, ThemeId::Custom,
        ]
    }

    pub fn palette(&self) -> &'static [(&'static str, &'static str)] {
        match self {
            ThemeId::Default  => default::THEME,
            ThemeId::Light    => light::THEME,
            ThemeId::Midnight => midnight::THEME,
            ThemeId::Ocean    => ocean::THEME,
            ThemeId::Forest   => forest::THEME,
            ThemeId::Atelier  => atelier::THEME,
            ThemeId::Professional => professional::THEME,
            ThemeId::Custom   => &[],
        }
    }

    pub fn preview_bg(&self) -> &'static str {
        match self {
            ThemeId::Default  => "#0f1117",
            ThemeId::Light    => "#f5f6fa",
            ThemeId::Midnight => "#0a0e1a",
            ThemeId::Ocean    => "#0b1622",
            ThemeId::Forest   => "#0d1a0f",
            ThemeId::Atelier  => "#12100f",
            ThemeId::Professional => "#0a0f1d",
            ThemeId::Custom   => "#1a1a2e",
        }
    }

    pub fn preview_accent(&self) -> &'static str {
        match self {
            ThemeId::Default  => "#6c5ce7",
            ThemeId::Light    => "#6c5ce7",
            ThemeId::Midnight => "#5b8def",
            ThemeId::Ocean    => "#00b4d8",
            ThemeId::Forest   => "#2d9f5a",
            ThemeId::Atelier  => "#d28c45",
            ThemeId::Professional => "#4299e1",
            ThemeId::Custom   => "#f39c12",
        }
    }

    pub fn is_dark(&self) -> bool {
        !matches!(self, ThemeId::Light)
    }
}

// ======== Apply theme to DOM ========

/// Set all CSS custom properties on `document.documentElement.style`.
pub fn apply_theme(theme_id: &ThemeId) {
    let style = match get_root_style() {
        Some(s) => s,
        None => return,
    };

    if *theme_id == ThemeId::Custom {
        let vars = load_custom_theme();
        for (key, value) in &vars {
            let prop = format!("--{}", key);
            let _ = style.set_property(&prop, value);
        }
    } else {
        for (key, value) in theme_id.palette() {
            let prop = format!("--{}", key);
            let _ = style.set_property(&prop, value);
        }
    }
}

fn get_root_style() -> Option<web_sys::CssStyleDeclaration> {
    let el = web_sys::window()?.document()?.document_element()?;
    let html = el.dyn_ref::<web_sys::HtmlElement>()?;
    Some(html.style())
}

// ======== Custom theme persistence ========

/// Load custom theme from localStorage, falling back to Default palette.
fn load_custom_theme() -> Vec<(String, String)> {
    let json = load_from_storage("custom_theme").unwrap_or_default();
    let map: std::collections::HashMap<String, String> =
        serde_json::from_str(&json).unwrap_or_default();

    THEME_VARS.iter().map(|key| {
        let value = map.get(*key).cloned().unwrap_or_else(|| {
            default::THEME.iter()
                .find(|(k, _)| *k == *key)
                .map(|(_, v)| v.to_string())
                .unwrap_or_default()
        });
        (key.to_string(), value)
    }).collect()
}

pub fn save_custom_theme(vars: &std::collections::HashMap<String, String>) {
    if let Ok(json) = serde_json::to_string(vars) {
        save_to_storage("custom_theme", &json);
    }
}

/// Load custom vars as HashMap for the editor, with Default fallback.
pub fn load_custom_vars_map() -> std::collections::HashMap<String, String> {
    let json = load_from_storage("custom_theme").unwrap_or_default();
    let saved: std::collections::HashMap<String, String> =
        serde_json::from_str(&json).unwrap_or_default();

    let mut map = std::collections::HashMap::new();
    for (key, val) in default::THEME {
        map.insert(key.to_string(), saved.get(*key).cloned().unwrap_or_else(|| val.to_string()));
    }
    map
}

// ======== Color helpers for custom theme derivation ========

fn parse_hex(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return None; }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

fn to_hex(r: u8, g: u8, b: u8) -> String {
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn lighten_hex(hex: &str, amount: i16) -> String {
    match parse_hex(hex) {
        Some((r, g, b)) => to_hex(
            (r as i16 + amount).clamp(0, 255) as u8,
            (g as i16 + amount).clamp(0, 255) as u8,
            (b as i16 + amount).clamp(0, 255) as u8,
        ),
        None => hex.to_string(),
    }
}

fn with_alpha(hex: &str, alpha: f32) -> String {
    match parse_hex(hex) {
        Some((r, g, b)) => format!("rgba({}, {}, {}, {:.2})", r, g, b, alpha),
        None => hex.to_string(),
    }
}

fn is_dark_color(hex: &str) -> bool {
    match parse_hex(hex) {
        Some((r, g, b)) => {
            let lum = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
            lum < 128.0
        }
        None => true,
    }
}

/// Build full 17-var theme from 8 user-selected colors.
pub fn build_full_custom_theme(
    bg_primary: &str,
    bg_secondary: &str,
    bg_card: &str,
    accent: &str,
    text_primary: &str,
    success: &str,
    danger: &str,
    border: &str,
) -> std::collections::HashMap<String, String> {
    let mut m = std::collections::HashMap::new();
    m.insert("bg-primary".into(), bg_primary.into());
    m.insert("bg-secondary".into(), bg_secondary.into());
    m.insert("bg-card".into(), bg_card.into());
    m.insert("bg-input".into(), lighten_hex(bg_card, 10));
    m.insert("text-primary".into(), text_primary.into());
    m.insert("text-secondary".into(), with_alpha(text_primary, 0.70));
    m.insert("text-muted".into(), with_alpha(text_primary, 0.45));
    m.insert("accent".into(), accent.into());
    m.insert("accent-hover".into(), lighten_hex(accent, 18));
    m.insert("accent-light".into(), with_alpha(accent, 0.15));
    m.insert("success".into(), success.into());
    m.insert("danger".into(), danger.into());
    m.insert("warning".into(), "#fbbf24".into());
    m.insert("border".into(), border.into());
    m.insert("shadow".into(),
        if is_dark_color(bg_primary) {
            "0 4px 24px rgba(0, 0, 0, 0.3)".into()
        } else {
            "0 4px 24px rgba(0, 0, 0, 0.08)".into()
        }
    );
    m.insert("radius".into(), "12px".into());
    m.insert("radius-sm".into(), "8px".into());
    m
}

// ======== UI Components ========

/// Theme selector grid for the Settings "Aspetto" section.
#[component]
pub fn ThemeSelector() -> impl IntoView {
    let theme: ReadSignal<ThemeId> = expect_context();
    let set_theme: WriteSignal<ThemeId> = expect_context();

    view! {
        <div class="card">
            <p class="text-sm text-muted mb-2">{move || t("settings.appearance")}</p>
            <div class="theme-hero">
                <div class="theme-hero-left">
                    <div class="theme-hero-kicker">{move || t("settings.appearance")}</div>
                    <div class="theme-hero-title">{move || t(theme.get().label_key())}</div>
                    <div class="theme-hero-meta">
                        <span class="theme-hero-dot"></span>
                        <span class="theme-hero-dot"></span>
                        <span class="theme-hero-dot"></span>
                    </div>
                </div>
                <div class="theme-hero-right">
                    <div class="theme-hero-card">
                        <div class="theme-hero-card-bar"></div>
                        <div class="theme-hero-card-line"></div>
                        <div class="theme-hero-card-line short"></div>
                    </div>
                    <div class="theme-hero-card muted">
                        <div class="theme-hero-card-bar"></div>
                        <div class="theme-hero-card-line"></div>
                        <div class="theme-hero-card-line short"></div>
                    </div>
                </div>
            </div>
            <div class="theme-grid">
                {ThemeId::all().iter().map(|tid| {
                    let tid = *tid;
                    view! {
                        <button
                            class="theme-tile"
                            style=move || format!(
                                "border-color: {};",
                                if theme.get() == tid { "var(--accent)" } else { "transparent" }
                            )
                            on:click=move |_| set_theme.set(tid)
                        >
                            <div style=format!(
                                "width: 100%; height: 32px; border-radius: 6px; \
                                 background: {}; position: relative; overflow: hidden;",
                                tid.preview_bg()
                            )>
                                <div style=format!(
                                    "position: absolute; bottom: 0; left: 0; right: 0; \
                                     height: 8px; background: {};",
                                    tid.preview_accent()
                                )></div>
                            </div>
                            <span style="font-size: 11px; font-weight: 500; color: var(--text-primary);">
                                {move || t(tid.label_key())}
                            </span>
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>

            {move || {
                if theme.get() == ThemeId::Custom {
                    Some(view! { <CustomThemeEditor /> })
                } else {
                    None
                }
            }}
        </div>
    }
}

/// Color picker editor for the Custom theme.
#[component]
fn CustomThemeEditor() -> impl IntoView {
    let initial = load_custom_vars_map();

    let get_val = |key: &str| -> String {
        let val = initial.get(key).cloned().unwrap_or_default();
        // Convert rgba/named values to hex for <input type="color">
        if val.starts_with('#') && val.len() == 7 { val } else {
            default::THEME.iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.to_string())
                .unwrap_or_else(|| "#000000".to_string())
        }
    };

    let (bg_primary, set_bg_primary) = signal(get_val("bg-primary"));
    let (bg_secondary, set_bg_secondary) = signal(get_val("bg-secondary"));
    let (bg_card, set_bg_card) = signal(get_val("bg-card"));
    let (accent, set_accent) = signal(get_val("accent"));
    let (text_primary, set_text_primary) = signal(get_val("text-primary"));
    let (success, set_success) = signal(get_val("success"));
    let (danger, set_danger) = signal(get_val("danger"));
    let (border, set_border) = signal(get_val("border"));

    // Live preview: apply on every change
    Effect::new(move |_| {
        let vars = build_full_custom_theme(
            &bg_primary.get(), &bg_secondary.get(), &bg_card.get(),
            &accent.get(), &text_primary.get(),
            &success.get(), &danger.get(), &border.get(),
        );
        // Apply directly
        if let Some(style) = get_root_style() {
            for (key, value) in &vars {
                let prop = format!("--{}", key);
                let _ = style.set_property(&prop, value);
            }
        }
        save_custom_theme(&vars);
    });

    let reset = move |_| {
        for (key, val) in default::THEME {
            match *key {
                "bg-primary" => set_bg_primary.set(val.to_string()),
                "bg-secondary" => set_bg_secondary.set(val.to_string()),
                "bg-card" => set_bg_card.set(val.to_string()),
                "accent" => set_accent.set(val.to_string()),
                "text-primary" => set_text_primary.set(val.to_string()),
                "success" => set_success.set(val.to_string()),
                "danger" => set_danger.set(val.to_string()),
                "border" => set_border.set(val.to_string()),
                _ => {}
            }
        }
    };

    view! {
        <div style="margin-top: 12px; display: grid; grid-template-columns: 1fr 1fr; gap: 8px;">
            <ColorPicker label="theme.bg_primary" value=bg_primary set_value=set_bg_primary />
            <ColorPicker label="theme.bg_secondary" value=bg_secondary set_value=set_bg_secondary />
            <ColorPicker label="theme.bg_card" value=bg_card set_value=set_bg_card />
            <ColorPicker label="theme.accent" value=accent set_value=set_accent />
            <ColorPicker label="theme.text_primary" value=text_primary set_value=set_text_primary />
            <ColorPicker label="theme.success" value=success set_value=set_success />
            <ColorPicker label="theme.danger" value=danger set_value=set_danger />
            <ColorPicker label="theme.border" value=border set_value=set_border />
        </div>
        <button
            class="btn btn-secondary btn-block mt-2"
            style="font-size: 12px; padding: 6px;"
            on:click=reset
        >
            {move || t("theme.reset_custom")}
        </button>
    }
}

#[component]
fn ColorPicker(
    label: &'static str,
    value: ReadSignal<String>,
    set_value: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <div style="display: flex; align-items: center; gap: 6px;">
            <input
                type="color"
                prop:value=move || value.get()
                on:input=move |ev| set_value.set(event_target_value(&ev))
                style="width: 28px; height: 28px; border: none; cursor: pointer; \
                       background: none; padding: 0; border-radius: 4px;"
            />
            <span style="font-size: 11px; color: var(--text-secondary);">
                {move || t(label)}
            </span>
        </div>
    }
}
