//! Configuration module for the API Gateway

use serde::{Deserialize, Serialize};
use std::env;

/// Gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// HTTP server host
    pub host: String,

    /// HTTP server port
    pub port: u16,

    /// Number of worker threads
    pub workers: usize,

    /// Redis connection URL
    pub redis_url: String,

    /// Enable Prometheus metrics
    pub enable_metrics: bool,

    /// Request timeout in seconds
    pub request_timeout_secs: u64,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: 4,
            redis_url: "redis://127.0.0.1:6379".to_string(),
            enable_metrics: true,
            request_timeout_secs: 30,
        }
    }
}

impl GatewayConfig {
    /// Load configuration from environment variables
    ///
    /// Environment variables:
    /// - `GATEWAY_HOST`: Server host (default: 0.0.0.0)
    /// - `GATEWAY_PORT`: Server port (default: 8080)
    /// - `GATEWAY_WORKERS`: Worker threads (default: 4)
    /// - `REDIS_URL`: Redis connection URL (default: redis://127.0.0.1:6379)
    /// - `ENABLE_METRICS`: Enable Prometheus metrics (default: true)
    /// - `REQUEST_TIMEOUT_SECS`: Request timeout (default: 30)
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: env::var("GATEWAY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("GATEWAY_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            workers: env::var("GATEWAY_WORKERS")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .unwrap_or(4),
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            enable_metrics: env::var("ENABLE_METRICS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            request_timeout_secs: env::var("REQUEST_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GatewayConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.workers, 4);
        assert_eq!(config.redis_url, "redis://127.0.0.1:6379");
        assert!(config.enable_metrics);
        assert_eq!(config.request_timeout_secs, 30);
    }

    #[test]
    fn test_from_env_defaults() {
        // Clear relevant env vars
        env::remove_var("GATEWAY_HOST");
        env::remove_var("GATEWAY_PORT");

        let config = GatewayConfig::from_env().unwrap();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
    }
}
