use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
/// Rate Limiting Service Implementation
///
/// Implements distributed rate limiting using Redis backend with token bucket algorithm.
/// Supports multiple quota tiers and per-endpoint rate limits.
///
/// # Features
/// - Token bucket algorithm with configurable burst size
/// - Redis-backed distributed state
/// - Multi-tier support (Free, Pro, Enterprise, Partner)
/// - Per-endpoint cost multipliers
/// - Graceful degradation under high load
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Rate limiting quota tier definitions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum QuotaTier {
    /// Free tier: 100 req/min, 50K/month
    Free,
    /// Pro tier: 1000 req/min, 1M/month ($99/month)
    Pro,
    /// Enterprise: Unlimited, custom SLA
    Enterprise,
    /// Partner: Unlimited (special program)
    Partner,
}

impl QuotaTier {
    /// Get minute-level rate limit for tier
    pub fn minute_limit(&self) -> u64 {
        match self {
            QuotaTier::Free => 100,
            QuotaTier::Pro => 1000,
            QuotaTier::Enterprise => u64::MAX,
            QuotaTier::Partner => u64::MAX,
        }
    }

    /// Get monthly quota for tier
    pub fn monthly_quota(&self) -> u64 {
        match self {
            QuotaTier::Free => 50_000,
            QuotaTier::Pro => 1_000_000,
            QuotaTier::Enterprise => u64::MAX,
            QuotaTier::Partner => u64::MAX,
        }
    }

    /// Get cost per request ratio
    pub fn cost_multiplier(&self) -> f64 {
        match self {
            QuotaTier::Free => 1.0,
            QuotaTier::Pro => 1.0,
            QuotaTier::Enterprise => 0.5,
            QuotaTier::Partner => 0.0,
        }
    }
}

/// Per-user quota state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQuota {
    /// User or API key identifier
    pub user_id: String,
    /// Current tier
    pub tier: QuotaTier,
    /// Tokens available (for minute limit)
    pub available_tokens: f64,
    /// Last refill time
    pub last_refill: u64,
    /// Requests this month
    pub month_requests: u64,
    /// Month reset epoch
    pub month_start: u64,
    /// Total requests (all time)
    pub total_requests: u64,
    /// Last request time
    pub last_request: u64,
}

impl UserQuota {
    /// Create new quota entry
    pub fn new(user_id: String, tier: QuotaTier) -> Self {
        let now = current_unix_timestamp();
        Self {
            user_id,
            tier,
            available_tokens: tier.minute_limit() as f64,
            last_refill: now,
            month_requests: 0,
            month_start: now,
            total_requests: 0,
            last_request: 0,
        }
    }

    /// Check if user has available quota
    pub fn has_quota(&self) -> bool {
        if matches!(self.tier, QuotaTier::Enterprise | QuotaTier::Partner) {
            return true;
        }
        self.available_tokens >= 1.0 && self.month_requests < self.tier.monthly_quota()
    }

    /// Consume tokens for a request
    pub fn consume(&mut self, endpoint_cost: f64) -> bool {
        if !self.has_quota() {
            return false;
        }

        let cost = endpoint_cost * self.tier.cost_multiplier();

        if self.available_tokens < cost {
            return false;
        }

        self.available_tokens -= cost;
        self.month_requests += 1;
        self.total_requests += 1;
        self.last_request = current_unix_timestamp();

        true
    }
}

/// Endpoint rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    /// Endpoint path
    pub path: String,
    /// Cost multiplier (1.0 = standard, 2.0 = costs 2x)
    pub cost_multiplier: f64,
    /// Whether endpoint requires authentication
    pub requires_auth: bool,
    /// Default bucket tokens for unauthenticated users
    pub default_ip_limit: u64,
}

impl EndpointConfig {
    pub fn new(path: String, cost_multiplier: f64) -> Self {
        Self {
            path,
            cost_multiplier,
            requires_auth: false,
            default_ip_limit: 30,
        }
    }

    pub fn with_auth(mut self, requires_auth: bool) -> Self {
        self.requires_auth = requires_auth;
        self
    }
}

/// Rate limiter state container
pub struct RateLimiter {
    /// User quotas (in-process cache)
    user_quotas: DashMap<String, UserQuota>,
    /// IP-based rate limiting (fallback for unauthenticated)
    ip_quotas: DashMap<String, UserQuota>,
    /// Endpoint configurations
    endpoints: Arc<RwLock<Vec<EndpointConfig>>>,
    /// Metrics
    metrics: Arc<RateLimiterMetrics>,
    /// Redis URL for distributed state
    #[allow(dead_code)]
    redis_url: String,
    /// Redis backend for distributed quota management
    redis_backend: Option<super::rate_limiting_redis::RedisRateLimitBackend>,
}

/// Metrics for rate limiter
#[derive(Debug, Default)]
pub struct RateLimiterMetrics {
    /// Total requests processed
    pub total_requests: parking_lot::Mutex<u64>,
    /// Total rejected requests
    pub total_rejected: parking_lot::Mutex<u64>,
    /// Cache hits
    pub cache_hits: parking_lot::Mutex<u64>,
    /// Cache misses
    pub cache_misses: parking_lot::Mutex<u64>,
}

impl RateLimiter {
    /// Create new rate limiter (without Redis backend)
    pub fn new(redis_url: String) -> Self {
        Self {
            user_quotas: DashMap::new(),
            ip_quotas: DashMap::new(),
            endpoints: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RateLimiterMetrics::default()),
            redis_url,
            redis_backend: None,
        }
    }

    /// Create new rate limiter with Redis backend
    pub fn with_redis(
        redis_url: String,
        backend: super::rate_limiting_redis::RedisRateLimitBackend,
    ) -> Self {
        Self {
            user_quotas: DashMap::new(),
            ip_quotas: DashMap::new(),
            endpoints: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RateLimiterMetrics::default()),
            redis_url,
            redis_backend: Some(backend),
        }
    }

    /// Check if Redis backend is enabled
    pub fn is_redis_enabled(&self) -> bool {
        self.redis_backend.is_some()
    }

    /// Get Redis backend reference
    pub fn redis_backend(&self) -> Option<&super::rate_limiting_redis::RedisRateLimitBackend> {
        self.redis_backend.as_ref()
    }

    /// Register endpoint configuration
    pub fn register_endpoint(&self, config: EndpointConfig) {
        let mut endpoints = self.endpoints.write();
        endpoints.push(config);
    }

    /// Check if request is allowed
    ///
    /// # Arguments
    /// * `user_id` - User or API key identifier
    /// * `tier` - User's quota tier
    /// * `endpoint` - Endpoint being accessed
    /// * `client_ip` - Client IP address (for fallback limiting)
    ///
    /// # Returns
    /// * `Ok(RateLimitResponse)` - Request allowed with remaining quota
    /// * `Err(RateLimitError)` - Request rejected
    pub fn check_limit(
        &self,
        user_id: Option<&str>,
        tier: QuotaTier,
        endpoint: &str,
        client_ip: Option<&str>,
    ) -> Result<RateLimitResponse, RateLimitError> {
        let now = current_unix_timestamp();
        let mut metrics = self.metrics.total_requests.lock();
        *metrics += 1;
        drop(metrics);

        // Get endpoint configuration
        let endpoints = self.endpoints.read();
        let endpoint_config = endpoints
            .iter()
            .find(|e| e.path == endpoint)
            .cloned()
            .unwrap_or_else(|| EndpointConfig::new(endpoint.to_string(), 1.0));
        drop(endpoints);

        // Authenticated user flow
        if let Some(user) = user_id {
            let cache_hit = self.user_quotas.contains_key(user);

            // Create quota entry if not exists (avoid borrow issues by computing first)
            let user_key = user.to_string();
            if !cache_hit {
                let new_quota = UserQuota::new(user_key.clone(), tier);
                self.user_quotas.insert(user_key.clone(), new_quota);
            }

            // Get mutable reference to the quota
            let mut entry = self.user_quotas.get_mut(&user_key).ok_or_else(|| {
                RateLimitError::RateLimitExceeded {
                    retry_after: Duration::from_secs(60),
                }
            })?;

            let quota = entry.value_mut();

            // Refill tokens (token bucket)
            self.refill_tokens(quota, now);

            // Check monthly and minute limits
            if !quota.has_quota() {
                let mut rejected = self.metrics.total_rejected.lock();
                *rejected += 1;
                drop(rejected);

                return Err(RateLimitError::QuotaExceeded {
                    retry_after: calculate_retry_after(quota, tier),
                    monthly_reset: quota.month_start + 30 * 86400,
                });
            }

            // Consume tokens
            if !quota.consume(endpoint_config.cost_multiplier) {
                let mut rejected = self.metrics.total_rejected.lock();
                *rejected += 1;
                drop(rejected);

                return Err(RateLimitError::RateLimitExceeded {
                    retry_after: Duration::from_secs(60),
                });
            }

            if cache_hit {
                let mut hits = self.metrics.cache_hits.lock();
                *hits += 1;
            } else {
                let mut misses = self.metrics.cache_misses.lock();
                *misses += 1;
            }

            Ok(RateLimitResponse {
                allowed: true,
                remaining: quota.available_tokens as u64,
                reset_after: Duration::from_secs(60),
                quota_tier: tier,
                monthly_remaining: quota.tier.monthly_quota() - quota.month_requests,
            })
        } else {
            // Unauthenticated flow - use IP-based limiting
            let ip = client_ip.unwrap_or("unknown");
            let mut entry = self
                .ip_quotas
                .entry(ip.to_string())
                .or_insert_with(|| UserQuota::new(ip.to_string(), QuotaTier::Free));

            let quota = entry.value_mut();

            // More aggressive refill for IP-based (1 minute window)
            self.refill_tokens_rapid(quota, now);

            // Check limits
            if !quota.has_quota() {
                let mut rejected = self.metrics.total_rejected.lock();
                *rejected += 1;

                return Err(RateLimitError::RateLimitExceeded {
                    retry_after: Duration::from_secs(60),
                });
            }

            // Consume tokens
            if !quota.consume(endpoint_config.cost_multiplier) {
                let mut rejected = self.metrics.total_rejected.lock();
                *rejected += 1;

                return Err(RateLimitError::RateLimitExceeded {
                    retry_after: Duration::from_secs(60),
                });
            }

            Ok(RateLimitResponse {
                allowed: true,
                remaining: quota.available_tokens as u64,
                reset_after: Duration::from_secs(60),
                quota_tier: QuotaTier::Free,
                monthly_remaining: 50_000 - quota.month_requests,
            })
        }
    }

    /// Refill tokens using token bucket algorithm
    fn refill_tokens(&self, quota: &mut UserQuota, now: u64) {
        let elapsed = now.saturating_sub(quota.last_refill);

        if elapsed >= 60 {
            // One minute has passed - full refill
            quota.available_tokens = quota.tier.minute_limit() as f64;
            quota.last_refill = now;
        } else if elapsed > 0 {
            // Partial refill: tokens_per_second * elapsed_seconds
            let rate = quota.tier.minute_limit() as f64 / 60.0;
            let tokens_to_add = rate * elapsed as f64;
            quota.available_tokens = (quota.available_tokens + tokens_to_add)
                .min(quota.tier.minute_limit() as f64 * 1.5); // Allow 50% burst
        }
    }

    /// Rapid refill for IP-based rate limiting (stricter)
    fn refill_tokens_rapid(&self, quota: &mut UserQuota, now: u64) {
        let elapsed = now.saturating_sub(quota.last_refill);

        if elapsed >= 60 {
            quota.available_tokens = 30.0; // 30 req/min for IPs
            quota.last_refill = now;
        } else if elapsed > 0 {
            let rate = 30.0 / 60.0;
            let tokens_to_add = rate * elapsed as f64;
            quota.available_tokens = (quota.available_tokens + tokens_to_add).min(45.0);
            // 1.5x burst
        }
    }

    /// Get metrics snapshot
    pub fn metrics_snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_requests: *self.metrics.total_requests.lock(),
            total_rejected: *self.metrics.total_rejected.lock(),
            cache_hits: *self.metrics.cache_hits.lock(),
            cache_misses: *self.metrics.cache_misses.lock(),
            active_users: self.user_quotas.len(),
            active_ips: self.ip_quotas.len(),
        }
    }

    /// Clear stale entries (should run periodically)
    pub fn cleanup_stale_entries(&self, retention_seconds: u64) {
        let now = current_unix_timestamp();
        let threshold = now.saturating_sub(retention_seconds);

        // Remove stale user quotas (inactive for > retention period)
        self.user_quotas
            .retain(|_, quota| quota.last_request > threshold);

        // Remove stale IP quotas
        self.ip_quotas
            .retain(|_, quota| quota.last_request > threshold);
    }

    /// Sync quota to Redis backend (async)
    ///
    /// This should be called after a quota change to ensure
    /// distributed consistency across multiple instances.
    pub async fn sync_quota_to_redis(&self, user_id: &str) -> Result<(), String> {
        if let Some(ref backend) = self.redis_backend {
            if let Some(quota) = self.user_quotas.get(user_id) {
                let entry = super::rate_limiting_redis::RedisQuotaEntry::from_user_quota(
                    user_id.to_string(),
                    &quota,
                );
                // Store with 1 hour TTL
                backend
                    .set_user_quota(user_id, &entry, 3600)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    /// Load quota from Redis backend (async)
    ///
    /// This should be called when a user is not found in local cache
    /// to check if they have existing quota state in Redis.
    pub async fn load_quota_from_redis(&self, user_id: &str) -> Option<UserQuota> {
        if let Some(ref backend) = self.redis_backend {
            if let Ok(Some(entry)) = backend.get_user_quota(user_id).await {
                // Convert RedisQuotaEntry back to UserQuota
                // Note: This is a simplified conversion
                let tier = match entry.tier.as_str() {
                    "Pro" => QuotaTier::Pro,
                    "Enterprise" => QuotaTier::Enterprise,
                    "Partner" => QuotaTier::Partner,
                    _ => QuotaTier::Free,
                };

                let mut quota = UserQuota::new(entry.user_id.clone(), tier);
                quota.available_tokens = entry.available_tokens;
                quota.last_refill = entry.last_refill;
                quota.month_requests = entry.month_requests;
                quota.month_start = entry.month_start;

                // Update local cache
                self.user_quotas.insert(user_id.to_string(), quota.clone());

                return Some(quota);
            }
        }
        None
    }
}

/// Response from rate limiter check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResponse {
    /// Whether request is allowed
    pub allowed: bool,
    /// Remaining requests in current window
    pub remaining: u64,
    /// Time until quota reset
    pub reset_after: Duration,
    /// User's quota tier
    pub quota_tier: QuotaTier,
    /// Remaining quota for current month
    pub monthly_remaining: u64,
}

/// Rate limit error types
#[derive(Debug, Clone)]
pub enum RateLimitError {
    /// Monthly quota exceeded
    QuotaExceeded {
        /// Seconds until monthly reset
        retry_after: u64,
        /// Epoch timestamp of monthly reset
        monthly_reset: u64,
    },
    /// Rate limit per minute exceeded
    RateLimitExceeded {
        /// Duration until retry
        retry_after: Duration,
    },
}

/// Metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub total_requests: u64,
    pub total_rejected: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub active_users: usize,
    pub active_ips: usize,
}

impl MetricsSnapshot {
    pub fn rejection_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.total_rejected as f64 / self.total_requests as f64
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}

/// Get current Unix timestamp
pub fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Calculate seconds until next token refill
fn calculate_retry_after(quota: &UserQuota, tier: QuotaTier) -> u64 {
    // If monthly quota exceeded, return until next month
    if quota.month_requests >= tier.monthly_quota() {
        (quota.month_start + 30 * 86400).saturating_sub(current_unix_timestamp())
    } else {
        // If minute limit exceeded, return 60 seconds
        60
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quota_tier_limits() {
        assert_eq!(QuotaTier::Free.minute_limit(), 100);
        assert_eq!(QuotaTier::Pro.minute_limit(), 1000);
        assert_eq!(QuotaTier::Free.monthly_quota(), 50_000);
    }

    #[test]
    fn test_user_quota_creation() {
        let quota = UserQuota::new("user1".to_string(), QuotaTier::Pro);
        assert_eq!(quota.tier, QuotaTier::Pro);
        assert_eq!(quota.available_tokens, 1000.0);
        assert_eq!(quota.month_requests, 0);
    }

    #[test]
    fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new("redis://localhost".to_string());

        let result = limiter.check_limit(Some("user1"), QuotaTier::Pro, "/identify", None);
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.allowed);
        assert!(response.remaining > 0);
    }

    #[test]
    fn test_token_consumption() {
        let mut quota = UserQuota::new("user1".to_string(), QuotaTier::Free);
        assert!(quota.consume(1.0));
        assert_eq!(quota.available_tokens, 99.0);
        assert_eq!(quota.total_requests, 1);
    }

    #[test]
    fn test_metrics_snapshot() {
        let limiter = RateLimiter::new("redis://localhost".to_string());

        let _ = limiter.check_limit(Some("user1"), QuotaTier::Pro, "/identify", None);

        let metrics = limiter.metrics_snapshot();
        assert!(metrics.total_requests > 0);
        assert!(metrics.active_users > 0);
    }
}
