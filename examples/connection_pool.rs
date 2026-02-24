//! 连接池使用示例
//!
//! 演示如何使用 netconnpool 进行连接复用

#[cfg(feature = "connection-pool")]
use fingerprint::{get_user_agent_by_profile_name, HttpClient, HttpClientConfig};

#[cfg(feature = "connection-pool")]
use fingerprint::http_client::PoolManagerConfig;

#[cfg(feature = "connection-pool")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║              fingerprint-rust 连接池示例                       ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // 1. 创建连接池配置
    let pool_config = PoolManagerConfig {
        max_connections: 20, // 最大连接数
        min_idle: 5,         // 最小空闲连接
        enable_reuse: true,  // 启用连接复用
        ..Default::default()
    };

    println!("✅ 连接池配置:");
    println!("  最大连接数: {}", pool_config.max_connections);
    println!("  最小空闲: {}", pool_config.min_idle);
    println!("  连接复用: {}\n", pool_config.enable_reuse);

    // 2. 获取浏览器指纹
    let user_agent = get_user_agent_by_profile_name("chrome_133")?;

    // 3. 创建 HTTP 客户端配置
    let config = HttpClientConfig {
        user_agent,
        ..Default::default()
    };

    // 4. 创建带连接池的 HTTP 客户端
    let client = HttpClient::with_pool(config, pool_config);

    println!("✅ HTTP 客户端已创建（启用连接池）\n");

    // 5. 发送多个请求到同一主机
    let urls = [
        "http://example.com/",
        "http://example.com/about",
        "http://example.com/contact",
    ];

    println!("📡 发送请求到 example.com:\n");

    for (i, url) in urls.iter().enumerate() {
        println!("  {}. {}", i + 1, url);
        match client.get(url) {
            Ok(response) => {
                println!("     ✅ 状态码: {}", response.status_code);
                println!("     ⏱️ 响应时间: {} ms", response.response_time_ms);
                println!("     📦 大小: {} bytes", response.body.len());
            }
            Err(e) => {
                println!("     ❌ 错误: {:?}", e);
            }
        }
        println!();
    }

    // 6. 显示连接池统计
    if let Some(stats) = client.pool_stats() {
        println!("📊 连接池统计:\n");
        for stat in stats {
            println!("  端点: {}", stat.endpoint);
            println!("  ├─ 总连接数: {}", stat.total_connections);
            println!("  ├─ 活跃连接: {}", stat.active_connections);
            println!("  ├─ 空闲连接: {}", stat.idle_connections);
            println!("  ├─ 总请求数: {}", stat.total_requests);
            println!("  ├─ 成功请求: {}", stat.successful_requests);
            println!("  ├─ 失败请求: {}", stat.failed_requests);
            println!("  └─ 成功率: {:.2}%", stat.success_rate());
            println!();
        }
    }

    // 7. Test multi-host connection pool
    println!("📡 Testing multi-host connection pool:\n");

    let multi_urls = [
        "http://example.com/",
        "http://httpbin.org/get",
        "http://example.com/", // Duplicate URL, should reuse connection
    ];

    for (i, url) in multi_urls.iter().enumerate() {
        println!("  {}. {}", i + 1, url);
        match client.get(url) {
            Ok(response) => {
                println!("     ✅ 状态码: {}", response.status_code);
            }
            Err(e) => {
                println!("     ❌ 错误: {:?}", e);
            }
        }
    }

    // 8. 最终统计
    println!("\n📊 最终连接池统计:\n");
    if let Some(stats) = client.pool_stats() {
        println!("  管理的端点数: {}", stats.len());
        for stat in stats {
            println!(
                "  - {}: {} 请求, {:.1}% 成功率",
                stat.endpoint,
                stat.total_requests,
                stat.success_rate()
            );
        }
    }

    println!("\n✅ 示例完成！\n");

    Ok(())
}

#[cfg(not(feature = "connection-pool"))]
fn main() {
    eprintln!("\n❌ 此示例需要启用 connection-pool 功能！");
    eprintln!("\n请使用以下命令运行:");
    eprintln!("cargo run --example connection_pool --features connection-pool\n");
}
