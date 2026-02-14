//! Data models for API Gateway

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Quota tier enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum QuotaTier {
    /// Free tier: 100 req/min, 50K/month
    Free,
    /// Pro tier: 1000 req/min, 1M/month
    Pro,
    /// Enterprise tier: unlimited
    Enterprise,
    /// Partner tier: unlimited
    Partner,
}

impl QuotaTier {
    /// Get the per-minute rate limit for this tier
    pub fn minute_limit(&self) -> Option<u32> {
        match self {
            Self::Free => Some(100),
            Self::Pro => Some(1000),
            Self::Enterprise | Self::Partner => None,
        }
    }

    /// Get the monthly quota for this tier
    pub fn monthly_quota(&self) -> Option<u64> {
        match self {
            Self::Free => Some(50_000),
            Self::Pro => Some(1_000_000),
            Self::Enterprise | Self::Partner => None,
        }
    }

    /// Check if this is an unlimited tier
    pub fn is_unlimited(&self) -> bool {
        matches!(self, Self::Enterprise | Self::Partner)
    }
}

/// Rate limit check request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitRequest {
    /// API key
    pub api_key: String,

    /// Endpoint being accessed
    pub endpoint: String,

    /// Client IP address (optional)
    pub client_ip: Option<String>,
}

/// Rate limit check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitResponse {
    /// Whether the request is allowed
    pub allowed: bool,

    /// Quota tier
    pub quota_tier: QuotaTier,

    /// Remaining quota (if applicable)
    pub remaining: Option<u32>,

    /// Rate limit (if applicable)
    pub limit: Option<u32>,

    /// When the rate limit resets
    pub reset_at: Option<DateTime<Utc>>,

    /// Error message if not allowed
    pub error: Option<String>,
}

/// Rate limit status for an API key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatus {
    /// API key
    pub api_key: String,

    /// Quota tier
    pub quota_tier: QuotaTier,

    /// Requests in current minute
    pub current_minute_requests: u32,

    /// Requests this month
    pub current_month_requests: u64,

    /// Per-minute limit
    pub minute_limit: Option<u32>,

    /// Monthly quota
    pub monthly_quota: Option<u64>,

    /// When the minute window resets
    pub minute_reset_at: DateTime<Utc>,

    /// When the month resets
    pub month_reset_at: DateTime<Utc>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,

    /// Service version
    pub version: String,

    /// Redis connection status
    pub redis_connected: bool,

    /// Server timestamp
    pub timestamp: DateTime<Utc>,
}

/// Metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    /// Total requests processed
    pub total_requests: u64,

    /// Allowed requests
    pub allowed_requests: u64,

    /// Blocked requests
    pub blocked_requests: u64,

    /// Average response time (ms)
    pub avg_response_time_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quota_tier_limits() {
        assert_eq!(QuotaTier::Free.minute_limit(), Some(100));
        assert_eq!(QuotaTier::Pro.minute_limit(), Some(1000));
        assert_eq!(QuotaTier::Enterprise.minute_limit(), None);
        assert_eq!(QuotaTier::Partner.minute_limit(), None);
    }

    #[test]
    fn test_quota_tier_monthly() {
        assert_eq!(QuotaTier::Free.monthly_quota(), Some(50_000));
        assert_eq!(QuotaTier::Pro.monthly_quota(), Some(1_000_000));
        assert_eq!(QuotaTier::Enterprise.monthly_quota(), None);
    }

    #[test]
    fn test_quota_tier_unlimited() {
        assert!(!QuotaTier::Free.is_unlimited());
        assert!(!QuotaTier::Pro.is_unlimited());
        assert!(QuotaTier::Enterprise.is_unlimited());
        assert!(QuotaTier::Partner.is_unlimited());
    }

    #[test]
    fn test_quota_tier_serialization() {
        let tier = QuotaTier::Pro;
        let json = serde_json::to_string(&tier).unwrap();
        assert_eq!(json, r#""Pro""#);

        let deserialized: QuotaTier = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, QuotaTier::Pro);
    }
}
