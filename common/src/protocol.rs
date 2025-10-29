use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Tipi di chat supportati
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatType {
    OneToOne,
    Group { max_participants: usize },
}

/// Messaggio dal client al server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Crea una nuova chat
    /// Il client genera il chat_code localmente e invia solo il room_id al server
    CreateChat {
        room_id: String, // SHA256 hash del chat_code generato dal client
        chat_type: ChatType,
        username: String,
    },
    /// Unisciti a una chat esistente
    /// room_id è un hash del chat_code, così il server non conosce mai il codice originale
    JoinChat {
        room_id: String, // SHA256 hash del chat_code
        username: String,
    },
    /// Invia un messaggio crittografato (il server lo inoltra senza decifrarlo)
    SendMessage {
        room_id: String, // SHA256 hash del chat_code
        encrypted_payload: Vec<u8>,
    },
    /// Disconnettiti dalla chat
    LeaveChat {
        room_id: String, // SHA256 hash del chat_code
    },
}

/// Messaggio dal server al client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Chat creata con successo (il server non conosce il chat_code originale)
    ChatCreated {
        room_id: String, // Il server conferma con il room_id
        chat_type: ChatType,
    },
    /// Join alla chat riuscito
    JoinedChat {
        room_id: String,
        chat_type: ChatType,
        participant_count: usize,
    },
    /// Errore
    Error {
        message: String,
    },
    /// Nuovo messaggio ricevuto (crittografato)
    MessageReceived {
        room_id: String,
        encrypted_payload: Vec<u8>,
        timestamp: i64,
    },
    /// Un utente si è unito
    UserJoined {
        room_id: String,
        username: String,
    },
    /// Un utente ha lasciato
    UserLeft {
        room_id: String,
        username: String,
    },
}

/// Payload del messaggio prima della crittografia
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct MessagePayload {
    pub username: String,
    pub content: String,
    pub timestamp: i64,
}

impl MessagePayload {
    pub fn new(username: String, content: String) -> Self {
        Self {
            username,
            content,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

// Implementazione temporale per chrono
mod chrono {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    pub struct Utc;
    
    impl Utc {
        pub fn now() -> Self {
            Self
        }
        
        pub fn timestamp(&self) -> i64 {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        }
    }
}
