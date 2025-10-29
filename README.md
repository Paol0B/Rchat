# 🔒 Rchat - End-to-End Encrypted Chat

Sistema di chat moderna in Rust con crittografia end-to-end completa, architettura client-server asincrona e interfaccia terminale (TUI) intuitiva.

## ✨ Caratteristiche

### Sicurezza Estrema
- **End-to-End Encryption (E2EE)** usando ChaCha20-Poly1305
- **Derivazione chiavi con HKDF-SHA256** dal codice chat
- **TLS 1.3** per tutte le connessioni client-server (rustls)
- **Nessun storage persistente**: tutti i dati esistono solo in RAM
- **Zeroizzazione automatica** di chiavi e dati sensibili (zeroize crate)
- **Server completamente oblivious**: non può leggere i messaggi
- **Codici chat crittograficamente sicuri**: 256-bit random, base64url

### Architettura Moderna
- **Server asincrono** con Tokio (porta 6666)
- **Gestione concorrenza** con Arc<Mutex> e canali mpsc
- **Workspace Cargo** con 3 crate: server, client, common
- **Serializzazione binaria** efficiente con bincode
- **Dipendenze aggiornate** (ottobre 2025)

### Interfaccia Utente
- **TUI reattiva** con Ratatui e Crossterm
- **ASCII art** minimalista nella schermata iniziale
- **Chat real-time** con timestamp [HH:MM]
- **Scroll automatico** dei messaggi
- **Copia/Incolla**: CTRL+V per incollare codici chat 📋
- **Auto-copy**: Codice chat copiato automaticamente alla creazione 📋
- **Notifiche** di entrata/uscita utenti

### Tipi di Chat
1. **1:1**: Massimo 2 partecipanti
2. **Gruppo**: Massimo 8 partecipanti (configurabile)

## 📋 Requisiti

- Rust 1.75+ (edizione 2021)
- OpenSSL per generazione certificati (opzionale per demo)

## 🚀 Setup e Compilazione

### 1. Clona il repository

```bash
cd /home/paol0b/sources/Rchat
```

### 2. Genera certificati TLS (self-signed per demo)

```bash
./generate_certs.sh
```

Oppure manualmente:

```bash
openssl req -x509 -newkey rsa:4096 -nodes \
  -keyout server.key -out server.crt -days 365 \
  -subj '/CN=localhost'
```

**IMPORTANTE**: In produzione, usa certificati firmati da una CA affidabile.

### 3. Compila il progetto

```bash
cargo build --release
```

## 🎮 Uso

### Avvia il Server

**Server standard (codici base64 completi - più sicuro):**
```bash
cargo run --bin server --release
```

**Server con codici numerici a 6 cifre (più semplice da condividere):**
```bash
cargo run --bin server --release -- --numeric-codes
```

⚠️ **ATTENZIONE**: I codici numerici hanno solo ~20 bit di entropia (1 milione di combinazioni) rispetto ai 256 bit dei codici completi. Sono più facili da digitare ma meno sicuri contro attacchi brute-force.

Parametri del server:
- `--host`: Indirizzo di bind (default: 0.0.0.0)
- `--port`: Porta del server (default: 6666)
- `--numeric-codes`: Usa codici a 6 cifre invece di base64 lunghi

Il server si avvia e attende connessioni.

### Avvia il Client

```bash
cargo run --bin client --release -- --host 127.0.0.1 --port 6666 --username Alice
```

Parametri:
- `--host`: Indirizzo IP del server (default: 127.0.0.1)
- `--port`: Porta del server (default: 6666)
- `--username`: Il tuo nome utente (richiesto)
- `--insecure`: Accetta certificati self-signed (⚠️ SOLO per testing!)

**Per testing locale con certificati self-signed:**

```bash
cargo run --bin client --release -- --username Alice --insecure
```

⚠️ **IMPORTANTE**: L'opzione `--insecure` disabilita la verifica dei certificati TLS e deve essere usata SOLO per testing in ambiente locale. NON usarla mai in produzione!

### Flusso di Utilizzo

1. **Schermata Welcome**:
   - Premi `1` per creare una nuova chat
   - Premi `2` per unirti a una chat esistente
   - Premi `Q` per uscire

2. **Creare una Chat**:
   - Scegli tipo: `1` per 1:1, `2` per gruppo
   - Il sistema genera un codice univoco:
     - Formato standard: `xJ4k9L2m...` (base64, 43 caratteri)
     - Formato numerico: `123456` (6 cifre) - solo se server avviato con `--numeric-codes`
   - **Il codice viene copiato automaticamente nella clipboard!** 📋
   - Condividi il codice con gli altri partecipanti

3. **Unirsi a una Chat**:
   - Inserisci il codice ricevuto
   - Oppure incolla con:
     - `CTRL+V` (potrebbe non funzionare in tutti i terminali)
     - `SHIFT+Insert` (standard Linux) 📋
     - **Click destro del mouse** 🖱️
   - Premi `ENTER` per confermare

4. **Chat**:
   - Scrivi il messaggio e premi `ENTER` per inviare
   - Incolla testo con `CTRL+V`, `SHIFT+Insert` o **click destro** 🖱️
   - I messaggi sono crittografati automaticamente
   - Usa `↑` / `↓` per scorrere i messaggi
   - `PageUp` / `PageDown` per scroll veloce
   - `Home` per andare all'inizio, `End` per andare alla fine
   - Premi `ESC` per uscire dalla chat
   - Premi `CTRL+C` per terminare il client

## 🔐 Architettura di Sicurezza

### Crittografia E2EE

```
┌─────────┐                 ┌────────┐                 ┌─────────┐
│ Client A│                 │ Server │                 │ Client B│
└────┬────┘                 └───┬────┘                 └────┬────┘
     │                          │                           │
     │  1. Crea Chat            │                           │
     ├─────────────────────────>│                           │
     │                          │                           │
     │  2. Chat Code (256-bit)  │                           │
     │<─────────────────────────┤                           │
     │                          │                           │
     │  3. Deriva chiave E2EE   │                           │
     │     (HKDF-SHA256)        │                           │
     │                          │                           │
     │                          │  4. Join con code         │
     │                          │<──────────────────────────┤
     │                          │                           │
     │                          │                           │  5. Deriva stessa chiave
     │                          │                           │     (HKDF-SHA256)
     │                          │                           │
     │  6. Messaggio plaintext  │                           │
     │     "Ciao!"              │                           │
     │                          │                           │
     │  7. Encrypt con          │                           │
     │     ChaCha20-Poly1305    │                           │
     │                          │                           │
     │  8. Ciphertext           │                           │
     ├─────────────────────────>│                           │
     │                          │                           │
     │                          │  9. Inoltro ciphertext    │
     │                          │     (server non decripta!)│
     │                          ├──────────────────────────>│
     │                          │                           │
     │                          │                           │  10. Decrypt con
     │                          │                           │      ChaCha20-Poly1305
     │                          │                           │
     │                          │                           │  11. "Ciao!"
```

### Derivazione Chiavi

```rust
chat_code (256-bit random) 
    ↓
base64url encoding
    ↓
HKDF-SHA256(chat_code, salt=None, info="rchat-e2ee-v1")
    ↓
encryption_key (256-bit)
    ↓
ChaCha20-Poly1305 cipher
```

### Protezione TLS 1.3

- Tutte le connessioni client-server usano TLS 1.3
- Protegge metadati e previene MITM
- Il server comunque NON può leggere i messaggi (E2EE)

## 📦 Struttura del Progetto

```
Rchat/
├── Cargo.toml              # Workspace root
├── common/                 # Libreria condivisa
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── protocol.rs     # Definizioni messaggi
│       └── crypto.rs       # E2EE con ChaCha20-Poly1305
├── server/                 # Server binario
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # Server TLS asincrono
│       └── chat.rs         # Gestione chat rooms
├── client/                 # Client binario
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # Client TLS + TUI
│       └── ui.rs           # Interfaccia Ratatui
├── server.crt              # Certificato TLS (generato)
├── server.key              # Chiave privata (generato)
└── README.md
```

## 🛡️ Garanzie di Sicurezza

✅ **E2EE completo**: Il server non può leggere i messaggi  
✅ **Nessun logging**: I messaggi non vengono mai scritti su disco  
✅ **RAM volatile**: Tutti i dati esistono solo in memoria  
✅ **Zeroizzazione**: Chiavi e dati sensibili sovrascritti alla disconnessione  
✅ **TLS 1.3**: Connessioni client-server crittografate  
✅ **Codici sicuri**: 256-bit random con entropia da OsRng  
✅ **AEAD**: ChaCha20-Poly1305 garantisce autenticità e confidenzialità  

## ⚠️ Limitazioni e Disclaimer

- **Demo/PoC**: Non auditato per uso produzione
- **Certificati self-signed**: Sostituisci con certificati CA validi
- **Nessuna persistenza**: I messaggi non consegnati vengono persi
- **Solo online**: Non c'è queue per messaggi offline
- **Rete locale consigliata**: Esporre su Internet richiede hardening

## 🧪 Testing

### Test Locale

1. Avvia il server in un terminale:
   ```bash
   cargo run --bin server
   ```

2. Avvia il primo client (Alice):
   ```bash
   cargo run --bin client -- -u Alice
   ```

3. Crea una chat e copia il codice generato

4. Avvia il secondo client (Bob):
   ```bash
   cargo run --bin client -- -u Bob
   ```

5. Unisciti con il codice copiato

6. Inizia a chattare in modo sicuro! 🔒

### Verifica Crittografia

Puoi usare Wireshark per confermare che:
- Le connessioni usano TLS 1.3
- I payload sono completamente crittografati
- Il server non può vedere i contenuti dei messaggi

## 📚 Dipendenze Principali

| Crate | Versione | Uso |
|-------|----------|-----|
| tokio | 1.41 | Async runtime |
| rustls | 0.23 | TLS 1.3 |
| chacha20poly1305 | 0.10 | E2EE AEAD cipher |
| hkdf | 0.12 | Key derivation |
| zeroize | 1.8 | Memory zeroization |
| ratatui | 0.29 | TUI framework |
| serde | 1.0 | Serialization |
| clap | 4.5 | CLI parsing |

## 🤝 Contribuire

Questo è un progetto educativo. Suggerimenti per migliorare la sicurezza sono benvenuti!

## 📄 Licenza

MIT License - Vedi LICENSE file

## 🔗 Risorse

- [ChaCha20-Poly1305 IETF RFC](https://tools.ietf.org/html/rfc8439)
- [HKDF RFC 5869](https://tools.ietf.org/html/rfc5869)
- [TLS 1.3 RFC 8446](https://tools.ietf.org/html/rfc8446)
- [Ratatui Documentation](https://ratatui.rs/)
- [Tokio Documentation](https://tokio.rs/)

---

**⚡️ Fatto con Rust 🦀 | 🔒 Privacy First | 💾 Zero Persistence**
