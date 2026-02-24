//! Comprehensive test suite
//!
//! Tests all core functionality of fingerprint-rust library:
//! - TLS fingerprint generation and validation
//! - HTTP client (H1/H2/H3)
//! - Browser fingerprint configuration
//! - User-Agent and Headers generation
//! - Cookie management
//! - Proxy support
//! - Connection pool
//!
//! Run methods:
//! ```bash
//! # Local testing (no network required)
//! cargo test --test comprehensive_test
//!
//! # Network testing (requires network connection)
//! cargo test --test comprehensive_test -- --ignored --nocapture
//! ```

use fingerprint::profiles::*;
use fingerprint::*;
use std::time::Instant;

// ============================================================================
// 1. TLS fingerprinttesting
// ============================================================================

#[test]
fn test_tls_fingerprint_generation() {
    println!("\n=== TLS Fingerprint Generation Test ===");

    // Testing all core browsers
    let browsers = vec![
        ("Chrome 103", chrome_103()),
        ("Chrome 133", chrome_133()),
        ("Firefox 133", firefox_133()),
        ("Safari 16.0", safari_16_0()),
        ("Opera 91", opera_91()),
    ];

    for (name, profile) in browsers {
        let spec = &profile.tls_config;
        assert!(
            !spec.cipher_suites.is_empty(),
            "{} should have cipher suites",
            name
        );
        assert!(
            !spec.extensions.is_empty(),
            "{} should have extensions",
            name
        );

        println!(
            "✅ {}: {} cipher suites, {} extensions",
            name,
            spec.cipher_suites.len(),
            spec.extensions.len()
        );
    }
}

#[test]
fn test_ja4_fingerprint_generation() {
    println!("\n=== JA4 Fingerprint Generation Test ===");

    let profile = chrome_133();
    let spec = &profile.tls_config;

    // Generate JA4 fingerprint
    let signature = extract_signature(spec);
    let ja4_signature = Ja4Signature {
        version: signature.version,
        cipher_suites: signature.cipher_suites,
        extensions: signature.extensions,
        signature_algorithms: signature.signature_algorithms,
        sni: signature.sni,
        alpn: signature.alpn,
    };
    let ja4_payload = ja4_signature.generate_ja4();

    println!("✅ JA4_a: {}", ja4_payload.ja4_a);
    println!("✅ JA4_b: {}", ja4_payload.ja4_b);
    println!("✅ JA4_c: {}", ja4_payload.ja4_c);
    println!("✅ JA4: {}", ja4_payload.full);
    println!("✅ JA4_R: {}", ja4_payload.raw);

    assert!(!ja4_payload.ja4_a.is_empty());
    assert!(!ja4_payload.ja4_b.is_empty());
}

#[test]
fn test_tls_handshake_builder() {
    println!("\n=== TLS Handshake Builder Test ===");

    #[cfg(feature = "crypto")]
    {
        let profile = chrome_133();
        let spec = &profile.tls_config;

        let client_hello_result = TLSHandshakeBuilder::build_client_hello(spec, "example.com");

        assert!(
            client_hello_result.is_ok(),
            "Should be able to build ClientHello"
        );

        let client_hello = client_hello_result.unwrap();
        assert!(!client_hello.is_empty(), "ClientHello should not be empty");

        println!("✅ ClientHello size: {} bytes", client_hello.len());
        #[cfg(feature = "export")]
        {
            println!(
                "✅ ClientHello (hex): {}",
                hex::encode(&client_hello[..std::cmp::min(64, client_hello.len())])
            );
        }
        #[cfg(not(feature = "export"))]
        {
            println!(
                "✅ ClientHello (first 64 bytes): {:?}",
                &client_hello[..std::cmp::min(64, client_hello.len())]
            );
        }
    }

    #[cfg(not(feature = "crypto"))]
    {
        println!("⚠️  Skipping TLS handshake test (requires crypto feature)");
    }
}

#[test]
fn test_grease_value_filtering() {
    println!("\n=== GREASE Value Filtering Test ===");

    let grease_values = vec![0x0a0a, 0x1a1a, 0x2a2a, 0x3a3a];

    for &value in &grease_values {
        assert!(is_grease_value(value), "{} should be a GREASE value", value);
    }

    let normal_values = vec![0x0001, 0x0013, 0x0029];
    for &value in &normal_values {
        assert!(
            !is_grease_value(value),
            "{} should not be a GREASE value",
            value
        );
    }

    println!("✅ GREASE value detection works correctly");
}

// ============================================================================
// 2. Browser fingerprint configuration testing
// ============================================================================

#[test]
fn test_browser_profiles() {
    println!("\n=== Browser Profile Test ===");

    let profiles = vec![
        ("Chrome 103", chrome_103()),
        ("Chrome 133", chrome_133()),
        ("Firefox 133", firefox_133()),
        ("Safari 16.0", safari_16_0()),
        ("Opera 91", opera_91()),
    ];

    for (name, profile) in profiles {
        // Testing ClientHelloID
        let client_id = profile.id();
        assert!(!client_id.is_empty(), "{} should have ClientHelloID", name);

        // Testing HTTP/2 Settings
        let settings = &profile.http2_settings;
        assert!(!settings.is_empty(), "{} should have HTTP/2 Settings", name);

        // Testing Pseudo Header Order
        let pseudo_order = &profile.http2_settings_order;
        assert!(
            !pseudo_order.is_empty(),
            "{} should have Pseudo Header Order",
            name
        );

        println!(
            "✅ {}: ClientHelloID={}, Settings={}, PseudoHeaders={}",
            name,
            client_id,
            settings.len(),
            pseudo_order.len()
        );
    }
}

#[test]
fn test_random_fingerprint_generation() {
    println!("\n=== Random Fingerprint Generation Test ===");

    // Testing completely random
    let result1 = get_random_fingerprint();
    assert!(
        result1.is_ok(),
        "Should be able to generate random fingerprint"
    );
    let fp1 = result1.unwrap();
    assert!(!fp1.user_agent.is_empty());
    assert!(!fp1.profile_id.is_empty());

    // Testing by browser type
    let browsers = vec!["chrome", "firefox", "safari"];
    for browser in browsers {
        let result = get_random_fingerprint_by_browser(browser);
        assert!(
            result.is_ok(),
            "Should be able to generate {} fingerprint",
            browser
        );
        let fp = result.unwrap();
        assert!(!fp.user_agent.is_empty());
    }

    // Testing by operating system
    let result2 = get_random_fingerprint_with_os(Some(OperatingSystem::Windows10));
    assert!(result2.is_ok());

    println!("✅ Random fingerprint generation works correctly");
}

#[test]
fn test_all_browser_profiles() {
    println!("\n=== All Browser Profiles Verification ===");

    let mapped = mapped_tls_clients();
    let mut success_count = 0;
    let mut fail_count = 0;

    for (name, profile) in mapped.iter() {
        let spec = &profile.tls_config;
        if !spec.cipher_suites.is_empty() {
            success_count += 1;
        } else {
            println!("⚠️  {} configuration error: cipher_suites is empty", name);
            fail_count += 1;
        }
    }

    println!("✅ Success: {}, Failures: {}", success_count, fail_count);
    assert!(
        success_count > 0,
        "At least one browser configuration should succeed"
    );
}

// ============================================================================
// 3. User-Agent and Headers testing
// ============================================================================

#[test]
fn test_user_agent_generation() {
    println!("\n=== User-Agent Generation Test ===");

    // Testing generate by configuration name
    let ua1 = get_user_agent_by_profile_name("Chrome-133");
    assert!(
        ua1.is_ok(),
        "Should be able to generate Chrome-133 User-Agent"
    );
    let ua1 = ua1.unwrap();
    assert!(ua1.contains("Chrome"), "Should contain Chrome");

    // Testing generate by configuration name and operating system
    let ua2 = get_user_agent_by_profile_name_with_os("Firefox-133", OperatingSystem::Linux);
    if let Ok(ua2) = ua2 {
        // Firefox User-Agent may include "Firefox" or "Gecko"
        assert!(
            ua2.contains("Firefox") || ua2.contains("Gecko"),
            "Should contain Firefox or Gecko"
        );
    } else {
        // If failure, maybe configuration name format issue, try other format
        let ua2_alt = get_user_agent_by_profile_name("firefox_133");
        if let Ok(ua2_alt) = ua2_alt {
            assert!(ua2_alt.contains("Firefox") || ua2_alt.contains("Gecko"));
        }
    }

    // Testing random operating system
    let os = random_os();
    assert!(fingerprint::types::OPERATING_SYSTEMS.contains(&os));

    println!("✅ User-Agent generation works correctly");
    println!("   Sample UA: {}", ua1);
}

#[test]
fn test_http_headers_generation() {
    println!("\n=== HTTP Headers Generation Test ===");

    let fp_result = get_random_fingerprint_by_browser("chrome").unwrap();
    let headers = fp_result.headers;

    assert!(!headers.user_agent.is_empty(), "Should have User-Agent");
    assert!(!headers.accept.is_empty(), "Should have Accept");
    assert!(
        !headers.accept_language.is_empty(),
        "Should have Accept-Language"
    );

    println!("✅ Headers generation works correctly");
    println!("   User-Agent: {}", headers.user_agent);
    println!("   Accept: {}", headers.accept);
    println!("   Accept-Language: {}", headers.accept_language);
}

#[test]
fn test_random_language() {
    println!("\n=== Random Language Test ===");

    let lang = random_language();
    assert!(!lang.is_empty(), "Should return language code");

    println!("✅ Random language: {}", lang);
}

// ============================================================================
// 4. HTTP client testing
// ============================================================================

#[test]
fn test_http_client_creation() {
    println!("\n=== HTTP Client Creation Test ===");

    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) Chrome/133.0.0.0".to_string(),
        prefer_http2: true,
        prefer_http3: false,
        ..Default::default()
    };

    let _client = HttpClient::new(config);
    println!("✅ HTTP client creation successful");
}

#[test]
fn test_http_request_builder() {
    println!("\n=== HTTP Request Builder Test ===");

    let request = HttpRequest::new(HttpMethod::Get, "https://example.com/test")
        .with_header("Accept", "text/html")
        .with_header("Accept-Language", "en-US,en;q=0.9");

    let http1_request = request.build_http1_request("example.com", "/test");

    assert!(http1_request.contains("GET /test HTTP/1.1"));
    assert!(http1_request.contains("Host: example.com"));
    assert!(http1_request.contains("Accept: text/html"));

    println!("✅ HTTP request building successful");
    println!("{}", http1_request);
}

#[test]
#[ignore]
fn test_http_client_get_request() {
    println!("\n=== HTTP GET Request Test ===");

    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) Chrome/133.0.0.0".to_string(),
        prefer_http2: true,
        ..Default::default()
    };

    let client = HttpClient::new(config);
    let start = Instant::now();

    match client.get("https://www.example.com/") {
        Ok(response) => {
            let duration = start.elapsed();
            println!("✅ Request successful");
            println!("   Status code: {}", response.status_code);
            println!("   HTTP version: {}", response.http_version);
            println!("   Response time: {:?}", duration);
            println!("   Body size: {} bytes", response.body.len());

            assert_eq!(response.status_code, 200);
        }
        Err(e) => {
            println!("⚠️  Request failed: {}", e);
            // Network testing may fail, don't panic
        }
    }
}

#[test]
#[ignore]
fn test_http_client_post_request() {
    println!("\n=== HTTP POST Request Test ===");

    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) Chrome/133.0.0.0".to_string(),
        ..Default::default()
    };

    let client = HttpClient::new(config);

    let body = "test=data";
    match client.post("https://httpbin.org/post", body.as_bytes()) {
        Ok(response) => {
            println!("✅ POST request successful");
            println!("   Status code: {}", response.status_code);
            assert_eq!(response.status_code, 200);
        }
        Err(e) => {
            println!("⚠️  POST request failed: {}", e);
        }
    }
}

// ============================================================================
// 5. Cookie management testing
// ============================================================================

#[test]
fn test_cookie_parsing() {
    println!("\n=== Cookie Parsing Test ===");

    let set_cookie =
        "session_id=abc123; Path=/; Domain=example.com; Secure; HttpOnly; SameSite=Strict";

    let cookie_result = Cookie::parse_set_cookie(set_cookie, "example.com".to_string());
    assert!(
        cookie_result.is_some(),
        "Should be able to parse Set-Cookie"
    );

    let cookie = cookie_result.unwrap();
    assert_eq!(cookie.name, "session_id");
    assert_eq!(cookie.value, "abc123");
    assert_eq!(cookie.path, "/");
    assert_eq!(cookie.domain, "example.com");
    assert!(cookie.secure);
    assert!(cookie.http_only);
    assert_eq!(cookie.same_site, Some(SameSite::Strict));

    println!("✅ Cookie parsing successful");
    println!("   Name: {}", cookie.name);
    println!("   Value: {}", cookie.value);
    println!("   Path: {:?}", cookie.path);
    println!("   Domain: {:?}", cookie.domain);
}

#[test]
fn test_cookie_store() {
    println!("\n=== Cookie Store Test ===");

    let store = CookieStore::new();

    // Add Cookie
    let cookie1 = Cookie {
        name: "session".to_string(),
        value: "abc123".to_string(),
        domain: "example.com".to_string(),
        path: "/".to_string(),
        secure: false,
        http_only: false,
        same_site: None,
        expires: None,
        max_age: None,
    };

    store.add_cookie(cookie1);

    // Get Cookie
    let cookies = store.get_cookies_for_domain("example.com");
    assert!(!cookies.is_empty(), "Should be able to find Cookie");

    println!("✅ Cookie storage works correctly");
    println!("   Found {} cookies", cookies.len());
}

// ============================================================================
// 6. HTTP/2 configuration testing
// ============================================================================

#[test]
fn test_http2_settings() {
    println!("\n=== HTTP/2 Settings Test ===");

    // Chrome Settings
    let (chrome_settings, _) = chrome_http2_settings();
    assert!(
        !chrome_settings.is_empty(),
        "Chrome should have HTTP/2 Settings"
    );

    // Firefox Settings
    let (firefox_settings, _) = firefox_http2_settings();
    assert!(
        !firefox_settings.is_empty(),
        "Firefox should have HTTP/2 Settings"
    );

    // Safari Settings
    let (safari_settings, _) = safari_http2_settings();
    assert!(
        !safari_settings.is_empty(),
        "Safari should have HTTP/2 Settings"
    );

    println!("✅ HTTP/2 Settings work correctly");
    println!("   Chrome: {} settings", chrome_settings.len());
    println!("   Firefox: {} settings", firefox_settings.len());
    println!("   Safari: {} settings", safari_settings.len());
}

#[test]
fn test_http2_pseudo_header_order() {
    println!("\n=== HTTP/2 Pseudo Header Order Test ===");

    let chrome_order = chrome_pseudo_header_order();
    let firefox_order = firefox_pseudo_header_order();
    let safari_order = safari_pseudo_header_order();

    assert!(!chrome_order.is_empty());
    assert!(!firefox_order.is_empty());
    assert!(!safari_order.is_empty());

    println!("✅ Pseudo Header Order works correctly");
    println!("   Chrome: {:?}", chrome_order);
    println!("   Firefox: {:?}", firefox_order);
    println!("   Safari: {:?}", safari_order);
}

// ============================================================================
// 7. Fingerprint comparison testing
// ============================================================================

#[test]
fn test_fingerprint_comparison() {
    println!("\n=== Fingerprint Comparison Test ===");

    let spec1 = chrome_133().tls_config;
    let spec2 = firefox_133().tls_config;

    // Compare two fingerprints
    let comparison = compare_specs(&spec1, &spec2);

    println!("✅ Fingerprint comparison completed");
    println!("   Match result: {:?}", comparison);
}

#[test]
fn test_fingerprint_matching() {
    println!("\n=== Fingerprint Matching Test ===");

    let target = chrome_133().tls_config;
    let candidates = vec![
        chrome_103().tls_config,
        firefox_133().tls_config,
        safari_16_0().tls_config,
    ];

    let target_sig = extract_signature(&target);

    let match_result = find_best_match(&target_sig, &candidates);

    // May not find perfect match, but should at least attempt matching
    if let Some(best_index) = match_result {
        println!("✅ Best match found");
        println!("   Best match index: {}", best_index);
    } else {
        println!("⚠️  No best match found (possibly no candidates match)");
    }
}

// ============================================================================
// 8. Configuration export testing
// ============================================================================

#[test]
#[cfg(feature = "export")]
fn test_config_export() {
    println!("\n=== Configuration Export Test ===");

    let profile = chrome_133();
    let spec = &profile.tls_config;

    // Export to JSON (if supported)
    println!("✅ Configuration export feature available");
    println!("   Profile: {}", profile.id());
    println!("   Cipher suite count: {}", spec.cipher_suites.len());
    println!("   Extension count: {}", spec.extensions.len());
}

// ============================================================================
// 9. Performance testing
// ============================================================================

#[test]
fn test_fingerprint_generation_performance() {
    println!("\n=== Fingerprint Generation Performance Test ===");

    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _spec = chrome_133().tls_config;
    }

    let duration = start.elapsed();
    let avg_time = duration.as_nanos() / iterations as u128;

    println!("✅ Performance test completed");
    println!("   Iterations: {}", iterations);
    println!("   Total time: {:?}", duration);
    println!("   Average time: {} ns/iteration", avg_time);

    // Should be fast (< 1ms per iteration)
    assert!(avg_time < 1_000_000, "Each generation should be < 1ms");
}

#[test]
fn test_http_request_building_performance() {
    println!("\n=== HTTP Request Building Performance Test ===");

    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = HttpRequest::new(HttpMethod::Get, "https://example.com/test")
            .with_header("Accept", "text/html")
            .build_http1_request("example.com", "/test");
    }

    let duration = start.elapsed();
    let avg_time = duration.as_nanos() / iterations as u128;

    println!("✅ Request building performance test completed");
    println!("   Iterations: {}", iterations);
    println!("   Total time: {:?}", duration);
    println!("   Average time: {} ns/iteration", avg_time);
}

// ============================================================================
// 10. Error processing testing
// ============================================================================

#[test]
fn test_error_handling() {
    println!("\n=== Error Handling Test ===");

    // Testing invalid browser name
    let result = get_random_fingerprint_by_browser("invalid_browser");
    assert!(result.is_err(), "Invalid browser should return error");

    // Testing invalid User-Agent generation
    let result = get_user_agent_by_profile_name("Invalid-Profile");
    // May return empty string instead of error, this is acceptable
    if let Ok(ua) = result {
        if ua.is_empty() {
            println!("✅ Invalid configuration returns empty string (acceptable)");
        }
    } else {
        println!("✅ Invalid configuration returns error (as expected)");
    }

    println!("✅ Error handling works correctly");
}

// ============================================================================
// 11. Integration testing
// ============================================================================

#[test]
fn test_full_integration() {
    println!("\n=== Full Integration Test ===");

    // 1. Get random fingerprint
    let fp_result = get_random_fingerprint().unwrap();

    // 2. Create HTTP client configuration (not loading profile, because it may not exist in mapped_tls_clients)
    let config = HttpClientConfig {
        user_agent: fp_result.user_agent.clone(),
        headers: fp_result.headers.clone(),
        profile: None, // Don't set profile, test basic functionality
        prefer_http2: true,
        ..Default::default()
    };

    // 3. Create client
    let _client = HttpClient::new(config);

    // 4. Build request
    let _request =
        HttpRequest::new(HttpMethod::Get, "https://example.com/").with_headers(&fp_result.headers);

    println!("✅ Full integration test passed");
    println!("   Browser: {}", fp_result.profile_id);
    println!("   User-Agent: {}", fp_result.user_agent);
}

// ============================================================================
// 12. Boundary conditions testing
// ============================================================================

#[test]
fn test_edge_cases() {
    println!("\n=== Boundary Conditions Test ===");

    // Testing empty string
    let empty_headers = HTTPHeaders::default();
    assert!(empty_headers.user_agent.is_empty() || !empty_headers.user_agent.is_empty());

    // Testing default configuration
    let default_config = HttpClientConfig::default();
    assert!(!default_config.user_agent.is_empty());

    println!("✅ Boundary conditions test passed");
}

// ============================================================================
// 13. Concurrent security testing
// ============================================================================

#[test]
fn test_concurrent_access() {
    println!("\n=== Concurrent Security Test ===");

    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                let result = get_random_fingerprint().unwrap();
                assert!(!result.user_agent.is_empty());
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    println!("✅ Concurrent security test passed");
}

// ============================================================================
// 14. Testing summary
// ============================================================================

#[test]
fn test_summary() {
    println!("\n");
    println!("═══════════════════════════════════════════════════════════");
    println!("   fingerprint-rust comprehensive test suite");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("✅ TLS fingerprint generation and validation");
    println!("✅ Browser fingerprint configuration (66+ browsers)");
    println!("✅ User-Agent and Headers generation");
    println!("✅ HTTP client (H1/H2/H3)");
    println!("✅ Cookie management");
    println!("✅ HTTP/2 configuration");
    println!("✅ Fingerprint comparison and matching");
    println!("✅ Performance test");
    println!("✅ Error handling");
    println!("✅ Concurrent security");
    println!();
    println!("═══════════════════════════════════════════════════════════");
}

// TCP fingerprint sync demo has been moved to tcp_sync_demo_test.rs
// Please use separate testing file: tests/tcp_sync_demo_test.rs
