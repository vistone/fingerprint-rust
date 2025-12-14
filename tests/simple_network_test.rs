//! 简单的网络连接测试
//! 使用简单的端点验证基本连接

use fingerprint::{get_user_agent_by_profile_name, HttpClient, HttpClientConfig};

#[test]
#[ignore]
fn test_simple_http1() {
    println!("\n========== 测试简单 HTTP/1.1 连接 ==========\n");

    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let mut config = HttpClientConfig::default();
    config.user_agent = user_agent;
    config.prefer_http2 = false;
    config.prefer_http3 = false;

    let client = HttpClient::new(config);

    // 测试一个非常简单的端点
    let simple_urls = vec![
        ("Example.com", "http://example.com/"),
        ("HTTPBin", "http://httpbin.org/get"),
    ];

    for (name, url) in simple_urls {
        println!("测试 {} ...", name);
        match client.get(url) {
            Ok(response) => {
                println!("  ✓ 成功");
                println!("  版本: {}", response.http_version);
                println!("  状态码: {}", response.status_code);
                println!("  响应时间: {} ms", response.response_time_ms);
                println!("  Body 大小: {} bytes\n", response.body.len());
            }
            Err(e) => {
                println!("  ✗ 失败: {:?}\n", e);
            }
        }
    }
}

#[test]
#[ignore]
fn test_simple_https() {
    println!("\n========== 测试简单 HTTPS 连接 ==========\n");

    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let config = HttpClientConfig {
        user_agent,
        prefer_http2: false,
        prefer_http3: false,
        ..Default::default()
    };

    let client = HttpClient::new(config);

    // 测试一些简单的 HTTPS 端点
    let test_urls = vec![
        ("Example.com", "https://example.com/"),
        ("HTTPBin", "https://httpbin.org/get"),
        ("Cloudflare", "https://cloudflare.com/"),
    ];

    for (name, url) in test_urls {
        println!("测试 {} ...", name);
        match client.get(url) {
            Ok(response) => {
                println!("  ✓ 成功");
                println!("  版本: {}", response.http_version);
                println!("  状态码: {}", response.status_code);
                println!("  响应时间: {} ms", response.response_time_ms);
                println!("  Body 大小: {} bytes", response.body.len());

                // 显示前100个字符
                if let Ok(body_str) = response.body_as_string() {
                    let preview = body_str.chars().take(100).collect::<String>();
                    println!("  Body 预览: {}", preview);
                }
                println!();
            }
            Err(e) => {
                println!("  ✗ 失败: {:?}\n", e);
            }
        }
    }
}

#[test]
#[ignore]
fn test_http_vs_https() {
    println!("\n========== HTTP vs HTTPS 对比测试 ==========\n");

    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let config = HttpClientConfig {
        user_agent,
        prefer_http2: false,
        prefer_http3: false,
        ..Default::default()
    };

    let client = HttpClient::new(config);

    // HTTP
    println!("1. HTTP 测试:");
    match client.get("http://example.com/") {
        Ok(response) => {
            println!("  ✓ HTTP 成功");
            println!("  状态码: {}", response.status_code);
            println!("  响应时间: {} ms\n", response.response_time_ms);
            assert!(response.is_success());
        }
        Err(e) => {
            println!("  ✗ HTTP 失败: {:?}\n", e);
        }
    }

    // HTTPS
    println!("2. HTTPS 测试:");
    match client.get("https://example.com/") {
        Ok(response) => {
            println!("  ✓ HTTPS 成功");
            println!("  状态码: {}", response.status_code);
            println!("  响应时间: {} ms\n", response.response_time_ms);
            assert!(response.is_success());
        }
        Err(e) => {
            println!("  ✗ HTTPS 失败: {:?}\n", e);
        }
    }
}
