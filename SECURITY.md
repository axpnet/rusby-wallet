# Security Policy

## Modello di sicurezza

Rusby Wallet adotta un modello **zero-trust locale**: il seed non esiste mai in chiaro su disco, la password non viene mai persistita, e la chiave di firma vive in memoria solo per il tempo strettamente necessario alla firma della transazione.

## Crittografia

### Cifratura del seed (at rest)

| Parametro | Valore |
|-----------|--------|
| Algoritmo | **AES-256-GCM** (authenticated encryption) |
| Key derivation | **PBKDF2-HMAC-SHA256** |
| Iterazioni | **100.000** |
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

## Gestione dei secret

### Cosa viene salvato su disco

- **Vault cifrato** (`wallet_store` in localStorage / chrome.storage.local): contiene salt, nonce e ciphertext. Senza la password, è computazionalmente impossibile recuperare il seed.
- **Preferenza tema** (`theme`): stringa "dark" o "light". Nessun dato sensibile.

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

- **Furto del vault**: senza la password, il seed è protetto da AES-256-GCM + PBKDF2 100k iterazioni
- **Brute force password**: 100.000 iterazioni PBKDF2 rendono impraticabile il brute force su password ragionevolmente complesse
- **Replay attack su cifratura**: salt e nonce random per ogni operazione
- **Tampering del vault**: AES-GCM è authenticated encryption, qualsiasi modifica viene rilevata
- **Leak di chiavi in memoria**: il seed viene decifrato solo quando necessario (unlock/firma) e scartato subito dopo

### Limitazioni note

- **Password deboli**: la sicurezza del vault dipende dalla complessità della password scelta dall'utente. Non è ancora implementato un password strength meter
- **Memory dump**: in ambiente browser/WASM, non è possibile garantire il wiping della memoria (il garbage collector potrebbe mantenere copie)
- **Estensioni malevole**: altre estensioni Chrome con permessi elevati potrebbero potenzialmente accedere allo storage
- **XSS**: un attacco XSS sulla pagina potrebbe accedere allo stato in memoria. La CSP nel manifest v3 mitiga questo rischio
- **Supply chain**: le dipendenze Rust sono verificate ma non tutte formalmente auditate

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
| 0.2.x | Si |
| < 0.2.0 | No |
