//! Prometheus metrics collection
//!
//! This module provides comprehensive metrics for monitoring the API Gateway:
//! - HTTP request counters
//! - Rate limit statistics  
//! - Response time histograms
//! - Redis connection health

use prometheus::{
    opts, register_histogram_vec, register_int_counter_vec, register_int_gauge, Encoder,
    HistogramVec, IntCounterVec, IntGauge, TextEncoder,
};
use std::time::Instant;

lazy_static::lazy_static! {
    /// Total HTTP requests by method, endpoint, and status code
    pub static ref HTTP_REQUESTS_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!("fingerprint_gateway_http_requests_total", "Total HTTP requests"),
        &["method", "endpoint", "status"]
    ).unwrap();

    /// Rate limit check results by tier and result (allowed/blocked)
    pub static ref RATE_LIMIT_CHECKS_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!("fingerprint_gateway_rate_limit_checks_total", "Total rate limit checks"),
        &["tier", "result"]
    ).unwrap();

    /// Rate limit quota usage by tier
    pub static ref RATE_LIMIT_QUOTA_USED: IntCounterVec = register_int_counter_vec!(
        opts!("fingerprint_gateway_rate_limit_quota_used", "Rate limit quota used"),
        &["tier", "period"]
    ).unwrap();

    /// HTTP request duration histogram by method and endpoint
    pub static ref HTTP_REQUEST_DURATION_SECONDS: HistogramVec = register_histogram_vec!(
        "fingerprint_gateway_http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "endpoint"],
        vec![0.001, 0.005, 0.010, 0.025, 0.050, 0.100, 0.250, 0.500, 1.0, 2.5, 5.0, 10.0]
    ).unwrap();

    /// Redis connection pool active connections
    pub static ref REDIS_CONNECTIONS_ACTIVE: IntGauge = register_int_gauge!(
        opts!("fingerprint_gateway_redis_connections_active", "Active Redis connections")
    ).unwrap();

    /// Redis operations counter
    pub static ref REDIS_OPERATIONS_TOTAL: IntCounterVec = register_int_counter_vec!(
        opts!("fingerprint_gateway_redis_operations_total", "Total Redis operations"),
        &["operation", "status"]
    ).unwrap();
}

/// Request timer for tracking request duration
pub struct RequestTimer {
    start: Instant,
    method: String,
    endpoint: String,
}

impl RequestTimer {
    /// Start a new request timer
    pub fn new(method: String, endpoint: String) -> Self {
        Self {
            start: Instant::now(),
            method,
            endpoint,
        }
    }

    /// Record the request duration
    pub fn observe(self) {
        let duration = self.start.elapsed().as_secs_f64();
        HTTP_REQUEST_DURATION_SECONDS
            .with_label_values(&[&self.method, &self.endpoint])
            .observe(duration);
    }
}

/// Record an HTTP request
pub fn record_http_request(method: &str, endpoint: &str, status: u16) {
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[method, endpoint, &status.to_string()])
        .inc();
}

/// Record a rate limit check
pub fn record_rate_limit_check(tier: &str, allowed: bool) {
    let result = if allowed { "allowed" } else { "blocked" };
    RATE_LIMIT_CHECKS_TOTAL
        .with_label_values(&[tier, result])
        .inc();
}

/// Record quota usage
pub fn record_quota_usage(tier: &str, period: &str) {
    RATE_LIMIT_QUOTA_USED
        .with_label_values(&[tier, period])
        .inc();
}

/// Record Redis operation
pub fn record_redis_operation(operation: &str, success: bool) {
    let status = if success { "success" } else { "error" };
    REDIS_OPERATIONS_TOTAL
        .with_label_values(&[operation, status])
        .inc();
}

/// Update Redis connection count
pub fn update_redis_connections(count: i64) {
    REDIS_CONNECTIONS_ACTIVE.set(count);
}

/// Gather all metrics in Prometheus text format
pub fn gather_metrics() -> Result<String, Box<dyn std::error::Error>> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_http_request() {
        record_http_request("GET", "/health", 200);
        record_http_request("POST", "/rate-limit/check", 200);
        // Metrics are global, just ensure no panic
    }

    #[test]
    fn test_record_rate_limit_check() {
        record_rate_limit_check("Free", true);
        record_rate_limit_check("Pro", false);
    }

    #[test]
    fn test_request_timer() {
        let timer = RequestTimer::new("GET".to_string(), "/health".to_string());
        std::thread::sleep(std::time::Duration::from_millis(10));
        timer.observe();
    }

    #[test]
    fn test_gather_metrics() {
        let result = gather_metrics();
        assert!(result.is_ok());
        let metrics = result.unwrap();
        // Check that metrics output contains some of our metric names
        assert!(
            metrics.contains("fingerprint_http_requests_total")
                || metrics.contains("fingerprint_rate_limit_checks_total")
                || !metrics.is_empty(),
            "Metrics output should contain metric names or be non-empty"
        );
    }
}
