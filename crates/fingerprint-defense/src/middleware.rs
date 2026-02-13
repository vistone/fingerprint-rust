/// API Gateway Middleware Integration
///
/// Provides middleware components for integrating fingerprint defense
/// capabilities into API gateways and web applications.
///
/// # Example Usage
///
/// ```rust
/// use fingerprint_defense::middleware::{ConsistencyCheckMiddleware, ConsistencyCheckConfig};
/// use fingerprint_defense::ConsistencyAnalyzer;
///
/// // Create middleware with default config
/// let analyzer = ConsistencyAnalyzer::new();
/// let middleware = ConsistencyCheckMiddleware::new(analyzer);
///
/// // Check incoming request
/// // let result = middleware.check_request(&request).await;
/// ```
use crate::passive::consistency::{ConsistencyAnalyzer, ConsistencyViolation};
use fingerprint_core::ja4::ConsistencyReport;
use fingerprint_core::system::{NetworkFlow, TrafficDirection};
use std::net::IpAddr;
use std::time::Duration;

/// Configuration for consistency check middleware
#[derive(Debug, Clone)]
pub struct ConsistencyCheckConfig {
    /// Whether to enable consistency checks
    pub enabled: bool,
    /// Minimum risk score to trigger an alert
    pub alert_threshold: u8,
    /// Whether to block requests with high risk scores
    pub block_high_risk: bool,
    /// Risk score threshold for blocking
    pub block_threshold: u8,
    /// Maximum age of a fingerprint to be considered valid
    pub max_fingerprint_age: Duration,
    /// Whether to check JA4+ consistency
    pub check_ja4_consistency: bool,
    /// Whether to check timestamp consistency
    pub check_timestamp_consistency: bool,
    /// Whether to check TCP/HTTP OS consistency
    pub check_os_consistency: bool,
}

impl Default for ConsistencyCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            alert_threshold: 50,
            block_high_risk: false,
            block_threshold: 80,
            max_fingerprint_age: Duration::from_secs(300),
            check_ja4_consistency: true,
            check_timestamp_consistency: true,
            check_os_consistency: true,
        }
    }
}

impl ConsistencyCheckConfig {
    /// Create new config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable checks
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set alert threshold
    pub fn with_alert_threshold(mut self, threshold: u8) -> Self {
        self.alert_threshold = threshold;
        self
    }

    /// Enable blocking for high risk
    pub fn with_blocking(mut self, enabled: bool, threshold: u8) -> Self {
        self.block_high_risk = enabled;
        self.block_threshold = threshold;
        self
    }
}

/// Result of consistency check
#[derive(Debug, Clone)]
pub struct ConsistencyCheckResult {
    /// Whether the request passed all checks
    pub passed: bool,
    /// Risk score (0-100)
    pub risk_score: u8,
    /// Whether the request should be blocked
    pub should_block: bool,
    /// Violations found (if any)
    pub violations: Vec<ConsistencyViolation>,
    /// Detailed report
    pub report: Option<ConsistencyReport>,
    /// Reason for blocking (if blocked)
    pub block_reason: Option<String>,
}

impl ConsistencyCheckResult {
    /// Create a passing result
    pub fn pass() -> Self {
        Self {
            passed: true,
            risk_score: 0,
            should_block: false,
            violations: Vec::new(),
            report: None,
            block_reason: None,
        }
    }

    /// Create a failing result
    pub fn fail(risk_score: u8, violations: Vec<ConsistencyViolation>) -> Self {
        Self {
            passed: false,
            risk_score,
            should_block: false,
            violations,
            report: None,
            block_reason: None,
        }
    }

    /// Create a blocked result
    pub fn blocked(reason: String, risk_score: u8) -> Self {
        Self {
            passed: false,
            risk_score,
            should_block: true,
            violations: Vec::new(),
            report: None,
            block_reason: Some(reason),
        }
    }
}

/// Consistency check middleware
pub struct ConsistencyCheckMiddleware {
    analyzer: ConsistencyAnalyzer,
    config: ConsistencyCheckConfig,
}

impl ConsistencyCheckMiddleware {
    /// Create new middleware with analyzer and config
    pub fn new(analyzer: ConsistencyAnalyzer, config: ConsistencyCheckConfig) -> Self {
        Self { analyzer, config }
    }

    /// Create new middleware with default config
    pub fn with_default_config(analyzer: ConsistencyAnalyzer) -> Self {
        Self::new(analyzer, ConsistencyCheckConfig::default())
    }

    /// Check a network flow for consistency
    ///
    /// This is the main entry point for API gateway integration.
    /// It analyzes the flow and returns a result indicating whether
    /// the request should be allowed, flagged, or blocked.
    pub fn check_flow(&self, flow: &NetworkFlow) -> ConsistencyCheckResult {
        if !self.config.enabled {
            return ConsistencyCheckResult::pass();
        }

        // Run consistency analysis
        let report = self.analyzer.analyze_flow(flow);

        // Calculate risk score based on discrepancies
        let risk_score = self.calculate_risk_score(&report);

        // Check if should block
        if self.config.block_high_risk && risk_score >= self.config.block_threshold {
            return ConsistencyCheckResult::blocked(
                format!(
                    "High risk score: {} (threshold: {})",
                    risk_score, self.config.block_threshold
                ),
                risk_score,
            );
        }

        // Check if has violations
        let violations = self.extract_violations(&report);

        if !violations.is_empty() && risk_score >= self.config.alert_threshold {
            return ConsistencyCheckResult {
                passed: false,
                risk_score,
                should_block: false,
                violations,
                report: Some(report),
                block_reason: None,
            };
        }

        ConsistencyCheckResult {
            passed: true,
            risk_score,
            should_block: false,
            violations,
            report: Some(report),
            block_reason: None,
        }
    }

    /// Create a network flow from HTTP request data
    ///
    /// Helper method to create a NetworkFlow from typical HTTP request info
    pub fn create_flow_from_request(
        &self,
        client_ip: IpAddr,
        user_agent: Option<&str>,
        tls_ja4: Option<&str>,
        http_ja4h: Option<&str>,
        tcp_fingerprint: Option<&str>,
    ) -> NetworkFlow {
        use chrono::Utc;
        use fingerprint_core::system::{ProtocolType, SystemContext};

        let mut context = SystemContext::new(
            client_ip,
            IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
            ProtocolType::Http,
        );
        context.direction = TrafficDirection::Inbound;
        context.timestamp = Utc::now();

        let flow = NetworkFlow::new(context);

        // Add TLS fingerprint if available
        // Note: In production, TLS fingerprints would be added by the TLS analyzer
        let _ = tls_ja4;

        // Add HTTP fingerprint if available
        // Note: In production, HTTP fingerprints would be added by the HTTP analyzer
        let _ = http_ja4h;
        let _ = user_agent;

        // Add TCP fingerprint if available
        // Note: In production, TCP fingerprints would be added by the TCP analyzer
        let _ = tcp_fingerprint;

        flow
    }

    /// Calculate risk score from consistency report
    fn calculate_risk_score(&self, report: &ConsistencyReport) -> u8 {
        let base_score = report.score;

        // Adjust based on number of discrepancies
        let discrepancy_penalty = (report.discrepancies.len() as u8).saturating_mul(10);

        // Cap at 100
        base_score.saturating_add(discrepancy_penalty).min(100)
    }

    /// Extract violations from consistency report
    fn extract_violations(&self, report: &ConsistencyReport) -> Vec<ConsistencyViolation> {
        // Convert discrepancies to violations
        report
            .discrepancies
            .iter()
            .map(|d| ConsistencyViolation::TcpStackMismatch {
                tcp_detected: d.clone(),
                ua_claimed: "unknown".to_string(),
            })
            .collect()
    }

    /// Get current configuration
    pub fn config(&self) -> &ConsistencyCheckConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: ConsistencyCheckConfig) {
        self.config = config;
    }
}

/// Rate limiting middleware integration
///
/// Provides integration between rate limiting and fingerprint defense
pub mod rate_limiting_integration {
    use super::*;
    use fingerprint_core::rate_limiting::{QuotaTier, RateLimitResponse, RateLimiter};

    /// Combined check result (consistency + rate limiting)
    #[derive(Debug, Clone)]
    pub struct SecurityCheckResult {
        /// Consistency check result
        pub consistency: ConsistencyCheckResult,
        /// Rate limiting result
        pub rate_limit: Option<RateLimitResponse>,
        /// Whether request is allowed
        pub allowed: bool,
        /// Combined reason if blocked
        pub reason: Option<String>,
    }

    /// Security middleware combining multiple checks
    pub struct SecurityMiddleware {
        consistency_middleware: ConsistencyCheckMiddleware,
        rate_limiter: Option<RateLimiter>,
    }

    impl SecurityMiddleware {
        /// Create new security middleware
        pub fn new(
            consistency_middleware: ConsistencyCheckMiddleware,
            rate_limiter: Option<RateLimiter>,
        ) -> Self {
            Self {
                consistency_middleware,
                rate_limiter,
            }
        }

        /// Perform full security check
        pub fn check_request(
            &self,
            flow: &NetworkFlow,
            user_id: Option<&str>,
            tier: QuotaTier,
        ) -> SecurityCheckResult {
            // Check consistency first
            let consistency_result = self.consistency_middleware.check_flow(flow);

            // If blocked by consistency check, don't proceed
            if consistency_result.should_block {
                return SecurityCheckResult {
                    consistency: consistency_result.clone(),
                    rate_limit: None,
                    allowed: false,
                    reason: consistency_result.block_reason.clone(),
                };
            }

            // Check rate limiting if enabled
            let rate_limit_result = if let Some(ref limiter) = self.rate_limiter {
                match limiter.check_limit(user_id, tier, "/api/request", None) {
                    Ok(response) => Some(response),
                    Err(e) => {
                        return SecurityCheckResult {
                            consistency: consistency_result,
                            rate_limit: None,
                            allowed: false,
                            reason: Some(format!("Rate limit exceeded: {:?}", e)),
                        };
                    }
                }
            } else {
                None
            };

            SecurityCheckResult {
                consistency: consistency_result,
                rate_limit: rate_limit_result,
                allowed: true,
                reason: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistency_config_default() {
        let config = ConsistencyCheckConfig::default();
        assert!(config.enabled);
        assert_eq!(config.alert_threshold, 50);
        assert_eq!(config.block_threshold, 80);
        assert!(!config.block_high_risk);
    }

    #[test]
    fn test_consistency_config_builder() {
        let config = ConsistencyCheckConfig::new()
            .with_enabled(false)
            .with_alert_threshold(30)
            .with_blocking(true, 70);

        assert!(!config.enabled);
        assert_eq!(config.alert_threshold, 30);
        assert!(config.block_high_risk);
        assert_eq!(config.block_threshold, 70);
    }

    #[test]
    fn test_check_result_pass() {
        let result = ConsistencyCheckResult::pass();
        assert!(result.passed);
        assert!(!result.should_block);
        assert_eq!(result.risk_score, 0);
    }

    #[test]
    fn test_check_result_blocked() {
        let result = ConsistencyCheckResult::blocked("High risk".to_string(), 85);
        assert!(!result.passed);
        assert!(result.should_block);
        assert_eq!(result.risk_score, 85);
        assert_eq!(result.block_reason, Some("High risk".to_string()));
    }
}
