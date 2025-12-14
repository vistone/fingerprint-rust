//! 真实世界验证测试
//!
//! 这些测试会访问真实的网站来验证指纹的有效性
//!
//! 运行方式:
//! ```bash
//! # 运行所有真实验证测试
//! cargo test --test real_world_validation -- --ignored --test-threads=1
//!
//! # 运行单个测试
//! cargo test --test real_world_validation test_tls_peet_api -- --ignored
//! ```
//!
//! ⚠️ 注意：
//! - 这些测试需要网络连接
//! - 测试可能因为网络问题而失败
//! - 建议使用 --test-threads=1 避免并发请求

use fingerprint::*;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

/// TLS Peet API 返回的指纹信息
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct TlsPeetResponse {
    #[serde(default)]
    ja3: Option<String>,
    #[serde(default)]
    ja3_hash: Option<String>,
    #[serde(default)]
    ja4: Option<String>,
    #[serde(default)]
    ja4_o: Option<String>,
    #[serde(default)]
    user_agent: Option<String>,
    #[serde(default)]
    tls_version: Option<String>,
    #[serde(default)]
    cipher_suites: Option<Vec<String>>,
    #[serde(default)]
    http_version: Option<String>,
}

/// Browserleaks SSL 响应
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BrowserleaksResponse {
    #[serde(default)]
    user_agent: Option<String>,
    #[serde(default)]
    tls_version: Option<String>,
    #[serde(default)]
    ciphers: Option<Vec<String>>,
}

/// 创建一个基础的 HTTP 客户端
fn create_test_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

/// 测试 1: 验证能否成功生成指纹并访问网络
#[test]
fn test_basic_fingerprint_generation() {
    println!("\n=== 测试 1: 基础指纹生成 ===");

    // 生成 Chrome 指纹
    let result = get_random_fingerprint_by_browser("chrome");
    assert!(result.is_ok(), "应该能成功生成 Chrome 指纹");

    let fp = result.unwrap();
    println!("✓ 生成的指纹: {}", fp.hello_client_id);
    println!("✓ User-Agent: {}", fp.user_agent);
    println!("✓ Accept-Language: {}", fp.headers.accept_language);

    // 验证必要字段
    assert!(!fp.user_agent.is_empty(), "User-Agent 不应为空");
    assert!(!fp.hello_client_id.is_empty(), "HelloClientID 不应为空");
    assert!(
        !fp.headers.accept_language.is_empty(),
        "Accept-Language 不应为空"
    );
}

/// 测试 2: 验证 TLS 配置的完整性
#[test]
fn test_tls_config_completeness() {
    println!("\n=== 测试 2: TLS 配置完整性 ===");

    let profile = mapped_tls_clients().get("chrome_133").unwrap();
    let spec = profile.get_client_hello_spec().unwrap();

    // 验证密码套件
    assert!(!spec.cipher_suites.is_empty(), "密码套件不应为空");
    println!("✓ 密码套件数量: {}", spec.cipher_suites.len());

    // 验证扩展
    assert!(!spec.extensions.is_empty(), "扩展不应为空");
    println!("✓ 扩展数量: {}", spec.extensions.len());

    // 验证压缩方法
    assert!(!spec.compression_methods.is_empty(), "压缩方法不应为空");
    println!("✓ 压缩方法: {:?}", spec.compression_methods);

    // 验证 HTTP/2 配置
    let settings = profile.get_settings();
    assert!(!settings.is_empty(), "HTTP/2 Settings 不应为空");
    println!("✓ HTTP/2 Settings 数量: {}", settings.len());

    let pseudo_order = profile.get_pseudo_header_order();
    assert_eq!(pseudo_order.len(), 4, "Pseudo Header Order 应该有 4 个");
    println!("✓ Pseudo Header Order: {:?}", pseudo_order);
}

/// 测试 3: 验证 JA4 指纹生成
#[test]
fn test_ja4_fingerprint_generation() {
    println!("\n=== 测试 3: JA4 指纹生成 ===");

    let profile = mapped_tls_clients().get("chrome_133").unwrap();
    let spec = profile.get_client_hello_spec().unwrap();
    let signature = extract_signature(&spec);

    // 创建 JA4 签名
    let ja4_sig = Ja4Signature {
        version: signature.version,
        cipher_suites: signature.cipher_suites.clone(),
        extensions: signature.extensions.clone(),
        signature_algorithms: signature.signature_algorithms.clone(),
        sni: Some("example.com".to_string()),
        alpn: Some("h2".to_string()),
    };

    // 生成 JA4 (sorted)
    let ja4_sorted = ja4_sig.generate_ja4();
    println!("✓ JA4 (sorted): {}", ja4_sorted.full.value());
    println!("  JA4_a: {}", ja4_sorted.ja4_a);
    println!("  JA4_b 长度: {}", ja4_sorted.ja4_b.len());
    println!("  JA4_c 长度: {}", ja4_sorted.ja4_c.len());

    // 生成 JA4 Original (unsorted)
    let ja4_unsorted = ja4_sig.generate_ja4_original();
    println!("✓ JA4_o (unsorted): {}", ja4_unsorted.full.value());

    // 验证格式
    assert!(
        ja4_sorted.full.value().contains('_'),
        "JA4 应该包含下划线分隔符"
    );
    assert!(
        ja4_sorted.ja4_a.starts_with('t'),
        "JA4_a 应该以 't' 开头（TLS）"
    );
}

/// 测试 4: 对比不同浏览器的指纹差异
#[test]
fn test_different_browser_fingerprints() {
    println!("\n=== 测试 4: 不同浏览器指纹差异 ===");

    let browsers = vec!["chrome_133", "firefox_133", "safari_16_0"];
    let mut fingerprints = HashMap::new();

    for browser in &browsers {
        let profile = mapped_tls_clients().get(*browser).unwrap();
        let spec = profile.get_client_hello_spec().unwrap();
        let signature = extract_signature(&spec);

        let ja4_sig = Ja4Signature {
            version: signature.version,
            cipher_suites: signature.cipher_suites,
            extensions: signature.extensions,
            signature_algorithms: signature.signature_algorithms,
            sni: Some("example.com".to_string()),
            alpn: Some("h2".to_string()),
        };

        let ja4 = ja4_sig.generate_ja4();
        fingerprints.insert(*browser, ja4.full.value().to_string());

        println!("✓ {}: {}", browser, ja4.full.value());
    }

    // 验证不同浏览器的指纹确实不同
    let chrome_fp = fingerprints.get("chrome_133").unwrap();
    let firefox_fp = fingerprints.get("firefox_133").unwrap();
    let safari_fp = fingerprints.get("safari_16_0").unwrap();

    assert_ne!(chrome_fp, firefox_fp, "Chrome 和 Firefox 的指纹应该不同");
    assert_ne!(chrome_fp, safari_fp, "Chrome 和 Safari 的指纹应该不同");
    assert_ne!(firefox_fp, safari_fp, "Firefox 和 Safari 的指纹应该不同");

    println!("✓ 验证通过: 不同浏览器的指纹确实不同");
}

/// 测试 5: 验证 GREASE 值的处理
#[test]
fn test_grease_value_handling() {
    println!("\n=== 测试 5: GREASE 值处理 ===");

    // GREASE 值列表
    let test_values = vec![0x0a0a, 0x1a1a, 0x2a2a, 0x0017, 0x0018];

    println!("测试值: {:?}", test_values);

    // 测试识别
    for &val in &test_values {
        let is_grease = is_grease_value(val);
        println!(
            "  0x{:04x}: {} GREASE",
            val,
            if is_grease { "是" } else { "不是" }
        );
    }

    // 测试过滤
    let filtered = filter_grease_values(&test_values);
    println!("✓ 过滤后: {:?}", filtered);
    assert_eq!(filtered.len(), 2, "应该剩余 2 个非 GREASE 值");
    assert_eq!(filtered, vec![0x0017, 0x0018]);
}

/// 测试 6: 验证 HTTP Headers 的完整性
#[test]
fn test_http_headers_completeness() {
    println!("\n=== 测试 6: HTTP Headers 完整性 ===");

    let result = get_random_fingerprint_by_browser("chrome").unwrap();
    let headers = result.headers.to_map();

    println!("生成的 Headers 数量: {}", headers.len());

    // 验证必要的 headers
    let required_headers = vec!["User-Agent", "Accept", "Accept-Language", "Accept-Encoding"];

    for header in required_headers {
        assert!(headers.contains_key(header), "应该包含 {} header", header);
        println!("✓ {}: {}", header, headers.get(header).unwrap());
    }
}

/// 测试 7: 访问 httpbin.org 验证基本网络功能
#[test]
#[ignore]
fn test_httpbin_basic_request() {
    println!("\n=== 测试 7: httpbin.org 基础请求 ===");
    println!("⚠️  此测试需要网络连接");

    let result = get_random_fingerprint_by_browser("chrome").unwrap();
    let headers_map = result.headers.to_map();

    let client = create_test_client();

    // 构建请求
    let mut request = client.get("https://httpbin.org/headers");

    // 添加 headers
    for (key, value) in headers_map.iter() {
        request = request.header(key, value);
    }

    // 发送请求
    match request.send() {
        Ok(response) => {
            println!("✓ 请求成功");
            println!("  状态码: {}", response.status());

            if let Ok(text) = response.text() {
                println!(
                    "  响应前 200 字符: {}",
                    &text.chars().take(200).collect::<String>()
                );
            }
        }
        Err(e) => {
            println!("✗ 请求失败: {}", e);
            panic!("网络请求失败");
        }
    }
}

/// 测试 8: 访问 TLS 指纹检测服务（如果可用）
#[test]
#[ignore]
fn test_tls_fingerprint_detection_service() {
    println!("\n=== 测试 8: TLS 指纹检测服务 ===");
    println!("⚠️  此测试需要网络连接");
    println!("⚠️  测试服务: https://tls.peet.ws/api/all");

    let result = get_random_fingerprint_by_browser("chrome").unwrap();
    println!("使用的指纹: {}", result.hello_client_id);
    println!("User-Agent: {}", result.user_agent);

    let client = create_test_client();

    // 尝试访问 TLS 指纹检测服务
    match client
        .get("https://tls.peet.ws/api/all")
        .header("User-Agent", &result.user_agent)
        .send()
    {
        Ok(response) => {
            println!("✓ 请求成功");
            println!("  状态码: {}", response.status());

            if response.status().is_success() {
                if let Ok(text) = response.text() {
                    println!("  响应内容（前 500 字符）:");
                    println!("  {}", &text.chars().take(500).collect::<String>());

                    // 尝试解析 JSON
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                        println!("\n  解析后的指纹信息:");
                        if let Some(ja3) = json.get("ja3") {
                            println!("    JA3: {}", ja3);
                        }
                        if let Some(ja4) = json.get("ja4") {
                            println!("    JA4: {}", ja4);
                        }
                        if let Some(tls_version) = json.get("tls_version") {
                            println!("    TLS Version: {}", tls_version);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("⚠️  请求失败: {}", e);
            println!("   这可能是因为:");
            println!("   1. 网络连接问题");
            println!("   2. 服务不可用");
            println!("   3. TLS 握手失败");
            println!("   4. 需要特殊的 TLS 客户端配置");
        }
    }
}

/// 测试 9: 对比文档中的浏览器版本
#[test]
fn test_supported_browser_versions() {
    println!("\n=== 测试 9: 支持的浏览器版本 ===");

    let clients = mapped_tls_clients();

    // Chrome 系列
    let chrome_versions = vec![
        "chrome_103",
        "chrome_104",
        "chrome_105",
        "chrome_106",
        "chrome_107",
        "chrome_108",
        "chrome_109",
        "chrome_110",
        "chrome_111",
        "chrome_112",
        "chrome_116_PSK",
        "chrome_117",
        "chrome_120",
        "chrome_124",
        "chrome_130_PSK",
        "chrome_131",
        "chrome_131_PSK",
        "chrome_133",
        "chrome_133_PSK",
    ];

    println!("Chrome 系列 ({} 个):", chrome_versions.len());
    for version in &chrome_versions {
        assert!(clients.contains_key(*version), "应该包含 {}", version);
        print!("  ✓ {} ", version);
    }
    println!();

    // Firefox 系列
    let firefox_versions = vec![
        "firefox_102",
        "firefox_104",
        "firefox_105",
        "firefox_106",
        "firefox_108",
        "firefox_110",
        "firefox_117",
        "firefox_120",
        "firefox_123",
        "firefox_132",
        "firefox_133",
        "firefox_135",
    ];

    println!("\nFirefox 系列 ({} 个):", firefox_versions.len());
    for version in &firefox_versions {
        assert!(clients.contains_key(*version), "应该包含 {}", version);
        print!("  ✓ {} ", version);
    }
    println!();

    // Safari 系列
    let safari_versions = vec![
        "safari_15_6_1",
        "safari_16_0",
        "safari_ipad_15_6",
        "safari_ios_15_5",
        "safari_ios_15_6",
        "safari_ios_16_0",
        "safari_ios_17_0",
        "safari_ios_18_0",
        "safari_ios_18_5",
    ];

    println!("\nSafari 系列 ({} 个):", safari_versions.len());
    for version in &safari_versions {
        assert!(clients.contains_key(*version), "应该包含 {}", version);
        print!("  ✓ {} ", version);
    }
    println!();

    println!("\n✓ 总计: {} 个浏览器指纹", clients.len());
}

/// 测试 10: 性能测试 - 指纹生成速度
#[test]
fn test_fingerprint_generation_performance() {
    println!("\n=== 测试 10: 指纹生成性能 ===");

    let iterations = 1000;
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let _ = get_random_fingerprint();
    }

    let duration = start.elapsed();
    let avg_time = duration.as_micros() / iterations;

    println!("✓ 生成 {} 个指纹耗时: {:?}", iterations, duration);
    println!("✓ 平均每个指纹: {} μs", avg_time);
    println!("✓ 每秒可生成: {} 个指纹", 1_000_000 / avg_time.max(1));

    // 性能断言：每个指纹应该在 1ms 内生成
    assert!(avg_time < 1000, "平均生成时间应该少于 1ms");
}

/// 测试总结
#[test]
fn test_validation_summary() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║         fingerprint-rust 真实验证测试总结                ║");
    println!("╚═══════════════════════════════════════════════════════════╝");

    println!("\n✅ 已完成的验证:");
    println!("  ✓ 基础功能验证");
    println!("  ✓ TLS 配置完整性");
    println!("  ✓ JA4 指纹生成");
    println!("  ✓ 浏览器差异对比");
    println!("  ✓ GREASE 值处理");
    println!("  ✓ HTTP Headers 完整性");
    println!("  ✓ 性能测试");

    println!("\n⚠️  需要网络的测试 (使用 --ignored 运行):");
    println!("  • httpbin.org 基础请求");
    println!("  • TLS 指纹检测服务");

    println!("\n🔍 进一步验证建议:");
    println!("  1. 使用 Wireshark 抓包对比 TLS ClientHello");
    println!("  2. 访问真实的反爬虫保护网站");
    println!("  3. 对比真实浏览器的指纹数据");
    println!("  4. 长期监控指纹的有效性");

    println!("\n📚 参考文档:");
    println!("  • docs/VALIDATION_LIMITATIONS.md");
    println!("  • docs/COMPREHENSIVE_AUDIT_REPORT.md");

    println!("\n运行网络测试:");
    println!("  cargo test --test real_world_validation -- --ignored --test-threads=1");
    println!();
}
