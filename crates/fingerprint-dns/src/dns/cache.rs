// ! DNS cachemodule
//! DNS cache utilities.
#![allow(clippy::empty_docs)]
// ! providememorycache functionality，减少重复 DNS parse，improveperformance

use crate::dns::types::{DNSError, DomainIPs};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

// / DNS cache条目
#[derive(Debug, Clone)]
struct CacheEntry {
    // / cacheof IP info
    ips: DomainIPs,
    // / cachecreatetime
    cached_at: Instant,
    // / cache TTL (生存time)
    ttl: Duration,
}

impl CacheEntry {
    // / checkcache是否过期
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

// / DNS cache
/// Thread-safe DNS cache with TTL and automatic cleanup.
// / providethreadsecurityof DNS parse结果cache，support TTL andautomatic过期cleanup
#[derive(Debug, Clone)]
pub struct DNSCache {
    // / cachestore (domain -> CacheEntry)
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    // / default TTL
    default_ttl: Duration,
}

impl DNSCache {
    // / createnew DNS cache
    ///
    /// # Arguments
    // / * `default_ttl` - default TTL，推荐 300 秒（5 分钟）
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    // / 从cachegetdomainof IP info
    ///
    /// # Arguments
    // / * `domain` - domain
    ///
    /// # Returns
    // / * `Some(DomainIPs)` - 如果cache命中且未过期
    // / * `None` - 如果cache未命中或已过期
    pub fn get(&self, domain: &str) -> Option<DomainIPs> {
        let cache = self.cache.read().ok()?;

        if let Some(entry) = cache.get(domain) {
            if !entry.is_expired() {
                return Some(entry.ips.clone());
            }
        }
        None
    }

    // / 将domainof IP info存入cache
    ///
    /// # Arguments
    // / * `domain` - domain
    // / * `ips` - IP info
    pub fn put(&self, domain: &str, ips: DomainIPs) {
        self.put_with_ttl(domain, ips, self.default_ttl);
    }

    // / 将domainof IP info存入cache，并指定 TTL
    ///
    /// # Arguments
    // / * `domain` - domain
    // / * `ips` - IP info
    // / * `ttl` - cache生存time
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

    // / 使cache失效（delete）
    ///
    /// # Arguments
    // / * `domain` - domain
    pub fn invalidate(&self, domain: &str) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(domain);
        }
    }

    // / cleanupall过期ofcache条目
    ///
    /// # Returns
    // / cleanupof条目count
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

    // / 清空allcache
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    // / getcachestatisticsinfo
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
        Self::new(Duration::from_secs(300)) // default 5 分钟 TTL
    }
}

// / 带cacheof DNS parse器wrap器
///
// / 在原有 DNS parse器of基础上添加cache functionality
pub struct CachedDNSResolver<R> {
    // / 底层parse器
    resolver: R,
    // / DNS cache
    cache: DNSCache,
}

impl<R> CachedDNSResolver<R> {
    // / create带cacheof DNS parse器
    ///
    /// # Arguments
    // / * `resolver` - 底层 DNS parse器
    // / * `cache` - DNS cache实例
    pub fn new(resolver: R, cache: DNSCache) -> Self {
        Self { resolver, cache }
    }

    // / getcache引用
    pub fn cache(&self) -> &DNSCache {
        &self.cache
    }

    // / get底层parse器引用
    pub fn resolver(&self) -> &R {
        &self.resolver
    }
}

impl<R> CachedDNSResolver<R>
where
    R: crate::dns::resolver::DNSResolverTrait,
{
    // / parsedomain（automaticusecache）
    ///
    /// # Arguments
    // / * `domain` - domain
    ///
    /// # Returns
    // / parse结果，include IPv4 and IPv6 address
    pub async fn resolve(&self, domain: &str) -> Result<crate::dns::types::DNSResult, DNSError> {
        // 先尝试从cacheget
        if let Some(cached_ips) = self.cache.get(domain) {
            return Ok(crate::dns::types::DNSResult {
                domain: domain.to_string(),
                ips: cached_ips,
            });
        }

        // cache未命中，执行实际parse
        let result = self.resolver.resolve(domain).await?;

        // 将结果存入cache
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

        // 初始state：cache未命中
        assert!(cache.get(domain).is_none());

        // 存入cache
        let mut ips = DomainIPs::new();
        ips.ipv4
            .push(crate::dns::types::IPInfo::new("93.184.216.34".to_string()));
        cache.put(domain, ips.clone());

        // cache命中
        assert!(cache.get(domain).is_some());
        let cached = cache.get(domain).unwrap();
        assert_eq!(cached.ipv4.len(), 1);
        assert_eq!(cached.ipv4[0].ip, "93.184.216.34");

        // 使cache失效
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

        // 添加两个domain
        let mut ips1 = DomainIPs::new();
        ips1.ipv4
            .push(crate::dns::types::IPInfo::new("1.1.1.1".to_string()));
        cache.put("domain1.com", ips1);

        let mut ips2 = DomainIPs::new();
        ips2.ipv4
            .push(crate::dns::types::IPInfo::new("2.2.2.2".to_string()));
        cache.put("domain2.com", ips2);

        // validatestatistics
        let (total, expired) = cache.stats();
        assert_eq!(total, 2);
        assert_eq!(expired, 0);

        // 等待过期
        std::thread::sleep(Duration::from_millis(150));

        // cleanup前statistics
        let (total, expired) = cache.stats();
        assert_eq!(total, 2);
        assert_eq!(expired, 2);

        // cleanup过期条目
        let cleaned = cache.cleanup_expired();
        assert_eq!(cleaned, 2);

        // cleanup后statistics
        let (total, _) = cache.stats();
        assert_eq!(total, 0);
    }

    #[test]
    fn test_cache_stats() {
        let cache = DNSCache::default();

        // 初始state
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
