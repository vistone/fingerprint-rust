/// Redis Integration for Rate Limiting
///
/// Provides connection pooling and distributed quota management via Redis.
/// Used by the RateLimiter service for shared state across multiple gateway instances.
use std::time::Duration;

// Redis 配置常量
const DEFAULT_REDIS_POOL_SIZE: u32 = 10;  // 默认连接池大小
const DEFAULT_REDIS_TIMEOUT_SECS: u64 = 5;  // 默认超时（秒）
const DEFAULT_REDIS_COMMAND_TIMEOUT_SECS: u64 = 2;  // 默认命令超时（秒）

/// Redis connection configuration
#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// Redis server connection URL (e.g., redis://localhost:6379)
    pub url: String,
    /// Connection pool size
    pub pool_size: u32,
    /// Connection timeout
    pub timeout: Duration,
    /// Command timeout
    pub command_timeout: Duration,
}

impl RedisConfig {
    /// Create new Redis configuration
    pub fn new(url: String) -> Self {
        Self {
            url,
            pool_size: DEFAULT_REDIS_POOL_SIZE,
            timeout: Duration::from_secs(DEFAULT_REDIS_TIMEOUT_SECS),
            command_timeout: Duration::from_secs(DEFAULT_REDIS_COMMAND_TIMEOUT_SECS),
        }
    }

    /// Set pool size for connections
    pub fn with_pool_size(mut self, size: u32) -> Self {
        self.pool_size = size;
        self
    }

    /// Set connection timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// Redis backend error types
#[derive(Debug, Clone)]
pub enum RedisBackendError {
    /// Connection failed
    ConnectionError(String),
    /// Command execution failed
    CommandError(String),
    /// Serialization failed
    SerializationError(String),
    /// Timeout
    TimeoutError(String),
}

impl std::fmt::Display for RedisBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RedisBackendError::ConnectionError(e) => write!(f, "Redis connection error: {}", e),
            RedisBackendError::CommandError(e) => write!(f, "Redis command error: {}", e),
            RedisBackendError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            RedisBackendError::TimeoutError(e) => write!(f, "Redis timeout: {}", e),
        }
    }
}

impl std::error::Error for RedisBackendError {}

/// Result type for Redis backend operations
pub type RedisResult<T> = Result<T, RedisBackendError>;

/// Distributed rate limit cache backend
pub struct RedisRateLimitBackend {
    config: RedisConfig,
    // Note: Actual Redis connection would be stored here
    // For now, we use a mock implementation that can be replaced with real Redis
}

impl RedisRateLimitBackend {
    /// Create new Redis backend
    pub fn new(config: RedisConfig) -> Self {
        Self { config }
    }

    /// Get configuration
    pub fn config(&self) -> &RedisConfig {
        &self.config
    }

    /// Get Redis key prefix for quota entries
    #[allow(dead_code)]
    fn quota_key(&self, user_id: &str) -> String {
        format!("rl:quota:{}", user_id)
    }

    /// Get Redis key for request counters
    #[allow(dead_code)]
    fn counter_key(&self, user_id: &str, month: u32) -> String {
        format!("rl:counter:{}:{}", user_id, month)
    }

    /// Get Redis key for metrics
    #[allow(dead_code)]
    fn metric_key(&self, metric_name: &str) -> String {
        format!("rl:metric:{}", metric_name)
    }

    /// Serialize quota entry to JSON
    #[allow(dead_code)]
    fn serialize_quota(&self, entry: &RedisQuotaEntry) -> RedisResult<String> {
        serde_json::to_string(entry)
            .map_err(|e| RedisBackendError::SerializationError(e.to_string()))
    }

    /// Deserialize quota entry from JSON
    #[allow(dead_code)]
    fn deserialize_quota(&self, json: &str) -> RedisResult<RedisQuotaEntry> {
        serde_json::from_str(json).map_err(|e| RedisBackendError::SerializationError(e.to_string()))
    }

    /// Check if user quota is in Redis (distributed cache)
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    ///
    /// # Returns
    /// * `Ok(Some(quota))` - Quota found
    /// * `Ok(None)` - Quota not found
    /// * `Err(e)` - Redis error
    pub async fn get_user_quota(&self, _user_id: &str) -> RedisResult<Option<RedisQuotaEntry>> {
        // Placeholder implementation - replace with actual Redis call
        // Example:
        // let mut conn = self.get_connection().await?;
        // let result: Option<String> = redis::cmd("GET")
        //     .arg(self.quota_key(user_id))
        //     .query_async(&mut conn)
        //     .await
        //     .map_err(|e| RedisBackendError::CommandError(e.to_string()))?;
        //
        // match result {
        //     Some(json) => Ok(Some(self.deserialize_quota(&json)?)),
        //     None => Ok(None),
        // }

        Ok(None)
    }

    /// Store user quota in Redis (distributed cache)
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `entry` - Quota entry to store
    /// * `ttl_seconds` - Time to live in seconds
    pub async fn set_user_quota(
        &self,
        _user_id: &str,
        _entry: &RedisQuotaEntry,
        _ttl_seconds: u64,
    ) -> RedisResult<()> {
        // Placeholder implementation - replace with actual Redis call
        // Example:
        // let mut conn = self.get_connection().await?;
        // let json = self.serialize_quota(entry)?;
        // redis::cmd("SETEX")
        //     .arg(self.quota_key(user_id))
        //     .arg(ttl_seconds)
        //     .arg(json)
        //     .query_async::<_, ()>(&mut conn)
        //     .await
        //     .map_err(|e| RedisBackendError::CommandError(e.to_string()))?;

        Ok(())
    }

    /// Increment request counter for user
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `month` - Current month (for key scoping)
    /// * `ttl_seconds` - TTL for the counter key
    ///
    /// # Returns
    /// * New counter value
    pub async fn increment_request_count(
        &self,
        _user_id: &str,
        _month: u32,
        _ttl_seconds: u64,
    ) -> RedisResult<u64> {
        // Placeholder implementation
        // Example:
        // let mut conn = self.get_connection().await?;
        // let key = self.counter_key(user_id, month);
        // let count: u64 = redis::cmd("INCR")
        //     .arg(&key)
        //     .query_async(&mut conn)
        //     .await
        //     .map_err(|e| RedisBackendError::CommandError(e.to_string()))?;
        //
        // // Set expiration on first increment
        // if count == 1 {
        //     let _: () = redis::cmd("EXPIRE")
        //         .arg(&key)
        //         .arg(ttl_seconds)
        //         .query_async(&mut conn)
        //         .await
        //         .map_err(|e| RedisBackendError::CommandError(e.to_string()))?;
        // }

        Ok(0)
    }

    /// Store rate limit metrics in Redis
    ///
    /// # Arguments
    /// * `metric_name` - Name of the metric
    /// * `value` - Metric value
    /// * `max_entries` - Maximum number of entries to keep (for list trimming)
    pub async fn push_metric(
        &self,
        _metric_name: &str,
        _value: f64,
        _max_entries: usize,
    ) -> RedisResult<()> {
        // Placeholder implementation
        // Example:
        // let mut conn = self.get_connection().await?;
        // let key = self.metric_key(metric_name);
        // let timestamp = chrono::Utc::now().timestamp() as f64;
        // let entry = format!("{}:{}", timestamp, value);
        //
        // redis::cmd("LPUSH")
        //     .arg(&key)
        //     .arg(entry)
        //     .query_async::<_, ()>(&mut conn)
        //     .await
        //     .map_err(|e| RedisBackendError::CommandError(e.to_string()))?;
        //
        // // Trim list to max_entries
        // redis::cmd("LTRIM")
        //     .arg(&key)
        //     .arg(0)
        //     .arg(max_entries - 1)
        //     .query_async::<_, ()>(&mut conn)
        //     .await
        //     .map_err(|e| RedisBackendError::CommandError(e.to_string()))?;

        Ok(())
    }

    /// Health check - verify Redis connectivity
    ///
    /// # Returns
    /// * `Ok(true)` - Redis is healthy
    /// * `Ok(false)` - Redis is not responding
    /// * `Err(e)` - Connection error
    pub async fn health_check(&self) -> RedisResult<bool> {
        // Placeholder implementation
        // Example:
        // let mut conn = self.get_connection().await?;
        // let response: String = redis::cmd("PING")
        //     .query_async(&mut conn)
        //     .await
        //     .map_err(|e| RedisBackendError::CommandError(e.to_string()))?;
        // Ok(response == "PONG")

        Ok(true)
    }

    /// Clear quota cache for user
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    pub async fn clear_user_quota(&self, _user_id: &str) -> RedisResult<()> {
        // Placeholder implementation
        // Example:
        // let mut conn = self.get_connection().await?;
        // redis::cmd("DEL")
        //     .arg(self.quota_key(user_id))
        //     .query_async::<_, ()>(&mut conn)
        //     .await
        //     .map_err(|e| RedisBackendError::CommandError(e.to_string()))?;

        Ok(())
    }

    /// Clear all rate limiting data (use with caution!)
    pub async fn clear_all(&self) -> RedisResult<()> {
        // Placeholder implementation
        // Example:
        // let mut conn = self.get_connection().await?;
        // let keys: Vec<String> = redis::cmd("KEYS")
        //     .arg("rl:*")
        //     .query_async(&mut conn)
        //     .await
        //     .map_err(|e| RedisBackendError::CommandError(e.to_string()))?;
        //
        // if !keys.is_empty() {
        //     redis::cmd("DEL")
        //         .arg(&keys)
        //         .query_async::<_, ()>(&mut conn)
        //         .await
        //         .map_err(|e| RedisBackendError::CommandError(e.to_string()))?;
        // }

        Ok(())
    }

    /// Get current request count for user
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `month` - Current month
    pub async fn get_request_count(&self, _user_id: &str, _month: u32) -> RedisResult<u64> {
        // Placeholder implementation
        // Example:
        // let mut conn = self.get_connection().await?;
        // let count: Option<u64> = redis::cmd("GET")
        //     .arg(self.counter_key(user_id, month))
        //     .query_async(&mut conn)
        //     .await
        //     .map_err(|e| RedisBackendError::CommandError(e.to_string()))?;
        // Ok(count.unwrap_or(0))

        Ok(0)
    }
}

impl Default for RedisRateLimitBackend {
    fn default() -> Self {
        Self::new(RedisConfig::new("redis://localhost:6379".to_string()))
    }
}

/// Rate limit state stored in Redis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RedisQuotaEntry {
    pub user_id: String,
    pub available_tokens: f64,
    pub last_refill: u64,
    pub month_requests: u64,
    pub month_start: u64,
    pub tier: String,
}

impl RedisQuotaEntry {
    /// Create a new quota entry
    pub fn new(
        user_id: String,
        available_tokens: f64,
        last_refill: u64,
        month_requests: u64,
        month_start: u64,
        tier: String,
    ) -> Self {
        Self {
            user_id,
            available_tokens,
            last_refill,
            month_requests,
            month_start,
            tier,
        }
    }

    /// Convert from UserQuota
    pub fn from_user_quota(user_id: String, quota: &super::UserQuota) -> Self {
        Self {
            user_id,
            available_tokens: quota.available_tokens,
            last_refill: quota.last_refill,
            month_requests: quota.month_requests,
            month_start: quota.month_start,
            tier: format!("{:?}", quota.tier),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_config_creation() {
        let config = RedisConfig::new("redis://localhost:6379".to_string());
        assert_eq!(config.url, "redis://localhost:6379");
        assert_eq!(config.pool_size, 10);
        assert_eq!(config.timeout.as_secs(), 5);
    }

    #[test]
    fn test_redis_config_customization() {
        let config = RedisConfig::new("redis://localhost:6379".to_string())
            .with_pool_size(20)
            .with_timeout(Duration::from_secs(10));

        assert_eq!(config.pool_size, 20);
        assert_eq!(config.timeout.as_secs(), 10);
    }

    #[test]
    fn test_redis_backend_creation() {
        let config = RedisConfig::new("redis://localhost:6379".to_string());
        let backend = RedisRateLimitBackend::new(config);
        assert_eq!(backend.config().url, "redis://localhost:6379");
    }

    #[test]
    fn test_backend_default() {
        let backend = RedisRateLimitBackend::default();
        assert_eq!(backend.config().url, "redis://localhost:6379");
    }

    #[test]
    fn test_key_generation() {
        let backend = RedisRateLimitBackend::default();
        assert_eq!(backend.quota_key("user123"), "rl:quota:user123");
        assert_eq!(backend.counter_key("user123", 1), "rl:counter:user123:1");
        assert_eq!(backend.metric_key("requests"), "rl:metric:requests");
    }

    #[test]
    fn test_quota_serialization() {
        let backend = RedisRateLimitBackend::default();
        let entry = RedisQuotaEntry::new(
            "user123".to_string(),
            100.0,
            1234567890,
            50,
            1234567890,
            "Pro".to_string(),
        );

        let json = backend.serialize_quota(&entry).unwrap();
        let deserialized = backend.deserialize_quota(&json).unwrap();

        assert_eq!(deserialized.user_id, "user123");
        assert_eq!(deserialized.available_tokens, 100.0);
        assert_eq!(deserialized.month_requests, 50);
    }

    #[test]
    fn test_quota_entry_from_user_quota() {
        use super::super::{QuotaTier, UserQuota};

        let user_quota = UserQuota::new("user123".to_string(), QuotaTier::Pro);
        let entry = RedisQuotaEntry::from_user_quota("user123".to_string(), &user_quota);

        assert_eq!(entry.user_id, "user123");
        assert_eq!(entry.available_tokens, 1000.0); // Pro tier minute limit
        assert_eq!(entry.tier, "Pro");
    }

    #[test]
    fn test_error_display() {
        let err = RedisBackendError::ConnectionError("refused".to_string());
        assert!(err.to_string().contains("refused"));

        let err = RedisBackendError::CommandError("syntax error".to_string());
        assert!(err.to_string().contains("syntax error"));
    }
}
