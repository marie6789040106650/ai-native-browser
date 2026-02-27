//! Core Engine Server Entry Point
//!
//! Standalone server for testing the core engine.

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn get_browser_path() -> Option<String> {
    // Try common Chrome locations
    let paths = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chrome.app/Contents/MacOS/Chrome",
        "/usr/bin/google-chrome",
        "/usr/local/bin/google-chrome",
    ];
    
    for path in paths {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    
    // Try environment variable
    std::env::var("BROWSER_PATH").ok()
}

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
    let mut config = core_engine::EngineConfig::default();
    
    // Override with environment variables
    if let Ok(path) = std::env::var("BROWSER_PATH") {
        config.browser_path = Some(path);
    } else if let Some(path) = get_browser_path() {
        config.browser_path = Some(path);
    }
    
    if let Ok(host) = std::env::var("SERVER_HOST") {
        config.host = host;
    }
    
    if let Ok(port) = std::env::var("SERVER_PORT") {
        config.port = port.parse().unwrap_or(9222);
    }
    
    if let Ok(headless) = std::env::var("HEADLESS") {
        config.headless = headless != "false";
    }
    
    tracing::info!("Browser path: {:?}", config.browser_path);

    // Start server
    core_engine::server::start_server(config).await?;

    Ok(())
}
