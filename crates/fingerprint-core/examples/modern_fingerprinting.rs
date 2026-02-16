//! Example: Modern Fingerprinting Technologies (2025-2026)
//!
//! Demonstrates the latest fingerprint detection capabilities including:
//! - JA4+ fingerprinting (JA4, JA4X)
//! - Post-Quantum Cryptography (PQC) detection
//! - WebAssembly (WASM) capabilities fingerprinting
//! - Comprehensive error handling

use fingerprint_core::{
    // JA4+ fingerprints
    JA4, JA4X, JA4L,
    // Post-Quantum Cryptography
    PQCCapabilities, PQCBrowserSupport,
    // WebAssembly
    WasmCapabilities, WasmBrowserSupport,
    // Error handling
    Result,
};

fn main() -> Result<()> {
    println!("=== Modern Fingerprinting Technologies Demo ===\n");

    // 1. JA4 TLS Fingerprinting
    demonstrate_ja4()?;

    // 2. JA4X Certificate Fingerprinting
    demonstrate_ja4x()?;

    // 3. Post-Quantum Cryptography Detection
    demonstrate_pqc()?;

    // 4. WebAssembly Fingerprinting
    demonstrate_wasm()?;

    // 5. Browser Consistency Checking
    demonstrate_consistency_checking()?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

fn demonstrate_ja4() -> Result<()> {
    println!("1. JA4 TLS Fingerprinting");
    println!("--------------------------");

    // Simulate a Chrome 120 TLS ClientHello
    let ja4 = JA4::generate(
        't',  // TCP transport
        "1.3", // TLS 1.3
        true,  // Has SNI
        &[0x1301, 0x1302, 0x1303], // Cipher suites: TLS_AES_128_GCM_SHA256, etc.
        &[0, 10, 11, 13, 16, 43, 45], // Extensions: SNI, supported_groups, etc.
        Some("h2"), // ALPN: HTTP/2
        &[0x0403, 0x0804], // Signature algorithms
    );

    println!("  Fingerprint: {}", ja4);
    println!("  Transport: {} (TCP)", ja4.transport);
    println!("  TLS Version: {}", ja4.version);
    println!("  Cipher Count: {}", ja4.cipher_count);
    println!("  Extension Count: {}", ja4.extension_count);
    println!("  ALPN: {}", ja4.alpn);
    println!();

    // Lightweight version for resource-constrained environments
    let ja4l = JA4L::generate(
        't',
        "1.3",
        true,
        &[0x1301, 0x1302, 0x1303],
        &[0, 10, 11, 13],
    );
    println!("  JA4L (Lightweight): {}", ja4l);
    println!("  Memory footprint: {} bytes", ja4l.memory_footprint());
    println!();

    Ok(())
}

fn demonstrate_ja4x() -> Result<()> {
    println!("2. JA4X X.509 Certificate Fingerprinting");
    println!("----------------------------------------");

    // Simulate a typical web server certificate
    let ja4x = JA4X::generate(
        "sha256_rsa",  // RSA signature with SHA-256
        "rsa",         // RSA public key
        2048,          // 2048-bit key
        &[
            "subjectAltName",
            "keyUsage",
            "extendedKeyUsage",
            "basicConstraints",
            "authorityKeyIdentifier",
        ],
    );

    println!("  Fingerprint: {}", ja4x);
    println!("  Signature Algorithm: {}", ja4x.signature_algorithm);
    println!("  Key Algorithm: {}", ja4x.key_algorithm);
    println!("  Key Size: {} bits", ja4x.key_size);
    println!("  Extension Count: {}", ja4x.extension_count);
    println!();

    // Modern certificate with ECDSA
    let ja4x_ecdsa = JA4X::generate(
        "sha256_ecdsa",
        "ecdsa",
        256,
        &["subjectAltName", "keyUsage", "extendedKeyUsage"],
    );
    println!("  ECDSA Fingerprint: {}", ja4x_ecdsa);
    println!();

    Ok(())
}

fn demonstrate_pqc() -> Result<()> {
    println!("3. Post-Quantum Cryptography Detection");
    println!("-------------------------------------");

    // Simulate TLS extensions indicating PQC support
    let pqc_extensions = vec![
        0xFE31, // Kyber768
        0xFE34, // Hybrid: X25519 + Kyber768
    ];

    let pqc_caps = PQCCapabilities::from_tls_extensions(&pqc_extensions);

    println!("  PQC Support: {}", pqc_caps.supported);
    println!("  Algorithms detected: {}", pqc_caps.algorithms.len());
    for algo in &pqc_caps.algorithms {
        println!("    - {} ({}bits security)", algo.tls_name(), algo.security_level());
    }
    println!("  Hybrid Mode: {}", pqc_caps.hybrid_mode);
    println!("  Max Security Level: {} bits", pqc_caps.max_security_level());
    println!("  Fingerprint: {}", pqc_caps.fingerprint());
    println!();

    // Check browser compatibility
    println!("  Browser PQC Support:");
    for (browser, version) in &[("chrome", 120), ("firefox", 125), ("safari", 17)] {
        let supports = PQCBrowserSupport::supports_pqc(browser, *version);
        println!("    {} v{}: {}", browser, version, if supports { "✓" } else { "✗" });
    }
    println!();

    // Detect anomalies
    let anomaly = PQCBrowserSupport::detect_anomaly("chrome", 120, &pqc_caps);
    match anomaly {
        Some(msg) => println!("  ⚠️ Anomaly detected: {}", msg),
        None => println!("  ✓ PQC support matches expected browser behavior"),
    }
    println!();

    Ok(())
}

fn demonstrate_wasm() -> Result<()> {
    println!("4. WebAssembly Fingerprinting");
    println!("-----------------------------");

    // Modern browser WASM capabilities
    let wasm_caps = WasmCapabilities::modern_browser();

    println!("  WASM Available: {}", wasm_caps.available);
    println!("  Supported Features:");
    for version in &wasm_caps.versions {
        println!("    - {}", version.feature_name());
    }
    println!("  Streaming Compilation: {}", wasm_caps.streaming_compilation);
    println!("  Memory: {}", wasm_caps.memory.fingerprint());
    if let Some(speed) = wasm_caps.compilation_speed {
        println!("  Compilation Speed: {:.1} modules/sec", speed);
    }
    println!("  Fingerprint: {}", wasm_caps.fingerprint());
    println!();

    // Browser-specific capabilities
    println!("  Expected WASM support by browser:");
    for (browser, version) in &[
        ("chrome", 120),
        ("firefox", 125),
        ("safari", 17),
        ("chrome", 50), // Old version without WASM
    ] {
        let expected = WasmBrowserSupport::expected_capabilities(browser, *version);
        println!(
            "    {} v{}: {} features",
            browser,
            version,
            expected.versions.len()
        );
    }
    println!();

    // Check consistency
    match wasm_caps.matches_browser("chrome", 120) {
        Ok(_) => println!("  ✓ WASM capabilities match Chrome 120"),
        Err(e) => println!("  ✗ Mismatch: {}", e),
    }
    println!();

    Ok(())
}

fn demonstrate_consistency_checking() -> Result<()> {
    println!("5. Cross-Feature Consistency Checking");
    println!("------------------------------------");

    // Simulate detecting inconsistencies that indicate automation/spoofing
    println!("  Checking consistency across multiple fingerprints...");
    println!();

    // Example: Chrome 120 should support both PQC and WASM SIMD
    let browser = "chrome";
    let version = 120;

    println!("  Checking {} v{}:", browser, version);

    // PQC check
    let pqc_expected = PQCBrowserSupport::supports_pqc(browser, version);
    println!("    PQC expected: {}", pqc_expected);

    // WASM check
    let wasm_expected = WasmBrowserSupport::supports_wasm(browser, version);
    let simd_expected = WasmBrowserSupport::supports_simd(browser, version);
    println!("    WASM expected: {}", wasm_expected);
    println!("    WASM SIMD expected: {}", simd_expected);

    // Simulate a case where features don't match
    println!();
    println!("  Example anomaly: Browser claims to be Chrome 120 but...");
    
    let no_pqc = PQCCapabilities::none();
    if let Some(msg) = PQCBrowserSupport::detect_anomaly(browser, version, &no_pqc) {
        println!("    ⚠️ {}", msg);
    }

    let no_wasm = WasmCapabilities::none();
    if let Err(msg) = no_wasm.matches_browser(browser, version) {
        println!("    ⚠️ {}", msg);
    }

    println!();
    println!("  Such inconsistencies can indicate:");
    println!("    - Headless browser automation");
    println!("    - User-Agent spoofing");
    println!("    - Modified browser builds");
    println!("    - Outdated automation frameworks");
    println!();

    Ok(())
}
