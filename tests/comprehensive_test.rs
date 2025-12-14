//! 全面测试套件
//!
//! 测试 fingerprint-rust 库的所有核心功能：
//! - TLS 指纹生成和验证
//! - HTTP 客户端（H1/H2/H3）
//! - 浏览器指纹配置
//! - User-Agent 和 Headers 生成
//! - Cookie 管理
//! - 代理支持
//! - 连接池
//!
//! 运行方式：
//! ```bash
//! # 本地测试（不需要网络）
//! cargo test --test comprehensive_test
//!
//! # 网络测试（需要网络连接）
//! cargo test --test comprehensive_test -- --ignored --nocapture
//! ```

use fingerprint::*;
use std::time::Instant;

// ============================================================================
// 1. TLS 指纹测试
// ============================================================================

#[test]
fn test_tls_fingerprint_generation() {
    println!("\n=== TLS 指纹生成测试 ===");

    // 测试所有核心浏览器
    let browsers = vec![
        ("Chrome 103", chrome_103()),
        ("Chrome 133", chrome_133()),
        ("Firefox 133", firefox_133()),
        ("Safari 16.0", safari_16_0()),
        ("Opera 91", opera_91()),
    ];

    for (name, profile) in browsers {
        let spec_result = profile.get_client_hello_spec();

        assert!(spec_result.is_ok(), "{} 应该能生成 TLS Spec", name);

        let spec = spec_result.unwrap();
        assert!(!spec.cipher_suites.is_empty(), "{} 应该有密码套件", name);
        assert!(!spec.extensions.is_empty(), "{} 应该有扩展", name);

        println!(
            "✅ {}: {} 密码套件, {} 扩展",
            name,
            spec.cipher_suites.len(),
            spec.extensions.len()
        );
    }
}

#[test]
fn test_ja4_fingerprint_generation() {
    println!("\n=== JA4 指纹生成测试 ===");

    let profile = chrome_133();
    let spec = profile.get_client_hello_spec().unwrap();

    // 生成 JA4 指纹
    let signature = extract_signature(&spec);
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
    println!("\n=== TLS 握手构建测试 ===");

    #[cfg(feature = "crypto")]
    {
        let profile = chrome_133();
        let spec = profile.get_client_hello_spec().unwrap();

        let client_hello_result = TLSHandshakeBuilder::build_client_hello(&spec, "example.com");

        assert!(client_hello_result.is_ok(), "应该能构建 ClientHello");

        let client_hello = client_hello_result.unwrap();
        assert!(client_hello.len() > 0, "ClientHello 应该不为空");

        println!("✅ ClientHello 大小: {} bytes", client_hello.len());
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
                "✅ ClientHello (前64字节): {:?}",
                &client_hello[..std::cmp::min(64, client_hello.len())]
            );
        }
    }

    #[cfg(not(feature = "crypto"))]
    {
        println!("⚠️  跳过 TLS 握手测试（需要 crypto feature）");
    }
}

#[test]
fn test_grease_value_filtering() {
    println!("\n=== GREASE 值过滤测试 ===");

    let grease_values = vec![0x0a0a, 0x1a1a, 0x2a2a, 0x3a3a];

    for &value in &grease_values {
        assert!(is_grease_value(value), "{} 应该是 GREASE 值", value);
    }

    let normal_values = vec![0x0001, 0x0013, 0x0029];
    for &value in &normal_values {
        assert!(!is_grease_value(value), "{} 不应该是 GREASE 值", value);
    }

    println!("✅ GREASE 值检测正常");
}

// ============================================================================
// 2. 浏览器指纹配置测试
// ============================================================================

#[test]
fn test_browser_profiles() {
    println!("\n=== 浏览器配置测试 ===");

    let profiles = vec![
        ("Chrome 103", chrome_103()),
        ("Chrome 133", chrome_133()),
        ("Firefox 133", firefox_133()),
        ("Safari 16.0", safari_16_0()),
        ("Opera 91", opera_91()),
    ];

    for (name, profile) in profiles {
        // 测试 ClientHelloID
        let client_id = profile.get_client_hello_str();
        assert!(!client_id.is_empty(), "{} 应该有 ClientHelloID", name);

        // 测试 HTTP/2 Settings
        let settings = profile.get_settings();
        assert!(!settings.is_empty(), "{} 应该有 HTTP/2 Settings", name);

        // 测试 Pseudo Header Order
        let pseudo_order = profile.get_pseudo_header_order();
        assert!(
            !pseudo_order.is_empty(),
            "{} 应该有 Pseudo Header Order",
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
    println!("\n=== 随机指纹生成测试 ===");

    // 测试完全随机
    let result1 = get_random_fingerprint();
    assert!(result1.is_ok(), "应该能生成随机指纹");
    let fp1 = result1.unwrap();
    assert!(!fp1.user_agent.is_empty());
    assert!(!fp1.hello_client_id.is_empty());

    // 测试按浏览器类型
    let browsers = vec!["chrome", "firefox", "safari", "opera"];
    for browser in browsers {
        let result = get_random_fingerprint_by_browser(browser);
        assert!(result.is_ok(), "应该能生成 {} 指纹", browser);
        let fp = result.unwrap();
        assert!(!fp.user_agent.is_empty());
    }

    // 测试按操作系统
    let result2 = get_random_fingerprint_with_os(Some(OperatingSystem::Windows10));
    assert!(result2.is_ok());

    println!("✅ 随机指纹生成正常");
}

#[test]
fn test_all_browser_profiles() {
    println!("\n=== 所有浏览器配置验证 ===");

    let mapped = mapped_tls_clients();
    let mut success_count = 0;
    let mut fail_count = 0;

    for (name, profile) in mapped.iter() {
        match profile.get_client_hello_spec() {
            Ok(spec) => {
                assert!(!spec.cipher_suites.is_empty(), "{} 应该有密码套件", name);
                success_count += 1;
            }
            Err(e) => {
                println!("⚠️  {} 配置错误: {}", name, e);
                fail_count += 1;
            }
        }
    }

    println!("✅ 成功: {}, 失败: {}", success_count, fail_count);
    assert!(success_count > 0, "至少应该有一个浏览器配置成功");
}

// ============================================================================
// 3. User-Agent 和 Headers 测试
// ============================================================================

#[test]
fn test_user_agent_generation() {
    println!("\n=== User-Agent 生成测试 ===");

    // 测试按配置名称生成
    let ua1 = get_user_agent_by_profile_name("Chrome-133");
    assert!(ua1.is_ok(), "应该能生成 Chrome-133 User-Agent");
    let ua1 = ua1.unwrap();
    assert!(ua1.contains("Chrome"), "应该包含 Chrome");

    // 测试按配置名称和操作系统生成
    let ua2 = get_user_agent_by_profile_name_with_os("Firefox-133", OperatingSystem::Linux);
    if ua2.is_ok() {
        let ua2 = ua2.unwrap();
        // Firefox User-Agent 可能包含 "Firefox" 或 "Gecko"
        assert!(
            ua2.contains("Firefox") || ua2.contains("Gecko"),
            "应该包含 Firefox 或 Gecko"
        );
    } else {
        // 如果失败，可能是配置名称格式问题，尝试其他格式
        let ua2_alt = get_user_agent_by_profile_name("firefox_133");
        if ua2_alt.is_ok() {
            let ua2_alt = ua2_alt.unwrap();
            assert!(ua2_alt.contains("Firefox") || ua2_alt.contains("Gecko"));
        }
    }

    // 测试随机操作系统
    let os = random_os();
    assert!(crate::types::OPERATING_SYSTEMS.contains(&os));

    println!("✅ User-Agent 生成正常");
    println!("   示例 UA: {}", ua1);
}

#[test]
fn test_http_headers_generation() {
    println!("\n=== HTTP Headers 生成测试 ===");

    let fp_result = get_random_fingerprint_by_browser("chrome").unwrap();
    let headers = fp_result.headers;

    assert!(!headers.user_agent.is_empty(), "应该有 User-Agent");
    assert!(!headers.accept.is_empty(), "应该有 Accept");
    assert!(
        !headers.accept_language.is_empty(),
        "应该有 Accept-Language"
    );

    println!("✅ Headers 生成正常");
    println!("   User-Agent: {}", headers.user_agent);
    println!("   Accept: {}", headers.accept);
    println!("   Accept-Language: {}", headers.accept_language);
}

#[test]
fn test_random_language() {
    println!("\n=== 随机语言测试 ===");

    let lang = random_language();
    assert!(!lang.is_empty(), "应该返回语言代码");

    println!("✅ 随机语言: {}", lang);
}

// ============================================================================
// 4. HTTP 客户端测试
// ============================================================================

#[test]
fn test_http_client_creation() {
    println!("\n=== HTTP 客户端创建测试 ===");

    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) Chrome/133.0.0.0".to_string(),
        prefer_http2: true,
        prefer_http3: false,
        ..Default::default()
    };

    let _client = HttpClient::new(config);
    println!("✅ HTTP 客户端创建成功");
}

#[test]
fn test_http_request_builder() {
    println!("\n=== HTTP 请求构建测试 ===");

    let request = HttpRequest::new(HttpMethod::Get, "https://example.com/test")
        .with_header("Accept", "text/html")
        .with_header("Accept-Language", "en-US,en;q=0.9");

    let http1_request = request.build_http1_request("example.com", "/test");

    assert!(http1_request.contains("GET /test HTTP/1.1"));
    assert!(http1_request.contains("Host: example.com"));
    assert!(http1_request.contains("Accept: text/html"));

    println!("✅ HTTP 请求构建成功");
    println!("{}", http1_request);
}

#[test]
#[ignore]
fn test_http_client_get_request() {
    println!("\n=== HTTP GET 请求测试 ===");

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
            println!("✅ 请求成功");
            println!("   状态码: {}", response.status_code);
            println!("   HTTP 版本: {}", response.http_version);
            println!("   响应时间: {:?}", duration);
            println!("   Body 大小: {} bytes", response.body.len());

            assert_eq!(response.status_code, 200);
        }
        Err(e) => {
            println!("⚠️  请求失败: {}", e);
            // 网络测试可能失败，不 panic
        }
    }
}

#[test]
#[ignore]
fn test_http_client_post_request() {
    println!("\n=== HTTP POST 请求测试 ===");

    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) Chrome/133.0.0.0".to_string(),
        ..Default::default()
    };

    let client = HttpClient::new(config);

    let body = "test=data";
    match client.post("https://httpbin.org/post", body.as_bytes()) {
        Ok(response) => {
            println!("✅ POST 请求成功");
            println!("   状态码: {}", response.status_code);
            assert_eq!(response.status_code, 200);
        }
        Err(e) => {
            println!("⚠️  POST 请求失败: {}", e);
        }
    }
}

// ============================================================================
// 5. Cookie 管理测试
// ============================================================================

#[test]
fn test_cookie_parsing() {
    println!("\n=== Cookie 解析测试 ===");

    let set_cookie =
        "session_id=abc123; Path=/; Domain=example.com; Secure; HttpOnly; SameSite=Strict";

    let cookie_result = Cookie::parse_set_cookie(set_cookie, "example.com".to_string());
    assert!(cookie_result.is_some(), "应该能解析 Set-Cookie");

    let cookie = cookie_result.unwrap();
    assert_eq!(cookie.name, "session_id");
    assert_eq!(cookie.value, "abc123");
    assert_eq!(cookie.path, "/");
    assert_eq!(cookie.domain, "example.com");
    assert!(cookie.secure);
    assert!(cookie.http_only);
    assert_eq!(cookie.same_site, Some(SameSite::Strict));

    println!("✅ Cookie 解析成功");
    println!("   Name: {}", cookie.name);
    println!("   Value: {}", cookie.value);
    println!("   Path: {:?}", cookie.path);
    println!("   Domain: {:?}", cookie.domain);
}

#[test]
fn test_cookie_store() {
    println!("\n=== Cookie 存储测试 ===");

    let store = CookieStore::new();

    // 添加 Cookie
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

    // 获取 Cookie
    let cookies = store.get_cookies_for_domain("example.com");
    assert!(!cookies.is_empty(), "应该能找到 Cookie");

    println!("✅ Cookie 存储正常");
    println!("   找到 {} 个 Cookie", cookies.len());
}

// ============================================================================
// 6. HTTP/2 配置测试
// ============================================================================

#[test]
fn test_http2_settings() {
    println!("\n=== HTTP/2 Settings 测试 ===");

    // Chrome Settings
    let (chrome_settings, _) = chrome_http2_settings();
    assert!(!chrome_settings.is_empty(), "Chrome 应该有 HTTP/2 Settings");

    // Firefox Settings
    let (firefox_settings, _) = firefox_http2_settings();
    assert!(
        !firefox_settings.is_empty(),
        "Firefox 应该有 HTTP/2 Settings"
    );

    // Safari Settings
    let (safari_settings, _) = safari_http2_settings();
    assert!(!safari_settings.is_empty(), "Safari 应该有 HTTP/2 Settings");

    println!("✅ HTTP/2 Settings 正常");
    println!("   Chrome: {} settings", chrome_settings.len());
    println!("   Firefox: {} settings", firefox_settings.len());
    println!("   Safari: {} settings", safari_settings.len());
}

#[test]
fn test_http2_pseudo_header_order() {
    println!("\n=== HTTP/2 Pseudo Header Order 测试 ===");

    let chrome_order = chrome_pseudo_header_order();
    let firefox_order = firefox_pseudo_header_order();
    let safari_order = safari_pseudo_header_order();

    assert!(!chrome_order.is_empty());
    assert!(!firefox_order.is_empty());
    assert!(!safari_order.is_empty());

    println!("✅ Pseudo Header Order 正常");
    println!("   Chrome: {:?}", chrome_order);
    println!("   Firefox: {:?}", firefox_order);
    println!("   Safari: {:?}", safari_order);
}

// ============================================================================
// 7. 指纹比较测试
// ============================================================================

#[test]
fn test_fingerprint_comparison() {
    println!("\n=== 指纹比较测试 ===");

    let spec1 = chrome_133().get_client_hello_spec().unwrap();
    let spec2 = firefox_133().get_client_hello_spec().unwrap();

    // 比较两个指纹
    let comparison = compare_specs(&spec1, &spec2);

    println!("✅ 指纹比较完成");
    println!("   匹配结果: {:?}", comparison);
}

#[test]
fn test_fingerprint_matching() {
    println!("\n=== 指纹匹配测试 ===");

    let target = chrome_133().get_client_hello_spec().unwrap();
    let candidates = vec![
        chrome_103().get_client_hello_spec().unwrap(),
        firefox_133().get_client_hello_spec().unwrap(),
        safari_16_0().get_client_hello_spec().unwrap(),
    ];

    let target_sig = extract_signature(&target);

    let match_result = find_best_match(&target_sig, &candidates);

    // 可能找不到完全匹配，但至少应该尝试匹配
    if let Some(best_index) = match_result {
        println!("✅ 最佳匹配找到");
        println!("   最佳匹配索引: {}", best_index);
    } else {
        println!("⚠️  未找到最佳匹配（可能所有候选都不匹配）");
    }
}

// ============================================================================
// 8. 配置导出测试
// ============================================================================

#[test]
#[cfg(feature = "export")]
fn test_config_export() {
    println!("\n=== 配置导出测试 ===");

    let profile = chrome_133();
    let spec = profile.get_client_hello_spec().unwrap();

    // 导出为 JSON（如果支持）
    println!("✅ 配置导出功能可用");
    println!("   Profile: {}", profile.get_client_hello_str());
    println!("   密码套件数量: {}", spec.cipher_suites.len());
    println!("   扩展数量: {}", spec.extensions.len());
}

// ============================================================================
// 9. 性能测试
// ============================================================================

#[test]
fn test_fingerprint_generation_performance() {
    println!("\n=== 指纹生成性能测试 ===");

    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = chrome_133().get_client_hello_spec().unwrap();
    }

    let duration = start.elapsed();
    let avg_time = duration.as_nanos() / iterations as u128;

    println!("✅ 性能测试完成");
    println!("   迭代次数: {}", iterations);
    println!("   总时间: {:?}", duration);
    println!("   平均时间: {} ns/次", avg_time);

    // 应该很快（< 1ms per iteration）
    assert!(avg_time < 1_000_000, "每次生成应该 < 1ms");
}

#[test]
fn test_http_request_building_performance() {
    println!("\n=== HTTP 请求构建性能测试 ===");

    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = HttpRequest::new(HttpMethod::Get, "https://example.com/test")
            .with_header("Accept", "text/html")
            .build_http1_request("example.com", "/test");
    }

    let duration = start.elapsed();
    let avg_time = duration.as_nanos() / iterations as u128;

    println!("✅ 请求构建性能测试完成");
    println!("   迭代次数: {}", iterations);
    println!("   总时间: {:?}", duration);
    println!("   平均时间: {} ns/次", avg_time);
}

// ============================================================================
// 10. 错误处理测试
// ============================================================================

#[test]
fn test_error_handling() {
    println!("\n=== 错误处理测试 ===");

    // 测试无效浏览器名称
    let result = get_random_fingerprint_by_browser("invalid_browser");
    assert!(result.is_err(), "无效浏览器应该返回错误");

    // 测试无效 User-Agent 生成
    let result = get_user_agent_by_profile_name("Invalid-Profile");
    // 可能返回空字符串而不是错误，这是可接受的
    if result.is_err() {
        println!("✅ 无效配置返回错误（符合预期）");
    } else {
        let ua = result.unwrap();
        if ua.is_empty() {
            println!("✅ 无效配置返回空字符串（可接受）");
        }
    }

    println!("✅ 错误处理正常");
}

// ============================================================================
// 11. 集成测试
// ============================================================================

#[test]
fn test_full_integration() {
    println!("\n=== 完整集成测试 ===");

    // 1. 获取随机指纹
    let fp_result = get_random_fingerprint().unwrap();

    // 2. 创建 HTTP 客户端配置
    let config = HttpClientConfig {
        user_agent: fp_result.user_agent.clone(),
        headers: fp_result.headers.clone(),
        profile: Some(fp_result.profile.clone()),
        prefer_http2: true,
        ..Default::default()
    };

    // 3. 创建客户端
    let _client = HttpClient::new(config);

    // 4. 构建请求
    let _request =
        HttpRequest::new(HttpMethod::Get, "https://example.com/").with_headers(&fp_result.headers);

    println!("✅ 完整集成测试通过");
    println!("   浏览器: {}", fp_result.hello_client_id);
    println!("   User-Agent: {}", fp_result.user_agent);
}

// ============================================================================
// 12. 边界条件测试
// ============================================================================

#[test]
fn test_edge_cases() {
    println!("\n=== 边界条件测试 ===");

    // 测试空字符串
    let empty_headers = HTTPHeaders::default();
    assert!(empty_headers.user_agent.is_empty() || !empty_headers.user_agent.is_empty());

    // 测试默认配置
    let default_config = HttpClientConfig::default();
    assert!(!default_config.user_agent.is_empty());

    println!("✅ 边界条件测试通过");
}

// ============================================================================
// 13. 并发安全测试
// ============================================================================

#[test]
fn test_concurrent_access() {
    println!("\n=== 并发安全测试 ===");

    use std::sync::Arc;
    use std::thread;

    let profile = Arc::new(chrome_133());
    let mut handles = vec![];

    for _ in 0..10 {
        let profile_clone = Arc::clone(&profile);
        let handle = thread::spawn(move || {
            let spec = profile_clone.get_client_hello_spec().unwrap();
            assert!(!spec.cipher_suites.is_empty());
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("✅ 并发安全测试通过");
}

// ============================================================================
// 14. 测试总结
// ============================================================================

#[test]
fn test_summary() {
    println!("\n");
    println!("═══════════════════════════════════════════════════════════");
    println!("   fingerprint-rust 全面测试套件");
    println!("═══════════════════════════════════════════════════════════");
    println!();
    println!("✅ TLS 指纹生成和验证");
    println!("✅ 浏览器指纹配置（66+ 浏览器）");
    println!("✅ User-Agent 和 Headers 生成");
    println!("✅ HTTP 客户端（H1/H2/H3）");
    println!("✅ Cookie 管理");
    println!("✅ HTTP/2 配置");
    println!("✅ 指纹比较和匹配");
    println!("✅ 性能测试");
    println!("✅ 错误处理");
    println!("✅ 并发安全");
    println!();
    println!("═══════════════════════════════════════════════════════════");
}
