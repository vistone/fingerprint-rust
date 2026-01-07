# Interactive Examples and Tutorials

This document provides links to interactive examples, video tutorials, and hands-on guides for learning fingerprint-rust.

## 📺 Video Tutorials

### Getting Started Series

> **Note**: Video tutorials are planned for future release. Check back soon or subscribe to our YouTube channel for updates.

**Planned Topics:**
1. **Introduction to TLS Fingerprinting** (15 min)
   - What is TLS fingerprinting?
   - JA3 and JA3S explained
   - Real-world use cases

2. **Quick Start with fingerprint-rust** (10 min)
   - Installation and setup
   - First TLS fingerprint
   - Reading fingerprint results

3. **Advanced Fingerprinting Techniques** (20 min)
   - JA4+ fingerprints
   - HASSH for SSH
   - JARM for server identification

4. **Building Custom Fingerprinting Tools** (25 min)
   - Creating a custom HTTP client
   - Integrating fingerprints into your app
   - Best practices

### Webinar Recordings

- **Detecting Bots with TLS Fingerprints** - Coming Soon
- **Fingerprint-Based Security** - Coming Soon
- **Real-World Case Studies** - Coming Soon

## 🎮 Interactive Examples

### Online Playground

Try fingerprint-rust directly in your browser:

> **Rust Playground**: https://play.rust-lang.org/
> 
> Copy one of our examples below and paste into the playground!

### Example 1: Basic JA3 Generation

```rust
use fingerprint_core::ja3::JA3;

fn main() {
    // Chrome-like TLS configuration
    let ja3 = JA3::generate(
        771,  // TLS 1.2
        &[0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f],  // Ciphers
        &[0, 10, 11, 13, 16, 23],  // Extensions
        &[23, 24, 25],  // Curves
        &[0],  // Point formats
    );
    
    println!("JA3 Fingerprint: {}", ja3.fingerprint);
    println!("JA3 String: {}", ja3.ja3_string);
}
```

[▶️ Try in Playground](https://play.rust-lang.org/)

### Example 2: HASSH SSH Fingerprint

```rust
use fingerprint_core::hassh::HASSH;

fn main() {
    // OpenSSH-like configuration
    let hassh = HASSH::generate(
        &["curve25519-sha256", "diffie-hellman-group-exchange-sha256"],
        &["chacha20-poly1305@openssh.com", "aes128-ctr"],
        &["hmac-sha2-256", "hmac-sha2-512"],
        &["none", "zlib@openssh.com"],
    );
    
    println!("HASSH Fingerprint: {}", hassh.fingerprint);
    println!("Client Type: {:?}", hassh.client_type);
}
```

[▶️ Try in Playground](https://play.rust-lang.org/)

### Example 3: JA4 Fingerprint

```rust
use fingerprint_core::ja4::JA4;

fn main() {
    // Modern browser configuration
    let ja4 = JA4::generate(
        't',      // TCP transport
        "1.3",    // TLS 1.3
        true,     // Has SNI
        &[0x1301, 0x1302, 0x1303],  // Ciphers
        &[0, 10, 11, 13, 16],  // Extensions
        Some("h2"),  // ALPN
        &[0x0403, 0x0804],  // Signature algorithms
    );
    
    println!("JA4: {}", ja4.to_fingerprint_string());
}
```

[▶️ Try in Playground](https://play.rust-lang.org/)

### Example 4: HTTP Client with Fingerprinting

```rust
use fingerprint_http::HttpClient;
use fingerprint_tls::TlsConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with Chrome-like fingerprint
    let tls_config = TlsConfig::chrome_latest();
    let client = HttpClient::builder()
        .tls_config(tls_config)
        .build()?;
    
    let response = client.get("https://example.com").send().await?;
    println!("Status: {}", response.status_code);
    
    Ok(())
}
```

## 🧪 Interactive Exercises

### Exercise 1: Identify Your Browser

**Goal**: Generate a JA3 fingerprint for your current browser

**Steps**:
1. Visit https://ja3er.com/
2. Note your JA3 fingerprint
3. Use fingerprint-rust to generate the same fingerprint
4. Compare and understand the differences

**Challenge**: Can you modify the fingerprint to match a different browser?

### Exercise 2: Bot Detection

**Goal**: Build a simple bot detector using TLS fingerprints

**Steps**:
1. Collect JA3 fingerprints from known bots
2. Collect JA3 fingerprints from real browsers
3. Create a database of fingerprints
4. Write code to classify incoming requests

**Starter Code**:
```rust
use std::collections::HashSet;
use fingerprint_core::ja3::JA3;

struct BotDetector {
    known_bots: HashSet<String>,
}

impl BotDetector {
    fn is_bot(&self, ja3: &JA3) -> bool {
        self.known_bots.contains(&ja3.fingerprint)
    }
}
```

### Exercise 3: Server Fingerprinting

**Goal**: Use JARM to identify different servers

**Steps**:
1. Set up test servers (nginx, Apache, Cloudflare)
2. Generate JARM fingerprints for each
3. Create a server identification function
4. Test against real-world servers

## 📚 Tutorial Series

### Beginner Tutorials

1. **[Getting Started](../README.md)** - Installation and basic usage
2. **[Understanding JA3](./FINGERPRINT_BASICS.md)** - Learn about JA3 fingerprints
3. **[First HTTP Client](../examples/README.md)** - Build your first client

### Intermediate Tutorials

1. **[Custom TLS Configurations](../examples/tls_client.rs)** - Advanced TLS setup
2. **[Packet Analysis](./PACKET_ANALYSIS.md)** - Parse network packets
3. **[Bot Detection](./BOT_DETECTION.md)** - Build a bot detector

### Advanced Tutorials

1. **[Building a Fingerprint Database](./DATABASE.md)** - Store and query fingerprints
2. **[Real-time Analysis](./REALTIME.md)** - Stream processing
3. **[Custom Fingerprints](./CUSTOM_FINGERPRINTS.md)** - Create new fingerprint types

## 🔬 Labs and Workshops

### Lab 1: TLS Fingerprint Analysis

**Duration**: 60 minutes

**Objectives**:
- Capture TLS handshakes from different clients
- Extract and compare JA3 fingerprints
- Identify patterns and anomalies

**Materials**:
- Wireshark or tcpdump
- fingerprint-rust library
- Sample PCAP files

**Guide**: Coming Soon

### Lab 2: SSH Client Detection

**Duration**: 45 minutes

**Objectives**:
- Analyze SSH KEX_INIT messages
- Generate HASSH fingerprints
- Identify SSH client types

**Materials**:
- SSH server
- Various SSH clients
- fingerprint-rust library

**Guide**: Coming Soon

### Workshop: Building a Fingerprint-Based WAF

**Duration**: 3 hours

**Topics**:
- Understanding web application attacks
- Using fingerprints for security
- Implementing rate limiting
- Bot detection strategies

**Materials**: Coming Soon

## 🎯 Challenge Projects

### Challenge 1: Fingerprint Collector

Build a web service that collects TLS fingerprints from visitors and displays statistics.

**Requirements**:
- Web server (Axum/Actix)
- Database (SQLite/PostgreSQL)
- Real-time dashboard
- API for querying fingerprints

### Challenge 2: Browser Emulator

Create a tool that can emulate any browser's TLS fingerprint.

**Requirements**:
- Parse fingerprint profiles
- Generate matching TLS configurations
- Validate against real browsers
- Support multiple TLS versions

### Challenge 3: Network Analyzer

Build a passive network analyzer that identifies devices based on fingerprints.

**Requirements**:
- Packet capture
- Real-time processing
- Device classification
- Web interface for results

## 📖 Documentation Deep Dives

### Deep Dive: JA3 Algorithm

Comprehensive explanation of how JA3 fingerprints work:
- [JA3 Specification](./JA3_SPEC.md)
- [Implementation Details](../crates/fingerprint-core/src/ja3.rs)
- [GREASE Handling](./GREASE.md)

### Deep Dive: JARM Probing

Understanding active TLS server fingerprinting:
- [JARM Specification](./JARM_SPEC.md)
- [Probe Design](./JARM_PROBES.md)
- [Server Detection](./SERVER_DETECTION.md)

## 🌐 Community Resources

### Code Examples Repository

Browse community-contributed examples:
- **GitHub**: https://github.com/vistone/fingerprint-rust/tree/main/examples
- **Discussions**: https://github.com/vistone/fingerprint-rust/discussions

### Third-Party Tutorials

- [Blog: TLS Fingerprinting with Rust](https://example.com) - Coming Soon
- [YouTube: Building Security Tools](https://youtube.com) - Coming Soon
- [Podcast: Fingerprint Technology](https://podcast.com) - Coming Soon

## 💬 Getting Help

### Ask Questions

- **Discord**: Join our community (link coming soon)
- **GitHub Discussions**: https://github.com/vistone/fingerprint-rust/discussions
- **Stack Overflow**: Tag your questions with `fingerprint-rust`

### Office Hours

We host weekly office hours for live Q&A:
- **Time**: Thursdays, 3 PM UTC (Coming Soon)
- **Platform**: Discord
- **Topics**: Any fingerprint-rust questions

## 🚀 Contributing Tutorials

Want to create a tutorial? We'd love your help!

1. Fork the repository
2. Create your tutorial in `docs/tutorials/`
3. Follow our [tutorial template](./TUTORIAL_TEMPLATE.md)
4. Submit a pull request

**Tutorial Guidelines**:
- Clear learning objectives
- Step-by-step instructions
- Runnable code examples
- Expected outcomes
- Troubleshooting section

## 📅 Upcoming Events

- **Webinar: Introduction to TLS Fingerprinting** - Coming Soon
- **Workshop: Advanced Fingerprinting Techniques** - Coming Soon
- **Hackathon: Build with fingerprint-rust** - Coming Soon

Subscribe to our [newsletter](#) for event announcements!

---

**Last Updated**: 2026-01-07

**Feedback**: Have ideas for tutorials or examples? [Open an issue](https://github.com/vistone/fingerprint-rust/issues)!
