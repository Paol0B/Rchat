# RChat Qt Client

Client Qt crossplatform per RChat con interfaccia grafica moderna.

## Caratteristiche

- ✅ **Interfaccia grafica moderna** con PyQt6
- 🔒 **Crittografia E2EE** tramite bindings Rust nativi
- 🎨 **Tema scuro/chiaro** moderno
- 💬 **UI simile ai moderni client di messaggistica**
- 🔐 **Tutte le funzionalità del client terminale**:
  - Creazione chat (1:1 e gruppi)
  - Join tramite codice condiviso
  - Forward secrecy con chain keys
  - Firma digitale messaggi (Ed25519)
  - Retry automatico messaggi falliti
  - Notifiche user joined/left

## Installazione

### Prerequisiti

- Python 3.8+
- Rust toolchain (per compilare i bindings)
- Qt6 (installato automaticamente con PyQt6)

### Setup

1. **Installa dipendenze Python:**
```bash
cd python-qt-client
pip install -r requirements.txt
```

2. **Compila i binding Rust con maturin:**
```bash
pip install maturin
maturin develop --release
```

Questo compilerà il modulo `rchat_core` che espone le funzionalità crittografiche del modulo `common` di Rust.

## Utilizzo

```bash
python main.py
```

### Prima connessione

1. Inserisci il tuo username
2. Configura server e porta (default: 127.0.0.1:6666)
3. Opzionale: abilita modalità insecure per certificati self-signed
4. Opzionale: usa codici numerici a 6 cifre invece di base64

### Creazione chat

1. Clicca "Crea Nuova Chat"
2. Scegli tipo (1:1 o Gruppo)
3. Il codice viene generato e copiato automaticamente negli appunti
4. Condividi il codice con gli altri partecipanti

### Unirsi a una chat

1. Clicca "Unisciti a una Chat"
2. Incolla il codice ricevuto (CTRL+V o tasto destro)
3. Conferma

### Chat

- **Invia messaggi**: Scrivi e premi Enter o clicca Invia
- **Scroll**: Usa mouse wheel o frecce
- **Esci**: Pulsante "Esci" in alto a destra

### Indicatori di stato

- ✓ = Messaggio inviato e verificato
- ✗ = Messaggio non inviato (retry automatico)
- ⚠ = Messaggio inviato ma firma non verificata

## Architettura

```
python-qt-client/
├── rchat-bindings/          # Bindings PyO3 per modulo common Rust
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs          # Wrapper Python per crypto+protocol
├── rchat/
│   ├── network.py          # Gestione connessione TLS
│   └── ui/
│       ├── main_window.py  # Finestra principale e schermate
│       └── styles.py       # Temi dark/light
├── main.py                 # Entry point
├── pyproject.toml          # Config maturin
└── requirements.txt
```

## Bindings Rust

Il modulo `rchat_core` espone:

### Classi

- `PyChatKey`: Chiave di crittografia (XChaCha20-Poly1305)
- `PyIdentityKey`: Chiave identità (Ed25519 per firme)
- `PyChainKey`: Chiave per forward secrecy
- `PyMessagePayload`: Payload messaggio
- `PyClientMessage`: Messaggi client→server
- `PyServerMessage`: Messaggi server→client

### Funzioni

- `py_generate_chat_code()`: Genera codice chat sicuro (512-bit base64)
- `py_generate_numeric_chat_code()`: Genera codice numerico 6 cifre
- `py_chat_code_to_room_id(code)`: Hash BLAKE3+SHA3-512 del codice

## Sicurezza

Stessa implementazione crittografica del client Rust:

- **XChaCha20-Poly1305**: Crittografia autenticata
- **Argon2id**: Key derivation resistente a GPU
- **Ed25519**: Firma digitale messaggi
- **BLAKE3 + SHA3-512**: Hashing room IDs
- **Forward Secrecy**: Chain keys ratcheting

## Compatibilità

- ✅ Linux
- ✅ macOS  
- ✅ Windows

## Note

- Il server non vede mai i codici chat originali
- Tutti i messaggi sono crittografati end-to-end
- Le chiavi non lasciano mai i client
- Nessuna persistenza: tutto è volatile
