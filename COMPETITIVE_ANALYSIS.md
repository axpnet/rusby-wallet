# Competitive Analysis

**Version**: 0.9.0 (February 2026)
**Scope**: MetaMask, Rabby, Phantom, Keplr, Rusby Wallet

---

## 1. Market Overview

| | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---|---|---|---|---|---|
| Active users | ~30M | ~2M | ~7M | ~1.5M | Pre-launch |
| Chain focus | EVM | EVM | Solana + EVM + BTC | Cosmos IBC | Multi-chain (16) |
| Core language | JavaScript | JavaScript | Rust + TypeScript | TypeScript | Pure Rust |
| Open source | Yes | Yes | No | Yes | Yes (GPL-3.0) |
| Revenue model | Swap fees, staking | Swap fees | Swap fees, staking | Staking fees | None |
| First release | 2016 | 2022 | 2021 | 2020 | 2026 |
| Codebase size | ~500k+ LOC | ~200k+ LOC | Undisclosed | ~100k+ LOC | ~21k LOC |

---

## 2. Feature Comparison

### 2.1 Account and Key Management

| Feature | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---------|----------|-------|---------|-------|-------|
| Wallet creation (BIP-39) | Yes | Yes | Yes | Yes | Yes |
| Mnemonic import | Yes | Yes | Yes | Yes | Yes |
| Private key import | Yes | Yes | Yes | Yes | No |
| Multi-account (HD paths) | Yes | Yes | Yes | Yes | No |
| Multi-wallet | Yes | Yes | Yes | Yes | Yes |
| Hardware wallet (Ledger/Trezor) | Yes | Yes (+ GridPlus) | Yes (Ledger) | Yes (Ledger) | Planned (v1.0) |
| Watch-only address | Yes | Yes | No | No | No |
| Social recovery | No | No | No | No | Planned (v2.0) |

### 2.2 Blockchain Network Support

| Feature | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---------|----------|-------|---------|-------|-------|
| Supported chains | ~20 EVM | ~100+ EVM | SOL + EVM + BTC + Base | 50+ Cosmos | 16 (6 EVM + 10 non-EVM) |
| Custom network (RPC) | Yes | Yes | No | Yes | Planned |
| Testnet toggle | Yes | No | Yes | Yes | Yes (v0.7.0) |
| Auto-detect chain | No | Yes | Yes | N/A | No |
| Native EVM L2 | Yes | Yes | Yes | No | Yes |
| Non-EVM native | No (Snaps only) | No | Yes (SOL+BTC) | Yes (Cosmos) | Yes (SOL+TON+BTC+Cosmos+LTC+XLM+XRP+DOGE+TRX) |

**Rusby differentiator**: Native multi-chain support spanning EVM, Solana, TON, Cosmos, Bitcoin, Litecoin, Stellar, Ripple, Dogecoin, and TRON from a single seed phrase. No other wallet provides native derivation and signing across all these chain families without plugins or bridges.

### 2.3 Token and Asset Support

| Feature | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---------|----------|-------|---------|-------|-------|
| Native token balance | Yes | Yes | Yes | Yes | Yes |
| ERC-20 | Yes | Yes (auto-detect) | Yes | N/A | Yes |
| SPL (Solana) | No | No | Yes (auto-detect) | No | Yes |
| CW-20 (Cosmos) | No | No | No | Yes | Yes |
| Jetton (TON) | No | No | No | No | Yes |
| NFT (ERC-721/1155) | Yes | Yes | Yes | No | Yes (Alchemy API) |
| NFT (Metaplex/DAS) | No | No | Yes | No | Yes (Helius API) |
| Token auto-discovery | No (manual) | Yes | Yes | Yes | No (predefined list) |
| Price feed (USD) | Yes | Yes | Yes | Yes | Yes (CoinGecko) |
| Portfolio valuation | Yes | Yes | Yes | No | Yes |

### 2.4 Transaction Capabilities

| Feature | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---------|----------|-------|---------|-------|-------|
| Native send | Yes | Yes | Yes | Yes | Yes (all 16 chains) |
| Token send | Yes | Yes | Yes | Yes | Yes (ERC-20, CW-20, Jetton) |
| Transaction history | Yes | Yes | Yes | Yes | Yes |
| Gas estimation | Yes | Yes (advanced) | Yes | Yes | Yes |
| Speed up / cancel TX | Yes | Yes | No | No | No |
| Batch transactions | No | Yes | No | No | No |
| EIP-1559 Type 2 | Yes | Yes | Yes | N/A | Yes |
| Pre-signing simulation | No | Yes | Yes | No | Yes (eth_call) |
| UTXO management (BTC) | N/A | N/A | Yes | N/A | Yes (BTC, LTC, DOGE) |

### 2.5 dApp Integration

| Feature | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---------|----------|-------|---------|-------|-------|
| EIP-1193 provider | Yes | Yes | Yes (custom) | Yes (custom) | Yes |
| EIP-6963 discovery | Yes | Yes | No | No | Yes |
| WalletConnect v2 | Yes | Yes | Yes | Yes | Yes |
| Permission management | Yes | Yes (advanced) | Yes | Yes | Yes |
| EIP-191 (personal_sign) | Yes | Yes | Yes | N/A | Yes |
| EIP-712 (typed data) | Yes | Yes | Yes | N/A | Yes |
| Plugin / Snap system | Yes | No | No | No | No |

### 2.6 Security

| Feature | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---------|----------|-------|---------|-------|-------|
| Encryption cipher | AES-GCM | AES-GCM | Undisclosed | AES-GCM | AES-256-GCM |
| KDF algorithm | PBKDF2 | PBKDF2 | Undisclosed | scrypt | PBKDF2 (600k iter.) |
| Auto-lock | Yes | Yes | Yes | Yes | Yes |
| Phishing detection | Yes | Yes (advanced) | Yes | No | Yes (blocklist + heuristic) |
| TX simulation | No | Yes | Yes | No | Yes |
| Token approval management | No | Yes (revoke.cash) | No | N/A | Yes |
| Scam address warning | Yes | Yes | Yes | No | Yes |
| Core in Rust/WASM | No (JS) | No (JS) | Partial | No (TS) | Yes (100%) |
| Key zeroization | Unknown | Unknown | Unknown | Unknown | Yes (zeroize crate) |
| Overflow-safe parsing | Unknown | Unknown | Unknown | Unknown | Yes (checked arithmetic) |
| Debug redaction | Unknown | Unknown | Unknown | Unknown | Yes |

**Rusby differentiator**: 100% Rust cryptographic core eliminates entire classes of vulnerabilities inherent to JavaScript-based wallets, including prototype pollution, type coercion exploits, and timing side-channels from dynamic dispatch. All sensitive memory is explicitly zeroized after use, and numeric parsing uses checked arithmetic to prevent overflow attacks.

### 2.7 User Experience

| Feature | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---------|----------|-------|---------|-------|-------|
| Chrome extension | Yes | Yes | Yes | Yes | Yes |
| Firefox extension | Yes | Yes | No | Yes | Planned |
| Mobile application | Yes | No | Yes | Yes | Planned |
| Desktop application | No | No | No | No | Planned (Tauri) |
| DEX swap | Yes | Yes (multi-DEX) | Yes | No | Yes (0x API) |
| Cross-chain bridge | Yes | Yes | Yes | Yes (IBC) | No |
| Staking | Yes | No | Yes | Yes | No |
| Fiat on-ramp | Yes | No | Yes | No | No |
| Theme system | Yes (limited) | Yes | Yes | Yes | Yes (7 themes + custom) |
| Popup + fullpage layout | Yes | Yes | Yes | Yes | Yes |
| Internationalization | ~30 languages | ~10 languages | ~20 languages | ~10 languages | 9 languages |

---

## 3. SWOT Analysis

### Strengths

- **Pure Rust architecture**: Memory safety guarantees at the language level eliminate entire vulnerability classes common in JavaScript wallets (prototype pollution, type coercion, reference sharing). The Rust ownership model prevents use-after-free, double-free, and buffer overflow conditions by construction.
- **Native multi-chain breadth**: 16 blockchain networks with native key derivation and transaction signing from a single BIP-39 seed. No other open-source wallet supports EVM, Solana, TON, Cosmos, Bitcoin, Litecoin, Stellar, Ripple, Dogecoin, and TRON natively.
- **Compact, auditable codebase**: ~21,000 lines of code total versus ~500,000+ for MetaMask. The entire cryptographic core can be reviewed by a single auditor in a reasonable timeframe.
- **Transparent cryptography**: All encryption parameters (AES-256-GCM, PBKDF2 600k iterations, 32-byte salt) are documented and based on NIST and OWASP recommendations. Pure Rust implementations with no C/C++ FFI bindings.
- **Modern extension architecture**: Chrome Manifest v3 with strict Content Security Policy, service worker, and sandboxed WASM execution.
- **Open source (GPL-3.0)**: Full transparency with copyleft protection.

### Weaknesses

- **No established user base**: Pre-launch status with zero production users and no community.
- **Feature gaps**: Hardware wallet support, staking, cross-chain bridging, and fiat on-ramp are absent.
- **No formal third-party audit**: Internal and AI-assisted audits have been conducted, but no engagement with a recognized security firm (e.g., Trail of Bits, OpenZeppelin, Halborn).
- **Token discovery**: Relies on predefined token lists rather than automatic on-chain discovery.
- **Single-developer origin**: Resource constraints compared to funded teams of 50-200+ engineers at competing projects.

### Opportunities

- **Security-conscious user segment**: Growing demand for wallets with verifiable security properties, particularly in the aftermath of high-profile wallet exploits. "100% Rust, zero JavaScript" is a defensible differentiator.
- **TON ecosystem growth**: The TON blockchain ecosystem is expanding rapidly with few native wallet options. Rusby is among the first open-source wallets to support TON alongside EVM, Solana, and Cosmos.
- **Developer-oriented positioning**: Rust developers increasingly seek auditable wallet implementations they can verify independently.
- **Desktop application gap**: No mainstream wallet offers a native desktop client. Tauri-based deployment would be first-to-market.
- **Account abstraction (EIP-4337)**: No competitor has implemented native account abstraction. Early adoption could establish first-mover advantage in the next generation of wallet UX.

### Threats

- **MetaMask Snaps ecosystem**: MetaMask's plugin architecture is expanding multi-chain support, potentially neutralizing Rusby's chain breadth advantage.
- **Phantom expansion**: Phantom has expanded from Solana to EVM and Bitcoin, with further chain additions planned.
- **Browser-integrated wallets**: Brave Wallet and Opera Wallet reduce the need for extension-based wallets entirely.
- **Regulatory pressure**: MiCA and Travel Rule requirements may impose KYC obligations on self-custody wallets.
- **Smart wallets**: Account abstraction wallets (Coinbase Smart Wallet, Safe) with social recovery may render traditional HD wallets less attractive to mainstream users.
- **AI-assisted attacks**: Increasingly sophisticated phishing and social engineering attacks targeting wallet users require continuous security investment.

---

## 4. Strategic Positioning

### Segments to Avoid

- **Mass consumer market**: MetaMask (~30M users) and Phantom (~7M users) have network effects and brand recognition that are impractical to challenge directly.
- **dApp ecosystem integration**: MetaMask is the de facto standard for EVM dApp connectivity. Competing on ecosystem integration is not viable at this stage.
- **Mobile-first strategy**: Mobile wallet development requires substantial investment and competes with well-funded incumbents.

### Target Positioning: Security-First Multi-Chain Wallet

Rusby Wallet is positioned for users who prioritize:

1. **Verifiable security**: Users who want to inspect the cryptographic implementation and trust the wallet based on code review rather than brand reputation.
2. **Multi-chain consolidation**: Users managing assets across EVM, Solana, TON, Cosmos, and UTXO chains who want a single, unified interface without chain-specific wallets.
3. **Technical transparency**: Developers and technically literate users who value open-source, auditable code with documented security properties.
4. **Minimal attack surface**: Users concerned about JavaScript-based wallet vulnerabilities who seek a Rust/WASM alternative.

### Competitive Moat

The defensible advantage of Rusby Wallet is architectural: a pure Rust cryptographic core compiled to WebAssembly provides security properties that cannot be retrofitted onto a JavaScript codebase. This includes:

- Compile-time memory safety (ownership, borrowing, lifetime checking)
- Explicit error handling (no uncaught exceptions, no implicit type coercion)
- Deterministic resource management (no garbage collector non-determinism)
- Static type system preventing entire classes of runtime errors

In an industry where wallets manage billions of dollars in aggregate, these properties are not merely academic advantages; they represent a fundamental reduction in vulnerability surface area.

---

## 5. Development Trajectory

| Version | Status | Milestone |
|---------|--------|-----------|
| 0.1.0 | Released | Core wallet (BIP-39, HD derivation, AES-256-GCM encryption) |
| 0.2.0 | Released | Balance queries, transaction signing, QR codes |
| 0.3.0 | Released | Bitcoin P2WPKH, ERC-20/SPL tokens, portfolio USD, auto-lock |
| 0.4.0 | Released | EIP-1193/6963 provider, EIP-191/712 signing, Chrome extension architecture |
| 0.5.0 | Released | WalletConnect v2, CAIP-2 mapping |
| 0.6.0 | Released | TX simulation, phishing detection, scam addresses, token approvals |
| 0.7.0 | Released | i18n (9 languages), NFT, swap, CW-20, Jetton, testnet toggle, backup |
| 0.8.0 | Released | Theme system (7 + custom), external security review, WASM OOM fix |
| 0.9.0 | Released | Litecoin, Stellar, Ripple, Dogecoin, TRON (16 chains total) |
| 1.0.0 | Planned | Tauri desktop, hardware wallet integration, Argon2id migration |

---

## 6. Conclusion

Rusby Wallet occupies a unique position in the cryptocurrency wallet landscape: the only open-source wallet with a 100% Rust cryptographic core that provides native key derivation and transaction signing for 16 blockchain networks spanning 4 curve families (secp256k1 BIP-32, secp256k1 BIP-44, Ed25519 SLIP-10, and chain-specific encodings).

The feature gap with established competitors (hardware wallet support, staking, bridging, mobile applications) is significant but addressable. The architectural advantage -- compile-time memory safety, explicit resource management, and a compact auditable codebase -- is structural and cannot be replicated by competitors without fundamental rewrites.

The path to market viability requires three milestones: a formal third-party security audit to establish credibility, hardware wallet integration to meet institutional requirements, and a Tauri desktop application to differentiate from browser-only competitors. With these elements in place, Rusby can credibly position itself as the wallet of choice for security-conscious users managing assets across multiple blockchain ecosystems.
