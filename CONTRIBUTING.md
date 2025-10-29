# Contributing to Rchat

Thank you for your interest in contributing to Rchat! 🎉

## 🚀 Quick Start

1. **Fork** the repository
2. **Clone** your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/Rchat.git
   cd Rchat
   ```
3. **Build** the project:
   ```bash
   make build
   ```

## 🔧 Development Setup

### Prerequisites

- Rust 1.75+ (2021 edition)
- OpenSSL (for certificate generation)
- Make (optional, but recommended)

### Building

```bash
# Debug build
cargo build --workspace

# Release build
cargo build --release --workspace

# Or using Make
make build
make release
```

### Running

```bash
# Generate certificates (first time only)
./generate_certs.sh

# Terminal 1: Start server
cargo run --bin server

# Terminal 2: Start client
cargo run --bin client -- --username Alice
```

## 📝 Code Style

We follow standard Rust conventions:

```bash
# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace -- -D warnings

# Or using Make
make fmt
make clippy
```

## 🧪 Testing

```bash
# Run all tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Or using Make
make test
```

## 🔒 Security Considerations

When contributing, keep in mind:

1. **Cryptographic code**: Changes to `common/src/crypto.rs` require extra scrutiny
2. **Memory safety**: Use `zeroize` for sensitive data
3. **No logging**: Never log message content or keys
4. **TLS**: All network communication must use TLS

### Security Checklist

- [ ] No sensitive data in logs
- [ ] Proper use of `zeroize` for secrets
- [ ] No storage to disk
- [ ] TLS for all connections
- [ ] Proper error handling (no panics in production code)

## 📂 Project Structure

```
Rchat/
├── common/         # Shared library (protocol, crypto)
│   └── src/
│       ├── protocol.rs  # Message definitions
│       └── crypto.rs    # E2EE implementation
├── server/         # Server binary
│   └── src/
│       ├── main.rs      # Server entry point
│       └── chat.rs      # Chat room management
├── client/         # Client binary
│   └── src/
│       ├── main.rs      # Client entry point
│       └── ui.rs        # TUI implementation
└── docs/           # Documentation
```

## 🐛 Reporting Bugs

When reporting bugs, please include:

1. **Description**: Clear description of the issue
2. **Steps to Reproduce**: Detailed steps
3. **Expected Behavior**: What should happen
4. **Actual Behavior**: What actually happens
5. **Environment**:
   - OS and version
   - Rust version (`rustc --version`)
   - Rchat version

## ✨ Feature Requests

We welcome feature requests! Please:

1. **Check existing issues** to avoid duplicates
2. **Describe the use case** clearly
3. **Consider security implications**
4. **Be realistic** about scope

### High Priority Features

- [ ] Better error handling and recovery
- [ ] Message history buffer (in-memory only)
- [ ] User list display in TUI
- [ ] Configurable max participants
- [ ] Server health checks

### Out of Scope (for this PoC)

- ❌ Persistent storage
- ❌ User registration/authentication
- ❌ File sharing
- ❌ Voice/video chat
- ❌ Message editing/deletion

## 📋 Pull Request Process

1. **Create a branch** for your feature:
   ```bash
   git checkout -b feature/my-awesome-feature
   ```

2. **Make your changes** with clear, atomic commits:
   ```bash
   git commit -m "Add: Feature X for Y reason"
   ```

3. **Test thoroughly**:
   ```bash
   make test
   make clippy
   make fmt
   ```

4. **Update documentation** if needed

5. **Submit PR** with:
   - Clear title and description
   - Reference any related issues
   - Screenshots/examples if UI changes

### Commit Message Format

We use conventional commits:

```
<type>: <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Adding tests
- `chore`: Maintenance tasks

Examples:
```
feat: Add message history buffer
fix: Handle disconnection gracefully
docs: Update README with new features
refactor: Simplify crypto key derivation
```

## 🎨 UI/UX Guidelines

When modifying the TUI:

1. **Keep it minimal**: ASCII art should be simple
2. **Clear feedback**: User should always know what's happening
3. **Error messages**: Helpful, not cryptic
4. **Keyboard shortcuts**: Document in UI
5. **Accessibility**: Consider color-blind users

## 🔐 Cryptography Guidelines

**IMPORTANT**: Cryptographic code requires extra care!

1. **Don't roll your own crypto**: Use established crates
2. **Constant-time operations**: Avoid timing attacks
3. **Proper randomness**: Use `OsRng`, never `rand::thread_rng()` for keys
4. **Key zeroization**: Always use `zeroize` for keys
5. **Test vectors**: Include test cases from RFCs

### Reviewing Crypto PRs

Crypto PRs require:
- [ ] Explanation of why the change is needed
- [ ] Reference to RFC/paper if applicable
- [ ] Test coverage
- [ ] Review by maintainer with crypto knowledge

## 📚 Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Ratatui Documentation](https://ratatui.rs/)
- [RustCrypto](https://github.com/RustCrypto)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)

## 🤝 Code of Conduct

Be respectful, constructive, and professional.

## 📄 License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing! 🦀🔒
