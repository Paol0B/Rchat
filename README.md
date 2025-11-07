# 🔒 Rchat v3.0 - Military-Grade End-to-End Encrypted Chat

Modern chat system in Rust with **EXTREME** end-to-end encryption, asynchronous client-server architecture, and intuitive terminal user interface (TUI).

## ✨ Features

### 🛡️ Extreme Security v3.0 (Military-Grade+)
- **End-to-End Encryption (E2EE)** using **XChaCha20-Poly1305** (192-bit nonce, AEAD)
- **Message Padding** (256-byte blocks) - Hides message length from traffic analysis
- **Forward Secrecy v3** with HKDF-SHA512 chain ratcheting
  - New encryption key per message via cryptographically secure HKDF
  - Even if current key is compromised, past messages remain secure
  - Constant-time operations prevent timing attacks
- **Message Signing v3** with **Ed25519** + **BLAKE3 Commitment**
  - Every message cryptographically signed by sender
  - BLAKE3 hash commitment for message integrity
  - Prevents spoofing, tampering, and impersonation attacks
  - Visual verification indicators (✓ verified, ⚠ unverified, ✗ failed)
- **Argon2id v3** EXTREME parameters for key derivation
  - **256-512 MB memory** for GPU/ASIC attack resistance (vs 128 MB v2)
  - **6-8 iterations** (vs 4 iterations v2)
  - 8 parallel threads for modern CPUs
  - Numeric codes: 512 MB, 8 iterations (compensates low entropy)
  - Protection against timing attacks and side-channel attacks
- **Triple-Hash v3** for room IDs: **BLAKE3 → SHA3-512 → Argon2id**
  - Prevents brute-force attacks on room IDs
  - Server cannot reverse-engineer chat codes
  - 32 MB Argon2id for server performance balance
- **Timestamp Validation** - ±5 minute window prevents replay attacks
- **512-bit chat codes** (vs 256-bit standard) for quantum resistance
- **Constant-time comparisons** using `subtle` crate (timing attack protection)
- **Sequence numbers** for message ordering and replay attack protection
- **Domain separation** in all cryptographic operations (prevents cross-protocol attacks)
- **TLS 1.3** for all client-server connections (rustls)
- **No persistent storage**: all data exists only in RAM
- **Automatic zeroization** of keys and sensitive data (zeroize crate)
- **Server zero-knowledge**: server never knows original chat codes
- **AEAD (Authenticated Encryption)**: XChaCha20-Poly1305 ensures authenticity and confidentiality

### 🆕 New in v3.0
- ✅ **HKDF-SHA512** for chain key derivation (RFC 5869 compliant)
- ✅ **Message padding** - 256-byte blocks hide message lengths
- ✅ **Triple-hash room IDs** with Argon2id (brute-force resistant)
- ✅ **Timestamp validation** - ±5 minute window
- ✅ **Message commitment** - BLAKE3 hash for integrity
- ✅ **Constant-time operations** - Timing attack protection
- ✅ **Extreme Argon2id** - 256-512 MB, 6-8 iterations
- ✅ **Domain separation** - All crypto operations isolated
- ✅ **Windows terminal fix** - No more duplicated characters (cciiaaoo → ciao)

### Modern Architecture
- **Asynchronous server** with Tokio (port 6666)
- **Concurrency management** with Arc<Mutex> and mpsc channels
- **Cargo workspace** with 3 crates: server, client, common
- **Efficient binary serialization** with bincode
- **Up-to-date dependencies** (November 2025)
- **Cross-platform** - Windows, Linux, macOS

### User Interface
- **Reactive TUI** with Ratatui and Crossterm
- **Minimalist ASCII art** on welcome screen
- **Real-time chat** with [HH:MM] timestamps
- **Automatic message scrolling**
- **Copy/Paste**: CTRL+V to paste chat codes 📋
- **Auto-copy**: Chat code copied automatically on creation 📋
- **User join/leave notifications**
- **Windows Terminal compatible** - Fixed character duplication bug

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

### Key Derivation v3.0

```rust
chat_code (512-bit random, generated by client) 
    ↓
room_id = BLAKE3(chat_code) → SHA3-512(blake3_hash) → Argon2id(sha3_hash) [TRIPLE hash]
    ↓
chat_code (shared out-of-band with other participants)
    ↓
Argon2id-v3(chat_code, memory=256MB, iterations=6, parallelism=8)
    ↓
encryption_key (256-bit)
    ↓
HKDF-SHA512 chain ratcheting (per-message keys)
    ↓
XChaCha20-Poly1305 cipher (192-bit nonce)
    ↓
Apply padding (256-byte blocks)
    ↓
Encrypted message
```

**Argon2id v3 Security**:
- **EXTREME GPU-Resistant**: 256-512 MB memory makes GPU attacks economically impossible
- **ASIC-Resistant**: Memory-hard design specific against dedicated hardware  
- **Timing Protection**: Constant-time operation prevents side-channel attacks
- **PHC Winner**: Password Hashing Competition winner (2015)
- **Numeric codes**: 512 MB, 8 iterations (compensates 20-bit entropy weakness)
- **Post-quantum ready**: Extreme parameters for future quantum resistance

**v3 Improvements**:
- Triple-hash room IDs (BLAKE3 → SHA3-512 → Argon2id)
- HKDF-SHA512 instead of simple BLAKE3 for chain keys
- Message padding to hide lengths (256-byte blocks)
- Timestamp validation (±5 minute window)
- Message commitment with BLAKE3 hash
- Constant-time comparisons throughout

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

## 🛡️ Security Guarantees v3.0

✅ **Complete E2EE**: Server cannot read messages  
✅ **Forward Secrecy v3**: HKDF-SHA512 chain ratcheting - cryptographically secure key derivation  
✅ **Message Authentication v3**: Ed25519 signatures + BLAKE3 commitment verify sender and integrity  
✅ **Replay Protection v3**: Sequence numbers + timestamp validation (±5 min window)  
✅ **Server zero-knowledge v3**: Triple-hash (BLAKE3→SHA3→Argon2id) - server cannot brute-force  
✅ **Quantum-resistant v3**: 512-bit codes + extreme Argon2id parameters  
✅ **GPU-resistant v3**: Argon2id with 256-512MB memory makes GPU farms impractical  
✅ **ASIC-resistant v3**: Memory-hard algorithm defeats dedicated hardware  
✅ **Side-channel resistant v3**: Constant-time operations using `subtle` crate  
✅ **Timing attack resistant v3**: Constant-time comparisons for all security-sensitive data  
✅ **Traffic analysis resistant v3**: Message padding hides lengths (256-byte blocks)  
✅ **No logging**: Messages never written to disk  
✅ **Volatile RAM**: All data exists only in memory  
✅ **Zeroization v3**: HKDF keys zeroized after each ratchet step  
✅ **TLS 1.3**: Encrypted client-server connections (protects metadata)  
✅ **Secure codes**: 512-bit random with OsRng entropy  
✅ **AEAD**: XChaCha20-Poly1305 ensures authenticity and confidentiality  
✅ **Client-side key derivation**: Keys derived only on clients, never on server  
✅ **Triple hashing v3**: BLAKE3 + SHA3-512 + Argon2id (no reverse engineering)  
✅ **Out-of-order protection**: Message chain synchronization handles network reordering  
✅ **Domain separation v3**: All crypto operations use unique context strings  
✅ **Message commitment v3**: BLAKE3 hash proves message integrity  
✅ **Cross-platform v3**: Works on Windows, Linux, macOS (character duplication fixed)  

### 🔬 Security Analysis Documents
- See [SECURITY_IMPROVEMENTS.md](SECURITY_IMPROVEMENTS.md) for v2→v3 changes
- See [CRYPTOGRAPHIC_ANALYSIS.md](CRYPTOGRAPHIC_ANALYSIS.md) for complete algorithm analysis  

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
| **argon2** | 0.5 | **Key derivation v3 (Argon2id, 256-512MB memory)** |
| **blake3** | 1.5 | **Modern hash & message commitment** |
| **sha3** | 0.10 | **SHA3-512 (Keccak, NIST standard)** |
| **sha2** | 0.10 | **SHA-512 for HKDF** |
| **hkdf** | 0.12 | **HKDF-SHA512 for chain ratcheting (RFC 5869)** |
| **ed25519-dalek** | 2.2 | **Ed25519 digital signatures** |
| **subtle** | 2.6 | **Constant-time operations (timing attack protection)** |
| zeroize | 1.8 | Memory zeroization |
| ratatui | 0.29 | TUI framework |
| crossterm | 0.28 | Terminal control |
| serde | 1.0 | Serialization |
| clap | 4.5 | CLI parsing |

## 🤝 Contributing

This is an educational project. Security improvement suggestions are welcome!

## 📄 License

MIT License - See LICENSE file

## 🔗 Resources

- [XChaCha20-Poly1305 RFC](https://datatracker.ietf.org/doc/html/draft-arciszewski-xchacha)
- [Argon2 RFC 9106](https://www.rfc-editor.org/rfc/rfc9106.html)
- [HKDF RFC 5869](https://www.rfc-editor.org/rfc/rfc5869.html)
- [Ed25519 RFC 8032](https://www.rfc-editor.org/rfc/rfc8032.html)
- [BLAKE3 Paper](https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf)
- [SHA-3 FIPS 202](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf)
- [TLS 1.3 RFC 8446](https://tools.ietf.org/html/rfc8446)
- [Ratatui Documentation](https://ratatui.rs/)
- [Tokio Documentation](https://tokio.rs/)
- [Subtle Crate](https://docs.rs/subtle/) - Constant-time operations

---

**⚡️ Built with Rust 🦀 | 🔒 Military-Grade+ Security v3.0 | 🛡️ Triple-Hash Zero-Knowledge | 💾 Zero Persistence | 🌐 Cross-Platform**
