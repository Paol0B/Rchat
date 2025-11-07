use pyo3::prelude::*;
use pyo3::types::{PyModule};

// Re-export common types
use common::{
    ChatKey, ChainKey, IdentityKey, ChatType, ClientMessage, ServerMessage, MessagePayload,
    generate_chat_code, generate_numeric_chat_code, chat_code_to_room_id,
};
use pyo3::exceptions::PyValueError;

// Wrapper per ChatKey
#[pyclass]
pub struct PyChatKey {
    inner: ChatKey,
}

#[pymethods]
impl PyChatKey {
    #[staticmethod]
    fn derive_from_code(chat_code: &str) -> PyResult<Self> {
        ChatKey::derive_from_code(chat_code)
            .map(|inner| PyChatKey { inner })
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }

    fn encrypt(&self, plaintext: &[u8]) -> PyResult<Vec<u8>> {
        self.inner
            .encrypt(plaintext)
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }

    fn decrypt(&self, encrypted: &[u8]) -> PyResult<Vec<u8>> {
        self.inner
            .decrypt(encrypted)
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }

    fn encrypt_with_chain(&self, plaintext: &[u8], chain_key: &[u8]) -> PyResult<Vec<u8>> {
        if chain_key.len() != 32 {
            return Err(PyValueError::new_err("Chain key must be 32 bytes"));
        }
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(chain_key);
        self.inner
            .encrypt_with_chain(plaintext, &key_array)
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }

    fn decrypt_with_chain(&self, encrypted: &[u8], chain_key: &[u8]) -> PyResult<Vec<u8>> {
        if chain_key.len() != 32 {
            return Err(PyValueError::new_err("Chain key must be 32 bytes"));
        }
        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(chain_key);
        self.inner
            .decrypt_with_chain(encrypted, &key_array)
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
}

// Wrapper per IdentityKey
#[pyclass]
pub struct PyIdentityKey {
    inner: IdentityKey,
}

#[pymethods]
impl PyIdentityKey {
    #[staticmethod]
    fn generate() -> Self {
        PyIdentityKey {
            inner: IdentityKey::generate(),
        }
    }

    fn public_key_bytes(&self) -> Vec<u8> {
        self.inner.public_key_bytes()
    }

    fn sign(&self, message: &[u8]) -> Vec<u8> {
        self.inner.sign(message)
    }

    #[staticmethod]
    fn verify(public_key_bytes: &[u8], message: &[u8], signature_bytes: &[u8]) -> PyResult<()> {
        IdentityKey::verify(public_key_bytes, message, signature_bytes)
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }
}

// Wrapper per ChainKey
#[pyclass]
pub struct PyChainKey {
    inner: ChainKey,
}

#[pymethods]
impl PyChainKey {
    #[staticmethod]
    fn from_chat_code(chat_code: &str) -> PyResult<Self> {
        ChainKey::from_chat_code(chat_code)
            .map(|inner| PyChainKey { inner })
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }

    fn next(&mut self) -> Vec<u8> {
        self.inner.next().to_vec()
    }

    fn index(&self) -> u64 {
        self.inner.index()
    }

    fn advance_to(&mut self, target_index: u64) {
        self.inner.advance_to(target_index);
    }

    fn clone_chain(&self) -> Self {
        PyChainKey {
            inner: self.inner.clone(),
        }
    }
}

// Wrapper per MessagePayload
#[pyclass]
#[derive(Clone)]
pub struct PyMessagePayload {
    #[pyo3(get, set)]
    pub username: String,
    #[pyo3(get, set)]
    pub content: String,
    #[pyo3(get, set)]
    pub timestamp: i64,
    #[pyo3(get, set)]
    pub sequence_number: u64,
    #[pyo3(get, set)]
    pub sender_public_key: Vec<u8>,
    #[pyo3(get, set)]
    pub signature: Vec<u8>,
    #[pyo3(get, set)]
    pub chain_key_index: u64,
}

#[pymethods]
impl PyMessagePayload {
    #[new]
    fn new(
        username: String,
        content: String,
        sequence_number: u64,
        sender_public_key: Vec<u8>,
        signature: Vec<u8>,
        chain_key_index: u64,
    ) -> Self {
        PyMessagePayload {
            username,
            content,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            sequence_number,
            sender_public_key,
            signature,
            chain_key_index,
        }
    }

    fn to_bytes(&self) -> PyResult<Vec<u8>> {
        let payload = MessagePayload {
            username: self.username.clone(),
            content: self.content.clone(),
            timestamp: self.timestamp,
            sequence_number: self.sequence_number,
            sender_public_key: self.sender_public_key.clone(),
            signature: self.signature.clone(),
            chain_key_index: self.chain_key_index,
        };
        bincode::serialize(&payload)
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }

    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        let payload: MessagePayload = bincode::deserialize(data)
            .map_err(|e| PyValueError::new_err(format!("Deserialization error: {}", e)))?;
        Ok(PyMessagePayload {
            username: payload.username.clone(),
            content: payload.content.clone(),
            timestamp: payload.timestamp,
            sequence_number: payload.sequence_number,
            sender_public_key: payload.sender_public_key.clone(),
            signature: payload.signature.clone(),
            chain_key_index: payload.chain_key_index,
        })
    }
}

// Wrapper per ClientMessage
#[pyclass]
#[derive(Clone)]
pub struct PyClientMessage {
    inner: ClientMessage,
}

#[pymethods]
impl PyClientMessage {
    #[staticmethod]
    fn create_chat(room_id: String, chat_type: &str, username: String, max_participants: Option<usize>) -> PyResult<Self> {
        let ct = match chat_type {
            "OneToOne" => ChatType::OneToOne,
            "Group" => ChatType::Group {
                max_participants: max_participants.unwrap_or(8),
            },
            _ => return Err(PyValueError::new_err("Invalid chat type")),
        };
        Ok(PyClientMessage {
            inner: ClientMessage::CreateChat {
                room_id,
                chat_type: ct,
                username,
            },
        })
    }

    #[staticmethod]
    fn join_chat(room_id: String, username: String) -> Self {
        PyClientMessage {
            inner: ClientMessage::JoinChat { room_id, username },
        }
    }

    #[staticmethod]
    fn send_message(room_id: String, encrypted_payload: Vec<u8>, message_id: String) -> Self {
        PyClientMessage {
            inner: ClientMessage::SendMessage {
                room_id,
                encrypted_payload,
                message_id,
            },
        }
    }

    #[staticmethod]
    fn leave_chat(room_id: String) -> Self {
        PyClientMessage {
            inner: ClientMessage::LeaveChat { room_id },
        }
    }

    fn to_bytes(&self) -> PyResult<Vec<u8>> {
        bincode::serialize(&self.inner)
            .map_err(|e| PyValueError::new_err(format!("Serialization error: {}", e)))
    }
}

// Wrapper per ServerMessage
#[pyclass]
#[derive(Clone)]
pub struct PyServerMessage {
    msg_type: String,
    room_id: Option<String>,
    chat_type: Option<String>,
    max_participants: Option<usize>,
    participant_count: Option<usize>,
    message: Option<String>,
    encrypted_payload: Option<Vec<u8>>,
    timestamp: Option<i64>,
    message_id: Option<String>,
    username: Option<String>,
}

#[pymethods]
impl PyServerMessage {
    #[getter]
    fn msg_type(&self) -> &str {
        &self.msg_type
    }

    #[getter]
    fn room_id(&self) -> Option<&str> {
        self.room_id.as_deref()
    }

    #[getter]
    fn chat_type(&self) -> Option<&str> {
        self.chat_type.as_deref()
    }

    #[getter]
    fn max_participants(&self) -> Option<usize> {
        self.max_participants
    }

    #[getter]
    fn participant_count(&self) -> Option<usize> {
        self.participant_count
    }

    #[getter]
    fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    #[getter]
    fn encrypted_payload(&self) -> Option<Vec<u8>> {
        self.encrypted_payload.clone()
    }

    #[getter]
    fn timestamp(&self) -> Option<i64> {
        self.timestamp
    }

    #[getter]
    fn message_id(&self) -> Option<&str> {
        self.message_id.as_deref()
    }

    #[getter]
    fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    #[staticmethod]
    fn from_bytes(data: &[u8]) -> PyResult<Self> {
        let msg: ServerMessage = bincode::deserialize(data)
            .map_err(|e| PyValueError::new_err(format!("Deserialization error: {}", e)))?;
        
        match msg {
            ServerMessage::ChatCreated { room_id, chat_type } => {
                let (ct, max_p) = match chat_type {
                    ChatType::OneToOne => ("OneToOne".to_string(), None),
                    ChatType::Group { max_participants } => ("Group".to_string(), Some(max_participants)),
                };
                Ok(PyServerMessage {
                    msg_type: "ChatCreated".to_string(),
                    room_id: Some(room_id),
                    chat_type: Some(ct),
                    max_participants: max_p,
                    participant_count: None,
                    message: None,
                    encrypted_payload: None,
                    timestamp: None,
                    message_id: None,
                    username: None,
                })
            }
            ServerMessage::JoinedChat { room_id, chat_type, participant_count } => {
                let (ct, max_p) = match chat_type {
                    ChatType::OneToOne => ("OneToOne".to_string(), None),
                    ChatType::Group { max_participants } => ("Group".to_string(), Some(max_participants)),
                };
                Ok(PyServerMessage {
                    msg_type: "JoinedChat".to_string(),
                    room_id: Some(room_id),
                    chat_type: Some(ct),
                    max_participants: max_p,
                    participant_count: Some(participant_count),
                    message: None,
                    encrypted_payload: None,
                    timestamp: None,
                    message_id: None,
                    username: None,
                })
            }
            ServerMessage::Error { message } => Ok(PyServerMessage {
                msg_type: "Error".to_string(),
                room_id: None,
                chat_type: None,
                max_participants: None,
                participant_count: None,
                message: Some(message),
                encrypted_payload: None,
                timestamp: None,
                message_id: None,
                username: None,
            }),
            ServerMessage::MessageReceived { room_id, encrypted_payload, timestamp, message_id } => {
                Ok(PyServerMessage {
                    msg_type: "MessageReceived".to_string(),
                    room_id: Some(room_id),
                    chat_type: None,
                    max_participants: None,
                    participant_count: None,
                    message: None,
                    encrypted_payload: Some(encrypted_payload),
                    timestamp: Some(timestamp),
                    message_id: Some(message_id),
                    username: None,
                })
            }
            ServerMessage::MessageAck { message_id } => Ok(PyServerMessage {
                msg_type: "MessageAck".to_string(),
                room_id: None,
                chat_type: None,
                max_participants: None,
                participant_count: None,
                message: None,
                encrypted_payload: None,
                timestamp: None,
                message_id: Some(message_id),
                username: None,
            }),
            ServerMessage::UserJoined { room_id, username } => Ok(PyServerMessage {
                msg_type: "UserJoined".to_string(),
                room_id: Some(room_id),
                chat_type: None,
                max_participants: None,
                participant_count: None,
                message: None,
                encrypted_payload: None,
                timestamp: None,
                message_id: None,
                username: Some(username),
            }),
            ServerMessage::UserLeft { room_id, username } => Ok(PyServerMessage {
                msg_type: "UserLeft".to_string(),
                room_id: Some(room_id),
                chat_type: None,
                max_participants: None,
                participant_count: None,
                message: None,
                encrypted_payload: None,
                timestamp: None,
                message_id: None,
                username: Some(username),
            }),
        }
    }
}

// Funzioni utility
#[pyfunction]
fn py_generate_chat_code() -> String {
    generate_chat_code()
}

#[pyfunction]
fn py_generate_numeric_chat_code() -> String {
    generate_numeric_chat_code()
}

#[pyfunction]
fn py_chat_code_to_room_id(chat_code: &str) -> String {
    chat_code_to_room_id(chat_code)
}

/// Module initialization
#[pymodule]
fn rchat_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyChatKey>()?;
    m.add_class::<PyIdentityKey>()?;
    m.add_class::<PyChainKey>()?;
    m.add_class::<PyMessagePayload>()?;
    m.add_class::<PyClientMessage>()?;
    m.add_class::<PyServerMessage>()?;
    m.add_function(wrap_pyfunction!(py_generate_chat_code, m)?)?;
    m.add_function(wrap_pyfunction!(py_generate_numeric_chat_code, m)?)?;
    m.add_function(wrap_pyfunction!(py_chat_code_to_room_id, m)?)?;
    Ok(())
}
