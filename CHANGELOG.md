# Changelog

All notable changes to this project are documented in this file. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.9.0] - 2026-02-07

### Added

**Five new blockchain networks (14 to 16 supported chains):**

- **Litecoin**: BIP-84 derivation (m/84'/2'/0'/0/0), Bech32 P2WPKH encoding (ltc1...), full SegWit transaction support with dynamic fee estimation via litecoinspace.org API
- **Stellar**: SLIP-10 Ed25519 derivation (m/44'/148'/0'), StrKey Base32 encoding (G...), XDR binary transaction serialization with Ed25519 local signing, Horizon API integration
- **Ripple (XRP)**: BIP-44 secp256k1 derivation (m/44'/144'/0'/0/0), Base58Check with Ripple-specific alphabet (r...), custom binary serialization with field codes and variable-length encoding, SHA-512 Half hashing
- **Dogecoin**: BIP-44 secp256k1 derivation (m/44'/3'/0'/0/0), Base58Check P2PKH legacy encoding (D...), legacy sighash transaction format (distinct from Bitcoin P2WPKH), fixed 0.01 DOGE fee, Blockbook API
- **TRON**: BIP-44 secp256k1 derivation (m/44'/195'/0'/0/0), Keccak256-to-Base58Check encoding (T...), API-assisted transaction construction via TronGrid with local SHA256+secp256k1 signing (65 bytes r||s||v), pre-signing parameter verification

**Cross-cutting updates:**
- ChainId enum extended to 16 variants with mainnet and testnet configurations
- CAIP-2 identifier mapping for all new chains (Dogecoin: bip122:1a91e3..., TRON: tron:mainnet)
- CoinGecko price feed integration for all 16 chains
- Transaction dispatch, balance queries, and fee estimation extended to all chains
- Chain selector, onboarding, and send flow updated for 16-chain support
- Official chain icon integration (PNG) replacing text-based abbreviations

### Security

- Zeroize applied on all signing paths for new chains
- TRON: `verify_tron_tx_params()` validates API-returned transaction parameters before signing
- Overflow-safe amount parsing (checked_mul/checked_add) for all new chains

### Metrics

- 176 unit tests (from 123), 53 new tests across 5 chains
- ~20,750 lines of Rust, ~769 lines of JavaScript
- 0 compiler errors, 0 compiler warnings
- 40 new files, ~30 modified files

---

## [0.8.1] - 2026-02-07

### Added

**Fullpage layout (Talisman-style):**
- TopNav: horizontal navigation header (Home, Send, Receive, History, Settings) visible in fullpage mode
- ChainSidebar: vertical chain selector with icon, name, and per-chain balance
- Flex layout: header, sidebar (240px), and main content area
- FullpageMode newtype wrapper to resolve ReadSignal<bool> TypeId conflict in Leptos context
- Popup layout (420px, bottom navigation) remains unchanged

**Onboarding and login UX improvements:**
- SVG spinner animation inline in buttons during cryptographic operations
- Dynamic loading text with phase-by-phase progression (BIP-39 seed, PBKDF2 key stretching, HD key derivation)
- Corrected tab order (tabindex="-1" on password visibility toggles)
- Security badge in onboarding displaying encryption parameters (AES-256-GCM, PBKDF2 600k, local keys)

### Fixed

**WASM out-of-memory resolution:**
- Replaced all `signal.get()` calls on WalletState with `signal.with(|s| s.field)` to eliminate unnecessary cloning of HashMap and Vec structures
- Introduced Memo signals (`is_unlocked`, `active_chain`, `current_address`) to break reactive cascade chains (Effect to update to Effect)
- Relocated auto-refresh interval outside of reactive Effects to prevent interval leak (N overlapping intervals on each re-run)
- Timer callbacks switched to `signal.with_untracked()` for borrowing without reactive tracking

### Metrics

- 123 unit tests passing
- ~17,500 lines of Rust, ~769 lines of JavaScript
- 0 compiler errors

---

## [0.8.0] - 2026-02-06

### Added

**Theme system:**
- 7 built-in themes: Default (dark purple), Light, Midnight (blue), Ocean (teal), Forest (green), Atelier (warm orange), Professional (corporate blue)
- Custom theme editor with 8 color pickers, automatic secondary variable derivation, and live preview
- Animated hero preview card for theme selection
- Three-column grid selector with mini-preview (background + accent) per theme
- Architecture: CSS custom properties set via `element.style.setProperty()`, no additional CSS blocks per theme
- Automatic migration from legacy localStorage values ("dark"/"light") to new theme codes
- Developer-friendly schema: adding a theme requires one `.rs` file with 17 CSS variables, one enum variant, and one i18n key

**Internationalization:**
- 18 new theme-related translation keys across 9 languages
- Total: ~331 i18n keys

### Security

External security review findings (all remediated):
- **Critical**: Complete rewrite of TON v4R2 address derivation with BOC parser, cell hash computation, state_init hash, and address decoder
- **High**: Zeroize applied to seed and private key on all signing paths (Bitcoin, Solana, Cosmos, TON)
- **High**: Overflow protection (checked_mul/checked_add) in amount parsers for EVM, Solana, and TON
- **Medium**: Origin check on `wallet_switchEthereumChain` in background.js
- **Medium**: postMessage with explicit targetOrigin (replaced wildcard `'*'`)
- **Medium**: Restrictive `sanitize_image_url` (unknown schemes return empty string)
- **Medium**: package-lock.json committed for WalletConnect supply chain integrity

### Metrics

- 123 unit tests (from 118), 5 new tests for TON v4R2
- 8 new files, ~20 modified files

---

## [0.7.0] - 2026-02-05

### Added

**Internationalization (i18n):**
- 9 languages: English, Italian, Spanish, French, German, Portuguese, Chinese, Japanese, Korean
- ~313 translation keys with automatic English fallback
- Reactive `t("key")` function in Leptos components
- Language preference persisted in localStorage

**Utility features:**
- Mainnet/Testnet toggle with dedicated RPC endpoints per chain
- Address book with contact labels and auto-completion in send flow
- Encrypted wallet backup export/import (AES-256-GCM, full vault serialization with version and integrity checks)

**Advanced token support:**
- CW-20 (Cosmos/Osmosis): CosmWasm balance queries and MsgExecuteContract transfers via Amino JSON
- Jetton (TON): predefined token list, toncenter v3 API with runGetMethod fallback
- IBC token display with denomination hash on Cosmos and Osmosis
- Extended token dropdown in send flow for Cosmos, Osmosis, and TON

**NFT display:**
- EVM: Alchemy NFT API v3 (getNFTsForOwner) for Ethereum, Polygon, Base, Arbitrum, Optimism
- Solana: Helius DAS API (getAssetsByOwner)
- Two-column card grid with image, name, and collection
- Detail modal with full-size image, description, contract address, token ID, and standard
- Empty state with API key configuration hint
- URL sanitization for IPFS, Arweave, and HTTP-to-HTTPS conversion

**DEX swap:**
- 0x Swap API v2 integration for 6 EVM chains
- Predefined common tokens per chain (ETH, USDC, USDT, DAI, WETH, WBTC)
- Configurable slippage (0.3%, 0.5%, 1%, 3%)
- Quote display with exchange rate, estimated gas, and DEX source routing
- Transaction execution with 0x API calldata and standard EVM signing

**Toast notifications:**
- Four notification types: Success, Error, Warning, Info
- Auto-dismiss after 5 seconds with slide-in animation

**Dashboard:** four action buttons (Send, Receive, Swap, NFT)
**Settings:** API key management section (Alchemy, Helius, 0x)
**CSP:** updated img-src directive for NFT CDN domains

### Security

Internal security audit (v0.7.0) findings remediated:
- Hardcoded backup password replaced with user password modal
- Zeroize on seed_bytes and private_key in transaction signing modules
- JSON injection prevention via `validate_json_safe()` and `serde_json::json!()`
- Sender validation in background.js onMessage handler
- Cargo.lock committed for reproducible builds
- Integer overflow protection (checked_mul/checked_add) in token amount parsers
- Custom Debug implementation on EncryptedData with ciphertext redaction
- PBKDF2 iterations increased from 100,000 to 600,000 (OWASP 2024)
- RLP access_list encoding corrected (0xc0 for empty list)
- Salt and nonce length validation in decrypt()
- Gas price safety via saturating_mul(2)
- Network request timeout (30s) with AbortController and status code validation

### Metrics

- 118 unit tests (from 87), 31 new tests
- ~313 i18n keys across 9 languages

---

## [0.6.0] - 2026-02-02

### Added

**Transaction simulation:**
- Pre-signing simulation via `eth_call` for EVM chains
- Revert reason decoding (Error(string) selector 0x08c379a0)
- SecurityWarning component with three severity levels (Low, Medium, High)

**Phishing detection:**
- Domain blocklist (~50 known phishing domains)
- Typosquatting detection (Levenshtein distance <= 2 from ~22 legitimate domains)
- Suspicious TLD heuristics (.xyz, .tk, .ml, .ga, .cf, .gq, .top, .buzz, .icu)
- Character substitution pattern detection (e.g., un1swap, meta4ask)
- 7 unit tests

**Scam address detection:**
- Known scam address database
- Self-send and zero-address warnings
- Real-time risk assessment during address entry
- 5 unit tests

**Token approval management:**
- Scan ERC-20 approvals for known DEX routers (Uniswap V3, 1inch V5, PancakeSwap, SushiSwap, 0x)
- Support for Ethereum, Polygon, BSC, Arbitrum, Base
- One-click revoke via approve(spender, 0) transaction

**Code quality:**
- Send page refactored: transaction logic extracted to tx_send/ module (665 to 315 UI lines)
- Error boundary with WASM panic hook and DOM overlay with reload button
- Logging module with log_info! and log_error! macros

### Metrics

- 87 unit tests (from 73), 14 new tests

---

## [0.5.0] - 2026-02-02

### Added

**WalletConnect v2:**
- @walletconnect/web3wallet SDK bundled via esbuild (ESM output)
- RusbyWC wrapper with chrome.storage adapter for service worker persistence
- Service worker keep-alive via chrome.alarms (24-second interval)
- Session proposal popup with chain display and approve/reject flow
- Session request handling via approval queue
- CAIP-2 to ChainId bidirectional mapping (eip155, solana, cosmos, bip122, ton) with 7 tests
- WalletConnect page: URI input, active session list, disconnect
- Full message signing in popup (password modal, on-demand seed decrypt, personal_sign + EIP-712)
- Namespace builder for WalletConnect sessions (methods + events per namespace)
- WalletConnect Project ID configuration in Settings

### Metrics

- 73 unit tests (from 66), 7 new tests for CAIP-2

---

## [0.4.0] - 2026-02-02

### Added

**Chrome extension architecture (three-context model):**

- **Background service worker**: message routing between popup, content script, and dApps; centralized lock state management; persistent request queue via chrome.storage; per-origin permission management
- **Content script**: bidirectional bridge between web page and background via long-lived port; inpage.js injection; message filtering
- **Injected provider (EIP-1193)**: `window.rusby` with `request({method, params})`, events (connect, disconnect, chainChanged, accountsChanged), methods (eth_requestAccounts, eth_accounts, eth_chainId, eth_sendTransaction, personal_sign, eth_signTypedData_v4, wallet_switchEthereumChain), legacy compatibility (enable, send, sendAsync)

**EIP-6963 Multi-Provider Discovery:**
- Provider announcement with UUID, name, SVG icon, RDNS (io.rusby.wallet)
- Listener for eip6963:requestProvider with re-announcement

**Message signing:**
- EIP-191 (personal_sign): Ethereum prefix + keccak256 + secp256k1 ECDSA, address recovery from signature (6 tests)
- EIP-712 (typed structured data): domain separator, struct hash, sign_typed_data_hash, hash_eip712_domain() (6 tests)

**dApp approval UI:**
- Approval page with origin, method, and parameters display
- Approve/Reject with user feedback
- Automatic popup opening via URL parameter (?approve=requestId)
- Connected dApps management in Settings with per-origin revocation

### Security

- Private keys never exposed outside the popup WASM context
- Background service worker handles routing and session state only
- Granular per-origin dApp permissions

### Metrics

- 66 unit tests (from 54), 12 new tests for EIP-191/712

---

## [0.3.0] - 2026-02-02

### Added

**Bitcoin P2WPKH:**
- BIP-84 address derivation (m/84'/0'/0'/0/0) with Bech32 encoding (bc1...)
- SegWit transaction signing with BIP-143 sighash, DER signature encoding, and witness serialization
- mempool.space API integration: balance, UTXO fetch, fee estimation, transaction broadcast

**ERC-20 tokens (6 EVM chains):**
- Predefined tokens: USDT, USDC, DAI, WETH, WBTC
- ABI encoding for balanceOf and transfer
- Token balance display in dashboard

**SPL tokens (Solana):**
- Predefined tokens: USDC, USDT, WSOL, JUP
- getTokenAccountsByOwner RPC integration
- Associated Token Account (ATA) derivation

**Transaction history:**
- EVM: Etherscan-compatible API for 6 chains
- Solana: getSignaturesForAddress
- TON: toncenter /getTransactions
- Cosmos: LCD /cosmos/tx/v1beta1/txs
- History page with direction, amount, and block explorer links

**Portfolio:**
- USD portfolio valuation via CoinGecko API
- Per-chain USD equivalent display
- Price caching in localStorage with 60-second refresh

**Password strength validation:**
- PasswordStrength enum (Weak/Fair/Strong) with color indicator
- Wallet creation blocked on Weak passwords

**Auto-lock:**
- Configurable inactivity timeout (1, 5, 15, 30 minutes)
- Timer reset on user interaction (click, keypress)
- Settings toggle (default: off)

### Security

- Cosmos address derivation corrected: SHA256 truncation replaced with RIPEMD160(SHA256(pubkey)) per Cosmos SDK specification
- Zeroize applied to decrypted seed in encrypt(), decrypt(), create_wallet(), and unlock_wallet()

### Metrics

- 54 unit tests (from 41), 13 new tests (Bitcoin: 9, ERC-20: 4)

---

## [0.2.0] - 2026-01-30

### Added

**Real-time balance queries:**
- RPC client module with per-chain implementations
- EVM: eth_getBalance JSON-RPC
- Solana: getBalance JSON-RPC
- TON: getAddressBalance via toncenter REST
- Cosmos/Osmosis: /cosmos/bank/v1beta1/balances REST
- 30-second auto-refresh with loading indicator

**Transaction signing and broadcast:**
- Per-chain transaction construction and signing
- EVM: EIP-1559 Type 2 with RLP encoding and secp256k1 ECDSA
- Solana: SystemProgram transfer with Ed25519
- TON: Wallet v4R2 external message with Ed25519
- Cosmos: MsgSend Amino JSON with secp256k1 ECDSA
- Automatic gas estimation via RPC
- Confirmation modal with transaction summary and password re-entry
- On-demand seed decryption for signing (never persistent)

**QR code generation:**
- Pure Rust QR code generation via qrcode crate
- SVG inline rendering in Receive page

### Dependencies Added

- qrcode 0.14 (QR generation)
- rlp 0.6 (RLP encoding for EVM transactions)
- gloo-net 0.6 (WASM-compatible HTTP)
- hex 0.4, bs58 0.5 (encoding utilities)

### Metrics

- 41 unit tests (from 28), 13 new tests

---

## [0.1.0] - 2026-01-30

### Added

**wallet-core library:**
- BIP-39 mnemonic generation and validation (12 or 24 words)
- BIP-32/BIP-44 HD key derivation (secp256k1) and SLIP-10 (Ed25519)
- Address derivation for 11 chains: Ethereum, Polygon, BSC, Optimism, Base, Arbitrum, Solana, TON, Bitcoin, Cosmos Hub, Osmosis
- AES-256-GCM encryption with PBKDF2 key derivation (100,000 iterations)
- Wallet manager: create and unlock encrypted wallets

**wallet-ui frontend:**
- Leptos 0.7 CSR compiled to WebAssembly
- Three-step onboarding wizard (generate/import seed, set password, confirm)
- Login with encrypted wallet unlock
- Dashboard with balance display, action buttons, and chain selector
- Send page (placeholder)
- Receive page with address display and clipboard copy
- Dark/light theme toggle
- Popup layout (420px, bottom navigation) for Chrome extension
- Fullpage layout (sidebar, chain grid) for web application
- Automatic extension/web context detection
- Popup-to-fullpage expansion button
- chrome.storage.local support with localStorage synchronization

**Chrome extension:**
- Manifest v3 with CSP for WebAssembly
- Automated build script (build-extension.sh)
- Auto-generated SVG placeholder icons

### Security

- Seed encrypted with AES-256-GCM before persistence
- Password never persisted (memory-only during session)
- PBKDF2 with 100,000 iterations for key derivation

### Metrics

- 28 unit tests passing
