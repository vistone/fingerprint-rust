//! Comprehensive metrics collection for fingerprint-rust
//!
//! This module provides Prometheus metrics for monitoring all critical components:
//! - Fingerprint recognition performance
//! - Cache hit rates
//! - Database operations
//! - Rate limiting statistics
//! - Error tracking

use lazy_static::lazy_static;
use prometheus::{
    opts, register_counter_vec, register_gauge_vec, register_histogram_vec, CounterVec, GaugeVec,
    HistogramVec,
};

lazy_static! {
    // ========== Fingerprint Recognition Metrics ==========

    /// Total fingerprint recognition requests by browser type
    pub static ref FINGERPRINT_RECOGNITION_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_recognition_total", "Total fingerprint recognition requests"),
        &["browser", "os", "result"]
    ).unwrap();

    /// Fingerprint recognition duration histogram in milliseconds
    pub static ref FINGERPRINT_RECOGNITION_DURATION_MS: HistogramVec =
        register_histogram_vec!(
            "fingerprint_recognition_duration_ms",
            "Fingerprint recognition duration in milliseconds",
            &["browser_type"],
            vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 50.0, 100.0, 500.0]
        ).unwrap();

    /// Fingerprint similarity scores distribution
    pub static ref FINGERPRINT_SIMILARITY_SCORE: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_similarity_score", "Fingerprint similarity scores"),
        &["comparison_type"]
    ).unwrap();

    // ========== Cache Metrics ==========

    /// Cache hit rate by cache type (L1/L2/L3)
    pub static ref CACHE_HIT_RATE: CounterVec = register_counter_vec!(
        opts!("fingerprint_cache_hits_total", "Total cache hits"),
        &["cache_level", "cache_type"]
    ).unwrap();

    /// Cache miss rate by cache type
    pub static ref CACHE_MISS_RATE: CounterVec = register_counter_vec!(
        opts!("fingerprint_cache_misses_total", "Total cache misses"),
        &["cache_level", "cache_type"]
    ).unwrap();

    /// Current cache size in bytes
    pub static ref CACHE_SIZE_BYTES: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_cache_size_bytes", "Cache size in bytes"),
        &["cache_level"]
    ).unwrap();

    /// Cache eviction count
    pub static ref CACHE_EVICTIONS_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_cache_evictions_total", "Total cache evictions"),
        &["cache_level", "reason"]
    ).unwrap();

    // ========== Database Metrics ==========

    /// Database operation duration by operation type
    pub static ref DB_OPERATION_DURATION_MS: HistogramVec =
        register_histogram_vec!(
            "fingerprint_db_operation_duration_ms",
            "Database operation duration in milliseconds",
            &["operation", "table"],
            vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0]
        ).unwrap();

    /// Database query count by operation type
    pub static ref DB_QUERIES_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_db_queries_total", "Total database queries"),
        &["operation", "table", "status"]
    ).unwrap();

    /// Database connection pool statistics
    pub static ref DB_CONNECTIONS_ACTIVE: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_db_connections_active", "Active database connections"),
        &["pool_name"]
    ).unwrap();

    // ========== TLS Fingerprinting Metrics ==========

    /// TLS ClientHello parsing duration
    pub static ref TLS_CLIENTHELLO_PARSE_MS: HistogramVec =
        register_histogram_vec!(
            "fingerprint_tls_clienthello_parse_ms",
            "TLS ClientHello parsing duration in milliseconds",
            &["tls_version"],
            vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
        ).unwrap();

    /// TLS fingerprint generation count
    pub static ref TLS_FINGERPRINT_GENERATION_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_tls_generation_total", "TLS fingerprint generations"),
        &["algorithm", "status"]
    ).unwrap();

    /// JA3/JA4 hash calculation success rate
    pub static ref JA_FINGERPRINT_CALC_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_ja_calculation_total", "JA fingerprint calculations"),
        &["ja_type", "status"]
    ).unwrap();

    // ========== HTTP Client Metrics ==========

    /// HTTP request count by method and protocol
    pub static ref HTTP_REQUEST_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_http_request_total", "Total HTTP requests"),
        &["method", "http_version", "status"]
    ).unwrap();

    /// HTTP request duration histogram
    pub static ref HTTP_REQUEST_DURATION_MS: HistogramVec =
        register_histogram_vec!(
            "fingerprint_http_request_duration_ms",
            "HTTP request duration in milliseconds",
            &["method", "http_version"],
            vec![10.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0]
        ).unwrap();

    /// HTTP connection pool statistics
    pub static ref HTTP_POOL_CONNECTIONS: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_http_pool_connections", "HTTP connection pool size"),
        &["pool_name", "state"]
    ).unwrap();

    // ========== Anomaly Detection Metrics ==========

    /// Anomaly detection count by type
    pub static ref ANOMALY_DETECTION_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_anomaly_detection_total", "Anomaly detections"),
        &["anomaly_type", "severity"]
    ).unwrap();

    /// Anomaly detection score distribution
    pub static ref ANOMALY_SCORE: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_anomaly_score", "Anomaly detection scores"),
        &["detector_type"]
    ).unwrap();

    /// False positive rate for anomaly detection
    pub static ref ANOMALY_FALSE_POSITIVE_RATE: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_anomaly_false_positive_rate", "Anomaly detection false positive rate"),
        &["detector_type"]
    ).unwrap();

    // ========== Machine Learning Metrics ==========

    /// ML model prediction count by model
    pub static ref ML_PREDICTION_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_ml_prediction_total", "ML predictions"),
        &["model", "confidence_level"]
    ).unwrap();

    /// ML model inference duration
    pub static ref ML_INFERENCE_DURATION_MS: HistogramVec =
        register_histogram_vec!(
            "fingerprint_ml_inference_duration_ms",
            "ML inference duration in milliseconds",
            &["model"],
            vec![5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0]
        ).unwrap();

    /// ML model prediction accuracy
    pub static ref ML_PREDICTION_ACCURACY: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_ml_prediction_accuracy", "ML prediction accuracy"),
        &["model", "dataset"]
    ).unwrap();

    // ========== Rate Limiting Metrics ==========

    /// Rate limit checks by tier
    pub static ref RATE_LIMIT_CHECK_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_rate_limit_check_total", "Rate limit checks"),
        &["tier", "result"]
    ).unwrap();

    /// Current rate limit quota usage
    pub static ref RATE_LIMIT_QUOTA_USAGE: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_rate_limit_quota_usage", "Rate limit quota usage percentage"),
        &["tier", "period"]
    ).unwrap();

    /// Rate limit rejections
    pub static ref RATE_LIMIT_REJECTIONS_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_rate_limit_rejections_total", "Rate limit rejections"),
        &["tier", "reason"]
    ).unwrap();

    // ========== Error Metrics ==========

    /// Error count by error type and module
    pub static ref ERRORS_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_errors_total", "Total errors"),
        &["error_type", "module", "severity"]
    ).unwrap();

    /// Error rate gauge
    pub static ref ERROR_RATE: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_error_rate", "Error rate per module"),
        &["module"]
    ).unwrap();

    // ========== System Metrics ==========

    /// Memory usage in megabytes
    pub static ref MEMORY_USAGE_MB: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_memory_usage_mb", "Memory usage in megabytes"),
        &["component"]
    ).unwrap();

    /// CPU usage percentage
    pub static ref CPU_USAGE_PERCENT: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_cpu_usage_percent", "CPU usage percentage"),
        &["component"]
    ).unwrap();

    /// Active goroutines/tasks count
    pub static ref GOROUTINES_ACTIVE: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_goroutines_active", "Active async tasks"),
        &["component"]
    ).unwrap();

    // ========== DNS Resolution Metrics ==========

    /// DNS resolution count by domain
    pub static ref DNS_RESOLUTION_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_dns_resolution_total", "DNS resolutions"),
        &["domain", "status"]
    ).unwrap();

    /// DNS resolution duration
    pub static ref DNS_RESOLUTION_DURATION_MS: HistogramVec =
        register_histogram_vec!(
            "fingerprint_dns_resolution_duration_ms",
            "DNS resolution duration in milliseconds",
            &["resolver_type"],
            vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0]
        ).unwrap();

    /// DNS cache hit rate
    pub static ref DNS_CACHE_HIT_RATE: CounterVec = register_counter_vec!(
        opts!("fingerprint_dns_cache_hits_total", "DNS cache hits"),
        &["cache_level"]
    ).unwrap();
}

/// Helper function to record fingerprint recognition timing
pub fn record_fingerprint_duration(browser: &str, duration_ms: f64) {
    FINGERPRINT_RECOGNITION_DURATION_MS
        .with_label_values(&[browser])
        .observe(duration_ms);
}

/// Helper function to record cache operations
pub fn record_cache_hit(level: &str, cache_type: &str) {
    CACHE_HIT_RATE.with_label_values(&[level, cache_type]).inc();
}

pub fn record_cache_miss(level: &str, cache_type: &str) {
    CACHE_MISS_RATE
        .with_label_values(&[level, cache_type])
        .inc();
}

/// Helper function to record database operations
pub fn record_db_operation(operation: &str, table: &str, duration_ms: f64) {
    DB_OPERATION_DURATION_MS
        .with_label_values(&[operation, table])
        .observe(duration_ms);
}

/// Helper function to record errors
pub fn record_error(error_type: &str, module: &str, severity: &str) {
    ERRORS_TOTAL
        .with_label_values(&[error_type, module, severity])
        .inc();
}

/// Helper function to record ML inference
pub fn record_ml_inference(model: &str, duration_ms: f64) {
    ML_INFERENCE_DURATION_MS
        .with_label_values(&[model])
        .observe(duration_ms);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        // Verify all metrics are properly initialized
        CACHE_HIT_RATE.with_label_values(&["L1", "profile"]).inc();

        FINGERPRINT_RECOGNITION_TOTAL
            .with_label_values(&["chrome", "linux", "success"])
            .inc();
        // Metrics initialized successfully
    }
}
