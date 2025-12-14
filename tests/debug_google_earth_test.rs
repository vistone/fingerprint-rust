//! 调试 Google Earth API 访问问题

use fingerprint::{get_user_agent_by_profile_name, HttpClient, HttpClientConfig};

#[test]
#[ignore]
fn test_http1_with_debug() {
    let url = "https://kh.google.com/rt/earth/PlanetoidMetadata";

    println!("\n═══════════════════════════════════════");
    println!("  调试 HTTP/1.1 访问");
    println!("═══════════════════════════════════════\n");

    let user_agent = get_user_agent_by_profile_name("chrome_133")
        .unwrap_or_else(|_| "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36".to_string());

    println!("User-Agent: {}", user_agent);

    let config = HttpClientConfig {
        user_agent: user_agent.clone(),
        prefer_http2: false,
        prefer_http3: false,
        verify_tls: true,
        ..Default::default()
    };

    let client = HttpClient::new(config);

    println!("\n发送请求...\n");

    match client.get(url) {
        Ok(response) => {
            println!("✅ 成功！");
            println!("  HTTP 版本: {}", response.http_version);
            println!("  状态码: {}", response.status_code);
            println!("  Body 大小: {} bytes", response.body.len());

            // 打印前 200 字节的 body
            if response.body.len() > 0 {
                let preview_len = response.body.len().min(200);
                println!("  Body 预览: {:?}", &response.body[..preview_len]);
            }

            // 打印一些关键 headers
            for (k, v) in &response.headers {
                println!("  Header: {}: {}", k, v);
            }
        }
        Err(e) => {
            println!("❌ 失败: {:?}", e);

            // 测试完毕
            println!("\n检查错误详情...");
            println!("Error: {:?}", e);
        }
    }
}

#[test]
#[cfg(feature = "http2")]
#[ignore]
fn test_http2_with_debug() {
    let url = "https://kh.google.com/rt/earth/PlanetoidMetadata";

    println!("\n═══════════════════════════════════════");
    println!("  调试 HTTP/2 访问");
    println!("═══════════════════════════════════════\n");

    let user_agent = get_user_agent_by_profile_name("chrome_133")
        .unwrap_or_else(|_| "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36".to_string());

    println!("User-Agent: {}", user_agent);

    let config = HttpClientConfig {
        user_agent: user_agent.clone(),
        prefer_http2: true,
        prefer_http3: false,
        verify_tls: true,
        ..Default::default()
    };

    let client = HttpClient::new(config);

    println!("\n发送 HTTP/2 请求...\n");

    match client.get(url) {
        Ok(response) => {
            println!("✅ 成功！");
            println!("  HTTP 版本: {}", response.http_version);
            println!("  状态码: {}", response.status_code);
            println!("  Body 大小: {} bytes", response.body.len());

            for (k, v) in &response.headers {
                println!("  Header: {}: {}", k, v);
            }
        }
        Err(e) => {
            println!("❌ 失败: {:?}", e);
        }
    }
}
