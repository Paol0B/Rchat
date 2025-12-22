#!/usr/bin/env python3
"""
Test per verificare la serializzazione del MessagePayload
"""

import sys
sys.path.insert(0, '/home/paol0b/sources/Rchat/python-qt-client')

from rchat.protocol import MessagePayload
from rchat.crypto import IdentityKey

def test_payload_serialization():
    """Testa la serializzazione del MessagePayload"""
    
    print("=" * 80)
    print("Test MessagePayload Serialization")
    print("=" * 80)
    
    # Crea identity key
    identity = IdentityKey.generate()
    public_key = identity.public_key_bytes()
    
    # Crea signature data
    content = "Hello World"
    sequence_number = 0
    chain_key_index = 0
    
    sig_data = content.encode('utf-8')
    sig_data += sequence_number.to_bytes(8, 'little')
    sig_data += chain_key_index.to_bytes(8, 'little')
    
    signature = identity.sign(sig_data)
    
    print(f"Content: {content}")
    print(f"Sequence: {sequence_number}")
    print(f"Chain index: {chain_key_index}")
    print(f"Public key ({len(public_key)} bytes): {public_key.hex()}")
    print(f"Signature ({len(signature)} bytes): {signature.hex()}")
    print()
    
    # Crea payload
    payload = MessagePayload.create(
        username="testuser",
        content=content,
        sequence_number=sequence_number,
        sender_public_key=public_key,
        signature=signature,
        chain_key_index=chain_key_index
    )
    
    print(f"Payload fields:")
    print(f"  username: {payload.username}")
    print(f"  content: {payload.content}")
    print(f"  timestamp: {payload.timestamp}")
    print(f"  sequence_number: {payload.sequence_number}")
    print(f"  chain_key_index: {payload.chain_key_index}")
    print(f"  sender_public_key ({len(payload.sender_public_key)} bytes): {payload.sender_public_key.hex()}")
    print(f"  signature ({len(payload.signature)} bytes): {payload.signature.hex()}")
    print(f"  message_hash ({len(payload.message_hash)} bytes): {payload.message_hash.hex()}")
    print()
    
    # Serializza
    serialized = payload.to_bytes()
    print(f"Serialized payload: {len(serialized)} bytes")
    print(f"Hex dump:")
    for i in range(0, len(serialized), 32):
        chunk = serialized[i:i+32]
        hex_str = chunk.hex()
        ascii_str = ''.join(chr(b) if 32 <= b < 127 else '.' for b in chunk)
        print(f"  {i:04x}: {hex_str:64s} {ascii_str}")
    print()
    
    # Deserializza
    print("Deserializing...")
    payload2 = MessagePayload.from_bytes(serialized)
    print(f"  username: {payload2.username}")
    print(f"  content: {payload2.content}")
    print(f"  timestamp: {payload2.timestamp}")
    print(f"  sequence_number: {payload2.sequence_number}")
    print(f"  chain_key_index: {payload2.chain_key_index}")
    print(f"  Signatures match: {payload.signature == payload2.signature}")
    print(f"  Hashes match: {payload.message_hash == payload2.message_hash}")
    print()
    
    print("=" * 80)
    print("✅ Test completed - compare with Rust output!")
    print("=" * 80)

if __name__ == '__main__':
    test_payload_serialization()
