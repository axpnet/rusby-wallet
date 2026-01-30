# Changelog

## [0.1.0] - 2026-01-30

### Aggiunto
- **wallet-core**: libreria crypto completa in pure Rust
  - Generazione e validazione mnemonic BIP39 (12-24 parole)
  - Derivazione HD keys BIP32/BIP44 (secp256k1) e SLIP-10 (Ed25519)
  - Derivazione indirizzi per 11 chain: Ethereum, Polygon, BSC, Optimism, Base, Arbitrum, Solana, TON, Bitcoin, Cosmos Hub, Osmosis
  - Cifratura/decifratura AES-256-GCM con PBKDF2 (100k iterazioni)
  - Wallet manager: creazione e unlock wallet cifrati
  - 28 test unitari passanti

- **wallet-ui**: frontend Leptos 0.7 compilato a WASM
  - Wizard onboarding 3 step (genera/importa seed → password → conferma)
  - Login con unlock wallet cifrato
  - Dashboard con balance hero, action buttons, chain selector
  - Pagina Send (mock)
  - Pagina Receive con display indirizzo e copia in clipboard
  - Tema dark/light con toggle
  - Layout popup (420px, bottom nav) per estensione
  - Layout fullpage (sidebar, griglia chain) per web
  - Rilevamento automatico contesto extension/web
  - Pulsante espandi popup → tab fullpage
  - Supporto `chrome.storage.local` per estensione con sync a localStorage

- **Extension Chrome**: Manifest v3 con CSP per WASM
  - Build script automatizzato (`build-extension.sh`)
  - Icone SVG placeholder generate automaticamente

### Sicurezza
- Seed sempre cifrato con AES-256-GCM prima di essere salvato
- Password mai persistita, solo in memoria durante la sessione
- PBKDF2 100.000 iterazioni per key derivation
