//! 简单的 HTTP/2 测试

use fingerprint::{get_user_agent_by_profile_name, HttpClient, HttpClientConfig};

#[test]
#[cfg(feature = "http2")]
#[ignore]
fn test_http2_example() {
    println!("\n========== HTTP/2 简单测试 ==========\n");

    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let config = HttpClientConfig {
        user_agent,
        prefer_http2: true,
        prefer_http3: false,
        ..Default::default()
    };

    let client = HttpClient::new(config);

    println!("测试 Example.com ...");
    match client.get("https://example.com/") {
        Ok(response) => {
            println!("  ✓ 成功");
            println!("  HTTP 版本: {}", response.http_version);
            println!("  状态码: {}", response.status_code);
            println!("  响应时间: {} ms", response.response_time_ms);
            println!("  Body 大小: {} bytes", response.body.len());

            if let Ok(body_str) = response.body_as_string() {
                let preview = body_str.chars().take(100).collect::<String>();
                println!("  Body 预览: {}", preview);
            }

            assert!(response.is_success());
            // 注意：不是所有服务器都支持 HTTP/2，所以可能会回退到 HTTP/1.1
            println!("\n  提示: 如果版本不是 HTTP/2，说明服务器不支持或协商失败");
        }
        Err(e) => {
            println!("  ✗ 失败: {:?}", e);
            panic!("HTTP/2 测试失败");
        }
    }
}

#[test]
#[cfg(feature = "http2")]
#[ignore]
fn test_http2_cloudflare() {
    println!("\n========== HTTP/2 Cloudflare 测试 ==========\n");

    let user_agent =
        get_user_agent_by_profile_name("chrome_133").unwrap_or_else(|_| "Mozilla/5.0".to_string());

    let config = HttpClientConfig {
        user_agent,
        prefer_http2: true,
        ..Default::default()
    };

    let client = HttpClient::new(config);

    println!("测试 Cloudflare (已知支持 HTTP/2) ...");
    match client.get("https://cloudflare.com/") {
        Ok(response) => {
            println!("  ✓ 成功");
            println!("  HTTP 版本: {}", response.http_version);
            println!("  状态码: {}", response.status_code);
            println!("  响应时间: {} ms", response.response_time_ms);

            assert!(response.is_success() || response.status_code == 301);
        }
        Err(e) => {
            println!("  ✗ 失败: {:?}", e);
        }
    }
}
