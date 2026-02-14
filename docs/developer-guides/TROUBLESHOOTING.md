# Troubleshooting Guide

**Version**: 2.1.0  
**Last Updated**: 2026-01-07

This guide helps you diagnose and resolve common issues with fingerprint-rust.

---

## Table of Contents

1. [Build Issues](#build-issues)
2. [Runtime Errors](#runtime-errors)
3. [Performance Problems](#performance-problems)
4. [Network Issues](#network-issues)
5. [Testing Issues](#testing-issues)
6. [Platform-Specific Issues](#platform-specific-issues)

---

## Build Issues

### Issue: Compilation Fails with Missing Dependencies

**Symptoms**:
```
error: failed to resolve: could not find `rustls` in the list of imported crates
```

**Solutions**:

1. **Enable required features**:
   ```bash
   cargo build --features "rustls-tls,http2"
   ```

2. **Check Cargo.toml dependencies**:
   ```toml
   [dependencies]
   fingerprint = { version = "2.1", features = ["rustls-tls", "http2", "http3"] }
   ```

3. **Clean and rebuild**:
   ```bash
   cargo clean
   cargo build
   ```

### Issue: Linker Errors on Linux

**Symptoms**:
```
error: linking with `cc` failed
note: /usr/bin/ld: cannot find -lssl
```

**Solutions**:

1. **Install OpenSSL development packages**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install libssl-dev pkg-config
   
   # Fedora/RHEL
   sudo dnf install openssl-devel
   
   # Arch Linux
   sudo pacman -S openssl pkg-config
   ```

2. **Use rustls instead** (recommended):
   ```toml
   fingerprint = { version = "2.1", features = ["rustls-tls"] }
   ```

### Issue: Build is Very Slow

**Symptoms**:
- Compilation takes more than 5 minutes
- High CPU and memory usage during build

**Solutions**:

1. **Use faster linker (mold or lld)**:
   ```toml
   # .cargo/config.toml
   [target.x86_64-unknown-linux-gnu]
   linker = "clang"
   rustflags = ["-C", "link-arg=-fuse-ld=mold"]
   ```

2. **Reduce parallel jobs**:
   ```bash
   cargo build -j 2
   ```

3. **Use sccache for caching**:
   ```bash
   cargo install sccache
   export RUSTC_WRAPPER=sccache
   ```

---

## Runtime Errors

### Issue: TLS Handshake Failures

**Symptoms**:
```
Error: TLS handshake failed: InvalidCertificate
```

**Solutions**:

1. **Update root certificates**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get update
   sudo apt-get install ca-certificates
   
   # macOS
   brew install ca-certificates
   ```

2. **Disable certificate validation** (development only):
   ```rust
   use fingerprint::HttpClientConfig;
   
   let config = HttpClientConfig {
       dangerous_accept_invalid_certs: true, // ⚠️ Development only!
       ..Default::default()
   };
   ```

3. **Check system time**:
   Ensure your system clock is accurate. TLS certificates have validity periods.

### Issue: HTTP Request Timeouts

**Symptoms**:
```
Error: operation timed out after 30s
```

**Solutions**:

1. **Increase timeout**:
   ```rust
   let config = HttpClientConfig {
       timeout: Duration::from_secs(60),
       ..Default::default()
   };
   ```

2. **Check network connectivity**:
   ```bash
   ping google.com
   curl -I https://example.com
   ```

3. **Test with minimal config**:
   ```rust
   let client = HttpClient::new(HttpClientConfig::default());
   let response = client.get("https://example.com")?;
   ```

### Issue: Panic in ClientHello Generation

**Symptoms**:
```
thread 'main' panicked at 'index out of bounds'
```

**Solutions**:

1. **Validate input data**:
   ```rust
   let profile = chrome_133();
   let spec = profile.get_client_hello_spec()?; // Returns Result
   ```

2. **Check for empty vectors**:
   ```rust
   if spec.cipher_suites.is_empty() {
       return Err("No cipher suites configured");
   }
   ```

3. **Report the issue**:
   If this happens with standard profiles, please file a bug report with:
   - Rust version (`rustc --version`)
   - Fingerprint-rust version
   - Minimal reproduction code

---

## Performance Problems

### Issue: High Memory Usage

**Symptoms**:
- Process uses more than 1GB of RAM
- Out-of-memory errors

**Solutions**:

1. **Use connection pooling**:
   ```rust
   use fingerprint::{HttpClient, HttpClientConfig};
   
   let config = HttpClientConfig {
       connection_pool_enabled: true,
       max_idle_connections: 10,
       ..Default::default()
   };
   ```

2. **Limit concurrent requests**:
   ```rust
   use tokio::sync::Semaphore;
   
   let semaphore = Arc::new(Semaphore::new(10)); // Max 10 concurrent
   ```

3. **Profile memory usage**:
   ```bash
   # Linux
   valgrind --tool=massif ./target/release/my_app
   
   # macOS
   cargo instruments --template "Allocations"
   ```

### Issue: Slow Request Processing

**Symptoms**:
- Requests take > 1 second on fast networks
- CPU usage is high during requests

**Solutions**:

1. **Enable HTTP/3** (fastest):
   ```rust
   let config = HttpClientConfig {
       prefer_http3: true,
       ..Default::default()
   };
   ```

2. **Use DNS pre-resolution**:
   ```rust
   use fingerprint::dns::DnsService;
   
   let dns_service = DnsService::new(config)?;
   dns_service.resolve("example.com")?;
   ```

3. **Profile with flamegraph**:
   ```bash
   cargo install flamegraph
   cargo flamegraph --example my_example
   ```

---

## Network Issues

### Issue: Connection Refused

**Symptoms**:
```
Error: Connection refused (os error 111)
```

**Solutions**:

1. **Check server is accessible**:
   ```bash
   curl -I https://target-server.com
   nc -zv target-server.com 443
   ```

2. **Check firewall settings**:
   ```bash
   # Linux
   sudo iptables -L -n -v
   
   # macOS
   /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate
   ```

3. **Test with different IP**:
   ```rust
   // Try IPv4 explicitly
   let response = client.get("https://142.250.185.46")?; // Google IP
   ```

### Issue: DNS Resolution Failures

**Symptoms**:
```
Error: failed to lookup address information
```

**Solutions**:

1. **Check /etc/resolv.conf** (Linux/macOS):
   ```bash
   cat /etc/resolv.conf
   # Should contain: nameserver 8.8.8.8
   ```

2. **Use custom DNS resolver**:
   ```rust
   use fingerprint::dns::DnsConfig;
   
   let config = DnsConfig {
       servers: vec!["8.8.8.8:53".parse()?],
       ..Default::default()
   };
   ```

3. **Test DNS manually**:
   ```bash
   nslookup google.com
   dig google.com
   ```

---

## Testing Issues

### Issue: Tests Fail Randomly

**Symptoms**:
- Tests pass sometimes, fail other times
- Network-dependent tests fail on CI

**Solutions**:

1. **Mark network tests as ignored**:
   ```rust
   #[test]
   #[ignore] // Run with: cargo test -- --ignored
   fn test_real_network_request() {
       // ...
   }
   ```

2. **Use mocking for network calls**:
   ```rust
   #[cfg(test)]
   mod tests {
       use mockito::{mock, server_url};
       
       #[test]
       fn test_with_mock() {
           let _m = mock("GET", "/")
               .with_status(200)
               .with_body("OK")
               .create();
           // ...
       }
   }
   ```

3. **Add retries for flaky tests**:
   ```rust
   for attempt in 1..=3 {
       match run_test() {
           Ok(_) => break,
           Err(e) if attempt < 3 => continue,
           Err(e) => panic!("Test failed: {}", e),
       }
   }
   ```

### Issue: Property Tests Take Too Long

**Symptoms**:
```
test proptest_ja3 ... running for 60+ seconds
```

**Solutions**:

1. **Reduce test cases**:
   ```rust
   proptest! {
       #![proptest_config(ProptestConfig::with_cases(100))]
       #[test]
       fn my_prop_test(input in any::<u32>()) {
           // ...
       }
   }
   ```

2. **Run in release mode**:
   ```bash
   cargo test --release
   ```

---

## Platform-Specific Issues

### Windows

**Issue**: Build fails with "procedure entry point" error

**Solution**:
```bash
# Update Visual C++ Redistributable
# Download from: https://aka.ms/vs/17/release/vc_redist.x64.exe
```

**Issue**: TLS errors with native-tls

**Solution**: Use rustls instead:
```toml
fingerprint = { version = "2.1", features = ["rustls-tls"] }
```

### macOS

**Issue**: "dyld: Library not loaded" error

**Solution**:
```bash
# Install Homebrew dependencies
brew install openssl@3
brew link openssl@3 --force
```

**Issue**: Code signing issues

**Solution**:
```bash
# Disable Gatekeeper for development
spctl --master-disable
```

### Linux (ARM/aarch64)

**Issue**: Build fails with "ring" compilation error

**Solution**:
```bash
# Install cross-compilation tools
sudo apt-get install gcc-aarch64-linux-gnu
export CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
```

---

## Getting Help

If you can't resolve your issue with this guide:

1. **Check existing issues**: https://github.com/vistone/fingerprint-rust/issues
2. **Search discussions**: https://github.com/vistone/fingerprint-rust/discussions
3. **Ask for help**: Create a new issue with:
   - Rust version (`rustc --version`)
   - Operating system and version
   - Minimal reproduction code
   - Full error message and stack trace
   - What you've tried so far

---

## Debug Checklist

Before reporting an issue, try:

- [ ] Update to latest stable Rust: `rustup update`
- [ ] Clean build: `cargo clean && cargo build`
- [ ] Check dependencies: `cargo tree | grep fingerprint`
- [ ] Run with logging: `RUST_LOG=debug cargo run`
- [ ] Test with minimal config: `HttpClientConfig::default()`
- [ ] Verify network connectivity: `curl -I https://example.com`
- [ ] Check system time: `date`
- [ ] Test on a different machine/network

---

## Performance Optimization Tips

1. **Use release builds**: Always test performance with `--release`
2. **Enable LTO**: Add to Cargo.toml:
   ```toml
   [profile.release]
   lto = true
   codegen-units = 1
   ```

3. **Profile before optimizing**:
   ```bash
   cargo flamegraph
   cargo bloat --release
   ```

4. **Consider connection pooling**: Reuse connections when possible
5. **Use HTTP/3**: Fastest protocol with lowest latency

---

**Need more help?** Join our community discussions or file an issue on GitHub!
