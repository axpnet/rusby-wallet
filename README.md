# Rusby Wallet

A multi-chain cryptocurrency wallet written entirely in Rust, compiled to WebAssembly for browser environments, Chrome extensions, and desktop applications (via Tauri). Rusby supports 16 blockchain networks, with all cryptographic operations executed locally in a sandboxed WASM runtime.

## Overview

Rusby Wallet implements hierarchical deterministic (HD) key derivation from a single BIP-39 mnemonic phrase, supporting both secp256k1 (BIP-32/BIP-44) and Ed25519 (SLIP-10) curves. The seed is encrypted at rest using AES-256-GCM with keys derived via PBKDF2-HMAC-SHA256 at 600,000 iterations, following OWASP 2024 guidelines for password-based key derivation.

The entire cryptographic core is implemented in pure Rust with no C/C++ bindings, ensuring memory safety guarantees and reproducible builds across all target platforms.

## Technology Stack

| Component | Technology | Purpose |
|-----------|-----------|---------|
| Cryptographic core | Pure Rust (k256, ed25519-dalek, aes-gcm, pbkdf2) | Key derivation, signing, encryption |
| Frontend framework | Leptos 0.7 (CSR) | Reactive UI compiled to WASM |
| Build system | Trunk | WASM compilation and asset bundling |
| Extension runtime | Chrome Manifest v3 | Service worker, content scripts, EIP-1193 provider |
| Desktop runtime | Tauri 2.0 (planned) | Native desktop application |
| Entropy source | Web Crypto API (via getrandom) | CSPRNG for key generation |

## Supported Blockchain Networks

Rusby supports 16 blockchain networks across 4 cryptographic curve families:

| Network | Curve | Derivation Path | Address Encoding |
|---------|-------|-----------------|------------------|
| Ethereum | secp256k1 | BIP-44 m/44'/60'/0'/0/0 | EIP-55 checksum hex |
| Polygon | secp256k1 | BIP-44 m/44'/60'/0'/0/0 | EIP-55 checksum hex |
| BNB Chain | secp256k1 | BIP-44 m/44'/60'/0'/0/0 | EIP-55 checksum hex |
| Optimism | secp256k1 | BIP-44 m/44'/60'/0'/0/0 | EIP-55 checksum hex |
| Base | secp256k1 | BIP-44 m/44'/60'/0'/0/0 | EIP-55 checksum hex |
| Arbitrum | secp256k1 | BIP-44 m/44'/60'/0'/0/0 | EIP-55 checksum hex |
| Bitcoin | secp256k1 | BIP-84 m/84'/0'/0'/0/0 | Bech32 P2WPKH (bc1...) |
| Litecoin | secp256k1 | BIP-84 m/84'/2'/0'/0/0 | Bech32 P2WPKH (ltc1...) |
| Dogecoin | secp256k1 | BIP-44 m/44'/3'/0'/0/0 | Base58Check P2PKH (D...) |
| Ripple (XRP) | secp256k1 | BIP-44 m/44'/144'/0'/0/0 | Base58Check Ripple (r...) |
| TRON | secp256k1 | BIP-44 m/44'/195'/0'/0/0 | Base58Check (T...) |
| Cosmos Hub | secp256k1 | BIP-44 m/44'/118'/0'/0/0 | Bech32 (cosmos1...) |
| Osmosis | secp256k1 | BIP-44 m/44'/118'/0'/0/0 | Bech32 (osmo1...) |
| Solana | Ed25519 | SLIP-10 m/44'/501'/0'/0' | Base58 |
| TON | Ed25519 | SLIP-10 m/44'/607'/0' | Base64url v4R2 |
| Stellar | Ed25519 | SLIP-10 m/44'/148'/0' | StrKey Base32 (G...) |

## Architecture

The project is organized into two Rust crates with a clear separation between cryptographic logic and user interface:

```
rusby-wallet/
  crates/
    wallet-core/                 # Cryptographic library (no UI dependencies)
      src/
        bip39_utils.rs           # Mnemonic generation and validation
        bip32_utils.rs           # HD key derivation (secp256k1 + Ed25519)
        crypto.rs                # AES-256-GCM encryption with PBKDF2 KDF
        wallet.rs                # Wallet lifecycle (create, unlock, derive)
        qr.rs                    # QR code generation (SVG output)
        backup.rs                # Encrypted wallet export/import
        caip.rs                  # CAIP-2 chain identifier mapping
        nft.rs                   # NFT data structures and URL sanitization
        swap.rs                  # DEX swap data structures
        chains/                  # Per-chain address derivation (16 chains)
        tokens/                  # Token standards (ERC-20, SPL, CW-20, Jetton)
        signing/                 # Message signing (EIP-191, EIP-712)
        security/                # Phishing detection, scam address database
        tx/                      # Transaction construction and signing (16 chains)
    wallet-ui/                   # Leptos frontend (compiled to WASM)
      src/
        app.rs                   # Root component, layout routing, context setup
        state.rs                 # Global state, storage helpers, chain display
        i18n/                    # Internationalization (9 languages, ~331 keys)
        theme/                   # Theme system (7 built-in + custom editor)
        pages/                   # UI pages (Dashboard, Send, Receive, Swap, NFT, etc.)
        components/              # Reusable components (Navbar, Toast, Security, etc.)
        rpc/                     # RPC clients (balance, broadcast, tokens, prices)
        tx_send/                 # Transaction dispatch with zeroization
  extension/                     # Chrome Extension (Manifest v3)
    background.js                # Service worker (message routing, permissions)
    content-script.js            # Web page to background bridge
    inpage.js                    # EIP-1193 provider (window.rusby) + EIP-6963
```

### Design Principles

- **Separation of concerns**: `wallet-core` contains zero UI dependencies and is independently testable with `cargo test`. All cryptographic operations reside in this crate.
- **Minimal trust surface**: Private keys exist in memory only during signing operations and are zeroized immediately after use.
- **Reactive state management**: Leptos signals with `provide_context`/`expect_context` pattern. All state reads use borrowing (`signal.with()`) rather than cloning to prevent WASM memory pressure.
- **Dual storage**: localStorage for synchronous reads in WASM, with automatic mirroring to `chrome.storage.local` for extension persistence across contexts.

## Security

Rusby implements defense-in-depth with multiple layers of protection. For a comprehensive security analysis, see [SECURITY.md](SECURITY.md).

Key security properties:

- **Seed encryption**: AES-256-GCM with PBKDF2-HMAC-SHA256 (600,000 iterations, 32-byte random salt, 12-byte random nonce)
- **Key zeroization**: All private keys and seed material are zeroized in memory immediately after use via the `zeroize` crate
- **Transaction simulation**: EVM transactions are simulated via `eth_call` before signing, with revert reason decoding
- **Phishing detection**: Domain blocklist, Levenshtein-distance typosquatting detection, suspicious TLD heuristics
- **Scam address database**: Known scam addresses, self-send detection, zero-address warnings
- **Content Security Policy**: Strict CSP for both web application and Chrome extension contexts
- **Origin validation**: All inter-context messages (background, content-script, popup) validate sender origin
- **Debug redaction**: Sensitive data structures implement custom `Debug` traits that redact cryptographic material
- **Overflow protection**: All numeric parsing for transaction amounts uses `checked_mul`/`checked_add` to prevent overflow

## Features

### Core Wallet

- HD wallet creation from BIP-39 mnemonic (12 or 24 words)
- Selective chain activation during wallet setup
- Real-time balance fetching with 30-second auto-refresh
- Portfolio value in USD via CoinGecko price feeds
- Transaction history across all supported chains
- QR code generation for receiving addresses
- Encrypted wallet backup export and import

### Token Support

- **ERC-20** (6 EVM chains): USDT, USDC, DAI, WETH, WBTC with custom token addition
- **SPL** (Solana): USDC, USDT, WSOL, JUP via Associated Token Accounts
- **CW-20** (Cosmos/Osmosis): CosmWasm token queries and transfers
- **Jetton** (TON): Token list with toncenter v3 API integration
- **IBC** (Cosmos/Osmosis): IBC denomination display

### DeFi Integration

- **DEX Swap**: Integrated swap via 0x API v2 for all 6 EVM chains with configurable slippage (0.3%-3%)
- **NFT Display**: EVM NFTs via Alchemy API v3, Solana NFTs via Helius DAS API
- **Token Approval Management**: Scan and revoke ERC-20 approvals for known DEX routers

### Web3 Connectivity

- **EIP-1193 Provider**: Full injected provider at `window.rusby` with `request()`, events, and legacy methods
- **EIP-6963**: Multi-provider discovery protocol support
- **WalletConnect v2**: Session management, proposal approval, cross-dApp signing
- **Message Signing**: EIP-191 (personal_sign) and EIP-712 (typed data) support

### User Experience

- **Dual layout**: Popup mode (420px, bottom navigation) and fullpage mode (sidebar + top navigation)
- **Internationalization**: 9 languages (EN, IT, ES, FR, DE, PT, ZH, JA, KO) with ~331 translation keys
- **Theme system**: 7 built-in themes + custom theme editor with 8 color pickers and live preview
- **Mainnet/Testnet toggle**: Dedicated testnet RPC endpoints for all supported networks
- **Address book**: Contact management with auto-completion in send flows

## Prerequisites

- Rust stable toolchain
- WebAssembly target: `rustup target add wasm32-unknown-unknown`
- Trunk build tool: `cargo install trunk`
- Node.js (optional, for WalletConnect bundle)

## Build Instructions

```bash
# Development server (port 3000, hot reload)
trunk serve

# Production build
trunk build --release

# Chrome extension build
bash build-extension.sh
# Output: extension-dist/ (load as unpacked extension in chrome://extensions)
```

## Testing

The project includes 176 unit tests covering key derivation, encryption, address encoding, transaction signing, token encoding, security modules, and data structure operations for all 16 supported chains.

```bash
# Run all tests
cargo test -p wallet-core

# Run with verbose output
cargo test -p wallet-core -- --nocapture

# Run specific test module
cargo test -p wallet-core chains::bitcoin
```

Test coverage includes:

| Module | Tests | Coverage Area |
|--------|-------|---------------|
| chains/ | 52 | Address derivation for 16 chains |
| tx/ | 37 | Transaction construction and signing |
| signing/ | 12 | EIP-191 and EIP-712 message signing |
| security/ | 12 | Phishing detection and scam address |
| tokens/ | 17 | ERC-20, SPL, CW-20, Jetton encoding |
| crypto | 4 | AES-256-GCM encrypt/decrypt |
| wallet | 3 | Wallet create/unlock lifecycle |
| backup | 7 | Export/import roundtrip |
| nft | 8 | NFT URL sanitization |
| swap | 7 | Swap amount parsing |
| caip | 7 | CAIP-2 chain mapping |
| qr | 3 | QR code generation |
| bip32/bip39 | 7 | HD key derivation |

## Metrics

- ~20,750 lines of Rust code
- ~769 lines of JavaScript (extension scripts)
- 176 unit tests, 0 failures
- 0 compiler warnings
- 16 blockchain networks
- 9 interface languages
- 7 built-in themes

## Project Status

Rusby Wallet is under active development. See [CHANGELOG.md](CHANGELOG.md) for version history and [SECURITY.md](SECURITY.md) for the complete security model.

Current version: **0.9.0**

## License

This project is licensed under the GNU General Public License v3.0 or later. See [LICENSE](LICENSE) for the full license text.

Copyright (C) 2025 axpnet & Claude Opus (Anthropic)
