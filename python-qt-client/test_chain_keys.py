#!/usr/bin/env python3
"""
Test per verificare che le chain keys derivate in Python siano identiche a Rust
"""

import sys
sys.path.insert(0, '/home/paol0b/sources/Rchat/python-qt-client')

from rchat.crypto import ChainKey

def test_chain_keys():
    """Testa la derivazione delle chain keys"""
    
    # Usa lo stesso chat_code di test
    chat_code = "123456"
    
    print("=" * 80)
    print("Test Chain Key Derivation")
    print("=" * 80)
    print(f"Chat code: {chat_code}")
    print()
    
    # Crea chain
    chain = ChainKey.from_chat_code(chat_code)
    print(f"Initial chain index: {chain.index}")
    print(f"Initial key (first 32 bytes): {bytes(chain.key).hex()}")
    print()
    
    # Deriva le prime 5 chiavi
    print("Deriving first 5 message keys:")
    print("-" * 80)
    
    for i in range(5):
        print(f"\nMessage {i}:")
        print(f"  Chain index before next(): {chain.index}")
        
        # Ottieni prossima chiave
        key = chain.next()
        
        print(f"  Chain index after next():  {chain.index}")
        print(f"  Message key (index {i}): {key.hex()}")
        print(f"  chain_key_index in payload: {i}")
    
    print("\n" + "=" * 80)
    print("Now let's verify decryption simulation:")
    print("=" * 80)
    
    # Simula decryption del messaggio 0
    print("\nDecrypting message with chain_key_index=0:")
    test_chain = ChainKey.from_chat_code(chat_code)
    test_chain.advance_to(0)  # Should stay at 0
    print(f"  Test chain index after advance_to(0): {test_chain.index}")
    test_key = test_chain.next()
    print(f"  Test chain index after next(): {test_chain.index}")
    print(f"  Derived key: {test_key.hex()}")
    
    # Simula decryption del messaggio 3
    print("\nDecrypting message with chain_key_index=3:")
    test_chain2 = ChainKey.from_chat_code(chat_code)
    test_chain2.advance_to(3)
    print(f"  Test chain index after advance_to(3): {test_chain2.index}")
    test_key2 = test_chain2.next()
    print(f"  Test chain index after next(): {test_chain2.index}")
    print(f"  Derived key: {test_key2.hex()}")
    
    print("\n" + "=" * 80)
    print("Verification: Compare with Rust output")
    print("=" * 80)
    print("Run the Rust client and compare the keys!")
    print()

if __name__ == '__main__':
    test_chain_keys()
