#!/bin/bash

# Script per generare certificati TLS self-signed per Rchat server

echo "🔐 Generazione certificati TLS per Rchat server..."
echo ""

# Verifica che openssl sia installato
if ! command -v openssl &> /dev/null; then
    echo "❌ Errore: openssl non trovato. Installalo con:"
    echo "   Ubuntu/Debian: sudo apt install openssl"
    echo "   Fedora: sudo dnf install openssl"
    echo "   macOS: brew install openssl"
    exit 1
fi

# Genera certificato e chiave
openssl req -x509 -newkey rsa:4096 -nodes \
    -keyout server.key \
    -out server.crt \
    -days 365 \
    -subj '/CN=localhost' \
    -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ Certificati generati con successo:"
    echo "   📄 server.crt - Certificato pubblico"
    echo "   🔑 server.key - Chiave privata"
    echo ""
    echo "⚠️  ATTENZIONE: Questi sono certificati self-signed per DEMO/TEST"
    echo "   In produzione, usa certificati firmati da una CA affidabile!"
    echo ""
    echo "🚀 Ora puoi avviare il server con:"
    echo "   cargo run --bin server --release"
else
    echo ""
    echo "❌ Errore durante la generazione dei certificati"
    exit 1
fi
