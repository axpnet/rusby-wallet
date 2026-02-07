# Rusby Wallet — Analisi Competitiva

**Data**: 5 Febbraio 2026 (aggiornato a v0.7.0)
**Wallet analizzati**: MetaMask, Rabby, Phantom, Keplr, Rusby Wallet

---

## 1. Panoramica Comparativa

| | **MetaMask** | **Rabby** | **Phantom** | **Keplr** | **Rusby** |
|---|---|---|---|---|---|
| **Utenti attivi** | ~30M | ~2M | ~7M | ~1.5M | 0 (pre-launch) |
| **Chain focus** | EVM | EVM | Solana + EVM + BTC | Cosmos IBC | Multi-chain |
| **Linguaggio core** | JavaScript | JavaScript | Rust + TypeScript | TypeScript | Rust puro |
| **Open source** | Sì | Sì | No | Sì | Sì |
| **Modello revenue** | Swap fee, staking, bridge | Swap fee | Swap fee, staking | Staking fee | Nessuno |
| **Prima release** | 2016 | 2022 | 2021 | 2020 | 2026 |

---

## 2. Confronto Feature Dettagliato

### 2.1 Gestione Account e Seed

| Feature | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---------|----------|-------|---------|-------|-------|
| Creazione wallet | ✅ | ✅ | ✅ | ✅ | ✅ |
| Import mnemonic | ✅ | ✅ | ✅ | ✅ | ✅ |
| Import private key | ✅ | ✅ | ✅ | ✅ | ❌ |
| Multi-account (HD) | ✅ | ✅ | ✅ | ✅ | ❌ |
| Multi-wallet | ✅ | ✅ | ✅ | ✅ | ✅ |
| Hardware wallet | ✅ Ledger+Trezor | ✅ Ledger+Trezor+GridPlus | ✅ Ledger | ✅ Ledger | ❌ |
| Watch-only address | ✅ | ✅ | ❌ | ❌ | ❌ |
| Social recovery | ❌ | ❌ | ❌ | ❌ | ❌ |

### 2.2 Chain e Network

| Feature | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---------|----------|-------|---------|-------|-------|
| Chain supportate | ~20 EVM | ~100+ EVM | Solana+EVM+BTC+Base | 50+ Cosmos | 11 (EVM×6+SOL+TON+BTC+ATOM+OSMO) |
| Custom network | ✅ | ✅ | ❌ | ✅ | ❌ |
| Testnet toggle | ✅ | ❌ | ✅ | ✅ | ❌ |
| Auto-detect chain | ❌ | ✅ | ✅ | N/A | ❌ |
| EVM L2 native | ✅ | ✅ | ✅ | ❌ | ✅ |
| Non-EVM chain | ❌ (solo Snaps) | ❌ | ✅ (SOL+BTC) | ✅ (Cosmos) | ✅ (SOL+TON+BTC+Cosmos) |

**Vantaggio Rusby**: Vero supporto multi-chain nativo (EVM + Solana + TON + Cosmos) — nessun altro wallet copre tutte queste famiglie.

### 2.3 Token e Asset

| Feature | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---------|----------|-------|---------|-------|-------|
| Token nativi (balance) | ✅ | ✅ | ✅ | ✅ | ✅ |
| ERC-20 | ✅ | ✅ auto-detect | ✅ | N/A | ✅ (v0.3.0) |
| SPL token | ❌ | ❌ | ✅ auto-detect | ❌ | ✅ display (v0.3.0) |
| CW-20 (Cosmos) | ❌ | ❌ | ❌ | ✅ | ✅ (v0.7.0) |
| NFT (ERC-721/1155) | ✅ | ✅ | ✅ | ❌ | ✅ Alchemy API (v0.7.0) |
| NFT (Metaplex) | ❌ | ❌ | ✅ | ❌ | ✅ Helius DAS (v0.7.0) |
| Token auto-discovery | ❌ (manual add) | ✅ | ✅ | ✅ | ❌ (lista predefinita) |
| Token price feed | ✅ CoinGecko | ✅ | ✅ | ✅ | ✅ CoinGecko (v0.3.0) |
| Portfolio totale ($) | ✅ | ✅ | ✅ | ❌ | ✅ (v0.3.0) |

**Progresso Rusby v0.7.0**: Token completi su tutte le chain (ERC-20, SPL, CW-20, Jetton). NFT display per EVM e Solana via Alchemy/Helius API.

### 2.4 Transazioni

| Feature | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---------|----------|-------|---------|-------|-------|
| Send nativo | ✅ | ✅ | ✅ | ✅ | ✅ (incl. BTC) |
| Send token | ✅ | ✅ | ✅ | ✅ | ✅ ERC-20 (v0.3.0) |
| Cronologia TX | ✅ | ✅ | ✅ | ✅ | ✅ basica (v0.3.0) |
| Gas estimation | ✅ | ✅ avanzato | ✅ | ✅ | ✅ (base) |
| Speed up/cancel TX | ✅ | ✅ | ❌ | ❌ | ❌ |
| Batch TX | ❌ | ✅ | ❌ | ❌ | ❌ |
| EIP-1559 (Type 2) | ✅ | ✅ | ✅ | N/A | ✅ |
| TX simulation | ❌ | ✅ | ✅ | ❌ | ✅ (v0.6.0) |

**Progresso Rusby v0.6.0**: TX simulation pre-firma, phishing detection, scam address warning e token approval management colmano il gap sicurezza con Rabby.

### 2.5 Integrazione dApp

| Feature | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---------|----------|-------|---------|-------|-------|
| Injected provider | ✅ EIP-1193 | ✅ EIP-1193+6963 | ✅ (custom) | ✅ (custom) | ✅ EIP-1193+6963 (v0.4.0) |
| WalletConnect v2 | ✅ | ✅ | ✅ | ✅ | ✅ (v0.5.0) |
| dApp permission management | ✅ | ✅ avanzato | ✅ | ✅ | ✅ basica (v0.4.0) |
| Sign message (EIP-191/712) | ✅ | ✅ | ✅ | N/A | ✅ (v0.4.0) |
| Background service worker | ✅ | ✅ | ✅ | ✅ | ✅ (v0.4.0) |
| Snaps/plugin system | ✅ | ❌ | ❌ | ❌ | ❌ |

**Progresso Rusby v0.5.0**: Injected provider EIP-1193/6963, firma EIP-191/712, background SW, gestione permessi dApp, WalletConnect v2 con CAIP-2 mapping. Connettività dApp completa.

### 2.6 Sicurezza

| Feature | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---------|----------|-------|---------|-------|-------|
| Encryption standard | AES-GCM | AES-GCM | Non divulgato | AES-GCM | AES-256-GCM |
| KDF | PBKDF2 | PBKDF2 | Non divulgato | scrypt | PBKDF2 100k |
| Auto-lock | ✅ | ✅ | ✅ | ✅ | ✅ (v0.3.0) |
| Phishing detection | ✅ | ✅ avanzato | ✅ | ❌ | ✅ blocklist+heuristic (v0.6.0) |
| TX simulation pre-firma | ❌ | ✅ | ✅ | ❌ | ✅ eth_call (v0.6.0) |
| Token approval check | ❌ | ✅ (revoke.cash) | ❌ | N/A | ✅ revoke UI (v0.6.0) |
| Scam address warning | ✅ | ✅ | ✅ | ❌ | ✅ (v0.6.0) |
| Open source | ✅ | ✅ | ❌ | ✅ | ✅ |
| Core in Rust/WASM | ❌ (JS) | ❌ (JS) | ✅ (parziale) | ❌ (TS) | ✅ (100%) |

**Vantaggio Rusby**: Core 100% Rust è un vantaggio reale — nessuna classe di bug tipiche di JavaScript (prototype pollution, type coercion, timing attacks).

### 2.7 UX e Piattaforme

| Feature | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---------|----------|-------|---------|-------|-------|
| Chrome extension | ✅ | ✅ | ✅ | ✅ | ✅ |
| Firefox | ✅ | ✅ | ❌ | ✅ | ❌ |
| Mobile app | ✅ | ❌ | ✅ | ✅ | ❌ |
| Desktop app | ❌ | ❌ | ❌ | ❌ | ❌ (Tauri planned) |
| Swap integrato | ✅ | ✅ (multi-DEX) | ✅ | ❌ | ✅ 0x API (v0.7.0) |
| Bridge integrato | ✅ | ✅ | ✅ | ✅ (IBC) | ❌ |
| Staking | ✅ | ❌ | ✅ | ✅ | ❌ |
| Fiat on-ramp | ✅ | ❌ | ✅ | ❌ | ❌ |
| Dark mode | ✅ | ✅ | ✅ | ✅ | ✅ |
| Popup + fullpage | ✅ | ✅ | ✅ | ✅ | ✅ |
| i18n/Lingue | ✅ ~30 | ✅ ~10 | ✅ ~20 | ✅ ~10 | ✅ 9 (v0.7.0) |

### 2.8 DeFi e Servizi Integrati

| Feature | MetaMask | Rabby | Phantom | Keplr | Rusby |
|---------|----------|-------|---------|-------|-------|
| DEX aggregator | ✅ (swap) | ✅ (multi-source) | ✅ (Jupiter) | ❌ | ✅ (0x API v2) |
| Cross-chain bridge | ✅ | ✅ | ✅ | ✅ (IBC) | ❌ |
| Staking nativo | ✅ (ETH) | ❌ | ✅ (SOL) | ✅ (ATOM+) | ❌ |
| Yield aggregator | ❌ | ❌ | ❌ | ❌ | ❌ |
| Gas sponsorship | ❌ | ✅ | ❌ | ❌ | ❌ |

---

## 3. Analisi SWOT — Rusby Wallet

### Strengths (Punti di forza)
- **Rust puro**: Sicurezza a livello di linguaggio, nessun overhead JS, performance prevedibili
- **Multi-chain nativo**: EVM + Solana + TON + Cosmos dallo stesso seed — unico nel mercato
- **Codebase leggero**: ~10.000 LOC totali vs ~500k+ di MetaMask — facile da auditare e mantenere
- **Crittografia corretta**: AES-256-GCM + PBKDF2 con parametri adeguati, dipendenze auditate
- **Open source con licenza GPL-3.0**: Trasparenza totale
- **Architettura moderna**: Leptos 0.7 con fine-grained reactivity, Manifest v3

### Weaknesses (Debolezze)
- **Zero utenti**: Nessuna traction, nessuna community
- **Feature gap**: Mancano hardware wallet, staking, bridge
- **Team**: Progetto singolo (?) vs team di 50-200+ dei competitor
- **Nessun modello di revenue**: Sostenibilità a lungo termine incerta
- **Nessun audit di sicurezza esterno**: Fondamentale per un wallet
- **TON come chain unica**: Nessun competitor mainstream supporta TON — potrebbe essere un vantaggio o un rischio

### Opportunities (Opportunità)
- **Segmento "security-first"**: Nessun wallet compete sul "100% Rust, zero JavaScript"
- **TON + multi-chain**: L'ecosistema TON cresce rapidamente, pochi wallet lo supportano nativamente
- **Fatica da MetaMask**: Molti utenti cercano alternative più sicure e veloci
- **Tauri desktop**: Nessun wallet mainstream ha un client desktop nativo
- **Developer audience**: Rust developer che vogliono un wallet auditabile
- **Account abstraction (EIP-4337)**: Nessun competitor lo ha integrato nativamente — first mover advantage possibile

### Threats (Minacce)
- **MetaMask Snaps**: MetaMask sta diventando multi-chain tramite plugin — erode il vantaggio di Rusby
- **Phantom multi-chain**: Phantom si è espanso da Solana a EVM+BTC e continua ad aggiungere chain
- **Wallet integrati nei browser**: Brave Wallet, Opera Wallet riducono la necessità di estensioni
- **Regolamentazione**: Requisiti MiCA/Travel Rule potrebbero richiedere KYC — complicazione enorme
- **Smart wallet (Coinbase, Safe)**: Account abstraction con social recovery è il futuro — l'HD wallet tradizionale potrebbe diventare obsoleto

---

## 4. Posizionamento Strategico

### Dove NON competere
- **Volume utenti mainstream**: MetaMask e Phantom hanno vantaggi di rete insormontabili
- **Ecosistema dApp**: MetaMask ha l'effetto network — è lo "standard de facto"
- **Mobile**: Richiede investimento enorme, Phantom e MetaMask dominano

### Dove competere (nicchia difendibile)

#### Posizionamento proposto: **"Il wallet più sicuro per utenti multi-chain"**

1. **Security-first narrative**: "100% Rust, zero JavaScript, completamente auditabile"
2. **Multi-chain nativo reale**: Non Snaps, non bridge — derivazione nativa per ogni chain
3. **Trasparenza**: Open source, codebase compatto, audit pubblici
4. **Developer-friendly**: Il wallet che i Rust developer costruiscono per sé stessi
5. **TON first-mover**: Primo wallet open source con TON + EVM + Solana + Cosmos nativo

---

## 5. Roadmap

La roadmap dettagliata è nel file [ROADMAP.md](ROADMAP.md).

---

## 6. Conclusione

Rusby Wallet ha un posizionamento unico: **l'unico wallet open source con core 100% Rust che supporta nativamente EVM, Solana, TON e Cosmos**. Nessun competitor offre questa combinazione.

Il gap con i big è significativo in termini di feature, ma il vantaggio tecnico (sicurezza a livello di linguaggio, codebase auditabile, performance WASM) è reale e difendibile.

La strategia vincente non è replicare MetaMask, ma costruire il wallet che gli utenti consapevoli della sicurezza scelgono quando vogliono un'alternativa trustless e trasparente. Il percorso è: prima diventare **usabile** (v0.3), poi **connesso** (v0.4-v0.5), poi **sicuro** (v0.6), poi **DeFi-ready** (v0.7), e infine **completo** (v1.0).

Con v0.7.0, Rusby ha colmato i gap su swap e NFT: lo swap integrato via 0x API e il display NFT per EVM (Alchemy) e Solana (Helius DAS) avvicinano significativamente l'esperienza utente a quella dei competitor principali. Il supporto token è ora completo su tutte le chain (ERC-20, SPL, CW-20, Jetton) e l'interfaccia è disponibile in 9 lingue. Le debolezze rimanenti — hardware wallet, staking e bridge — sono le prossime priorità.

Il vero moat di Rusby è Rust: meno superficie di attacco, meno classi di bug, e un codebase che un singolo auditor può leggere in un giorno. In un mondo dove i wallet gestiscono miliardi di dollari, questo conta.
