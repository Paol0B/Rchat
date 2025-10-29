use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Supported chat types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatType {
    OneToOne,
    Group { max_participants: usize },
}

/// Message from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Create a new chat
    /// Client generates chat_code locally and sends only room_id to server
    CreateChat {
        room_id: String, // BLAKE3+SHA3-512 hash of client-generated chat_code
        chat_type: ChatType,
        username: String,
    },
    /// Join an existing chat
    /// room_id is a hash of chat_code, so server never knows the original code
    JoinChat {
        room_id: String, // BLAKE3+SHA3-512 hash of chat_code
        username: String,
    },
    /// Send encrypted message (server forwards without decrypting)
    SendMessage {
        room_id: String, // BLAKE3+SHA3-512 hash of chat_code
        encrypted_payload: Vec<u8>,
        message_id: String, // Unique ID for ACK tracking
    },
    /// Disconnect from chat
    LeaveChat {
        room_id: String, // BLAKE3+SHA3-512 hash of chat_code
    },
}

/// Message from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Chat created successfully (server never knows the original chat_code)
    ChatCreated {
        room_id: String, // Server confirms with room_id
        chat_type: ChatType,
    },
    /// Successfully joined chat
    JoinedChat {
        room_id: String,
        chat_type: ChatType,
        participant_count: usize,
    },
    /// Error
    Error {
        message: String,
    },
    /// New message received (encrypted)
    MessageReceived {
        room_id: String,
        encrypted_payload: Vec<u8>,
        timestamp: i64,
        message_id: String, // ID for deduplication
    },
    /// Message acknowledgment
    MessageAck {
        message_id: String, // Confirms message was received by server
    },
    /// A user joined
    UserJoined {
        room_id: String,
        username: String,
    },
    /// A user left
    UserLeft {
        room_id: String,
        username: String,
    },
}

/// Message payload before encryption
/// Now includes forward secrecy and sender verification
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize)]
#[zeroize(drop)]
pub struct MessagePayload {
    pub username: String,
    pub content: String,
    pub timestamp: i64,
    pub sequence_number: u64,           // For message ordering and replay protection
    pub sender_public_key: Vec<u8>,     // Ed25519 public key for sender verification
    pub signature: Vec<u8>,             // Ed25519 signature over (content || timestamp || sequence)
    pub chain_key_index: u64,           // For forward secrecy ratcheting
}

impl MessagePayload {
    pub fn new(
        username: String,
        content: String,
        sequence_number: u64,
        sender_public_key: Vec<u8>,
        signature: Vec<u8>,
        chain_key_index: u64,
    ) -> Self {
        Self {
            username,
            content,
            timestamp: chrono::Utc::now().timestamp(),
            sequence_number,
            sender_public_key,
            signature,
            chain_key_index,
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
