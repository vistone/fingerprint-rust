//! # fingerprint-gateway
//!
//! High-performance API Gateway with rate limiting for fingerprint-rust.
//!
//! ## Features
//!
//! - **Rate Limiting**: Token bucket algorithm with Redis backend
//! - **Quota Management**: Multi-tier quota system (Free, Pro, Enterprise, Partner)
//! - **Metrics**: Prometheus metrics for monitoring
//! - **High Performance**: Built on actix-web, 10x faster than Python FastAPI
//! - **Type Safe**: Full Rust type safety
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────┐
//! │   HTTP Client   │
//! └────────┬────────┘
//!          │
//!          ▼
//! ┌─────────────────┐
//! │  actix-web API  │
//! │   Gateway       │
//! └────────┬────────┘
//!          │
//!          ├──────────────┐
//!          │              │
//!          ▼              ▼
//! ┌─────────────┐  ┌─────────────┐
//! │ Rate Limiter│  │  Prometheus │
//! │  (Redis)    │  │   Metrics   │
//! └─────────────┘  └─────────────┘
//! ```
//!
//! ## Usage
//!
//! ### As a library
//!
//! ```rust,no_run
//! use fingerprint_gateway::{GatewayConfig, run_server};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = GatewayConfig::from_env()?;
//!     run_server(config).await
//! }
//! ```
//!
//! ### As a binary
//!
//! ```bash
//! cargo run --bin gateway --release
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod auth;
pub mod config;
pub mod error;
pub mod metrics;
pub mod middleware;
pub mod models;
pub mod rate_limit;
pub mod routes;

use actix_web::{web, App, HttpServer};
use std::sync::Arc;
use tracing::{info, warn};

pub use config::GatewayConfig;
pub use error::{GatewayError, Result};
pub use models::QuotaTier;
pub use rate_limit::RateLimiter;

/// Run the API Gateway server
///
/// # Arguments
///
/// * `config` - Gateway configuration
///
/// # Returns
///
/// Returns `Ok(())` when server shuts down gracefully, or an error if startup fails
///
/// # Example
///
/// ```rust,no_run
/// use fingerprint_gateway::{GatewayConfig, run_server};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let config = GatewayConfig::default();
///     run_server(config).await
/// }
/// ```
pub async fn run_server(config: GatewayConfig) -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    info!(
        "Starting fingerprint-gateway v{}",
        env!("CARGO_PKG_VERSION")
    );
    info!("Configuration: {:?}", config);

    // Initialize rate limiter
    let rate_limiter = Arc::new(
        RateLimiter::new(config.redis_url.clone())
            .await
            .map_err(|e| {
                warn!("Failed to initialize rate limiter: {}", e);
                e
            })?,
    );

    info!("Rate limiter initialized with Redis backend");

    // Initialize API key validator
    let api_key_validator = Arc::new(auth::ApiKeyValidator::new());
    info!("API key validator initialized");

    // Start HTTP server
    let host = config.host.clone();
    let port = config.port;
    let workers = config.workers;

    info!(
        "Starting HTTP server on {}:{} with {} workers",
        host, port, workers
    );

    HttpServer::new(move || {
        App::new()
            // Share state
            .app_data(web::Data::new(rate_limiter.clone()))
            .app_data(web::Data::new(api_key_validator.clone()))
            .app_data(web::Data::new(config.clone()))
            // Middleware
            .wrap(tracing_actix_web::TracingLogger::default())
            .wrap(actix_cors::Cors::permissive())
            // Routes
            .configure(routes::configure)
    })
    .workers(workers)
    .bind((host.as_str(), port))?
    .run()
    .await
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = GatewayConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.workers, 4);
    }
}
