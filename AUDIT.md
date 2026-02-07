# Rusby Wallet — Audit Completo v0.7.0

**Data**: 5 Febbraio 2026
**Versione analizzata**: v0.7.0
**Scope**: wallet-core + wallet-ui + extension setup + dApp connectivity + sicurezza avanzata + DeFi (NFT, Swap) + i18n

---

## 1. Executive Summary

Rusby Wallet è un wallet multi-chain scritto in Rust puro con frontend Leptos (WASM). Il progetto ha raggiunto lo stadio **DeFi-ready wallet**: le fondamenta crittografiche sono solide, la copertura test è eccellente (118 test), l'architettura è completa con connettività dApp, funzionalità DeFi (NFT, Swap), internazionalizzazione (9 lingue), e sicurezza avanzata (phishing detection, TX simulation, token approval management).

**Punteggio complessivo: 8.5/10** — DeFi features implementate (NFT display, swap integrato), i18n completa, WalletConnect v2, sicurezza avanzata. Mancano multi-account HD e hardware wallet per piena competitività enterprise.

---

## 2. Architettura

### Punti di forza
- **Separazione netta core/ui**: `wallet-core` è una libreria pura senza dipendenze UI, testabile indipendentemente
- **Zero JavaScript nel core**: tutta la crittografia gira in Rust compilato a WASM
- **Dipendenze crypto auditate**: RustCrypto (aes-gcm, pbkdf2, k256) e Dalek (ed25519)
- **Derivazione HD corretta**: BIP32 secp256k1 + SLIP-10 Ed25519 con path standard
- **11 chain da singolo seed**: derivazione deterministica conforme agli standard di settore

### Criticità
- **Nessuna astrazione di rete**: i client RPC in `wallet-ui` sono tightly coupled alla UI. Servono in un layer separato per poter essere riutilizzati (es. background service worker)
- ~~**Nessun background script**~~ — ✅ **RISOLTO v0.4.0**: implementato service worker (`background.js`) con routing messaggi e gestione permessi dApp
- **State management minimale**: signal Leptos con `provide_context` funziona per un MVP, ma non scala per gestire token multipli, cronologia TX, connessioni dApp contemporanee
- ~~**Nessun sistema di eventi**~~ — ✅ **RISOLTO v0.4.0**: implementato message passing tra popup ↔ background ↔ content script ↔ dApp

---

## 3. Sicurezza

### Crittografia — ✅ Solida (8/10)

| Aspetto | Implementazione | Giudizio |
|---------|----------------|----------|
| Cifratura seed | AES-256-GCM | ✅ Standard industriale |
| Key derivation | PBKDF2-HMAC-SHA256, 100k iter | ⚠️ Adeguato, Argon2id sarebbe meglio |
| Salt | 32 byte random per vault | ✅ Corretto |
| Nonce | 12 byte random per operazione | ✅ Corretto |
| Entropy source | `getrandom` → Web Crypto API | ✅ CSPRNG conforme |
| Chiavi in memoria | Derivate on-demand, zeroize dopo firma | ✅ Implementato v0.3.0 |

### Vulnerabilità e rischi identificati

#### 🔴 Critici

1. ~~**Nessun auto-lock timeout**~~ — ✅ **RISOLTO v0.3.0**
   Implementato auto-lock con timer inattività configurabile (1/5/15/30 min). Default OFF per non interferire con i test, attivabile da Settings.

2. ~~**Nessuna validazione forza password**~~ — ✅ **RISOLTO v0.3.0**
   Implementato `validate_password_strength()` con enum Weak/Fair/Strong. Indicatore colorato nella UI, blocco creazione wallet se Weak. Policy: minimo 8 caratteri + mix maiuscole/numeri/simboli.

3. ~~**Cosmos: SHA256 troncato invece di RIPEMD160(SHA256)**~~ — ✅ **RISOLTO v0.3.0**
   Implementato `RIPEMD160(SHA256(pubkey))` conforme allo standard Cosmos SDK. Dipendenza `ripemd` aggiunta.

4. ~~**Bitcoin è un placeholder**~~ — ✅ **RISOLTO v0.3.0**
   Implementazione Bitcoin P2WPKH completa: derivazione BIP84, firma TX SegWit (BIP-143), RPC via mempool.space (balance, UTXO, fee estimation, broadcast).

#### 🟡 Medi

5. **Nessun controllo integrità del vault**
   Se il vault cifrato viene corrotto nel localStorage, il wallet non rileva la corruzione prima del tentativo di decrypt (GCM fallirà, ma l'errore non è gestito gracefully).
   **Remediation**: Checksum aggiuntivo o gestione errore specifica per vault corrotto vs password errata.

6. **RPC endpoints hardcoded senza fallback robusto**
   Se un RPC è down, il balance non si aggiorna. Nessun retry con exponential backoff, nessun health check.
   **Remediation**: Pool di RPC con round-robin, retry con backoff, health monitoring.

7. **Nessun rate limiting sulle chiamate RPC**
   Auto-refresh ogni 30s × 11 chain = ~22 chiamate/min. Con token multipli salirà rapidamente.
   **Remediation**: Batching, caching, refresh solo per chain attiva.

8. ~~**Chiavi private possono persistere nella WASM linear memory**~~ — ✅ **MITIGATO v0.3.0**
   Aggiunto `zeroize` crate sui buffer sensibili in `encrypt()`, `decrypt()`, `create_wallet()`, `unlock_wallet()`. Le chiavi vengono azzerate dopo l'uso. Nota: mitigazione parziale — WASM linear memory non garantisce zero-fill al 100%, ma il rischio è significativamente ridotto.

#### 🟢 Bassi

9. **Nessun Content Security Policy per web app standalone**
   Il CSP è definito solo nel manifest dell'estensione. La versione web app è priva di protezione CSP.
   **Remediation**: Meta tag CSP in `index.html` o header server-side.

10. **Nessun meccanismo anti-phishing**
    Manca un'immagine/parola segreta personalizzabile che confermi all'utente di essere sul wallet reale.
    **Remediation**: Feature anti-phishing con immagine/parola custom al login.

---

## 4. Funzionalità — Gap Analysis

### ✅ Implementato e funzionante
- Creazione/import wallet da mnemonic BIP39
- Derivazione multi-chain (11 chain) — incluso Bitcoin P2WPKH bech32
- Balance fetching via RPC (tutte le chain, incluso Bitcoin via mempool.space)
- Firma e broadcast TX (EVM, Solana, TON, Cosmos, **Bitcoin P2WPKH SegWit**)
- Token ERC-20 (display + send) per 6 chain EVM
- Token SPL (display balance) per Solana
- Token CW-20 (Cosmos) + Jetton (TON)
- IBC token display
- Cronologia TX basica (EVM, Solana, TON, Cosmos)
- Portfolio totale in USD (CoinGecko API)
- QR code per ricezione
- Validazione forza password con indicatore visivo
- Auto-lock configurabile (default OFF)
- Pagina Settings
- Zeroize chiavi in memoria
- Tema dark/light
- Layout responsive popup/fullpage
- Storage sincronizzato localStorage + chrome.storage
- NFT Display (EVM via Alchemy + Solana via Helius)
- Swap integrato (0x API v2 per chain EVM)
- Toast notifications
- Internazionalizzazione (i18n) — 9 lingue, ~313 chiavi
- Testnet toggle
- Address book
- Export/import backup cifrato
- WalletConnect v2
- TX simulation pre-firma
- Phishing detection
- Scam address warning
- Token approval management

### ❌ Mancante — Essenziale per un wallet moderno

| Feature | Impatto | Priorità |
|---------|---------|----------|
| ~~**Token ERC-20/SPL**~~ | ~~Senza token il wallet è inutilizzabile~~ | ✅ DONE |
| ~~**Cronologia transazioni**~~ | ~~Gli utenti devono vedere cosa hanno fatto~~ | ✅ DONE |
| ~~**Auto-lock**~~ | ~~Rischio sicurezza critico~~ | ✅ DONE |
| ~~**WalletConnect v2**~~ | ~~Senza connessione dApp il wallet è isolato~~ | ✅ DONE v0.5.0 |
| ~~**Injected provider (window.ethereum)**~~ | ~~Requisito per interagire con qualsiasi dApp web~~ | ✅ DONE v0.4.0 (window.rusby + EIP-6963) |
| ~~**NFT display**~~ | ~~Atteso da tutti gli utenti multi-chain~~ | ✅ DONE v0.7.0 |
| ~~**Swap integrato**~~ | ~~Feature chiave di Rabby, Phantom, MetaMask~~ | ✅ DONE v0.7.0 |
| ~~**Token approval management**~~ | ~~Sicurezza essenziale per utenti DeFi~~ | ✅ DONE v0.6.0 |
| **Multi-account** | Standard in tutti i wallet HD | P1 |
| **Custom RPC** | Power user feature essenziale | P1 |
| ~~**CW-20 token (Cosmos)**~~ | ~~Completare supporto token per Cosmos~~ | ✅ DONE v0.7.0 |
| ~~**Testnet toggle**~~ | ~~Necessario per sviluppatori~~ | ✅ DONE v0.7.0 |
| ~~**Address book**~~ | ~~Quality of life~~ | ✅ DONE v0.7.0 |
| **Ledger/hardware wallet** | Sicurezza enterprise | P2 |
| **Notifiche TX** | UX essenziale | P2 |
| ~~**Export/import backup**~~ | ~~Disaster recovery~~ | ✅ DONE v0.7.0 |
| ~~**Simulazione TX**~~ | ~~Sicurezza avanzata (Rabby-style)~~ | ✅ DONE v0.6.0 |

---

## 5. Qualità del Codice

### Metriche

| Metrica | Valore | Giudizio |
|---------|--------|----------|
| Test totali | 118 | ✅ Ottima copertura core |
| Test passanti | 118/118 (100%) | ✅ |
| LOC wallet-core | ~5.000 | ✅ Conciso |
| LOC wallet-ui | ~6.500 | ✅ Ragionevole |
| LOC JS extension | ~700 | ✅ Leggero |
| Dipendenze dirette core | 18 | ✅ Ragionevole |
| Dipendenze dirette UI | 13 | ✅ Leggero |
| Clippy warnings | Da verificare | — |
| Audit dipendenze (cargo-audit) | Da verificare | — |

### Osservazioni sul codice

**Positivi:**
- Codice idiomatico Rust, buon uso dei tipi
- Gestione errori con `Result` consistente nel core
- Nessun `unwrap()` nei path critici di sicurezza
- Struct ben organizzate con serializzazione Serde
- Test deterministici con seed fissi per regressione

**Negativi:**
- ~~`send.rs` (454 righe) è troppo lungo — dovrebbe essere separato per chain~~ — ✅ **RISOLTO**: refactored in `tx_send/` con moduli separati per chain
- Duplicazione nella logica RPC (pattern fetch → parse → format ripetuto 4 volte)
- ~~Nessun logging/tracing per debug in produzione~~ — ✅ **RISOLTO**: implementato `logging.rs`
- ~~Nessun error boundary nella UI — un panic WASM crasha l'intero wallet~~ — ✅ **RISOLTO**: implementato `error_boundary.rs`
- `clone()` eccessivo sui signal Leptos in diversi componenti

---

## 6. Performance

### WASM Bundle
- Build release con Trunk produce un bundle WASM ragionevole per le dipendenze crypto
- Nessun code splitting — tutto viene caricato al primo avvio
- Nessuna lazy loading delle chain non utilizzate

### RPC
- Fetch sequenziali per chain diverse — dovrebbero essere paralleli
- Nessun caching delle risposte
- 30s di polling è ragionevole ma non adattivo (dovrebbe accelerare durante operazioni attive)

### Rendering
- Leptos CSR è efficiente con fine-grained reactivity
- Nessun virtual scrolling (problema futuro con liste token lunghe)

---

## 7. Conformità e Standard

| Standard | Stato |
|----------|-------|
| BIP39 (mnemonic) | ✅ Conforme |
| BIP32 (HD keys) | ✅ Conforme |
| BIP44 (multi-account) | ⚠️ Solo account 0 |
| BIP84 (P2WPKH) | ✅ Conforme (v0.3.0) |
| SLIP-10 (Ed25519) | ✅ Conforme |
| EIP-55 (checksum) | ✅ Conforme |
| EIP-1559 (Type 2 TX) | ✅ Conforme |
| EIP-191 (personal_sign) | ✅ Conforme (v0.4.0) |
| EIP-712 (typed data) | ✅ Conforme (v0.4.0) |
| EIP-1193 (provider API) | ✅ Conforme (v0.4.0) |
| EIP-6963 (multi-provider) | ✅ Conforme (v0.4.0) |
| WalletConnect v2 | ✅ Conforme (v0.5.0) |
| CAIP-2 | ✅ Conforme (v0.5.0) |
| 0x Swap API v2 | ✅ Conforme (v0.7.0) |
| Alchemy NFT API v3 | ✅ Conforme (v0.7.0) |
| Chrome Extension Manifest v3 | ✅ Conforme |

---

## 8. Raccomandazioni Prioritarie

### ~~Fase 1 — Sicurezza critica~~ ✅ COMPLETATA (v0.3.0)
1. ✅ Auto-lock con timeout configurabile (default OFF)
2. ✅ Validazione forza password (Weak/Fair/Strong)
3. ✅ Derivazione Cosmos corretta (RIPEMD160)
4. ✅ Bitcoin P2WPKH completo (derivazione + TX + RPC)
5. ✅ Zeroize per chiavi in memoria
6. ✅ Token ERC-20 + SPL (display + transfer)
7. ✅ Cronologia TX (block explorer API)
8. ✅ Portfolio USD (CoinGecko)

### ~~Fase 2 — Connettività dApp~~ ✅ COMPLETATA (v0.4.0)
8. ✅ Injected provider `window.rusby` (EIP-1193/EIP-6963)
9. ✅ Background service worker per l'estensione
10. ✅ Content script bridge (relay messaggi)
11. ✅ Firma EIP-191 (personal_sign) + EIP-712 (typed data)
12. ✅ Pagina approvazione richieste dApp
13. ✅ Gestione permessi dApp (revoca in Settings)
14. ✅ CSP per web app standalone
15. ⚠️ WalletConnect v2 — pendente

### ~~Fase 3 — Competitività~~ ⚠️ PARZIALMENTE COMPLETATA
11. ✅ **COMPLETATO v0.7.0** — Swap integrato (0x API v2 per chain EVM)
12. ✅ **COMPLETATO v0.7.0** — NFT display (EVM via Alchemy + Solana via Helius)
13. ✅ **COMPLETATO v0.6.0** — Simulazione TX pre-firma
14. Multi-account HD (BIP44 account index > 0) — pendente
15. Hardware wallet support (Ledger via WebHID) — pendente

### Fase 4 — Polish e crescita ⚠️ PARZIALMENTE COMPLETATA
16. Notifiche TX in real-time (WebSocket/SSE) — pendente
17. Fiat on-ramp integration — pendente
18. ✅ **COMPLETATO v0.7.0** — Export/import backup cifrato
19. Tauri desktop build — pendente
20. ✅ **COMPLETATO v0.7.0** — Internazionalizzazione (i18n) — 9 lingue, ~313 chiavi

---

## 9. Conclusione

Rusby Wallet ha raggiunto con la v0.7.0 lo stadio di **wallet DeFi-ready**. Le funzionalità DeFi critiche sono state implementate: NFT display (EVM via Alchemy, Solana via Helius), swap integrato (0x API v2), token approval management, e simulazione TX pre-firma. Il supporto token è ora completo con CW-20 (Cosmos), Jetton (TON) e IBC token display.

L'internazionalizzazione copre 9 lingue con ~313 chiavi, rendendo il wallet accessibile a un pubblico globale. La sicurezza avanzata include phishing detection, scam address warning, e TX simulation. WalletConnect v2 completa la connettività dApp.

La copertura test è salita a 118 (da 66 in v0.4.0) con 100% passing. Il codebase resta auditabile (~12.200 LOC totali + 700 LOC JS vs ~500k+ di MetaMask), con una crescita controllata e ben strutturata.

I gap rimanenti sono: **multi-account HD** (BIP44 account index > 0), **hardware wallet** (Ledger via WebHID), **custom RPC**, **notifiche TX**, e **Tauri desktop**. Nessuno di questi è bloccante per l'utilizzo quotidiano — il wallet è pienamente funzionale per utenti DeFi su 11 chain.
