//! HTTP/2 连接池示例
//!
//! 演示如何使用 netconnpool 管理 HTTP/2 连接

#[cfg(all(feature = "connection-pool", feature = "http2"))]
use fingerprint::{get_user_agent_by_profile_name, HttpClient, HttpClientConfig};

#[cfg(all(feature = "connection-pool", feature = "http2"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║          HTTP/2 连接池示例                                ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // 1. 生成 User-Agent
    let user_agent = get_user_agent_by_profile_name("chrome_133")?;
    println!("🌐 User-Agent: {}\n", user_agent);

    // 2. 配置 HTTP 客户端
    let config = HttpClientConfig {
        user_agent,
        prefer_http2: true,
        ..Default::default()
    };

    // 3. 创建带连接池的客户端
    let client = HttpClient::with_pool(
        config,
        fingerprint::http_client::PoolManagerConfig::default(),
    );

    println!("✅ HTTP 客户端已创建（启用 HTTP/2 + 连接池）\n");

    // 4. 发送多个请求到同一主机（应该复用连接）
    let urls = [
        "https://httpbin.org/get",
        "https://httpbin.org/headers",
        "https://httpbin.org/user-agent",
    ];

    println!("📡 发送多个 HTTP/2 请求到 httpbin.org:\n");

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
            }
        }
        println!();
    }

    println!("✅ 所有请求完成！");
    println!("💡 连接池自动管理了 HTTP/2 连接的复用");

    Ok(())
}

#[cfg(not(all(feature = "connection-pool", feature = "http2")))]
fn main() {
    println!("此示例需要 'connection-pool' 和 'http2' features");
    println!("运行: cargo run --example http2_with_pool --features \"rustls-tls,compression,http2,connection-pool\"");
}
