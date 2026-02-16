//! DNS cache module
//!
//! Provides memory cache functionality to reduce redundant DNS lookups and improve performance

use crate::dns::types::{DNSError, DomainIPs};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// DNS cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Cached IP information
    ips: DomainIPs,
    /// Cache creation time
    cached_at: Instant,
    /// Cache TTL (time to live)
    ttl: Duration,
}

impl CacheEntry {
    /// Check if cache entry is expired
    fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

/// DNS cache
///
/// Thread-safe DNS cache with TTL and automatic cleanup.
#[derive(Debug, Clone)]
pub struct DNSCache {
    /// Cache storage (domain -> CacheEntry)
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// Default TTL
    default_ttl: Duration,
}

impl DNSCache {
    /// Create a new DNS cache
    ///
    /// # Arguments
    /// * `default_ttl` - Default TTL, recommended 300 seconds (5 minutes)
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    /// Get domain IP information from cache
    ///
    /// # Arguments
    /// * `domain` - Domain name
    ///
    /// # Returns
    /// * `Some(DomainIPs)` - If cache hit and not expired
    /// * `None` - If cache miss or expired
    pub fn get(&self, domain: &str) -> Option<DomainIPs> {
        let cache = self.cache.read().ok()?;

        if let Some(entry) = cache.get(domain) {
            if !entry.is_expired() {
                return Some(entry.ips.clone());
            }
        }
        None
    }

    /// Store domain IP information into cache
    ///
    /// # Arguments
    /// * `domain` - Domain name
    /// * `ips` - IP information
    pub fn put(&self, domain: &str, ips: DomainIPs) {
        self.put_with_ttl(domain, ips, self.default_ttl);
    }

    /// Store domain IP information into cache with specified TTL
    ///
    /// # Arguments
    /// * `domain` - Domain name
    /// * `ips` - IP information
    /// * `ttl` - Cache time to live
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

    /// Invalidate cache entry (delete)
    ///
    /// # Arguments
    /// * `domain` - Domain name
    pub fn invalidate(&self, domain: &str) {
        if let Ok(mut cache) = self.cache.write() {
            cache.remove(domain);
        }
    }

    /// Cleanup all expired cache entries
    ///
    /// # Returns
    /// Number of cleaned up entries
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

    /// Clear all cache
    pub fn clear(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Get cache statistics information
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
        Self::new(Duration::from_secs(300)) // Default 5 minutes TTL
    }
}

/// Cached DNS resolver wrapper
///
/// Adds cache functionality on top of existing DNS resolver
pub struct CachedDNSResolver<R> {
    /// Underlying resolver
    resolver: R,
    /// DNS cache
    cache: DNSCache,
}

impl<R> CachedDNSResolver<R> {
    /// Create a cached DNS resolver
    ///
    /// # Arguments
    /// * `resolver` - Underlying DNS resolver
    /// * `cache` - DNS cache instance
    pub fn new(resolver: R, cache: DNSCache) -> Self {
        Self { resolver, cache }
    }

    /// Get cache reference
    pub fn cache(&self) -> &DNSCache {
        &self.cache
    }

    /// Get underlying resolver reference
    pub fn resolver(&self) -> &R {
        &self.resolver
    }
}

impl<R> CachedDNSResolver<R>
where
    R: crate::dns::resolver::DNSResolverTrait,
{
    /// Resolve domain (automatically using cache)
    ///
    /// # Arguments
    /// * `domain` - Domain name
    ///
    /// # Returns
    /// Resolution result including IPv4 and IPv6 addresses
    pub async fn resolve(&self, domain: &str) -> Result<crate::dns::types::DNSResult, DNSError> {
        // First try to get from cache
        if let Some(cached_ips) = self.cache.get(domain) {
            return Ok(crate::dns::types::DNSResult {
                domain: domain.to_string(),
                ips: cached_ips,
            });
        }

        // Cache miss, perform actual resolution
        let result = self.resolver.resolve(domain).await?;

        // Store result in cache
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

        // Initial state: cache miss
        assert!(cache.get(domain).is_none());

        // Store in cache
        let mut ips = DomainIPs::new();
        ips.ipv4
            .push(crate::dns::types::IPInfo::new("93.184.216.34".to_string()));
        cache.put(domain, ips.clone());

        // Cache hit
        assert!(cache.get(domain).is_some());
        let cached = cache.get(domain).unwrap();
        assert_eq!(cached.ipv4.len(), 1);
        assert_eq!(cached.ipv4[0].ip, "93.184.216.34");

        // Invalidate cache
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

        // Immediate access: should hit
        assert!(cache.get(domain).is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        // Access: should miss (expired)
        assert!(cache.get(domain).is_none());
    }

    #[test]
    fn test_cache_cleanup() {
        let cache = DNSCache::new(Duration::from_millis(100));

        // Add two domains
        let mut ips1 = DomainIPs::new();
        ips1.ipv4
            .push(crate::dns::types::IPInfo::new("1.1.1.1".to_string()));
        cache.put("domain1.com", ips1);

        let mut ips2 = DomainIPs::new();
        ips2.ipv4
            .push(crate::dns::types::IPInfo::new("2.2.2.2".to_string()));
        cache.put("domain2.com", ips2);

        // Verify statistics
        let (total, expired) = cache.stats();
        assert_eq!(total, 2);
        assert_eq!(expired, 0);

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));

        // Statistics before cleanup
        let (total, expired) = cache.stats();
        assert_eq!(total, 2);
        assert_eq!(expired, 2);

        // Cleanup expired entries
        let cleaned = cache.cleanup_expired();
        assert_eq!(cleaned, 2);

        // Statistics after cleanup
        let (total, _) = cache.stats();
        assert_eq!(total, 0);
    }

    #[test]
    fn test_cache_stats() {
        let cache = DNSCache::default();

        // Initial state
        let (total, expired) = cache.stats();
        assert_eq!(total, 0);
        assert_eq!(expired, 0);

        // Add an entry
        let mut ips = DomainIPs::new();
        ips.ipv4
            .push(crate::dns::types::IPInfo::new("1.1.1.1".to_string()));
        cache.put("example.com", ips);

        let (total, expired) = cache.stats();
        assert_eq!(total, 1);
        assert_eq!(expired, 0);
    }
}
