# Security Policy

## Table of Contents

1. [Security Model](#security-model)
2. [Cryptographic Primitives](#cryptographic-primitives)
3. [Key Management Lifecycle](#key-management-lifecycle)
4. [Transaction Security](#transaction-security)
5. [Extension Security](#extension-security)
6. [Application Security](#application-security)
7. [Threat Model](#threat-model)
8. [Dependency Audit](#dependency-audit)
9. [Security Hardening Roadmap](#security-hardening-roadmap)
10. [Vulnerability Disclosure](#vulnerability-disclosure)
11. [Supported Versions](#supported-versions)

---

## Security Model

Rusby Wallet adopts a **zero-trust local** security model. The fundamental invariants are:

1. **The seed never exists in plaintext on disk.** The BIP-39 mnemonic is encrypted with AES-256-GCM before any persistence operation.
2. **The password is never persisted.** The user's password exists only in the WASM linear memory during decryption and is discarded immediately after key derivation.
3. **Private keys are ephemeral.** Signing keys are derived on-demand for each transaction, used once, and zeroized before the function returns.
4. **All cryptography executes locally.** No private key material is ever transmitted over the network. Remote services are used only for balance queries, transaction broadcast, and price feeds.

### Trust Boundaries

```
 Untrusted                    Trust Boundary                    Trusted
+-----------+           +---------------------+           +----------------+
| Web pages |  <---->   | Content Script      |  <---->   | Background SW  |
| dApps     |           | (message relay)     |           | (routing only) |
+-----------+           +---------------------+           +----------------+
                                                                  |
                                                                  v
                                                          +----------------+
                                                          | Popup (WASM)   |
                                                          | - Crypto ops   |
                                                          | - Key mgmt     |
                                                          | - TX signing   |
                                                          +----------------+
```

All cryptographic operations occur exclusively within the popup WASM context. The background service worker handles message routing and session state only; it never has access to private keys or seed material.

---

## Cryptographic Primitives

### Seed Encryption (At Rest)

| Parameter | Value | Reference |
|-----------|-------|-----------|
| Cipher | AES-256-GCM | NIST SP 800-38D |
| Key derivation | PBKDF2-HMAC-SHA256 | RFC 8018, NIST SP 800-132 |
| Iterations | 600,000 | OWASP 2024 recommendation |
| Salt length | 32 bytes (random) | NIST SP 800-132 Section 5.1 |
| Nonce length | 12 bytes (random) | NIST SP 800-38D Section 8.2 |
| Authentication tag | 128 bits | GCM default |

Each encryption operation generates a fresh random salt and nonce. Encrypting the same plaintext twice produces entirely different ciphertext, preventing ciphertext comparison attacks.

**Implementation**: `crates/wallet-core/src/crypto.rs` (crate `aes-gcm 0.10`, `pbkdf2 0.12`)

### Input Validation

The decryption routine validates structural integrity before attempting decryption:
- Salt length must be exactly 32 bytes
- Nonce length must be exactly 12 bytes
- Invalid lengths result in an immediate error return, preventing oracle attacks on malformed input

### HD Key Derivation

Rusby implements two distinct derivation schemes:

**secp256k1 chains (BIP-32/BIP-44):**
- Master key derived from BIP-39 seed via HMAC-SHA512 with key `"Bitcoin seed"`
- Child key derivation follows BIP-32 with hardened derivation for purpose, coin type, and account levels
- Implementation: crate `bip32 0.5`, `k256 0.13`
- Chains: Ethereum, Polygon, BSC, Optimism, Base, Arbitrum, Bitcoin, Litecoin, Dogecoin, Ripple, TRON, Cosmos Hub, Osmosis

**Ed25519 chains (SLIP-10):**
- Master key derived via HMAC-SHA512 with key `"ed25519 seed"`
- All derivation levels use hardened indices (Ed25519 does not support public parent to public child derivation)
- Implementation: crate `ed25519-dalek 2`, `curve25519-dalek 4`
- Chains: Solana, TON, Stellar

### Entropy Source

In WebAssembly environments, cryptographic randomness is sourced from `crypto.getRandomValues()` (Web Crypto API) via the `getrandom` crate with the `js` feature. This provides access to the browser's cryptographically secure pseudorandom number generator (CSPRNG), conforming to the W3C Web Crypto specification.

---

## Key Management Lifecycle

### Phase 1: Wallet Creation

```
User password  -->  PBKDF2 (600k iterations, 32B salt)  -->  AES-256 key
BIP-39 mnemonic -->  Seed (64 bytes)  -->  AES-256-GCM encrypt  -->  Vault (persisted)
                                                                       |
                                                                       v
                                                              salt + nonce + ciphertext
```

After encryption, the plaintext seed and derived AES key are zeroized in memory.

### Phase 2: Wallet Unlock

```
Vault (stored)  -->  salt + nonce + ciphertext
User password   -->  PBKDF2 (same salt)  -->  AES key  -->  Decrypt  -->  Seed
Seed  -->  Derive addresses for all enabled chains  -->  Populate UI state
Seed  -->  zeroize()
```

The seed is held in memory only long enough to derive public addresses. After derivation, the seed is zeroized. Addresses (public information) remain in the UI state.

### Phase 3: Transaction Signing

```
User confirms TX  -->  Password modal  -->  PBKDF2  -->  Decrypt vault  -->  Seed
Seed  -->  BIP-32/SLIP-10 derive private key for target chain
Private key  -->  Sign transaction  -->  Broadcast to network
Private key  -->  zeroize()
Seed  -->  zeroize()
```

This on-demand decryption model ensures that private keys never persist in memory between user actions.

### Phase 4: Auto-Lock

After a configurable inactivity timeout (1, 5, 15, or 30 minutes), the wallet state is reset to its default (locked) state. All derived data, including addresses and balances, is cleared from memory.

### Zeroization Coverage

The `zeroize` crate is used to overwrite sensitive data in memory before deallocation. Zeroization is applied on:

| Context | Variables Zeroized |
|---------|-------------------|
| Transaction signing (all 16 chains) | `seed_bytes`, `private_key`, intermediate key material |
| Wallet unlock | Decrypted seed after address derivation |
| Wallet creation | Seed bytes after encryption |
| Backup export/import | Decrypted vault data |

**Limitation**: In WebAssembly, the linear memory is managed by the WASM runtime. While `zeroize` overwrites the Rust-owned memory, the WASM engine's garbage collection may retain copies in uncontrolled memory regions. This is an inherent limitation of the WebAssembly execution model (see [Hardening Roadmap](#security-hardening-roadmap)).

---

## Transaction Security

### Pre-Signing Simulation (EVM)

Before presenting the confirmation dialog for EVM transactions, Rusby executes an `eth_call` simulation on the current RPC endpoint. If the simulation indicates that the transaction would revert on-chain, a security warning is displayed with the decoded revert reason (selector `0x08c379a0` for `Error(string)`).

**Implementation**: `crates/wallet-ui/src/rpc/simulate.rs`

### TRON Transaction Verification

TRON transactions use an API-assisted model where TronGrid constructs the transaction object. Before signing, Rusby performs parameter verification (`verify_tron_tx_params()`) to ensure the API-returned transaction matches the user's intended recipient, amount, and contract parameters.

**Implementation**: `crates/wallet-core/src/tx/tron.rs`

### Overflow Protection

All transaction amount parsing functions use `checked_mul` and `checked_add` for decimal-to-base-unit conversion (e.g., ETH to Wei, BTC to Satoshi). This prevents integer overflow attacks where a carefully crafted amount string could wrap around and send an unintended value.

Affected parsers:
- `tx/evm.rs`: `parse_ether_to_wei()`
- `tokens/erc20.rs`: `parse_token_amount()`
- `tokens/cw20.rs`: `parse_token_amount()`
- `tx/bitcoin.rs`: `parse_btc_to_satoshi()`
- `tx/solana.rs`: `parse_sol_to_lamports()`
- `tx/ton.rs`: `parse_ton_to_nanoton()`
- `tx/litecoin.rs`: `parse_ltc_to_litoshi()`
- `tx/stellar.rs`: `parse_xlm_to_stroops()`
- `tx/ripple.rs`: `parse_xrp_to_drops()`
- `tx/dogecoin.rs`: `parse_doge_to_satoshi()`
- `tx/tron.rs`: `parse_trx_to_sun()`

### RLP Encoding (EVM)

EVM transaction signing uses RLP (Recursive Length Prefix) encoding following the Ethereum Yellow Paper specification. The implementation correctly distinguishes between empty byte strings (`0x80`) and empty lists (`0xc0`), with the `access_list` field properly encoded as an empty list for EIP-1559 Type 2 transactions.

### Gas Price Safety

Gas price estimation for EVM transactions uses `saturating_mul(2)` to apply a 2x safety multiplier, preventing potential overflow in gas calculation and ensuring transactions are not rejected due to underpriced gas.

---

## Extension Security

### Content Security Policy

The Chrome extension enforces a strict Content Security Policy:

```
default-src 'self';
script-src 'self' 'wasm-unsafe-eval';
connect-src https:;
style-src 'self' 'unsafe-inline';
img-src 'self' data: [whitelisted NFT CDN domains];
```

The `wasm-unsafe-eval` directive is required by Chrome Manifest v3 for WebAssembly execution. All scripts are loaded from the extension bundle; no inline scripts or remote scripts are permitted.

### Message Validation

All messages received by the background service worker validate the sender:

```javascript
// background.js — sender validation
if (sender.id !== chrome.runtime.id) return;
```

Messages from content scripts include origin validation to prevent cross-origin message injection:

```javascript
// Verified: sender.tab.url origin matches stored approval
```

### Inter-Context Communication

- `postMessage` calls use explicit `targetOrigin` (the page's own origin) instead of wildcard `'*'`
- Content script to background communication uses Chrome's long-lived port connections with sender verification
- The injected provider (`window.rusby`) communicates exclusively through the content script relay

### Network Request Timeouts

All outbound HTTP requests (RPC calls, API queries) enforce a 30-second timeout via `AbortController`. Response status codes are validated before processing. This prevents:
- Denial-of-service from unresponsive RPC endpoints
- Resource exhaustion from hanging connections in the WASM runtime

**Implementation**: `crates/wallet-ui/src/rpc/mod.rs` (`post_json()`, `get_json()`)

---

## Application Security

### Phishing Detection

Every dApp connection request (EIP-1193 or WalletConnect) is evaluated against multiple detection layers:

1. **Domain blocklist**: ~50 known phishing domains (e.g., `uniswap-airdrop.com`, `metamask-io.com`)
2. **Typosquatting detection**: Levenshtein distance <= 2 from ~22 legitimate domains (e.g., `uniswap.org`, `metamask.io`)
3. **Suspicious TLD heuristics**: Domains using `.xyz`, `.tk`, `.ml`, `.ga`, `.cf`, `.gq`, `.top`, `.buzz`, `.icu` with cryptocurrency-related keywords
4. **Character substitution patterns**: Detection of digit/letter swaps (e.g., `un1swap`, `meta4ask`)

Flagged domains display a non-dismissable red warning banner in the approval page. The user may still choose to proceed.

**Implementation**: `crates/wallet-core/src/security/phishing.rs` (7 tests)

### Scam Address Detection

When entering a recipient address in the send flow, real-time validation checks:

1. **Known scam addresses**: Database of confirmed scam and burn addresses
2. **Self-send detection**: Warning when the recipient matches the sender
3. **Zero-address detection**: Warning for `0x0000...0000` (funds permanently lost)

**Implementation**: `crates/wallet-core/src/security/scam_addresses.rs` (5 tests)

### Token Approval Management

A dedicated page allows users to scan and revoke ERC-20 token approvals for known DEX router contracts (Uniswap V3, 1inch V5, PancakeSwap, SushiSwap, 0x Exchange Proxy) across Ethereum, Polygon, BSC, Arbitrum, and Base. Unlimited approvals are visually highlighted as a security concern.

### NFT Image Sanitization

NFT image URLs undergo sanitization before rendering:
- IPFS URIs (`ipfs://`) are converted to HTTPS gateway URLs
- Arweave URIs are converted to `arweave.net` HTTPS URLs
- HTTP URLs are upgraded to HTTPS
- Unknown or unsupported schemes return an empty string (preventing protocol-level attacks)

**Implementation**: `crates/wallet-core/src/nft.rs` — `sanitize_image_url()`

### Debug Information Redaction

The `EncryptedData` struct implements a custom `Debug` trait that redacts the ciphertext field. This prevents accidental exposure of encrypted vault data in log messages, panic backtraces, or debug output.

### Internationalization Security

All translation strings are compiled as static `&[(&str, &str)]` arrays directly into the WASM binary. Translations are never loaded from external sources at runtime, eliminating the risk of injection via malicious translation files.

### WASM Memory Management

To prevent out-of-memory conditions in the WASM linear memory:
- All reactive state reads use `signal.with(|s| ...)` (borrowing) instead of `signal.get()` (cloning) to avoid unnecessary allocation of large data structures
- Memoized signals (`Memo`) break reactive cascades, preventing O(n) re-computation chains
- Timer-based operations (auto-refresh, auto-lock) use single-instance patterns to prevent interval leaks

---

## Threat Model

### Threats Mitigated

| Threat | Mitigation | Strength |
|--------|-----------|----------|
| Vault theft (stolen device/backup) | AES-256-GCM + PBKDF2 600k iterations | High: computationally infeasible without password |
| Password brute force | 600,000 PBKDF2 iterations (~60s per attempt) | High: ~10^6 guesses/day on consumer hardware |
| Ciphertext replay/comparison | Random salt + nonce per encryption | High: identical plaintexts produce different ciphertexts |
| Vault tampering | GCM authentication tag (128-bit) | High: any modification detected before decryption |
| Key residue in memory | Zeroize on all signing paths | Medium: effective within Rust-controlled memory |
| Phishing dApp connections | Multi-layer domain analysis | Medium: covers common attack vectors |
| Scam recipient addresses | Known address database + heuristics | Medium: growing database |
| Transaction simulation failure | Pre-signing eth_call + revert decoding | Medium: depends on RPC endpoint availability |
| ERC-20 unlimited approvals | Approval scanning + one-click revoke | High: direct on-chain revocation |
| Integer overflow in amounts | checked_mul/checked_add throughout | High: compile-time enforced |
| XSS in extension context | Manifest v3 CSP, no inline scripts | High: browser-enforced |
| Cross-origin message injection | Sender ID and origin validation | High: verified at runtime |
| Malicious NFT image URIs | URL sanitization + CSP img-src whitelist | Medium: defense-in-depth |

### Known Limitations

| Limitation | Description | Planned Mitigation |
|-----------|-------------|-------------------|
| WASM memory model | WebAssembly linear memory may retain copies of zeroized data due to engine-level optimizations | Investigate `wasm-bindgen` memory fencing; Tauri desktop build with native memory control |
| Password strength dependency | Vault security is bounded by password entropy | Password strength meter (implemented); consider Argon2id migration |
| Extension isolation | Other Chrome extensions with broad permissions could potentially read localStorage | Investigate `chrome.storage.session` for ephemeral secrets |
| Supply chain risk | Not all Rust crate dependencies have formal security audits | `cargo-audit` integration in CI; `cargo-vet` for supply chain verification |
| RPC endpoint trust | Balance queries and transaction broadcasts trust the configured RPC endpoint | Multi-RPC verification for critical operations (planned) |
| 0x API trust | Swap calldata is provided by an external service | User-facing calldata display; contract address verification |
| Memory dump attacks | A malicious process with sufficient privileges could read WASM memory | Out of scope for browser extension; mitigated in Tauri desktop via OS-level protections |

---

## Dependency Audit

All cryptographic dependencies are pure Rust implementations from the RustCrypto project or established Rust cryptography libraries:

| Crate | Version | Purpose | Audit Status |
|-------|---------|---------|-------------|
| aes-gcm | 0.10 | Authenticated encryption | RustCrypto; NCC Group audit (2019) |
| pbkdf2 | 0.12 | Password-based key derivation | RustCrypto |
| k256 | 0.13 | secp256k1 ECDSA | RustCrypto; Trail of Bits audit |
| ed25519-dalek | 2.x | Ed25519 signatures | Dalek project; multiple audits |
| sha2 | 0.10 | SHA-256, SHA-512 | RustCrypto |
| hmac | 0.12 | HMAC construction | RustCrypto |
| bip39 | 2.x | Mnemonic generation | Community maintained |
| bip32 | 0.5 | HD key derivation | RustCrypto |
| rand | 0.8 | CSPRNG interface | Rust project |
| getrandom | 0.2 | Platform entropy (Web Crypto) | Rust project |
| zeroize | 1.x | Memory zeroing | RustCrypto |

`Cargo.lock` is committed to the repository to ensure reproducible builds and prevent supply chain attacks via dependency version drift.

---

## Security Hardening Roadmap

The following security enhancements are planned for future releases:

### Near-term (v1.0)

- **Argon2id migration**: Replace PBKDF2 with Argon2id for password-based key derivation, providing stronger resistance against GPU-accelerated brute force attacks (RFC 9106)
- **Hardware wallet integration**: Ledger and Trezor support for transaction signing without seed exposure in software
- **Multi-RPC verification**: Cross-reference balance queries and transaction receipts across multiple independent RPC endpoints to detect compromised nodes
- **Cargo-audit CI integration**: Automated vulnerability scanning for all Rust dependencies on every build
- **Subresource integrity**: Verify WASM binary integrity at load time in the extension context

### Medium-term (v1.x)

- **Formal verification**: Apply formal verification tools to critical cryptographic paths (encryption, key derivation, transaction signing)
- **Biometric unlock**: Fingerprint and Face ID support via WebAuthn for wallet unlock (desktop and mobile)
- **Rate limiting**: Progressive delays on failed password attempts to further impede brute force attacks
- **Secure enclave**: On Tauri desktop builds, leverage OS-level secure enclaves (macOS Keychain, Windows DPAPI, Linux Secret Service) for key storage
- **Transaction allowlists**: User-configurable address allowlists that bypass approval flows for trusted recipients

### Long-term (v2.0)

- **MPC signing**: Multi-party computation for threshold signing, eliminating single-point-of-failure in key management
- **Social recovery**: Shamir's Secret Sharing for seed recovery via trusted contacts
- **Independent security audit**: Engage a reputable third-party security firm for a comprehensive audit of the cryptographic core and extension architecture
- **Bug bounty program**: Establish a formal vulnerability reward program

---

## Vulnerability Disclosure

If you discover a security vulnerability in Rusby Wallet:

1. **Do not open a public issue.** Security vulnerabilities must be reported privately.
2. Send an email to the project maintainer with:
   - A detailed description of the vulnerability
   - Steps to reproduce
   - Potential impact assessment
   - A proof of concept, if available
3. Allow up to 48 hours for initial acknowledgment.
4. Coordinate disclosure timing with the maintainer.

We are committed to:
- Acknowledging reports within 48 hours
- Providing a remediation timeline within 7 days
- Releasing patches for critical vulnerabilities within 7 days of confirmation
- Crediting reporters in the security advisory (with permission)

---

## Supported Versions

| Version | Security Support |
|---------|-----------------|
| 0.9.x | Active support |
| 0.8.x | Active support |
| 0.7.x | Security patches only |
| < 0.7.0 | End of life |

Users on versions prior to 0.7.0 should upgrade immediately, as these versions lack critical security fixes including PBKDF2 iteration count increase, overflow protection, and comprehensive key zeroization.

---

## Audit History

| Date | Auditor | Scope | Findings | Status |
|------|---------|-------|----------|--------|
| 2026-02-05 | Internal review | Full codebase (v0.7.0) | 12 Critical, 18 High, 22 Medium | All Critical and High fixed |
| 2026-02-06 | External review | Crypto core + extension | 2 Critical, 2 High, 4 Medium | All findings fixed |
| 2026-02-07 | Internal review | Dead code, warnings, build hygiene | 23 warnings | All eliminated |

All identified vulnerabilities have been remediated. Detailed audit reports are maintained in the project's internal documentation.

---

## References

- NIST SP 800-38D: Recommendation for Block Cipher Modes of Operation: Galois/Counter Mode (GCM)
- NIST SP 800-132: Recommendation for Password-Based Key Derivation
- RFC 8018: PKCS #5: Password-Based Cryptography Specification Version 2.1
- RFC 9106: Argon2 Memory-Hard Function for Password Hashing
- OWASP Password Storage Cheat Sheet (2024 edition)
- BIP-32: Hierarchical Deterministic Wallets
- BIP-39: Mnemonic Code for Generating Deterministic Keys
- BIP-44: Multi-Account Hierarchy for Deterministic Wallets
- BIP-84: Derivation Scheme for P2WPKH Based Accounts
- BIP-143: Transaction Signature Verification for Version 0 Witness Program
- SLIP-10: Universal Private Key Derivation from Master Private Key
- EIP-155: Simple Replay Attack Protection
- EIP-191: Signed Data Standard
- EIP-712: Typed Structured Data Hashing and Signing
- EIP-1193: Ethereum Provider JavaScript API
- EIP-1559: Fee Market Change for ETH 1.0 Chain
- EIP-6963: Multi Injected Provider Discovery
