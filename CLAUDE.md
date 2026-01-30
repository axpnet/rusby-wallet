# CLAUDE.md

Rispondi sempre in italiano.

## Progetto

**Rusby Wallet** — Wallet multi-chain in Rust puro con frontend Leptos (WASM). Target: estensione Chrome, web app, desktop (Tauri).

## Struttura

- `crates/wallet-core/` — libreria crypto, zero dipendenze UI, testabile con `cargo test`
- `crates/wallet-ui/` — frontend Leptos 0.7 CSR, compila a WASM con Trunk
- `extension/` — manifest.json e risorse statiche per Chrome Extension
- `build-extension.sh` — script per creare il pacchetto estensione in `extension-dist/`

## Comandi

```bash
cargo test -p wallet-core          # test unitari core
trunk serve                        # dev server (porta 3000)
trunk build --release              # build produzione
bash build-extension.sh            # build estensione Chrome
```

## Convenzioni

- Chain derivation: ogni chain implementa il trait `Chain` in `crates/wallet-core/src/chains/`
- Storage: `save_to_storage()` / `load_from_storage()` in `state.rs` — scrive sia localStorage che chrome.storage.local
- Stato UI: signal Leptos con `provide_context` / `expect_context`
- Layout: popup (≤500px) usa bottom nav, fullpage usa sidebar
- Sicurezza: seed sempre cifrato con AES-256-GCM, password mai salvata

## Roadmap prioritaria

1. Selezione chain durante creazione wallet
2. Supporto chain custom (EVM e Cosmos SDK)
3. Balance reale via RPC
4. Firma e invio transazioni
5. QR code ricezione
6. Tauri desktop
