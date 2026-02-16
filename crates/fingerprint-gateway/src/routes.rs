//! API routes for the Gateway

use actix_web::{web, HttpRequest, HttpResponse, Responder};
use chrono::Utc;
use tracing::info;

use crate::{
    auth::ApiKeyValidator,
    error::GatewayError,
    models::{HealthResponse, QuotaTier, RateLimitRequest},
    rate_limit::RateLimiter,
};

/// Health check endpoint
///
/// GET /api/v1/health
pub async fn health(rate_limiter: web::Data<RateLimiter>) -> Result<impl Responder, GatewayError> {
    use crate::metrics;

    let _timer = metrics::RequestTimer::new("GET".to_string(), "/health".to_string());

    // Test Redis connection
    let redis_connected = rate_limiter
        .get_status("health_check", QuotaTier::Free)
        .await
        .is_ok();

    let response = HealthResponse {
        status: if redis_connected {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
        redis_connected,
        timestamp: Utc::now(),
    };

    metrics::record_http_request("GET", "/health", 200);
    Ok(HttpResponse::Ok().json(response))
}

/// Rate limit check endpoint
///
/// POST /api/v1/rate-limit/check
pub async fn check_rate_limit(
    rate_limiter: web::Data<RateLimiter>,
    validator: web::Data<ApiKeyValidator>,
    req: web::Json<RateLimitRequest>,
) -> Result<impl Responder, GatewayError> {
    use crate::metrics;

    let _timer = metrics::RequestTimer::new("POST".to_string(), "/rate-limit/check".to_string());

    info!(
        "Rate limit check for API key: {}, endpoint: {}",
        req.api_key, req.endpoint
    );

    // Validate API key and get tier
    let key_info = validator.validate(&req.api_key)?;
    let quota_tier = key_info.tier;

    let result = rate_limiter
        .check_rate_limit(&req.api_key, quota_tier)
        .await?;

    // Record metrics
    let tier_str = format!("{:?}", quota_tier);
    metrics::record_rate_limit_check(&tier_str, result.allowed);

    if result.allowed {
        metrics::record_quota_usage(&tier_str, "minute");
        info!("Rate limit check passed for API key: {}", req.api_key);
        metrics::record_http_request("POST", "/rate-limit/check", 200);
        Ok(HttpResponse::Ok().json(result))
    } else {
        info!("Rate limit check failed for API key: {}", req.api_key);
        metrics::record_http_request("POST", "/rate-limit/check", 429);
        Ok(HttpResponse::TooManyRequests().json(result))
    }
}

/// Get rate limit status for an API key
///
/// GET /api/v1/rate-limit/status?api_key={key}
pub async fn get_status(
    rate_limiter: web::Data<RateLimiter>,
    validator: web::Data<ApiKeyValidator>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<impl Responder, GatewayError> {
    let api_key = query
        .get("api_key")
        .ok_or_else(|| GatewayError::InvalidRequest("Missing api_key parameter".to_string()))?;

    // Validate API key and get tier
    let key_info = validator.validate(api_key)?;
    let quota_tier = key_info.tier;

    let status = rate_limiter.get_status(api_key, quota_tier).await?;

    Ok(HttpResponse::Ok().json(status))
}

/// Reset rate limits for an API key (admin endpoint)
///
/// POST /api/v1/rate-limit/reset
pub async fn reset_rate_limit(
    rate_limiter: web::Data<RateLimiter>,
    validator: web::Data<ApiKeyValidator>,
    request: HttpRequest,
    req: web::Json<std::collections::HashMap<String, String>>,
) -> Result<impl Responder, GatewayError> {
    let api_key = req
        .get("api_key")
        .ok_or_else(|| GatewayError::InvalidRequest("Missing api_key field".to_string()))?;

    let admin_key = request
        .headers()
        .get("X-Admin-Key")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| GatewayError::InvalidApiKey("Missing admin key".to_string()))?;

    let admin_info = validator.validate(admin_key)?;
    if !matches!(admin_info.tier, QuotaTier::Enterprise | QuotaTier::Partner) {
        return Err(GatewayError::InvalidApiKey(
            "Admin key not authorized".to_string(),
        ));
    }

    rate_limiter.reset_limits(api_key).await?;

    info!("Rate limits reset for API key: {}", api_key);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("Rate limits reset for API key: {}", api_key)
    })))
}

/// Prometheus metrics endpoint
///
/// GET /metrics
pub async fn metrics() -> Result<impl Responder, GatewayError> {
    use crate::metrics;

    let metrics_data = metrics::gather_metrics()
        .map_err(|e| GatewayError::InternalError(format!("Failed to gather metrics: {}", e)))?;

    Ok(HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(metrics_data))
}

/// Configure all routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .route("/health", web::get().to(health))
            .route("/rate-limit/check", web::post().to(check_rate_limit))
            .route("/rate-limit/status", web::get().to(get_status))
            .route("/rate-limit/reset", web::post().to(reset_rate_limit)),
    )
    .route("/metrics", web::get().to(metrics));
}

/// Determine quota tier based on API key
///
/// **Note**: This function is kept for backward compatibility and testing purposes only.
/// In production, use the validator with database/configuration service integration.
///
/// # Implementation Note
/// For production use, replace this with actual database queries or configuration service calls.
/// Current implementation is a simple prefix-based matcher for testing.
#[allow(dead_code)]
fn determine_quota_tier(api_key: &str) -> QuotaTier {
    if api_key.starts_with("sk_test_") {
        QuotaTier::Free
    } else if api_key.starts_with("sk_live_") {
        QuotaTier::Pro
    } else if api_key.starts_with("sk_enterprise_") {
        QuotaTier::Enterprise
    } else if api_key.starts_with("sk_partner_") {
        QuotaTier::Partner
    } else {
        QuotaTier::Free
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_quota_tier() {
        assert_eq!(determine_quota_tier("sk_test_12345"), QuotaTier::Free);
        assert_eq!(determine_quota_tier("sk_live_12345"), QuotaTier::Pro);
        assert_eq!(
            determine_quota_tier("sk_enterprise_12345"),
            QuotaTier::Enterprise
        );
        assert_eq!(determine_quota_tier("sk_partner_12345"), QuotaTier::Partner);
    }
}
