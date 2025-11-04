"""
Modulo protocollo per RChat - Implementazione Python pura
Compatibile con il formato bincode del server Rust usando MessagePack
"""

import msgpack
import time
from typing import Optional, Dict, Any
from dataclasses import dataclass


@dataclass
class MessagePayload:
    """Payload del messaggio prima della crittografia"""
    username: str
    content: str
    timestamp: int
    sequence_number: int
    sender_public_key: bytes
    signature: bytes
    chain_key_index: int
    
    def to_bytes(self) -> bytes:
        """Serializza a bytes usando struct binario (compatibile bincode)"""
        # Bincode serializza struct come sequenza di campi
        result = b''
        
        # String: lunghezza u64 LE + bytes UTF-8
        username_bytes = self.username.encode('utf-8')
        result += len(username_bytes).to_bytes(8, 'little')
        result += username_bytes
        
        # String: lunghezza u64 LE + bytes UTF-8
        content_bytes = self.content.encode('utf-8')
        result += len(content_bytes).to_bytes(8, 'little')
        result += content_bytes
        
        # u64 little endian
        result += self.timestamp.to_bytes(8, 'little')
        
        # u64 little endian
        result += self.sequence_number.to_bytes(8, 'little')
        
        # Vec<u8>: lunghezza u64 LE + bytes
        result += len(self.sender_public_key).to_bytes(8, 'little')
        result += self.sender_public_key
        
        # Vec<u8>: lunghezza u64 LE + bytes
        result += len(self.signature).to_bytes(8, 'little')
        result += self.signature
        
        # u64 little endian
        result += self.chain_key_index.to_bytes(8, 'little')
        
        return result
    
    @classmethod
    def from_bytes(cls, data: bytes) -> 'MessagePayload':
        """Deserializza da bytes (formato bincode)"""
        pos = 0
        
        # Leggi username
        username_len = int.from_bytes(data[pos:pos+8], 'little')
        pos += 8
        username = data[pos:pos+username_len].decode('utf-8')
        pos += username_len
        
        # Leggi content
        content_len = int.from_bytes(data[pos:pos+8], 'little')
        pos += 8
        content = data[pos:pos+content_len].decode('utf-8')
        pos += content_len
        
        # Leggi timestamp
        timestamp = int.from_bytes(data[pos:pos+8], 'little')
        pos += 8
        
        # Leggi sequence_number
        sequence_number = int.from_bytes(data[pos:pos+8], 'little')
        pos += 8
        
        # Leggi sender_public_key
        key_len = int.from_bytes(data[pos:pos+8], 'little')
        pos += 8
        sender_public_key = data[pos:pos+key_len]
        pos += key_len
        
        # Leggi signature
        sig_len = int.from_bytes(data[pos:pos+8], 'little')
        pos += 8
        signature = data[pos:pos+sig_len]
        pos += sig_len
        
        # Leggi chain_key_index
        chain_key_index = int.from_bytes(data[pos:pos+8], 'little')
        
        return cls(
            username=username,
            content=content,
            timestamp=timestamp,
            sequence_number=sequence_number,
            sender_public_key=sender_public_key,
            signature=signature,
            chain_key_index=chain_key_index
        )


class ClientMessage:
    """Messaggi dal client al server - formato bincode compatibile"""
    
    @staticmethod
    def _encode_string(s: str) -> bytes:
        """Codifica stringa in formato bincode"""
        s_bytes = s.encode('utf-8')
        return len(s_bytes).to_bytes(8, 'little') + s_bytes
    
    @staticmethod
    def _encode_option_u64(val: Optional[int]) -> bytes:
        """Codifica Option<u64> in formato bincode"""
        if val is None:
            return b'\x00'  # None variant
        else:
            return b'\x01' + val.to_bytes(8, 'little')  # Some(val)
    
    @staticmethod
    def create_chat(room_id: str, chat_type: str, username: str, 
                   max_participants: Optional[int] = None) -> bytes:
        """Crea messaggio CreateChat"""
        # Enum variant CreateChat = 0
        result = b'\x00\x00\x00\x00'  # u32 LE variant index
        
        # room_id: String
        result += ClientMessage._encode_string(room_id)
        
        # chat_type: enum ChatType
        if chat_type == 'OneToOne':
            result += b'\x00\x00\x00\x00'  # OneToOne = 0
        else:
            result += b'\x01\x00\x00\x00'  # Group = 1
            result += ClientMessage._encode_option_u64(max_participants)
        
        # username: String
        result += ClientMessage._encode_string(username)
        
        return result
    
    @staticmethod
    def join_chat(room_id: str, username: str) -> bytes:
        """Crea messaggio JoinChat"""
        # Enum variant JoinChat = 1
        result = b'\x01\x00\x00\x00'  # u32 LE variant index
        
        # room_id: String
        result += ClientMessage._encode_string(room_id)
        
        # username: String
        result += ClientMessage._encode_string(username)
        
        return result
    
    @staticmethod
    def send_message(room_id: str, encrypted_payload: bytes, message_id: str) -> bytes:
        """Crea messaggio SendMessage"""
        # Enum variant SendMessage = 2
        result = b'\x02\x00\x00\x00'  # u32 LE variant index
        
        # room_id: String
        result += ClientMessage._encode_string(room_id)
        
        # encrypted_payload: Vec<u8>
        result += len(encrypted_payload).to_bytes(8, 'little')
        result += encrypted_payload
        
        # message_id: String
        result += ClientMessage._encode_string(message_id)
        
        return result
    
    @staticmethod
    def leave_chat(room_id: str) -> bytes:
        """Crea messaggio LeaveChat"""
        # Enum variant LeaveChat = 3
        result = b'\x03\x00\x00\x00'  # u32 LE variant index
        
        # room_id: String
        result += ClientMessage._encode_string(room_id)
        
        return result


class ServerMessage:
    """Messaggi dal server al client"""
    
    def __init__(self, data: Dict[str, Any]):
        self.data = data
        # Il tipo è la chiave del dict (enum variant)
        self.msg_type = list(data.keys())[0] if data else 'Unknown'
        self.payload = data.get(self.msg_type, {})
    
    @classmethod
    def from_bytes(cls, data: bytes) -> 'ServerMessage':
        """Deserializza messaggio server (formato bincode)"""
        try:
            # Leggi variant (u32 LE)
            if len(data) < 4:
                return cls({'Error': {'message': 'Invalid message: too short'}})
            
            variant = int.from_bytes(data[0:4], 'little')
            pos = 4
            
            def read_string(data: bytes, pos: int) -> tuple[str, int]:
                """Leggi stringa bincode"""
                str_len = int.from_bytes(data[pos:pos+8], 'little')
                pos += 8
                s = data[pos:pos+str_len].decode('utf-8')
                return s, pos + str_len
            
            def read_vec_u8(data: bytes, pos: int) -> tuple[bytes, int]:
                """Leggi Vec<u8> bincode"""
                vec_len = int.from_bytes(data[pos:pos+8], 'little')
                pos += 8
                vec = data[pos:pos+vec_len]
                return vec, pos + vec_len
            
            def read_u64(data: bytes, pos: int) -> tuple[int, int]:
                """Leggi u64 LE"""
                val = int.from_bytes(data[pos:pos+8], 'little')
                return val, pos + 8
            
            def read_i64(data: bytes, pos: int) -> tuple[int, int]:
                """Leggi i64 LE"""
                val = int.from_bytes(data[pos:pos+8], 'little', signed=True)
                return val, pos + 8
            
            def read_chat_type(data: bytes, pos: int) -> tuple[Dict[str, Any], int]:
                """Leggi ChatType enum"""
                type_variant = int.from_bytes(data[pos:pos+4], 'little')
                pos += 4
                if type_variant == 0:
                    return {'OneToOne': None}, pos
                elif type_variant == 1:
                    max_p, pos = read_u64(data, pos)
                    return {'Group': {'max_participants': max_p}}, pos
                else:
                    return {'Unknown': None}, pos
            
            # Parse based on variant
            if variant == 0:  # ChatCreated
                room_id, pos = read_string(data, pos)
                chat_type, pos = read_chat_type(data, pos)
                return cls({'ChatCreated': {
                    'room_id': room_id,
                    'chat_type': chat_type
                }})
            
            elif variant == 1:  # JoinedChat
                room_id, pos = read_string(data, pos)
                chat_type, pos = read_chat_type(data, pos)
                count, _ = read_u64(data, pos)
                return cls({'JoinedChat': {
                    'room_id': room_id,
                    'chat_type': chat_type,
                    'participant_count': count
                }})
            
            elif variant == 2:  # Error
                message, _ = read_string(data, pos)
                return cls({'Error': {'message': message}})
            
            elif variant == 3:  # MessageReceived
                room_id, pos = read_string(data, pos)
                encrypted, pos = read_vec_u8(data, pos)
                timestamp, pos = read_i64(data, pos)
                msg_id, _ = read_string(data, pos)
                return cls({'MessageReceived': {
                    'room_id': room_id,
                    'encrypted_payload': encrypted,
                    'timestamp': timestamp,
                    'message_id': msg_id
                }})
            
            elif variant == 4:  # MessageAck
                msg_id, _ = read_string(data, pos)
                return cls({'MessageAck': {'message_id': msg_id}})
            
            elif variant == 5:  # UserJoined
                room_id, pos = read_string(data, pos)
                username, _ = read_string(data, pos)
                return cls({'UserJoined': {
                    'room_id': room_id,
                    'username': username
                }})
            
            elif variant == 6:  # UserLeft
                room_id, pos = read_string(data, pos)
                username, _ = read_string(data, pos)
                return cls({'UserLeft': {
                    'room_id': room_id,
                    'username': username
                }})
            
            else:
                return cls({'Error': {'message': f'Unknown variant: {variant}'}})
        
        except Exception as e:
            return cls({'Error': {'message': f'Deserialization failed: {str(e)}'}})
    
    def get_type(self) -> str:
        """Ottieni tipo messaggio"""
        return self.msg_type
    
    def get(self, key: str, default=None):
        """Ottieni campo dal payload del messaggio"""
        if isinstance(self.payload, dict):
            return self.payload.get(key, default)
        return default
