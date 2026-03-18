use fingerprint_core::rate_limiting::{MetricsSnapshot, QuotaTier};
use std::fmt::Write as FmtWrite;

#[derive(Debug, Clone)]
pub struct PrometheusMetrics {
    pub total_requests: u64,
    pub total_rejected: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub active_users: usize,
    pub active_ips: usize,
}

impl PrometheusMetrics {
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

    pub fn to_prometheus_format(&self) -> String {
        use std::fmt::Write;
        let mut output = String::new();

        let _ = writeln!(
            output,
            "# HELP rate_limiter_requests_total Total number of requests processed"
        );
        let _ = writeln!(output, "# TYPE rate_limiter_requests_total counter");
        let _ = writeln!(
            output,
            "rate_limiter_requests_total {}",
            self.total_requests
        );

        let _ = writeln!(
            output,
            "# HELP rate_limiter_requests_rejected_total Total number of rejected requests"
        );
        let _ = writeln!(
            output,
            "# TYPE rate_limiter_requests_rejected_total counter"
        );
        let _ = writeln!(
            output,
            "rate_limiter_requests_rejected_total {}",
            self.total_rejected
        );

        let _ = writeln!(
            output,
            "# HELP rate_limiter_cache_hits_total Total number of cache hits"
        );
        let _ = writeln!(output, "# TYPE rate_limiter_cache_hits_total counter");
        let _ = writeln!(output, "rate_limiter_cache_hits_total {}", self.cache_hits);

        let _ = writeln!(
            output,
            "# HELP rate_limiter_cache_misses_total Total number of cache misses"
        );
        let _ = writeln!(output, "# TYPE rate_limiter_cache_misses_total counter");
        let _ = writeln!(
            output,
            "rate_limiter_cache_misses_total {}",
            self.cache_misses
        );

        let _ = writeln!(
            output,
            "# HELP rate_limiter_active_users_gauge Current number of active users"
        );
        let _ = writeln!(output, "# TYPE rate_limiter_active_users_gauge gauge");
        let _ = writeln!(
            output,
            "rate_limiter_active_users_gauge {}",
            self.active_users
        );

        let _ = writeln!(
            output,
            "# HELP rate_limiter_active_ips_gauge Current number of active IP addresses"
        );
        let _ = writeln!(output, "# TYPE rate_limiter_active_ips_gauge gauge");
        let _ = writeln!(output, "rate_limiter_active_ips_gauge {}", self.active_ips);

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

        let _ = writeln!(
            output,
            "# HELP rate_limiter_rejection_rate_percent Percentage of requests rejected"
        );
        let _ = writeln!(output, "# TYPE rate_limiter_rejection_rate_percent gauge");
        let _ = writeln!(
            output,
            "rate_limiter_rejection_rate_percent {:.2}",
            rejection_rate
        );

        let _ = writeln!(
            output,
            "# HELP rate_limiter_cache_hit_ratio_percent Percentage of cache hits"
        );
        let _ = writeln!(output, "# TYPE rate_limiter_cache_hit_ratio_percent gauge");
        let _ = writeln!(
            output,
            "rate_limiter_cache_hit_ratio_percent {:.2}",
            cache_hit_ratio
        );

        output
    }

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

#[derive(Debug, Clone)]
pub struct TierMetrics {
    pub tier: QuotaTier,
    pub user_count: usize,
    pub total_requests: u64,
    pub rejected_requests: u64,
}

impl TierMetrics {
    pub fn to_prometheus_format(&self) -> String {
        let mut output = String::new();
        let tier_name = format!("{:?}", self.tier).to_lowercase();

        let _ = writeln!(
            &mut output,
            "rate_limiter_tier_users{{tier=\"{}\"}} {}",
            tier_name, self.user_count
        );
        let _ = writeln!(
            &mut output,
            "rate_limiter_tier_requests_total{{tier=\"{}\"}} {}",
            tier_name, self.total_requests
        );
        let _ = writeln!(
            &mut output,
            "rate_limiter_tier_rejected_total{{tier=\"{}\"}} {}",
            tier_name, self.rejected_requests
        );

        output
    }
}

pub struct MetricsHandler;

impl MetricsHandler {
    pub fn prometheus_response(metrics: &PrometheusMetrics) -> String {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
            metrics.to_prometheus_format().len(),
            metrics.to_prometheus_format()
        )
    }

    pub fn json_response(metrics: &PrometheusMetrics) -> String {
        let json = metrics.to_json();
        let body = json.to_string();
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        )
    }

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
