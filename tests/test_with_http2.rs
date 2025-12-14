//! 使用 HTTP/2 测试 Google Earth API

use fingerprint::{HttpClient, HttpClientConfig};

#[test]
#[cfg(feature = "http2")]
#[ignore]
fn test_google_earth_with_http2() {
    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36".to_string(),
        prefer_http2: true,
        prefer_http3: false,
        ..Default::default()
    };

    let client = HttpClient::new(config);

    println!("\n测试 https://kh.google.com/rt/earth/PlanetoidMetadata (HTTP/2)");
    match client.get("https://kh.google.com/rt/earth/PlanetoidMetadata") {
        Ok(resp) => {
            println!("✅ 成功: {}", resp.status_code);
            println!("HTTP 版本: {}", resp.http_version);
            println!("Body 大小: {}", resp.body.len());
            assert!(resp.is_success());
        }
        Err(e) => {
            println!("❌ 失败: {:?}", e);
            panic!("HTTP/2 请求失败");
        }
    }
}
