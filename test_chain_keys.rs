use rchat_common::crypto::ChainKey;

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn main() {
    println!("{}", "=".repeat(80));
    println!("Test Chain Key Derivation (Rust)");
    println!("{}", "=".repeat(80));
    
    let chat_code = "123456";
    println!("Chat code: {}\n", chat_code);
    
    // Crea chain
    let mut chain = ChainKey::from_chat_code(chat_code).unwrap();
    println!("Initial chain index: {}", chain.index());
    println!();
    
    // Deriva le prime 5 chiavi
    println!("Deriving first 5 message keys:");
    println!("{}", "-".repeat(80));
    
    for i in 0..5 {
        println!("\nMessage {}:", i);
        println!("  Chain index before next(): {}", chain.index());
        
        // Ottieni prossima chiave
        let key = chain.next();
        
        println!("  Chain index after next():  {}", chain.index());
        println!("  Message key (index {}): {}", i, to_hex(&key));
        println!("  chain_key_index in payload: {}", i);
    }
    
    println!("\n{}", "=".repeat(80));
    println!("Now let's verify decryption simulation:");
    println!("{}", "=".repeat(80));
    
    // Simula decryption del messaggio 0
    println!("\nDecrypting message with chain_key_index=0:");
    let mut test_chain = ChainKey::from_chat_code(chat_code).unwrap();
    test_chain.advance_to(0);
    println!("  Test chain index after advance_to(0): {}", test_chain.index());
    let test_key = test_chain.next();
    println!("  Test chain index after next(): {}", test_chain.index());
    println!("  Derived key: {}", to_hex(&test_key));
    
    // Simula decryption del messaggio 3
    println!("\nDecrypting message with chain_key_index=3:");
    let mut test_chain2 = ChainKey::from_chat_code(chat_code).unwrap();
    test_chain2.advance_to(3);
    println!("  Test chain index after advance_to(3): {}", test_chain2.index());
    let test_key2 = test_chain2.next();
    println!("  Test chain index after next(): {}", test_chain2.index());
    println!("  Derived key: {}", to_hex(&test_key2));
    
    println!("\n{}", "=".repeat(80));
    println!("Compare with Python output to verify compatibility!");
    println!("{}", "=".repeat(80));
}
