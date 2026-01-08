//! DNS 缓存模块
//!
//! 提供内存缓存功能，减少重复 DNS 解析，提高性能

use crate::dns::types::{DNSError, DomainIPs};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// DNS 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    /// 缓存的 IP 信息
    ips: DomainIPs,
    /// 缓存创建时间
    cached_at: Instant,
    /// 缓存 TTL (生存时间)
    ttl: Duration,
}

impl CacheEntry {
    /// 检查缓存是否过期
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

/// DNS 缓存
///
/// 提供线程安全的 DNS 解析结果缓存，支持 TTL 和自动过期清理
#[derive(Debug, Clone)]
pub struct DNSCache {
    /// 缓存存储 (domain -> CacheEntry)
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// 默认 TTL
    default_ttl: Duration,
}

impl DNSCache {
    /// 创建新的 DNS 缓存
    ///
    /// # Arguments
    /// * `default_ttl` - 默认 TTL，推荐 300 秒（5 分钟）
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    /// 从缓存获取域名的 IP 信息
    ///
    /// # Arguments
    /// * `domain` - 域名
    ///
    /// # Returns
    /// * `Some(DomainIPs)` - 如果缓存命中且未过期
    /// * `None` - 如果缓存未命中或已过期
    pub fn get(&self, domain: &str) -> Option<DomainIPs> {
        let cache = self.cache.read().ok()?;

        if let Some(entry) = cache.get(domain) {
            if !entry.is_expired() {
                return Some(entry.ips.clone());
            }
        }
        None
    }

    /// 将域名的 IP 信息存入缓存
    ///
    /// # Arguments
    /// * `domain` - 域名
    /// * `ips` - IP 信息
    pub fn put(&self, domain: &str, ips: DomainIPs) {
        self.put_with_ttl(domain, ips, self.default_ttl);
    }

    /// 将域名的 IP 信息存入缓存，并指定 TTL
    ///
    /// # Arguments
    /// * `domain` - 域名
    /// * `ips` - IP 信息
    /// * `ttl` - 缓存生存时间
    pub fn put_with_ttl(&self, domain: &str, ips: DomainIPs, ttl: Duration) {
        if let Ok(mut cache) = self.cache.write() {
            let entry = CacheEntry {
                ips,
                cached_at: Instant::now(),
                ttl,
            };
            cache.insert(domain.to_string(), entry);
        }
    }

    /// 使缓存失效（删除）
    ///
    /// # Arguments
    /// * `domain` - 域名
    pub fn invalidate(&self, domain: &str) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(domain);
        }
    }

    /// 清理所有过期的缓存条目
    ///
    /// # Returns
    /// 清理的条目数量
    pub fn cleanup_expired(&self) -> usize {
        if let Ok(mut cache) = self.cache.write() {
            let before_count = cache.len();
            cache.retain(|_, entry| !entry.is_expired());
            let after_count = cache.len();
            before_count - after_count
        } else {
            0
        }
    }

    /// 清空所有缓存
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// 获取缓存统计信息
    ///
    /// # Returns
    /// (total_entries, expired_entries)
    pub fn stats(&self) -> (usize, usize) {
        if let Ok(cache) = self.cache.read() {
            let total = cache.len();
            let expired = cache.values().filter(|e| e.is_expired()).count();
            (total, expired)
        } else {
            (0, 0)
        }
    }
}

impl Default for DNSCache {
    fn default() -> Self {
        Self::new(Duration::from_secs(300)) // 默认 5 分钟 TTL
    }
}

/// 带缓存的 DNS 解析器包装器
///
/// 在原有 DNS 解析器的基础上添加缓存功能
pub struct CachedDNSResolver<R> {
    /// 底层解析器
    resolver: R,
    /// DNS 缓存
    cache: DNSCache,
}

impl<R> CachedDNSResolver<R> {
    /// 创建带缓存的 DNS 解析器
    ///
    /// # Arguments
    /// * `resolver` - 底层 DNS 解析器
    /// * `cache` - DNS 缓存实例
    pub fn new(resolver: R, cache: DNSCache) -> Self {
        Self { resolver, cache }
    }

    /// 获取缓存引用
    pub fn cache(&self) -> &DNSCache {
        &self.cache
    }

    /// 获取底层解析器引用
    pub fn resolver(&self) -> &R {
        &self.resolver
    }
}

impl<R> CachedDNSResolver<R>
where
    R: crate::dns::resolver::DNSResolverTrait,
{
    /// 解析域名（自动使用缓存）
    ///
    /// # Arguments
    /// * `domain` - 域名
    ///
    /// # Returns
    /// 解析结果，包含 IPv4 和 IPv6 地址
    pub async fn resolve(&self, domain: &str) -> Result<crate::dns::types::DNSResult, DNSError> {
        // 先尝试从缓存获取
        if let Some(cached_ips) = self.cache.get(domain) {
            return Ok(crate::dns::types::DNSResult {
                domain: domain.to_string(),
                ips: cached_ips,
            });
        }

        // 缓存未命中，执行实际解析
        let result = self.resolver.resolve(domain).await?;

        // 将结果存入缓存
        self.cache.put(domain, result.ips.clone());

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic() {
        let cache = DNSCache::new(Duration::from_secs(60));
        let domain = "example.com";

        // 初始状态：缓存未命中
        assert!(cache.get(domain).is_none());

        // 存入缓存
        let mut ips = DomainIPs::new();
        ips.ipv4
            .push(crate::dns::types::IPInfo::new("93.184.216.34".to_string()));
        cache.put(domain, ips.clone());

        // 缓存命中
        assert!(cache.get(domain).is_some());
        let cached = cache.get(domain).unwrap();
        assert_eq!(cached.ipv4.len(), 1);
        assert_eq!(cached.ipv4[0].ip, "93.184.216.34");

        // 使缓存失效
        cache.invalidate(domain);
        assert!(cache.get(domain).is_none());
    }

    #[test]
    fn test_cache_expiration() {
        let cache = DNSCache::new(Duration::from_millis(100));
        let domain = "example.com";

        let mut ips = DomainIPs::new();
        ips.ipv4
            .push(crate::dns::types::IPInfo::new("93.184.216.34".to_string()));
        cache.put(domain, ips);

        // 立即访问：应该命中
        assert!(cache.get(domain).is_some());

        // 等待过期
        std::thread::sleep(Duration::from_millis(150));

        // 访问：应该未命中（已过期）
        assert!(cache.get(domain).is_none());
    }

    #[test]
    fn test_cache_cleanup() {
        let cache = DNSCache::new(Duration::from_millis(100));

        // 添加两个域名
        let mut ips1 = DomainIPs::new();
        ips1.ipv4
            .push(crate::dns::types::IPInfo::new("1.1.1.1".to_string()));
        cache.put("domain1.com", ips1);

        let mut ips2 = DomainIPs::new();
        ips2.ipv4
            .push(crate::dns::types::IPInfo::new("2.2.2.2".to_string()));
        cache.put("domain2.com", ips2);

        // 验证统计
        let (total, expired) = cache.stats();
        assert_eq!(total, 2);
        assert_eq!(expired, 0);

        // 等待过期
        std::thread::sleep(Duration::from_millis(150));

        // 清理前统计
        let (total, expired) = cache.stats();
        assert_eq!(total, 2);
        assert_eq!(expired, 2);

        // 清理过期条目
        let cleaned = cache.cleanup_expired();
        assert_eq!(cleaned, 2);

        // 清理后统计
        let (total, _) = cache.stats();
        assert_eq!(total, 0);
    }

    #[test]
    fn test_cache_stats() {
        let cache = DNSCache::default();

        // 初始状态
        let (total, expired) = cache.stats();
        assert_eq!(total, 0);
        assert_eq!(expired, 0);

        // 添加一个条目
        let mut ips = DomainIPs::new();
        ips.ipv4
            .push(crate::dns::types::IPInfo::new("1.1.1.1".to_string()));
        cache.put("example.com", ips);

        let (total, expired) = cache.stats();
        assert_eq!(total, 1);
        assert_eq!(expired, 0);
    }
}
