# RChat Protocol Specification v3.0

## Table of Contents
1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Cryptographic Primitives](#cryptographic-primitives)
4. [Key Derivation](#key-derivation)
5. [Message Protocol](#message-protocol)
6. [Network Protocol](#network-protocol)
7. [Security Properties](#security-properties)
8. [Implementation Guide](#implementation-guide)

---

## Overview

RChat is an end-to-end encrypted chat application that provides:
- **Zero-Knowledge Server**: Server never sees plaintext messages or encryption keys
- **Forward Secrecy**: Past messages cannot be decrypted even if keys are compromised
- **Sender Authentication**: Each message is cryptographically signed by the sender
- **Replay Protection**: Messages include timestamps and sequence numbers
- **Post-Quantum Resistance**: Uses high-entropy keys and quantum-resistant KDFs

### Key Features
- Chat codes can be 512-bit base64url strings or 6-digit numeric codes
- Uses XChaCha20-Poly1305 for authenticated encryption (AEAD)
- Argon2id for key derivation (GPU/ASIC resistant)
- Ed25519 for digital signatures
- BLAKE3 + SHA3-512 for hashing
- No key exchange protocol needed (pre-shared secret model)

---

## Architecture

### System Components

```
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│   Client A  │         │   Server    │         │   Client B  │
│             │         │             │         │             │
│ Chat Code   │         │  Room ID    │         │ Chat Code   │
│ (Secret)    │◄───────►│ (Hash only) │◄───────►│ (Secret)    │
│             │         │             │         │             │
│ Encrypt/    │         │  Forward    │         │ Decrypt/    │
│ Sign        │         │  Messages   │         │ Verify      │
└─────────────┘         └─────────────┘         └─────────────┘
```

### Trust Model
- **Client**: Trusted, holds secrets (chat code, private keys)
- **Server**: Untrusted, only forwards encrypted messages
- **Network**: Untrusted, assumes passive and active adversaries
- **Threat Model**: Protects against compromised server, network eavesdropping, MITM attacks

---

## Cryptographic Primitives

### 1. XChaCha20-Poly1305 (AEAD Cipher)

**Purpose**: Encrypt and authenticate messages

**Parameters**:
- Key size: 256 bits (32 bytes)
- Nonce size: 192 bits (24 bytes)
- Tag size: 128 bits (16 bytes)
- Max message size: 2^64 bytes

**Properties**:
- Authenticated Encryption with Associated Data (AEAD)
- Resists key/nonce reuse better than ChaCha20 (extended nonce)
- Quantum-resistant against known attacks

**Format**:
```
Ciphertext = Nonce (24 bytes) || Encrypted_Data || Auth_Tag (16 bytes)
```

### 2. Argon2id (Key Derivation Function)

**Purpose**: Derive encryption keys from chat codes

**Algorithm**: Argon2id (hybrid of Argon2i and Argon2d)

**Properties**:
- Memory-hard (resistant to GPU/ASIC attacks)
- Side-channel resistant
- Time-memory tradeoff resistant
- Tunable difficulty

**Parameters Used**:

For chat code expansion (numeric codes only):
- Memory cost: 65536 KiB (64 MiB)
- Time cost: 3 iterations
- Parallelism: 4 threads
- Output length: 512 bits (64 bytes)
- Salt: `"rchat-numeric-salt-v2-extreme"`

For final key derivation (all codes):
- Memory cost: 131072 KiB (128 MiB)
- Time cost: 4 iterations
- Parallelism: 8 threads
- Output length: 256 bits (32 bytes)
- Salt: BLAKE3(b"rchat-e2ee-v2-salt:" || chat_secret)

For chain key initialization:
- Memory cost: 131072 KiB (128 MiB)
- Time cost: 4 iterations
- Parallelism: 8 threads
- Output length: 256 bits (32 bytes)
- Salt: `b"chain-key-init"`

### 3. Ed25519 (Digital Signatures)

**Purpose**: Authenticate message senders

**Parameters**:
- Private key: 256 bits (32 bytes)
- Public key: 256 bits (32 bytes)
- Signature: 512 bits (64 bytes)

**Properties**:
- Fast signature generation and verification
- Deterministic (no random number generation needed)
- Collision-resistant
- Existential unforgeability under chosen message attack (EUF-CMA)

### 4. BLAKE3 (Cryptographic Hash)

**Purpose**: Fast key derivation and hashing

**Parameters**:
- Output: Variable length (default 256 bits)
- Based on Bao and BLAKE2

**Properties**:
- Faster than SHA-2 and SHA-3
- Parallelizable
- Can be used as KDF, MAC, PRF, or hash

**Usage in RChat**:
- Room ID generation (first pass)
- Salt derivation for Argon2id
- Chain key ratcheting (forward secrecy)
- Message commitment hashing

### 5. SHA3-512 (Cryptographic Hash)

**Purpose**: Second-pass hashing for defense in depth

**Parameters**:
- Output: 512 bits (64 bytes)
- Based on Keccak sponge construction

**Properties**:
- NIST standard
- Different construction than SHA-2 (provides diversity)
- Resistant to length-extension attacks

**Usage in RChat**:
- Room ID generation (second pass after BLAKE3)

---

## Key Derivation

### Chat Code Format

#### Full Security Format (Recommended)
- **Length**: 512 bits (64 bytes)
- **Encoding**: Base64url (URL-safe, no padding)
- **Entropy**: 512 bits (~154 decimal digits)
- **Example**: `vQx7T2mP9R3fK8nL5pW1cY6hZ4jX0oU7tS9qD3eA5bN2gM8iV1wC6rF4kH0zY...`

#### Numeric Format (Convenience)
- **Length**: 6 digits
- **Range**: 100000 to 999999
- **Entropy**: ~19.9 bits
- **Example**: `123456`
- **Security**: Much weaker, suitable only for low-risk scenarios

### Room ID Derivation

The server never sees the chat code. Instead, clients derive a room ID:

```
room_id = SHA3-512(b"rchat-double-hash:" || BLAKE3(b"rchat-room-id-v2:" || chat_code))
```

**Step-by-step**:
1. Concatenate prefix `b"rchat-room-id-v2:"` with chat code
2. Hash with BLAKE3 → `blake3_hash` (32 bytes)
3. Concatenate `b"rchat-double-hash:"` with `blake3_hash`
4. Hash with SHA3-512 → `final_hash` (64 bytes)
5. Encode with Base64url → `room_id`

**Properties**:
- One-way function (server cannot reverse to get chat code)
- Collision-resistant (different codes → different room IDs)
- Deterministic (same code always → same room ID)
- Double-hashing provides defense in depth

### Encryption Key Derivation

**For 512-bit chat codes**:
```python
# Step 1: Decode base64url to 64 bytes
chat_secret = base64url_decode(chat_code)  # 64 bytes

# Step 2: Derive salt from chat_secret
salt_hash = BLAKE3(b"rchat-e2ee-v2-salt:" || chat_secret)
salt = salt_hash[0:32]  # First 32 bytes

# Step 3: Derive encryption key with Argon2id
encryption_key = Argon2id(
    secret=chat_secret,
    salt=salt,
    memory_cost=131072,  # 128 MiB
    time_cost=4,
    parallelism=8,
    output_length=32     # 256 bits
)
```

**For 6-digit numeric codes**:
```python
# Step 1: Expand numeric code to 64 bytes with Argon2id
numeric_bytes = chat_code.encode('utf-8')  # "123456" → 6 bytes
expanded_secret = Argon2id(
    secret=numeric_bytes,
    salt=b"rchat-numeric-salt-v2-extreme",
    memory_cost=65536,   # 64 MiB
    time_cost=3,
    parallelism=4,
    output_length=64     # 512 bits
)

# Step 2 & 3: Same as above, using expanded_secret as chat_secret
```

### Chain Key Derivation (Forward Secrecy)

Each message uses a unique encryption key derived from a ratcheting chain:

```python
# Initialize chain
chain_key = Argon2id(
    secret=chat_code.encode('utf-8'),
    salt=b"chain-key-init",
    memory_cost=131072,  # 128 MiB
    time_cost=4,
    parallelism=8,
    output_length=32
)
chain_index = 0

# Derive next key in chain
def next_chain_key(current_key, current_index):
    new_key = BLAKE3(
        b"rchat-chain-ratchet:" || 
        current_key || 
        current_index.to_bytes(8, 'little')
    )[0:32]
    return new_key, current_index + 1
```

**Properties**:
- Each message uses a different key
- Old keys are immediately discarded (forward secrecy)
- Fast ratcheting using BLAKE3
- Synchronized by including `chain_key_index` in message

---

## Message Protocol

### Message Structure

#### Plaintext Payload (Before Encryption)
```rust
struct MessagePayload {
    username: String,           // Sender's username
    content: String,            // Message content
    timestamp: i64,             // Unix timestamp (seconds)
    sequence_number: u64,       // Message sequence (per user)
    sender_public_key: Vec<u8>, // Ed25519 public key (32 bytes)
    signature: Vec<u8>,         // Ed25519 signature (64 bytes)
    chain_key_index: u64,       // For forward secrecy
    message_hash: Vec<u8>,      // BLAKE3 commitment (32 bytes)
}
```

#### Binary Serialization (Bincode Format)

All multi-byte integers use **little-endian** encoding.

```
Field                 | Type      | Size (bytes)           | Format
----------------------|-----------|------------------------|------------------------
username_len          | u64       | 8                      | Length of username
username              | [u8]      | username_len           | UTF-8 encoded string
content_len           | u64       | 8                      | Length of content
content               | [u8]      | content_len            | UTF-8 encoded string
timestamp             | i64       | 8                      | Unix timestamp (signed)
sequence_number       | u64       | 8                      | Message sequence
pubkey_len            | u64       | 8                      | Always 32
sender_public_key     | [u8]      | 32                     | Ed25519 public key
signature_len         | u64       | 8                      | Always 64
signature             | [u8]      | 64                     | Ed25519 signature
chain_key_index       | u64       | 8                      | Chain key index
hash_len              | u64       | 8                      | Always 32
message_hash          | [u8]      | 32                     | BLAKE3 hash
```

### Message Signing

**Data to sign**:
```python
signature_data = (
    content.encode('utf-8') ||
    timestamp.to_bytes(8, 'little') ||
    sequence_number.to_bytes(8, 'little')
)
signature = Ed25519_sign(private_key, signature_data)
```

### Message Commitment

**Commitment hash** (integrity proof):
```python
message_hash = BLAKE3(
    b"rchat-v3-message-commitment:" ||
    username.encode('utf-8') ||
    content.encode('utf-8') ||
    sequence_number.to_bytes(8, 'little') ||
    chain_key_index.to_bytes(8, 'little')
)
```

### Encryption Process

1. **Serialize** payload to binary (bincode format)
2. **Derive** chain key for current index
3. **Generate** random 24-byte nonce
4. **Encrypt** with XChaCha20-Poly1305:
   ```python
   ciphertext = XChaCha20Poly1305.encrypt(
       key=chain_key,
       nonce=nonce,
       plaintext=serialized_payload,
       associated_data=None
   )
   ```
5. **Concatenate**: `nonce || ciphertext || auth_tag`

### Decryption Process

1. **Extract** nonce (first 24 bytes)
2. **Extract** ciphertext (remaining bytes)
3. **Derive** chain key for received index
4. **Decrypt** and verify:
   ```python
   plaintext = XChaCha20Poly1305.decrypt(
       key=chain_key,
       nonce=nonce,
       ciphertext=ciphertext
   )
   # Raises exception if authentication fails
   ```
5. **Deserialize** binary payload
6. **Verify** signature
7. **Verify** timestamp (within ±5 minutes)
8. **Verify** message commitment hash
9. **Check** sequence number (must be increasing)

---

## Network Protocol

### Transport Layer

- **Protocol**: TLS 1.3 over TCP
- **Port**: 6666 (default, configurable)
- **Certificate**: Self-signed certificates supported with `--insecure` flag
- **Frame Format**: Length-prefixed messages

### Message Framing

Each message is prefixed with its length:

```
┌────────────────┬──────────────────────┐
│  Length (u32)  │  Message Data        │
│  (4 bytes, LE) │  (Length bytes)      │
└────────────────┴──────────────────────┘
```

**Reading a message**:
1. Read 4 bytes (u32 little-endian) → `msg_length`
2. Read `msg_length` bytes → `msg_data`
3. Deserialize `msg_data` using bincode

### Client-to-Server Messages

All client messages are enums serialized with bincode.

#### Enum Variants
```rust
enum ClientMessage {
    CreateChat = 0,
    JoinChat = 1,
    SendMessage = 2,
    LeaveChat = 3,
}
```

#### 1. CreateChat
```
Offset | Field             | Type            | Size
-------|-------------------|-----------------|--------
0      | variant           | u32 LE          | 4
4      | room_id_len       | u64 LE          | 8
12     | room_id           | [u8]            | room_id_len
       | chat_type_variant | u32 LE          | 4
       | [max_participants]| Option<u64> LE  | 0 or 9
       | username_len      | u64 LE          | 8
       | username          | [u8]            | username_len
```

**ChatType variants**:
- `OneToOne = 0`: No additional data
- `Group = 1`: Followed by `Some(max_participants)` or `None`

**Option encoding**:
- `None = 0`: Single byte 0x00
- `Some(val) = 1`: Byte 0x01 followed by value

#### 2. JoinChat
```
Offset | Field        | Type     | Size
-------|--------------|----------|--------
0      | variant      | u32 LE   | 4
4      | room_id_len  | u64 LE   | 8
12     | room_id      | [u8]     | room_id_len
       | username_len | u64 LE   | 8
       | username     | [u8]     | username_len
```

#### 3. SendMessage
```
Offset | Field             | Type     | Size
-------|-------------------|----------|--------
0      | variant           | u32 LE   | 4
4      | room_id_len       | u64 LE   | 8
12     | room_id           | [u8]     | room_id_len
       | payload_len       | u64 LE   | 8
       | encrypted_payload | [u8]     | payload_len
       | message_id_len    | u64 LE   | 8
       | message_id        | [u8]     | message_id_len
```

#### 4. LeaveChat
```
Offset | Field       | Type     | Size
-------|-------------|----------|--------
0      | variant     | u32 LE   | 4
4      | room_id_len | u64 LE   | 8
12     | room_id     | [u8]     | room_id_len
```

### Server-to-Client Messages

```rust
enum ServerMessage {
    ChatCreated = 0,
    JoinedChat = 1,
    Error = 2,
    MessageReceived = 3,
    MessageAck = 4,
    UserJoined = 5,
    UserLeft = 6,
}
```

#### 1. ChatCreated
```
Offset | Field             | Type     | Size
-------|-------------------|----------|--------
0      | variant           | u32 LE   | 4
4      | room_id_len       | u64 LE   | 8
12     | room_id           | [u8]     | room_id_len
       | chat_type_variant | u32 LE   | 4
       | [max_participants]| ...      | ...
```

#### 2. JoinedChat
```
Offset | Field             | Type     | Size
-------|-------------------|----------|--------
0      | variant           | u32 LE   | 4
4      | room_id_len       | u64 LE   | 8
12     | room_id           | [u8]     | room_id_len
       | chat_type_variant | u32 LE   | 4
       | [max_participants]| ...      | ...
       | participant_count | u64 LE   | 8
```

#### 3. Error
```
Offset | Field       | Type     | Size
-------|-------------|----------|--------
0      | variant     | u32 LE   | 4
4      | message_len | u64 LE   | 8
12     | message     | [u8]     | message_len
```

#### 4. MessageReceived
```
Offset | Field             | Type     | Size
-------|-------------------|----------|--------
0      | variant           | u32 LE   | 4
4      | room_id_len       | u64 LE   | 8
12     | room_id           | [u8]     | room_id_len
       | payload_len       | u64 LE   | 8
       | encrypted_payload | [u8]     | payload_len
       | timestamp         | i64 LE   | 8
       | message_id_len    | u64 LE   | 8
       | message_id        | [u8]     | message_id_len
```

#### 5. MessageAck
```
Offset | Field          | Type     | Size
-------|----------------|----------|--------
0      | variant        | u32 LE   | 4
4      | message_id_len | u64 LE   | 8
12     | message_id     | [u8]     | message_id_len
```

#### 6. UserJoined
```
Offset | Field        | Type     | Size
-------|--------------|----------|--------
0      | variant      | u32 LE   | 4
4      | room_id_len  | u64 LE   | 8
12     | room_id      | [u8]     | room_id_len
       | username_len | u64 LE   | 8
       | username     | [u8]     | username_len
```

#### 7. UserLeft
```
Offset | Field        | Type     | Size
-------|--------------|----------|--------
0      | variant      | u32 LE   | 4
4      | room_id_len  | u64 LE   | 8
12     | room_id      | [u8]     | room_id_len
       | username_len | u64 LE   | 8
       | username     | [u8]     | username_len
```

---

## Security Properties

### 1. End-to-End Encryption
- Server never sees plaintext messages
- Server cannot decrypt messages (lacks encryption key)
- Only participants with chat code can decrypt

### 2. Forward Secrecy
- Each message uses unique encryption key
- Old keys immediately discarded after use
- Compromise of current key doesn't reveal past messages
- Chain ratcheting prevents backward key derivation

### 3. Sender Authentication
- Each message signed with Ed25519 private key
- Recipients verify signature with sender's public key
- Prevents message spoofing and impersonation

### 4. Replay Protection
- Timestamp validation (±5 minutes window)
- Sequence numbers must be monotonically increasing
- Message ID prevents duplicate delivery

### 5. Message Integrity
- BLAKE3 commitment hash in payload
- Poly1305 authentication tag in ciphertext
- Tampering detected and rejected

### 6. Key Derivation Security
- Argon2id resistant to GPU/ASIC attacks
- High memory cost prevents parallel cracking
- Salting prevents rainbow table attacks
- Multiple iterations increase brute-force cost

### 7. Server Compromise Resilience
- Server only knows room IDs (hashes)
- Cannot reverse-engineer chat codes from room IDs
- Cannot decrypt stored messages
- Cannot inject messages (no valid signatures)

### 8. Quantum Resistance
- 512-bit keys exceed Grover's algorithm halving
- Argon2id remains quantum-resistant (memory-hard)
- Ed25519 potentially vulnerable (Shor's algorithm)
- Future: Consider post-quantum signature schemes (CRYSTALS-Dilithium, Falcon)

---

## Implementation Guide

### Language-Specific Considerations

#### Rust Implementation
- Use `bincode` for serialization
- Use `chacha20poly1305` crate with `XChaCha20Poly1305`
- Use `argon2` crate with `ParamsBuilder`
- Use `blake3` crate for hashing
- Use `ed25519-dalek` for signatures
- Use `zeroize` to clear sensitive data from memory

#### Python Implementation
- Use `struct` or manual byte manipulation for bincode compatibility
- Use `PyNaCl` (`nacl.secret.SecretBox`) for XChaCha20-Poly1305
- Use `argon2-cffi` with `hash_secret_raw`
- Use `blake3` package (install via pip)
- Use `cryptography` for Ed25519
- Explicitly zero sensitive byte arrays

### Cross-Platform Compatibility

**Critical Requirements**:
1. **Same hashing**: Both must use BLAKE3 (not BLAKE2b substitute)
2. **Same Argon2 parameters**: Exact m_cost, t_cost, p_cost values
3. **Same serialization**: Bincode-compatible format
4. **Same endianness**: Little-endian for all integers
5. **Same salt values**: Exact byte-for-byte match

**Testing Compatibility**:
```python
# Test vectors
chat_code = "123456"
expected_room_id = "..."  # Known good value from Rust

# Test key derivation
key = ChatKey.derive_from_code(chat_code)
test_plaintext = b"Hello, World!"
ciphertext = key.encrypt(test_plaintext)
decrypted = key.decrypt(ciphertext)
assert decrypted == test_plaintext

# Test room ID derivation
room_id = chat_code_to_room_id(chat_code)
assert room_id == expected_room_id
```

### Performance Considerations

**Argon2id tuning**:
- Higher memory cost → slower, more secure
- Higher time cost → slower, more secure
- Higher parallelism → faster with multiple cores
- Adjust based on hardware capabilities and threat model

**Typical timings (on modern CPU)**:
- Key derivation (128 MiB, t=4, p=8): ~200-500ms
- Chain key ratchet (BLAKE3): <1ms
- Encryption (XChaCha20): <1ms per KB
- Signature verification: <1ms

### Error Handling

**Critical errors (abort operation)**:
- Invalid signature → reject message
- Authentication tag mismatch → reject message
- Timestamp outside window → reject message
- Invalid bincode format → reject message

**Recoverable errors (inform user)**:
- Network disconnection → retry
- Server rejection → display error
- Invalid chat code → prompt again

### Security Best Practices

1. **Never log secrets**: Don't log chat codes, keys, plaintexts
2. **Zeroize memory**: Clear sensitive data after use
3. **Use constant-time comparisons**: Prevent timing attacks
4. **Validate inputs**: Check lengths, ranges, formats
5. **Use secure random**: CSPRNG for nonces, keys
6. **Update dependencies**: Keep crypto libraries current
7. **Rate limiting**: Prevent brute-force attacks
8. **Audit trails**: Log non-sensitive events for debugging

---

## Appendix: Test Vectors

### A. Key Derivation Test

**Input**:
- Chat code (numeric): `"123456"`

**Step 1: Expand**:
```
salt = b"rchat-numeric-salt-v2-extreme"
expanded = Argon2id(
    secret=b"123456",
    salt=salt,
    m_cost=65536,
    t_cost=3,
    p_cost=4,
    output_len=64
)
expanded (hex) = [implementation-specific]
```

**Step 2: Derive salt**:
```
salt_preimage = b"rchat-e2ee-v2-salt:" || expanded
salt_hash = BLAKE3(salt_preimage)
salt = salt_hash[0:32]
salt (hex) = [implementation-specific]
```

**Step 3: Final key**:
```
final_key = Argon2id(
    secret=expanded,
    salt=salt,
    m_cost=131072,
    t_cost=4,
    p_cost=8,
    output_len=32
)
final_key (hex) = [implementation-specific]
```

### B. Room ID Test

**Input**:
- Chat code: `"123456"`

**Expected output**:
```
blake3_hash = BLAKE3(b"rchat-room-id-v2:123456")
final_hash = SHA3-512(b"rchat-double-hash:" || blake3_hash)
room_id = base64url(final_hash)
room_id = [implementation-specific]
```

### C. Message Serialization Test

**Input payload**:
```json
{
    "username": "Alice",
    "content": "Hello",
    "timestamp": 1699999999,
    "sequence_number": 1,
    "sender_public_key": [32 bytes of 0xAA],
    "signature": [64 bytes of 0xBB],
    "chain_key_index": 0,
    "message_hash": [32 bytes of 0xCC]
}
```

**Expected bincode output (hex)**:
```
05 00 00 00 00 00 00 00  // username_len = 5
41 6C 69 63 65           // "Alice"
05 00 00 00 00 00 00 00  // content_len = 5
48 65 6C 6C 6F           // "Hello"
FF 65 5F 65 00 00 00 00  // timestamp = 1699999999
01 00 00 00 00 00 00 00  // sequence_number = 1
20 00 00 00 00 00 00 00  // pubkey_len = 32
[32 bytes of 0xAA]
40 00 00 00 00 00 00 00  // signature_len = 64
[64 bytes of 0xBB]
00 00 00 00 00 00 00 00  // chain_key_index = 0
20 00 00 00 00 00 00 00  // hash_len = 32
[32 bytes of 0xCC]
```

---

## Version History

- **v3.0** (2025-01): Added forward secrecy, message commitment, replay protection
- **v2.0** (2024-12): Changed to BLAKE3+SHA3-512, increased Argon2id parameters
- **v1.0** (2024-11): Initial specification

---

## References

1. [XChaCha20-Poly1305 RFC Draft](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha)
2. [Argon2 Specification](https://github.com/P-H-C/phc-winner-argon2/blob/master/argon2-specs.pdf)
3. [Ed25519 Paper](https://ed25519.cr.yp.to/ed25519-20110926.pdf)
4. [BLAKE3 Specification](https://github.com/BLAKE3-team/BLAKE3-specs/blob/master/blake3.pdf)
5. [SHA-3 Standard (FIPS 202)](https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf)
6. [Bincode Specification](https://github.com/bincode-org/bincode/blob/trunk/docs/spec.md)

---

**Document Status**: Living Document
**Last Updated**: 2025-11-07
**Maintainer**: RChat Development Team
