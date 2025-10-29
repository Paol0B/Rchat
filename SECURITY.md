# Security Policy

## 🔐 Security Model

Rchat implements a **defense-in-depth** security architecture:

### Cryptographic Primitives

- **End-to-End Encryption**: ChaCha20-Poly1305 AEAD cipher
- **Key Derivation**: HKDF-SHA256 with 256-bit entropy
- **Transport Security**: TLS 1.3 via rustls
- **Random Number Generation**: OS-level entropy via `getrandom`
- **Memory Security**: Automatic zeroization via `zeroize` crate

### Threat Model

#### ✅ Protected Against:

1. **Server Compromise**: Server cannot read message content (E2EE)
2. **Network Eavesdropping**: All traffic encrypted with TLS 1.3
3. **Memory Dumps**: Sensitive data is zeroized after use
4. **Replay Attacks**: Unique nonces for each message
5. **Message Tampering**: AEAD provides authenticity

#### ⚠️ NOT Protected Against:

1. **Endpoint Compromise**: If a client is compromised, messages can be read
2. **Malicious Server**: Server can log metadata (who talks to whom, when)
3. **Side-Channel Attacks**: No countermeasures for timing attacks
4. **Social Engineering**: Users sharing chat codes insecurely
5. **Denial of Service**: No rate limiting implemented

### Security Assumptions

1. **Trusted Setup**: Chat code must be shared via secure out-of-band channel
2. **No Forward Secrecy**: Same key is used for entire chat session
3. **No Identity Verification**: Usernames are self-declared, no authentication
4. **Ephemeral Only**: No persistence means no recovery after disconnect
5. **Local Network**: Designed for trusted network environments

## 🛡️ Security Features

### E2EE Implementation

```
Chat Code (256-bit random) 
    ↓ base64url
Chat Code String
    ↓ HKDF-SHA256(salt=None, info="rchat-e2ee-v1")
Encryption Key (256-bit)
    ↓ ChaCha20-Poly1305
Encrypted Message
```

**Properties**:
- Server is **computationally unable** to decrypt messages
- Forward secrecy: **NOT implemented** (same key for session)
- Post-compromise security: **NOT implemented**
- Deniability: **NOT implemented**

### Memory Safety

- All sensitive data (`ChatKey`, `MessagePayload`) use `zeroize`
- Automatic cleanup on `Drop`
- No persistent storage = no data remanence

### Network Security

- TLS 1.3 for all client-server connections
- Certificate validation (self-signed for demo)
- No plaintext transmission

## 🚨 Known Limitations

### Critical

1. **No Authentication**: Anyone with chat code can join
2. **No Forward Secrecy**: Compromised key decrypts all messages
3. **Self-Signed Certs**: Demo uses self-signed certificates (MITM risk)
4. **No Rate Limiting**: Vulnerable to spam/DoS

### Important

1. **Metadata Leakage**: Server sees who participates and when
2. **No Audit Trail**: No logging means no forensics
3. **No User Verification**: Usernames can be spoofed
4. **Single Key**: All participants share same symmetric key

### Minor

1. **No Message Ordering**: Timestamp-based, not cryptographically guaranteed
2. **No Offline Messages**: Messages lost if recipient offline
3. **No Read Receipts**: No delivery confirmation

## 📢 Reporting a Vulnerability

This is a **proof-of-concept** project for educational purposes.

If you discover a security vulnerability:

1. **Do NOT** open a public issue
2. Email: [your-email@example.com]
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

We will acknowledge receipt within 48 hours.

## 🔬 Security Audits

**Status**: ❌ NOT AUDITED

This software has **NOT** undergone professional security auditing.

**Do NOT use in production** without:
- Professional cryptographic audit
- Penetration testing
- Code review by security experts
- Formal threat modeling

## 📚 Cryptographic Dependencies

| Crate | Version | Purpose | Audit Status |
|-------|---------|---------|--------------|
| `chacha20poly1305` | 0.10 | AEAD encryption | ✅ Audited |
| `hkdf` | 0.12 | Key derivation | ✅ Audited |
| `sha2` | 0.10 | Hash function | ✅ Audited |
| `rustls` | 0.23 | TLS implementation | ✅ Audited |
| `getrandom` | 0.2 | CSPRNG | ✅ Audited |

*Note: "Audited" refers to crates maintained by RustCrypto with public audits*

## 🛠️ Security Hardening Recommendations

### For Production Use (NOT CURRENTLY SUITABLE):

1. **Authentication**:
   - Implement proper user authentication
   - Use public-key cryptography for identity
   - Consider Signal Protocol or Matrix Olm

2. **Key Management**:
   - Implement key rotation
   - Add forward secrecy (Double Ratchet)
   - Use Hardware Security Modules (HSMs)

3. **Network**:
   - Use valid TLS certificates from trusted CA
   - Implement certificate pinning
   - Add rate limiting and DDoS protection

4. **Monitoring**:
   - Add audit logging (metadata only)
   - Implement intrusion detection
   - Monitor for anomalous behavior

5. **Code**:
   - Professional security audit
   - Fuzzing of protocol parsers
   - Static analysis with tools like `cargo-geiger`

## 🔗 Security Resources

- [ChaCha20-Poly1305 RFC 8439](https://tools.ietf.org/html/rfc8439)
- [HKDF RFC 5869](https://tools.ietf.org/html/rfc5869)
- [TLS 1.3 RFC 8446](https://tools.ietf.org/html/rfc8446)
- [Signal Protocol](https://signal.org/docs/)
- [OWASP Cryptographic Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)

## ⚖️ Legal Disclaimer

This software is provided "AS IS" without warranty of any kind.

**USE AT YOUR OWN RISK.**

The authors are NOT liable for:
- Security breaches
- Data loss
- Privacy violations
- Any damages resulting from use

This is an **educational project** demonstrating E2EE concepts.

---

*Last Updated: October 2025*
*Security Level: EXPERIMENTAL / EDUCATIONAL ONLY*
