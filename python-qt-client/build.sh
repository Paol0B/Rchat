#!/bin/bash
# Build script per RChat Qt Client

set -e

echo "🔧 Building RChat Qt Client..."

# Check se maturin è installato
if ! command -v maturin &> /dev/null; then
    echo "📦 Installing maturin..."
    pip install maturin
fi

# Build bindings Rust
echo "🦀 Building Rust bindings..."
cd "$(dirname "$0")"
maturin develop --release

echo "✅ Build completed!"
echo ""
echo "Run the client with: python main.py"
