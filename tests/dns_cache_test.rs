//! DNS 缓存集成测试
//!
//! 测试 DNS 缓存功能和 HTTP 客户端集成

#![cfg(feature = "dns")]

use fingerprint::dns::{DNSCache, DNSResolver, DomainIPs, IPInfo};
use fingerprint::{DNSHelper, HttpClient, HttpClientConfig};
use std::sync::Arc;
use std::time::Duration;
use tokio;

#[tokio::test]
async fn test_dns_cache_basic() {
    // 创建 DNS 缓存
    let cache = DNSCache::new(Duration::from_secs(60));

    // 初始状态：缓存为空
    let (total, expired) = cache.stats();
    assert_eq!(total, 0);
    assert_eq!(expired, 0);

    // 创建测试数据
    let domain = "example.com";
    let mut domain_ips = DomainIPs::new();
    domain_ips
        .ipv4
        .push(IPInfo::new("93.184.216.34".to_string()));

    // 存入缓存
    cache.put(domain, domain_ips.clone());

    // 验证缓存命中
    let cached = cache.get(domain);
    assert!(cached.is_some());
    let cached = cached.unwrap();
    assert_eq!(cached.ipv4.len(), 1);
    assert_eq!(cached.ipv4[0].ip, "93.184.216.34");

    // 验证统计
    let (total, expired) = cache.stats();
    assert_eq!(total, 1);
    assert_eq!(expired, 0);
}

#[tokio::test]
async fn test_dns_cache_expiration() {
    // 创建短 TTL 的缓存
    let cache = DNSCache::new(Duration::from_millis(100));

    let domain = "example.com";
    let mut domain_ips = DomainIPs::new();
    domain_ips
        .ipv4
        .push(IPInfo::new("93.184.216.34".to_string()));

    // 存入缓存
    cache.put(domain, domain_ips);

    // 立即访问：应该命中
    assert!(cache.get(domain).is_some());

    // 等待过期
    tokio::time::sleep(Duration::from_millis(150)).await;

    // 访问：应该未命中（已过期）
    assert!(cache.get(domain).is_none());

    // 验证统计
    let (total, expired) = cache.stats();
    assert_eq!(total, 1);
    assert_eq!(expired, 1);

    // 清理过期条目
    let cleaned = cache.cleanup_expired();
    assert_eq!(cleaned, 1);

    let (total, _) = cache.stats();
    assert_eq!(total, 0);
}

#[tokio::test]
async fn test_dns_cache_invalidate() {
    let cache = DNSCache::new(Duration::from_secs(60));

    // 添加多个域名
    let domains = ["example.com", "test.com", "demo.com"];
    for domain in &domains {
        let mut ips = DomainIPs::new();
        ips.ipv4.push(IPInfo::new("1.1.1.1".to_string()));
        cache.put(domain, ips);
    }

    // 验证都已缓存
    let (total, _) = cache.stats();
    assert_eq!(total, 3);

    // 使一个域名失效
    cache.invalidate("example.com");

    let (total, _) = cache.stats();
    assert_eq!(total, 2);

    // 验证 example.com 未命中
    assert!(cache.get("example.com").is_none());

    // 验证其他域名仍然命中
    assert!(cache.get("test.com").is_some());
    assert!(cache.get("demo.com").is_some());
}

#[test]
fn test_dns_helper_basic() {
    // 创建 DNS 辅助器
    let helper = DNSHelper::new(Duration::from_secs(60));

    // 解析 localhost
    let addrs = helper.resolve("localhost", 8080);
    assert!(addrs.is_ok());
    let addrs = addrs.unwrap();
    assert!(!addrs.is_empty());
    assert_eq!(addrs[0].port(), 8080);

    // 验证缓存
    let (cached, _) = helper.stats();
    assert_eq!(cached, 1);
}

#[test]
fn test_dns_helper_ip_address() {
    let helper = DNSHelper::new(Duration::from_secs(60));

    // 直接使用 IP 地址（不应该被缓存）
    let addrs = helper.resolve("127.0.0.1", 8080);
    assert!(addrs.is_ok());
    let addrs = addrs.unwrap();
    assert_eq!(addrs.len(), 1);
    assert_eq!(addrs[0].port(), 8080);

    // IP 地址不应该被缓存
    let (cached, _) = helper.stats();
    assert_eq!(cached, 0);
}

#[test]
fn test_dns_helper_warmup() {
    let helper = DNSHelper::new(Duration::from_secs(60));

    // 预热缓存
    helper.warmup(&["localhost"]);

    // 验证缓存
    let (cached, _) = helper.stats();
    assert_eq!(cached, 1);

    // 验证可以解析
    let addrs = helper.resolve("localhost", 8080);
    assert!(addrs.is_ok());
}

#[test]
fn test_dns_helper_clear_cache() {
    let helper = DNSHelper::new(Duration::from_secs(60));

    // 添加缓存
    let _ = helper.resolve("localhost", 8080);
    let (cached, _) = helper.stats();
    assert!(cached > 0);

    // 清除缓存
    helper.clear_cache();
    let (cached, _) = helper.stats();
    assert_eq!(cached, 0);
}

#[test]
fn test_http_client_with_dns_helper() {
    // 创建 DNS 辅助器
    let dns_helper = Arc::new(DNSHelper::new(Duration::from_secs(300)));

    // 预热缓存
    dns_helper.warmup(&["localhost"]);

    // 配置 HTTP 客户端
    let config = HttpClientConfig {
        user_agent: "TestAgent/1.0".to_string(),
        dns_helper: Some(dns_helper.clone()),
        ..Default::default()
    };

    // 创建客户端
    let _client = HttpClient::new(config);

    // 验证 DNS 辅助器被正确配置
    let (cached, _) = dns_helper.stats();
    assert_eq!(cached, 1);
}

#[tokio::test]
async fn test_dns_resolver_integration() {
    // 创建 DNS 解析器
    let resolver = DNSResolver::new(Duration::from_secs(4));

    // 解析本地主机（应该总是成功）
    let result = resolver.resolve("localhost").await;
    assert!(result.is_ok());

    let result = result.unwrap();
    assert_eq!(result.domain, "localhost");

    // 应该至少有 IPv4 或 IPv6 地址
    let has_ips = !result.ips.ipv4.is_empty() || !result.ips.ipv6.is_empty();
    assert!(has_ips, "localhost should have at least one IP address");
}

#[tokio::test]
async fn test_dns_cache_with_resolver() {
    // 创建缓存和解析器
    let cache = DNSCache::new(Duration::from_secs(300));
    let resolver = DNSResolver::new(Duration::from_secs(4));

    let domain = "localhost";

    // 首次解析
    let result = resolver.resolve(domain).await;
    assert!(result.is_ok());

    let result = result.unwrap();
    cache.put(domain, result.ips.clone());

    // 从缓存获取（应该命中）
    let cached = cache.get(domain);
    assert!(cached.is_some());

    let cached = cached.unwrap();
    // 验证缓存的数据与解析结果一致
    assert_eq!(cached.ipv4.len(), result.ips.ipv4.len());
    assert_eq!(cached.ipv6.len(), result.ips.ipv6.len());
}

#[test]
fn test_dns_helper_expiration() {
    // 创建短 TTL 的辅助器
    let helper = DNSHelper::new(Duration::from_millis(100));

    // 解析域名
    let _ = helper.resolve("localhost", 8080);

    // 立即检查：应该被缓存
    let (cached, expired) = helper.stats();
    assert_eq!(cached, 1);
    assert_eq!(expired, 0);

    // 等待过期
    std::thread::sleep(Duration::from_millis(150));

    // 检查过期
    let (cached, expired) = helper.stats();
    assert_eq!(cached, 1);
    assert_eq!(expired, 1);

    // 清理过期条目
    let cleaned = helper.cleanup_expired();
    assert_eq!(cleaned, 1);

    let (cached, _) = helper.stats();
    assert_eq!(cached, 0);
}

#[tokio::test]
async fn test_multiple_domains_caching() {
    let cache = DNSCache::new(Duration::from_secs(60));

    // 创建多个域名的测试数据
    let domains = ["domain1.com", "domain2.com", "domain3.com"];

    for (i, domain) in domains.iter().enumerate() {
        let mut ips = DomainIPs::new();
        ips.ipv4
            .push(IPInfo::new(format!("1.1.1.{}", i + 1)));
        cache.put(domain, ips);
    }

    // 验证所有域名都被缓存
    let (total, _) = cache.stats();
    assert_eq!(total, 3);

    // 验证每个域名都能正确获取
    for (i, domain) in domains.iter().enumerate() {
        let cached = cache.get(domain);
        assert!(cached.is_some());
        let cached = cached.unwrap();
        assert_eq!(cached.ipv4.len(), 1);
        assert_eq!(cached.ipv4[0].ip, format!("1.1.1.{}", i + 1));
    }
}
