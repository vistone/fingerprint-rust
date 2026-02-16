# Contributing to Fingerprint-Rust

Thank you for your interest in contributing to fingerprint-rust! This document provides guidelines and requirements for contributions.

## 🔒 Mandatory Requirements

### All contributions MUST satisfy these requirements before merge:

1. **✅ All Tests Pass** - No exceptions
2. **✅ Code Formatting** - Must pass `cargo fmt --all -- --check`
3. **✅ Clippy Clean** - Must pass `cargo clippy -- -D warnings`
4. **✅ Builds Successfully** - On Linux, macOS, and Windows
5. **✅ Security Audit** - No known vulnerabilities

## 📋 Before You Start

### Setup Your Development Environment

1. Install Rust (latest stable):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Install development tools:
   ```bash
   rustup component add rustfmt clippy
   cargo install cargo-nextest  # For faster testing
   ```

3. Install pre-commit hook (recommended):
   ```bash
   cp .github/pre-commit-hook.sh .git/hooks/pre-commit
   chmod +x .git/hooks/pre-commit
   ```

## 🧪 Testing Requirements

### Run Tests Locally

Before submitting a PR, ensure all tests pass:

```bash
# Run all tests
cargo nextest run --workspace --all-features

# Or use standard cargo test
cargo test --workspace --all-features

# Run tests for specific crate
cd crates/fingerprint-ai-models
cargo nextest run --all-features
```

### Test Coverage

- All new features must include tests
- Bug fixes should include regression tests
- Aim for >80% code coverage on new code

### AI Models Testing

If working on the AI models crate:

```bash
cd crates/fingerprint-ai-models

# Run all tests
cargo nextest run --all-features

# Test specific examples
cargo run --example unified_ai_detector -- test.txt
cargo run --example analyze_real_image -- image.jpg
cargo run --example learn_model_fingerprints
cargo run --example train_characteristic_library
```

## 📝 Code Style

### Formatting

All code must be formatted with rustfmt:

```bash
# Check formatting
cargo fmt --all -- --check

# Auto-format code
cargo fmt --all
```

### Linting

All code must pass clippy with zero warnings:

```bash
# Check with clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Fix clippy suggestions
cargo clippy --workspace --all-targets --all-features --fix
```

## 🔨 Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 2. Make Changes

- Write clear, concise commit messages
- Keep changes focused and atomic
- Add tests for new functionality
- Update documentation as needed

### 3. Test Locally

```bash
# Run pre-commit checks
.git/hooks/pre-commit

# Or manually:
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
```

### 4. Commit and Push

```bash
git add .
git commit -m "feat: add new feature"
git push origin feature/your-feature-name
```

### 5. Create Pull Request

- Fill out the PR template completely
- Link related issues
- Ensure all CI checks pass
- Request review from maintainers

## 🚫 What Will Block Your PR

Your PR will be automatically blocked if:

- ❌ Any test fails
- ❌ Code is not formatted (`cargo fmt`)
- ❌ Clippy warnings exist
- ❌ Build fails on any platform
- ❌ Security vulnerabilities detected

## 🤖 CI/CD Pipeline

Our GitHub Actions workflows automatically check:

1. **Required Checks** (`required-checks.yml`)
   - All tests pass
   - Code formatting
   - Clippy warnings
   - Multi-platform builds
   - Security audit

2. **Comprehensive Testing** (`comprehensive-testing.yml`)
   - Unit tests
   - Integration tests
   - Example tests
   - Doc tests

3. **AI Models Validation** (`ai-models-validation.yml`)
   - AI-specific tests
   - Fingerprint database validation
   - Detection accuracy tests

4. **Coverage** (`coverage.yml`)
   - Code coverage reporting
   - Coverage trends

### Required Status Checks

These checks MUST pass before merge:
- ✅ All Required Checks Passed
- ✅ All Tests (Required)
- ✅ Format Check (Required)
- ✅ Clippy Check (Required)
- ✅ Build Check (Required) - all platforms
- ✅ Security Audit (Required)

## 📚 Documentation

### Code Documentation

- Use `///` for item documentation
- Use `//!` for module documentation
- Include examples in doc comments
- Keep documentation up-to-date

### Example:

```rust
/// Detects if text content is AI-generated
///
/// # Arguments
///
/// * `text` - The text to analyze
///
/// # Returns
///
/// * `AIDetectionResult` - Detection results with confidence score
///
/// # Example
///
/// ```
/// use fingerprint_ai_models::detect_ai_content;
///
/// let result = detect_ai_content("Sample text...");
/// println!("AI-generated: {}", result.is_ai_generated);
/// ```
pub fn detect_ai_content(text: &str) -> AIDetectionResult {
    // Implementation
}
```

## 🐛 Reporting Issues

When reporting issues:

1. Search existing issues first
2. Use issue templates
3. Provide minimal reproduction example
4. Include environment details
5. Add relevant logs/errors

## 💡 Feature Requests

For new features:

1. Open an issue first to discuss
2. Wait for maintainer feedback
3. Implement after approval
4. Follow contribution guidelines

## 🔄 Pull Request Process

1. **Create PR** with clear description
2. **All CI checks** must pass (automatically enforced)
3. **Code review** by maintainers
4. **Address feedback** if requested
5. **Merge** after approval

## ❓ Questions?

- Open a discussion in GitHub Discussions
- Check existing documentation
- Review examples in `examples/` directory
- Read workflow files in `.github/workflows/`

## 📜 License

By contributing, you agree that your contributions will be licensed under the BSD-3-Clause License.

---

Thank you for contributing to fingerprint-rust! 🎉
