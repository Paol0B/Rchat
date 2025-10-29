# 🔒 Rchat - End-to-End Encrypted Chat

Modern chat system in Rust with complete end-to-end encryption, asynchronous client-server architecture, and intuitive terminal user interface (TUI).

## ✨ Features

### Extreme Security (Military-Grade)
- **End-to-End Encryption (E2EE)** using **XChaCha20-Poly1305** (192-bit nonce)
- **Argon2id** for key derivation (Password Hashing Competition winner)
  - 128 MB memory for GPU/ASIC attack resistance
  - 4 iterations + 8 parallel threads
  - Protection against timing attacks and side-channel attacks
- **BLAKE3 + SHA3-512** double hashing for room IDs
- **512-bit chat codes** (vs 256-bit standard) for quantum resistance
- **TLS 1.3** for all client-server connections (rustls)
- **No persistent storage**: all data exists only in RAM
- **Automatic zeroization** of keys and sensitive data (zeroize crate)
- **Server zero-knowledge**: server never knows original chat codes
- **AEAD (Authenticated Encryption)**: XChaCha20-Poly1305 ensures authenticity and confidentiality

### Modern Architecture
- **Asynchronous server** with Tokio (port 6666)
- **Concurrency management** with Arc<Mutex> and mpsc channels
- **Cargo workspace** with 3 crates: server, client, common
- **Efficient binary serialization** with bincode
- **Up-to-date dependencies** (October 2025)

### User Interface
- **Reactive TUI** with Ratatui and Crossterm
- **Minimalist ASCII art** on welcome screen
- **Real-time chat** with [HH:MM] timestamps
- **Automatic message scrolling**
- **Copy/Paste**: CTRL+V to paste chat codes 📋
- **Auto-copy**: Chat code copied automatically on creation 📋
- **User join/leave notifications**

### Chat Types
1. **1:1**: Maximum 2 participants
2. **Group**: Maximum 8 participants (configurable)

## 📋 Requirements

- Rust 1.75+ (edition 2021)
- OpenSSL for certificate generation (optional for demo)

## 🚀 Setup and Compilation

### 1. Clone the repository

```bash
cd /home/paol0b/sources/Rchat
```

### 2. Generate TLS certificates (self-signed for demo)

```bash
./generate_certs.sh
```

Or manually:

```bash
openssl req -x509 -newkey rsa:4096 -nodes \
  -keyout server.key -out server.crt -days 365 \
  -subj '/CN=localhost'
```

**IMPORTANT**: In production, use certificates signed by a trusted CA.

### 3. Compile the project

```bash
cargo build --release
```

## 🎮 Usage

### Start the Server

```bash
cargo run --bin server --release
```

The server starts and waits for connections. It doesn't need to know whether clients will use numeric or base64 codes.

Server parameters:
- `--host`: Bind address (default: 0.0.0.0)
- `--port`: Server port (default: 6666)

### Start the Client

**Standard client (full base64 codes - more secure):**
```bash
cargo run --bin client --release -- --host 127.0.0.1 --port 6666 --username Alice
```

**Client with 6-digit numeric codes (easier to share):**
```bash
cargo run --bin client --release -- --host 127.0.0.1 --port 6666 --username Alice --numeric-codes
```

⚠️ **WARNING**: Numeric codes have only ~20 bits of entropy (1 million combinations) compared to 512 bits of full codes. They're easier to type but less secure against brute-force attacks.

Client parameters:
- `--host`: Server IP address (default: 127.0.0.1)
- `--port`: Server port (default: 6666)
- `--username`: Your username (required)
- `--insecure`: Accept self-signed certificates (⚠️ TESTING ONLY!)
- `--numeric-codes`: Generate 6-digit codes instead of base64 (easier to share)

**For local testing with self-signed certificates:**

```bash
cargo run --bin client --release -- --username Alice --insecure
```

⚠️ **IMPORTANT**: The `--insecure` option disables TLS certificate verification and should ONLY be used for testing in a local environment. NEVER use it in production!

### Usage Flow

1. **Welcome Screen**:
   - Press `1` to create a new chat
   - Press `2` to join an existing chat
   - Press `Q` to quit

2. **Create a Chat**:
   - Choose type: `1` for 1:1, `2` for group
   - System generates a unique code:
     - Standard format: `xJ4k9L2m...` (base64, 43 characters)
     - Numeric format: `123456` (6 digits) - only if client started with `--numeric-codes`
   - **Code is automatically copied to clipboard!** 📋
   - Share the code with other participants

3. **Join a Chat**:
   - Enter the received code
   - Or paste with:
     - `CTRL+V` (may not work on all terminals)
     - `SHIFT+Insert` (standard Linux) 📋
     - **Right mouse click** 🖱️
   - Press `ENTER` to confirm

4. **Chat**:
   - Type your message and press `ENTER` to send
   - Paste text with `CTRL+V`, `SHIFT+Insert` or **right click** 🖱️
   - Messages are automatically encrypted
   - Use `↑` / `↓` to scroll messages
   - `PageUp` / `PageDown` for fast scrolling
   - `Home` to go to beginning, `End` to go to end
   - Press `ESC` to exit the chat
   - Press `CTRL+C` to terminate the client

## 🔐 Security Architecture

### End-to-End Encryption

**Important**: The server never knows the original chat code! The client generates the code locally and sends only a BLAKE3+SHA3-512 hash to the server (room_id). This ensures that:
- Server cannot derive the E2EE key
- Server only relays encrypted messages
- Even with server database access, messages remain secure

```
┌─────────┐                 ┌────────┐                 ┌─────────┐
│ Client A│                 │ Server │                 │ Client B│
└────┬────┘                 └───┬────┘                 └────┬────┘
     │                          │                           │
     │  1. Generate chat_code   │                           │
     │     locally (512-bit)    │                           │
     │                          │                           │
     │  2. Calculate room_id =  │                           │
     │     BLAKE3(SHA3-512...)  │                           │
     │                          │                           │
     │  3. Create Chat with     │                           │
     │     room_id              │                           │
     ├─────────────────────────>│                           │
     │                          │                           │
     │  4. Chat Created         │                           │
     │<─────────────────────────┤                           │
     │                          │                           │
     │  5. Derive E2EE key      │                           │
     │     (Argon2id)           │                           │
     │     from chat_code       │                           │
     │                          │                           │
     │  6. Share chat_code      │                           │
     │     (out-of-band)        ├──────────────────────────>│
     │                          │                           │
     │                          │  7. Join with room_id =   │
     │                          │     BLAKE3(SHA3-512...)   │
     │                          │<──────────────────────────┤
     │                          │                           │
     │                          │                           │  8. Derive same key
     │                          │                           │     (Argon2id)
     │                          │                           │
     │  9. Message plaintext    │                           │
     │     "Hello!"             │                           │
     │                          │                           │
     │  10. Encrypt with        │                           │
     │      XChaCha20-Poly1305  │                           │
     │                          │                           │
     │  11. Ciphertext          │                           │
     ├─────────────────────────>│                           │
     │                          │                           │
     │                          │  12. Relay ciphertext    │
     │                          │      (server can't read!) │
     │                          ├──────────────────────────>│
     │                          │                           │
     │                          │                           │  13. Decrypt with own
     │                          │                           │      key (Argon2id)
     │                          │                           │
     │                          │                           │  14. Display "Hello!"
```

### Key Derivation

```rust
chat_code (512-bit random, generated by client) 
    ↓
room_id = BLAKE3(chat_code) → SHA3-512(blake3_hash) [double hash for server]
    ↓
chat_code (shared out-of-band with other participants)
    ↓
Argon2id(chat_code, memory=128MB, iterations=4, parallelism=8)
    ↓
encryption_key (256-bit)
    ↓
XChaCha20-Poly1305 cipher (192-bit nonce)
```

**Argon2id Security**:
- **GPU-Resistant**: 128 MB memory makes GPU attacks economically impractical
- **ASIC-Resistant**: Memory-hard design specific against dedicated hardware
- **Timing Protection**: Constant-time operation to prevent side-channel attacks
- **PHC Winner**: Password Hashing Competition winner (2015)

### TLS 1.3 Protection

- All client-server connections use TLS 1.3
- Protects metadata and prevents MITM
- Server still CANNOT read messages (E2EE)

## 📦 Project Structure

```
Rchat/
├── Cargo.toml              # Workspace root
├── common/                 # Shared library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── protocol.rs     # Message type definitions
│       └── crypto.rs       # E2EE with XChaCha20-Poly1305
├── server/                 # Server binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # Async TLS server
│       └── chat.rs         # Chat room management
├── client/                 # Client binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # TLS client + TUI
│       └── ui.rs           # Ratatui interface
├── server.crt              # TLS certificate (generated)
├── server.key              # Private key (generated)
└── README.md
```

## 🛡️ Security Guarantees

✅ **Complete E2EE**: Server cannot read messages  
✅ **Server zero-knowledge**: Server never knows original chat code, only double hash (BLAKE3+SHA3-512)  
✅ **Quantum-resistant**: 512-bit chat codes for resistance to future quantum computers  
✅ **GPU-resistant**: Argon2id with 128MB memory makes GPU attacks impractical  
✅ **ASIC-resistant**: Memory-hard algorithm specific against dedicated hardware  
✅ **Side-channel resistant**: Constant-time operations in Argon2id  
✅ **No logging**: Messages never written to disk  
✅ **Volatile RAM**: All data exists only in memory  
✅ **Zeroization**: Keys and sensitive data overwritten on disconnect  
✅ **TLS 1.3**: Encrypted client-server connections (protects metadata)  
✅ **Secure codes**: 512-bit random with OsRng entropy  
✅ **AEAD**: XChaCha20-Poly1305 ensures authenticity and confidentiality  
✅ **Client-side key derivation**: Keys derived only on clients, never on server  
✅ **Double hashing**: BLAKE3 + SHA3-512 for room routing (no length-extension attacks)  

## ⚠️ Limitations and Disclaimer
- **Self-signed certificates**: Replace with valid CA certificates
- **No persistence**: Undelivered messages are lost
- **Online only**: No queue for offline messages
- **Local network recommended**: Exposing on Internet requires hardening

## 🧪 Testing

### Local Testing

1. Start the server in a terminal:
   ```bash
   cargo run --bin server
   ```

2. Start first client (Alice):
   ```bash
   cargo run --bin client -- -u Alice
   ```

3. Create a chat and copy the generated code

4. Start second client (Bob):
   ```bash
   cargo run --bin client -- -u Bob
   ```

5. Join with the copied code

6. Start chatting securely! 🔒

### Verify Encryption

You can use Wireshark to confirm that:
- Connections use TLS 1.3
- Payloads are completely encrypted
- Server cannot see message contents

## 📚 Main Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1.41 | Async runtime |
| rustls | 0.23 | TLS 1.3 |
| **chacha20poly1305** | 0.10 | **E2EE AEAD cipher (192-bit nonce)** |
| **argon2** | 0.5 | **Key derivation (Argon2id, 128MB memory)** |
| **blake3** | 1.5 | **Modern hash function** |
| **sha3** | 0.10 | **SHA3-512 (Keccak, NIST standard)** |
| zeroize | 1.8 | Memory zeroization |
| ratatui | 0.29 | TUI framework |
| serde | 1.0 | Serialization |
| clap | 4.5 | CLI parsing |

## 🤝 Contributing

This is an educational project. Security improvement suggestions are welcome!

## 📄 License

MIT License - See LICENSE file

## 🔗 Resources

- [XChaCha20-Poly1305 RFC](https://datatracker.ietf.org/doc/html/draft-arciszewski-xchacha)
- [Argon2 RFC 9106](https://www.rfc-editor.org/rfc/rfc9106.html)
- [BLAKE3 Paper](https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf)
- [SHA-3 FIPS 202](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf)
- [TLS 1.3 RFC 8446](https://tools.ietf.org/html/rfc8446)
- [Ratatui Documentation](https://ratatui.rs/)
- [Tokio Documentation](https://tokio.rs/)

---

**⚡️ Built with Rust 🦀 | 🔒 Military-Grade Security | 🛡️ Zero-Knowledge Server | 💾 Zero Persistence**
