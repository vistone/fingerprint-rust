//! HTTP/3 连接池示例
//!
//! 演示如何使用 netconnpool 管理 HTTP/3 (QUIC) 连接

#[cfg(all(feature = "connection-pool", feature = "http3"))]
use fingerprint::{get_user_agent_by_profile_name, HttpClient, HttpClientConfig};

#[cfg(all(feature = "connection-pool", feature = "http3"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║          HTTP/3 连接池示例                                ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // 1. 生成 User-Agent
    let user_agent = get_user_agent_by_profile_name("chrome_133")?;
    println!("🌐 User-Agent: {}\n", user_agent);

    // 2. 配置 HTTP 客户端
    let config = HttpClientConfig {
        user_agent,
        prefer_http3: true,
        ..Default::default()
    };

    // 3. 创建带连接池的客户端
    let client = HttpClient::with_pool(
        config,
        fingerprint::http_client::PoolManagerConfig::default(),
    );

    println!("✅ HTTP 客户端已创建（启用 HTTP/3 + 连接池）\n");

    // 4. 测试 HTTP/3 请求
    // 注意：需要服务器支持 HTTP/3 (QUIC)
    let urls = [
        "https://cloudflare-quic.com/",
        "https://quic.aiortc.org:443/",
    ];

    println!("📡 发送 HTTP/3 请求:\n");

    for (i, url) in urls.iter().enumerate() {
        println!("请求 {} - {}", i + 1, url);

        match client.get(url) {
            Ok(response) => {
                println!(
                    "  ✓ 成功: {} {}",
                    response.http_version, response.status_code
                );
                println!("  Body 大小: {} bytes", response.body.len());
            }
            Err(e) => {
                println!("  ✗ 失败: {}", e);
                println!("  提示: 服务器可能不支持 HTTP/3");
            }
        }
        println!();
    }

    println!("✅ 测试完成！");
    println!("💡 HTTP/3 使用 QUIC 协议，提供更快的连接建立和更好的性能");

    Ok(())
}

#[cfg(not(all(feature = "connection-pool", feature = "http3")))]
fn main() {
    println!("此示例需要 'connection-pool' 和 'http3' features");
    println!(
        "运行: cargo run --example http3_with_pool --features \"rustls-tls,http3,connection-pool\""
    );
}
