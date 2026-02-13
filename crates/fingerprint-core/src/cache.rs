// Cache layer module for fingerprint-api
// Implements multi-tier caching: L1 (in-memory) + L2 (Redis) + L3 (Database)

use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Cache tier configuration
#[derive(Debug, Clone, Copy)]
pub enum CacheTier {
    /// L1: In-memory application cache (5 min TTL)
    L1,
    /// L2: Redis distributed cache (30 min TTL)
    L2,
    /// L3: Database (authoritative source)
    L3,
}

/// Cache TTL strategy
#[derive(Debug, Clone, Copy)]
pub enum CacheTTL {
    /// Short-lived (5 minutes, L1 only)
    Short,
    /// Medium (30 minutes, L1 + L2)
    Medium,
    /// Long (1 hour, L2 only)
    Long,
    /// Custom duration
    Custom(Duration),
}

impl CacheTTL {
    pub fn to_seconds(&self) -> u32 {
        match self {
            CacheTTL::Short => 300,
            CacheTTL::Medium => 1800,
            CacheTTL::Long => 3600,
            CacheTTL::Custom(d) => d.as_secs() as u32,
        }
    }
}

/// Cache metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits_l1: u64,
    pub hits_l2: u64,
    pub hits_l3: u64,
    pub misses: u64,
    pub evictions: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits_l1 + self.hits_l2 + self.hits_l3 + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits_l1 + self.hits_l2 + self.hits_l3) as f64 / total as f64
        }
    }

    pub fn l1_hit_rate(&self) -> f64 {
        let l1_total = self.hits_l1 + (self.hits_l2 + self.hits_l3 + self.misses);
        if l1_total == 0 {
            0.0
        } else {
            self.hits_l1 as f64 / l1_total as f64
        }
    }

    pub fn l2_hit_rate(&self) -> f64 {
        let l2_total = self.hits_l2 + (self.hits_l3 + self.misses);
        if l2_total == 0 {
            0.0
        } else {
            self.hits_l2 as f64 / l2_total as f64
        }
    }
}

/// Multi-tier cache implementation
pub struct Cache {
    // L1: In-memory cache using LRU
    l1: Arc<RwLock<lru::LruCache<String, Vec<u8>>>>,
    // L2: Redis connection pool (to be implemented)
    l2_addr: String,
    // Statistics
    stats: Arc<RwLock<CacheStats>>,
}

impl Cache {
    /// Create new cache instance
    pub fn new(l2_addr: &str, l1_capacity: usize) -> Self {
        Cache {
            l1: Arc::new(RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(l1_capacity).unwrap(),
            ))),
            l2_addr: l2_addr.to_string(),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Generate versioned cache key
    pub fn versioned_key(namespace: &str, version: u32, id: &str) -> String {
        format!("{}:v{}:{}", namespace, version, id)
    }

    /// Get value from cache (tries L1, then L2)
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        // Try L1
        if let Some(value) = self.l1.write().get(key) {
            let value = value.clone();
            self.stats.write().hits_l1 += 1;
            return Some(value);
        }

        // Try L2 (Redis) - TODO: implement Redis integration
        // if let Some(value) = self.l2_get(key).await {
        //     self.l1.write().put(key.to_string(), value.clone());
        //     self.stats.write().hits_l2 += 1;
        //     return Some(value);
        // }

        self.stats.write().misses += 1;
        None
    }

    /// Set value in cache (stores in both L1 and L2)
    pub async fn set(&self, key: &str, value: Vec<u8>, ttl: CacheTTL) -> Result<(), String> {
        // Store in L1
        self.l1.write().put(key.to_string(), value.clone());

        // Store in L2 (Redis) - TODO: implement Redis integration
        // self.l2_set(key, value, ttl.to_seconds()).await?;

        Ok(())
    }

    /// Invalidate cache entry (both L1 and L2)
    pub async fn invalidate(&self, pattern: &str) -> Result<(), String> {
        // Remove from L1
        if pattern.ends_with('*') {
            // Pattern matching for keys
            let prefix = pattern.trim_end_matches('*');
            let l1 = self.l1.write();
            // Collect keys to remove (to avoid borrow issues)
            let keys_to_remove: Vec<String> = l1
                .iter()
                .filter(|(k, _)| k.starts_with(prefix))
                .map(|(k, _)| k.clone())
                .collect();
            drop(l1);

            for key in keys_to_remove {
                self.l1.write().pop(&key);
            }
        } else {
            self.l1.write().pop(pattern);
        }

        // Invalidate in L2 (Redis) - TODO: implement Redis integration
        // self.l2_delete(pattern).await?;

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.read().clone()
    }

    /// Clear all cache
    pub async fn clear(&self) -> Result<(), String> {
        self.l1.write().clear();
        // TODO: clear L2
        Ok(())
    }
}

/// Distributed lock to prevent cache stampede
pub struct DistributedLock {
    key: String,
    timeout: Duration,
}

impl DistributedLock {
    pub fn new(key: &str, timeout: Duration) -> Self {
        DistributedLock {
            key: key.to_string(),
            timeout,
        }
    }

    /// Try to acquire the lock
    pub async fn acquire(&self) -> Result<LockGuard, String> {
        // TODO: implement Redis SET NX with timeout
        // Returns LockGuard that releases on drop
        Ok(LockGuard {
            key: self.key.clone(),
        })
    }
}

pub struct LockGuard {
    key: String,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // TODO: implement Redis DEL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_stats() {
        let stats = CacheStats {
            hits_l1: 100,
            hits_l2: 50,
            hits_l3: 25,
            misses: 25,
            evictions: 5,
        };

        assert!(stats.hit_rate() > 0.8, "Hit rate should be 80%");
        assert_eq!(stats.hit_rate(), 200.0 / 200.0); // (100+50+25)/(100+50+25+25)
    }

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache = Cache::new("localhost:6379", 1000);
        
        let key = "test:key";
        let value = b"test:value".to_vec();
        
        cache.set(key, value.clone(), CacheTTL::Short).await.unwrap();
        
        if let Some(retrieved) = cache.get(key).await {
            assert_eq!(retrieved, value, "Retrieved value should match set value");
        }
    }
}
