//! Core Engine Server Entry Point
//!
//! Standalone server for testing the core engine.

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,core_engine=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting AI Native Browser Core Engine Server...");

    // Load configuration
    let config = core_engine::EngineConfig::default();

    // Start server
    core_engine::server::start_server(config).await?;

    Ok(())
}
