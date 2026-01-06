//! HTTP 连接池示例
//!
//! 演示如何使用连接池管理 HTTP/1.1、HTTP/2 和 HTTP/3 连接
//!
//! 运行方式:
//! ```bash
//! # HTTP/1.1 连接池
//! cargo run --example http_pool --features connection-pool
//!
//! # HTTP/2 连接池
//! cargo run --example http_pool --features connection-pool,http2
//!
//! # HTTP/3 连接池
//! cargo run --example http_pool --features connection-pool,http3
//! ```

#[cfg(feature = "connection-pool")]
use fingerprint::{get_user_agent_by_profile_name, HttpClient, HttpClientConfig};
#[cfg(feature = "connection-pool")]
use fingerprint::http_client::PoolManagerConfig;

// ============================================================================
// HTTP/1.1 连接池示例
// ============================================================================

#[cfg(feature = "connection-pool")]
fn example_http1_pool() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║          HTTP/1.1 连接池示例                            ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let user_agent = get_user_agent_by_profile_name("chrome_133")?;
    let config = HttpClientConfig {
        user_agent,
        prefer_http2: false,
        prefer_http3: false,
        ..Default::default()
    };

    let pool_config = PoolManagerConfig {
        max_connections: 20,
        min_idle: 5,
        enable_reuse: true,
        ..Default::default()
    };

    let client = HttpClient::with_pool(config, pool_config);
    println!("✅ HTTP/1.1 客户端已创建（启用连接池）\n");

    let urls = [
        "http://example.com/",
        "http://example.com/about",
        "http://example.com/contact",
    ];

    println!("📡 发送多个请求（应该复用连接）:\n");
    for (i, url) in urls.iter().enumerate() {
        println!("  {}. {}", i + 1, url);
        match client.get(url) {
            Ok(response) => {
                println!("     ✅ {} ({} bytes)", response.status_code, response.body.len());
            }
            Err(e) => {
                println!("     ❌ 错误: {:?}", e);
            }
        }
    }

    if let Some(stats) = client.pool_stats() {
        println!("\n📊 连接池统计:");
        for stat in stats {
            println!("  {}: {} 请求, {:.1}% 成功率", stat.endpoint, stat.total_requests, stat.success_rate());
        }
    }

    Ok(())
}

// ============================================================================
// HTTP/2 连接池示例
// ============================================================================

#[cfg(all(feature = "connection-pool", feature = "http2"))]
async fn example_http2_pool() -> Result<(), Box<dyn std::error::Error>> {
    use fingerprint::http_client::{http2_pool, ConnectionPoolManager};
    use fingerprint::{HttpMethod, HttpRequest};
    use std::sync::Arc;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║          HTTP/2 连接池示例                                ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let user_agent = get_user_agent_by_profile_name("chrome_133")?;
    let config = HttpClientConfig {
        user_agent,
        prefer_http2: true,
        ..Default::default()
    };

    let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));
    println!("✅ HTTP/2 连接池管理器已创建\n");

    let urls = [
        "https://httpbin.org/get",
        "https://httpbin.org/headers",
        "https://httpbin.org/user-agent",
    ];

    println!("📡 发送多个 HTTP/2 请求:\n");
    for (i, url) in urls.iter().enumerate() {
        println!("  {}. {}", i + 1, url);
        let request = HttpRequest::new(HttpMethod::Get, url);
        let host = "httpbin.org";
        let port = 443;
        let path = url.replace("https://httpbin.org", "");

        match http2_pool::send_http2_request_with_pool(
            host,
            port,
            &path,
            &request,
            &config,
            &pool_manager,
        )
        .await
        {
            Ok(response) => {
                println!("     ✅ {} {} ({} bytes)", response.http_version, response.status_code, response.body.len());
            }
            Err(e) => {
                println!("     ❌ 错误: {}", e);
            }
        }
    }

    println!("\n✅ HTTP/2 连接池示例完成！");
    Ok(())
}

// ============================================================================
// HTTP/3 连接池示例
// ============================================================================

#[cfg(all(feature = "connection-pool", feature = "http3"))]
async fn example_http3_pool() -> Result<(), Box<dyn std::error::Error>> {
    use fingerprint::http_client::{http3_pool, ConnectionPoolManager};
    use fingerprint::{HttpMethod, HttpRequest};
    use std::sync::Arc;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║          HTTP/3 连接池示例                                ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let user_agent = get_user_agent_by_profile_name("chrome_133")?;
    let config = HttpClientConfig {
        user_agent,
        prefer_http3: true,
        ..Default::default()
    };

    let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));
    println!("✅ HTTP/3 连接池管理器已创建\n");

    let urls = [
        "https://cloudflare-quic.com/",
        "https://quic.aiortc.org:443/",
    ];

    println!("📡 发送 HTTP/3 请求:\n");
    for (i, url) in urls.iter().enumerate() {
        println!("  {}. {}", i + 1, url);
        let request = HttpRequest::new(HttpMethod::Get, url);
        let uri: http::Uri = url.parse()?;
        let host = uri.host().unwrap();
        let port = uri.port_u16().unwrap_or(443);
        let path = uri.path_and_query().map(|p| p.as_str()).unwrap_or("/");

        match http3_pool::send_http3_request_with_pool(
            host,
            port,
            path,
            &request,
            &config,
            &pool_manager,
        )
        .await
        {
            Ok(response) => {
                println!("     ✅ {} {} ({} bytes)", response.http_version, response.status_code, response.body.len());
            }
            Err(e) => {
                println!("     ❌ 错误: {} (服务器可能不支持 HTTP/3)", e);
            }
        }
    }

    println!("\n✅ HTTP/3 连接池示例完成！");
    Ok(())
}

// ============================================================================
// 主函数
// ============================================================================

#[cfg(feature = "connection-pool")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // HTTP/1.1 连接池（同步）
    example_http1_pool()?;

    // HTTP/2 连接池（异步）
    #[cfg(feature = "http2")]
    {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(example_http2_pool())?;
    }

    // HTTP/3 连接池（异步）
    #[cfg(feature = "http3")]
    {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(example_http3_pool())?;
    }

    println!("\n✅ 所有连接池示例完成！");
    Ok(())
}

#[cfg(not(feature = "connection-pool"))]
fn main() {
    eprintln!("\n❌ 此示例需要启用 connection-pool 功能！");
    eprintln!("\n请使用以下命令运行:");
    eprintln!("cargo run --example http_pool --features connection-pool\n");
}
