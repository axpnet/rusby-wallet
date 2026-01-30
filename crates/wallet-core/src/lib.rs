// Rusby Wallet — Pure Rust multi-chain crypto wallet
// Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
// SPDX-License-Identifier: GPL-3.0-or-later
//
// wallet-core: crypto library with zero UI dependencies
//
// Modules:
//   bip39_utils  — Mnemonic generation & validation (12-24 words)
//   bip32_utils  — HD key derivation (secp256k1 + Ed25519 SLIP-10)
//   chains       — Address derivation per chain (EVM, Solana, TON, Cosmos)
//   crypto       — AES-256-GCM encrypt/decrypt with PBKDF2
//   wallet       — Multi-wallet manager (create, unlock, store)

pub mod bip39_utils;
pub mod bip32_utils;
pub mod chains;
pub mod crypto;
pub mod wallet;
