//! AI Native Browser Gateway - Main Entry Point
//!
//! Tauri application for managing the core engine lifecycle and system integration.

use std::process::Command;
use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, warn};

/// Run the Tauri application
fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,gateway_app=debug".into()),
        )
        .init();

    info!("===========================================");
    info!("AI Native Browser Gateway v{}", env!("CARGO_PKG_VERSION"));
    info!("===========================================");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            info!("Setting up application...");
            
            // Get main window
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_title("AI Native Browser Gateway");
            }
            
            // Start API server in background
            start_api_server()?;
            
            info!("Application setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_core_engine,
            stop_core_engine,
            get_status,
            get_api_logs,
            request_human_bridge,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Start API server in background
fn start_api_server() -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting API server in background...");
    
    // Check if already running
    let output = Command::new("curl")
        .args(["-s", "http://127.0.0.1:9222/health"])
        .output();
    
    match output {
        Ok(o) if o.status.success() => {
            info!("API server already running");
        }
        _ => {
            info!("API server would be started here (production: spawn subprocess)");
        }
    }
    
    Ok(())
}

/// Start the core engine
#[tauri::command]
async fn start_core_engine(app: AppHandle) -> Result<String, String> {
    info!("Starting core engine...");
    let _ = app.emit("engine:starting", ());
    let _ = app.emit("engine:started", ());
    Ok("Core engine started".to_string())
}

/// Stop the core engine
#[tauri::command]
async fn stop_core_engine(app: AppHandle) -> Result<String, String> {
    info!("Stopping core engine...");
    let _ = app.emit("engine:stopping", ());
    let _ = app.emit("engine:stopped", ());
    Ok("Core engine stopped".to_string())
}

/// Get engine/API status
#[tauri::command]
async fn get_status() -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let health = client
        .get("http://127.0.0.1:9222/health")
        .send()
        .await
        .ok()
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    
    Ok(serde_json::json!({
        "status": if health { "running" } else { "stopped" },
        "api_server": if health { "ok" } else { "unavailable" },
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Get recent API logs (placeholder)
#[tauri::command]
async fn get_api_logs() -> Result<Vec<serde_json::Value>, String> {
    Ok(vec![])
}

/// Request human bridge (for captcha/401 scenarios)
#[tauri::command]
async fn request_human_bridge(app: AppHandle, reason: String) -> Result<String, String> {
    warn!("Human bridge requested: {}", reason);
    let _ = app.emit("human_bridge:requested", serde_json::json!({
        "reason": reason,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));
    Ok("Human intervention requested".to_string())
}
