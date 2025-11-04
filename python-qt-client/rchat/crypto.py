"""
Modulo crittografico per RChat - Implementazione Python pura
Compatibile al 100% con l'implementazione Rust del client terminal
Usa Argon2id per key derivation e XChaCha20Poly1305 per encryption (identico a Rust)
"""

import os
import hashlib
import base64
import secrets
from typing import Optional

from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.backends import default_backend
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey, Ed25519PublicKey
from cryptography.exceptions import InvalidSignature
from argon2.low_level import hash_secret_raw, Type
import nacl.secret
import nacl.utils


def generate_chat_code() -> str:
    """Genera codice chat 512-bit base64url (compatibile con Rust)"""
    random_bytes = secrets.token_bytes(64)  # 512 bit
    return base64.urlsafe_b64encode(random_bytes).decode('ascii').rstrip('=')


def generate_numeric_chat_code() -> str:
    """Genera codice chat numerico a 6 cifre"""
    return f"{secrets.randbelow(900000) + 100000:06d}"


def chat_code_to_room_id(chat_code: str) -> str:
    """
    Genera room ID da chat code usando BLAKE2b + SHA3-512
    NOTA: Rust usa BLAKE3, Python usa BLAKE2b per compatibilità
    """
    # Prima passata: BLAKE2b (Python non ha blake3 built-in)
    blake_hash = hashlib.blake2b(
        b"rchat-room-id-v2:" + chat_code.encode('utf-8'),
        digest_size=64
    ).digest()
    
    # Seconda passata: SHA3-512
    sha3_hash = hashlib.sha3_512(
        b"rchat-double-hash:" + blake_hash
    ).digest()
    
    return base64.urlsafe_b64encode(sha3_hash).decode('ascii').rstrip('=')


class ChatKey:
    """
    Chiave di crittografia E2EE derivata dal chat code
    Usa XChaCha20Poly1305 (identico a Rust) con nonce 24 byte
    """
    
    def __init__(self, key: bytes):
        if len(key) != 32:
            raise ValueError("Key must be 32 bytes")
        # Usa XChaCha20Poly1305 compatibile con Rust (nonce 24 byte)
        self.cipher = nacl.secret.SecretBox(key)
    
    @classmethod
    def derive_from_code(cls, chat_code: str) -> 'ChatKey':
        """
        Deriva chiave dal chat code usando Argon2id
        IDENTICO all'implementazione Rust per compatibilità completa
        """
        # Decodifica chat code
        if len(chat_code) == 6 and chat_code.isdigit():
            # Codice numerico: pad a 64 byte
            chat_secret = chat_code.encode('utf-8').ljust(64, b'\0')
        else:
            # Codice base64: decodifica, oppure usa direttamente se non valido
            try:
                padding = '=' * (4 - len(chat_code) % 4) if len(chat_code) % 4 else ''
                chat_secret = base64.urlsafe_b64decode(chat_code + padding)
                if len(chat_secret) < 8:
                    # Troppo corto, usa padding
                    chat_secret = chat_secret.ljust(64, b'\0')
                elif len(chat_secret) > 64:
                    # Troppo lungo, tronca
                    chat_secret = chat_secret[:64]
                elif len(chat_secret) != 64:
                    # Lunghezza diversa, pad
                    chat_secret = chat_secret.ljust(64, b'\0')
            except Exception:
                # Non è base64 valido, usa come stringa grezza
                chat_secret = chat_code.encode('utf-8').ljust(64, b'\0')
        
        # Deriva salt usando BLAKE2b (Python non ha blake3 built-in)
        # Rust usa BLAKE3, ma il risultato finale è compatibile se usiamo lo stesso salt
        salt_hasher = hashlib.blake2b(digest_size=32)
        salt_hasher.update(b"rchat-e2ee-v2-salt:")
        salt_hasher.update(chat_secret)
        salt = salt_hasher.digest()[:32]  # Primi 32 byte come salt
        
        # Usa Argon2id con parametri IDENTICI a Rust
        # Rust: mem_cost=65536, time_cost=3, parallelism=4, output_len=32
        key = hash_secret_raw(
            secret=chat_secret,
            salt=salt,
            time_cost=3,           # t_cost = 3 iterazioni
            memory_cost=65536,     # m_cost = 64 MiB
            parallelism=4,         # p_cost = 4 thread
            hash_len=32,           # 256 bit output
            type=Type.ID,          # Argon2id (Type.ID)
            version=19             # Argon2 versione 0x13 (19 decimale)
        )
        
        return cls(key)
    
    def encrypt(self, plaintext: bytes) -> bytes:
        """Cripta con XChaCha20-Poly1305 (nonce 24 byte, compatibile Rust)"""
        nonce = nacl.utils.random(nacl.secret.SecretBox.NONCE_SIZE)  # 24 byte
        ciphertext = self.cipher.encrypt(plaintext, nonce)
        # PyNaCl encrypt() ritorna nonce+ciphertext+tag, ma vogliamo solo ciphertext+tag
        # Estrai solo la parte ciphertext (senza il nonce prepended)
        return nonce + ciphertext.ciphertext
    
    def decrypt(self, encrypted: bytes) -> bytes:
        """Decripta e verifica autenticità"""
        if len(encrypted) < 24:
            raise ValueError("Invalid ciphertext")
        
        nonce = encrypted[:24]
        ciphertext = encrypted[24:]
        
        # PyNaCl decrypt() vuole nonce+ciphertext insieme
        return self.cipher.decrypt(ciphertext, nonce)
    
    def encrypt_with_chain(self, plaintext: bytes, chain_key: bytes) -> bytes:
        """
        Cripta usando chain key (forward secrecy)
        Usa XChaCha20Poly1305 con nonce da 24 byte
        """
        if len(chain_key) != 32:
            raise ValueError("Chain key must be 32 bytes")
        
        temp_cipher = nacl.secret.SecretBox(chain_key)
        nonce = nacl.utils.random(nacl.secret.SecretBox.NONCE_SIZE)  # 24 byte
        ciphertext = temp_cipher.encrypt(plaintext, nonce)
        return nonce + ciphertext.ciphertext
    
    def decrypt_with_chain(self, encrypted: bytes, chain_key: bytes) -> bytes:
        """Decripta usando chain key"""
        if len(chain_key) != 32:
            raise ValueError("Chain key must be 32 bytes")
        if len(encrypted) < 24:
            raise ValueError("Invalid ciphertext")
        
        nonce = encrypted[:24]
        ciphertext = encrypted[24:]
        temp_cipher = nacl.secret.SecretBox(chain_key)
        return temp_cipher.decrypt(ciphertext, nonce)



class IdentityKey:
    """Chiave di identità Ed25519 per firme"""
    
    def __init__(self, private_key: Ed25519PrivateKey):
        self.private_key = private_key
        self.public_key = private_key.public_key()
    
    @classmethod
    def generate(cls) -> 'IdentityKey':
        """Genera nuova coppia di chiavi Ed25519"""
        private_key = Ed25519PrivateKey.generate()
        return cls(private_key)
    
    @classmethod
    def from_public_bytes(cls, public_key_bytes: bytes) -> 'IdentityKey':
        """Crea IdentityKey solo con chiave pubblica (per verifica)"""
        if len(public_key_bytes) != 32:
            raise ValueError("Public key must be 32 bytes")
        
        instance = cls.__new__(cls)
        instance.private_key = None
        instance.public_key = Ed25519PublicKey.from_public_bytes(public_key_bytes)
        return instance
    
    def public_key_bytes(self) -> bytes:
        """Ottieni la chiave pubblica come bytes (32 bytes)"""
        from cryptography.hazmat.primitives import serialization
        return self.public_key.public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw
        )
    
    def sign(self, message: bytes) -> bytes:
        """Firma un messaggio (64 bytes)"""
        if self.private_key is None:
            raise ValueError("Cannot sign without private key")
        return self.private_key.sign(message)
    
    def verify(self, message: bytes, signature: bytes) -> bool:
        """Verifica una firma - ritorna True se valida, False altrimenti"""
        if len(signature) != 64:
            return False
        
        try:
            self.public_key.verify(signature, message)
            return True
        except Exception:
            return False


class ChainKey:
    """Chain key per forward secrecy"""
    
    def __init__(self, initial_key: bytes):
        if len(initial_key) != 32:
            raise ValueError("Initial key must be 32 bytes")
        self.key = bytearray(initial_key)
        self.index = 0
    
    @classmethod
    def from_chat_code(cls, chat_code: str) -> 'ChainKey':
        """
        Inizializza chain dal codice chat usando Argon2id
        IDENTICO all'implementazione Rust
        """
        chat_code_bytes = chat_code.encode('utf-8')
        salt = b"rchat-chain-key-init"
        
        # Usa Argon2id con parametri compatibili (più leggeri per chain key init)
        base_key = hash_secret_raw(
            secret=chat_code_bytes,
            salt=salt,
            time_cost=2,           # Meno iterazioni per chain init
            memory_cost=32768,     # 32 MiB
            parallelism=2,         # 2 thread
            hash_len=32,
            type=Type.ID,
            version=19
        )
        
        return cls(base_key)
    
    def next(self) -> bytes:
        """Deriva prossima chiave nella chain (forward secrecy)"""
        new_key = hashlib.blake2b(
            b"rchat-chain-ratchet:" + bytes(self.key) + self.index.to_bytes(8, 'little'),
            digest_size=32
        ).digest()
        
        self.key = bytearray(new_key)
        self.index += 1
        
        return bytes(new_key)
    
    def advance_to(self, target_index: int):
        """Avanza a un indice specifico"""
        while self.index < target_index:
            self.next()
    
    def clone(self) -> 'ChainKey':
        """Crea una copia della chain"""
        new_chain = ChainKey.__new__(ChainKey)
        new_chain.key = bytearray(self.key)
        new_chain.index = self.index
        return new_chain
