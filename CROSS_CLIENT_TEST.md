# Test di Compatibilità Cross-Client

## Problema Identificato
I client Python e Rust stanno usando **chat codes diversi**, quindi generano chiavi di crittografia diverse e non possono decriptare i messaggi reciproci.

## Verifica Completata
✅ Le chain keys derivate sono **identiche** tra Python e Rust per lo stesso chat_code
✅ La serializzazione del MessagePayload è **identica** (202 bytes per "BestMak", "asd")
✅ Gli algoritmi crittografici sono **compatibili**

## Test da Eseguire

### Test 1: Codice Numerico Semplice
1. **Client 1 (Rust)**:
   ```bash
   ./target/release/client --username Alice --insecure --numeric-codes
   ```
   - Premere 'C' per Create
   - Annotare il codice numerico a 6 cifre (es: `123456`)

2. **Client 2 (Python)**:
   ```bash
   cd python-qt-client && python main.py
   ```
   - Cliccare "Join Chat"
   - Inserire ESATTAMENTE lo stesso codice (es: `123456`)

3. **Test Messaggi**:
   - Alice (Rust) invia: "Hello from Rust"
   - Python dovrebbe ricevere e decriptare correttamente
   - Python invia: "Hello from Python"
   - Alice (Rust) dovrebbe ricevere e decriptare correttamente

### Test 2: Codice Condiviso
1. **Client 1 (Python)** - Crea chat
2. Il codice viene copiato automaticamente negli appunti
3. **Client 2 (Rust)** - Join con codice incollato
4. Testare invio bidirezionale

## Debug Output
Entrambi i client ora mostrano:
- `[DEBUG] Chain index used: X` - Indice chain usato per crittografia
- `[DEBUG] Message key (first 16 bytes): XXXX` - Chiave messaggio
- `[DEBUG] Encrypted data: X bytes` - Dimensione dato crittografato

**Le chiavi devono essere identiche per lo stesso chain index se i client usano lo stesso chat_code!**

## Esempio Output Corretto
```
Client A invia messaggio 0:
[DEBUG] Chain index used: 0
[DEBUG] Message key (first 16 bytes): 7812a1cf6d625149...

Client B riceve:
[DEBUG] Testing index 0:
[DEBUG]   Test key (first 16 bytes): 7812a1cf6d625149...  ← DEVE ESSERE UGUALE!
[DEBUG] ✅ Successfully decrypted at index 0
```

## Se il Test Fallisce
Se le chiavi sono diverse:
1. Verificare che entrambi i client usino lo stesso chat_code esatto
2. Verificare che il room_id sia lo stesso (deriva dal chat_code)
3. Controllare i log per vedere quale chat_code viene usato

## Comandi Rapidi Test

Terminal 1 - Server:
```bash
cd /home/paol0b/sources/Rchat
./target/release/server --insecure
```

Terminal 2 - Client Rust:
```bash
cd /home/paol0b/sources/Rchat
./target/release/client --username RustUser --insecure --numeric-codes
# Premere 'C', annotare codice
```

Terminal 3 - Client Python:
```bash
cd /home/paol0b/sources/Rchat/python-qt-client
python main.py
# Join con codice del client Rust
```
