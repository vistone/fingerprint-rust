//! API Gateway binary

use fingerprint_gateway::{run_server, GatewayConfig};
use tracing::info;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from environment
    let config = GatewayConfig::from_env()?;

    info!(
        "Starting fingerprint-gateway v{}",
        env!("CARGO_PKG_VERSION")
    );
    info!("Configuration loaded: {:?}", config);

    // Run server
    run_server(config).await?;

    Ok(())
}
