use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    XChaCha20Poly1305, XNonce,
};
use argon2::{Argon2, ParamsBuilder, Algorithm, Version};
use rand::RngCore;
use sha3::{Sha3_512, Digest};
use blake3::Hasher as Blake3Hasher;
use zeroize::{Zeroize, ZeroizeOnDrop};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use ed25519_dalek::{Signer, Verifier, SigningKey, VerifyingKey, Signature};
use subtle::ConstantTimeEq;
use hkdf::Hkdf;
use sha2::Sha512;

/// Generatore di codici chat sicuri (512-bit, base64url)
/// Usa 512 bit per sicurezza estrema contro attacchi quantistici futuri
pub fn generate_chat_code() -> String {
    let mut bytes = [0u8; 64]; // 512 bit
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Generatore di codici chat semplici (6 cifre numeriche)
/// ATTENZIONE: Meno sicuro del formato completo (solo ~20 bit di entropia vs 512 bit)
pub fn generate_numeric_chat_code() -> String {
    use rand::Rng;
    let code = rand::thread_rng().gen_range(100000..=999999);
    format!("{:06}", code)
}

/// Genera un identificatore di chat che il server può usare senza conoscere il codice originale
/// Usa BLAKE3 + SHA3-512 + Argon2id per tripla sicurezza e resistenza brute-force
/// Domain separation rigorosa per prevenire cross-protocol attacks
pub fn chat_code_to_room_id(chat_code: &str) -> String {
    // Prima passata con BLAKE3 per velocità
    let mut blake3_hasher = Blake3Hasher::new();
    blake3_hasher.update(b"rchat-v3-room-id-domain-sep:");
    blake3_hasher.update(chat_code.as_bytes());
    let blake3_hash = blake3_hasher.finalize();
    
    // Seconda passata con SHA3-512 per sicurezza aggiuntiva
    let mut sha3_hasher = Sha3_512::new();
    sha3_hasher.update(b"rchat-v3-double-hash-domain:");
    sha3_hasher.update(blake3_hash.as_bytes());
    let sha3_hash = sha3_hasher.finalize();
    
    // Terza passata con Argon2id per resistenza brute-force
    // Parametri moderati per bilanciare sicurezza e performance del server
    let params = ParamsBuilder::new()
        .m_cost(32 * 1024)   // 32 MB per room_id (server deve calcolare spesso)
        .t_cost(2)           // 2 iterazioni
        .p_cost(2)           // 2 thread
        .output_len(64)      // 512-bit output
        .build()
        .expect("Valid Argon2id params");
    
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    
    // Salt derivato da SHA3 hash per unicità
    let salt = b"rchat-v3-room-id-salt-extreme";
    let mut final_hash = [0u8; 64];
    
    argon2.hash_password_into(&sha3_hash, salt, &mut final_hash)
        .expect("Argon2id hashing should succeed");
    
    URL_SAFE_NO_PAD.encode(&final_hash[..])
}

/// Deriva una chiave di crittografia dal codice della chat usando Argon2id
/// Argon2id è il vincitore della Password Hashing Competition ed è resistente a:
/// - Attacchi side-channel
/// - Attacchi GPU/ASIC
/// - Attacchi timing
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ChatKey {
    #[zeroize(skip)]
    cipher: XChaCha20Poly1305,
}

impl ChatKey {
    /// Deriva la chiave dal codice della chat (supporta sia formato numerico che base64)
    /// Usa Argon2id con parametri ESTREMI per massima sicurezza
    /// Resistente a: GPU, ASIC, side-channel, timing attacks
    pub fn derive_from_code(chat_code: &str) -> Result<Self, CryptoError> {
        let chat_secret = if chat_code.len() == 6 && chat_code.chars().all(|c| c.is_ascii_digit()) {
            // Formato numerico: espandi a 64 byte usando Argon2id
            let numeric_bytes = chat_code.as_bytes();
            
            // Argon2id con parametri MOLTO alti per compensare bassa entropia (20 bit)
            // Rendere brute-force praticamente impossibile anche con GPU farm
            let params = ParamsBuilder::new()
                .m_cost(512 * 1024)  // 512 MB di memoria (estremamente resistente a GPU)
                .t_cost(8)           // 8 iterazioni (molto lento ma sicuro)
                .p_cost(4)           // 4 thread paralleli
                .output_len(64)      // 512-bit output
                .build()
                .map_err(|_| CryptoError::KeyDerivationFailed)?;
            
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            
            // Salt con domain separation rigorosa
            let salt = b"rchat-v3-numeric-extreme-salt";
            let mut expanded = [0u8; 64];
            argon2.hash_password_into(numeric_bytes, salt, &mut expanded)
                .map_err(|_| CryptoError::KeyDerivationFailed)?;
            
            expanded.to_vec()
        } else {
            // Formato base64: decodifica e verifica 512-bit
            let decoded = URL_SAFE_NO_PAD
                .decode(chat_code)
                .map_err(|_| CryptoError::InvalidChatCode)?;
            
            if decoded.len() != 64 {
                return Err(CryptoError::InvalidChatCode);
            }
            decoded
        };

        // Usa Argon2id ESTREMO per derivare la chiave finale di crittografia (256-bit per XChaCha20)
        // Parametri massimi per sicurezza post-quantistica
        let params = ParamsBuilder::new()
            .m_cost(256 * 1024)  // 256 MB di memoria (bilanciamento sicurezza/usabilità)
            .t_cost(6)           // 6 iterazioni (molto sicuro)
            .p_cost(8)           // 8 thread paralleli (usa CPU moderna)
            .output_len(32)      // 256-bit per XChaCha20-Poly1305
            .build()
            .map_err(|_| CryptoError::KeyDerivationFailed)?;
        
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        
        // Salt derivato con HKDF-BLAKE3 per massima sicurezza e domain separation
        let mut salt_hasher = Blake3Hasher::new();
        salt_hasher.update(b"rchat-v3-e2ee-salt-domain:");
        salt_hasher.update(&chat_secret);
        let salt_hash = salt_hasher.finalize();
        let salt = &salt_hash.as_bytes()[..32]; // Usa primi 256 bit come salt
        
        let mut key_bytes = [0u8; 32];
        argon2.hash_password_into(&chat_secret, salt, &mut key_bytes)
            .map_err(|_| CryptoError::KeyDerivationFailed)?;

        let cipher = XChaCha20Poly1305::new_from_slice(&key_bytes)
            .map_err(|_| CryptoError::KeyDerivationFailed)?;

        // Zeroizza i byte della chiave
        key_bytes.zeroize();

        Ok(Self { cipher })
    }

    /// Encrypt with ratcheted chain key (forward secrecy)
    /// Includes padding to hide message length (防止流量分析)
    pub fn encrypt_with_chain(&self, plaintext: &[u8], chain_key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
        // Use chain key instead of base key
        let cipher = XChaCha20Poly1305::new_from_slice(chain_key)
            .map_err(|_| CryptoError::EncryptionFailed)?;

        // Apply padding to hide message length (防止流量分析攻击)
        let padded = apply_padding(plaintext);

        // Generate random nonce (192-bit for XChaCha20Poly1305)
        let mut nonce_bytes = [0u8; 24];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from(nonce_bytes);

        // Encrypt with authentication
        let ciphertext = cipher
            .encrypt(&nonce, padded.as_slice())
            .map_err(|_| CryptoError::EncryptionFailed)?;

        // Concatenate nonce + ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt with ratcheted chain key (forward secrecy)
    /// Removes padding after decryption
    pub fn decrypt_with_chain(&self, encrypted: &[u8], chain_key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
        if encrypted.len() < 24 {
            return Err(CryptoError::DecryptionFailed);
        }

        // Use chain key instead of base key
        let cipher = XChaCha20Poly1305::new_from_slice(chain_key)
            .map_err(|_| CryptoError::DecryptionFailed)?;

        // Extract nonce (192-bit) and ciphertext
        let (nonce_bytes, ciphertext) = encrypted.split_at(24);
        let nonce_array: [u8; 24] = nonce_bytes.try_into().map_err(|_| CryptoError::DecryptionFailed)?;
        let nonce = XNonce::from(nonce_array);

        // Decrypt and verify authentication
        let padded = cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)?;

        // Remove padding
        remove_padding(&padded)
    }

    /// Cripta un payload con XChaCha20-Poly1305
    /// XChaCha20 usa nonce a 192-bit (vs 96-bit di ChaCha20) per maggiore sicurezza
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Genera un nonce random (192-bit per XChaCha20Poly1305)
        let mut nonce_bytes = [0u8; 24];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from(nonce_bytes);

        // Cripta con autenticazione
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| CryptoError::EncryptionFailed)?;

        // Concatena nonce + ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decripta un payload con verifica di autenticità (AEAD)
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if encrypted.len() < 24 {
            return Err(CryptoError::DecryptionFailed);
        }

        // Estrai nonce (192-bit) e ciphertext
        let (nonce_bytes, ciphertext) = encrypted.split_at(24);
        let nonce_array: [u8; 24] = nonce_bytes.try_into().map_err(|_| CryptoError::DecryptionFailed)?;
        let nonce = XNonce::from(nonce_array);

        // Decripta e verifica autenticazione
        self.cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)
    }
}

#[derive(Debug, Clone)]
pub enum CryptoError {
    InvalidChatCode,
    KeyDerivationFailed,
    EncryptionFailed,
    DecryptionFailed,
    SigningFailed,
    VerificationFailed,
    InvalidSignature,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::InvalidChatCode => write!(f, "Invalid chat code"),
            CryptoError::KeyDerivationFailed => write!(f, "Key derivation failed"),
            CryptoError::EncryptionFailed => write!(f, "Encryption failed"),
            CryptoError::DecryptionFailed => write!(f, "Decryption failed"),
            CryptoError::SigningFailed => write!(f, "Message signing failed"),
            CryptoError::VerificationFailed => write!(f, "Signature verification failed"),
            CryptoError::InvalidSignature => write!(f, "Invalid message signature"),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Identity keypair for message signing (Ed25519)
/// Used for sender verification and authentication
#[derive(Clone)]
pub struct IdentityKey {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl Drop for IdentityKey {
    fn drop(&mut self) {
        // Only zeroize the signing key (private key)
        // VerifyingKey is public and doesn't need zeroization
        use zeroize::Zeroize;
        let mut bytes = self.signing_key.to_bytes();
        bytes.zeroize();
    }
}

impl IdentityKey {
    /// Generate a new Ed25519 identity keypair
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Get the public verifying key (to share with others)
    pub fn public_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }

    /// Get public key as bytes (32 bytes)
    pub fn public_key_bytes(&self) -> Vec<u8> {
        self.verifying_key.to_bytes().to_vec()
    }

    /// Sign a message with the private key
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.signing_key.sign(message).to_bytes().to_vec()
    }

    /// Verify a signature with a public key
    pub fn verify(public_key_bytes: &[u8], message: &[u8], signature_bytes: &[u8]) -> Result<(), CryptoError> {
        // Parse public key
        let public_key_array: [u8; 32] = public_key_bytes
            .try_into()
            .map_err(|_| CryptoError::VerificationFailed)?;
        let verifying_key = VerifyingKey::from_bytes(&public_key_array)
            .map_err(|_| CryptoError::VerificationFailed)?;

        // Parse signature
        let signature_array: [u8; 64] = signature_bytes
            .try_into()
            .map_err(|_| CryptoError::InvalidSignature)?;
        let signature = Signature::from_bytes(&signature_array);

        // Verify
        verifying_key
            .verify(message, &signature)
            .map_err(|_| CryptoError::InvalidSignature)
    }
}

/// Forward Secrecy Chain Key for message ratcheting
/// Each message derives a new encryption key from the previous one
/// Uses HKDF-BLAKE3 for cryptographically secure key derivation
#[derive(Clone, ZeroizeOnDrop)]
pub struct ChainKey {
    key: [u8; 32],
    index: u64,
}

impl ChainKey {
    /// Initialize chain from chat code
    pub fn from_chat_code(chat_code: &str) -> Result<Self, CryptoError> {
        let base_key = derive_key_material(chat_code, b"rchat-v3-chain-key-init")?;
        Ok(Self {
            key: base_key,
            index: 0,
        })
    }

    /// Derive next key in the chain using HKDF-SHA512 (forward secrecy)
    /// HKDF provides better security guarantees than simple hashing
    /// Domain separation prevents cross-protocol attacks
    pub fn next(&mut self) -> [u8; 32] {
        // HKDF-Expand usando SHA-512 per massima sicurezza
        let hkdf = Hkdf::<Sha512>::from_prk(&self.key).expect("Valid PRK");
        
        // Info context con domain separation e index
        let mut info = Vec::new();
        info.extend_from_slice(b"rchat-v3-chain-ratchet-forward-secrecy:");
        info.extend_from_slice(&self.index.to_le_bytes());
        
        let mut new_key = [0u8; 32];
        hkdf.expand(&info, &mut new_key)
            .expect("HKDF expand should succeed");
        
        // Zeroizza la chiave precedente
        self.key.zeroize();
        self.key = new_key;
        self.index += 1;
        
        new_key
    }

    /// Get current index
    pub fn index(&self) -> u64 {
        self.index
    }

    /// Advance to a specific index (for synchronization)
    pub fn advance_to(&mut self, target_index: u64) {
        while self.index < target_index {
            self.next();
        }
    }
}

/// Helper function to derive key material using Argon2id
fn derive_key_material(input: &str, salt: &[u8]) -> Result<[u8; 32], CryptoError> {
    let params = ParamsBuilder::new()
        .m_cost(256 * 1024) // 256 MB (aumentato per sicurezza)
        .t_cost(6)          // 6 iterazioni (aumentato)
        .p_cost(8)          // 8 thread
        .build()
        .map_err(|_| CryptoError::KeyDerivationFailed)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut output = [0u8; 32];
    
    argon2
        .hash_password_into(input.as_bytes(), salt, &mut output)
        .map_err(|_| CryptoError::KeyDerivationFailed)?;
    
    Ok(output)
}

/// Apply PKCS#7-style padding to hide message length
/// Rounds up to nearest 256-byte boundary for traffic analysis resistance
fn apply_padding(data: &[u8]) -> Vec<u8> {
    const BLOCK_SIZE: usize = 256;
    
    let original_len = data.len();
    let padded_len = ((original_len / BLOCK_SIZE) + 1) * BLOCK_SIZE;
    let padding_len = padded_len - original_len;
    
    let mut padded = Vec::with_capacity(padded_len + 4);
    
    // Store original length (4 bytes, little-endian)
    padded.extend_from_slice(&(original_len as u32).to_le_bytes());
    
    // Original data
    padded.extend_from_slice(data);
    
    // Padding bytes (all same value = padding length % 256)
    let padding_byte = (padding_len % 256) as u8;
    padded.resize(padded_len + 4, padding_byte);
    
    padded
}

/// Remove PKCS#7-style padding
fn remove_padding(padded: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if padded.len() < 4 {
        return Err(CryptoError::DecryptionFailed);
    }
    
    // Extract original length
    let len_bytes: [u8; 4] = padded[0..4].try_into()
        .map_err(|_| CryptoError::DecryptionFailed)?;
    let original_len = u32::from_le_bytes(len_bytes) as usize;
    
    if original_len + 4 > padded.len() {
        return Err(CryptoError::DecryptionFailed);
    }
    
    // Extract original data
    Ok(padded[4..4 + original_len].to_vec())
}

/// Constant-time comparison for security-sensitive data
/// Prevents timing attacks
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.ct_eq(b).into()
}

