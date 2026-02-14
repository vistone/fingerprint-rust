//! Rate limiting module using Token Bucket algorithm with Redis backend

use crate::error::{GatewayError, Result};
use crate::models::{QuotaTier, RateLimitResponse, RateLimitStatus};
use chrono::{Datelike, Duration, Timelike, Utc};
use redis::AsyncCommands;
use tracing::{debug, error, warn};

pub use crate::models::QuotaTier as QuotaTierEnum;

/// Rate limiter with Redis backend
pub struct RateLimiter {
    redis_pool: bb8::Pool<bb8_redis::RedisConnectionManager>,
}

impl RateLimiter {
    /// Create a new rate limiter with Redis backend
    ///
    /// # Arguments
    ///
    /// * `redis_url` - Redis connection URL (e.g., "redis://127.0.0.1:6379")
    pub async fn new(redis_url: String) -> Result<Self> {
        let manager = bb8_redis::RedisConnectionManager::new(redis_url.as_str())
            .map_err(|e| GatewayError::ConfigError(format!("Invalid Redis URL: {}", e)))?;

        let pool = bb8::Pool::builder()
            .max_size(15)
            .build(manager)
            .await
            .map_err(|e| {
                GatewayError::ConfigError(format!("Failed to create Redis pool: {}", e))
            })?;

        // Test connection
        {
            let mut conn = pool.get().await.map_err(|e| {
                GatewayError::RedisError(redis::RedisError::from((
                    redis::ErrorKind::IoError,
                    "Connection pool error",
                    e.to_string(),
                )))
            })?;

            redis::cmd("PING")
                .query_async::<_, String>(&mut *conn)
                .await
                .map_err(|e| {
                    GatewayError::RedisError(redis::RedisError::from((
                        redis::ErrorKind::IoError,
                        "Redis PING failed",
                        e.to_string(),
                    )))
                })?;
        }

        debug!("Rate limiter initialized with Redis backend");

        Ok(Self { redis_pool: pool })
    }

    /// Check if a request is allowed under rate limits
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key making the request
    /// * `quota_tier` - The quota tier for this API key
    ///
    /// # Returns
    ///
    /// Returns a `RateLimitResponse` indicating if the request is allowed
    pub async fn check_rate_limit(
        &self,
        api_key: &str,
        quota_tier: QuotaTier,
    ) -> Result<RateLimitResponse> {
        // Unlimited tiers always allow
        if quota_tier.is_unlimited() {
            return Ok(RateLimitResponse {
                allowed: true,
                quota_tier,
                remaining: None,
                limit: None,
                reset_at: None,
                error: None,
            });
        }

        let mut conn = self.redis_pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection: {}", e);
            crate::metrics::record_redis_operation("get_connection", false);
            GatewayError::RedisError(redis::RedisError::from((
                redis::ErrorKind::IoError,
                "Connection pool error",
                e.to_string(),
            )))
        })?;

        crate::metrics::record_redis_operation("get_connection", true);

        let now = Utc::now();

        // Check per-minute rate limit
        let minute_key = format!("ratelimit:{}:minute:{}", api_key, now.format("%Y%m%d%H%M"));
        let current_count: u32 = conn.get(&minute_key).await.unwrap_or(0);

        let minute_limit = quota_tier.minute_limit().unwrap_or(u32::MAX);

        if current_count >= minute_limit {
            let reset_at =
                now.with_second(0).unwrap().with_nanosecond(0).unwrap() + Duration::minutes(1);

            warn!(
                "Rate limit exceeded for API key {} (tier: {:?}): {}/{}",
                api_key, quota_tier, current_count, minute_limit
            );

            return Ok(RateLimitResponse {
                allowed: false,
                quota_tier,
                remaining: Some(0),
                limit: Some(minute_limit),
                reset_at: Some(reset_at),
                error: Some(format!(
                    "Rate limit exceeded: {}/{} requests per minute",
                    current_count, minute_limit
                )),
            });
        }

        // Check monthly quota
        let month_key = format!("ratelimit:{}:month:{}", api_key, now.format("%Y%m"));
        let month_count: u64 = conn.get(&month_key).await.unwrap_or(0);

        if let Some(monthly_quota) = quota_tier.monthly_quota() {
            if month_count >= monthly_quota {
                let next_month = if now.month() == 12 {
                    now.with_year(now.year() + 1)
                        .unwrap()
                        .with_month(1)
                        .unwrap()
                } else {
                    now.with_month(now.month() + 1).unwrap()
                };

                warn!(
                    "Monthly quota exceeded for API key {} (tier: {:?}): {}/{}",
                    api_key, quota_tier, month_count, monthly_quota
                );

                return Ok(RateLimitResponse {
                    allowed: false,
                    quota_tier,
                    remaining: Some(0),
                    limit: Some(minute_limit),
                    reset_at: Some(next_month),
                    error: Some(format!(
                        "Monthly quota exceeded: {}/{} requests",
                        month_count, monthly_quota
                    )),
                });
            }
        }

        // Increment counters
        let _: () = redis::pipe()
            .atomic()
            .incr(&minute_key, 1)
            .ignore()
            .expire(&minute_key, 120) // 2 minutes TTL
            .ignore()
            .incr(&month_key, 1)
            .ignore()
            .expire(&month_key, 32 * 24 * 3600) // ~1 month TTL
            .ignore()
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                error!("Failed to increment rate limit counters: {}", e);
                GatewayError::RedisError(e)
            })?;

        let remaining = minute_limit.saturating_sub(current_count + 1);
        let reset_at =
            now.with_second(0).unwrap().with_nanosecond(0).unwrap() + Duration::minutes(1);

        debug!(
            "Rate limit check passed for API key {} (tier: {:?}): {}/{}",
            api_key,
            quota_tier,
            current_count + 1,
            minute_limit
        );

        Ok(RateLimitResponse {
            allowed: true,
            quota_tier,
            remaining: Some(remaining),
            limit: Some(minute_limit),
            reset_at: Some(reset_at),
            error: None,
        })
    }

    /// Get the current rate limit status for an API key
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key to check
    /// * `quota_tier` - The quota tier for this API key
    pub async fn get_status(
        &self,
        api_key: &str,
        quota_tier: QuotaTier,
    ) -> Result<RateLimitStatus> {
        let mut conn = self.redis_pool.get().await.map_err(|e| {
            GatewayError::RedisError(redis::RedisError::from((
                redis::ErrorKind::IoError,
                "Connection pool error",
                e.to_string(),
            )))
        })?;

        let now = Utc::now();

        let minute_key = format!("ratelimit:{}:minute:{}", api_key, now.format("%Y%m%d%H%M"));
        let month_key = format!("ratelimit:{}:month:{}", api_key, now.format("%Y%m"));

        let current_minute_requests: u32 = conn.get(&minute_key).await.unwrap_or(0);
        let current_month_requests: u64 = conn.get(&month_key).await.unwrap_or(0);

        let minute_reset_at =
            now.with_second(0).unwrap().with_nanosecond(0).unwrap() + Duration::minutes(1);

        let month_reset_at = if now.month() == 12 {
            now.with_year(now.year() + 1)
                .unwrap()
                .with_month(1)
                .unwrap()
                .with_day(1)
                .unwrap()
        } else {
            now.with_month(now.month() + 1)
                .unwrap()
                .with_day(1)
                .unwrap()
        };

        Ok(RateLimitStatus {
            api_key: api_key.to_string(),
            quota_tier,
            current_minute_requests,
            current_month_requests,
            minute_limit: quota_tier.minute_limit(),
            monthly_quota: quota_tier.monthly_quota(),
            minute_reset_at,
            month_reset_at,
        })
    }

    /// Reset rate limits for an API key (admin function)
    pub async fn reset_limits(&self, api_key: &str) -> Result<()> {
        let mut conn = self.redis_pool.get().await.map_err(|e| {
            GatewayError::RedisError(redis::RedisError::from((
                redis::ErrorKind::IoError,
                "Connection pool error",
                e.to_string(),
            )))
        })?;

        let pattern = format!("ratelimit:{}:*", api_key);
        let mut cursor: u64 = 0;

        loop {
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(200)
                .query_async(&mut *conn)
                .await
                .map_err(GatewayError::RedisError)?;

            if !keys.is_empty() {
                redis::cmd("DEL")
                    .arg(&keys)
                    .query_async::<_, ()>(&mut *conn)
                    .await
                    .map_err(GatewayError::RedisError)?;
            }

            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }

        debug!("Reset rate limits for API key: {}", api_key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quota_tier_unlimited() {
        // Mock test - actual tests would need Redis
        let tier = QuotaTier::Enterprise;
        assert!(tier.is_unlimited());
        assert_eq!(tier.minute_limit(), None);
    }

    #[test]
    fn test_quota_tier_limits() {
        assert_eq!(QuotaTier::Free.minute_limit(), Some(100));
        assert_eq!(QuotaTier::Pro.minute_limit(), Some(1000));
        assert_eq!(QuotaTier::Free.monthly_quota(), Some(50_000));
        assert_eq!(QuotaTier::Pro.monthly_quota(), Some(1_000_000));
    }
}
