# FAQ (Frequently Asked Questions)

**Chinese** | [中文](/docs/zh/FAQ.md)

---

## General Questions

### Q: What is browser fingerprinting?
**A:** Browser fingerprinting is a technique to identify and track users by collecting various characteristics of their browser and system. These characteristics include:
- User-Agent string
- TLS configuration and cipher suites
- HTTP request header order
- Timing information
- Canvas/WebGL output
- Font list
- Audio context information

### Q: Why would I use fingerprint-rust?
**A:** Common use cases include:
- ✅ Web scraping with realistic browser profiles
- ✅ API testing and integration tests
- ✅ Enhanced privacy protection
- ✅ Academic research
- ✅ Security defense and detection

## Usage Questions

### Q: How do I use fingerprint-rust in my project?
**A:** 
1. Add dependency to Cargo.toml:
```toml
[dependencies]
fingerprint = "2.1"
```
2. Use the simple API:
```rust
use fingerprint::get_random_fingerprint;
let fp = get_random_fingerprint()?;
```

### Q: What browsers are supported?
**A:** Supports 97+ browser fingerprints:
- **Chrome**: v90-133 (all major versions)
- **Firefox**: v88-121
- **Safari**: v14-16
- **Edge**: v90-120
- **Opera**: v76-108
- **Mobile**: Chrome/Safari iOS, Chrome Android, etc.

### Q: How realistic are the generated fingerprints?
**A:** 
- ✅ Based on real browser data (not generated)
- ✅ Includes real TLS ClientHello Specs
- ✅ 97%+ accuracy matching real browsers
- ✅ Passed Google Earth API end-to-end testing

### Q: What are valid use cases?
**A:** Legal uses include:
- ✅ Web scraping (respecting robots.txt)
- ✅ API testing and integration testing
- ✅ Privacy enhancement
- ✅ Academic research
- ✅ Security defense and detection

## Performance Questions

### Q: How fast is fingerprint recognition?
**A:** 
```
Recognition time: ~0.5ms
HTTP request latency: <100ms (with network)
Memory usage: ~5-10MB (full cache)
Throughput: >10,000 req/sec (single thread)
```

### Q: How do I optimize performance?
**A:** 
1. **Use connection pooling**:
```rust
let config = PoolManagerConfig::default()
    .with_max_idle_per_host(32)
    .with_connection_timeout_secs(5);
```

2. **Enable caching**:
```rust
// Auto-enables L1/L2/L3 cache
let fp = get_random_fingerprint()?;
```

3. **Batch processing**:
```rust
let fingerprints: Vec<_> = (0..100)
    .map(|_| get_random_fingerprint())
    .collect::<Result<_, _>>()?;
```

### Q: Does it support HTTP/3?
**A:** Yes! Full support:
- ✅ HTTP/1.1
- ✅ HTTP/2 (h2 protocol)
- ✅ HTTP/3 (QUIC protocol)
Auto-negotiates the best version.

## Security & Privacy Questions

### Q: Is fingerprint spoofing illegal?
**A:** It depends on your use case and local law. Generally:
- ✅ Legal: Personal privacy protection, testing your own systems
- ⚠️ Gray area: Scraping, accessing restricted resources
- ❌ Illegal: Fraud, deception, privacy violations

**Recommendation**: Always comply with terms of service and local laws.

### Q: Does fingeprinter leak privacy?
**A:** No! fingerprint-rust is locally computed:
- 🔒 No data uploaded to servers
- 🔒 No request tracking
- 🔒 Processed entirely locally
- 🔒 Open source code is auditable

### Q: How do I avoid detection?
**A:** 
1. **Use real fingerprints**:
```rust
let fp = get_random_fingerprint()?; // Not generated
```

2. **Add random delays**:
```rust
use std::time::Duration;
tokio::time::sleep(Duration::from_secs(rand::random::<u64>() % 5)).await;
```

3. **Rotate fingerprints**:
```rust
loop {
    let fp = get_random_fingerprint()?; // New fingerprint each time
    // Send request...
}
```

4. **Handle CAPTCHAs**: Most websites use CAPTCHAs as last defense.

## Development Questions

### Q: How do I contribute code?
**A:** 
1. Fork the repository
2. Create feature branch: `git checkout -b feature/xxx`
3. Make changes and pass all checks:
```bash
cargo fmt --all
cargo clippy --all-features
cargo test --all-features
```
4. Submit PR and wait for review

### Q: What is the minimum Rust version?
**A:** Rust 1.92.0 or higher.

### Q: Can I use it in production?
**A:** Yes! Project features:
- ✅ Production-grade code quality
- ✅ Extensive test coverage
- ✅ Complete error handling
- ✅ Performance optimizations

### Q: How do I report bugs?
**A:** 
1. Check existing issues
2. Create new issue with:
   - Rust version
   - Operating system
   - Minimal reproduction code
   - Error message and stack trace

### Q: Does it support WebAssembly?
**A:** Partial support:
- ✅ Can compile to wasm
- ⚠️ Some features unavailable in browser
- 📋 Full wasm support planned

## Integration Questions

### Q: How do I integrate with web frameworks?

**Actix-web example:**
```rust
use actix_web::{web, HttpResponse};
use fingerprint::get_random_fingerprint;

async fn get_fingerprint() -> HttpResponse {
    match get_random_fingerprint() {
        Ok(fp) => HttpResponse::Ok().json(serde_json::json!(fp)),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
```

**Tokio example:**
```rust
#[tokio::main]
async fn main() {
    let fp = get_random_fingerprint().unwrap();
    println!("User-Agent: {}", fp.user_agent);
}
```

### Q: Does it support async?
**A:** Full async support! All I/O operations are async:
```rust
let response = client.get(url).await?; // Async HTTP request
```

### Q: How do I integrate with databases?
**A:** Serialize and store:
```rust
let fp = get_random_fingerprint()?;
let json_str = serde_json::to_string(&fp)?;
// Store in database...
```

## Compatibility Questions

### Q: Does it work on Windows?
**A:** Yes! Full support:
- ✅ Windows 10/11
- ✅ All features available
- ✅ CI/CD tests Windows

### Q: macOS support?
**A:** 
- ✅ Intel and Apple Silicon (M1/M2/M3)
- ✅ macOS 10.14+
- ✅ All features available

### Q: Which Linux distributions?
**A:** 
- ✅ Ubuntu, Debian, Fedora, CentOS, etc.
- ✅ All major distributions
- ⚠️ Requires glibc 2.17+

## License Questions

### Q: What is the license?
**A:** MIT and BSD-3-Clause dual license, meaning you can:
- ✅ Use and modify freely
- ✅ Use in commercial projects
- ✅ Create closed-source applications
- 📝 Must preserve license notices

### Q: Can I use it for commercial purposes?
**A:** Yes! Completely allowed in commercial projects.

## Debugging Questions

### Q: How do I enable logging?
**A:** 
```bash
# All modules
RUST_LOG=debug cargo run

# Specific module
RUST_LOG=fingerprint_core=trace cargo run

# Specific level
RUST_LOG=fingerprint=info,fingerprint_core=debug cargo run
```

### Q: How do I report performance issues?
**A:** 
1. Run benchmarks: `cargo bench`
2. Enable profiling: `cargo flamegraph`
3. Attach results and environment info in issue

## Future Plans

### Q: What's the project roadmap?
**A:** Near-term plans:
- 🚀 Full wasm support
- 🚀 More ML model integration
- 🚀 Enhanced privacy protection
- 🚀 Improved documentation
- 🚀 Performance optimization

### Q: How do I follow project updates?
**A:** 
- ⭐ GitHub Star
- 👁️ Watch repository
- 📧 Subscribe to Release notifications
- 💬 Join Discussions

## Getting More Help

- 📖 [Full Documentation](https://github.com/vistone/fingerprint-rust)
- 🐛 [GitHub Issues](https://github.com/vistone/fingerprint-rust/issues)
- 💬 [Discussions](https://github.com/vistone/fingerprint-rust/discussions)
- 📧 Contributors and maintainers are ready to help

---

**Last Updated**: February 24, 2026
