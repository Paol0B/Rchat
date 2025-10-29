#!/bin/bash

# Script di test rapido per Rchat

echo "🧪 Test Build di Rchat"
echo ""

# Compila il progetto
echo "📦 Compilazione in corso..."
cargo build --release 2>&1 | tail -n 5

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Compilazione completata!"
    echo ""
    echo "📋 Binari disponibili:"
    ls -lh target/release/server target/release/client 2>/dev/null || echo "⚠️  Binari non trovati in target/release/"
    echo ""
    echo "🚀 Per avviare:"
    echo "   Terminal 1: ./target/release/server"
    echo "   Terminal 2: ./target/release/client --username Alice"
    echo "   Terminal 3: ./target/release/client --username Bob"
else
    echo ""
    echo "❌ Errore di compilazione"
    echo ""
    echo "💡 Suggerimenti:"
    echo "   1. Verifica che tutte le dipendenze siano corrette"
    echo "   2. Esegui: cargo clean && cargo build --release"
    echo "   3. Controlla gli errori sopra"
    exit 1
fi
