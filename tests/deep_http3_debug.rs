//! 深度调试 HTTP/3 实现

use fingerprint::{HttpClient, HttpClientConfig};

#[test]
#[cfg(feature = "http3")]
#[ignore]
fn test_http3_with_client() {
    println!("\n═══ HTTP/3 客户端测试 ═══\n");

    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36".to_string(),
        prefer_http3: true,
        ..Default::default()
    };

    let client = HttpClient::new(config);

    println!("发送 HTTP/3 请求到 kh.google.com...");

    match client.get("https://kh.google.com/rt/earth/PlanetoidMetadata") {
        Ok(response) => {
            println!("✅ 成功！");
            println!("  状态: {}", response.status_code);
            println!("  HTTP 版本: {}", response.http_version);
            println!("  Body 大小: {} bytes", response.body.len());

            assert_eq!(response.status_code, 200);
            assert!(!response.body.is_empty());
        }
        Err(e) => {
            println!("❌ 失败: {:?}", e);
            println!("\n尝试简单的 HTTP/3 URL...");

            // 尝试 Cloudflare 的 HTTP/3 (已知支持良好)
            match client.get("https://cloudflare-quic.com/") {
                Ok(r) => {
                    println!("✅ Cloudflare HTTP/3 成功: {}", r.status_code);
                }
                Err(e2) => {
                    println!("❌ Cloudflare 也失败: {:?}", e2);
                }
            }

            panic!("HTTP/3 请求失败: {:?}", e);
        }
    }
}

#[cfg(not(feature = "http3"))]
#[test]
fn test_http3_feature_required() {
    println!("需要启用 http3 feature");
}
