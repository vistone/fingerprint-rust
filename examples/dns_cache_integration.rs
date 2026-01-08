//! DNS 缓存集成示例
//!
//! 展示如何将 DNS 缓存集成到 HTTP 客户端中，提高性能并减少 DNS 查询次数
//!
//! 使用方法：
//!   cargo run --example dns_cache_integration --features rustls-tls,http2

use fingerprint::{chrome_133, DNSHelper, HttpClient, HttpClientConfig};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 DNS 缓存集成示例");
    println!("=" .repeat(60));
    println!();

    // 1. 创建 DNS 辅助器（带 5 分钟 TTL 的缓存）
    println!("📦 步骤 1: 创建 DNS 辅助器");
    let dns_helper = Arc::new(DNSHelper::new(Duration::from_secs(300)));
    println!("   ✅ DNS 缓存已启用，TTL = 300 秒");
    println!();

    // 2. （可选）预热 DNS 缓存
    println!("🔥 步骤 2: 预热 DNS 缓存");
    let domains = ["www.google.com", "www.github.com", "www.rust-lang.org"];
    dns_helper.warmup(&domains);
    println!("   ✅ 已预热 {} 个域名", domains.len());
    let (cached, _) = dns_helper.stats();
    println!("   📊 当前缓存: {} 个域名", cached);
    println!();

    // 3. 创建带 DNS 缓存的 HTTP 客户端配置
    println!("⚙️  步骤 3: 配置 HTTP 客户端");
    let profile = chrome_133();
    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/133.0.0.0 Safari/537.36".to_string(),
        prefer_http2: true,
        dns_helper: Some(dns_helper.clone()),  // 集成 DNS 缓存
        profile: Some(profile),
        ..Default::default()
    };
    println!("   ✅ HTTP 客户端已配置 DNS 缓存");
    println!();

    // 4. 创建 HTTP 客户端
    let client = HttpClient::new(config);
    println!("🌐 步骤 4: 发送 HTTP 请求（首次，使用缓存）");
    println!();

    // 5. 发送多个请求，观察 DNS 缓存的效果
    let test_urls = [
        "https://www.google.com/",
        "https://www.github.com/",
        "https://www.rust-lang.org/",
    ];

    for (i, url) in test_urls.iter().enumerate() {
        println!("   请求 {}: {}", i + 1, url);
        
        let start = Instant::now();
        match client.get(url) {
            Ok(response) => {
                let elapsed = start.elapsed();
                println!("      ✅ 状态码: {}", response.status_code);
                println!("      ✅ HTTP 版本: {}", response.http_version);
                println!("      ✅ 响应大小: {} 字节", response.body.len());
                println!("      ⏱️  耗时: {:?}", elapsed);
                
                // 显示缓存统计
                let (cached, expired) = dns_helper.stats();
                println!("      📊 DNS 缓存: {} 个域名 ({} 个已过期)", cached, expired);
            }
            Err(e) => {
                println!("      ❌ 请求失败: {}", e);
            }
        }
        println!();
    }

    // 6. 重复请求相同的 URL，观察性能提升
    println!("🔄 步骤 5: 重复请求（充分利用 DNS 缓存）");
    println!();

    for (i, url) in test_urls.iter().enumerate() {
        println!("   重复请求 {}: {}", i + 1, url);
        
        let start = Instant::now();
        match client.get(url) {
            Ok(response) => {
                let elapsed = start.elapsed();
                println!("      ✅ 状态码: {}", response.status_code);
                println!("      ⏱️  耗时: {:?} (DNS 已缓存)", elapsed);
            }
            Err(e) => {
                println!("      ❌ 请求失败: {}", e);
            }
        }
        println!();
    }

    // 7. 显示最终统计信息
    println!("📊 最终统计");
    println!("=" .repeat(60));
    let (cached, expired) = dns_helper.stats();
    println!("   缓存域名数: {}", cached);
    println!("   已过期数: {}", expired);
    println!();

    // 8. 演示缓存管理功能
    println!("🧹 缓存管理演示");
    println!("=" .repeat(60));
    
    // 清理过期缓存
    let cleaned = dns_helper.cleanup_expired();
    println!("   ✅ 清理了 {} 个过期条目", cleaned);
    
    // 使特定域名失效
    dns_helper.invalidate("www.google.com");
    println!("   ✅ 已使 www.google.com 的缓存失效");
    
    let (cached, _) = dns_helper.stats();
    println!("   📊 当前缓存: {} 个域名", cached);
    println!();

    println!("🎉 示例完成！");
    println!();
    println!("💡 关键要点:");
    println!("   1. DNS 缓存可以显著减少 DNS 查询次数");
    println!("   2. 预热功能可以在请求前准备好 DNS 缓存");
    println!("   3. 缓存自动过期，保证 IP 地址的新鲜度");
    println!("   4. 提供灵活的缓存管理接口（清理、失效）");

    Ok(())
}
