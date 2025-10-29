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
    CreateChat {
        chat_type: ChatType,
        username: String,
    },
    /// Unisciti a una chat esistente
    JoinChat {
        chat_code: String,
        username: String,
    },
    /// Invia un messaggio crittografato (il server lo inoltra senza decifrarlo)
    SendMessage {
        chat_code: String,
        encrypted_payload: Vec<u8>,
    },
    /// Disconnettiti dalla chat
    LeaveChat {
        chat_code: String,
    },
}

/// Messaggio dal server al client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Chat creata con successo
    ChatCreated {
        chat_code: String,
        chat_type: ChatType,
    },
    /// Join alla chat riuscito
    JoinedChat {
        chat_code: String,
        chat_type: ChatType,
        participant_count: usize,
    },
    /// Errore
    Error {
        message: String,
    },
    /// Nuovo messaggio ricevuto (crittografato)
    MessageReceived {
        chat_code: String,
        encrypted_payload: Vec<u8>,
        timestamp: i64,
    },
    /// Un utente si è unito
    UserJoined {
        chat_code: String,
        username: String,
    },
    /// Un utente ha lasciato
    UserLeft {
        chat_code: String,
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
