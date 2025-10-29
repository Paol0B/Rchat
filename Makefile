.PHONY: all build release clean test certs server client help

# Default target
all: certs build

# Build in debug mode
build:
	@echo "🔨 Building Rchat..."
	cargo build --workspace

# Build in release mode
release:
	@echo "🚀 Building Rchat (release)..."
	cargo build --release --workspace

# Clean build artifacts
clean:
	@echo "🧹 Cleaning..."
	cargo clean
	rm -f server.crt server.key

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test --workspace

# Generate TLS certificates
certs:
	@echo "🔐 Generating TLS certificates..."
	@if [ ! -f server.crt ]; then \
		./generate_certs.sh; \
	else \
		echo "✅ Certificates already exist"; \
	fi

# Run server
server: certs build
	@echo "🚀 Starting server..."
	cargo run --bin server

# Run client (requires USERNAME variable)
client: certs build
	@if [ -z "$(USERNAME)" ]; then \
		echo "❌ Error: USERNAME not set"; \
		echo "Usage: make client USERNAME=Alice"; \
		exit 1; \
	fi
	@echo "🚀 Starting client as $(USERNAME)..."
	cargo run --bin client -- --username $(USERNAME)

# Check code without building
check:
	@echo "🔍 Checking code..."
	cargo check --workspace

# Format code
fmt:
	@echo "✨ Formatting code..."
	cargo fmt --all

# Lint code
clippy:
	@echo "📎 Running clippy..."
	cargo clippy --workspace -- -D warnings

# Security audit
audit:
	@echo "🔒 Running security audit..."
	@if command -v cargo-audit >/dev/null 2>&1; then \
		cargo audit; \
	else \
		echo "⚠️  cargo-audit not installed. Install with: cargo install cargo-audit"; \
	fi

# Help
help:
	@echo "Rchat - Makefile targets:"
	@echo ""
	@echo "  make build        - Build in debug mode"
	@echo "  make release      - Build in release mode"
	@echo "  make certs        - Generate TLS certificates"
	@echo "  make server       - Run the server"
	@echo "  make client       - Run the client (requires USERNAME=...)"
	@echo "  make test         - Run tests"
	@echo "  make check        - Check code without building"
	@echo "  make fmt          - Format code with rustfmt"
	@echo "  make clippy       - Lint code with clippy"
	@echo "  make audit        - Run security audit"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make help         - Show this help"
	@echo ""
	@echo "Examples:"
	@echo "  make server"
	@echo "  make client USERNAME=Alice"
	@echo "  make release && ./target/release/server"
