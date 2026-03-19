use async_trait::async_trait;
use fingerprint_core::{DistributedRateLimitBackend, QuotaTier, UserQuota};
use std::time::Duration;

const DEFAULT_REDIS_POOL_SIZE: u32 = 10;
const DEFAULT_REDIS_TIMEOUT_SECS: u64 = 5;
const DEFAULT_REDIS_COMMAND_TIMEOUT_SECS: u64 = 2;

#[derive(Debug, Clone)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub timeout: Duration,
    pub command_timeout: Duration,
}

impl RedisConfig {
    pub fn new(url: String) -> Self {
        Self {
            url,
            pool_size: DEFAULT_REDIS_POOL_SIZE,
            timeout: Duration::from_secs(DEFAULT_REDIS_TIMEOUT_SECS),
            command_timeout: Duration::from_secs(DEFAULT_REDIS_COMMAND_TIMEOUT_SECS),
        }
    }

    pub fn with_pool_size(mut self, size: u32) -> Self {
        self.pool_size = size;
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[derive(Debug, Clone)]
pub enum RedisBackendError {
    ConnectionError(String),
    CommandError(String),
    SerializationError(String),
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

pub type RedisResult<T> = Result<T, RedisBackendError>;

pub struct RedisRateLimitBackend {
    config: RedisConfig,
}

impl RedisRateLimitBackend {
    pub fn new(config: RedisConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &RedisConfig {
        &self.config
    }

    #[allow(dead_code)]
    fn quota_key(&self, user_id: &str) -> String {
        format!("rl:quota:{}", user_id)
    }

    #[allow(dead_code)]
    fn counter_key(&self, user_id: &str, month: u32) -> String {
        format!("rl:counter:{}:{}", user_id, month)
    }

    #[allow(dead_code)]
    fn metric_key(&self, metric_name: &str) -> String {
        format!("rl:metric:{}", metric_name)
    }

    #[allow(dead_code)]
    fn serialize_quota(&self, entry: &RedisQuotaEntry) -> RedisResult<String> {
        serde_json::to_string(entry)
            .map_err(|e| RedisBackendError::SerializationError(e.to_string()))
    }

    #[allow(dead_code)]
    fn deserialize_quota(&self, json: &str) -> RedisResult<RedisQuotaEntry> {
        serde_json::from_str(json).map_err(|e| RedisBackendError::SerializationError(e.to_string()))
    }

    pub async fn increment_request_count(
        &self,
        _user_id: &str,
        _month: u32,
        _ttl_seconds: u64,
    ) -> RedisResult<u64> {
        Ok(0)
    }

    pub async fn push_metric(
        &self,
        _metric_name: &str,
        _value: f64,
        _max_entries: usize,
    ) -> RedisResult<()> {
        Ok(())
    }

    pub async fn clear_user_quota(&self, _user_id: &str) -> RedisResult<()> {
        Ok(())
    }

    pub async fn clear_all(&self) -> RedisResult<()> {
        Ok(())
    }

    pub async fn get_request_count(&self, _user_id: &str, _month: u32) -> RedisResult<u64> {
        Ok(0)
    }
}

impl Default for RedisRateLimitBackend {
    fn default() -> Self {
        Self::new(RedisConfig::new("redis://localhost:6379".to_string()))
    }
}

#[async_trait]
impl DistributedRateLimitBackend for RedisRateLimitBackend {
    async fn get_user_quota(&self, _user_id: &str) -> Result<Option<UserQuota>, String> {
        Ok(None)
    }

    async fn set_user_quota(
        &self,
        _user_id: &str,
        quota: &UserQuota,
        _ttl_seconds: u64,
    ) -> Result<(), String> {
        let entry = RedisQuotaEntry::from_user_quota(quota.user_id.clone(), quota);
        self.serialize_quota(&entry)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    async fn health_check(&self) -> Result<bool, String> {
        Ok(true)
    }
}

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

    pub fn from_user_quota(user_id: String, quota: &UserQuota) -> Self {
        Self {
            user_id,
            available_tokens: quota.available_tokens,
            last_refill: quota.last_refill,
            month_requests: quota.month_requests,
            month_start: quota.month_start,
            tier: format!("{:?}", quota.tier),
        }
    }

    pub fn to_user_quota(&self) -> UserQuota {
        let tier = match self.tier.as_str() {
            "Pro" => QuotaTier::Pro,
            "Enterprise" => QuotaTier::Enterprise,
            "Partner" => QuotaTier::Partner,
            _ => QuotaTier::Free,
        };

        let mut quota = UserQuota::new(self.user_id.clone(), tier);
        quota.available_tokens = self.available_tokens;
        quota.last_refill = self.last_refill;
        quota.month_requests = self.month_requests;
        quota.month_start = self.month_start;
        quota
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
        let user_quota = UserQuota::new("user123".to_string(), QuotaTier::Pro);
        let entry = RedisQuotaEntry::from_user_quota("user123".to_string(), &user_quota);

        assert_eq!(entry.user_id, "user123");
        assert_eq!(entry.available_tokens, 1000.0);
        assert_eq!(entry.tier, "Pro");
    }

    #[test]
    fn test_quota_entry_to_user_quota() {
        let entry = RedisQuotaEntry::new(
            "user123".to_string(),
            100.0,
            1234567890,
            50,
            1234567890,
            "Pro".to_string(),
        );

        let quota = entry.to_user_quota();
        assert_eq!(quota.user_id, "user123");
        assert_eq!(quota.available_tokens, 100.0);
        assert_eq!(quota.month_requests, 50);
        assert_eq!(quota.tier, QuotaTier::Pro);
    }

    #[test]
    fn test_error_display() {
        let err = RedisBackendError::ConnectionError("refused".to_string());
        assert!(err.to_string().contains("refused"));

        let err = RedisBackendError::CommandError("syntax error".to_string());
        assert!(err.to_string().contains("syntax error"));
    }
}
