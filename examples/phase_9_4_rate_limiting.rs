/// Phase 9.4 Integration Example
///
/// This example demonstrates how to use the rate limiting service with FastAPI/Fingerprint API.
/// It shows the complete flow from rate limit check to response generation.

use fingerprint_core::{
    EndpointConfig, MetricsSnapshot, PrometheusMetrics, QuotaTier, RateLimiter, RateLimitResponse,
};

/// Example: Rate Limiting for Fingerprint API
pub struct FingerprintApiGateway {
    rate_limiter: RateLimiter,
}

impl FingerprintApiGateway {
    /// Initialize gateway with rate limiter
    pub fn new(redis_url: String) -> Self {
        let rate_limiter = RateLimiter::new(redis_url);

        // Register endpoint configurations
        rate_limiter.register_endpoint(EndpointConfig::new("/identify".to_string(), 1.0));
        rate_limiter.register_endpoint(EndpointConfig::new("/compare".to_string(), 2.0));
        rate_limiter.register_endpoint(EndpointConfig::new("/batch".to_string(), 1.0));

        Self { rate_limiter }
    }

    /// Process incoming request through rate limiter
    pub fn handle_request(
        &self,
        user_id: Option<&str>,
        user_tier: QuotaTier,
        endpoint: &str,
        client_ip: Option<&str>,
    ) -> Result<RateLimitResponse, String> {
        // Check rate limits
        self.rate_limiter
            .check_limit(user_id, user_tier, endpoint, client_ip)
            .map_err(|err| match err {
                fingerprint_core::RateLimitError::QuotaExceeded {
                    retry_after,
                    monthly_reset,
                } => {
                    format!(
                        "Monthly quota exceeded. Reset at timestamp: {}. Retry after: {}s",
                        monthly_reset, retry_after
                    )
                }
                fingerprint_core::RateLimitError::RateLimitExceeded { retry_after } => {
                    format!(
                        "Rate limit exceeded. Retry after: {}s",
                        retry_after.as_secs()
                    )
                }
            })
    }

    /// Generate response headers for rate limit info
    pub fn generate_rate_limit_headers(
        response: &RateLimitResponse,
    ) -> Vec<(String, String)> {
        let reset_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            + response.reset_after.as_secs();

        vec![
            ("X-RateLimit-Remaining".to_string(), response.remaining.to_string()),
            ("X-RateLimit-Reset".to_string(), reset_timestamp.to_string()),
            (
                "X-Quota-Tier".to_string(),
                format!("{:?}", response.quota_tier).to_lowercase(),
            ),
            (
                "X-Quota-Monthly-Remaining".to_string(),
                response.monthly_remaining.to_string(),
            ),
        ]
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> MetricsSnapshot {
        self.rate_limiter.metrics_snapshot()
    }

    /// Export metrics in Prometheus format
    pub fn metrics_prometheus(&self) -> String {
        let snapshot = self.rate_limiter.metrics_snapshot();
        let metrics = PrometheusMetrics::from_snapshot(snapshot);
        metrics.to_prometheus_format()
    }

    /// Export metrics as JSON
    pub fn metrics_json(&self) -> String {
        let snapshot = self.rate_limiter.metrics_snapshot();
        let metrics = PrometheusMetrics::from_snapshot(snapshot);
        metrics.to_json().to_string()
    }
}

/// Example FastAPI middleware usage (pseudo code)
/// 
/// ```python
/// from fastapi import Request, Response
/// from fastapi.responses import JSONResponse
/// import httpx
///
/// class RateLimitMiddleware:
///     async def dispatch(self, request: Request, call_next):
///         user_id = request.headers.get("X-API-Key")
///         tier = get_user_tier(user_id)  # From database
///         endpoint = request.url.path
///         client_ip = request.client.host
///         
///         # Call Rust rate limiter via FFI or HTTP
///         try:
///             result = check_rate_limit(user_id, tier, endpoint, client_ip)
///         except QuotaExceededError as e:
///             return JSONResponse(
///                 status_code=429,
///                 content={"error": "quota_exceeded", "reset": e.monthly_reset},
///                 headers={"Retry-After": str(e.retry_after)}
///             )
///         except RateLimitExceededError as e:
///             return JSONResponse(
///                 status_code=429,
///                 content={"error": "rate_limit_exceeded"},
///                 headers={"Retry-After": str(int(e.retry_after.total_seconds()))}
///             )
///         
///         # Request allowed
///         response = await call_next(request)
///         
///         # Add rate limit headers
///         for key, value in result.headers.items():
///             response.headers[key] = value
///         
///         return response
/// ```

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_creation() {
        let _gateway = FingerprintApiGateway::new("redis://localhost:6379".to_string());
        // Gateway created successfully
    }

    #[test]
    fn test_rate_limit_check() {
        let gateway = FingerprintApiGateway::new("redis://localhost:6379".to_string());
        
        // Check limit for authenticated user (Pro tier)
        let result = gateway.handle_request(
            Some("user123"),
            QuotaTier::Pro,
            "/identify",
            Some("192.168.1.100"),
        );

        match result {
            Ok(response) => {
                // Verify response structure
                assert!(response.allowed);
                assert!(response.remaining > 0);
                println!("Request allowed. Remaining: {}", response.remaining);
            }
            Err(e) => {
                println!("Rate limit error: {}", e);
            }
        }
    }

    #[test]
    fn test_response_headers() {
        let response = RateLimitResponse {
            allowed: true,
            remaining: 987,
            reset_after: std::time::Duration::from_secs(60),
            quota_tier: QuotaTier::Pro,
            monthly_remaining: 999_000,
        };

        let headers = FingerprintApiGateway::generate_rate_limit_headers(&response);

        // Verify headers
        assert_eq!(
            headers
                .iter()
                .find(|(k, _)| k == "X-RateLimit-Remaining")
                .map(|(_, v)| v.as_str()),
            Some("987")
        );

        assert_eq!(
            headers
                .iter()
                .find(|(k, _)| k == "X-Quota-Tier")
                .map(|(_, v)| v.as_str()),
            Some("pro")
        );
    }
}

/// Integration with Kong API Gateway
///
/// Kong configuration (via admin API) to use rate limiter:
///
/// ```bash
/// # 1. Create service pointing to fingerprint-api
/// curl -X POST http://kong-admin:8001/services \
///   -d name=fingerprint-api \
///   -d url=http://fingerprint-api:3000
///
/// # 2. Create route to service
/// curl -X POST http://kong-admin:8001/services/fingerprint-api/routes \
///   -d name=identify-route \
///   -d paths[]=/identify
///
/// # 3. Add rate-limiting plugin
/// curl -X POST http://kong-admin:8001/services/fingerprint-api/plugins \
///   -d name=rate-limiting \
///   -d config.minute=1000 \
///   -d config.policy=redis \
///   -d config.redis_host=redis-cluster.caching \
///   -d config.redis_port=6379
///
/// # 4. Add key-auth plugin
/// curl -X POST http://kong-admin:8001/services/fingerprint-api/plugins \
///   -d name=key-auth \
///   -d config.key_names[]="X-API-Key" \
///   -d config.header_names[]="X-API-Key"
/// ```

/// Load testing rate limiter
///
/// ```bash
/// # Using Apache Bench
/// ab -n 10000 -c 100 -H "X-API-Key: test-key" \
///    http://kong-gateway/identify
///
/// # Using k6
/// cat > load_test.js << 'EOF'
/// import http from 'k6/http';
/// import { check } from 'k6';
///
/// export let options = {
///   vus: 100,
///   duration: '5m',
/// };
///
/// export default function () {
///   let res = http.get(
///     'http://kong-gateway/identify',
///     { headers: { 'X-API-Key': 'test-key' }}
///   );
///   check(res, {
///     'status is 200': (r) => r.status === 200,
///     'status is 429': (r) => r.status === 429,
///   });
/// }
/// EOF
///
/// k6 run load_test.js
/// ```

fn main() {
    println!("Phase 9.4 Integration Example");
    println!("============================\n");

    // Initialize gateway
    let gateway = FingerprintApiGateway::new("redis://localhost:6379".to_string());

    println!("✓ Gateway initialized with rate limiting");

    // Simulate some requests
    println!("\nSimulating requests:");
    println!("-----------");

    // Request 1: Authenticated user, Pro tier
    match gateway.handle_request(Some("user123"), QuotaTier::Pro, "/identify", Some("192.168.1.1")) {
        Ok(response) => {
            println!(
                "✓ User 'user123' ({:?} tier): {} remaining requests",
                response.quota_tier, response.remaining
            );
        }
        Err(e) => println!("✗ Error: {}", e),
    }

    // Request 2: Unauthenticated user (IP-based)
    match gateway.handle_request(None, QuotaTier::Free, "/compare", Some("192.168.1.2")) {
        Ok(response) => {
            println!(
                "✓ IP 192.168.1.2 (unauthenticated): {} remaining requests",
                response.remaining
            );
        }
        Err(e) => println!("✗ Error: {}", e),
    }

    // Display metrics
    println!("\nMetrics:");
    println!("--------");
    let metrics = gateway.get_metrics();
    println!("Total requests: {}", metrics.total_requests);
    println!("Rejected requests: {}", metrics.total_rejected);
    println!("Cache hit ratio: {:.1}%", metrics.cache_hit_rate() * 100.0);
    println!("Active users: {}", metrics.active_users);
    println!("Active IPs: {}", metrics.active_ips);

    // Show Prometheus format
    println!("\nPrometheus Metrics Sample:");
    println!("--------------------------");
    let prometheus_output = gateway.metrics_prometheus();
    println!("{}", prometheus_output.lines().take(10).collect::<Vec<_>>().join("\n"));
    println!("...\n");
}
