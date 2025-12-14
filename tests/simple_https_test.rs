//! 测试简单的 HTTPS 请求

use fingerprint::{HttpClient, HttpClientConfig};

#[test]
#[ignore]
fn test_google_homepage() {
    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36".to_string(),
        ..Default::default()
    };

    let client = HttpClient::new(config);

    println!("\n测试 https://www.google.com/");
    match client.get("https://www.google.com/") {
        Ok(resp) => {
            println!("✅ 成功: {}", resp.status_code);
            println!("Body 大小: {}", resp.body.len());
        }
        Err(e) => {
            println!("❌ 失败: {:?}", e);
            panic!("请求失败");
        }
    }
}

#[test]
#[ignore]
fn test_httpbin() {
    let config = HttpClientConfig {
        user_agent: "TestClient/1.0".to_string(),
        ..Default::default()
    };

    let client = HttpClient::new(config);

    println!("\n测试 https://httpbin.org/get");
    match client.get("https://httpbin.org/get") {
        Ok(resp) => {
            println!("✅ 成功: {}", resp.status_code);
            println!(
                "Body: {}",
                String::from_utf8_lossy(&resp.body[..resp.body.len().min(200)])
            );
            assert!(resp.is_success());
        }
        Err(e) => {
            println!("❌ 失败: {:?}", e);
            panic!("请求失败");
        }
    }
}

#[test]
#[ignore]
fn test_example_com() {
    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0".to_string(),
        ..Default::default()
    };

    let client = HttpClient::new(config);

    println!("\n测试 https://example.com/");
    match client.get("https://example.com/") {
        Ok(resp) => {
            println!("✅ 成功: {}", resp.status_code);
            println!("Body 大小: {}", resp.body.len());
            assert!(resp.is_success());
        }
        Err(e) => {
            println!("❌ 失败: {:?}", e);
            panic!("请求失败");
        }
    }
}

#[test]
#[ignore]
fn test_google_earth_simple() {
    let mut headers = fingerprint::HTTPHeaders::default();
    headers.accept = "*/*".to_string();

    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36".to_string(),
        headers,
        ..Default::default()
    };

    let client = HttpClient::new(config);

    println!("\n测试 https://kh.google.com/rt/earth/PlanetoidMetadata");
    match client.get("https://kh.google.com/rt/earth/PlanetoidMetadata") {
        Ok(resp) => {
            println!("✅ 成功: {}", resp.status_code);
            println!("Headers:");
            for (k, v) in &resp.headers {
                println!("  {}: {}", k, v);
            }
            println!("Body 大小: {}", resp.body.len());
            if resp.body.len() > 0 {
                println!(
                    "Body (前 100 bytes): {:?}",
                    &resp.body[..resp.body.len().min(100)]
                );
            }
            assert!(resp.is_success(), "预期成功响应，得到 {}", resp.status_code);
        }
        Err(e) => {
            println!("❌ 失败: {:?}", e);

            // 尝试使用 reqwest 对比
            println!("\n尝试使用 reqwest...");
            panic!("请求失败");
        }
    }
}
