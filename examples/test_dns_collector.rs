//! 测试 DNS 服务器收集器
//!
//! 使用方法：
//!   cargo run --example test_dns_collector --features dns

#[cfg(feature = "dns")]
use fingerprint::dns::ServerCollector;
#[cfg(feature = "dns")]
use std::time::Duration;

#[cfg(feature = "dns")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 测试 DNS 服务器收集器...\n");

    // 测试 1: 从 public-dns.info 获取 DNS 服务器列表
    println!("📡 测试 1: 从 public-dns.info 获取公共 DNS 服务器列表");
    match ServerCollector::collect_public_dns(Some(Duration::from_secs(30))).await {
        Ok(pool) => {
            println!("✅ 成功获取 DNS 服务器列表");
            println!("   服务器数量: {}", pool.len());
            
            // 显示前 10 个服务器
            let servers = pool.servers();
            let display_count = servers.len().min(10);
            println!("   前 {} 个服务器:", display_count);
            for (i, server) in servers.iter().take(display_count).enumerate() {
                println!("     {}. {}", i + 1, server);
            }
            if servers.len() > display_count {
                println!("     ... (还有 {} 个)", servers.len() - display_count);
            }
        }
        Err(e) => {
            println!("❌ 获取失败: {}", e);
            println!("   将使用默认服务器列表");
        }
    }

    println!("\n📡 测试 2: 使用 collect_all（带自动回退）");
    let pool = ServerCollector::collect_all(Some(Duration::from_secs(30))).await;
    println!("✅ 成功获取 DNS 服务器列表（可能包含默认服务器）");
    println!("   服务器数量: {}", pool.len());

    // 显示前 5 个服务器
    let servers = pool.servers();
    let display_count = servers.len().min(5);
    println!("   前 {} 个服务器:", display_count);
    for (i, server) in servers.iter().take(display_count).enumerate() {
        println!("     {}. {}", i + 1, server);
    }

    println!("\n✅ 所有测试完成！");
    Ok(())
}

#[cfg(not(feature = "dns"))]
fn main() {
    println!("此示例需要启用 'dns' feature");
    println!("使用方法: cargo run --example test_dns_collector --features dns");
}

