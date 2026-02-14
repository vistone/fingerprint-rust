# Architecture

**Version**: v2.1.0 (Workspace with Active/Passive Defense)
**Last Updated**: 2026-02-13

---

## 1. Overview

`fingerprint-rust` is a production-grade browser fingerprinting library built as a Cargo workspace.
It provides TLS and HTTP fingerprinting, a high-performance HTTP client, and optional defense and
analysis components.

## 2. Workspace Architecture

### 2.1 Crate Layout

The workspace contains 20 crates:

- fingerprint-core: Core types and utilities
- fingerprint-tls: TLS configuration, extensions, and handshake
- fingerprint-profiles: Browser fingerprint profiles
- fingerprint-headers: HTTP headers and User-Agent generation
- fingerprint-http: HTTP client (HTTP/1.1, HTTP/2, HTTP/3)
- fingerprint-dns: DNS pre-resolution (optional)
- fingerprint-defense: Passive identification and active defense (optional)
- fingerprint-api-noise: API noise generation and mitigation
- fingerprint-gateway: Rust API gateway (rate limiting, metrics)
- fingerprint: Main library, re-exports all functionality
- fingerprint-canvas: Canvas fingerprinting
- fingerprint-webgl: WebGL fingerprinting
- fingerprint-audio: Audio fingerprinting
- fingerprint-fonts: Fonts fingerprinting
- fingerprint-storage: Storage fingerprinting
- fingerprint-webrtc: WebRTC fingerprinting
- fingerprint-hardware: Hardware fingerprinting
- fingerprint-timing: Timing fingerprinting
- fingerprint-ml: ML-based fingerprinting
- fingerprint-anomaly: Anomaly detection

### 2.2 Core Dependencies (Workspace)

- rustls 0.23 (TLS implementation)
- tokio 1.40 (async runtime)
- h2 0.4 (HTTP/2)
- quinn 0.11 + h3 0.0.8 (HTTP/3)
- ring 0.17.14 (crypto)
- netconnpool-rust (connection pool)

## 3. Design Principles

- Modular crates with clear responsibilities
- Strict typing and explicit configuration
- Optional components behind features or separate crates
- Production-readiness with observability and testing

## 4. Related Documents

- Architecture (Chinese): docs/ARCHITECTURE.md
- Index (English): docs/INDEX.en.md
- Index (Chinese): docs/INDEX.md
