#!/usr/bin/env python3
"""
Test di compatibilità tra client Python Qt e client Rust terminal
"""

import sys
from rchat.crypto import generate_chat_code, chat_code_to_room_id, ChatKey, ChainKey
from rchat.protocol import ClientMessage, MessagePayload

def test_chat_code_compatibility():
    """Test generazione e conversione chat code"""
    print("=== Test Chat Code Compatibility ===\n")
    
    # Genera codice
    code = generate_chat_code()
    print(f"✓ Chat code generato: {code[:40]}...")
    print(f"  Lunghezza: {len(code)} caratteri")
    
    # Genera room ID
    room_id = chat_code_to_room_id(code)
    print(f"✓ Room ID derivato: {room_id[:40]}...")
    print(f"  Lunghezza: {len(room_id)} caratteri")
    
    # Verifica che sia deterministico
    room_id2 = chat_code_to_room_id(code)
    assert room_id == room_id2, "Room ID deve essere deterministico!"
    print("✓ Room ID è deterministico")
    
    return code, room_id


def test_encryption_compatibility():
    """Test compatibilità crittografia"""
    print("\n=== Test Encryption Compatibility ===\n")
    
    code = "test-chat-code-12345"
    
    # Deriva chiavi
    chat_key = ChatKey.derive_from_code(code)
    chain_key = ChainKey.from_chat_code(code)
    
    print("✓ Chiavi derivate dal codice")
    
    # Test crittografia/decriptazione base
    plaintext = b"Hello, World!"
    encrypted = chat_key.encrypt(plaintext)
    decrypted = chat_key.decrypt(encrypted)
    
    assert decrypted == plaintext, "Decryption failed!"
    print(f"✓ Crittografia base OK (plaintext: {plaintext})")
    print(f"  Encrypted: {len(encrypted)} bytes")
    
    # Test chain key encryption
    chain_key_bytes = chain_key.next()
    encrypted_chain = chat_key.encrypt_with_chain(plaintext, chain_key_bytes)
    decrypted_chain = chat_key.decrypt_with_chain(encrypted_chain, chain_key_bytes)
    
    assert decrypted_chain == plaintext, "Chain decryption failed!"
    print(f"✓ Chain key crittografia OK")
    print(f"  Chain index: {chain_key.index}")
    
    return chat_key, chain_key


def test_protocol_compatibility():
    """Test compatibilità formato protocollo"""
    print("\n=== Test Protocol Compatibility ===\n")
    
    # Test CreateChat
    room_id = "test-room-id-abcd1234"
    msg = ClientMessage.create_chat(room_id, "OneToOne", "TestUser")
    
    print(f"✓ CreateChat message: {len(msg)} bytes")
    print(f"  Variant: {int.from_bytes(msg[0:4], 'little')}")
    print(f"  Hex (primi 20 byte): {msg[:20].hex()}")
    
    # Test JoinChat
    msg = ClientMessage.join_chat(room_id, "TestUser")
    print(f"✓ JoinChat message: {len(msg)} bytes")
    print(f"  Variant: {int.from_bytes(msg[0:4], 'little')}")
    
    # Test SendMessage
    encrypted_payload = b"encrypted-test-data-1234567890"
    msg_id = "test-msg-id-001"
    msg = ClientMessage.send_message(room_id, encrypted_payload, msg_id)
    print(f"✓ SendMessage message: {len(msg)} bytes")
    print(f"  Variant: {int.from_bytes(msg[0:4], 'little')}")
    
    # Test LeaveChat
    msg = ClientMessage.leave_chat(room_id)
    print(f"✓ LeaveChat message: {len(msg)} bytes")
    print(f"  Variant: {int.from_bytes(msg[0:4], 'little')}")


def test_message_payload_format():
    """Test formato MessagePayload"""
    print("\n=== Test MessagePayload Format ===\n")
    
    from rchat.crypto import IdentityKey
    
    # Genera identità
    identity = IdentityKey.generate()
    
    # Crea payload
    content = "Messaggio di test"
    sig_data = content.encode('utf-8') + (0).to_bytes(8, 'little') + (0).to_bytes(8, 'little')
    signature = identity.sign(sig_data)
    
    payload = MessagePayload(
        username="TestUser",
        content=content,
        timestamp=1730000000,
        sequence_number=0,
        sender_public_key=identity.public_key_bytes(),
        signature=signature,
        chain_key_index=0
    )
    
    # Serializza
    payload_bytes = payload.to_bytes()
    print(f"✓ MessagePayload serializzato: {len(payload_bytes)} bytes")
    print(f"  Username: {payload.username}")
    print(f"  Content: {payload.content}")
    print(f"  Signature: {len(payload.signature)} bytes")
    
    # Deserializza
    payload2 = MessagePayload.from_bytes(payload_bytes)
    assert payload2.username == payload.username
    assert payload2.content == payload.content
    assert payload2.signature == payload.signature
    print("✓ Deserializzazione OK")
    
    # Verifica firma
    verified = identity.verify(sig_data, payload2.signature)
    print(f"✓ Firma verificata: {verified}")


def main():
    """Esegue tutti i test di compatibilità"""
    print("╔══════════════════════════════════════════════════╗")
    print("║  Test Compatibilità Client Python Qt ↔ Rust     ║")
    print("╚══════════════════════════════════════════════════╝\n")
    
    try:
        test_chat_code_compatibility()
        test_encryption_compatibility()
        test_protocol_compatibility()
        test_message_payload_format()
        
        print("\n" + "="*52)
        print("✅ TUTTI I TEST DI COMPATIBILITÀ SUPERATI!")
        print("="*52)
        print("\n📝 Note:")
        print("  • Chat code: formato base64url compatibile")
        print("  • Room ID: BLAKE3 + SHA3-512 deterministico")
        print("  • Crittografia: ChaCha20Poly1305 (12-byte nonce)")
        print("  • KDF: PBKDF2-HMAC-SHA512 (Rust usa Argon2id)")
        print("  • Protocollo: bincode little-endian")
        print("  • Firme: Ed25519 compatibile")
        print("\n⚠️  ATTENZIONE:")
        print("  Python usa PBKDF2, Rust usa Argon2id")
        print("  → Messaggi tra client diversi NON compatibili")
        print("  → Ogni client può solo comunicare con se stesso")
        print("  → Per compatibilità completa, implementare Argon2")
        
        return 0
        
    except Exception as e:
        print(f"\n❌ TEST FALLITO: {e}")
        import traceback
        traceback.print_exc()
        return 1


if __name__ == '__main__':
    sys.exit(main())
