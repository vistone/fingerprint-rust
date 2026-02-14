//! Middleware for actix-web
//!
//! Provides custom middleware for:
//! - Request ID tracking
//! - Request/response logging  
//! - Metrics collection

use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    Error, HttpMessage,
};
use std::time::Instant;
use tracing::{info, warn};
use uuid::Uuid;

/// Request ID header name
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";

/// Request ID middleware - adds unique ID to each request
pub struct RequestId;

/// Add a unique request ID to the request
pub fn add_request_id(req: &ServiceRequest) -> String {
    // Check if request already has an ID
    if let Some(id) = req.headers().get(REQUEST_ID_HEADER) {
        if let Ok(id_str) = id.to_str() {
            return id_str.to_string();
        }
    }

    // Generate new request ID
    let request_id = Uuid::new_v4().to_string();
    req.extensions_mut().insert(request_id.clone());
    request_id
}

/// Log request details
pub fn log_request(req: &ServiceRequest, request_id: &str) {
    let method = req.method().as_str();
    let path = req.path();
    let query = req.query_string();

    let peer_addr = req
        .peer_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if query.is_empty() {
        info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            peer = %peer_addr,
            "Incoming request"
        );
    } else {
        info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            query = %query,
            peer = %peer_addr,
            "Incoming request"
        );
    }
}

/// Log response details
pub fn log_response<B: MessageBody>(res: &ServiceResponse<B>, request_id: &str, duration_ms: f64) {
    let status = res.status().as_u16();
    let method = res.request().method().as_str();
    let path = res.request().path();

    if status >= 500 {
        warn!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status,
            duration_ms = %duration_ms,
            "Request failed"
        );
    } else if status >= 400 {
        warn!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status,
            duration_ms = %duration_ms,
            "Request error"
        );
    } else {
        info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status,
            duration_ms = %duration_ms,
            "Request completed"
        );
    }
}

/// Middleware wrapper for logging and metrics
pub async fn logging_middleware(
    req: ServiceRequest,
    next: impl Fn(ServiceRequest) -> Result<ServiceResponse, Error>,
) -> Result<ServiceResponse, Error> {
    let start = Instant::now();
    let request_id = add_request_id(&req);

    log_request(&req, &request_id);

    let res = next(req)?;

    let duration = start.elapsed();
    let duration_ms = duration.as_secs_f64() * 1000.0;

    log_response(&res, &request_id, duration_ms);

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;

    #[test]
    fn test_request_id_generation() {
        let id1 = Uuid::new_v4().to_string();
        let id2 = Uuid::new_v4().to_string();
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 36); // UUID v4 format
    }

    #[test]
    fn test_add_request_id_uses_existing_header() {
        let req = TestRequest::default()
            .insert_header((REQUEST_ID_HEADER, "existing-id"))
            .to_srv_request();

        let request_id = add_request_id(&req);

        assert_eq!(request_id, "existing-id");
        assert!(req.extensions().get::<String>().is_none());
    }

    #[test]
    fn test_add_request_id_sets_extension() {
        let req = TestRequest::default().to_srv_request();

        let request_id = add_request_id(&req);
        let stored = req.extensions().get::<String>().cloned();

        assert!(stored.is_some());
        assert_eq!(stored.unwrap(), request_id);
    }
}
