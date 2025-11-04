"""
Controller per gestire la logica business del client
Usa implementazione Python pura per crittografia e protocollo
"""

import time
import uuid
from typing import Optional, Dict, List
from PyQt6.QtCore import QObject, pyqtSignal

from .crypto import ChatKey, IdentityKey, ChainKey, generate_chat_code, generate_numeric_chat_code, chat_code_to_room_id
from .protocol import MessagePayload, ClientMessage, ServerMessage


class ChatController(QObject):
    """Controller per gestire lo stato e la logica della chat"""
    
    # Segnali
    message_sent = pyqtSignal(dict)  # Messaggio inviato con successo
    message_received = pyqtSignal(dict)  # Nuovo messaggio ricevuto
    chat_created = pyqtSignal(str)  # Chat creata con codice
    chat_joined = pyqtSignal(int)  # Unito a chat con N partecipanti
    user_joined = pyqtSignal(str)  # Utente unito
    user_left = pyqtSignal(str)  # Utente uscito
    error_occurred = pyqtSignal(str)  # Errore
    message_ack = pyqtSignal(str)  # ACK messaggio ricevuto
    
    def __init__(self, username: str, numeric_codes: bool = False):
        super().__init__()
        self.username = username
        self.numeric_codes = numeric_codes
        
        # State
        self.current_chat_code: Optional[str] = None
        self.current_room_id: Optional[str] = None
        self.chat_key: Optional[ChatKey] = None
        self.identity_key: IdentityKey = IdentityKey.generate()
        self.chain_key: Optional[ChainKey] = None
        self.sequence_number = 0
        
        # Pending messages for retry
        self.pending_messages: Dict[str, dict] = {}
    
    def generate_chat_code(self) -> str:
        """Genera un nuovo codice chat"""
        if self.numeric_codes:
            return generate_numeric_chat_code()
        else:
            return generate_chat_code()
    
    def create_chat(self, chat_code: str, chat_type: str = "OneToOne", 
                   max_participants: int = 8) -> bytes:
        """
        Crea una nuova chat
        Returns: Messaggio serializzato da inviare al server
        """
        # Deriva chiavi dal codice
        self.current_chat_code = chat_code
        self.current_room_id = chat_code_to_room_id(chat_code)
        self.chat_key = ChatKey.derive_from_code(chat_code)
        self.chain_key = ChainKey.from_chat_code(chat_code)
        self.sequence_number = 0
        
        # Crea messaggio per server
        max_p = max_participants if chat_type == "Group" else None
        return ClientMessage.create_chat(
            self.current_room_id,
            chat_type,
            self.username,
            max_p
        )
    
    def join_chat(self, chat_code: str) -> bytes:
        """
        Unisciti a una chat esistente
        Returns: Messaggio serializzato da inviare al server
        """
        # Deriva chiavi dal codice
        self.current_chat_code = chat_code
        self.current_room_id = chat_code_to_room_id(chat_code)
        self.chat_key = ChatKey.derive_from_code(chat_code)
        self.chain_key = ChainKey.from_chat_code(chat_code)
        self.sequence_number = 0
        
        # Crea messaggio per server
        return ClientMessage.join_chat(self.current_room_id, self.username)
    
    def send_message(self, content: str) -> Optional[bytes]:
        """
        Cripta e firma un messaggio
        Returns: Messaggio serializzato da inviare, o None se errore
        """
        if not self.chat_key or not self.chain_key:
            return None
        
        # Genera ID univoco per il messaggio
        message_id = str(uuid.uuid4())
        
        # Ottieni prossima chiave dalla chain
        message_key = self.chain_key.next()
        chain_index = self.chain_key.index - 1  # index è un attributo, non una funzione
        
        # Crea dati per firma
        sig_data = content.encode('utf-8')
        sig_data += self.sequence_number.to_bytes(8, 'little')
        sig_data += chain_index.to_bytes(8, 'little')
        
        # Firma il messaggio
        signature = self.identity_key.sign(sig_data)
        public_key = self.identity_key.public_key_bytes()
        
        # Crea payload
        payload = MessagePayload(
            username=self.username,
            content=content,
            timestamp=int(time.time()),
            sequence_number=self.sequence_number,
            sender_public_key=public_key,
            signature=signature,
            chain_key_index=chain_index
        )
        
        # Serializza payload
        payload_bytes = payload.to_bytes()
        
        # Cripta con chiave derivata
        encrypted = self.chat_key.encrypt_with_chain(payload_bytes, message_key)
        
        # Crea messaggio per server
        msg_bytes = ClientMessage.send_message(
            self.current_room_id,
            encrypted,
            message_id
        )
        
        # Salva come pending
        self.pending_messages[message_id] = {
            'content': content,
            'timestamp': int(time.time()),
            'sequence': self.sequence_number,
            'encrypted': encrypted,
            'retry_count': 0
        }
        
        self.sequence_number += 1
        
        return msg_bytes
    
    def handle_server_message(self, data: bytes):
        """Gestisce un messaggio ricevuto dal server"""
        try:
            msg = ServerMessage.from_bytes(data)
            msg_type = msg.get_type()
            
            if msg_type == "ChatCreated":
                self.chat_created.emit(self.current_chat_code)
            
            elif msg_type == "JoinedChat":
                count = msg.get('participant_count', 0)
                self.chat_joined.emit(count)
            
            elif msg_type == "Error":
                self.error_occurred.emit(msg.get('message', 'Unknown error'))
            
            elif msg_type == "MessageAck":
                msg_id = msg.get('message_id')
                if msg_id and msg_id in self.pending_messages:
                    del self.pending_messages[msg_id]
                    self.message_ack.emit(msg_id)
            
            elif msg_type == "MessageReceived":
                encrypted = msg.get('encrypted_payload')
                if encrypted and self.chat_key and self.chain_key:
                    decrypted_msg = self._decrypt_message(encrypted)
                    if decrypted_msg:
                        self.message_received.emit(decrypted_msg)
            
            elif msg_type == "UserJoined":
                username = msg.get('username')
                if username:
                    self.user_joined.emit(username)
            
            elif msg_type == "UserLeft":
                username = msg.get('username')
                if username:
                    self.user_left.emit(username)
        
        except Exception as e:
            self.error_occurred.emit(f"Error handling message: {str(e)}")
    
    def _decrypt_message(self, encrypted: bytes) -> Optional[dict]:
        """Decripta un messaggio ricevuto"""
        # Prova a decriptare con diversi indici della chain
        current_index = self.chain_key.index  # index è un attributo, non una funzione
        
        for test_index in range(
            max(0, current_index - 5),
            current_index + 20
        ):
            try:
                # Crea chain temporanea per test
                test_chain = self.chain_key.clone()
                test_chain.advance_to(test_index)
                test_key = test_chain.next()
                
                # Prova a decriptare
                decrypted = self.chat_key.decrypt_with_chain(encrypted, test_key)
                
                # Prova a deserializzare
                payload = MessagePayload.from_bytes(decrypted)
                
                # Verifica indice chain
                if payload.chain_key_index != test_index:
                    continue
                
                # Verifica firma
                sig_data = payload.content.encode('utf-8')
                sig_data += payload.sequence_number.to_bytes(8, 'little')
                sig_data += payload.chain_key_index.to_bytes(8, 'little')
                
                # Verifica firma con chiave pubblica del sender
                sender_identity = IdentityKey.from_public_bytes(payload.sender_public_key)
                
                try:
                    verified = sender_identity.verify(sig_data, payload.signature)
                except:
                    verified = False
                
                # Avanza chain al prossimo indice
                self.chain_key.advance_to(test_index + 1)
                
                # Controlla se è il nostro messaggio (echo dal server)
                is_own = payload.username == self.username
                
                return {
                    'username': payload.username,
                    'content': payload.content,
                    'timestamp': payload.timestamp,
                    'verified': verified,
                    'sent': True,
                    'is_own': is_own
                }
            
            except Exception:
                continue
        
        return None
    
    def leave_chat(self) -> bytes:
        """
        Esci dalla chat corrente
        Returns: Messaggio serializzato da inviare al server
        """
        msg_bytes = ClientMessage.leave_chat(self.current_room_id)
        
        # Reset state
        self.current_chat_code = None
        self.current_room_id = None
        self.chat_key = None
        self.chain_key = None
        self.sequence_number = 0
        self.pending_messages.clear()
        
        return msg_bytes
    
    def retry_pending_messages(self) -> List[bytes]:
        """
        Riprova a inviare messaggi pending
        Returns: Lista di messaggi da re-inviare
        """
        messages_to_retry = []
        current_time = int(time.time())
        
        for msg_id, msg_data in list(self.pending_messages.items()):
            elapsed = current_time - msg_data['timestamp']
            
            if elapsed >= 2:  # Timeout 2 secondi
                if msg_data['retry_count'] < 3:
                    # Retry
                    msg_data['retry_count'] += 1
                    msg_data['timestamp'] = current_time
                    
                    retry_msg = ClientMessage.send_message(
                        self.current_room_id,
                        msg_data['encrypted'],
                        msg_id
                    )
                    messages_to_retry.append(retry_msg)
                else:
                    # Max retries reached - remove
                    del self.pending_messages[msg_id]
        
        return messages_to_retry
