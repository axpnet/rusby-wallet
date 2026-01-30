# Changelog

## [0.2.0] - 2026-01-30

### Aggiunto
- **Balance reale via RPC** per tutte le chain (EVM, Solana, TON, Cosmos)
  - Modulo `rpc/` in wallet-ui con client per ogni chain
  - EVM: `eth_getBalance` JSON-RPC
  - Solana: `getBalance` JSON-RPC
  - TON: `getAddressBalance` via toncenter REST
  - Cosmos/Osmosis: `/cosmos/bank/v1beta1/balances` REST
  - Auto-refresh ogni 30 secondi, indicatore loading nella dashboard

- **Firma e invio transazioni** per tutte le chain
  - Modulo `tx/` in wallet-core con signing per ogni chain
  - EVM: EIP-1559 (Type 2) con RLP encoding e secp256k1 signing
  - Solana: SystemProgram transfer con Ed25519 signing
  - TON: wallet v4r2 external message con Ed25519 signing
  - Cosmos: MsgSend Amino JSON con secp256k1 signing
  - Gas estimation automatica via RPC
  - Modale di conferma con riepilogo TX e richiesta password
  - Decrypt seed on-demand per la firma (mai in memoria persistente)

- **QR code ricezione** in Rust puro
  - Generazione QR come SVG inline via crate `qrcode`
  - Rendering diretto nella pagina Receive, nessuna dipendenza JS

- 13 nuovi test unitari (totale: 41 passanti)

### Dipendenze aggiunte
- `qrcode 0.14` — generazione QR code (wallet-core)
- `rlp 0.6` — RLP encoding per tx EVM (wallet-core)
- `gloo-net 0.6` — HTTP fetch WASM-compatibile (wallet-ui)
- `hex 0.4`, `bs58 0.5` — encoding in wallet-ui

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
