# Developer Guide

**Chinese** | [中文](/docs/zh/guides/DEVELOPMENT.md)

---

## 🚀 Development Environment Setup

### Prerequisites

- Rust 1.92.0 or higher
- Cargo
- Git

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Clone Repository

```bash
git clone https://github.com/vistone/fingerprint-rust.git
cd fingerprint-rust
```

### Install Development Tools

```bash
# Install formatter
rustup component add rustfmt

# Install linter
rustup component add clippy

# Install fast test tool
cargo install cargo-nextest

# Install coverage tool
cargo install cargo-tarpaulin
```

## 📝 Code Contribution Guidelines

### Naming Conventions

```rust
// ✅ Correct: snake_case functions
pub fn parse_fingerprint(data: &[u8]) -> Result<Fingerprint> { }

// ✅ Correct: PascalCase structs
pub struct FingerprintData {
    pub browser: BrowserType,
}

// ✅ Correct: UPPER_CASE constants
pub const MAX_RETRY_COUNT: usize = 3;

// ❌ Wrong: Mixed naming
pub fn ParseFingerprint() { }
pub struct fingerprint_data { }
```

### Documentation Comments

All public APIs must have documentation:

```rust
/// Parse browser fingerprint data
///
/// Extracts fingerprint information from raw byte data.
///
/// # Arguments
///
/// * `data` - Fingerprint data byte array
/// * `flags` - Parse option flags
///
/// # Returns
///
/// On success, returns parsed `Fingerprint` struct. On failure, returns `FingerprintError`.
///
/// # Errors
///
/// Returns `ParseError` if data format is incorrect or incomplete.
///
/// # Examples
///
/// ```
/// use fingerprint_core::{parse_fingerprint, FingerprintError};
///
/// let data = vec![0x01, 0x02, 0x03];
/// match parse_fingerprint(&data, 0) {
///     Ok(fp) => println!("Browser: {:?}", fp.browser),
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn parse_fingerprint(data: &[u8], flags: u8) -> Result<Fingerprint, FingerprintError> {
    // implementation
}
```

## 🧪 Testing

### Run All Tests

```bash
# Fast testing (recommended)
cargo nextest run --workspace

# Standard testing
cargo test --workspace

# Include doc tests
cargo test --workspace --doc

# Test specific crate
cd crates/fingerprint-core
cargo test
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_fingerprint() {
        let data = vec![/* ... */];
        let result = parse_fingerprint(&data, 0);
        
        assert!(result.is_ok());
        let fp = result.unwrap();
        assert_eq!(fp.browser, BrowserType::Chrome);
    }

    #[test]
    #[should_panic(expected = "invalid format")]
    fn test_parse_invalid_fingerprint() {
        let invalid_data = vec![];
        let _ = parse_fingerprint(&invalid_data, 0);
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = async_parse_fingerprint(&data).await;
        assert!(result.is_ok());
    }
}
```

### Benchmark Testing

```bash
# Run benchmarks
cargo bench --workspace

# Run specific benchmark
cargo bench --workspace fingerprint_parsing
```

## 📊 Code Quality Checks

### Format Checking

```bash
# Check format
cargo fmt --all -- --check

# Auto-format
cargo fmt --all
```

### Lint Checking

```bash
# Run clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Auto-fix issues
cargo clippy --workspace --fix
```

### Coverage Report

```bash
# Generate coverage
cargo tarpaulin --workspace --all-features --out Html --output-dir coverage

# View report
open coverage/index.html
```

### Security Audit

```bash
# Check dependency security
cargo audit

# Check dependency licenses
cargo deny check
```

## 🏗️ Project Structure

```
crates/
├── fingerprint/               # Main library (top-level API)
├── fingerprint-core/          # Core types and utilities
├── fingerprint-tls/           # TLS fingerprinting
├── fingerprint-http/          # HTTP client
├── fingerprint-profiles/      # Browser profiles
├── fingerprint-headers/       # Header generation
├── fingerprint-defense/       # Defense mechanisms
├── fingerprint-gateway/       # API gateway
├── fingerprint-ml/            # Machine learning
└── ...
```

### Adding New Features

1. **Create new module**: Add new file in existing crate
2. **Add tests**: Write tests for each new feature
3. **Update docs**: Add doc comments and README
4. **Run checks**: Ensure all tests and lint pass

Example:

```rust
// crates/fingerprint-core/src/new_feature.rs
//! New feature module

/// Main type for new feature
#[derive(Debug, Clone)]
pub struct NewFeature {
    // ...
}

impl NewFeature {
    /// Create new instance
    pub fn new() -> Self {
        Self { /* ... */ }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature() {
        let feature = NewFeature::new();
        // ...
    }
}
```

## 🔄 Workflow

### Before Committing Code

```bash
# 1. Update to latest code
git pull origin main

# 2. Create feature branch
git checkout -b feature/new-feature

# 3. Make changes and commit
git add .
git commit -m "feat: add new feature"

# 4. Run all checks
./scripts/pre_commit_test.sh

# 5. Push to remote
git push origin feature/new-feature
```

### Complete Checklist

Run before submitting:

```bash
# Format
cargo fmt --all

# Lint
cargo clippy --workspace --all-features -- -D warnings

# Compile
cargo check --workspace --all-features

# Test
cargo test --workspace --all-features

# Documentation
cargo doc --workspace --no-deps

# Security audit
cargo audit
```

## 📈 Performance Optimization

### Analyze Performance

```bash
# Run performance benchmarks
cargo bench --workspace

# Generate flame graphs
cargo install flamegraph
cargo flamegraph --bin fingerprint
```

### Common Optimizations

1. **Use immutable references**: Avoid unnecessary copies
2. **Cache common values**: Use LRU cache
3. **Async operations**: Use tokio for I/O
4. **SIMD optimization**: Use vectorized operations if available

## 🐛 Debugging

### Enable Logging

```rust
// In code
use log::{info, warn, error};

fn process_fingerprint(data: &[u8]) {
    info!("Starting fingerprint processing");
    warn!("Some warning");
    error!("Error occurred");
}
```

Run with logging:

```bash
RUST_LOG=debug cargo run
RUST_LOG=fingerprint_core=trace cargo test --lib
```

### Using Debugger

```bash
# Use rust-gdb
rust-gdb target/debug/fingerprint

# Use rust-lldb (macOS)
rust-lldb target/debug/fingerprint
```

## 📚 Related Resources

- [Rust Official Documentation](https://doc.rust-lang.org/)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Clippy Documentation](https://doc.rust-lang.org/clippy/)

## 🤝 Getting Help

- 📖 Check existing documentation
- 🐛 Check GitHub Issues
- 💬 Join Discussions
- 📧 Contact maintainers
