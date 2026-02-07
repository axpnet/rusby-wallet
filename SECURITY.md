# Security Policy

## Modello di sicurezza

Rusby Wallet adotta un modello **zero-trust locale**: il seed non esiste mai in chiaro su disco, la password non viene mai persistita, e la chiave di firma vive in memoria solo per il tempo strettamente necessario alla firma della transazione.

## Crittografia

### Cifratura del seed (at rest)

| Parametro | Valore |
|-----------|--------|
| Algoritmo | **AES-256-GCM** (authenticated encryption) |
| Key derivation | **PBKDF2-HMAC-SHA256** |
| Iterazioni | **600.000** (OWASP 2024 recommendation) |
| Salt | **32 byte** (random, unico per vault) |
| Nonce | **12 byte** (random, unico per operazione) |
| Implementazione | Crate `aes-gcm 0.10` + `pbkdf2 0.12` (pure Rust) |

Il seed BIP39 (64 byte) viene cifrato con AES-256-GCM prima di essere salvato. La chiave AES viene derivata dalla password utente tramite PBKDF2 con salt random da 32 byte. Ogni operazione di cifratura genera salt e nonce nuovi, quindi lo stesso seed cifrato due volte produce output diversi.

### Derivazione chiavi (HD wallet)

| Chain | Algoritmo | Curva | Standard |
|-------|-----------|-------|----------|
| EVM (ETH, Polygon, BSC, OP, Base, Arb) | BIP32/BIP44 | secp256k1 | m/44'/60'/0'/0/0 |
| Bitcoin | BIP32/BIP44 | secp256k1 | m/44'/0'/0'/0/0 |
| Solana | SLIP-10 | Ed25519 | m/44'/501'/0'/0' |
| TON | SLIP-10 | Ed25519 | m/44'/607'/0'/0' |
| Cosmos Hub | BIP32/BIP44 | secp256k1 | m/44'/118'/0'/0/0 |
| Osmosis | BIP32/BIP44 | secp256k1 | m/44'/118'/0'/0/0 |

Implementazione: crate `k256 0.13` (secp256k1), `ed25519-dalek 2` (Ed25519), `bip39 2`, `bip32 0.5`.

### Firma transazioni

| Chain | Tipo TX | Firma |
|-------|---------|-------|
| EVM | EIP-1559 (Type 2) con RLP encoding | ECDSA secp256k1 su keccak256(tx) |
| Solana | Legacy message (SystemProgram.transfer) | Ed25519 |
| TON | Wallet v4r2 external message | Ed25519 |
| Cosmos | Amino JSON sign doc | ECDSA secp256k1 su SHA256(sign_doc) |
| Bitcoin | P2WPKH (BIP-143 SegWit) | ECDSA secp256k1 su SHA256d(sighash) |

## Gestione dei secret

### Cosa viene salvato su disco

- **Vault cifrato** (`wallet_store` in localStorage / chrome.storage.local): contiene salt, nonce e ciphertext. Senza la password, è computazionalmente impossibile recuperare il seed.
- **Preferenza tema** (`theme`): codice ThemeId (default, light, midnight, ocean, forest, atelier, professional). Nessun dato sensibile.
- **Tema custom** (`custom_theme`): JSON HashMap con 8 colori CSS. Nessun dato sensibile.

### Cosa NON viene mai salvato

- Password utente
- Seed / mnemonic in chiaro
- Chiavi private
- Chiavi AES derivate

### Ciclo di vita dei secret in memoria

1. **Unlock**: l'utente inserisce la password → PBKDF2 deriva la chiave AES → AES-GCM decifra il seed → gli indirizzi vengono derivati → il seed viene scartato
2. **Firma TX**: l'utente reinserisce la password nella modale di conferma → il seed viene decifrato on-demand → la chiave privata viene derivata → la TX viene firmata → seed e chiave vengono scartati
3. **Lock**: lo stato viene resettato a default, tutti i dati sensibili in memoria vengono sovrascritti

## Storage

| Contesto | Storage primario | Sync |
|----------|-----------------|------|
| Web app | `localStorage` | — |
| Estensione Chrome | `chrome.storage.local` | Sync bidirezionale con localStorage all'avvio |

L'estensione Chrome usa `chrome.storage.local` per persistenza cross-context (popup, background, content script). All'avvio viene eseguito un sync verso localStorage per letture sincrone nel runtime WASM.

## Dipendenze crittografiche

Tutte le dipendenze sono **pure Rust** (no binding C/C++), compilate a WebAssembly:

| Crate | Versione | Scopo | Audit |
|-------|----------|-------|-------|
| `aes-gcm` | 0.10 | Cifratura simmetrica | RustCrypto (audited) |
| `pbkdf2` | 0.12 | Key derivation da password | RustCrypto |
| `k256` | 0.13 | secp256k1 ECDSA | RustCrypto (audited) |
| `ed25519-dalek` | 2 | Ed25519 signatures | Dalek (audited) |
| `sha2` | 0.10 | SHA-256/SHA-512 | RustCrypto |
| `hmac` | 0.12 | HMAC per BIP32 | RustCrypto |
| `bip39` | 2 | Mnemonic BIP39 | Community |
| `bip32` | 0.5 | HD key derivation | RustCrypto |
| `rand` | 0.8 | CSPRNG | Rust standard |
| `getrandom` | 0.2 (js) | Entropia in WASM (Web Crypto API) | Rust standard |

### Fonte di entropia

In ambiente WASM, `getrandom` usa `crypto.getRandomValues()` del browser (Web Crypto API), che è un CSPRNG conforme alle specifiche W3C.

## Threat model

### Minacce mitigate

- **Furto del vault**: senza la password, il seed è protetto da AES-256-GCM + PBKDF2 600k iterazioni
- **Brute force password**: 600.000 iterazioni PBKDF2 (conforme OWASP 2024) rendono impraticabile il brute force su password ragionevolmente complesse
- **Replay attack su cifratura**: salt e nonce random per ogni operazione
- **Tampering del vault**: AES-GCM è authenticated encryption, qualsiasi modifica viene rilevata
- **Leak di chiavi in memoria**: il seed viene decifrato solo quando necessario (unlock/firma) e scartato subito dopo

### Limitazioni note

- **Password deboli**: la sicurezza del vault dipende dalla complessità della password scelta dall'utente. Un password strength meter è disponibile dalla v0.3.0
- **Memory dump**: in ambiente browser/WASM, non è possibile garantire il wiping della memoria (il garbage collector potrebbe mantenere copie). `zeroize` viene usato dove possibile
- **Estensioni malevole**: altre estensioni Chrome con permessi elevati potrebbero potenzialmente accedere allo storage
- **XSS**: un attacco XSS sulla pagina potrebbe accedere allo stato in memoria. La CSP nel manifest v3 mitiga questo rischio
- **Supply chain**: le dipendenze Rust sono verificate ma non tutte formalmente auditate

## Sicurezza avanzata (v0.6.0)

### TX simulation pre-firma

Prima di mostrare la conferma di invio su chain EVM, Rusby esegue una simulazione della transazione tramite `eth_call` sull'RPC corrente. Se la simulazione indica che la TX fallirebbe on-chain (revert, out of gas), viene mostrato un warning `SecurityWarning` con severità Medium. Il modulo `decode_revert_reason()` decodifica il selettore `Error(string)` (0x08c379a0) per mostrare il motivo del revert in chiaro.

**File**: `crates/wallet-ui/src/rpc/simulate.rs`

### Phishing detection

Ogni richiesta dApp (EIP-1193 o WalletConnect) viene verificata contro:

1. **Blocklist** — ~50 domini phishing noti (uniswap-airdrop.com, metamask-io.com, ecc.)
2. **Typosquatting** — Levenshtein distance ≤ 2 da ~22 domini legittimi (uniswap.org, metamask.io, ecc.)
3. **TLD sospetti** — Domini `.xyz`, `.tk`, `.ml`, `.ga`, `.cf`, `.gq`, `.top`, `.buzz`, `.icu` con keyword crypto
4. **Sostituzione cifre/lettere** — Rilevamento di pattern come `un1swap`, `meta4ask`

Se un dominio è sospetto, un banner rosso non-dismissable viene mostrato nella pagina di approvazione. L'utente può comunque procedere.

**File**: `crates/wallet-core/src/security/phishing.rs` (7 test)

### Scam address warning

Quando l'utente inserisce un indirizzo destinatario nella pagina Send, viene verificato contro:

1. **Database indirizzi noti** — Indirizzi scam/burn confermati
2. **Self-send** — Invio a sé stessi (probabilmente un errore)
3. **Zero-address** — Invio a `0x0000...0000` (fondi persi per sempre)

Se il rischio non è `Safe`, un warning viene mostrato in tempo reale.

**File**: `crates/wallet-core/src/security/scam_addresses.rs` (5 test)

### Token approval management

Una pagina dedicata (raggiungibile da Settings → "Gestisci Approvazioni") permette di:

1. Scansionare le approvazioni ERC-20 attive per spender noti (Uniswap V3, 1inch V5, PancakeSwap, SushiSwap, 0x Exchange Proxy) su Ethereum, Polygon, BSC, Arbitrum e Base
2. Visualizzare spender, allowance (con evidenziazione per "Unlimited")
3. Revocare approvazioni con un click (costruisce TX `approve(spender, 0)`)

**File**: `crates/wallet-ui/src/pages/approvals.rs`, `crates/wallet-ui/src/rpc/approvals.rs`

### Error boundary

Un panic hook WASM custom cattura i panic Rust e mostra un overlay DOM con messaggio di errore e bottone "Ricarica Wallet", anziché crashare silenziosamente. Il panic viene anche loggato su `console.error`.

**File**: `crates/wallet-ui/src/components/error_boundary.rs`

## Sicurezza DeFi e API (v0.7.0)

### Swap — sicurezza calldata

Lo swap integrato utilizza l'API 0x v2 per ottenere calldata di transazione (router contract address + data). La TX viene firmata localmente dall'utente nel popup WASM — le chiavi private non escono mai dal contesto locale.

**Rischi mitigati:**
- La calldata viene visualizzata all'utente prima della firma (indirizzo router, valore, gas)
- Lo swap richiede la password per decriptare il seed (stessa procedura di un invio normale)
- Solo chain EVM supportate — nessun rischio cross-chain per MVP

**Rischi residui:**
- L'API 0x è un servizio esterno — se compromesso, potrebbe fornire calldata malevola. L'utente deve verificare l'indirizzo del router contract prima di confermare
- Le API key (0x, Alchemy, Helius) sono archiviate in localStorage in chiaro — non sono secret crittografici ma credenziali di servizi rate-limited

### API Keys storage

Le API key per servizi esterni (NFT, Swap) vengono archiviate in localStorage:

| Chiave | Servizio | Rischio se esposta |
|--------|----------|-------------------|
| `alchemy_api_key` | Alchemy (NFT EVM) | Rate limit personale esaurito |
| `helius_api_key` | Helius (NFT Solana) | Rate limit personale esaurito |
| `zeroex_api_key` | 0x (Swap EVM) | Rate limit personale esaurito |

Queste chiavi non sono secret crittografici — permettono solo l'accesso a API con rate limiting. Non possono essere usate per accedere ai fondi dell'utente.

### Content Security Policy — NFT images

La CSP è stata aggiornata per consentire il caricamento di immagini NFT da CDN esterni:

```
img-src 'self' data: https://nft-cdn.alchemy.com https://res.cloudinary.com https://ipfs.io https://gateway.pinata.cloud https://arweave.net https://img-cdn.magiceden.dev https://*.nftstorage.link;
```

La funzione `sanitize_image_url()` in `wallet-core/src/nft.rs` converte URL IPFS e Arweave in gateway HTTPS, riducendo il rischio di caricamento da protocolli non standard.

### Internazionalizzazione (i18n)

Le traduzioni sono stringhe statiche compilate nel binario WASM (array `&[(&str, &str)]`). Non vengono caricate da fonti esterne, eliminando rischi di injection via traduzioni.

## Sicurezza WASM e memoria (v0.8.1)

### Ottimizzazione memoria WASM

In v0.8.1 è stato risolto un problema di Out-Of-Memory (OOM) WASM causato da cloni ripetuti di `WalletState` (struct con `HashMap`/`Vec`):

- **`signal.with()` ovunque**: tutti i `signal.get()` su WalletState convertiti a `signal.with(|s| s.field)` — borrow anziché clone
- **Memos**: `is_unlocked`, `active_chain`, `current_address` spezzano la cascata reattiva Effect→update→Effect nella dashboard
- **Interval leak fix**: l'auto-refresh 30s era dentro un Effect, creando N interval sovrapposti ad ogni re-run. Ora è un setup one-time con `forget()`
- **`with_untracked()`**: i timer callbacks usano `signal.with_untracked()` per borrow senza tracking reattivo

### Zeroize completo

Dalla v0.8.0 (audit GPT Codex), `zeroize` è applicato su tutti i percorsi di firma per tutte le chain (EVM, Solana, TON, Cosmos, Bitcoin). Il seed e le chiavi private vengono azzerati in memoria subito dopo l'uso.

### Debug redaction

`EncryptedData` implementa `Debug` manualmente con redaction del ciphertext — nessun dato sensibile viene stampato in log o panic message.

---

## Segnalazione vulnerabilità

Se trovi una vulnerabilità di sicurezza in Rusby Wallet:

1. **Non aprire una issue pubblica**
2. Invia una email a: *[da configurare]*
3. Includi una descrizione dettagliata e, se possibile, un proof of concept
4. Attendi conferma prima di divulgare pubblicamente

Ci impegniamo a rispondere entro 48 ore e a rilasciare una patch entro 7 giorni per vulnerabilità critiche.

## Versioni supportate

| Versione | Supportata |
|----------|-----------|
| 0.8.x | Si |
| 0.7.x | Si |
| 0.5.x–0.6.x | Aggiornare consigliato |
| < 0.5.0 | No |
