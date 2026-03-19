//! Comprehensive metrics collection for fingerprint-rust.

use lazy_static::lazy_static;
use prometheus::{
    opts, register_counter_vec, register_gauge_vec, register_histogram_vec, CounterVec, GaugeVec,
    HistogramVec,
};

lazy_static! {
    pub static ref FINGERPRINT_RECOGNITION_TOTAL: CounterVec = register_counter_vec!(
        opts!(
            "fingerprint_recognition_total",
            "Total fingerprint recognition requests"
        ),
        &["browser", "os", "result"]
    )
    .unwrap();
    pub static ref FINGERPRINT_RECOGNITION_DURATION_MS: HistogramVec = register_histogram_vec!(
        "fingerprint_recognition_duration_ms",
        "Fingerprint recognition duration in milliseconds",
        &["browser_type"],
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 50.0, 100.0, 500.0]
    )
    .unwrap();
    pub static ref FINGERPRINT_SIMILARITY_SCORE: GaugeVec = register_gauge_vec!(
        opts!(
            "fingerprint_similarity_score",
            "Fingerprint similarity scores"
        ),
        &["comparison_type"]
    )
    .unwrap();
    pub static ref CACHE_HIT_RATE: CounterVec = register_counter_vec!(
        opts!("fingerprint_cache_hits_total", "Total cache hits"),
        &["cache_level", "cache_type"]
    )
    .unwrap();
    pub static ref CACHE_MISS_RATE: CounterVec = register_counter_vec!(
        opts!("fingerprint_cache_misses_total", "Total cache misses"),
        &["cache_level", "cache_type"]
    )
    .unwrap();
    pub static ref CACHE_SIZE_BYTES: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_cache_size_bytes", "Cache size in bytes"),
        &["cache_level"]
    )
    .unwrap();
    pub static ref CACHE_EVICTIONS_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_cache_evictions_total", "Total cache evictions"),
        &["cache_level", "reason"]
    )
    .unwrap();
    pub static ref DB_OPERATION_DURATION_MS: HistogramVec = register_histogram_vec!(
        "fingerprint_db_operation_duration_ms",
        "Database operation duration in milliseconds",
        &["operation", "table"],
        vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0]
    )
    .unwrap();
    pub static ref DB_QUERIES_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_db_queries_total", "Total database queries"),
        &["operation", "table", "status"]
    )
    .unwrap();
    pub static ref DB_CONNECTIONS_ACTIVE: GaugeVec = register_gauge_vec!(
        opts!(
            "fingerprint_db_connections_active",
            "Active database connections"
        ),
        &["pool_name"]
    )
    .unwrap();
    pub static ref TLS_CLIENTHELLO_PARSE_MS: HistogramVec = register_histogram_vec!(
        "fingerprint_tls_clienthello_parse_ms",
        "TLS ClientHello parsing duration in milliseconds",
        &["tls_version"],
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
    )
    .unwrap();
    pub static ref TLS_FINGERPRINT_GENERATION_TOTAL: CounterVec = register_counter_vec!(
        opts!(
            "fingerprint_tls_generation_total",
            "TLS fingerprint generations"
        ),
        &["algorithm", "status"]
    )
    .unwrap();
    pub static ref JA_FINGERPRINT_CALC_TOTAL: CounterVec = register_counter_vec!(
        opts!(
            "fingerprint_ja_calculation_total",
            "JA fingerprint calculations"
        ),
        &["ja_type", "status"]
    )
    .unwrap();
    pub static ref HTTP_REQUEST_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_http_request_total", "Total HTTP requests"),
        &["method", "http_version", "status"]
    )
    .unwrap();
    pub static ref HTTP_REQUEST_DURATION_MS: HistogramVec = register_histogram_vec!(
        "fingerprint_http_request_duration_ms",
        "HTTP request duration in milliseconds",
        &["method", "http_version"],
        vec![10.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0]
    )
    .unwrap();
    pub static ref HTTP_POOL_CONNECTIONS: GaugeVec = register_gauge_vec!(
        opts!(
            "fingerprint_http_pool_connections",
            "HTTP connection pool size"
        ),
        &["pool_name", "state"]
    )
    .unwrap();
    pub static ref ANOMALY_DETECTION_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_anomaly_detection_total", "Anomaly detections"),
        &["anomaly_type", "severity"]
    )
    .unwrap();
    pub static ref ANOMALY_SCORE: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_anomaly_score", "Anomaly detection scores"),
        &["detector_type"]
    )
    .unwrap();
    pub static ref ANOMALY_FALSE_POSITIVE_RATE: GaugeVec = register_gauge_vec!(
        opts!(
            "fingerprint_anomaly_false_positive_rate",
            "Anomaly detection false positive rate"
        ),
        &["detector_type"]
    )
    .unwrap();
    pub static ref ML_PREDICTION_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_ml_prediction_total", "ML predictions"),
        &["model", "confidence_level"]
    )
    .unwrap();
    pub static ref ML_INFERENCE_DURATION_MS: HistogramVec = register_histogram_vec!(
        "fingerprint_ml_inference_duration_ms",
        "ML inference duration in milliseconds",
        &["model"],
        vec![5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0]
    )
    .unwrap();
    pub static ref ML_PREDICTION_ACCURACY: GaugeVec = register_gauge_vec!(
        opts!(
            "fingerprint_ml_prediction_accuracy",
            "ML prediction accuracy"
        ),
        &["model", "dataset"]
    )
    .unwrap();
    pub static ref RATE_LIMIT_CHECK_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_rate_limit_check_total", "Rate limit checks"),
        &["tier", "result"]
    )
    .unwrap();
    pub static ref RATE_LIMIT_QUOTA_USAGE: GaugeVec = register_gauge_vec!(
        opts!(
            "fingerprint_rate_limit_quota_usage",
            "Rate limit quota usage percentage"
        ),
        &["tier", "period"]
    )
    .unwrap();
    pub static ref RATE_LIMIT_REJECTIONS_TOTAL: CounterVec = register_counter_vec!(
        opts!(
            "fingerprint_rate_limit_rejections_total",
            "Rate limit rejections"
        ),
        &["tier", "reason"]
    )
    .unwrap();
    pub static ref ERRORS_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_errors_total", "Total errors"),
        &["error_type", "module", "severity"]
    )
    .unwrap();
    pub static ref ERROR_RATE: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_error_rate", "Error rate per module"),
        &["module"]
    )
    .unwrap();
    pub static ref MEMORY_USAGE_MB: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_memory_usage_mb", "Memory usage in megabytes"),
        &["component"]
    )
    .unwrap();
    pub static ref CPU_USAGE_PERCENT: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_cpu_usage_percent", "CPU usage percentage"),
        &["component"]
    )
    .unwrap();
    pub static ref GOROUTINES_ACTIVE: GaugeVec = register_gauge_vec!(
        opts!("fingerprint_goroutines_active", "Active async tasks"),
        &["component"]
    )
    .unwrap();
    pub static ref DNS_RESOLUTION_TOTAL: CounterVec = register_counter_vec!(
        opts!("fingerprint_dns_resolution_total", "DNS resolutions"),
        &["domain", "status"]
    )
    .unwrap();
    pub static ref DNS_RESOLUTION_DURATION_MS: HistogramVec = register_histogram_vec!(
        "fingerprint_dns_resolution_duration_ms",
        "DNS resolution duration in milliseconds",
        &["resolver_type"],
        vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0]
    )
    .unwrap();
    pub static ref DNS_CACHE_HIT_RATE: CounterVec = register_counter_vec!(
        opts!("fingerprint_dns_cache_hits_total", "DNS cache hits"),
        &["cache_level"]
    )
    .unwrap();
}

pub fn record_fingerprint_duration(browser: &str, duration_ms: f64) {
    FINGERPRINT_RECOGNITION_DURATION_MS
        .with_label_values(&[browser])
        .observe(duration_ms);
}

pub fn record_cache_hit(level: &str, cache_type: &str) {
    CACHE_HIT_RATE.with_label_values(&[level, cache_type]).inc();
}

pub fn record_cache_miss(level: &str, cache_type: &str) {
    CACHE_MISS_RATE
        .with_label_values(&[level, cache_type])
        .inc();
}

pub fn record_db_operation(operation: &str, table: &str, duration_ms: f64) {
    DB_OPERATION_DURATION_MS
        .with_label_values(&[operation, table])
        .observe(duration_ms);
    DB_QUERIES_TOTAL
        .with_label_values(&[operation, table, "success"])
        .inc();
}

pub fn record_error(error_type: &str, module: &str, severity: &str) {
    ERRORS_TOTAL
        .with_label_values(&[error_type, module, severity])
        .inc();
}

pub fn record_ml_inference(model: &str, duration_ms: f64) {
    ML_INFERENCE_DURATION_MS
        .with_label_values(&[model])
        .observe(duration_ms);
    ML_PREDICTION_TOTAL
        .with_label_values(&[model, "unknown"])
        .inc();
}
