/// Prometheus Metrics for Rate Limiting
///
/// Exports rate limiting metrics in Prometheus format for monitoring and alerting.
/// Integrates with Phase 9.2 monitoring stack.
use super::rate_limiting::{MetricsSnapshot, QuotaTier};
use std::fmt::Write as FmtWrite;

/// Prometheus metric collection
#[derive(Debug, Clone)]
pub struct PrometheusMetrics {
    /// Total requests processed
    pub total_requests: u64,
    /// Total rejected requests
    pub total_rejected: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
    /// Active users tracked
    pub active_users: usize,
    /// Active IPs tracked
    pub active_ips: usize,
}

impl PrometheusMetrics {
    /// Create from rate limiter metrics snapshot
    pub fn from_snapshot(snapshot: MetricsSnapshot) -> Self {
        Self {
            total_requests: snapshot.total_requests,
            total_rejected: snapshot.total_rejected,
            cache_hits: snapshot.cache_hits,
            cache_misses: snapshot.cache_misses,
            active_users: snapshot.active_users,
            active_ips: snapshot.active_ips,
        }
    }

    /// Export metrics in Prometheus text format
    pub fn to_prometheus_format(&self) -> String {
        let mut output = String::new();

        // HELP and TYPE comments
        writeln!(
            &mut output,
            "# HELP rate_limiter_requests_total Total number of requests processed"
        )
        .unwrap();
        writeln!(&mut output, "# TYPE rate_limiter_requests_total counter").unwrap();
        writeln!(
            &mut output,
            "rate_limiter_requests_total {}",
            self.total_requests
        )
        .unwrap();

        writeln!(
            &mut output,
            "# HELP rate_limiter_requests_rejected_total Total number of rejected requests"
        )
        .unwrap();
        writeln!(
            &mut output,
            "# TYPE rate_limiter_requests_rejected_total counter"
        )
        .unwrap();
        writeln!(
            &mut output,
            "rate_limiter_requests_rejected_total {}",
            self.total_rejected
        )
        .unwrap();

        writeln!(
            &mut output,
            "# HELP rate_limiter_cache_hits_total Total number of cache hits"
        )
        .unwrap();
        writeln!(&mut output, "# TYPE rate_limiter_cache_hits_total counter").unwrap();
        writeln!(
            &mut output,
            "rate_limiter_cache_hits_total {}",
            self.cache_hits
        )
        .unwrap();

        writeln!(
            &mut output,
            "# HELP rate_limiter_cache_misses_total Total number of cache misses"
        )
        .unwrap();
        writeln!(
            &mut output,
            "# TYPE rate_limiter_cache_misses_total counter"
        )
        .unwrap();
        writeln!(
            &mut output,
            "rate_limiter_cache_misses_total {}",
            self.cache_misses
        )
        .unwrap();

        writeln!(
            &mut output,
            "# HELP rate_limiter_active_users_gauge Current number of active users"
        )
        .unwrap();
        writeln!(&mut output, "# TYPE rate_limiter_active_users_gauge gauge").unwrap();
        writeln!(
            &mut output,
            "rate_limiter_active_users_gauge {}",
            self.active_users
        )
        .unwrap();

        writeln!(
            &mut output,
            "# HELP rate_limiter_active_ips_gauge Current number of active IP addresses"
        )
        .unwrap();
        writeln!(&mut output, "# TYPE rate_limiter_active_ips_gauge gauge").unwrap();
        writeln!(
            &mut output,
            "rate_limiter_active_ips_gauge {}",
            self.active_ips
        )
        .unwrap();

        // Calculated metrics
        let rejection_rate = if self.total_requests == 0 {
            0.0
        } else {
            (self.total_rejected as f64 / self.total_requests as f64) * 100.0
        };

        let cache_hit_ratio = if self.cache_hits + self.cache_misses == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64) * 100.0
        };

        writeln!(
            &mut output,
            "# HELP rate_limiter_rejection_rate_percent Percentage of requests rejected"
        )
        .unwrap();
        writeln!(
            &mut output,
            "# TYPE rate_limiter_rejection_rate_percent gauge"
        )
        .unwrap();
        writeln!(
            &mut output,
            "rate_limiter_rejection_rate_percent {:.2}",
            rejection_rate
        )
        .unwrap();

        writeln!(
            &mut output,
            "# HELP rate_limiter_cache_hit_ratio_percent Percentage of cache hits"
        )
        .unwrap();
        writeln!(
            &mut output,
            "# TYPE rate_limiter_cache_hit_ratio_percent gauge"
        )
        .unwrap();
        writeln!(
            &mut output,
            "rate_limiter_cache_hit_ratio_percent {:.2}",
            cache_hit_ratio
        )
        .unwrap();

        output
    }

    /// Export as JSON for alternative monitoring systems
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "total_requests": self.total_requests,
            "total_rejected": self.total_rejected,
            "cache_hits": self.cache_hits,
            "cache_misses": self.cache_misses,
            "active_users": self.active_users,
            "active_ips": self.active_ips,
            "rejection_rate_percent": if self.total_requests == 0 {
                0.0
            } else {
                (self.total_rejected as f64 / self.total_requests as f64) * 100.0
            },
            "cache_hit_ratio_percent": if self.cache_hits + self.cache_misses == 0 {
                0.0
            } else {
                (self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64) * 100.0
            }
        })
    }
}

/// Per-tier quota metrics
#[derive(Debug, Clone)]
pub struct TierMetrics {
    /// Tier name
    pub tier: QuotaTier,
    /// Users in this tier
    pub user_count: usize,
    /// Total requests from this tier
    pub total_requests: u64,
    /// Rejected requests from this tier
    pub rejected_requests: u64,
}

impl TierMetrics {
    /// Export as Prometheus metrics
    pub fn to_prometheus_format(&self) -> String {
        let mut output = String::new();
        let tier_name = format!("{:?}", self.tier).to_lowercase();

        writeln!(
            &mut output,
            "rate_limiter_tier_users{{tier=\"{}\"}} {}",
            tier_name, self.user_count
        )
        .unwrap();
        writeln!(
            &mut output,
            "rate_limiter_tier_requests_total{{tier=\"{}\"}} {}",
            tier_name, self.total_requests
        )
        .unwrap();
        writeln!(
            &mut output,
            "rate_limiter_tier_rejected_total{{tier=\"{}\"}} {}",
            tier_name, self.rejected_requests
        )
        .unwrap();

        output
    }
}

/// HTTP handler for Prometheus metrics endpoint
pub struct MetricsHandler;

impl MetricsHandler {
    /// Generate HTTP 200 response with metrics
    pub fn prometheus_response(metrics: &PrometheusMetrics) -> String {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
            metrics.to_prometheus_format().len(),
            metrics.to_prometheus_format()
        )
    }

    /// Generate HTTP 200 response with JSON metrics
    pub fn json_response(metrics: &PrometheusMetrics) -> String {
        let json = metrics.to_json();
        let body = json.to_string();
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        )
    }

    /// Generate HTTP 503 response for unhealthy status
    pub fn unavailable_response() -> String {
        let body = "Rate limiter service unavailable";
        format!(
            "HTTP/1.1 503 Service Unavailable\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prometheus_format_output() {
        let metrics = PrometheusMetrics {
            total_requests: 1000,
            total_rejected: 50,
            cache_hits: 800,
            cache_misses: 200,
            active_users: 42,
            active_ips: 10,
        };

        let output = metrics.to_prometheus_format();
        assert!(output.contains("rate_limiter_requests_total 1000"));
        assert!(output.contains("rate_limiter_requests_rejected_total 50"));
        assert!(output.contains("rate_limiter_cache_hits_total 800"));
        assert!(output.contains("rate_limiter_active_users_gauge 42"));
    }

    #[test]
    fn test_rejection_rate_calculation() {
        let metrics = PrometheusMetrics {
            total_requests: 1000,
            total_rejected: 50,
            cache_hits: 800,
            cache_misses: 200,
            active_users: 42,
            active_ips: 10,
        };

        let output = metrics.to_prometheus_format();
        // 50/1000 = 5.0%
        assert!(output.contains("rate_limiter_rejection_rate_percent 5.00"));
    }

    #[test]
    fn test_cache_hit_ratio_calculation() {
        let metrics = PrometheusMetrics {
            total_requests: 1000,
            total_rejected: 50,
            cache_hits: 800,
            cache_misses: 200,
            active_users: 42,
            active_ips: 10,
        };

        let output = metrics.to_prometheus_format();
        // 800 / (800+200) = 80.0%
        assert!(output.contains("rate_limiter_cache_hit_ratio_percent 80.00"));
    }

    #[test]
    fn test_json_export() {
        let metrics = PrometheusMetrics {
            total_requests: 1000,
            total_rejected: 50,
            cache_hits: 800,
            cache_misses: 200,
            active_users: 42,
            active_ips: 10,
        };

        let json = metrics.to_json();
        assert_eq!(json["total_requests"], 1000);
        assert_eq!(json["total_rejected"], 50);
        assert_eq!(json["cache_hits"], 800);
    }

    #[test]
    fn test_http_response_generation() {
        let metrics = PrometheusMetrics {
            total_requests: 1000,
            total_rejected: 50,
            cache_hits: 800,
            cache_misses: 200,
            active_users: 42,
            active_ips: 10,
        };

        let response = MetricsHandler::prometheus_response(&metrics);
        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("Content-Type: text/plain"));
        assert!(response.contains("rate_limiter_requests_total 1000"));
    }
}
