# Rusby Wallet

Wallet multi-chain scritto interamente in Rust, compilato a WebAssembly per browser, estensione Chrome e desktop.

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
| Bitcoin | secp256k1 / BIP44 m/44'/0'/0'/0/0 | Base58Check (P2PKH) |
| Cosmos Hub | secp256k1 / BIP44 m/44'/118'/0'/0/0 | Bech32 (cosmos1...) |
| Osmosis | secp256k1 / BIP44 m/44'/118'/0'/0/0 | Bech32 (osmo1...) |

## Architettura

```
wallet-multichain-rust/
├── crates/
│   ├── wallet-core/        # Libreria crypto (no UI, testabile standalone)
│   │   ├── bip39_utils.rs  # Mnemonic generation/validation
│   │   ├── bip32_utils.rs  # HD key derivation (secp256k1 + Ed25519)
│   │   ├── chains/         # Derivazione indirizzi per chain
│   │   ├── crypto.rs       # AES-256-GCM encrypt/decrypt
│   │   ├── wallet.rs       # Wallet manager (create/unlock)
│   │   ├── qr.rs           # QR code generation (SVG)
│   │   └── tx/             # Transaction signing (EVM, Solana, TON, Cosmos)
│   └── wallet-ui/          # Frontend Leptos → WASM
│       ├── app.rs          # Root component + layout
│       ├── pages/          # Onboarding, Login, Dashboard, Send, Receive
│       ├── components/     # Navbar, Sidebar, ChainSelector, AddressDisplay, ConfirmationModal
│       ├── rpc/            # RPC client per chain (balance, nonce, broadcast)
│       ├── state.rs        # Stato globale + storage helpers
│       └── style.css       # CSS con tema dark/light
├── extension/              # Manifest v3 + icone
├── build-extension.sh      # Script build estensione
└── Trunk.toml              # Config trunk
```

## Sicurezza

- **Seed mai in chiaro su disco**: il mnemonic viene cifrato con AES-256-GCM prima di essere salvato
- **Key derivation**: PBKDF2 con 100.000 iterazioni per derivare la chiave di cifratura dalla password
- **Nessun secret in localStorage**: solo il vault cifrato viene persistito
- **Password solo in memoria**: la chiave di decifratura vive solo nel runtime, mai salvata
- **Storage extension**: `chrome.storage.local` per il contesto estensione, con sync a localStorage

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

```bash
# Test wallet-core
cargo test -p wallet-core

# Test con output verboso
cargo test -p wallet-core -- --nocapture
```

## Roadmap

- [ ] Selezione chain durante creazione wallet (attiva/disattiva)
- [ ] Aggiunta chain custom (EVM custom RPC, Cosmos SDK custom)
- [x] Balance reale via RPC per ogni chain
- [x] Firma e invio transazioni reale
- [x] Generazione QR code per ricezione
- [ ] WalletConnect v2
- [ ] Toggle Mainnet/Testnet
- [ ] Export/import backup wallet
- [ ] Build desktop con Tauri 2.0
- [ ] Supporto multi-wallet (più seed phrase)

## Licenza

GPL-3.0 — vedi [LICENSE](LICENSE)
