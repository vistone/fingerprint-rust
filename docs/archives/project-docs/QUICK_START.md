# 🚀 Fingerprint Rust - Quick Start Guide

**版本**: v1.0  
**最后更新**: 2026-02-13  
**文档类型**: 技术文档

---

**Status**: ✅ Production Ready (Core Modules Complete, 95% Overall)

## 📋 Quick Links

| Need | File | Purpose |
|------|------|---------|
| **Getting Started** | [docs/user-guides/getting-started.md](docs/user-guides/getting-started.md) | Basic usage and setup |
| **Architecture** | [docs/developer-guides/architecture.md](docs/developer-guides/architecture.md) | System design overview |
| **API Usage** | [docs/user-guides/api-usage.md](docs/user-guides/api-usage.md) | REST API endpoints |
| **Fingerprint Guide** | [docs/user-guides/fingerprint-guide.md](docs/user-guides/fingerprint-guide.md) | Browser fingerprint configuration |
| **Deployment** | [docs/reference/deployment-manual.md](docs/reference/deployment-manual.md) | Deployment instructions |
| **Troubleshooting** | [docs/guides/TROUBLESHOOTING_GUIDE.md](docs/guides/TROUBLESHOOTING_GUIDE.md) | Issue resolution |

---

## 🎯 5-Minute Setup

```bash
# 1. Clone the repository
git clone https://github.com/vistone/fingerprint-rust.git
cd fingerprint-rust

# 2. Install dependencies
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version  # Should be 1.92.0+

# 3. Build the project
cargo build --workspace --features "rustls-tls,http2,http3"

# 4. Run basic example
cargo run --example basic

# 5. Run tests
cargo test --workspace
```

---

## 📊 What You Get

### Core Capabilities
- ✅ **66+ Real Browser Fingerprints**: Chrome, Firefox, Safari, Opera, Edge
- ✅ **Multi-Protocol Support**: HTTP/1.1, HTTP/2, HTTP/3 (QUIC)
- ✅ **Advanced TLS Fingerprinting**: JA3, JA4+ generation and matching
- ✅ **High Performance**: Zero-allocation on critical paths
- ✅ **Production Ready**: Enterprise-grade reliability

### Module Structure
```
crates/
├── fingerprint-core/      # Core types and utilities
├── fingerprint-tls/       # TLS configuration and handshake
├── fingerprint-http/      # HTTP client implementation
├── fingerprint-profiles/  # Browser fingerprint profiles
├── fingerprint-gateway/   # High-performance API gateway
└── fingerprint-defense/   # Passive detection and protection
```

---

## 🚀 Basic Usage

### Simple HTTP Request
```rust
use fingerprint_core::{FingerprintClient, Profile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with Chrome 120 fingerprint
    let client = FingerprintClient::builder()
        .with_profile(Profile::Chrome120)
        .build()?;

    // Send request
    let response = client.get("https://httpbin.org/headers").await?;
    println!("Status: {}", response.status());
    
    Ok(())
}
```

### Advanced Configuration
```rust
use fingerprint_core::FingerprintClient;
use std::time::Duration;

let client = FingerprintClient::builder()
    .with_timeout(Duration::from_secs(30))
    .with_connection_pool(50)
    .enable_http2(true)
    .enable_http3(true)
    .build()?;
```

---

## 🧪 Examples

Run built-in examples:
```bash
# Basic usage
cargo run --example basic

# TLS fingerprinting
cargo run --example tls_client

# HTTP/2 client
cargo run --example http2_client

# Browser fingerprint pool
cargo run --example fingerprint_pool
```

---

## 📚 Next Steps

1. **Read the Docs**: Start with [Getting Started Guide](docs/user-guides/getting-started.md)
2. **Explore Examples**: Check the `examples/` directory
3. **Run Tests**: `cargo test --workspace`
4. **Check Performance**: `cargo bench`
5. **Deploy**: Follow [Deployment Guide](docs/reference/deployment-manual.md)

---

## 🆘 Need Help?

- **Documentation**: [docs/](docs/)
- **Issues**: [GitHub Issues](https://github.com/vistone/fingerprint-rust/issues)
- **Discussions**: [GitHub Discussions](https://github.com/vistone/fingerprint-rust/discussions)
- **Email**: support@fingerprint-rust.org

---
*Last updated: 2026-02-13*
