use common::{ChatType, ServerMessage};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Una stanza chat con i suoi partecipanti
pub struct ChatRoom {
    pub chat_type: ChatType,
    pub participants: HashMap<String, (String, mpsc::Sender<ServerMessage>)>, // client_id -> (username, sender)
}

impl ChatRoom {
    pub fn new(chat_type: ChatType) -> Self {
        Self {
            chat_type,
            participants: HashMap::new(),
        }
    }

    pub fn can_join(&self) -> bool {
        match &self.chat_type {
            ChatType::OneToOne => self.participants.len() < 2,
            ChatType::Group { max_participants } => self.participants.len() < *max_participants,
        }
    }

    pub fn add_participant(
        &mut self,
        client_id: String,
        username: String,
        sender: mpsc::Sender<ServerMessage>,
    ) {
        self.participants.insert(client_id, (username, sender));
    }

    pub fn remove_participant(&mut self, client_id: &str) -> Option<String> {
        self.participants.remove(client_id).map(|(username, _)| username)
    }

    pub async fn broadcast(&self, msg: ServerMessage, exclude_client: Option<&str>) {
        for (client_id, (_, tx)) in &self.participants {
            if let Some(exclude) = exclude_client {
                if client_id == exclude {
                    continue;
                }
            }
            let _ = tx.send(msg.clone()).await;
        }
    }
}

impl Drop for ChatRoom {
    fn drop(&mut self) {
        // Cleanup: zeroizza dati sensibili
        self.participants.clear();
    }
}

/// Stato globale del server
pub struct ChatState {
    chats: Arc<Mutex<HashMap<String, Arc<Mutex<ChatRoom>>>>>,
}

impl ChatState {
    pub fn new(_numeric_codes: bool) -> Self {
        // Il parametro numeric_codes non è più necessario perché il client genera il codice
        Self {
            chats: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Crea una nuova chat usando room_id (il server non conosce mai il chat_code originale)
    pub async fn create_chat(&self, room_id: String, chat_type: ChatType) {
        let room = Arc::new(Mutex::new(ChatRoom::new(chat_type)));
        self.chats.lock().await.insert(room_id, room);
    }

    /// Unisciti a una chat usando room_id
    pub async fn join_chat(
        &self,
        room_id: &str,
        username: String,
        sender: mpsc::Sender<ServerMessage>,
    ) -> Result<(ChatType, usize), String> {
        let chats = self.chats.lock().await;
        let room = chats
            .get(room_id)
            .ok_or_else(|| "Chat non trovata".to_string())?;

        let mut room = room.lock().await;

        if !room.can_join() {
            return Err("Chat piena".to_string());
        }

        let client_id = format!("{}_{}", username, uuid::Uuid::new_v4());
        room.add_participant(client_id, username, sender);

        Ok((room.chat_type.clone(), room.participants.len()))
    }

    pub async fn leave_chat(&self, room_id: &str, client_id: &str) -> Option<String> {
        let chats = self.chats.lock().await;
        let room = chats.get(room_id)?;
        let mut room = room.lock().await;
        room.remove_participant(client_id)
    }

    pub async fn broadcast_message(
        &self,
        room_id: &str,
        encrypted_payload: Vec<u8>,
        _sender_id: &str,
    ) {
        let chats = self.chats.lock().await;
        if let Some(room) = chats.get(room_id) {
            let room = room.lock().await;
            let msg = ServerMessage::MessageReceived {
                room_id: room_id.to_string(),
                encrypted_payload,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            };
            // Invia a TUTTI, incluso il mittente (None = nessuna esclusione)
            room.broadcast(msg, None).await;
        }
    }

    pub async fn broadcast_user_event(&self, room_id: &str, username: String, joined: bool) {
        let chats = self.chats.lock().await;
        if let Some(room) = chats.get(room_id) {
            let room = room.lock().await;
            let msg = if joined {
                ServerMessage::UserJoined {
                    room_id: room_id.to_string(),
                    username,
                }
            } else {
                ServerMessage::UserLeft {
                    room_id: room_id.to_string(),
                    username,
                }
            };
            room.broadcast(msg, None).await;
        }
    }
}

// UUID semplificato per generare client_id
mod uuid {
    use rand::Rng;

    pub struct Uuid([u8; 16]);

    impl Uuid {
        pub fn new_v4() -> Self {
            let mut bytes = [0u8; 16];
            rand::thread_rng().fill(&mut bytes);
            Self(bytes)
        }
    }

    impl std::fmt::Display for Uuid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}",
                self.0[0], self.0[1], self.0[2], self.0[3],
                self.0[4], self.0[5], self.0[6], self.0[7]
            )
        }
    }
}
