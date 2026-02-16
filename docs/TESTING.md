# Testing Requirements

## Overview

This document outlines the comprehensive testing requirements for fingerprint-rust. **All tests must pass before code can be merged.**

## 🔒 Mandatory Testing Policy

### Non-Negotiable Requirements

1. **All tests must pass** - Zero tolerance for failing tests
2. **No test can be skipped** - Unless marked with `#[ignore]` for good reason
3. **New code requires tests** - No untested code merged
4. **Coverage maintained** - No significant coverage drops

## 📊 Test Categories

### 1. Unit Tests

Test individual functions and modules in isolation.

```bash
# Run all unit tests
cargo nextest run --workspace --lib

# Run tests for specific crate
cd crates/fingerprint-ai-models
cargo nextest run --lib
```

**Requirements:**
- Every public function should have unit tests
- Edge cases must be covered
- Error conditions must be tested

### 2. Integration Tests

Test how components work together.

```bash
# Run integration tests
cargo nextest run --workspace --test "*"
```

**Requirements:**
- Test cross-crate functionality
- Test with real data when possible
- Test failure scenarios

### 3. Doc Tests

Tests embedded in documentation.

```bash
# Run doc tests
cargo test --workspace --doc
```

**Requirements:**
- All code examples in docs must work
- Examples should be realistic
- Cover common use cases

### 4. Example Tests

Validate example programs work correctly.

```bash
# Test all examples
cargo nextest run --workspace --examples

# Test specific example
cargo run --example unified_ai_detector -- test.txt
```

**Requirements:**
- All examples must compile
- Examples should run without errors
- Examples should demonstrate real usage

## 🎯 Test Coverage Requirements

### Minimum Coverage Targets

- **Overall**: >70% line coverage
- **New Code**: >80% line coverage
- **Critical Paths**: 100% coverage
- **AI Models Crate**: >85% coverage

### Check Coverage

```bash
# Generate coverage report
cargo llvm-cov --workspace --all-features

# View HTML report
cargo llvm-cov --workspace --all-features --html
open target/llvm-cov/html/index.html
```

## 🧪 AI Models Testing

Special requirements for `fingerprint-ai-models` crate:

### Required Tests

1. **Content Detection Tests**
   ```bash
   cargo test --package fingerprint-ai-models content_detection
   ```

2. **Provider Detection Tests**
   ```bash
   cargo test --package fingerprint-ai-models providers
   ```

3. **Real File Analysis Tests**
   ```bash
   cargo test --package fingerprint-ai-models real_detection
   cargo test --package fingerprint-ai-models real_video_detection
   ```

4. **Fingerprint Learning Tests**
   ```bash
   cargo test --package fingerprint-ai-models model_fingerprints
   cargo test --package fingerprint-ai-models characteristic_library
   ```

5. **Advanced Detection Tests**
   ```bash
   cargo test --package fingerprint-ai-models advanced_detection
   ```

### Example Validation

All 8 AI models examples must work:

```bash
cd crates/fingerprint-ai-models

# 1. Content detection
cargo run --example detect_ai_content

# 2. Provider detection
cargo run --example detect_ai_providers

# 3. Global providers
cargo run --example detect_global_providers

# 4. Image analysis
cargo run --example analyze_real_image

# 5. Video analysis
cargo run --example analyze_short_video

# 6. Unified detector
cargo run --example unified_ai_detector

# 7. Learn fingerprints
cargo run --example learn_model_fingerprints

# 8. Train library
cargo run --example train_characteristic_library
```

## 🚀 Running Tests

### Quick Test (Recommended)

```bash
# Fast testing with nextest
cargo nextest run --workspace --all-features
```

### Complete Test Suite

```bash
# All tests with cargo test
cargo test --workspace --all-features

# Include ignored tests (network-dependent)
cargo test --workspace --all-features -- --include-ignored
```

### Platform-Specific Tests

```bash
# Linux
cargo test --workspace --all-features --target x86_64-unknown-linux-gnu

# macOS
cargo test --workspace --all-features --target x86_64-apple-darwin

# Windows
cargo test --workspace --all-features --target x86_64-pc-windows-msvc
```

## 🔍 Test Organization

### File Structure

```
crates/
└── fingerprint-ai-models/
    ├── src/
    │   ├── lib.rs                 # Doc tests here
    │   ├── content_detection.rs   # Unit tests at bottom
    │   ├── providers.rs           # Unit tests at bottom
    │   └── ...
    ├── tests/
    │   ├── integration_test.rs    # Integration tests
    │   └── ...
    └── examples/
        ├── detect_ai_content.rs   # Example programs
        └── ...
```

### Test Naming

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_functionality() {
        // Test basic case
    }
    
    #[test]
    fn test_edge_case_empty_input() {
        // Test edge case
    }
    
    #[test]
    #[should_panic(expected = "error message")]
    fn test_panic_condition() {
        // Test panic
    }
    
    #[test]
    #[ignore]  // Only for network/slow tests
    fn test_network_dependent() {
        // Test requiring network
    }
}
```

## 🎨 Writing Good Tests

### Test Principles

1. **Arrange-Act-Assert** pattern
   ```rust
   #[test]
   fn test_example() {
       // Arrange
       let input = "test data";
       
       // Act
       let result = function_under_test(input);
       
       // Assert
       assert_eq!(result, expected);
   }
   ```

2. **One concept per test**
   - Test one thing at a time
   - Clear test names
   - Focused assertions

3. **Independent tests**
   - No test dependencies
   - No shared state
   - Repeatable results

4. **Fast tests**
   - Avoid slow operations
   - Use mocks for I/O
   - Mark slow tests with `#[ignore]`

### Test Quality

✅ **Good Test:**
```rust
#[test]
fn test_detect_gpt4_with_characteristic_phrases() {
    let text = "Let's delve into the intricate details...";
    let result = detect_ai_content(text);
    
    assert!(result.is_ai_generated);
    assert!(result.confidence > 0.7);
    assert!(result.model_probabilities.contains_key("gpt4"));
}
```

❌ **Bad Test:**
```rust
#[test]
fn test_stuff() {  // Vague name
    let x = do_thing();  // Unclear what's being tested
    assert!(x);  // What does this verify?
}
```

## 🚦 CI/CD Integration

### Automated Testing

Tests run automatically on:
- Every push to main/develop
- Every pull request
- Daily scheduled runs
- Before releases

### Required Checks

These checks must pass:
1. ✅ All unit tests
2. ✅ All integration tests
3. ✅ All doc tests
4. ✅ All example tests
5. ✅ All AI models tests
6. ✅ Platform-specific tests

### Status Check Names

In GitHub Actions:
- `All Tests (Required)`
- `AI Models Validation`
- `Comprehensive Testing`
- `Coverage Report`

## 📈 Test Metrics

### Track These Metrics

1. **Test Count**
   ```bash
   cargo test --workspace -- --list | wc -l
   ```

2. **Test Success Rate**
   - Should always be 100%

3. **Test Duration**
   ```bash
   cargo nextest run --workspace --all-features --timings
   ```

4. **Coverage Percentage**
   ```bash
   cargo llvm-cov --workspace --summary-only
   ```

## 🔧 Test Tools

### Installed Tools

```bash
# Fast test runner
cargo install cargo-nextest

# Coverage tool
cargo install cargo-llvm-cov

# Watch tests
cargo install cargo-watch

# Mutation testing (optional)
cargo install cargo-mutants
```

### Useful Commands

```bash
# Watch and rerun tests
cargo watch -x test

# Test with output
cargo test -- --nocapture

# Test single function
cargo test test_function_name

# List all tests
cargo test -- --list
```

## ❌ Common Test Failures

### 1. Flaky Tests

**Problem**: Test passes sometimes, fails others

**Solution**:
- Remove timing dependencies
- Avoid shared state
- Use deterministic data
- Fix race conditions

### 2. Platform-Specific Failures

**Problem**: Test fails on Windows but passes on Linux

**Solution**:
- Use platform-agnostic code
- Add conditional compilation
- Test on all platforms locally

### 3. Network-Dependent Tests

**Problem**: Test requires internet connection

**Solution**:
- Mark with `#[ignore]` attribute
- Use mocks for external services
- Document in test docstring

## 📚 Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-nextest Documentation](https://nexte.st/)
- [Coverage with cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov)

## ✅ Checklist for Contributors

Before submitting PR:

- [ ] All new code has tests
- [ ] All tests pass locally
- [ ] Coverage hasn't decreased
- [ ] Tests follow naming conventions
- [ ] No flaky tests introduced
- [ ] Examples work correctly
- [ ] Doc tests pass
- [ ] CI checks will pass

---

**Remember**: No code merges without passing tests! 🧪
