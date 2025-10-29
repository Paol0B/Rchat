use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::{Sha256, Digest};
use zeroize::{Zeroize, ZeroizeOnDrop};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

/// Generatore di codici chat sicuri (256-bit, base64url)
pub fn generate_chat_code() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Generatore di codici chat semplici (6 cifre numeriche)
/// ATTENZIONE: Meno sicuro del formato completo (solo ~20 bit di entropia vs 256 bit)
pub fn generate_numeric_chat_code() -> String {
    use rand::Rng;
    let code = rand::thread_rng().gen_range(100000..=999999);
    format!("{:06}", code)
}

/// Genera un identificatore di chat che il server può usare senza conoscere il codice originale
/// Il server usa questo hash per identificare la chat, ma non può derivare la chiave E2EE
pub fn chat_code_to_room_id(chat_code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"rchat-room-id-v1:");
    hasher.update(chat_code.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

/// Deriva una chiave di crittografia dal codice della chat usando HKDF
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ChatKey {
    #[zeroize(skip)]
    cipher: ChaCha20Poly1305,
}

impl ChatKey {
    /// Deriva la chiave dal codice della chat (supporta sia formato numerico che base64)
    pub fn derive_from_code(chat_code: &str) -> Result<Self, CryptoError> {
        let chat_secret = if chat_code.len() == 6 && chat_code.chars().all(|c| c.is_ascii_digit()) {
            // Formato numerico: espandi a 32 byte usando HKDF
            let numeric_bytes = chat_code.as_bytes();
            let hkdf = Hkdf::<Sha256>::new(Some(b"rchat-numeric-code"), numeric_bytes);
            let mut expanded = [0u8; 32];
            hkdf.expand(b"rchat-secret-expansion", &mut expanded)
                .map_err(|_| CryptoError::InvalidChatCode)?;
            expanded.to_vec()
        } else {
            // Formato base64: decodifica direttamente
            let decoded = URL_SAFE_NO_PAD
                .decode(chat_code)
                .map_err(|_| CryptoError::InvalidChatCode)?;
            
            if decoded.len() != 32 {
                return Err(CryptoError::InvalidChatCode);
            }
            decoded
        };

        // Usa HKDF per derivare una chiave di crittografia
        let hkdf = Hkdf::<Sha256>::new(None, &chat_secret);
        let mut key_bytes = [0u8; 32];
        hkdf.expand(b"rchat-e2ee-v1", &mut key_bytes)
            .map_err(|_| CryptoError::KeyDerivationFailed)?;

        let cipher = ChaCha20Poly1305::new_from_slice(&key_bytes)
            .map_err(|_| CryptoError::KeyDerivationFailed)?;

        // Zeroizza i byte della chiave
        key_bytes.zeroize();

        Ok(Self { cipher })
    }

    /// Cripta un payload
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        // Genera un nonce random (96-bit per ChaCha20Poly1305)
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        // Cripta
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| CryptoError::EncryptionFailed)?;

        // Concatena nonce + ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decripta un payload
    pub fn decrypt(&self, encrypted: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if encrypted.len() < 12 {
            return Err(CryptoError::DecryptionFailed);
        }

        // Estrai nonce e ciphertext
        let (nonce_bytes, ciphertext) = encrypted.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decripta
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)
    }
}

#[derive(Debug, Clone)]
pub enum CryptoError {
    InvalidChatCode,
    KeyDerivationFailed,
    EncryptionFailed,
    DecryptionFailed,
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::InvalidChatCode => write!(f, "Codice chat non valido"),
            CryptoError::KeyDerivationFailed => write!(f, "Fallita derivazione chiave"),
            CryptoError::EncryptionFailed => write!(f, "Crittografia fallita"),
            CryptoError::DecryptionFailed => write!(f, "Decrittografia fallita"),
        }
    }
}

impl std::error::Error for CryptoError {}
