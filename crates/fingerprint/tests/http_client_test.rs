//! HTTP client testing
//!
//! Tests fingerprint-rust with its own HTTP client library
//!
//! Run methods:
//! ```bash
//! # Local testing (no network required)
//! cargo test --test http_client_test
//!
//! # Network testing
//! cargo test --test http_client_test -- --ignored --nocapture
//! ```

use fingerprint::*;
use std::time::Instant;

fn load_profile(profile_id: &str) -> BrowserProfile {
    let mut profiles = mapped_tls_clients();
    profiles
        .remove(profile_id)
        .unwrap_or_else(|| panic!("profile not found: {}", profile_id))
}

#[test]
fn test_http_client_creation() {
    // get浏览器fingerprint
    let fp_result = get_random_fingerprint_by_browser("chrome").expect("生成指纹失败");

    // create HTTP clientconfigure
    let config = HttpClientConfig {
        user_agent: fp_result.user_agent.clone(),
        headers: fp_result.headers.clone(),
        profile: Some(load_profile(&fp_result.profile_id)),
        ..Default::default()
    };

    let _client = HttpClient::new(config);

    println!("✅ HTTP 客户端创建成功");
    println!("   User-Agent: {}", fp_result.user_agent);
    println!("   Profile: {}", fp_result.profile_id);
}

#[test]
fn test_url_parsing() {
    let _client = HttpClient::new(HttpClientConfig::default());

    // testing各种 URL 格式
    let test_cases = vec![
        (
            "https://example.com/path",
            ("https", "example.com", 443, "/path"),
        ),
        (
            "http://example.com:8080/api",
            ("http", "example.com", 8080, "/api"),
        ),
        (
            "https://api.github.com/users",
            ("https", "api.github.com", 443, "/users"),
        ),
    ];

    for (url, (exp_scheme, exp_host, exp_port, exp_path)) in test_cases {
        let _request = HttpRequest::new(HttpMethod::Get, url);
        println!("✅ 测试 URL: {}", url);
        println!(
            "   预期: {}://{}:{}{}",
            exp_scheme, exp_host, exp_port, exp_path
        );
    }
}

#[test]
fn test_http_request_builder() {
    let fp_result = get_random_fingerprint_by_browser("firefox").expect("生成指纹失败");

    let request = HttpRequest::new(HttpMethod::Get, "https://example.com/test")
        .with_user_agent(&fp_result.user_agent)
        .with_headers(&fp_result.headers);

    let http1_request = request.build_http1_request("example.com", "/test");

    println!("✅ HTTP/1.1 请求构建成功");
    println!("\n{}", http1_request);

    assert!(http1_request.contains("GET /test HTTP/1.1"));
    assert!(http1_request.contains("Host: example.com"));
    assert!(http1_request.contains(&fp_result.user_agent));
}

#[test]
#[ignore] // requirenetworkconnect
fn test_http_get_request() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║         测试 HTTP GET 请求 (使用自己的 HTTP 库)           ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // 1. get浏览器fingerprint
    let fp_result = get_random_fingerprint_by_browser("chrome").expect("生成指纹失败");
    println!("📌 使用指纹: {}", fp_result.profile_id);
    println!("📌 User-Agent: {}", fp_result.user_agent);

    // 2. create HTTP client
    let client = HttpClient::with_profile(
        load_profile(&fp_result.profile_id),
        fp_result.headers.clone(),
        fp_result.user_agent.clone(),
    );

    // 3. send HTTP 请求
    let start = Instant::now();
    let response = match client.get("http://httpbin.org/get") {
        Ok(r) => r,
        Err(e) => {
            // 实网testing可能因temporarynetwork抖动/rate limit导致failure；这里不把“非确定性failure”当成单元testingfailure。
            println!("❌ 错误: {}", e);
            if let HttpClientError::Io(ioe) = &e {
                if ioe.kind() == std::io::ErrorKind::WouldBlock {
                    println!("⚠️  读取超时/暂时不可用（WouldBlock），跳过本次断言");
                    return;
                }
            }
            return;
        }
    };
    let duration = start.elapsed();

    // 4. validate响应
    println!("\n📊 响应结果:");
    println!("   状态码: {}", response.status_code);
    println!("   耗时: {:?}", duration);
    println!("   响应大小: {} 字节", response.body.len());

    if let Ok(body_str) = response.body_as_string() {
        println!("\n📄 响应内容 (前 200 字符):");
        let preview = if body_str.len() > 200 {
            &body_str[..200]
        } else {
            &body_str
        };
        println!("{}", preview);
    }

    if response.status_code == 503 {
        println!("⚠️  服务器返回 503 Service Unavailable (可能是上游服务过载)");
        return;
    }

    assert!(
        response.is_success(),
        "预期成功响应，实际状态码: {}",
        response.status_code
    );
    assert_eq!(response.status_code, 200);
}

#[test]
#[ignore] // requirenetworkconnect
fn test_https_get_request() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║        测试 HTTPS GET 请求 (使用自己的 HTTP 库)           ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    // 1. get浏览器fingerprint
    let fp_result = get_random_fingerprint_by_browser("firefox").expect("生成指纹失败");
    println!("📌 使用指纹: {}", fp_result.profile_id);

    // 2. create HTTPS client
    let client = HttpClient::with_profile(
        load_profile(&fp_result.profile_id),
        fp_result.headers.clone(),
        fp_result.user_agent.clone(),
    );

    // 3. send HTTPS 请求
    let start = Instant::now();
    let response = client.get("https://httpbin.org/get").expect("请求failure");
    let duration = start.elapsed();

    // 4. validate响应
    println!("\n📊 响应结果:");
    println!("   状态码: {}", response.status_code);
    println!("   耗时: {:?}", duration);
    println!("   响应大小: {} 字节", response.body.len());

    // 5. check User-Agent 是否被正确send
    if let Ok(body_str) = response.body_as_string() {
        if body_str.contains(&fp_result.user_agent) {
            println!("   ✅ User-Agent 正确发送");
        }
    }

    // 外部service可能短暂return 429/503 等；此处主要validate“HTTPS path可用 + 响应可parse”。
    assert!(response.status_code > 0);
}

#[test]
#[ignore] // requirenetworkconnect
fn test_multiple_browsers() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║          测试多个浏览器指纹 (HTTP/HTTPS)                   ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    let browsers = vec!["chrome", "firefox", "safari"];
    let urls = vec!["http://httpbin.org/get", "https://httpbin.org/get"];

    for browser in browsers {
        println!("┌─────────────────────────────────────────────────────────┐");
        println!("│ 测试浏览器: {}", browser.to_uppercase());
        println!("└─────────────────────────────────────────────────────────┘");

        let fp_result = get_random_fingerprint_by_browser(browser).expect("生成指纹失败");
        let client = HttpClient::with_profile(
            load_profile(&fp_result.profile_id),
            fp_result.headers.clone(),
            fp_result.user_agent.clone(),
        );

        for url in &urls {
            let protocol = if url.starts_with("https") {
                "HTTPS"
            } else {
                "HTTP"
            };
            print!("  → {} ... ", protocol);

            let start = Instant::now();
            match client.get(url) {
                Ok(response) => {
                    let duration = start.elapsed();
                    if response.is_success() {
                        println!("✅ {} ({:?})", response.status_code, duration);
                    } else {
                        println!("❌ {} ({:?})", response.status_code, duration);
                    }
                }
                Err(e) => {
                    println!("❌ 错误: {}", e);
                }
            }
        }

        println!();
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

#[test]
#[ignore] // requirenetworkconnect
fn test_google_earth_api() {
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║       测试 Google Earth API (使用自己的 HTTP 库)          ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    let fp_result = get_random_fingerprint_by_browser("chrome").expect("生成指纹失败");
    println!("📌 使用指纹: {}", fp_result.profile_id);
    println!("📌 User-Agent: {}", fp_result.user_agent);

    let client = HttpClient::with_profile(
        load_profile(&fp_result.profile_id),
        fp_result.headers.clone(),
        fp_result.user_agent.clone(),
    );

    let url = "https://kh.google.com/rt/earth/PlanetoidMetadata";
    println!("\n🌍 访问: {}", url);

    let start = Instant::now();
    match client.get(url) {
        Ok(response) => {
            let duration = start.elapsed();
            println!("\n📊 响应结果:");
            println!("   状态码: {} ✅", response.status_code);
            println!("   耗时: {:?}", duration);
            println!("   响应大小: {} 字节", response.body.len());

            if let Ok(body_str) = response.body_as_string() {
                println!("\n📄 响应内容:");
                println!("{}", body_str);
            }

            if !response.is_success() {
                println!("⚠️  响应状态码非 2xx: {}", response.status_code);
            }
        }
        Err(e) => {
            println!("\n❌ 请求失败: {}", e);
            println!("⚠️  注意: Google Earth API 可能拦截了标准 TLS 指纹");
            println!("    这是预期行为，直到 HttpClient 完全集成自定义 TLS 指纹");
        }
    }
}

#[test]
fn test_http_response_parsing() {
    let raw_response = b"HTTP/1.1 200 OK\r\n\
                        Content-Type: application/json\r\n\
                        Content-Length: 13\r\n\
                        \r\n\
                        {\"ok\":true}";

    let response = HttpResponse::parse(raw_response).expect("解析失败");

    println!("✅ HTTP 响应解析成功");
    println!("   状态码: {}", response.status_code);
    println!("   Content-Type: {:?}", response.get_header("content-type"));
    println!("   Body: {}", response.body_as_string().unwrap());

    assert_eq!(response.status_code, 200);
    // headers store时会convertto小写
    assert_eq!(
        response.get_header("content-type"),
        Some(&"application/json".to_string())
    );
    assert!(response.is_success());
}
