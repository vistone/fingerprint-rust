//! HTTP Connection Pool Example
//!
//! Demonstrates how to use connection pools to manage HTTP/1.1, HTTP/2, and HTTP/3 connections
//!
//! Running methods:
//! ```bash
//! # HTTP/1.1 connection pool
//! cargo run --example http_pool --features connection-pool
//!
//! # HTTP/2 connection pool
//! cargo run --example http_pool --features connection-pool,http2
//!
//! # HTTP/3 connection pool
//! cargo run --example http_pool --features connection-pool,http3
//! ```

#[cfg(feature = "connection-pool")]
use fingerprint::{get_user_agent_by_profile_name, HttpClient, HttpClientConfig};
#[cfg(feature = "connection-pool")]
use fingerprint::http_client::PoolManagerConfig;

// ============================================================================
// HTTP/1.1 Connection Pool Example
// ============================================================================

#[cfg(feature = "connection-pool")]
fn example_http1_pool() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║          HTTP/1.1 Connection Pool Example               ║");
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
    println!("✅ HTTP/1.1 client created (connection pool enabled)\n");

    let urls = [
        "http://example.com/",
        "http://example.com/about",
        "http://example.com/contact",
    ];

    println!("📡 Sending multiple requests (should reuse connections):\n");
    for (i, url) in urls.iter().enumerate() {
        println!("  {}. {}", i + 1, url);
        match client.get(url) {
            Ok(response) => {
                println!("     ✅ {} ({} bytes)", response.status_code, response.body.len());
            }
            Err(e) => {
                println!("     ❌ Error: {:?}", e);
            }
        }
    }

    if let Some(stats) = client.pool_stats() {
        println!("\n📊 Connection pool statistics:");
        for stat in stats {
            println!("  {}: {} requests, {:.1}% success rate", stat.endpoint, stat.total_requests, stat.success_rate());
        }
    }

    Ok(())
}

// ============================================================================
// HTTP/2 Connection Pool Example
// ============================================================================

#[cfg(all(feature = "connection-pool", feature = "http2"))]
async fn example_http2_pool() -> Result<(), Box<dyn std::error::Error>> {
    use fingerprint::http_client::{http2_pool, ConnectionPoolManager};
    use fingerprint::{HttpMethod, HttpRequest};
    use std::sync::Arc;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║          HTTP/2 Connection Pool Example                  ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let user_agent = get_user_agent_by_profile_name("chrome_133")?;
    let config = HttpClientConfig {
        user_agent,
        prefer_http2: true,
        ..Default::default()
    };

    let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));
    println!("✅ HTTP/2 connection pool manager created\n");

    let urls = [
        "https://httpbin.org/get",
        "https://httpbin.org/headers",
        "https://httpbin.org/user-agent",
    ];

    println!("📡 Sending multiple HTTP/2 requests:\n");
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
                println!("     ❌ Error: {}", e);
            }
        }
    }

    println!("\n✅ HTTP/2 connection pool example completed!");
    Ok(())
}

// ============================================================================
// HTTP/3 Connection Pool Example
// ============================================================================

#[cfg(all(feature = "connection-pool", feature = "http3"))]
async fn example_http3_pool() -> Result<(), Box<dyn std::error::Error>> {
    use fingerprint::http_client::{http3_pool, ConnectionPoolManager};
    use fingerprint::{HttpMethod, HttpRequest};
    use std::sync::Arc;

    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║          HTTP/3 Connection Pool Example                  ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let user_agent = get_user_agent_by_profile_name("chrome_133")?;
    let config = HttpClientConfig {
        user_agent,
        prefer_http3: true,
        ..Default::default()
    };

    let pool_manager = Arc::new(ConnectionPoolManager::new(PoolManagerConfig::default()));
    println!("✅ HTTP/3 connection pool manager created\n");

    let urls = [
        "https://cloudflare-quic.com/",
        "https://quic.aiortc.org:443/",
    ];

    println!("📡 Sending HTTP/3 requests:\n");
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
                println!("     ❌ Error: {} (server may not support HTTP/3)", e);
            }
        }
    }

    println!("\n✅ HTTP/3 connection pool example completed!");
    Ok(())
}

// ============================================================================
// Main Function
// ============================================================================

#[cfg(feature = "connection-pool")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // HTTP/1.1 connection pool (synchronous)
    example_http1_pool()?;

    // HTTP/2 connection pool (asynchronous)
    #[cfg(feature = "http2")]
    {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(example_http2_pool())?;
    }

    // HTTP/3 connection pool (asynchronous)
    #[cfg(feature = "http3")]
    {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(example_http3_pool())?;
    }

    println!("\n✅ All connection pool examples completed!");
    Ok(())
}

#[cfg(not(feature = "connection-pool"))]
fn main() {
    eprintln!("\n❌ This example requires the connection-pool feature to be enabled!");
    eprintln!("\nPlease run with the following command:");
    eprintln!("cargo run --example http_pool --features connection-pool\n");
}