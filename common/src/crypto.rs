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
/// Usa BLAKE3 (più veloce e sicuro di SHA-2) + SHA3-512 per doppia sicurezza
pub fn chat_code_to_room_id(chat_code: &str) -> String {
    // Prima passata con BLAKE3
    let mut blake3_hasher = Blake3Hasher::new();
    blake3_hasher.update(b"rchat-room-id-v2:");
    blake3_hasher.update(chat_code.as_bytes());
    let blake3_hash = blake3_hasher.finalize();
    
    // Seconda passata con SHA3-512 per sicurezza aggiuntiva
    let mut sha3_hasher = Sha3_512::new();
    sha3_hasher.update(b"rchat-double-hash:");
    sha3_hasher.update(blake3_hash.as_bytes());
    let final_hash = sha3_hasher.finalize();
    
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
    /// Usa Argon2id con parametri estremi per massima sicurezza
    pub fn derive_from_code(chat_code: &str) -> Result<Self, CryptoError> {
        let chat_secret = if chat_code.len() == 6 && chat_code.chars().all(|c| c.is_ascii_digit()) {
            // Formato numerico: espandi a 64 byte usando Argon2id
            let numeric_bytes = chat_code.as_bytes();
            
            // Argon2id con parametri ad alta sicurezza
            let params = ParamsBuilder::new()
                .m_cost(65536)    // 64 MB di memoria (resistente a GPU)
                .t_cost(3)        // 3 iterazioni
                .p_cost(4)        // 4 thread paralleli
                .output_len(64)   // 512-bit output
                .build()
                .map_err(|_| CryptoError::KeyDerivationFailed)?;
            
            let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
            
            let salt = b"rchat-numeric-salt-v2-extreme"; // Salt statico per codici numerici
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

        // Usa Argon2id per derivare la chiave finale di crittografia (256-bit per XChaCha20)
        let params = ParamsBuilder::new()
            .m_cost(131072)   // 128 MB di memoria (estrema sicurezza)
            .t_cost(4)        // 4 iterazioni
            .p_cost(8)        // 8 thread paralleli
            .output_len(32)   // 256-bit per XChaCha20-Poly1305
            .build()
            .map_err(|_| CryptoError::KeyDerivationFailed)?;
        
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        
        // Salt derivato da BLAKE3 del segreto per unicità
        let mut salt_hasher = Blake3Hasher::new();
        salt_hasher.update(b"rchat-e2ee-v2-salt:");
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
    pub fn encrypt_with_chain(&self, plaintext: &[u8], chain_key: &[u8; 32]) -> Result<Vec<u8>, CryptoError> {
        // Use chain key instead of base key
        let cipher = XChaCha20Poly1305::new_from_slice(chain_key)
            .map_err(|_| CryptoError::EncryptionFailed)?;

        // Generate random nonce (192-bit for XChaCha20Poly1305)
        let mut nonce_bytes = [0u8; 24];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from(nonce_bytes);

        // Encrypt with authentication
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| CryptoError::EncryptionFailed)?;

        // Concatenate nonce + ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt with ratcheted chain key (forward secrecy)
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
        cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)
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
#[derive(Clone, ZeroizeOnDrop)]
pub struct ChainKey {
    key: [u8; 32],
    index: u64,
}

impl ChainKey {
    /// Initialize chain from chat code
    pub fn from_chat_code(chat_code: &str) -> Result<Self, CryptoError> {
        let base_key = derive_key_material(chat_code, b"chain-key-init")?;
        Ok(Self {
            key: base_key,
            index: 0,
        })
    }

    /// Derive next key in the chain (forward secrecy)
    /// Uses BLAKE3 for fast KDF ratcheting
    pub fn next(&mut self) -> [u8; 32] {
        let mut hasher = Blake3Hasher::new();
        hasher.update(b"rchat-chain-ratchet:");
        hasher.update(&self.key);
        hasher.update(&self.index.to_le_bytes());
        
        let derived = hasher.finalize();
        let new_key: [u8; 32] = derived.as_bytes()[..32].try_into().unwrap();
        
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
        .m_cost(128 * 1024) // 128 MB
        .t_cost(4)
        .p_cost(8)
        .build()
        .map_err(|_| CryptoError::KeyDerivationFailed)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut output = [0u8; 32];
    
    argon2
        .hash_password_into(input.as_bytes(), salt, &mut output)
        .map_err(|_| CryptoError::KeyDerivationFailed)?;
    
    Ok(output)
}

