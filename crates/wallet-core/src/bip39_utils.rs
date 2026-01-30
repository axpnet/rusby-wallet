// Rusby Wallet — Pure Rust multi-chain crypto wallet
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// bip39_utils: BIP39 mnemonic generation, validation, and seed derivation
//
// Functions:
//   generate_mnemonic()  — Generate new mnemonic (12/15/18/21/24 words)
//   validate_mnemonic()  — Validate a mnemonic phrase
//   mnemonic_to_seed()   — Convert mnemonic + passphrase to 64-byte seed

use bip39::Mnemonic;

/// Word count options for mnemonic generation
#[derive(Debug, Clone, Copy)]
pub enum WordCount {
    W12 = 12,
    W15 = 15,
    W18 = 18,
    W21 = 21,
    W24 = 24,
}

/// Generate a new BIP39 mnemonic phrase
pub fn generate_mnemonic(word_count: WordCount) -> String {
    let mnemonic = Mnemonic::generate_in(bip39::Language::English, word_count as usize)
        .expect("Valid word count");
    mnemonic.to_string()
}

/// Validate a BIP39 mnemonic phrase
pub fn validate_mnemonic(phrase: &str) -> bool {
    Mnemonic::parse_in(bip39::Language::English, phrase).is_ok()
}

/// Convert mnemonic phrase to seed bytes (64 bytes) with optional passphrase
pub fn mnemonic_to_seed(phrase: &str, passphrase: &str) -> Result<[u8; 64], String> {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, phrase)
        .map_err(|e| format!("Invalid mnemonic: {}", e))?;
    let seed = mnemonic.to_seed(passphrase);
    Ok(seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_12_words() {
        let phrase = generate_mnemonic(WordCount::W12);
        let words: Vec<&str> = phrase.split_whitespace().collect();
        assert_eq!(words.len(), 12);
        assert!(validate_mnemonic(&phrase));
    }

    #[test]
    fn test_generate_24_words() {
        let phrase = generate_mnemonic(WordCount::W24);
        let words: Vec<&str> = phrase.split_whitespace().collect();
        assert_eq!(words.len(), 24);
        assert!(validate_mnemonic(&phrase));
    }

    #[test]
    fn test_validate_invalid() {
        assert!(!validate_mnemonic("invalid mnemonic phrase"));
    }

    #[test]
    fn test_mnemonic_to_seed() {
        let phrase = generate_mnemonic(WordCount::W12);
        let seed = mnemonic_to_seed(&phrase, "").unwrap();
        assert_eq!(seed.len(), 64);
        let seed2 = mnemonic_to_seed(&phrase, "").unwrap();
        assert_eq!(seed, seed2);
    }

    #[test]
    fn test_passphrase_changes_seed() {
        let phrase = generate_mnemonic(WordCount::W12);
        let seed1 = mnemonic_to_seed(&phrase, "").unwrap();
        let seed2 = mnemonic_to_seed(&phrase, "mypassphrase").unwrap();
        assert_ne!(seed1, seed2);
    }
}
