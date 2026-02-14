/// Redis-backed cache implementation
///
/// Provides L2 distributed caching using Redis.
/// This module is only available when the `redis-cache` feature is enabled.
#[cfg(feature = "redis-cache")]
pub mod redis_impl {
    use super::super::{CacheError, CacheResult, CacheTTL};
    use std::time::Duration;

    /// Redis cache configuration
    #[derive(Debug, Clone)]
    pub struct RedisCacheConfig {
        /// Redis connection URL
        pub url: String,
        /// Connection timeout
        pub connection_timeout: Duration,
        /// Command timeout
        pub command_timeout: Duration,
        /// Maximum retries
        pub max_retries: u32,
        /// Retry delay
        pub retry_delay: Duration,
    }

    impl Default for RedisCacheConfig {
        fn default() -> Self {
            Self {
                url: "redis://localhost:6379".to_string(),
                connection_timeout: Duration::from_secs(5),
                command_timeout: Duration::from_secs(2),
                max_retries: 3,
                retry_delay: Duration::from_millis(100),
            }
        }
    }

    impl RedisCacheConfig {
        /// Create new config with URL
        pub fn new(url: String) -> Self {
            Self {
                url,
                ..Default::default()
            }
        }

        /// Set connection timeout
        pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
            self.connection_timeout = timeout;
            self
        }

        /// Set max retries
        pub fn with_max_retries(mut self, retries: u32) -> Self {
            self.max_retries = retries;
            self
        }
    }

    /// Redis cache backend
    pub struct RedisCache {
        config: RedisCacheConfig,
        connection: Option<redis::aio::ConnectionManager>,
    }

    impl RedisCache {
        /// Create new Redis cache (without connecting)
        pub fn new(config: RedisCacheConfig) -> Self {
            Self {
                config,
                connection: None,
            }
        }

        /// Connect to Redis
        pub async fn connect(&mut self) -> CacheResult<()> {
            let client = redis::Client::open(&self.config.url[..])
                .map_err(|e| CacheError::RedisError(format!("Failed to create client: {}", e)))?;

            let conn = client
                .get_connection_manager()
                .await
                .map_err(|e| CacheError::RedisError(format!("Failed to connect: {}", e)))?;

            self.connection = Some(conn);
            Ok(())
        }

        /// Check if connected
        pub fn is_connected(&self) -> bool {
            self.connection.is_some()
        }

        /// Get value from Redis
        pub async fn get(&self, key: &str) -> CacheResult<Option<Vec<u8>>> {
            let conn = self
                .connection
                .as_ref()
                .ok_or_else(|| CacheError::RedisError("Not connected".to_string()))?;

            let mut conn = conn.clone();
            let result: Option<Vec<u8>> =
                redis::cmd("GET")
                    .arg(key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| CacheError::RedisError(format!("GET failed: {}", e)))?;

            Ok(result)
        }

        /// Set value in Redis with TTL
        pub async fn set(&self, key: &str, value: &[u8], ttl: CacheTTL) -> CacheResult<()> {
            let conn = self
                .connection
                .as_ref()
                .ok_or_else(|| CacheError::RedisError("Not connected".to_string()))?;

            let mut conn = conn.clone();
            let ttl_secs = ttl.to_seconds() as u64;

            redis::cmd("SETEX")
                .arg(key)
                .arg(ttl_secs)
                .arg(value)
                .query_async::<_, ()>(&mut conn)
                .await
                .map_err(|e| CacheError::RedisError(format!("SETEX failed: {}", e)))?;

            Ok(())
        }

        /// Delete key from Redis
        pub async fn delete(&self, key: &str) -> CacheResult<()> {
            let conn = self
                .connection
                .as_ref()
                .ok_or_else(|| CacheError::RedisError("Not connected".to_string()))?;

            let mut conn = conn.clone();
            redis::cmd("DEL")
                .arg(key)
                .query_async::<_, ()>(&mut conn)
                .await
                .map_err(|e| CacheError::RedisError(format!("DEL failed: {}", e)))?;

            Ok(())
        }

        /// Delete keys matching pattern
        pub async fn delete_pattern(&self, pattern: &str) -> CacheResult<u64> {
            let conn = self
                .connection
                .as_ref()
                .ok_or_else(|| CacheError::RedisError("Not connected".to_string()))?;

            let mut conn = conn.clone();

            // Get keys matching pattern
            let keys: Vec<String> = redis::cmd("KEYS")
                .arg(pattern)
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::RedisError(format!("KEYS failed: {}", e)))?;

            if keys.is_empty() {
                return Ok(0);
            }

            // Delete keys
            let deleted: u64 = redis::cmd("DEL")
                .arg(&keys)
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::RedisError(format!("DEL failed: {}", e)))?;

            Ok(deleted)
        }

        /// Clear all cache keys with prefix
        pub async fn clear_with_prefix(&self, prefix: &str) -> CacheResult<u64> {
            let pattern = format!("{}:*", prefix);
            self.delete_pattern(&pattern).await
        }

        /// Health check
        pub async fn health_check(&self) -> CacheResult<bool> {
            if self.connection.is_none() {
                return Ok(false);
            }

            let mut conn = self.connection.as_ref().unwrap().clone();
            let result: String = redis::cmd("PING")
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::RedisError(format!("PING failed: {}", e)))?;

            Ok(result == "PONG")
        }

        /// Get Redis info
        pub async fn info(&self) -> CacheResult<String> {
            let conn = self
                .connection
                .as_ref()
                .ok_or_else(|| CacheError::RedisError("Not connected".to_string()))?;

            let mut conn = conn.clone();
            let info: String = redis::cmd("INFO")
                .arg("memory")
                .query_async(&mut conn)
                .await
                .map_err(|e| CacheError::RedisError(format!("INFO failed: {}", e)))?;

            Ok(info)
        }
    }

    /// Redis cluster cache configuration
    #[derive(Debug, Clone)]
    pub struct RedisClusterConfig {
        /// Cluster node URLs
        pub nodes: Vec<String>,
        /// Connection timeout
        pub connection_timeout: Duration,
        /// Command timeout
        pub command_timeout: Duration,
        /// Read from replicas
        pub read_from_replicas: bool,
    }

    impl Default for RedisClusterConfig {
        fn default() -> Self {
            Self {
                nodes: vec![
                    "redis://127.0.0.1:7000".to_string(),
                    "redis://127.0.0.1:7001".to_string(),
                    "redis://127.0.0.1:7002".to_string(),
                ],
                connection_timeout: Duration::from_secs(5),
                command_timeout: Duration::from_secs(2),
                read_from_replicas: true,
            }
        }
    }

    impl RedisClusterConfig {
        /// Create new cluster config with nodes
        pub fn new(nodes: Vec<String>) -> Self {
            Self {
                nodes,
                ..Default::default()
            }
        }
    }

    /// Redis cluster cache backend
    /// Note: Requires redis-cluster feature to be enabled
    pub struct RedisClusterCache {
        #[allow(dead_code)]
        config: RedisClusterConfig,
    }

    impl RedisClusterCache {
        /// Create new cluster cache (placeholder)
        /// Actual implementation requires redis-cluster crate
        pub fn new(config: RedisClusterConfig) -> Self {
            Self { config }
        }

        /// Connect to cluster
        pub async fn connect(&mut self) -> CacheResult<()> {
            // Placeholder - requires redis-cluster crate
            Ok(())
        }

        /// Get value (placeholder)
        pub async fn get(&self, _key: &str) -> CacheResult<Option<Vec<u8>>> {
            // Placeholder implementation
            Ok(None)
        }

        /// Set value (placeholder)
        pub async fn set(&self, _key: &str, _value: &[u8], _ttl: CacheTTL) -> CacheResult<()> {
            // Placeholder implementation
            Ok(())
        }
    }
}

#[cfg(not(feature = "redis-cache"))]
pub mod redis_impl {
    use std::time::Duration;

    /// Placeholder config when redis-cache feature is disabled
    #[derive(Debug, Clone)]
    pub struct RedisCacheConfig {
        pub url: String,
        pub connection_timeout: Duration,
        pub command_timeout: Duration,
        pub max_retries: u32,
        pub retry_delay: Duration,
    }

    impl Default for RedisCacheConfig {
        fn default() -> Self {
            Self {
                url: "redis://localhost:6379".to_string(),
                connection_timeout: Duration::from_secs(5),
                command_timeout: Duration::from_secs(2),
                max_retries: 3,
                retry_delay: Duration::from_millis(100),
            }
        }
    }

    impl RedisCacheConfig {
        /// Create new config with URL
        pub fn new(url: String) -> Self {
            Self {
                url,
                ..Default::default()
            }
        }

        /// Set max retries
        pub fn with_max_retries(mut self, retries: u32) -> Self {
            self.max_retries = retries;
            self
        }
    }
}

// Re-exports
#[cfg(feature = "redis-cache")]
pub use redis_impl::{RedisCache, RedisCacheConfig, RedisClusterCache, RedisClusterConfig};

#[cfg(not(feature = "redis-cache"))]
pub use redis_impl::RedisCacheConfig;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_config_default() {
        let config = redis_impl::RedisCacheConfig::default();
        assert_eq!(config.url, "redis://localhost:6379");
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_redis_config_builder() {
        let config = redis_impl::RedisCacheConfig::new("redis://example.com:6379".to_string())
            .with_max_retries(5);

        assert_eq!(config.url, "redis://example.com:6379");
        assert_eq!(config.max_retries, 5);
    }
}
