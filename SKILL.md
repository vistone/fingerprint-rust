# fingerprint-rust: Comprehensive Browser & Network Fingerprinting Library

## 📋 Project Overview

**fingerprint-rust** is a **production-grade, multi-layered fingerprinting library** written in Rust, supporting comprehensive browser identification across network, protocol, and API layers. It combines 19 specialized modules to provide TLS fingerprinting, HTTP fingerprinting, browser API fingerprinting, and advanced threat detection.

### Current Version
- **Version**: 2.1.0
- **Architecture**: Cargo Workspace with 19 modular crates
- **Quality**: 100% test pass rate, production-ready
- **License**: BSD-3-Clause
- **Repository**: https://github.com/vistone/fingerprint-rust

### 🎯 Key Capabilities

| Layer | Technology | Status | Browsers |
|-------|-----------|--------|----------|
| **Network** | JA4 (TLS), JA4H (HTTP), JA4T (TCP) | ✅ Complete | 69+ versions |
| **TLS** | ClientHello Fingerprinting, Real Key Generation | ✅ Complete | 6 cores × 69+ versions |
| **HTTP** | HTTP/1.1, HTTP/2, HTTP/3 (QUIC) | ✅ Complete | Multi-protocol support |
| **Browser APIs** | Canvas, WebGL, WebRTC, Audio, Fonts, Storage, Hardware | ✅ Complete | Real-time detection |
| **Anomaly Detection** | Cross-layer consistency, ML-based anomalies | ✅ Complete | Threat detection |
| **Performance** | 40-48ms avg response time | ✅ Complete | Optimized |

---

## 🏗️ Architecture Overview

### Workspace Structure: 19 Specialized Crates

```
fingerprint-rust/
├── Core Foundation (2 crates)
│   ├── fingerprint-core          # Types, utilities, TLS dictionary
│   ├── fingerprint-tls           # TLS configuration, extensions, handshakes
│
├── Profile & Configuration (2 crates)
│   ├── fingerprint-profiles      # 69+ browser fingerprint configurations
│   ├── fingerprint-headers       # HTTP headers, User-Agent generation
│
├── Protocol Support (2 crates)
│   ├── fingerprint-http          # HTTP/1.1, HTTP/2, HTTP/3 client
│   ├── fingerprint-dns           # DNS resolution with caching
│
├── Defense & Analysis (2 crates)
│   ├── fingerprint-defense       # JA4+, consistency audit, threat detection
│   ├── fingerprint-anomaly       # ML-based anomaly detection
│
├── Browser API Fingerprinting (6 crates)
│   ├── fingerprint-canvas        # Canvas fingerprinting detection
│   ├── fingerprint-webgl         # WebGL properties fingerprinting
│   ├── fingerprint-audio         # Audio context fingerprinting
│   ├── fingerprint-fonts         # Font enumeration detection
│   ├── fingerprint-webrtc        # WebRTC IP leakage detection
│   ├── fingerprint-hardware      # Hardware capabilities detection
│
├── Advanced Features (3 crates)
│   ├── fingerprint-timing        # Timing-based fingerprinting
│   ├── fingerprint-storage       # LocalStorage, IndexedDB analysis
│   ├── fingerprint-ml            # Machine learning classification
│
├── Integration (1 crate)
│   ├── fingerprint-api-noise     # API noise/obfuscation
│
└── Main Library (1 crate)
    └── fingerprint               # Unified public API
```

### Crate Responsibilities

#### **fingerprint-core**
- Core data types: `BrowserType`, `OperatingSystem`, `HTTPHeaders`
- TLS dictionary: cipher suites, extensions, curves, point formats
- Utility functions for fingerprint generation
- JA3/JA3S generation primitives

#### **fingerprint-tls**
- TLS ClientHello specification structures
- TLS extension implementations (supported_groups, signature_algorithms, etc.)
- Handshake message builders
- Real key pair generation using `ring` (X25519, P-256, P-384)
- GREASE value generation and handling

#### **fingerprint-profiles**
- 69+ browser fingerprint profiles (Chrome, Firefox, Safari, Opera, Edge)
- Version management (Chrome 103/133, Firefox 133, etc.)
- Mobile and application-specific profiles
- Profile lookup and randomization

#### **fingerprint-headers**
- HTTP header generation matching browser versions
- User-Agent string synthesis
- Accept, Accept-Language, Accept-Encoding generation
- HTTP/2 specific header handling (pseudo-headers)
- Security headers (Sec-CH-UA, Sec-Fetch-*)

#### **fingerprint-http**
- Multi-protocol HTTP client (HTTP/1.1, HTTP/2, HTTP/3)
- Automatic protocol negotiation with fallback
- Connection pooling integration
- Request/response handling
- Compression support (gzip, deflate, brotli)
- Redirect handling and keep-alive management

#### **fingerprint-dns**
- DNS resolver with caching
- DNS service integration
- Domain resolution with timeout handling
- Integration with HTTP client

#### **fingerprint-defense**
- **JA4+ generation**: TLS, HTTP, TCP fingerprinting
- **Passive analysis**: Traffic analysis without network modification
- **Consistency auditing**: Detect inconsistencies between layers
- **Self-learning**: Automatically identify unknown stable fingerprints
- **SQLite database**: Persistent analysis storage
- **Real-time capture**: Pcap-based traffic capture support

#### **fingerprint-canvas**
- Canvas fingerprinting detection and generation
- Canvas text rendering analysis
- Canvas image data fingerprinting
- WebWorker canvas fingerprinting support

#### **fingerprint-webgl**
- WebGL capability fingerprinting
- Renderer/Vendor string extraction
- Shader compilation fingerprinting
- Texture size and format detection

#### **fingerprint-audio**
- Audio context fingerprinting
- Audio properties analysis
- Oscillator and analyzer detection
- Media stream capabilities

#### **fingerprint-fonts**
- Font enumeration detection
- System font fingerprinting
- Web font detection
- Font rendering characteristics

#### **fingerprint-webrtc**
- WebRTC IP leakage detection
- ICE candidate fingerprinting
- Connection state analysis
- Media track capabilities

#### **fingerprint-hardware**
- Hardware capability detection (CPU cores, memory, GPU)
- Device orientation/motion sensor detection
- Battery API detection
- Screen resolution and DPI fingerprinting

#### **fingerprint-timing**
- Performance.timing analysis
- Request timing fingerprinting
- Navigation timing characteristics
- Resource timing analysis

#### **fingerprint-storage**
- LocalStorage, SessionStorage analysis
- IndexedDB capabilities detection
- Cookie policies and characteristics
- Storage quota detection

#### **fingerprint-ml**
- Machine learning model integration
- Browser classification using multiple features
- Anomaly scoring and detection
- Threat classification models

#### **fingerprint-api-noise**
- API response obfuscation
- Noise injection for fingerprint evasion
- Plausible value generation
- Randomization strategies

#### **fingerprint**
- Unified public API
- Feature flag management
- Configuration export (JSON, YAML, TOML)
- Random fingerprint generation
- All crate re-exports

---

## 🚀 Core Use Cases

### 1. **Browser Identification & Detection**
Use TLS and HTTP fingerprints to identify legitimate browsers vs. bots:
```rust
use fingerprint::get_random_fingerprint;

let fp = get_random_fingerprint()?;
println!("Browser: {}", fp.profile.get_client_hello_str());
println!("User-Agent: {}", fp.user_agent);
```

### 2. **Bot Detection & Fraud Prevention**
Detect inconsistencies between TLS, HTTP, and browser API fingerprints:
```rust
// Cross-layer consistency checking
let tls_fp = analyze_tls_handshake(client_hello)?;
let http_fp = analyze_http_headers(headers)?;
let canvas_fp = analyze_canvas(canvas_data)?;

if !consistent(tls_fp, http_fp, canvas_fp) {
    // Likely a bot!
}
```

### 3. **Legitimate Browser Emulation**
Generate realistic browser fingerprints for legitimate automation:
```rust
use fingerprint::{chrome_133, HttpClient, HttpClientConfig};

let profile = chrome_133();
let config = HttpClientConfig {
    user_agent: chrome_133_ua(),
    prefer_http3: true,
    ..Default::default()
};
let client = HttpClient::new(config);
let response = client.get("https://example.com")?;
```

### 4. **Security Monitoring & Anomaly Detection**
Implement ML-based threat detection:
```rust
use fingerprint::anomaly_detection::AnomalyDetector;

let detector = AnomalyDetector::new("models/threat.model")?;
let score = detector.score(&fingerprint_data)?;
if score > 0.8 {
    // High anomaly score - potential threat
}
```

### 5. **API Testing & Quality Assurance**
Test your API against realistic fingerprints:
```rust
for i in 0..100 {
    let fp = get_random_fingerprint()?;
    let response = client.get_with_fingerprint("https://api.example.com", &fp)?;
    assert_eq!(response.status, 200);
}
```

### 6. **Privacy Research & Analysis**
Study fingerprinting resistance and privacy:
```rust
let canvas_fp = canvas_fingerprinting::analyze(canvas_data)?;
let webgl_fp = webgl_fingerprinting::analyze(webgl_data)?;
let browser_fp = ja4_fingerprinting::generate(tls_hello)?;
// Analyze fingerprint uniqueness and stability
```

---

## 📚 Browser Support Matrix

### Core Browsers (6)
- **Google Chrome**: 103-133 (31 versions)
- **Mozilla Firefox**: 102-135 (≥20 versions)
- **Apple Safari**: 16.0-17.x (5+ versions)
- **Opera**: 89-91+ (3+ versions)
- **Microsoft Edge**: 120-133 (14+ versions)
- **Mobile Clients**: Android Chrome, iOS Safari (22+ variants)

### Version Coverage
- **Latest Versions**: Chrome 133, Firefox 135, Safari 17+
- **Legacy Support**: Chrome 20 (2012)
- **Mobile Diversity**: iOS/Android fingerprints across versions
- **Total**: 69+ unique fingerprint configurations

---

## 🔧 Technical Stack

### Core Dependencies
```toml
# TLS & Cryptography
rustls = "0.23"          # Modern TLS implementation
ring = "0.17.14"         # Cryptographic operations (real key generation)
sha2, md5 = "0.10"       # Hashing algorithms

# HTTP & Networking
h2 = "0.4"               # HTTP/2 implementation
quinn = "0.11"           # QUIC (HTTP/3) implementation
h3, h3-quinn = "0.0.8"   # HTTP/3 implementation
http = "1.1"             # HTTP types and utilities
flate2, brotli = "*"     # Compression support

# Async Runtime
tokio = "1.40"           # Async runtime (with full features)
tokio-rustls = "0.26"    # Tokio TLS integration

# Utilities
serde, serde_json = "*"  # JSON serialization
toml, serde_yaml = "*"   # Configuration format support
chrono = "0.4"           # Time handling
rand = "0.8"             # Random number generation

# Network
socket2 = "0.5"          # Low-level socket operations
netconnpool = "1.0.4"    # Connection pooling

# DNS
hickory-resolver = "0.25" # DNS resolution

# Data
bytes = "1.10"           # Byte utilities
httparse = "1.10"        # HTTP parsing
hex = "0.4"              # Hex encoding/decoding
```

### Quality Assurance Stack
```toml
# Testing
proptest = "1.4"         # Property-based testing
criterion = "*"          # Benchmarking

# Development
cargo-audit = "*"        # Dependency security auditing
cargo-fuzz = "*"         # Fuzzing infrastructure
```

### Rust Version
- **Minimum**: 1.92.0
- **Edition**: 2021
- **Platform Targets**: Linux, macOS, Windows
- **Architecture**: x86_64, ARM64

---

## 💡 Feature Flags & Configuration

### Essential Feature Combinations

```toml
# Full-featured (recommended)
fingerprint = { version = "2.1", features = [
    "rustls-tls",          # TLS support
    "compression",         # Gzip, deflate, brotli
    "http2",              # HTTP/2 support
    "http3",              # HTTP/3 (QUIC) support
    "connection-pool",    # Connection pooling
    "export",             # Config export
    "defense",            # JA4+, anomaly detection
]}

# Minimal (just TLS)
fingerprint = { version = "2.1", features = ["rustls-tls"] }

# API fingerprinting only
fingerprint = { version = "2.1", features = [
    "canvas",
    "webgl",
    "audio",
    "fonts",
    "webrtc",
    "hardware"
]}

# Network protocols only
fingerprint = { version = "2.1", features = [
    "rustls-tls",
    "http2",
    "http3",
    "dns"
]}

# Security-focused
fingerprint = { version = "2.1", features = [
    "rustls-tls",
    "defense",
    "anomaly-detection",
    "ml"
]}
```

---

## 🛠️ Common Development Tasks

### Task 1: Identify a Browser from TLS Handshake
```rust
use fingerprint::fingerprint_defense::JA4;

// Parse captured ClientHello bytes
let ja4 = JA4::generate_from_client_hello(&client_hello)?;
println!("JA4: {}", ja4.fingerprint);
println!("Identified: {}", ja4.identify())?;
```

### Task 2: Generate Realistic Browser Requests
```rust
use fingerprint::{chrome_133, HttpClient, HttpClientConfig};

let profile = chrome_133();
let config = HttpClientConfig {
    user_agent: profile.get_user_agent()?,
    headers: profile.get_http_headers()?,
    prefer_http3: true,
    ..Default::default()
};

let client = HttpClient::new(config);
for _ in 0..10 {
    let response = client.get("https://example.com")?;
    // Process response
}
```

### Task 3: Detect Anomalies
```rust
use fingerprint::fingerprint_defense::{ConsistencyAudit, AnomalyDetector};

let tls_data = capture_tls_handshake(&tcp_stream)?;
let http_data = capture_http_headers(&stream)?;
let api_data = capture_browser_api(&js_context)?;

let audit = ConsistencyAudit::new();
let score = audit.check_consistency(&tls_data, &http_data, &api_data)?;

if score < 0.7 {
    eprintln!("Potential bot or manipulation detected!");
}
```

### Task 4: Export Configuration
```rust
use fingerprint::{chrome_133, export::ExportFormat};

let profile = chrome_133();
let json = profile.export(ExportFormat::Json)?;
let yaml = profile.export(ExportFormat::Yaml)?;
let toml = profile.export(ExportFormat::Toml)?;

std::fs::write("chrome_133.json", json)?;
std::fs::write("chrome_133.yaml", yaml)?;
std::fs::write("chrome_133.toml", toml)?;
```

### Task 5: Random Fingerprint Generation
```rust
use fingerprint::{get_random_fingerprint, OperatingSystem};

// Pure random
let fp = get_random_fingerprint()?;

// Random with OS constraint
let fp = get_random_fingerprint_with_os(Some(OperatingSystem::Windows))?;

// Random by browser type
let fp = get_random_fingerprint_by_browser("Chrome")?;

// Random by browser + OS
let fp = get_random_fingerprint_by_browser_with_os("Firefox", OperatingSystem::Linux)?;
```

---

## 📋 API Reference: Key Functions

### Fingerprint Generation

```rust
// Random fingerprinting API
pub fn get_random_fingerprint() -> Result<FingerprintResult, Box<dyn Error>>
pub fn get_random_fingerprint_with_os(os: Option<OperatingSystem>) -> Result<FingerprintResult, Box<dyn Error>>
pub fn get_random_fingerprint_by_browser(browser_type: &str) -> Result<FingerprintResult, Box<dyn Error>>

// Structured API
pub fn chrome_133() -> BrowserProfile
pub fn firefox_135() -> BrowserProfile
pub fn safari_17() -> BrowserProfile
pub fn edge_133() -> BrowserProfile
pub fn opera_91() -> BrowserProfile
```

### Browser Profile API

```rust
impl BrowserProfile {
    pub fn get_client_hello_str(&self) -> String
    pub fn get_client_hello_spec(&self) -> Result<ClientHelloSpec, Error>
    pub fn get_user_agent(&self) -> String
    pub fn get_http_headers(&self) -> HTTPHeaders
    pub fn export(&self, format: ExportFormat) -> Result<String, Error>
}
```

### HTTP Client API

```rust
pub struct HttpClientConfig {
    pub user_agent: String,
    pub headers: HTTPHeaders,
    pub prefer_http3: bool,
    pub prefer_http2: bool,
    pub timeout: Duration,
    pub follow_redirects: bool,
    pub max_redirects: usize,
}

pub struct HttpClient {
    // ...
}

impl HttpClient {
    pub fn new(config: HttpClientConfig) -> Self
    pub fn get(&self, url: &str) -> Result<HttpResponse, Box<dyn Error>>
    pub fn post(&self, url: &str, body: &[u8]) -> Result<HttpResponse, Box<dyn Error>>
    pub fn head(&self, url: &str) -> Result<HttpResponse, Box<dyn Error>>
}
```

### Defense API

```rust
// JA4+ Fingerprinting
pub fn generate_ja4_tls(client_hello: &[u8]) -> Result<JA4Fingerprint, Error>
pub fn generate_ja4h_http(headers: &HTTPHeaders) -> Result<JA4HFingerprint, Error>
pub fn generate_ja4t_tcp(packet: &[u8]) -> Result<JA4TFingerprint, Error>

// Consistency Checking
pub struct ConsistencyAudit;
impl ConsistencyAudit {
    pub fn check_layers(&self, tls: &TLSData, http: &HTTPData, api: &APIData) -> Result<Score, Error>
}

// Anomaly Detection
pub struct AnomalyDetector;
impl AnomalyDetector {
    pub fn score(&self, fingerprint: &FingerprintData) -> Result<f64, Error>
}
```

---

## 🧪 Testing Strategy

### Test Coverage
- **Unit Tests**: All 19 crates with comprehensive coverage
- **Property-Based Tests**: 14+ proptest-based tests for fingerprint generation
- **Integration Tests**: Multi-crate interaction validation
- **Fuzzing**: cargo-fuzz targets for protocol parsing
- **Performance Tests**: Benchmark suite with statistical analysis
- **Real Environment**: Google Earth API end-to-end validation (100% pass)

### Running Tests

```bash
# All tests
cargo test --all --verbose

# Specific crate
cargo test -p fingerprint-tls

# With logging
RUST_LOG=debug cargo test --all -- --nocapture

# Fuzzing
cargo +nightly fuzz run fuzz_ja3_generation

# Benchmarks
cargo bench --all
```

---

## 📊 Performance Characteristics

### HTTP Request Performance
| Protocol | Avg Time | P95 | P99 | Throughput |
|----------|----------|-----|-----|-----------|
| HTTP/1.1 | 44.4ms | 64ms | 78ms | 22.5 req/s |
| HTTP/2 | 48.0ms | 72ms | 86ms | 20.8 req/s |
| HTTP/3 | 40.3ms | 61ms | 74ms | 24.8 req/s |

### Fingerprint Generation Performance
| Operation | Latency | Throughput |
|-----------|---------|-----------|
| Random Fingerprint | 0.564 μs | 1.8M ops/sec |
| HTTP Client Creation | 0.098 μs | 10.2M ops/sec |
| Profile Lookup | 0.015 μs | 66.7M ops/sec |
| JA3 Generation | 0.234 μs | 4.3M ops/sec |

### Memory Usage
- **Per Client**: ~256KB (with connection pool)
- **Per Fingerprint**: ~4KB
- **Profile Cache**: ~512KB (all 69+ profiles)
- **Connection Pool**: Configurable, typically 1-10MB

---

## 🔒 Security Considerations

### Design Principles
1. **Memory Safety**: No unsafe code except in critical crypto paths
2. **Constant Time**: Comparison operations use constant-time algorithms
3. **Random Generation**: Uses `ring` for cryptographically secure randomness
4. **Dependency Audit**: `cargo audit` in CI/CD pipeline
5. **Threat Model**: Active and passive monitoring for bot/attack patterns

### Security Features
- Real TLS key generation (not predictable values)
- GREASE value handling (unpredictable extensions)
- BoringSSL padding compatibility
- TLS 1.3 specific handling
- Automatic protocol downgrade on failure (secure fallback)
- Cross-layer consistency validation

### Vulnerability Reporting
- Use [SECURITY.md](SECURITY.md) for responsible disclosure
- GitHub Security Advisories support
- Regular dependency updates
- Security audit workflow in CI/CD

---

## 📖 Documentation Resources

| Document | Purpose | Location |
|----------|---------|----------|
| **README** | Quick start and features | [README.md](README.md) |
| **Architecture** | Deep design documentation | [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) |
| **API Reference** | Function signatures and usage | [docs/API.md](docs/API.md) |
| **Tutorials** | Step-by-step guides | [docs/TUTORIALS.md](docs/TUTORIALS.md) |
| **Troubleshooting** | Common issues and solutions | [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md) |
| **Changelog** | Version history | [docs/CHANGELOG.md](docs/CHANGELOG.md) |
| **Contributing** | Development guidelines | [CONTRIBUTING.md](CONTRIBUTING.md) |
| **Enhancement Summary** | Recent improvements | [docs/ENHANCEMENT_SUMMARY.md](docs/ENHANCEMENT_SUMMARY.md) |

---

## 🎯 Use Case Scenarios

### Scenario 1: Web Application Firewall (WAF)
```
Incoming Request → TLS Analysis → HTTP Analysis → API Fingerprinting → 
Cross-layer Consistency Check → Anomaly Score → Accept/Block Decision
```
Use: `fingerprint-defense`, `fingerprint-anomaly`

### Scenario 2: Legitimate Browser Automation
```
Target API → Get Browser Profile → Generate Realistic Fingerprint → 
Configure HTTP Client → Send Requests with Coordination
```
Use: `fingerprint-profiles`, `fingerprint-headers`, `fingerprint-http`

### Scenario 3: Research & Privacy Analysis
```
Capture Browser Data → Canvas/WebGL/Audio Analysis → TLS/HTTP Analysis → 
Uniqueness Computation → Tracking Report Generation
```
Use: All fingerprinting crates

### Scenario 4: Bot Detection Service
```
Client Connection → Capture TLS Handshake → Generate JA4 → 
Compare Against Known Bot Signatures → ML Classification → 
Generate Threat Score
```
Use: `fingerprint-defense`, `fingerprint-ml`, `fingerprint-anomaly`

### Scenario 5: API Rate Limiting
```
Request → Extract Fingerprint → Check Request History → 
Identify Clusters (same fingerprint) → Apply Rate Limits per Cluster
```
Use: `fingerprint-core`, Custom clustering logic

---

## 🚀 Getting Started Workflow

### 1. **Installation**
```bash
cargo add fingerprint --features "rustls-tls,http2,http3"
```

### 2. **Basic Generation**
```rust
use fingerprint::get_random_fingerprint;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fp = get_random_fingerprint()?;
    println!("Browser: {}", fp.profile.get_client_hello_str());
    println!("User-Agent: {}", fp.user_agent);
    Ok(())
}
```

### 3. **HTTP Request**
```rust
use fingerprint::{HttpClient, HttpClientConfig};

let config = HttpClientConfig {
    user_agent: fp.user_agent.clone(),
    prefer_http3: true,
    ..Default::default()
};
let client = HttpClient::new(config);
let response = client.get("https://api.example.com")?;
println!("Status: {}", response.status_code);
```

### 4. **Advanced: Consistency Checking**
```rust
use fingerprint::fingerprint_defense::ConsistencyAudit;

let audit = ConsistencyAudit::new();
let score = audit.check_layers(&tls_data, &http_data, &api_data)?;
if score < 0.8 {
    eprintln!("Inconsistency detected!");
}
```

---

## 🔗 External Resources

- **JA4 Specification**: https://github.com/FoxIO-LLC/ja4
- **JA3 Specification**: https://github.com/salesforce/ja3
- **TLS 1.3 RFC**: https://tools.ietf.org/html/rfc8446
- **HTTP/2 RFC**: https://tools.ietf.org/html/rfc7540
- **QUIC RFC**: https://tools.ietf.org/html/rfc9000
- **Rust async-await**: https://tokio.rs/

---

## 📝 Contributing

The project welcomes contributions for:
1. New browser versions and fingerprints
2. Additional API fingerprinting modules
3. Performance optimizations
4. Documentation improvements
5. Bug fixes and testing

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## 📈 Project Metrics

- **Total Lines of Code**: 50,000+
- **Number of Crates**: 19
- **Test Coverage**: 100% pass rate
- **Documented APIs**: 150+
- **Example Programs**: 15+
- **Browser Versions**: 69+
- **Dependencies**: Carefully curated and audited

---

## 🎓 Learning Paths

### Path 1: Beginner (Browser Identification)
1. Read [README.md](README.md)
2. Run `examples/basic.rs`
3. Run `examples/unified_fingerprint_demo.rs`
4. Try custom fingerprint in REPL

### Path 2: Intermediate (HTTP Client Integration)
1. Study [docs/API.md](docs/API.md)
2. Run `examples/http_headers.rs`, `examples/http_pool.rs`
3. Implement custom HTTP request logic
4. Integrate into your application

### Path 3: Advanced (Defense & Anomaly Detection)
1. Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
2. Study `fingerprint-defense` crate source
3. Implement custom anomaly detectors
4. Deploy threat detection service

### Path 4: Expert (Custom Fingerprinting)
1. Deep dive into TLS specifications
2. Implement custom browser profiles
3. Contribute new attack detection methods
4. Publish research using the library

---

## ✨ Key Differentiators

1. **Multi-Layer Approach**: Combines network, protocol, and API fingerprinting
2. **Real Key Generation**: Actual cryptographic keys, not fake data
3. **Production Ready**: 100% test pass rate, security audited
4. **Modular Design**: Use only what you need via feature flags
5. **Performance**: Sub-millisecond fingerprint generation
6. **Machine Learning**: Built-in anomaly detection and classification
7. **Comprehensive Coverage**: 69+ browser versions across 6 core browsers
8. **Active Defense**: Support for both detection and legitimate emulation

---

## 📞 Support & Community

- **Issues**: GitHub Issues for bug reports
- **Discussions**: GitHub Discussions for feature requests
- **Security**: See [SECURITY.md](SECURITY.md) for responsible disclosure
- **License**: BSD-3-Clause (commercial friendly)

---

**Documentation Version**: 2.1.0  
**Last Updated**: 2026-01-07  
**Maintained By**: fingerprint-rust team
