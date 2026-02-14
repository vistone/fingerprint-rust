# Architecture Design Document

**Version**: v2.1.0 (Workspace with Active/Passive Defense)  
**Last Updated**: 2026-02-13

---

## 📋 Table of Contents

1. [Project Overview](#project-overview)
2. [Workspace Architecture](#workspace-architecture)
3. [Crate Responsibilities](#crate-responsibilities)
4. [Dependencies](#dependencies)
5. [Design Principles](#design-principles)
6. [File Organization](#file-organization)
7. [Testing Strategy](#testing-strategy)
8. [Performance Considerations](#performance-considerations)
9. [Scalability](#scalability)

---

## 1. Project Overview

### 1.1 Project Positioning

`fingerprint-rust` is a **Production-ready** Browser Fingerprint Library using Cargo Workspace Architecture, providing:

- **97+ Browser Fingerprint Profiles**: Chrome, Firefox, Safari, Opera, Edge, and mainstream browsers plus mobile variants
- **Complete TLS Fingerprint Generation**: ClientHello Spec, Cipher Suites, Extensions, etc.
- **High-Performance HTTP Client**: Support HTTP/1.1, HTTP/2, HTTP/3 (QUIC)
- **Real-world Environment Verification**: Google Earth API end-to-end testing with 100% Pass Rate
- **Machine Learning Classification**: Three-layer hierarchical classifier architecture with 95%+ Accuracy
- **Passive Recognition Defense**: JA4+ full-stack fingerprint identification and threat detection

### 1.2 Technology Stack

- **Language**: Rust 1.92.0+
- **Architecture**: Cargo Workspace (20 independent crates)
- **TLS Implementation**: rustls 0.23 (optional), in-house TLS Handshake Builder
- **HTTP/2**: h2 0.4
- **HTTP/3**: quinn 0.11 + h3 0.0.8
- **Async Runtime**: tokio 1.40
- **Cryptographic Library**: ring 0.17.14 (real key generation)
- **Connection Pool**: netconnpool-rust (custom)
- **DNS Resolution**: hickory-resolver 0.24 (optional)
- **Machine Learning**: candle-core 0.8 (Rust ML framework)

---

## 2. Workspace Architecture

### 2.1 Directory Structure

```
fingerprint-rust/
├── Cargo.toml                    # Workspace root configuration
├── crates/                        # All crate code
│   ├── fingerprint-core/          # System-level protection core abstraction layer
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types.rs           # Core type definitions
│   │       ├── utils.rs           # Utility functions
│   │       └── traits.rs          # Core trait definitions
│   │
│   ├── fingerprint-tls/          # TLS configuration, extensions and handshake
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── tls_config/        # TLS ClientHello Spec
│   │       ├── tls_extensions.rs  # TLS extension implementation
│   │       └── tls_handshake/     # TLS handshake message construction
│   │
│   ├── fingerprint-profiles/     # Browser fingerprint configuration module
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── profiles.rs        # 97+ browser fingerprint configuration functions
│   │
│   ├── fingerprint-headers/      # HTTP Headers and User-Agent generation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── headers.rs         # HTTP request header generation
│   │       ├── useragent.rs       # User-Agent generation
│   │       └── http2_config.rs    # HTTP/2 configuration
│   │
│   ├── fingerprint-http/         # HTTP client implementation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── http_client/       # HTTP/1.1, HTTP/2, HTTP/3 support
│   │
│   ├── fingerprint-dns/          # DNS pre-resolution service
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── dns_resolver.rs    # DNS resolver implementation
│   │
│   ├── fingerprint-defense/      # System-level protection implementation layer
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── passive/           # Passive analyzer (TCP/HTTP/TLS/JA4+)
│   │       ├── consistency/       # Cross-layer consistency audit
│   │       ├── database/          # Fingerprint database (SQLite)
│   │       ├── learner/           # Self-learning mechanism
│   │       └── capture/           # Packet capture
│   │
│   ├── fingerprint-anomaly/      # Anomaly detection module
│   │   └── src/ - ML anomaly detection implementation
│   │
│   ├── fingerprint-canvas/       # Canvas Fingerprinting
│   ├── fingerprint-webgl/        # WebGL Fingerprinting
│   ├── fingerprint-audio/        # Audio Context Fingerprint
│   ├── fingerprint-fonts/        # Font enumeration detection
│   ├── fingerprint-webrtc/       # WebRTC IP leak detection
│   ├── fingerprint-hardware/     # Hardware capability detection
│   ├── fingerprint-timing/       # Timing attack protection
│   ├── fingerprint-storage/      # Storage fingerprint identification
│   ├── fingerprint-ml/           # Machine learning fingerprint matching
│   ├── fingerprint-api-noise/    # API noise injection
│   │
│   ├── fingerprint-gateway/      # High-performance API gateway
│   │
│   └── fingerprint/              # Independent browser TLS fingerprint library
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           └── fingerprint.rs     # Unified public API
│
├── examples/                      # Usage examples
├── tests/                         # Integration tests
├── docs/                          # Documentation
├── config/                        # Configuration files
└── output/                        # Output files
```

### 2.2 Workspace Configuration

Root `Cargo.toml` Configuration:

```toml
[workspace]
members = [
    "crates/fingerprint-core",
    "crates/fingerprint-tls",
    "crates/fingerprint-profiles",
    "crates/fingerprint-headers",
    "crates/fingerprint-http",
    "crates/fingerprint-dns",
    "crates/fingerprint-defense",
    "crates/fingerprint-api-noise",
    "crates/fingerprint-gateway",
    "crates/fingerprint",
    "crates/fingerprint-canvas",
    "crates/fingerprint-webgl",
    "crates/fingerprint-audio",
    "crates/fingerprint-fonts",
    "crates/fingerprint-storage",
    "crates/fingerprint-webrtc",
    "crates/fingerprint-hardware",
    "crates/fingerprint-timing",
    "crates/fingerprint-ml",
    "crates/fingerprint-anomaly",
]
resolver = "2"

[workspace.package]
version = "1.0.0"
edition = "2021"
# ... other common configuration

[workspace.dependencies]
# All dependencies defined here, sub-crates reference through workspace = true
rand = "0.8"
# ...
```

### 2.3 Why Use the `crates/` Directory?

Using the `crates/` directory is a **standard practice** for Rust Workspace Projects:

- ✅ **Community Convention**: Popular projects like tokio, serde, hyper all use `crates/`
- ✅ **Clear Semantics**: Directly indicates "a collection of multiple crates"
- ✅ **Clear Structure**: Visibly distinguishes from root directory files
- ✅ **Easy Extension**: Adding new crates doesn't clutter the root directory

---

## 3. Crate Responsibilities

### 3.1 fingerprint-core

**Responsibility**: Core types and utility functions  
**Code Location**: `crates/fingerprint-core/src/`  
**Included Modules**: types, utils, dicttls

### 3.2 fingerprint-tls

**Responsibility**: TLS Configuration, Extensions, and Handshake  
**Code Location**: `crates/fingerprint-tls/src/`  
**Included Modules**: tls_config, tls_extensions, tls_handshake

### 3.3 fingerprint-profiles

**Responsibility**: Browser fingerprint configuration management  
**Code Location**: `crates/fingerprint-profiles/src/`  
**Included Modules**: profiles.rs with 69+ browser fingerprint configurations

### 3.4 fingerprint-headers

**Responsibility**: HTTP Headers and User-Agent Generation  
**Code Location**: `crates/fingerprint-headers/src/`  
**Included Modules**: headers, useragent, http2_config

### 3.5 fingerprint-http

**Responsibility**: HTTP Client Implementation (HTTP/1.1, HTTP/2, HTTP/3)  
**Code Location**: `crates/fingerprint-http/src/http_client/`  
**Included Modules**: http1, http2, http3, pool management, response parsing

### 3.6 fingerprint-dns

**Responsibility**: DNS Pre-resolution Service (Optional Feature)  
**Code Location**: `crates/fingerprint-dns/src/dns/`  
**Included Modules**: service, resolver, server pool, collector, IP info

### 3.7 fingerprint-defense

**Responsibility**: Full-stack Passive Fingerprint Identification and Active Consistency Audit  
**Code Location**: `crates/fingerprint-defense/src/`  
**Included Modules**: 
- `passive/`: TCP, TLS, HTTP analysis
- `database.rs`: SQLite-based traffic persistence
- `learner.rs`: Self-learning mechanism
- `capture/`: Packet capture engine

### 3.8 Other Extension Crates

Supplement frontend and feature dimension fingerprinting capabilities:
- `fingerprint-api-noise`: API noise generation
- `fingerprint-gateway`: Rust API Gateway
- `fingerprint-canvas`: Canvas Fingerprinting
- `fingerprint-webgl`: WebGL Fingerprinting
- `fingerprint-audio`: Audio Fingerprinting
- `fingerprint-fonts`: Font fingerprinting
- `fingerprint-storage`: Storage Fingerprinting
- `fingerprint-webrtc`: WebRTC Fingerprinting
- `fingerprint-hardware`: Hardware Fingerprinting
- `fingerprint-timing`: Timing Fingerprinting
- `fingerprint-ml`: ML fingerprint analysis
- `fingerprint-anomaly`: Anomaly detection

### 3.9 fingerprint

**Responsibility**: Main library, re-exports all features  
**Code Location**: `crates/fingerprint/src/`  
**Functions**: Random fingerprint generation, configuration export

---

## 4. Dependencies

### 4.1 Dependency Graph

```
fingerprint (main library)
├── fingerprint-core
├── fingerprint-tls
├── fingerprint-profiles
├── fingerprint-headers
├── fingerprint-http
└── fingerprint-dns (optional)
└── fingerprint-defense (optional)
```

### 4.2 Dependency Management

- All dependencies defined in root `Cargo.toml` under `[workspace.dependencies]`
- Sub-crates reference via `dependency.workspace = true`

---

## 5. Design Principles

### 5.1 Single Responsibility
Each crate is responsible for only one clear functional domain

### 5.2 Clear Input and Output
Every function has clear input parameters and return values

### 5.3 Avoid Unnecessary Nesting and Coupling
Crates interact through public interfaces using traits and enumerations

### 5.4 Thread Safety
All public APIs are thread-safe using appropriate synchronization primitives

### 5.5 Performance Optimization
- Use HashMap for fast lookups
- Avoid unnecessary cloning
- Support parallel compilation

---

## 6. File Organization

### 6.1 Source Code Organization

```
crates/
├── fingerprint-core/src/
├── fingerprint-tls/src/
├── fingerprint-profiles/src/
├── fingerprint-headers/src/
├── fingerprint-http/src/
├── fingerprint-dns/src/
└── fingerprint/src/
```

### 6.2 Test Organization

```
tests/
├── integration_test.rs
├── http_client_test.rs
├── dns_service_test.rs
└── ...
```

### 6.3 Example Organization

```
examples/
├── basic.rs
├── custom_tls_fingerprint.rs
├── http2_with_pool.rs
├── http3_with_pool.rs
└── dns_service.rs
```

---

## 7. Testing Strategy

### 7.1 Unit Tests
Each crate includes unit tests covering core functionality

### 7.2 Integration Tests
Comprehensive tests in `tests/` directory covering all public APIs

### 7.3 Test Coverage
- ✅ Random fingerprint retrieval
- ✅ Fingerprint retrieval by browser type
- ✅ User-Agent generation
- ✅ HTTP Headers generation
- ✅ TLS fingerprint generation
- ✅ HTTP/1.1, HTTP/2, HTTP/3 clients
- ✅ Connection pool functionality
- ✅ DNS service
- ✅ Concurrent access safety
- ✅ Error handling

### 7.4 Test Results
- **Total Tests**: 74
- **Passed**: 74
- **Failed**: 0
- **Success Rate**: 100%

---

## 8. Performance Considerations

### 8.1 Compilation Performance
- **Parallel Compilation**: Workspace supports parallel compilation of multiple crates
- **Incremental Compilation**: Only recompile modified crates
- **Projected Improvement**: 30-50% compilation speed improvement

### 8.2 Runtime Performance
- **Zero-allocation Operations**: Critical path avoids unnecessary memory allocation
- **Fast Lookups**: Use HashMap for O(1) lookups
- **Thread Safety**: Use thread-local random number generators
- **Lazy Initialization**: Use `OnceLock` for lazy initialization

### 8.3 HTTP Client Performance

| Protocol | Average Response Time | Min | Max | Success Rate |
|----------|----------------------|-----|-----|--------------|
| **HTTP/3** | 40.3ms | 35ms | 48ms | 100% 🥇 |
| **HTTP/1.1** | 44.4ms | 37ms | 79ms | 100% 🥈 |
| **HTTP/2** | 48.0ms | 43ms | 60ms | 100% 🥉 |

---

## 9. Scalability

The project design supports the following extensions:

### 9.1 Add New Browser Fingerprint
Add function in `crates/fingerprint-profiles/src/profiles.rs`

### 9.2 Add New User-Agent Template
Update `crates/fingerprint-headers/src/useragent.rs`

### 9.3 Add New Language
Add to `LANGUAGES` array in `crates/fingerprint-headers/src/headers.rs`

### 9.4 Add New Operating System
Add to `OperatingSystem` enumeration in `crates/fingerprint-core/src/types.rs`

### 9.5 Add New Crate
1. Create new crate under `crates/` directory
2. Add member in `[workspace]` in root `Cargo.toml`
3. Configure dependency relationships

---

## 10. Build and Testing

### 10.1 Build All Crates

```bash
# Build entire workspace
cargo build --workspace

# Build specific crate
cargo build -p fingerprint-core
cargo build -p fingerprint-http --features "rustls-tls,http2"
```

### 10.2 Run Tests

```bash
# Test entire workspace
cargo test --workspace

# Test specific crate
cargo test -p fingerprint-core
```

### 10.3 Check Compilation

```bash
# Check entire workspace
cargo check --workspace
```

---

**Documentation Version**: v2.1.0  
**Last Updated**: 2026-02-13
