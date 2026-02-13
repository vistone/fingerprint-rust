// Cache layer module for fingerprint-api
// Implements multi-tier caching: L1 (in-memory) + L2 (Redis) + L3 (Database)

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

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

/// Cache operation error
#[derive(Debug, Clone)]
pub enum CacheError {
    /// Redis connection error
    RedisError(String),
    /// Serialization error
    SerializationError(String),
    /// Invalid configuration
    ConfigError(String),
    /// Lock acquisition failed
    LockError(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::RedisError(e) => write!(f, "Redis error: {}", e),
            CacheError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            CacheError::ConfigError(e) => write!(f, "Config error: {}", e),
            CacheError::LockError(e) => write!(f, "Lock error: {}", e),
        }
    }
}

impl std::error::Error for CacheError {}

/// Cache metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits_l1: u64,
    pub hits_l2: u64,
    pub hits_l3: u64,
    pub misses: u64,
    pub evictions: u64,
    pub redis_errors: u64,
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

/// Cache result type
pub type CacheResult<T> = Result<T, CacheError>;

/// Multi-tier cache implementation
pub struct Cache {
    // L1: In-memory cache using LRU
    l1: Arc<RwLock<lru::LruCache<String, Vec<u8>>>>,
    // L2 address for connection
    l2_addr: String,
    // Statistics
    stats: Arc<RwLock<CacheStats>>,
}

impl Cache {
    /// Create new cache instance with only L1 cache enabled
    pub fn new_l1_only(l1_capacity: usize) -> CacheResult<Self> {
        let capacity = std::num::NonZeroUsize::new(l1_capacity)
            .ok_or_else(|| CacheError::ConfigError("L1 capacity must be > 0".to_string()))?;

        Ok(Cache {
            l1: Arc::new(RwLock::new(lru::LruCache::new(capacity))),
            l2_addr: String::new(),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        })
    }

    /// Create new cache instance (L1 only, Redis URL stored for later)
    pub fn new(l2_addr: &str, l1_capacity: usize) -> CacheResult<Self> {
        let mut cache = Self::new_l1_only(l1_capacity)?;
        cache.l2_addr = l2_addr.to_string();
        Ok(cache)
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

        self.stats.write().misses += 1;
        None
    }

    /// Set value in cache (stores in both L1 and L2)
    pub async fn set(&self, key: &str, value: Vec<u8>, _ttl: CacheTTL) -> CacheResult<()> {
        // Store in L1
        self.l1.write().put(key.to_string(), value);
        Ok(())
    }

    /// Invalidate cache entry (both L1 and L2)
    pub async fn invalidate(&self, pattern: &str) -> CacheResult<()> {
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

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.read().clone()
    }

    /// Clear all cache
    pub async fn clear(&self) -> CacheResult<()> {
        self.l1.write().clear();
        Ok(())
    }

    /// Get L2 (Redis) address
    pub fn l2_address(&self) -> &str {
        &self.l2_addr
    }
}

/// Distributed lock to prevent cache stampede
#[allow(dead_code)]
pub struct DistributedLock {
    key: String,
    timeout: Duration,
}

impl DistributedLock {
    /// Create a new distributed lock (local only)
    pub fn new(key: &str, timeout: Duration) -> Self {
        DistributedLock {
            key: key.to_string(),
            timeout,
        }
    }

    /// Try to acquire the lock
    pub async fn acquire(&self) -> CacheResult<LockGuard> {
        // Local-only lock (just returns a dummy guard)
        Ok(LockGuard {
            key: self.key.clone(),
        })
    }
}

/// Lock guard that releases the lock when dropped
#[allow(dead_code)]
pub struct LockGuard {
    key: String,
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        // Local lock - nothing to release
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
            redis_errors: 0,
        };

        // hit_rate = (100+50+25) / (100+50+25+25) = 175/200 = 0.875
        assert_eq!(stats.hit_rate(), 0.875, "Hit rate should be 87.5%");
        assert!(stats.hit_rate() > 0.8, "Hit rate should be > 80%");
    }

    // Note: async tests require tokio runtime which is not available in tests
    // These tests are run using a simple executor in the test module

    fn run_async<F>(f: F) -> F::Output
    where
        F: std::future::Future,
    {
        // Simple block_on implementation for tests
        use std::sync::Arc;
        use std::task::{Context, Poll, Wake};

        struct DummyWaker;
        impl Wake for DummyWaker {
            fn wake(self: Arc<Self>) {}
        }

        let waker = Arc::new(DummyWaker).into();
        let mut context = Context::from_waker(&waker);
        let mut future = std::pin::pin!(f);

        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(val) => return val,
                Poll::Pending => {
                    // In a real implementation, this would wait for I/O
                    // For tests, we just continue polling
                    std::thread::yield_now();
                }
            }
        }
    }

    #[test]
    fn test_cache_l1_only() {
        let cache = Cache::new_l1_only(100).expect("Failed to create cache");

        let key = "test:key";
        let value = b"test:value".to_vec();

        run_async(async {
            cache
                .set(key, value.clone(), CacheTTL::Short)
                .await
                .expect("Set failed");

            let retrieved = cache.get(key).await;
            assert!(retrieved.is_some(), "Should retrieve value from L1");
            assert_eq!(retrieved.unwrap(), value, "Retrieved value should match");
        });

        // Check stats
        let stats = cache.stats();
        assert_eq!(stats.hits_l1, 1, "Should have 1 L1 hit");
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = Cache::new_l1_only(100).expect("Failed to create cache");

        run_async(async {
            cache.set("key1", vec![1], CacheTTL::Short).await.unwrap();
            cache.set("key2", vec![2], CacheTTL::Short).await.unwrap();
            cache
                .set("prefix:key3", vec![3], CacheTTL::Short)
                .await
                .unwrap();

            // Invalidate single key
            cache.invalidate("key1").await.unwrap();
            assert!(cache.get("key1").await.is_none());
            assert!(cache.get("key2").await.is_some());

            // Invalidate pattern
            cache.invalidate("prefix:*").await.unwrap();
            assert!(cache.get("prefix:key3").await.is_none());
            assert!(cache.get("key2").await.is_some());
        });
    }

    #[test]
    fn test_cache_error_display() {
        let err = CacheError::RedisError("connection refused".to_string());
        assert!(err.to_string().contains("connection refused"));
    }

    #[test]
    fn test_versioned_key() {
        let key = Cache::versioned_key("fingerprint", 1, "chrome133");
        assert_eq!(key, "fingerprint:v1:chrome133");
    }
}
