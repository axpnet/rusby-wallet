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

1. ~~Selezione chain durante creazione wallet~~
2. Supporto chain custom (EVM e Cosmos SDK)
3. ~~Balance reale via RPC~~ (DONE v0.2.0)
4. ~~Firma e invio transazioni~~ (DONE v0.2.0)
5. ~~QR code ricezione~~ (DONE v0.2.0)
6. ~~Token ERC-20/SPL + Cronologia TX + Portfolio USD~~ (DONE v0.3.0)
7. ~~Bitcoin P2WPKH completo~~ (DONE v0.3.0)
8. ~~Sicurezza: auto-lock, password strength, zeroize, fix Cosmos~~ (DONE v0.3.0)
9. ~~Injected provider EIP-1193 + EIP-6963 + background SW~~ (DONE v0.4.0)
10. ~~Firma EIP-191/712 + UI approvazione dApp~~ (DONE v0.4.0)
11. ~~Swap integrato + NFT + WalletConnect v2~~ (DONE v0.5.0-v0.7.0)
12. ~~Toggle Mainnet/Testnet~~ (DONE v0.7.0)
13. ~~Export/import backup wallet~~ (DONE v0.7.0)
14. ~~Sistema temi template (7 builtin + custom)~~ (DONE v0.8.0)
15. Tauri desktop

## Moduli principali

- `crates/wallet-core/src/tx/` — firma transazioni (evm, solana, ton, cosmos, bitcoin)
- `crates/wallet-core/src/tokens/` — token ERC-20 e SPL (encoding, lista predefinita)
- `crates/wallet-core/src/chains/bitcoin.rs` — derivazione BIP84, Hash160, bech32
- `crates/wallet-core/src/qr.rs` — generazione QR code SVG
- `crates/wallet-core/src/signing/` — firma messaggi (EIP-191 personal_sign, EIP-712 typed data)
- `crates/wallet-ui/src/rpc/` — client RPC (evm, solana, ton, cosmos, bitcoin, erc20, spl, prices, history)
- `extension/background.js` — service worker (routing messaggi, gestione permessi dApp)
- `extension/content-script.js` — bridge pagina web ↔ background
- `extension/inpage.js` — provider EIP-1193 (`window.rusby`) + EIP-6963
- `crates/wallet-ui/src/theme/` — sistema temi (7 builtin + custom, CSS variables via JS)
