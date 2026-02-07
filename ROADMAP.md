# Rusby Wallet — Roadmap Strategica

**Ultimo aggiornamento**: 7 Febbraio 2026
**Versione corrente**: v0.8.1
**Posizionamento**: Il wallet più sicuro per utenti multi-chain — 100% Rust, zero JavaScript nel core

---

## Stato Attuale

| Metrica | Valore |
|---------|--------|
| Chain supportate | 11 (EVM×6 + Solana + TON + Bitcoin + Cosmos Hub + Osmosis) |
| Test unitari | 123 passanti (100%) |
| LOC totali | ~17.500 Rust + ~770 JS |
| Standard conformi | BIP39/32/44/84, SLIP-10, EIP-55/191/712/1193/1559/6963, WalletConnect v2, Manifest v3, 0x API v2, Alchemy NFT v3, Helius DAS, i18n 9 lingue |
| Punteggio audit | 8.5/10 |

---

## ✅ Milestone 1 — "Usable Wallet" (v0.3.0) — COMPLETATA

**Obiettivo**: Rendere il wallet effettivamente utilizzabile per utenti reali.

- [x] Bitcoin P2WPKH completo (derivazione BIP84 + firma SegWit BIP-143 + RPC mempool.space)
- [x] Token ERC-20 (display balance + send) per 6 chain EVM — USDT, USDC, DAI, WETH, WBTC
- [x] Token SPL (display balance) per Solana — USDC, USDT, WSOL, JUP
- [x] Cronologia TX basica (EVM, Solana, TON, Cosmos via block explorer API)
- [x] Portfolio totale in USD (CoinGecko API, cache 60s)
- [x] Auto-lock con timeout configurabile (1/5/15/30 min, default OFF)
- [x] Validazione forza password (Weak/Fair/Strong con indicatore colorato)
- [x] Fix derivazione Cosmos — `RIPEMD160(SHA256(pubkey))` conforme
- [x] Zeroize chiavi in memoria (`zeroize` crate su buffer sensibili)

**Rilascio**: 2 Febbraio 2026 | **Test**: 54 passanti

---

## ✅ Milestone 2 — "Connected Wallet" (v0.4.0) — COMPLETATA

**Obiettivo**: Connettere il wallet all'ecosistema dApp via provider iniettato.

**Architettura 3-contesti**: `dApp (inpage.js) ↔ content-script.js ↔ background.js ↔ popup (WASM/Leptos)`

- [x] Background Service Worker — routing messaggi, stato sessione, coda richieste persistente
- [x] Content Script — bridge bidirezionale pagina web ↔ background
- [x] Injected Provider EIP-1193 (`window.rusby`) — `request()`, eventi, legacy `enable()`/`send()`
- [x] EIP-6963 Multi-Provider Discovery — annuncio automatico con `rdns: io.rusby.wallet`
- [x] Firma EIP-191 (personal_sign) in Rust puro — prefix + keccak256 + secp256k1 + recovery
- [x] Firma EIP-712 (typed data) in Rust puro — domain separator, struct hash, firma
- [x] Pagina approvazione richieste dApp nel popup
- [x] Gestione permessi dApp in Settings (lista origini + revoca)
- [x] CSP per web app standalone

**Principio di sicurezza**: Le chiavi private NON escono MAI dal popup WASM.

**Rilascio**: 2 Febbraio 2026 | **Test**: 66 passanti (+12 nuovi per EIP-191/712)

---

## ✅ Milestone 3 — "WalletConnect" (v0.5.0) — COMPLETATA

**Obiettivo**: Connettività universale dApp via WalletConnect v2.

- [x] WalletConnect SDK bundlato (`@walletconnect/web3wallet` via esbuild → ESM)
- [x] Wrapper `RusbyWC` con storage adapter per `chrome.storage` (sopravvive restart SW)
- [x] Keep-alive Service Worker con `chrome.alarms` ogni 24s
- [x] Event handlers WC: session_proposal → popup, session_request → coda approvazione
- [x] Mapping CAIP-2 ↔ ChainId (eip155, solana, cosmos, bip122, ton) — 7 test
- [x] Pagina WalletConnect in UI — input URI `wc:...`, lista sessioni attive, disconnessione
- [x] Pagina proposta sessione WC — mostra dApp, chain richieste, approva/rifiuta
- [x] Firma completa nel popup — modale password, decrypt seed on-demand, personal_sign + EIP-712
- [x] Namespace builder per sessioni WC (metodi + eventi per namespace)
- [x] Configurazione WC Project ID in Settings

**Rilascio**: 2 Febbraio 2026 | **Test**: 73 passanti (+7 nuovi per CAIP-2)

---

## ✅ Milestone 4 — "Smart Wallet" (v0.6.0) — COMPLETATA

**Obiettivo**: Differenziarsi con sicurezza avanzata ispirata a Rabby/Phantom.

### Sicurezza avanzata
- [x] **TX simulation pre-firma** — simulazione via `eth_call`, warning se TX fallirebbe, decode revert reason (Error(string))
- [x] **Phishing detection** — blocklist ~50 domini + typosquatting (Levenshtein distance ≤2) + heuristic TLD sospetti + keyword crypto — 7 test
- [x] **Scam address warning** — database indirizzi noti + risk assessment (self-send, zero-address, known scam) — 5 test
- [x] **Token approval management** — pagina gestione approvazioni ERC-20, scan spender noti (Uniswap, 1inch, PancakeSwap, SushiSwap, 0x) su 5 chain, bottone revoke
- [x] **SecurityWarning component** — componente riusabile con 3 livelli severità (Low/Medium/High), usato in send.rs e approve.rs

### Qualità
- [x] **Refactoring send.rs** — logica TX estratta in `tx_send/` con sotto-moduli per chain (665→315 righe UI)
- [x] **Error boundary UI** — panic hook WASM con overlay DOM + bottone ricarica, no crash silenzioso
- [x] **Logging** — modulo leggero `logging.rs` con macro `log_info!/warn!/error!/debug!` su console API, debug compilato via solo in dev

**Rilascio**: 2 Febbraio 2026 | **Test**: 87 passanti (+14 nuovi: 7 phishing + 5 scam + 2 erc20)

---

## ✅ Milestone 5 — "DeFi Wallet" (v0.7.0) — COMPLETATA

**Obiettivo**: Feature DeFi essenziali e account management.

### Feature DeFi
- [x] **Swap integrato** — 0x Swap API v2 per 6 chain EVM, quote multi-source, slippage configurabile
- [x] **NFT display base** — ERC-721/ERC-1155 via Alchemy API v3 per EVM + Metaplex via Helius DAS per Solana
- [x] **CW-20 token (Cosmos)** — query balance CosmWasm, invio MsgExecuteContract
- [x] **Jetton token (TON)** — token list predefinita, query toncenter v3 API
- [x] **IBC token display** — visualizzazione token IBC con denom hash
- [x] **Internazionalizzazione (i18n)** — 9 lingue, ~304 chiavi, fallback automatico
- [x] **Toast notifications** — 4 tipi, auto-dismiss 5s
- [x] **API Keys management** — Alchemy, Helius, 0x in Settings

### Account management
- [ ] **Multi-account HD** — BIP44 account index > 0, UI per creare/selezionare account
- [ ] **Custom RPC endpoints** — aggiungere/modificare RPC per ogni chain

**Rilascio**: 5 Febbraio 2026 | **Test**: 118 passanti

---

## ✅ Milestone 5.5 — "Polished Wallet" (v0.8.0–v0.8.1) — COMPLETATA

**Obiettivo**: UX professionale, temi, performance WASM e layout fullpage.

### Temi (v0.8.0)
- [x] **7 temi builtin**: Default, Light, Midnight, Ocean, Forest, Atelier (GPT Codex), Professional (Qwen3Max)
- [x] **Tema custom**: 8 color picker con derivazione automatica variabili secondarie e live preview
- [x] **Architettura CSS variables**: `style.setProperty()` — nessun blocco CSS aggiuntivo per tema

### Layout fullpage Talisman-like (v0.8.1)
- [x] **TopNav**: navigazione orizzontale nell'header con 5 tab (Home, Send, Receive, History, Settings)
- [x] **ChainSidebar**: sidebar sinistra con selettore chain, icona, bilancio per ogni chain
- [x] **FullpageMode newtype**: risolto conflitto TypeId per `ReadSignal<bool>` nel context Leptos

### UX (v0.8.1)
- [x] **SVG spinner animato**: triple-arc spinner inline nei pulsanti durante operazioni crypto
- [x] **Loading dinamico**: progressione fase per fase (BIP-39 → PBKDF2 → derivazione HD)
- [x] **Security badge**: badge enterprise-grade nell'onboarding (AES-256-GCM, PBKDF2 600k, chiavi locali)

### Performance WASM (v0.8.1)
- [x] **Fix OOM**: `signal.with()` ovunque, Memos dashboard, interval leak fix, `with_untracked()` per timer
- [x] **PBKDF2 600k**: iterazioni aumentate da 100k a 600k (conforme OWASP 2024)

### Audit (v0.8.0)
- [x] **Audit GPT Codex 5.2**: 8 finding (2 CRIT + 2 HIGH + 4 MED) — tutti fixati
- [x] **TON v4r2 riscrittura completa**: BOC parser + cell hash + state_init + address decoder
- [x] **Zeroize completo**: seed/private key su tutti i percorsi firma (EVM, Solana, TON, Cosmos, Bitcoin)

**Rilascio**: 7 Febbraio 2026 | **Test**: 123 passanti

---

## 🔮 Milestone 6 — "Power Wallet" (v1.0.0)

**Obiettivo**: Feature complete per release stabile e audit di sicurezza esterno.

### Hardware wallet
- [ ] **Ledger support** via WebHID — firma TX senza esporre chiavi al browser
- [ ] **Trezor support** via WebUSB

### Piattaforme
- [ ] **Tauri desktop build** — app nativa per macOS/Windows/Linux
- [ ] **Firefox extension** — porting Manifest v3 → Firefox

### Feature avanzate
- [ ] **Bridge cross-chain base** — EVM ↔ EVM via aggregatore bridge (Li.Fi o Socket)
- [ ] **Staking nativo** — ETH (Lido), SOL (native delegation), ATOM (native delegation)
- [x] **Export/import backup cifrato** — AES-256-GCM export del vault completo
- [x] **Testnet toggle globale** — switch mainnet/testnet con RPC dedicati
- [ ] **Notifiche TX in real-time** — WebSocket per EVM, WebSocket/RPC polling per altre chain
- [x] **Address book** — etichette per indirizzi frequenti, auto-completamento
- [ ] **Fiat on-ramp** — integrazione MoonPay/Transak/Ramp

### Qualità e compliance
- [ ] **Audit sicurezza esterno** — audit professionale del core crypto + extension
- [ ] **Virtual scrolling** — liste token/NFT performanti con grandi dataset
- [ ] **Code splitting WASM** — lazy loading per chain non utilizzate

**Target**: 20+ chain | **Test target**: 120+ | **Audit**: Completato

---

## 🚀 Milestone 7 — "Next-Gen Wallet" (v2.0.0)

**Obiettivo**: Feature di nuova generazione per differenziazione massima.

### Account Abstraction (EIP-4337)
- [ ] **Smart account support** — deploy UserOperation, bundler integration
- [ ] **Social recovery** — guardian-based recovery via smart contract
- [ ] **Gasless TX** — paymaster integration per TX sponsorizzate
- [ ] **Batch transactions** — raggruppare più operazioni in una TX

### Multi-sig e governance
- [ ] **Multi-sig support** — Safe-compatible multi-signature wallet
- [ ] **DAO voting** — firma e invio voti on-chain direttamente dal wallet

### Token avanzati
- [x] **CW-20 (Cosmos)** + **Jetton (TON)** — supporto token completo per tutte le chain
- [ ] **Token auto-discovery** — scan automatico dei token posseduti (Alchemy/Moralis API)

### Mobile e estensibilità
- [ ] **Mobile app** — Tauri Mobile o PWA con supporto biometrico
- [ ] **Plugin system** — estensioni WASM sicure (ispirato a MetaMask Snaps ma sandbox Rust)

### AI e UX avanzata
- [ ] **TX explanation** — descrizione human-readable delle transazioni prima della firma
- [ ] **Portfolio analytics** — grafici P&L, allocation per chain, storico valore

**Target**: 30+ chain | **Test target**: 150+ | **Utenti target**: 200k+

---

## Priorità Feature per Competitività

Analisi basata su gap rispetto a MetaMask, Rabby, Phantom e Keplr:

| Priorità | Feature | Gap vs competitor | Milestone |
|----------|---------|-------------------|-----------|
| 🔴 P0 | ~~WalletConnect v2~~ | ~~Tutti i competitor lo hanno~~ | ✅ v0.5.0 |
| 🔴 P0 | ~~Injected provider~~ | ~~Requisito base per dApp~~ | ✅ v0.4.0 |
| 🔴 P0 | ~~Token ERC-20/SPL~~ | ~~Senza token il wallet è inutile~~ | ✅ v0.3.0 |
| 🔴 P0 | ~~TX simulation~~ | ~~Rabby leader, Phantom lo ha~~ | ✅ v0.6.0 |
| 🔴 P0 | ~~Phishing detection~~ | ~~Rabby, MetaMask lo hanno~~ | ✅ v0.6.0 |
| 🔴 P0 | ~~Token approval mgmt~~ | ~~Rabby lo ha (revoke.cash)~~ | ✅ v0.6.0 |
| 🔴 P0 | ~~Swap integrato~~ | ~~MetaMask, Rabby, Phantom lo hanno~~ | ✅ v0.7.0 |
| 🔴 P0 | ~~NFT display~~ | ~~Phantom, MetaMask, Rabby lo hanno~~ | ✅ v0.7.0 |
| 🔴 P0 | ~~i18n (9 lingue)~~ | ~~Standard per utenza globale~~ | ✅ v0.7.0 |
| 🔴 P0 | ~~Address book~~ | ~~Tutti i competitor lo hanno~~ | ✅ v0.7.0 |
| 🔴 P0 | ~~Testnet toggle~~ | ~~MetaMask, Rabby lo hanno~~ | ✅ v0.7.0 |
| 🔴 P0 | ~~Export/import backup~~ | ~~Standard per recovery~~ | ✅ v0.7.0 |
| 🟡 P1 | Multi-account HD | Standard in tutti i wallet | v1.0.0 |
| 🟢 P2 | Ledger support | MetaMask, Rabby lo hanno | v1.0.0 |
| 🟢 P2 | Staking nativo | Phantom, Keplr lo hanno | v1.0.0 |
| 🟢 P2 | Desktop app | Nessun competitor — vantaggio Rusby | v1.0.0 |
| 🔵 P3 | Account abstraction | Nessun competitor nativo — first mover | v2.0.0 |
| 🔵 P3 | Plugin system | Solo MetaMask (Snaps) | v2.0.0 |

---

## Vantaggi Competitivi Unici

1. **100% Rust** — Nessuna classe di bug tipiche JavaScript (prototype pollution, type coercion). Core completamente auditabile.
2. **Multi-chain nativo reale** — EVM + Solana + TON + Cosmos + Bitcoin da singolo seed. Nessun competitor copre tutte queste famiglie nativamente.
3. **Codebase leggero** — ~18.000 LOC vs ~500k+ MetaMask. Un singolo auditor può leggerlo in un giorno.
4. **TON first-mover** — Primo wallet open source con TON + EVM + Solana + Cosmos nativo.
5. **Architettura sicura** — Chiavi private confinate nel popup WASM, mai nel background SW.

---

## KPI per Milestone

| Milestone | Versione | Target utenti | CWS Rating | Chain | Test |
|-----------|----------|--------------|------------|-------|------|
| Usable Wallet | v0.3.0 ✅ | 100 beta | N/A | 11 | 54 |
| Connected Wallet | v0.4.0 ✅ | 500 | N/A | 11 | 66 |
| WalletConnect | v0.5.0 ✅ | 1.000 | 4.0+ | 11 | 73 |
| Smart Wallet | v0.6.0 ✅ | 5.000 | 4.3+ | 11 | 87 |
| DeFi Wallet | v0.7.0 ✅ | 10.000 | 4.3+ | 15+ | 118 |
| Polished Wallet | v0.8.1 ✅ | 15.000 | 4.5+ | 11 | 123 |
| Power Wallet | v1.0.0 | 50.000 | 4.5+ | 20+ | 130+ |
| Next-Gen | v2.0.0 | 200.000 | 4.5+ | 30+ | 150+ |

---

## Standard e Conformità

| Standard | Stato | Versione |
|----------|-------|----------|
| BIP39 (mnemonic) | ✅ Conforme | v0.1.0 |
| BIP32 (HD keys) | ✅ Conforme | v0.1.0 |
| BIP44 (multi-account) | ⚠️ Solo account 0 | v0.6.0 |
| BIP84 (P2WPKH Bitcoin) | ✅ Conforme | v0.3.0 |
| SLIP-10 (Ed25519) | ✅ Conforme | v0.1.0 |
| EIP-55 (checksum address) | ✅ Conforme | v0.1.0 |
| EIP-1559 (Type 2 TX) | ✅ Conforme | v0.2.0 |
| EIP-191 (personal_sign) | ✅ Conforme | v0.4.0 |
| EIP-712 (typed data) | ✅ Conforme | v0.4.0 |
| EIP-1193 (provider API) | ✅ Conforme | v0.4.0 |
| EIP-6963 (multi-provider) | ✅ Conforme | v0.4.0 |
| WalletConnect v2 | ✅ Conforme | v0.5.0 |
| CAIP-2 (chain ID) | ✅ Conforme | v0.5.0 |
| Chrome Extension Manifest v3 | ✅ Conforme | v0.1.0 |
| 0x Swap API v2 | ✅ Conforme | v0.7.0 |
| Alchemy NFT API v3 | ✅ Conforme | v0.7.0 |
| EIP-4337 (account abstraction) | ❌ Pianificato | v2.0.0 |

---

*Rusby Wallet — Il wallet che i Rust developer costruiscono per sé stessi.*
