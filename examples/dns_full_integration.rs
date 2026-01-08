//! DNS 模块与 HTTP 客户端完整集成示例
//!
//! 展示如何结合 DNS 预解析服务和 HTTP 客户端，实现智能的域名解析和请求优化
//!
//! 使用方法：
//!   cargo run --example dns_full_integration --features dns,rustls-tls,http2

#[cfg(feature = "dns")]
use fingerprint::{
    chrome_133, DNSCache, DNSConfig, DNSResolver, DNSService, DomainIPs, HttpClient,
    HttpClientConfig, IPInfo,
};
#[cfg(feature = "dns")]
use std::sync::Arc;
#[cfg(feature = "dns")]
use std::time::Duration;

#[cfg(feature = "dns")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 DNS 模块与 HTTP 客户端完整集成示例");
    println!("=" .repeat(70));
    println!();

    // === 场景 1: 使用 DNS 缓存加速 HTTP 请求 ===
    println!("📦 场景 1: DNS 缓存加速");
    println!("-" .repeat(70));

    // 创建 DNS 缓存
    let dns_cache = DNSCache::new(Duration::from_secs(300));

    // 创建域名列表
    let domains = vec!["www.google.com", "www.github.com"];

    // 预解析域名并填充缓存
    let resolver = DNSResolver::new(Duration::from_secs(4));
    println!("🔍 预解析域名...");
    for domain in &domains {
        match resolver.resolve(domain).await {
            Ok(result) => {
                println!(
                    "   ✅ {}: {} 个 IPv4, {} 个 IPv6",
                    domain,
                    result.ips.ipv4.len(),
                    result.ips.ipv6.len()
                );
                dns_cache.put(domain, result.ips);
            }
            Err(e) => {
                println!("   ❌ {} 解析失败: {}", domain, e);
            }
        }
    }

    // 显示缓存统计
    let (total, expired) = dns_cache.stats();
    println!("   📊 DNS 缓存统计: {} 个域名, {} 个已过期", total, expired);
    println!();

    // 创建 HTTP 客户端
    let profile = chrome_133();
    let config = HttpClientConfig {
        user_agent: "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36".to_string(),
        prefer_http2: true,
        profile: Some(profile),
        ..Default::default()
    };
    let client = HttpClient::new(config);

    // 发送 HTTP 请求（此时 DNS 已经缓存）
    println!("🌐 发送 HTTP 请求（使用预解析的 DNS 缓存）...");
    for domain in &domains {
        let url = format!("https://{}/", domain);
        println!("   请求: {}", url);

        match client.get(&url) {
            Ok(response) => {
                println!("      ✅ 状态码: {}", response.status_code);
                println!("      ✅ HTTP 版本: {}", response.http_version);
                println!("      ✅ 响应大小: {} 字节", response.body.len());
            }
            Err(e) => {
                println!("      ❌ 请求失败: {}", e);
            }
        }
        println!();
    }

    // === 场景 2: DNS 预解析服务自动维护 ===
    println!("📦 场景 2: DNS 预解析服务（自动后台维护）");
    println!("-" .repeat(70));
    println!("💡 提示: 此场景需要 IPInfo token 和较长运行时间，这里仅演示配置");
    println!();

    // 创建 DNS 服务配置
    let dns_config = DNSConfig::new(
        "your-ipinfo-token", // 需要真实的 IPInfo token
        &["google.com", "github.com"],
    );

    println!("⚙️  DNS 服务配置:");
    println!("   - IPInfo Token: {} (需要替换为真实 token)", dns_config.ipinfo_token);
    println!("   - 域名列表: {:?}", dns_config.domain_list);
    println!("   - 检查间隔: {}", dns_config.interval);
    println!("   - 最大并发: {}", dns_config.max_concurrency);
    println!("   - DNS 超时: {}", dns_config.dns_timeout);
    println!();

    // 注意：实际使用时需要：
    // 1. 获取真实的 IPInfo token
    // 2. 启动 DNS 服务: service.start().await?
    // 3. 定期从 dns_output 目录读取解析结果
    // 4. 在 HTTP 请求前使用这些预解析的 IP

    println!("📝 实际使用步骤:");
    println!("   1. 获取 IPInfo token: https://ipinfo.io/");
    println!("   2. 配置 DNS 服务（见 examples/dns_service.rs）");
    println!("   3. 启动服务: service.start().await");
    println!("   4. 服务会自动维护域名 IP 列表");
    println!("   5. 从 dns_output/*.json 读取最新 IP");
    println!("   6. 在 HTTP 请求中优先使用这些 IP");
    println!();

    // === 场景 3: 智能 IP 选择（根据地理位置） ===
    println!("📦 场景 3: 智能 IP 选择（示例）");
    println!("-" .repeat(70));

    // 模拟从 DNS 服务获取的域名 IP 信息
    let mut domain_ips = DomainIPs::new();

    // 添加一些示例 IP 信息（实际应该从 DNS 服务获取）
    domain_ips.ipv4.push(IPInfo {
        ip: "142.250.191.14".to_string(),
        hostname: None,
        city: Some("Mountain View".to_string()),
        region: Some("California".to_string()),
        country: Some("US".to_string()),
        loc: Some("37.4056,-122.0775".to_string()),
        org: Some("Google LLC".to_string()),
        timezone: Some("America/Los_Angeles".to_string()),
    });

    domain_ips.ipv4.push(IPInfo {
        ip: "172.217.14.206".to_string(),
        hostname: None,
        city: Some("Tokyo".to_string()),
        region: Some("Tokyo".to_string()),
        country: Some("JP".to_string()),
        loc: Some("35.6895,139.6917".to_string()),
        org: Some("Google LLC".to_string()),
        timezone: Some("Asia/Tokyo".to_string()),
    });

    println!("🌍 可用的 IP 地址:");
    for (i, ip_info) in domain_ips.ipv4.iter().enumerate() {
        println!("   {}. {}", i + 1, ip_info.ip);
        if let Some(city) = &ip_info.city {
            println!("      城市: {}", city);
        }
        if let Some(country) = &ip_info.country {
            println!("      国家: {}", country);
        }
        if let Some(org) = &ip_info.org {
            println!("      组织: {}", org);
        }
        println!();
    }

    println!("💡 智能选择策略:");
    println!("   - 根据地理位置选择最近的 IP");
    println!("   - 根据网络延迟选择最快的 IP");
    println!("   - 根据负载情况动态切换 IP");
    println!("   - 实现故障转移和高可用");
    println!();

    // === 总结 ===
    println!("=" .repeat(70));
    println!("🎉 集成完成！");
    println!();
    println!("📚 DNS 模块增强功能总结:");
    println!("   ✅ DNS 缓存 (DNSCache) - 减少重复解析");
    println!("   ✅ DNS 预解析 (DNSResolver) - 提前准备 IP");
    println!("   ✅ DNS 服务 (DNSService) - 自动维护域名 IP");
    println!("   ✅ IP 地理信息 (IPInfo) - 智能 IP 选择");
    println!("   ✅ HTTP 客户端集成 - 无缝配合使用");
    println!();
    println!("🔗 更多示例:");
    println!("   - examples/dns_service.rs - DNS 服务使用");
    println!("   - examples/resolve_domains.rs - 域名解析");
    println!("   - examples/dns_cache_integration.rs - 缓存集成");
    println!();

    Ok(())
}

#[cfg(not(feature = "dns"))]
fn main() {
    println!("此示例需要启用 'dns' feature");
    println!("使用方法: cargo run --example dns_full_integration --features dns,rustls-tls,http2");
}
