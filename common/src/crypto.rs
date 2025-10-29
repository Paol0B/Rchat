use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

/// Generatore di codici chat sicuri (256-bit, base64url)
pub fn generate_chat_code() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Deriva una chiave di crittografia dal codice della chat usando HKDF
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ChatKey {
    #[zeroize(skip)]
    cipher: ChaCha20Poly1305,
}

impl ChatKey {
    /// Deriva la chiave dal codice della chat
    pub fn derive_from_code(chat_code: &str) -> Result<Self, CryptoError> {
        // Decodifica il codice base64
        let chat_secret = URL_SAFE_NO_PAD
            .decode(chat_code)
            .map_err(|_| CryptoError::InvalidChatCode)?;

        if chat_secret.len() != 32 {
            return Err(CryptoError::InvalidChatCode);
        }

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
