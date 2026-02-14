# Contributing to fingerprint-rust

**版本 (Version)**: v1.0  
**最后更新 (Last Updated)**: 2026-02-13  
**文档类型 (Document Type)**: 技术文档

---



Thank you for your interest in contributing to fingerprint-rust! This document provides guidelines and best practices for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [测试 Guidelines](#测试-guidelines)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Security](#security)

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of background or experience level.

### Expected Behavior

- Be respectful and considerate
- Welcome newcomers and help them get started
- Accept constructive criticism gracefully
- Focus on what is best for the project
- Show empathy towards other contributors

### Unacceptable Behavior

- Harassment, discrimination, or offensive comments
- Trolling or insulting remarks
- Public or private harassment
- Publishing others' private information
- Other conduct which could reasonably be considered inappropriate

## Getting Started

### Prerequisites

- **Rust**: 1.92.0 or later (use `rustup` for installation)
- **Git**: For 版本 (Version) control
- **Cargo**: Comes with Rust installation

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/fingerprint-rust.git
   cd fingerprint-rust
   ```

3. Add upstream remote:
   ```bash
   git remote add upstream https://github.com/vistone/fingerprint-rust.git
   ```

### Build the Project

```bash
# Build all workspace crates
cargo build --workspace

# Build with all features
cargo build --workspace --all-features

# Build specific crate
cargo build -p fingerprint-core
```

### Run Tests

```bash
# Run all tests
cargo test --workspace --lib

# Run tests with all features
cargo test --workspace --all-features

# Run specific test
cargo test -p fingerprint-core test_name
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

Branch naming conventions:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation updates
- `refactor/` - Code refactoring
- `test/` - Test additions or improvements
- `perf/` - Performance improvements

### 2. Make Changes

Follow the [Coding Standards](#coding-standards) section below.

### 3. Test Your Changes

```bash
# Run tests
cargo test --workspace --lib

# Run Clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all

# Check documentation
cargo doc --workspace --no-deps --all-features
```

### 4. Commit Changes

Write clear, descriptive commit messages:

```bash
git commit -m "feat: Add new browser fingerprint for Chrome 135"
git commit -m "fix: Resolve buffer overflow in packet parsing"
git commit -m "docs: Update API documentation for HTTP client"
```

Commit 消息 format:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `perf:` - Performance improvements
- `chore:` - Maintenance tasks

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a pull request on GitHub.

## Coding Standards

### General Principles

1. **Safety First**: Avoid `unsafe` code unless absolutely necessary
2. **Error Handling**: Use `Result` and `?` operator, avoid `unwrap()` in production code
3. **Documentation**: Document all public APIs
4. **测试**: Write tests for new functionality
5. **Performance**: Consider performance implications of changes

### Rust Style Guide

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/):

```rust
// ✅ Good: Proper error handling
pub fn parse_packet(data: &[u8]) -> Result<Packet, PacketError> {
    if data.len() < MIN_SIZE {
        return Err(PacketError::TooShort);
    }
    // ...
    Ok(packet)
}

// ❌ Bad: Using unwrap() in production
pub fn parse_packet(data: &[u8]) -> Packet {
    let value = data.get(0).unwrap(); // Can panic!
    // ...
}
```

### Error Handling

Use `thiserror` for error types:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Invalid packet size: {0}")]
    InvalidSize(usize),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### Documentation

Document all public APIs:

```rust
/// Parses an IPv4 packet from raw bytes.
///
/// # Arguments
///
/// * `data` - Raw packet data
///
/// # Returns
///
/// * `Ok(Packet)` - Successfully parsed packet
/// * `Err(PacketError)` - If packet is malformed
///
/// # Examples
///
/// ```
/// use fingerprint::parse_packet;
///
/// let data = vec![0x45, 0x00, /* ... */];
/// let packet = parse_packet(&data)?;
/// ```
///
/// # Errors
///
/// Returns `PacketError::TooShort` if packet is smaller than minimum size.
/// Returns `PacketError::InvalidIhl` if IHL field is invalid.
pub fn parse_packet(data: &[u8]) -> Result<Packet, PacketError> {
    // ...
}
```

### Code Organization

```rust
// 1. Module documentation at top
//! # Module Name
//!
//! Brief description of the module.

// 2. Imports
use std::io;
use crate::types::*;

// 3. Constants
const MAX_SIZE: usize = 1024;

// 4. Type definitions
pub struct MyStruct {
    // fields
}

// 5. Trait implementations
impl MyTrait for MyStruct {
    // ...
}

// 6. Methods
impl MyStruct {
    pub fn new() -> Self {
        // ...
    }
}

// 7. Tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_something() {
        // ...
    }
}
```

### 性能考虑

```rust
// ✅ Good: Avoid unnecessary allocations
pub fn process_data(data: &[u8]) -> Result<(), Error> {
    // Use references, not clones
}

// ❌ Bad: Unnecessary cloning
pub fn process_data(data: Vec<u8>) -> Result<(), Error> {
    let copied = data.clone(); // Avoid if possible
}

// ✅ Good: Reuse allocations
let mut buffer = Vec::with_capacity(1024);
for item in items {
    buffer.clear();
    // Reuse buffer
}

// ❌ Bad: Allocate in loop
for item in items {
    let buffer = Vec::new(); // Allocates every iteration
}
```

## 测试 Guidelines

### Unit Tests

Write unit tests for all public functions:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_packet() {
        let data = vec![/* valid packet */];
        let result = parse_packet(&data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_packet() {
        let data = vec![/* invalid packet */];
        let result = parse_packet(&data);
        assert!(result.is_err());
    }

    #[test]
    #[should_panic(expected = "buffer overflow")]
    fn test_panic_on_overflow() {
        // Test that panic occurs as expected
    }

    #[test]
    #[ignore] // Mark tests requiring network access
    fn test_network_operation() {
        // Test requiring network
    }
}
```

### Integration Tests

Place integration tests in `tests/` directory:

```rust
// tests/integration_test.rs
use fingerprint::*;

#[test]
fn test_end_to_end() {
    // Test complete workflow
}
```

### Property-Based Tests

Consider using `proptest` for property-based 测试:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parse_never_panics(data in prop::collection::vec(any::<u8>(), 0..1024)) {
        // Should never panic, even with random data
        let _ = parse_packet(&data);
    }
}
```

## Documentation

### Code Documentation

- Document all public APIs with `///` comments
- Include examples in documentation
- Explain errors and edge cases
- Use `//!` for 模块-level documentation

### User Documentation

- Update README.md for user-facing changes
- Add examples to `examples/` directory
- Update relevant guides in `docs/` directory
- Keep CHANGELOG.md up to date

### API Documentation

Generate and review documentation:

```bash
cargo doc --workspace --no-deps --all-features --open
```

## Pull Request Process

### Before Submitting

1. **Update your branch**:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run all checks**:
   ```bash
   cargo test --workspace --lib
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo fmt --all -- --check
   cargo doc --workspace --no-deps --all-features
   ```

3. **Update documentation** if needed

4. **Add tests** for new functionality

### PR Description Template

```markdown
## Description

Brief description of changes.

## Type of Change

- [ ] Bug fix (non-breaking change fixing an issue)
- [ ] New feature (non-breaking change adding functionality)
- [ ] Breaking change (fix or feature causing existing functionality to change)
- [ ] Documentation update

## Testing

- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] All tests pass locally

## Checklist

- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex code
- [ ] Documentation updated
- [ ] No new warnings introduced
- [ ] Tests added and passing

## Related Issues

Closes #123
Relates to #456
```

### Review Process

1. Maintainers will review your PR
2. Address feedback and make requested changes
3. Once approved, your PR will be merged

### After Merge

1. Delete your branch:
   ```bash
   git branch -d feature/your-feature-name
   git push origin --delete feature/your-feature-name
   ```

2. Update your main branch:
   ```bash
   git checkout main
   git pull upstream main
   ```

## Security

### Reporting Security Issues

**DO NOT** report security vulnerabilities through public issues.

Instead:
1. Use GitHub Security Advisories (preferred)
2. See [SECURITY.md](SECURITY.md) for details

### Security Considerations

When contributing, consider:
- Input validation
- Buffer overflow prevention
- Integer overflow handling
- Denial of service prevention
- Information disclosure risks

### Code Review Checklist

- [ ] No `unsafe` code without justification
- [ ] Proper error handling (no `unwrap()` in production)
- [ ] Input validation on external data
- [ ] Bounds checking on array access
- [ ] No integer overflow possibilities
- [ ] Proper resource cleanup (RAII)
- [ ] No information leaks in error messages

## 识别

Contributors will be:
- Listed in release notes
- Acknowledged in README.md (for significant contributions)
- Credited in commit history

## Questions?

- **GitHub Discussions**: For general questions
- **GitHub Issues**: For bug reports and feature requests
- **Pull Requests**: For code contributions

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/)

Thank you for contributing to fingerprint-rust! 🦀
