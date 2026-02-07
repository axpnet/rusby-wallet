# Rusby Wallet

Wallet multi-chain scritto interamente in Rust (~17.500 righe Rust, ~770 JS), compilato a WebAssembly per browser, estensione Chrome e desktop. Supporta 11 chain, token ERC-20/SPL/CW-20/Jetton, NFT, swap, 7 temi + custom, i18n (9 lingue), layout fullpage Talisman-like e injected provider EIP-1193.

## Stack

| Componente | Tecnologia |
|-----------|-----------|
| Frontend | Leptos 0.7 (CSR) |
| Crypto core | Pure Rust (`k256`, `ed25519-dalek`, `aes-gcm`, `bip39`, `bip32`) |
| Build WASM | Trunk |
| Extension | Chrome Manifest v3 |
| Desktop | Tauri 2.0 (roadmap) |

## Chain supportate

| Chain | Derivazione | Encoding |
|-------|------------|----------|
| Ethereum | secp256k1 / BIP44 m/44'/60'/0'/0/0 | EIP-55 checksum |
| Polygon | secp256k1 / BIP44 m/44'/60'/0'/0/0 | EIP-55 checksum |
| BNB Chain | secp256k1 / BIP44 m/44'/60'/0'/0/0 | EIP-55 checksum |
| Optimism | secp256k1 / BIP44 m/44'/60'/0'/0/0 | EIP-55 checksum |
| Base | secp256k1 / BIP44 m/44'/60'/0'/0/0 | EIP-55 checksum |
| Arbitrum | secp256k1 / BIP44 m/44'/60'/0'/0/0 | EIP-55 checksum |
| Solana | Ed25519 / SLIP-10 m/44'/501'/0'/0' | Base58 |
| TON | Ed25519 / SLIP-10 m/44'/607'/0' | Base64url (v4R2) |
| Bitcoin | secp256k1 / BIP84 m/84'/0'/0'/0/0 | Bech32 (P2WPKH bc1...) |
| Cosmos Hub | secp256k1 / BIP44 m/44'/118'/0'/0/0 | Bech32 (cosmos1...) |
| Osmosis | secp256k1 / BIP44 m/44'/118'/0'/0/0 | Bech32 (osmo1...) |

## Architettura

```
wallet-multichain-rust/
├── crates/
│   ├── wallet-core/          # Libreria crypto (no UI, testabile standalone)
│   │   ├── bip39_utils.rs    # Mnemonic generation/validation
│   │   ├── bip32_utils.rs    # HD key derivation (secp256k1 + Ed25519)
│   │   ├── chains/           # Derivazione indirizzi per chain (EVM, Solana, TON, Cosmos, Bitcoin)
│   │   ├── crypto.rs         # AES-256-GCM encrypt/decrypt
│   │   ├── wallet.rs         # Wallet manager (create/unlock)
│   │   ├── qr.rs             # QR code generation (SVG)
│   │   ├── nft.rs            # NFT types e strutture dati
│   │   ├── swap.rs           # Swap types e strutture dati
│   │   ├── tokens/           # Token support (ERC-20, SPL, CW-20, Jetton)
│   │   ├── signing/          # Firma messaggi (EIP-191 personal_sign, EIP-712 typed data)
│   │   ├── security/         # Phishing detection, scam address warning
│   │   └── tx/               # Transaction signing (EVM, Solana, TON, Cosmos, Bitcoin)
│   └── wallet-ui/            # Frontend Leptos → WASM
│       ├── app.rs            # Root component + layout
│       ├── i18n/             # Internazionalizzazione (9 lingue, ~304 chiavi)
│       ├── pages/            # Onboarding, Login, Dashboard, Send, Receive,
│       │                     #   NFT, Swap, History, Settings, Approvals, WalletConnect
│       ├── components/       # Navbar, Sidebar, TopNav, ChainSidebar, ChainSelector,
│       │                     #   AddressDisplay, ConfirmationModal, Toast, SecurityWarning, ErrorBoundary
│       ├── rpc/              # RPC client per chain (balance, nonce, broadcast,
│       │                     #   nft, swap, erc20, spl, cw20, prices, history,
│       │                     #   approvals, simulate)
│       ├── theme/            # Sistema temi (7 builtin + custom, ~18 chiavi i18n)
│       ├── state.rs          # Stato globale + storage helpers
│       └── style.css         # CSS con variabili tema
├── extension/                # Manifest v3 + service worker + injected provider
│   ├── background.js         # Service worker (routing messaggi, gestione permessi)
│   ├── content-script.js     # Bridge pagina web ↔ background
│   └── inpage.js             # Provider EIP-1193 (window.rusby) + EIP-6963
├── build-extension.sh        # Script build estensione
└── Trunk.toml                # Config trunk
```

## Sicurezza

- **Seed mai in chiaro su disco**: il mnemonic viene cifrato con AES-256-GCM prima di essere salvato
- **Key derivation**: PBKDF2 con 600.000 iterazioni per derivare la chiave di cifratura dalla password
- **Nessun secret in localStorage**: solo il vault cifrato viene persistito
- **Password solo in memoria**: la chiave di decifratura vive solo nel runtime, mai salvata
- **Storage extension**: `chrome.storage.local` per il contesto estensione, con sync a localStorage
- **TX simulation pre-firma**: simulazione transazioni prima della firma per prevenire errori e frodi
- **Phishing detection**: blocklist + euristica per rilevamento siti malevoli
- **Scam address warning**: avviso automatico per indirizzi noti come scam
- **Token approval management**: gestione e revoca delle approvazioni token per smart contract
- **API keys**: chiavi API (Alchemy, Helius, 0x) salvate in localStorage, fornite dall'utente e non segrete

## Prerequisiti

- Rust (stable)
- `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
- Trunk: `cargo install trunk`

## Sviluppo

```bash
# Web app (dev server su porta 3000)
trunk serve

# Build release
trunk build --release

# Build estensione Chrome
bash build-extension.sh
# → output in extension-dist/, caricare come "unpacked" in chrome://extensions
```

## Test

**123 test unitari** coprono derivazione chiavi, cifratura, encoding indirizzi, firma transazioni, token encoding, sicurezza e NFT/swap types.

```bash
# Test wallet-core (123 test)
cargo test -p wallet-core

# Test con output verboso
cargo test -p wallet-core -- --nocapture
```

## Roadmap

- [x] Selezione chain durante creazione wallet (attiva/disattiva)
- [ ] Aggiunta chain custom (EVM custom RPC, Cosmos SDK custom)
- [x] Balance reale via RPC per ogni chain
- [x] Firma e invio transazioni reale
- [x] Generazione QR code per ricezione
- [x] Token ERC-20/SPL + Cronologia TX + Portfolio USD
- [x] Bitcoin P2WPKH completo
- [x] Sicurezza: auto-lock, password strength, zeroize
- [x] Injected provider EIP-1193 + EIP-6963 + background service worker
- [x] Firma EIP-191/712 + UI approvazione dApp
- [x] Swap integrato (0x API v2 per EVM) + NFT display (EVM + Solana) + WalletConnect v2
- [x] Toggle Mainnet/Testnet
- [x] Export/import backup wallet
- [x] Internazionalizzazione (i18n) — 9 lingue (EN, IT, ES, FR, DE, PT, ZH, JA, KO)
- [x] Token CW-20 (Cosmos) + Jetton (TON) + IBC token display + Token discovery
- [x] NFT Display (EVM via Alchemy + Solana via Helius)
- [x] Toast notifications + gestione API keys
- [x] Sistema temi — 7 builtin + custom editor con color picker
- [x] Layout fullpage Talisman-like — TopNav orizzontale + ChainSidebar verticale
- [x] UX onboarding — spinner animato, loading dinamico, security badge
- [x] Fix OOM WASM — `signal.with()` ovunque, Memos, interval leak fix
- [ ] Build desktop con Tauri 2.0
- [ ] Supporto multi-wallet (più seed phrase)

## Licenza

GPL-3.0 — vedi [LICENSE](LICENSE)
