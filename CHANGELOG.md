# Changelog

## [0.8.1] - 2026-02-07

### Aggiunto — UX & Layout Overhaul

#### Layout fullpage Talisman-like
- **TopNav**: navigazione orizzontale nell'header (Home, Send, Receive, History, Settings) — visibile solo in fullpage
- **ChainSidebar**: sidebar sinistra con selettore chain, icona, nome e bilancio per ogni chain — visibile solo in fullpage
- **Layout flex**: header → [sidebar | content] — struttura flex-row con chain sidebar 240px + main content
- **FullpageMode newtype**: risolto conflitto TypeId per `ReadSignal<bool>` nel context Leptos
- **Popup invariato**: layout ≤500px con bottom nav + ChainSelector nella dashboard

#### UX onboarding & login
- **SVG spinner animato**: triple-arc spinner inline nei pulsanti durante operazioni crypto
- **Testo loading dinamico**: progressione fase per fase (BIP-39 seed → PBKDF2 key stretching → derivazione HD keys)
- **Tab order corretto**: `tabindex="-1"` sui toggle visibilità password — Tab va direttamente tra i campi input
- **Security badge**: badge "Enterprise-grade Security" nell'onboarding con 3 check (AES-256-GCM, PBKDF2 600k, chiavi locali)

#### Performance WASM (fix OOM)
- **`signal.with()` ovunque**: eliminati tutti i `signal.get()` su WalletState — zero cloni di HashMap/Vec
- **Memos dashboard**: `is_unlocked`, `active_chain`, `current_address` spezzano la cascata reattiva Effect→update→Effect
- **Interval leak fix**: auto-refresh 30s spostato fuori da Effect (singola istanza con `forget()`, no leak)
- **`with_untracked()`**: timer callbacks usano borrow senza tracking reattivo

#### Nuovi file
- `components/top_nav.rs` — barra navigazione orizzontale fullpage
- `components/chain_sidebar.rs` — sidebar selettore chain fullpage

#### i18n
- 8 nuove chiavi × 9 lingue: fasi di caricamento (`loading.*`) + badge sicurezza (`security.*`)
- ~304 chiavi totali in 9 lingue

### Metriche
- 123 test unitari passanti
- ~17.500 LOC Rust + ~770 JS
- 0 errori compilazione

## [0.8.0] - 2026-02-06

### Aggiunto — Theme Template System

#### Sistema di temi estensibile
- **7 temi predefiniti**: Default (dark purple), Light, Midnight (blue), Ocean (teal), Forest (green), Atelier (warm orange, by GPT 5.2 Codex), Professional (corporate blue, by Qwen3Max)
- **Tema custom**: 8 color picker con anteprima live + derivazione automatica variabili secondarie
- **Hero preview**: anteprima animata del tema corrente con card mockup e accent dots (by GPT 5.2 Codex)
- **Griglia selettore 3 colonne**: mini-preview bg+accent per ogni tema
- **Schema facile per sviluppatori**: aggiungere un tema = 1 file `.rs` con 17 variabili CSS + 1 variante enum + 1 chiave i18n
- **Architettura**: CSS custom properties impostate via `element.style.setProperty()` — nessun blocco CSS aggiuntivo per tema
- **Migrazione automatica**: vecchi valori localStorage `"dark"`/`"light"` convertiti ai nuovi codici

#### Modulo `theme/`
- `theme/mod.rs` — `ThemeId` enum, `apply_theme()`, `ThemeSelector`, `CustomThemeEditor`, helper colori
- `theme/default.rs` — tema scuro predefinito (#6c5ce7 accent)
- `theme/light.rs` — tema chiaro
- `theme/midnight.rs` — scuro profondo, accent blu (#5b8def)
- `theme/ocean.rs` — scuro, accent teal (#00b4d8)
- `theme/forest.rs` — scuro, accent verde (#2d9f5a)
- `theme/atelier.rs` — caldo, accent arancione (#d28c45)
- `theme/professional.rs` — corporate, accent blu (#4299e1), radius ridotto (10px/6px)

#### i18n
- 18 nuove chiavi tema in 9 lingue (settings.appearance, theme.*, color picker labels)

#### Sicurezza (Audit esterno GPT Codex 5.2)
- **CRIT-1+2**: Riscrittura completa derivazione TON v4r2 con BOC parser, cell hash, state_init hash, address decoder
- **HIGH-3**: Zeroize seed/private key su tutti i percorsi firma (Bitcoin, Solana, Cosmos, TON)
- **HIGH-4**: Overflow check `checked_mul`/`checked_add` nei parser importi (EVM, Solana, TON)
- **MED-5**: Origin check su `wallet_switchEthereumChain` in background.js
- **MED-6**: `postMessage` con `targetOrigin: window.location.origin` (era `'*'`)
- **MED-7**: `sanitize_image_url` restrittiva — scheme sconosciuti restituiscono stringa vuota
- **MED-8**: Lockfile `package-lock.json` per walletconnect (supply chain)

### Metriche
- 123 test unitari passanti (da 118) — 5 nuovi per TON v4r2
- ~331 chiavi i18n in 9 lingue
- 8 file nuovi + ~20 file modificati

## [0.7.0] - 2026-02-05

### Aggiunto — Milestone 5 "DeFi Wallet"

#### FASE 0: Internazionalizzazione (i18n)
- Sistema i18n con 9 lingue: EN, IT, ES, FR, DE, PT, ZH, JA, KO
- ~313 chiavi di traduzione con fallback automatico a inglese
- `t("key")` reattivo nei componenti Leptos
- Persistenza lingua in localStorage
- Modulo `i18n/` con file per lingua

#### FASE 1: Utility
- **Testnet toggle** — switch mainnet/testnet globale con RPC dedicati
- **Address book** — etichette per indirizzi frequenti, auto-completamento nella pagina Send
- **Export/import backup** — export cifrato AES-256-GCM del vault completo, import con verifica

#### FASE 2: Token Avanzati
- **CW-20 (Cosmos/Osmosis)** — query balance CosmWasm, invio con MsgExecuteContract Amino JSON
- **Jetton (TON)** — token list predefinita, query via toncenter v3 API + fallback runGetMethod
- **IBC token display** — visualizzazione token IBC con denom hash su Cosmos/Osmosis
- **Token discovery esteso** — dropdown token nella pagina Send per Cosmos/Osmosis/TON

#### FASE 3: NFT + Swap + Toast
- **NFT Display** per EVM e Solana
  - EVM: Alchemy NFT API v3 (`getNFTsForOwner`) per Ethereum, Polygon, Base, Arbitrum, Optimism
  - Solana: Helius DAS API (`getAssetsByOwner`)
  - Griglia 2 colonne con card (immagine, nome, collezione)
  - Modale dettaglio con immagine grande, descrizione, contratto, token ID, standard
  - Stato vuoto con hint per configurare API key
  - Sanitizzazione URL immagini (IPFS, Arweave, HTTP → HTTPS gateway)

- **Swap integrato** per chain EVM
  - 0x Swap API v2 (price + quote con calldata TX)
  - Token comuni predefiniti per 6 chain (ETH, USDC, USDT, DAI, WETH, WBTC)
  - Slippage configurabile (0.3%, 0.5%, 1%, 3%)
  - Display quote: rate, gas stimato, fonti DEX
  - Esecuzione TX con calldata da 0x API, firma EVM standard
  - Guard: messaggio per chain non-EVM

- **Toast notifications**
  - 4 tipi: Success, Error, Warning, Info
  - Auto-dismiss dopo 5 secondi
  - Animazione slide-in, posizione fixed top-right

- **Dashboard**: 4 action buttons (Send, Receive, Swap, NFT)
- **Settings**: sezione API Keys (Alchemy, Helius, 0x) con salvataggio in localStorage
- **CSP**: aggiornata `img-src` per NFT CDN (alchemy, cloudinary, ipfs, pinata, arweave, magiceden, nftstorage)

### Metriche
- 118 test unitari passanti (da 87) — 15 nuovi per NFT + Swap, 16 per i18n/utility
- ~313 chiavi i18n in 9 lingue
- 5 file nuovi + ~14 file modificati per FASE 3

## [0.6.0] - 2026-02-02

### Aggiunto — Milestone 4 "Smart Wallet"

#### Sicurezza Avanzata
- **TX simulation pre-firma** — simulazione via `eth_call`, warning se TX fallirebbe, decode revert reason `Error(string)`
- **Phishing detection** — blocklist ~50 domini + typosquatting (Levenshtein ≤2) + heuristic TLD sospetti + keyword crypto — 7 test
- **Scam address warning** — database indirizzi noti + risk assessment (self-send, zero-address, known scam) — 5 test
- **Token approval management** — pagina gestione approvazioni ERC-20, scan spender noti (Uniswap, 1inch, PancakeSwap, SushiSwap, 0x) su 5 chain, revoke
- **SecurityWarning component** — componente riusabile con 3 livelli severità (Low/Medium/High)

#### Qualità
- **Refactoring send.rs** — logica TX estratta in `tx_send/` con sotto-moduli per chain (665→315 righe UI)
- **Error boundary UI** — panic hook WASM con overlay DOM + bottone ricarica
- **Logging** — modulo `logging.rs` con macro `log_info!/warn!/error!/debug!`

### Metriche
- 87 test unitari passanti (da 73) — 14 nuovi: 7 phishing + 5 scam + 2 erc20

## [0.5.0] - 2026-02-02

### Aggiunto — Milestone 3 "WalletConnect"

#### WalletConnect v2
- **WalletConnect SDK** bundlato (`@walletconnect/web3wallet` via esbuild → ESM)
- **Wrapper RusbyWC** con storage adapter per `chrome.storage` (sopravvive restart SW)
- **Keep-alive Service Worker** con `chrome.alarms` ogni 24s
- **Event handlers WC**: session_proposal → popup, session_request → coda approvazione
- **Mapping CAIP-2 ↔ ChainId** (eip155, solana, cosmos, bip122, ton) — 7 test
- **Pagina WalletConnect** in UI — input URI `wc:...`, lista sessioni attive, disconnessione
- **Pagina proposta sessione WC** — mostra dApp, chain richieste, approva/rifiuta
- **Firma completa nel popup** — modale password, decrypt seed on-demand, personal_sign + EIP-712
- **Namespace builder** per sessioni WC (metodi + eventi per namespace)
- **Configurazione WC Project ID** in Settings

### Metriche
- 73 test unitari passanti (da 66) — 7 nuovi per CAIP-2

## [0.4.0] - 2026-02-02

### Aggiunto — Milestone 2 "Connected Wallet"

- **Background Service Worker** per estensione Chrome
  - Routing messaggi tra popup, content script e dApp
  - Gestione stato lock wallet centralizzato
  - Coda richieste pendenti con persistenza in chrome.storage
  - Origini approvate con gestione permessi

- **Content Script** — bridge pagina web ↔ background
  - Iniezione inpage.js nel contesto pagina
  - Relay bidirezionale via port long-lived
  - Filtering messaggi per evitare conflitti

- **Injected Provider EIP-1193** (`window.rusby`)
  - `request({method, params})` → Promise
  - Eventi: `connect`, `disconnect`, `chainChanged`, `accountsChanged`
  - Metodi: `eth_requestAccounts`, `eth_accounts`, `eth_chainId`, `eth_sendTransaction`, `personal_sign`, `eth_signTypedData_v4`, `wallet_switchEthereumChain`
  - Legacy: `enable()`, `send()`, `sendAsync()`

- **EIP-6963 Multi-Provider Discovery**
  - Annuncio provider con uuid, name, icon SVG, rdns `io.rusby.wallet`
  - Ascolto `eip6963:requestProvider` e ri-annuncio

- **Firma messaggi EIP-191 (personal_sign)** in Rust puro
  - Prefix `\x19Ethereum Signed Message:\n` + keccak256 + secp256k1
  - Recovery address da firma
  - 6 test unitari

- **Firma dati tipizzati EIP-712** in Rust puro
  - Domain separator, struct hash, sign_typed_data_hash
  - hash_eip712_domain() per calcolo domain separator
  - 6 test unitari

- **Pagina approvazione dApp** nel popup
  - Mostra origine, metodo, parametri della richiesta
  - Bottoni Approva/Rifiuta con feedback
  - Apertura automatica da URL param `?approve=requestId`

- **Gestione dApp connesse** in Settings
  - Lista origini approvate
  - Bottone "Revoca" per ciascuna origine

- **CSP per web app standalone** via meta tag in index.html

### Sicurezza

- Chiavi private MAI esposte fuori dal popup WASM
- Background service worker gestisce solo routing e stato sessione
- Permessi dApp granulari per origine

### Metriche

- 66 test unitari passanti (da 54) — 12 nuovi per EIP-191/712
- 3 file JS nuovi per architettura 3-contesti (background, content-script, inpage)

## [0.3.0] - 2026-02-02

### Aggiunto — Milestone 1 "Usable Wallet"

- **Bitcoin P2WPKH completo** — derivazione, firma TX, balance e broadcast
  - Derivazione indirizzo BIP84 `m/84'/0'/0'/0/0` → bech32 (`bc1q...`)
  - Firma TX SegWit con BIP-143 sighash, DER encoding, witness serialization
  - RPC via mempool.space API: balance, UTXO fetch, fee estimation, broadcast
  - Moduli: `chains/bitcoin.rs`, `tx/bitcoin.rs`, `rpc/bitcoin.rs`

- **Token ERC-20** — display balance e invio per 6 chain EVM
  - Token predefiniti: USDT, USDC, DAI, WETH, WBTC
  - Encoding ABI: `balanceOf`, `transfer`
  - Sezione "Tokens" nella dashboard con balance reali

- **Token SPL** — display balance per Solana
  - Token predefiniti: USDC, USDT, WSOL, JUP
  - Fetch via `getTokenAccountsByOwner` RPC
  - Associated Token Account (ATA) derivation

- **Cronologia TX basica** per tutte le chain
  - EVM: Etherscan-like API per 6 chain
  - Solana: `getSignaturesForAddress`
  - TON: toncenter `/getTransactions`
  - Cosmos: LCD `/cosmos/tx/v1beta1/txs`
  - Pagina History con direction, importo, link a block explorer

- **Portfolio totale in USD**
  - Fetch prezzi da CoinGecko API
  - Equivalente USD sotto ogni balance + totale portfolio
  - Cache in localStorage, refresh ogni 60 secondi

- **Validazione forza password** nella creazione wallet
  - Enum `PasswordStrength` (Weak/Fair/Strong) con indicatore colorato
  - Blocco creazione wallet se password è Weak

- **Auto-lock con timeout** (implementato, default OFF)
  - Timer inattività con reset su click/keypress
  - Pagina Settings con toggle ON/OFF e dropdown timeout (1/5/15/30 min)

- **Pagina Settings** nella navigazione

### Sicurezza

- **Fix derivazione Cosmos**: sostituito SHA256 troncato con `RIPEMD160(SHA256(pubkey))` — ora conforme allo standard Cosmos SDK
- **Zeroize chiavi in memoria**: `zeroize` su seed decriptato in `encrypt()`, `decrypt()`, `create_wallet()`, `unlock_wallet()`
- Dipendenza `ripemd = "0.1"` e `zeroize = { version = "1", features = ["derive"] }`

### Metriche

- 54 test unitari passanti (da 41)
- 13 nuovi test: Bitcoin derivazione (4), Bitcoin TX (5), ERC-20 encoding (4)

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
