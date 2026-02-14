//! Error types for the API Gateway

use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};

/// Result type alias for Gateway operations
pub type Result<T> = std::result::Result<T, GatewayError>;

/// Gateway error types
#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    /// Invalid API key
    #[error("Invalid API key: {0}")]
    InvalidApiKey(String),

    /// Quota exceeded
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    /// Redis connection error
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Internal server error
    #[error("Internal server error: {0}")]
    InternalError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl ResponseError for GatewayError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::RateLimitExceeded(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::InvalidApiKey(_) => StatusCode::UNAUTHORIZED,
            Self::QuotaExceeded(_) => StatusCode::PAYMENT_REQUIRED,
            Self::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            Self::RedisError(_) | Self::ConfigError(_) | Self::InternalError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let error_message = self.to_string();

        HttpResponse::build(status).json(serde_json::json!({
            "error": {
                "message": error_message,
                "code": status.as_u16(),
                "type": self.error_type(),
            }
        }))
    }
}

impl GatewayError {
    fn error_type(&self) -> &str {
        match self {
            Self::RateLimitExceeded(_) => "rate_limit_exceeded",
            Self::InvalidApiKey(_) => "invalid_api_key",
            Self::QuotaExceeded(_) => "quota_exceeded",
            Self::RedisError(_) => "redis_error",
            Self::ConfigError(_) => "config_error",
            Self::InvalidRequest(_) => "invalid_request",
            Self::InternalError(_) => "internal_error",
            Self::IoError(_) => "io_error",
            Self::Other(_) => "unknown_error",
        }
    }
}

impl From<anyhow::Error> for GatewayError {
    fn from(err: anyhow::Error) -> Self {
        Self::Other(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            GatewayError::RateLimitExceeded("test".to_string()).status_code(),
            StatusCode::TOO_MANY_REQUESTS
        );
        assert_eq!(
            GatewayError::InvalidApiKey("test".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            GatewayError::QuotaExceeded("test".to_string()).status_code(),
            StatusCode::PAYMENT_REQUIRED
        );
    }

    #[test]
    fn test_error_types() {
        let err = GatewayError::RateLimitExceeded("test".to_string());
        assert_eq!(err.error_type(), "rate_limit_exceeded");

        let err = GatewayError::InvalidApiKey("test".to_string());
        assert_eq!(err.error_type(), "invalid_api_key");
    }
}
