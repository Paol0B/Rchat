# Rchat Examples

Esempi pratici di utilizzo di Rchat.

## 🎯 Scenario 1: Chat 1:1 Locale

Due utenti sulla stessa macchina vogliono chattare in modo sicuro.

### Terminale 1: Server

```bash
$ cargo run --bin server
🔒 Rchat Server v0.1.0
🚀 Avvio server sulla porta 6666...
✅ Server in ascolto su 0.0.0.0:6666
⚠️  ATTENZIONE: Tutti i dati sono volatili e NON persistiti su disco

📡 Nuova connessione da 127.0.0.1:45678
```

### Terminale 2: Alice (crea la chat)

```bash
$ cargo run --bin client -- --username Alice
```

**Schermata iniziale:**
```
┌─────────────────────────────────────────────────┐
│ Welcome                                         │
│ ╦═╗┌─┐┬ ┬┌─┐┌┬┐                                │
│ ╠╦╝│  ├─┤├─┤ │                                  │
│ ╩╚═└─┘┴ ┴┴ ┴ ┴                                  │
│                                                  │
│ 🔒 End-to-End Encrypted Chat                   │
│ ━━━━━━━━━━━━━━━━━━━━━━━━━━━                    │
└─────────────────────────────────────────────────┘
```

1. Premi `1` (Crea chat)
2. Premi `1` (Chat 1:1)
3. Sistema genera codice: `xJ4k9L2mN3pQ5rS7tU9vW1xY2zA4bC6d`
4. **Copia questo codice** per Bob

### Terminale 3: Bob (si unisce alla chat)

```bash
$ cargo run --bin client -- --username Bob
```

1. Premi `2` (Unisciti a chat)
2. Incolla il codice: `xJ4k9L2mN3pQ5rS7tU9vW1xY2zA4bC6d`
3. Premi `ENTER`

### Schermata Chat (per entrambi)

```
┌─────────────────────────────────────────────────┐
│ 🔒 Chat: xJ4k9L2mN3pQ | User: Alice            │
├─────────────────────────────────────────────────┤
│ Messaggi (E2EE)                                 │
│ [14:30] <Alice>: Ciao Bob!                     │
│ [14:31] <Bob>: Ciao Alice! Come va?            │
│ [14:32] <Alice>: Tutto bene, questa chat è     │
│                   completamente crittografata! │
├─────────────────────────────────────────────────┤
│ Messaggio                                       │
│ Test message...                                 │
├─────────────────────────────────────────────────┤
│ [ENTER] Invia | [ESC] Esci | [CTRL+C] Termina │
└─────────────────────────────────────────────────┘
```

## 🎯 Scenario 2: Chat di Gruppo (Team)

Alice, Bob e Charlie vogliono una chat di gruppo sicura.

### Alice (crea gruppo)

```bash
$ cargo run --bin client -- --username Alice --host 192.168.1.100
```

1. Premi `1` (Crea chat)
2. Premi `2` (Chat di gruppo)
3. Codice generato: `aB1cD2eF3gH4iJ5kL6mN7oP8qR9sT0u`
4. Condividi con Bob e Charlie

### Bob si unisce

```bash
$ cargo run --bin client -- --username Bob --host 192.168.1.100
```

1. Premi `2` (Unisciti)
2. Inserisci: `aB1cD2eF3gH4iJ5kL6mN7oP8qR9sT0u`

### Charlie si unisce

```bash
$ cargo run --bin client -- --username Charlie --host 192.168.1.100
```

1. Premi `2` (Unisciti)
2. Inserisci: `aB1cD2eF3gH4iJ5kL6mN7oP8qR9sT0u`

### Output Chat di Gruppo

```
┌─────────────────────────────────────────────────┐
│ 🔒 Chat: aB1cD2eF3gH4iJ | User: Alice          │
├─────────────────────────────────────────────────┤
│ Messaggi (E2EE)                                 │
│ Bob si è unito                                  │
│ Charlie si è unito                              │
│ [15:00] <Alice>: Benvenuti nel gruppo!         │
│ [15:01] <Bob>: Ciao a tutti!                   │
│ [15:02] <Charlie>: Hey team! 👋                │
│ [15:03] <Alice>: Pronti per il meeting?        │
├─────────────────────────────────────────────────┤
```

## 🎯 Scenario 3: Rete Remota

Server pubblico, client remoti.

### Server (VPS pubblico)

```bash
# Su server 203.0.113.10
$ cargo run --release --bin server

# O con systemd
$ sudo systemctl start rchat-server
```

### Client 1 (Casa)

```bash
$ cargo run --bin client -- \
    --host 203.0.113.10 \
    --port 6666 \
    --username Alice
```

### Client 2 (Ufficio)

```bash
$ cargo run --bin client -- \
    --host 203.0.113.10 \
    --port 6666 \
    --username Bob
```

## 🔧 Uso Avanzato

### Variabili d'Ambiente

```bash
# Configura host di default
export RCHAT_HOST=192.168.1.100
export RCHAT_PORT=6666

# Usa con client
cargo run --bin client -- --username Alice
```

### Build Release per Performance

```bash
# Compila ottimizzato
cargo build --release

# Esegui binari release
./target/release/server
./target/release/client --username Alice
```

### Docker (esempio)

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin server

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/server /usr/local/bin/
COPY --from=builder /app/server.crt /etc/rchat/
COPY --from=builder /app/server.key /etc/rchat/
EXPOSE 6666
CMD ["server"]
```

```bash
# Build e run
docker build -t rchat-server .
docker run -p 6666:6666 rchat-server
```

## 🐛 Troubleshooting

### Errore: "Certificati TLS mancanti"

```bash
$ ./generate_certs.sh
# O manualmente:
$ openssl req -x509 -newkey rsa:4096 -nodes \
    -keyout server.key -out server.crt -days 365 \
    -subj '/CN=localhost'
```

### Errore: "Chat non trovata"

Il server è stato riavviato o la chat è scaduta. Crea una nuova chat.

### Errore: "Chat piena"

Chat 1:1 ha già 2 partecipanti. Usa una chat di gruppo.

### Errore: "Connessione rifiutata"

```bash
# Verifica che il server sia in esecuzione
$ ss -tlnp | grep 6666

# Verifica firewall
$ sudo ufw allow 6666/tcp  # Ubuntu
$ sudo firewall-cmd --add-port=6666/tcp  # Fedora
```

## 📊 Testing di Carico

### Stress Test (molti client)

```bash
# Script di test
for i in {1..10}; do
    cargo run --bin client -- --username "User$i" &
done
```

### Monitoraggio Server

```bash
# Connessioni attive
$ ss -tn | grep :6666 | wc -l

# Memoria utilizzata
$ ps aux | grep server

# Network traffic
$ sudo iftop -i eth0 -f "port 6666"
```

## 🔐 Verifica Sicurezza

### Wireshark: Verifica Crittografia

```bash
# Cattura traffico
$ sudo tcpdump -i lo -w rchat.pcap port 6666

# Analizza con Wireshark
# Verifica che:
# 1. Handshake TLS 1.3
# 2. Application Data crittografata
# 3. Nessun plaintext visibile
```

### Memory Dump: Verifica Zeroization

```bash
# Crea memory dump (richiede gdb)
$ gdb -p $(pgrep client)
(gdb) gcore client-dump.core
(gdb) quit

# Cerca stringhe sospette
$ strings client-dump.core | grep -i "secret\|password\|key"
# Dovrebbe essere vuoto per chiavi zeroizzate
```

## 📚 Scenari Reali

### Use Case 1: Whistleblowing Sicuro

Reporter e fonte si scambiano informazioni sensibili:

1. Fonte crea chat 1:1
2. Condivide codice via ProtonMail/Signal
3. Conversazione E2EE su Rchat
4. Dopo la chat: entrambi escono, dati cancellati dalla RAM

### Use Case 2: Team Remoto Temporaneo

Team distribuito per meeting confidenziale:

1. Leader crea gruppo (max 8)
2. Condivide codice via canale sicuro
3. Meeting via Rchat con chat testuale
4. Fine meeting: tutti escono, nessun log

### Use Case 3: Gaming LAN Party

Amici in LAN vogliono chat privata durante gaming:

1. Host avvia server su LAN
2. Ogni player si connette con username
3. Chat crittografata durante il gaming
4. Nessuna traccia dopo la sessione

---

**💡 Tip**: Combina Rchat con altri tool per sicurezza layered:
- VPN per anonimato IP
- Tor per anonimato rete
- Signal/Wire per condividere codici
- OTP per autenticazione out-of-band
