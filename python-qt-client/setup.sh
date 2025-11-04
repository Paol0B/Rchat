#!/bin/bash
# Setup completo per RChat Qt Client

set -e

echo "🚀 RChat Qt Client - Setup"
echo "=========================="
echo ""

# Check Python version
python_version=$(python3 --version 2>&1 | awk '{print $2}')
echo "✓ Python version: $python_version"

# Check Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust non trovato!"
    echo "   Installa Rust da: https://rustup.rs/"
    exit 1
fi
echo "✓ Rust: $(rustc --version)"
echo ""

# Vai alla directory del progetto
cd "$(dirname "$0")"

# 1. Installa dipendenze Python
echo "📦 Installazione dipendenze Python..."
pip install -r requirements.txt

# 2. Installa maturin se non presente
if ! command -v maturin &> /dev/null; then
    echo "📦 Installazione maturin..."
    pip install maturin
fi

# 3. Compila bindings Rust
echo ""
echo "🦀 Compilazione bindings Rust (rchat_core)..."
echo "   Questo potrebbe richiedere qualche minuto..."
maturin build --release

# 4. Installa wheel generato
echo ""
echo "📦 Installazione wheel..."
wheel_file=$(find . -name "*.whl" -type f -newer pyproject.toml 2>/dev/null | head -n1)
if [ -n "$wheel_file" ]; then
    pip install --force-reinstall "$wheel_file"
else
    echo "❌ Nessun wheel trovato!"
    exit 1
fi

# 5. Test import
echo ""
echo "🧪 Test import modulo..."
if python3 -c "import rchat_core; print('✅ rchat_core importato con successo!')"; then
    echo ""
    echo "✅ Setup completato con successo!"
    echo ""
    echo "Per avviare il client:"
    echo "  python3 main.py"
    echo ""
else
    echo ""
    echo "❌ Errore durante l'import di rchat_core"
    echo "   Verifica i log sopra per dettagli"
    exit 1
fi
