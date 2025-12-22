#!/usr/bin/env python3
"""
Test per verificare compatibilità completa della serializzazione
"""

import sys
sys.path.insert(0, '/home/paol0b/sources/Rchat/python-qt-client')

from rchat.protocol import MessagePayload
from rchat.crypto import IdentityKey

def test_exact_serialization():
    """Testa la serializzazione con valori fissi per confronto con Rust"""
    
    print("=" * 80)
    print("Test Exact MessagePayload Serialization")
    print("=" * 80)
    
    # Usa valori FISSI per confronto
    username = "BestMak"
    content = "asd"
    sequence_number = 2
    chain_key_index = 2
    timestamp = 1700000000  # Fixed timestamp
    
    # Genera chiavi fisse per il test (nota: saranno diverse ogni volta)
    # Per un test reale dovremmo usare chiavi fisse
    identity = IdentityKey.generate()
    public_key = identity.public_key_bytes()
    
    # Crea signature data
    sig_data = content.encode('utf-8')
    sig_data += sequence_number.to_bytes(8, 'little')
    sig_data += chain_key_index.to_bytes(8, 'little')
    
    signature = identity.sign(sig_data)
    
    print(f"Fixed values:")
    print(f"  username: '{username}' ({len(username)} chars)")
    print(f"  content: '{content}' ({len(content)} chars)")
    print(f"  sequence: {sequence_number}")
    print(f"  chain_index: {chain_key_index}")
    print(f"  timestamp: {timestamp}")
    print()
    
    # Crea payload
    payload = MessagePayload.create(
        username=username,
        content=content,
        sequence_number=sequence_number,
        sender_public_key=public_key,
        signature=signature,
        chain_key_index=chain_key_index
    )
    
    # Override timestamp for fixed test
    payload.timestamp = timestamp
    
    # Ricalcola message_hash con timestamp fisso
    import blake3
    hasher = blake3.blake3()
    hasher.update(b"rchat-v3-message-commitment:")
    hasher.update(payload.username.encode('utf-8'))
    hasher.update(payload.content.encode('utf-8'))
    hasher.update(payload.sequence_number.to_bytes(8, 'little'))
    hasher.update(payload.chain_key_index.to_bytes(8, 'little'))
    payload.message_hash = hasher.digest()[:32]
    
    print(f"Payload structure:")
    print(f"  username: {len(payload.username)} bytes -> '{payload.username}'")
    print(f"  content: {len(payload.content)} bytes -> '{payload.content}'")
    print(f"  timestamp: 8 bytes -> {payload.timestamp}")
    print(f"  sequence_number: 8 bytes -> {payload.sequence_number}")
    print(f"  sender_public_key: {len(payload.sender_public_key)} bytes")
    print(f"  signature: {len(payload.signature)} bytes")
    print(f"  chain_key_index: 8 bytes -> {payload.chain_key_index}")
    print(f"  message_hash: {len(payload.message_hash)} bytes")
    print()
    
    # Calcola dimensioni previste
    expected_size = (
        8 + len(payload.username) +  # username
        8 + len(payload.content) +    # content
        8 +                            # timestamp
        8 +                            # sequence_number
        8 + len(payload.sender_public_key) +  # sender_public_key
        8 + len(payload.signature) +          # signature
        8 +                            # chain_key_index
        8 + len(payload.message_hash)         # message_hash
    )
    
    print(f"Expected serialized size calculation:")
    print(f"  username: 8 + {len(payload.username)} = {8 + len(payload.username)}")
    print(f"  content: 8 + {len(payload.content)} = {8 + len(payload.content)}")
    print(f"  timestamp: 8")
    print(f"  sequence: 8")
    print(f"  public_key: 8 + {len(payload.sender_public_key)} = {8 + len(payload.sender_public_key)}")
    print(f"  signature: 8 + {len(payload.signature)} = {8 + len(payload.signature)}")
    print(f"  chain_index: 8")
    print(f"  message_hash: 8 + {len(payload.message_hash)} = {8 + len(payload.message_hash)}")
    print(f"  TOTAL: {expected_size} bytes")
    print()
    
    # Serializza
    serialized = payload.to_bytes()
    print(f"Actual serialized size: {len(serialized)} bytes")
    
    if len(serialized) != expected_size:
        print(f"❌ SIZE MISMATCH! Expected {expected_size}, got {len(serialized)}")
    else:
        print(f"✅ Size matches expected!")
    
    print()
    print("Hex dump (full):")
    for i in range(0, len(serialized), 32):
        chunk = serialized[i:i+32]
        hex_str = chunk.hex()
        ascii_str = ''.join(chr(b) if 32 <= b < 127 else '.' for b in chunk)
        print(f"  {i:04x}: {hex_str:64s} {ascii_str}")
    
    print()
    print("=" * 80)
    print("Compare this with Rust serialization of same values!")
    print("=" * 80)

if __name__ == '__main__':
    test_exact_serialization()
