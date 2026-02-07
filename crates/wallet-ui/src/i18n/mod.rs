// Rusby Wallet — Internationalization (i18n)
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later

pub mod en;
pub mod it;
pub mod es;
pub mod fr;
pub mod de;
pub mod pt;
pub mod zh;
pub mod ja;
pub mod ko;

use leptos::prelude::*;
use std::cell::Cell;

// Cache the current locale so t() works outside reactive contexts
// (e.g., inside gloo_timers::callback::Timeout callbacks).
thread_local! {
    static CACHED_LOCALE: Cell<Locale> = const { Cell::new(Locale::En) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    En,
    It,
    Es,
    Fr,
    De,
    Pt,
    Zh,
    Ja,
    Ko,
}

impl Locale {
    pub fn code(&self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::It => "it",
            Locale::Es => "es",
            Locale::Fr => "fr",
            Locale::De => "de",
            Locale::Pt => "pt",
            Locale::Zh => "zh",
            Locale::Ja => "ja",
            Locale::Ko => "ko",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Locale::En => "English",
            Locale::It => "Italiano",
            Locale::Es => "Espanol",
            Locale::Fr => "Francais",
            Locale::De => "Deutsch",
            Locale::Pt => "Portugues",
            Locale::Zh => "\u{4e2d}\u{6587}",
            Locale::Ja => "\u{65e5}\u{672c}\u{8a9e}",
            Locale::Ko => "\u{d55c}\u{ad6d}\u{c5b4}",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "it" => Locale::It,
            "es" => Locale::Es,
            "fr" => Locale::Fr,
            "de" => Locale::De,
            "pt" => Locale::Pt,
            "zh" => Locale::Zh,
            "ja" => Locale::Ja,
            "ko" => Locale::Ko,
            _ => Locale::En,
        }
    }

    pub fn all() -> &'static [Locale] {
        &[
            Locale::En,
            Locale::It,
            Locale::Es,
            Locale::Fr,
            Locale::De,
            Locale::Pt,
            Locale::Zh,
            Locale::Ja,
            Locale::Ko,
        ]
    }

    fn translations(&self) -> &'static [(&'static str, &'static str)] {
        match self {
            Locale::En => en::TRANSLATIONS,
            Locale::It => it::TRANSLATIONS,
            Locale::Es => es::TRANSLATIONS,
            Locale::Fr => fr::TRANSLATIONS,
            Locale::De => de::TRANSLATIONS,
            Locale::Pt => pt::TRANSLATIONS,
            Locale::Zh => zh::TRANSLATIONS,
            Locale::Ja => ja::TRANSLATIONS,
            Locale::Ko => ko::TRANSLATIONS,
        }
    }
}

/// Translate a key using the current locale from Leptos context.
/// Reactive: re-renders when locale signal changes (inside view closures).
/// Safe to call from timer callbacks and other non-reactive contexts
/// (falls back to cached locale instead of panicking).
pub fn t(key: &str) -> String {
    match use_context::<ReadSignal<Locale>>() {
        Some(signal) => {
            let current = signal.get();
            CACHED_LOCALE.with(|c| c.set(current));
            lookup(&current, key)
        }
        None => {
            // Outside reactive owner (gloo_timers::Timeout, Closure::wrap, etc.)
            let current = CACHED_LOCALE.with(|c| c.get());
            lookup(&current, key)
        }
    }
}

fn lookup(locale: &Locale, key: &str) -> String {
    locale
        .translations()
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v.to_string())
        .unwrap_or_else(|| {
            // Fallback to English
            if *locale != Locale::En {
                Locale::En
                    .translations()
                    .iter()
                    .find(|(k, _)| *k == key)
                    .map(|(_, v)| v.to_string())
                    .unwrap_or_else(|| key.to_string())
            } else {
                key.to_string()
            }
        })
}
