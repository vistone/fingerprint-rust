//! Google Earth API 完整testing
//!
//! testingaddress: https://kh.google.com/rt/earth/PlanetoidMetadata
//! testingall浏览器fingerprintandallprotocol（HTTP/1.1、HTTP/2、HTTP/3）
//!
//! run方式:
//! ```bash
//! # testingall浏览器fingerprintandprotocol
//! cargo test --test google_earth_test --features rustls-tls,http2,http3 -- --ignored --nocapture
//!
//! # testing特定protocol
//! cargo test --test google_earth_test test_google_earth_http1 --features rustls-tls -- --ignored
//! cargo test --test google_earth_test test_google_earth_http2 --features rustls-tls,http2 -- --ignored
//! cargo test --test google_earth_test test_google_earth_http3 --features rustls-tls,http3 -- --ignored
//! ```

use fingerprint::{get_user_agent_by_profile_name, HttpClient, HttpClientConfig};
use std::time::Instant;

const TEST_URL: &str = "https://kh.google.com/rt/earth/PlanetoidMetadata";

// ============================================================================
// 1. 单protocoltesting
// ============================================================================

/// testing HTTP/1.1
#[test]
#[ignore] // requirenetworkconnect
fn test_google_earth_http1() {
    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let config = HttpClientConfig {
        user_agent,
        prefer_http2: false,
        prefer_http3: false,
        ..Default::default()
    };

    let client = HttpClient::new(config);
    let start = Instant::now();

    match client.get(TEST_URL) {
        Ok(response) => {
            let elapsed = start.elapsed();
            assert!(
                response.is_success(),
                "预期成功响应，实际状态码: {}",
                response.status_code
            );
            assert_eq!(response.status_code, 200);
            assert!(!response.body.is_empty(), "响应体不应该为空");
            println!(
                "✅ HTTP/1.1: {} ({}ms)",
                response.status_code,
                elapsed.as_millis()
            );
        }
        Err(e) => panic!("❌ HTTP/1.1 测试失败: {}", e),
    }
}

/// testing HTTP/2
#[test]
#[cfg(feature = "http2")]
#[ignore] // requirenetworkconnect
fn test_google_earth_http2() {
    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let config = HttpClientConfig {
        user_agent,
        prefer_http2: true,
        prefer_http3: false,
        ..Default::default()
    };

    let client = HttpClient::new(config);
    let start = Instant::now();

    match client.get(TEST_URL) {
        Ok(response) => {
            let elapsed = start.elapsed();
            assert!(
                response.is_success(),
                "预期成功响应，实际状态码: {}",
                response.status_code
            );
            assert_eq!(response.status_code, 200);
            assert!(!response.body.is_empty(), "响应体不应该为空");
            assert!(
                response.http_version.contains("HTTP/2") || response.http_version.contains("h2"),
                "预期 HTTP/2 响应，实际: {}",
                response.http_version
            );
            println!(
                "✅ HTTP/2: {} ({}ms)",
                response.status_code,
                elapsed.as_millis()
            );
        }
        Err(e) => panic!("❌ HTTP/2 测试失败: {}", e),
    }
}

/// testing HTTP/3
#[test]
#[cfg(feature = "http3")]
#[ignore] // requirenetworkconnect
fn test_google_earth_http3() {
    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let config = HttpClientConfig {
        user_agent,
        prefer_http2: false,
        prefer_http3: true,
        ..Default::default()
    };

    let client = HttpClient::new(config);
    let start = Instant::now();

    match client.get(TEST_URL) {
        Ok(response) => {
            let elapsed = start.elapsed();
            assert!(
                response.is_success(),
                "预期成功响应，实际状态码: {}",
                response.status_code
            );
            assert_eq!(response.status_code, 200);
            assert!(!response.body.is_empty(), "响应体不应该为空");
            assert!(
                response.http_version.contains("HTTP/3")
                    || response.http_version.contains("h3")
                    || response.http_version.contains("quic"),
                "预期 HTTP/3 响应，实际: {}",
                response.http_version
            );
            println!(
                "✅ HTTP/3: {} ({}ms)",
                response.status_code,
                elapsed.as_millis()
            );
        }
        Err(e) => panic!("❌ HTTP/3 测试失败: {}", e),
    }
}

// ============================================================================
// 2. allprotocoltesting
// ============================================================================

/// testingallprotocol（HTTP/1.1、HTTP/2、HTTP/3）
#[test]
#[ignore] // requirenetworkconnect
fn test_google_earth_all_protocols() {
    println!("\n=== Google Earth API 全协议测试 ===");
    println!("URL: {}\n", TEST_URL);

    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let mut results = Vec::new();

    // testing HTTP/1.1
    println!("🔹 测试 HTTP/1.1");
    let config_h1 = HttpClientConfig {
        user_agent: user_agent.clone(),
        prefer_http2: false,
        prefer_http3: false,
        ..Default::default()
    };
    let client_h1 = HttpClient::new(config_h1);
    let start = Instant::now();
    match client_h1.get(TEST_URL) {
        Ok(response) => {
            let elapsed = start.elapsed();
            println!(
                "  ✅ HTTP/1.1: {} ({}ms)",
                response.status_code,
                elapsed.as_millis()
            );
            results.push(("HTTP/1.1", true, response.status_code));
        }
        Err(e) => {
            println!("  ❌ HTTP/1.1: {}", e);
            results.push(("HTTP/1.1", false, 0));
        }
    }

    // testing HTTP/2
    #[cfg(feature = "http2")]
    {
        println!("\n🔹 测试 HTTP/2");
        let config_h2 = HttpClientConfig {
            user_agent: user_agent.clone(),
            prefer_http2: true,
            prefer_http3: false,
            ..Default::default()
        };
        let client_h2 = HttpClient::new(config_h2);
        let start = Instant::now();
        match client_h2.get(TEST_URL) {
            Ok(response) => {
                let elapsed = start.elapsed();
                println!(
                    "  ✅ HTTP/2: {} ({}ms)",
                    response.status_code,
                    elapsed.as_millis()
                );
                results.push(("HTTP/2", true, response.status_code));
            }
            Err(e) => {
                println!("  ❌ HTTP/2: {}", e);
                results.push(("HTTP/2", false, 0));
            }
        }
    }

    // testing HTTP/3
    #[cfg(feature = "http3")]
    {
        println!("\n🔹 测试 HTTP/3");
        let config_h3 = HttpClientConfig {
            user_agent,
            prefer_http2: false,
            prefer_http3: true,
            ..Default::default()
        };
        let client_h3 = HttpClient::new(config_h3);
        let start = Instant::now();
        match client_h3.get(TEST_URL) {
            Ok(response) => {
                let elapsed = start.elapsed();
                println!(
                    "  ✅ HTTP/3: {} ({}ms)",
                    response.status_code,
                    elapsed.as_millis()
                );
                results.push(("HTTP/3", true, response.status_code));
            }
            Err(e) => {
                println!("  ❌ HTTP/3: {}", e);
                results.push(("HTTP/3", false, 0));
            }
        }
    }

    // 汇总结果
    println!("\n=== 测试结果汇总 ===");
    let success_count = results.iter().filter(|(_, success, _)| *success).count();
    let total_count = results.len();

    for (protocol, success, status) in &results {
        if *success {
            println!("✅ {}: 状态码 {}", protocol, status);
        } else {
            println!("❌ {}: 失败", protocol);
        }
    }

    println!("\n📊 成功率: {}/{}", success_count, total_count);
    assert_eq!(
        success_count, total_count,
        "部分协议测试失败！预期 {} 个通过，实际 {} 个通过",
        total_count, success_count
    );
}

// ============================================================================
// 3. all浏览器fingerprinttesting
// ============================================================================

/// testingall浏览器fingerprint（核心浏览器）
#[test]
#[ignore] // requirenetworkconnect
fn test_google_earth_all_browsers() {
    println!("\n=== Google Earth API 所有浏览器指纹测试 ===");

    let browsers = vec![
        ("chrome_103", "Chrome 103"),
        ("chrome_133", "Chrome 133"),
        ("firefox_133", "Firefox 133"),
        ("safari_16_0", "Safari 16.0"),
        ("opera_91", "Opera 91"),
    ];

    let mut results = Vec::new();

    for (profile_name, browser_name) in browsers {
        println!("\n🔹 测试 {}", browser_name);

        let user_agent = get_user_agent_by_profile_name(profile_name)
            .unwrap_or_else(|_| "Mozilla/5.0".to_string());

        let config = HttpClientConfig {
            user_agent,
            prefer_http2: false,
            prefer_http3: false,
            ..Default::default()
        };

        let client = HttpClient::new(config);
        let start = Instant::now();

        match client.get(TEST_URL) {
            Ok(response) => {
                let elapsed = start.elapsed();
                println!(
                    "  ✅ {}: {} ({}ms)",
                    browser_name,
                    response.status_code,
                    elapsed.as_millis()
                );
                results.push((browser_name, true, response.status_code));
            }
            Err(e) => {
                println!("  ❌ {}: {}", browser_name, e);
                results.push((browser_name, false, 0));
            }
        }
    }

    // 汇总结果
    println!("\n=== 测试结果汇总 ===");
    let success_count = results.iter().filter(|(_, success, _)| *success).count();
    let total_count = results.len();

    for (browser, success, status) in &results {
        if *success {
            println!("✅ {}: 状态码 {}", browser, status);
        } else {
            println!("❌ {}: 失败", browser);
        }
    }

    println!("\n📊 成功率: {}/{}", success_count, total_count);
    assert_eq!(
        success_count, total_count,
        "部分浏览器测试失败！预期 {} 个通过，实际 {} 个通过",
        total_count, success_count
    );
}
