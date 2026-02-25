//! AI Native Browser Gateway Application
//! 
//! Tauri application for managing the core engine lifecycle and system integration.
//! Blueprint 5: Client Integration & Lifecycle
//!
//! Features:
//! - System tray with manual/auto-start toggle
//! - Background API server management
//! - Log panel for API call timing
//! - Shadow window for human intervention

use std::sync::atomic::{AtomicBool, Ordering};
use std::process::Command;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime,
};
use tracing::{error, info, warn};

mod tray;

/// Global state
static AUTO_START_ENABLED: AtomicBool = AtomicBool::new(false);

/// Run the Tauri application
pub fn run() {
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
            
            // Setup system tray
            setup_tray(app)?;
            
            // Start API server in background
            start_api_server()?;
            
            info!("Application setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_core_engine,
            stop_core_engine,
            get_status,
            toggle_auto_start,
            get_api_logs,
            request_human_bridge,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Setup system tray with menu
fn setup_tray<R: Runtime>(app: &tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let start = MenuItem::with_id(app, "start", "Start Engine", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop", "Stop Engine", true, None::<&str>)?;
    let auto_start = MenuItem::with_id(app, "auto_start", "Enable Auto-Start", true, None::<&str>)?;
    let logs = MenuItem::with_id(app, "logs", "View Logs", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show, &separator, &start, &stop, &auto_start, &separator, &logs, &quit])?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .menu_on_left_click(false)
        .tooltip("AI Native Browser Gateway")
        .on_menu_event(move |app, event| {
            handle_menu_event(app, event.id.as_ref());
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    info!("System tray initialized");
    Ok(())
}

/// Handle tray menu events
fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, id: &str) {
    match id {
        "quit" => {
            info!("Quit requested from tray");
            app.exit(0);
        }
        "show" => {
            info!("Show window requested");
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "start" => {
            info!("Start engine requested");
            let _ = app.emit("engine:start", ());
        }
        "stop" => {
            info!("Stop engine requested");
            let _ = app.emit("engine:stop", ());
        }
        "auto_start" => {
            let current = AUTO_START_ENABLED.load(Ordering::SeqCst);
            AUTO_START_ENABLED.store(!current, Ordering::SeqCst);
            info!("Auto-start toggled: {}", !current);
            let _ = app.emit("auto_start:toggled", !current);
        }
        "logs" => {
            info!("View logs requested");
            let _ = app.emit("logs:show", ());
        }
        _ => {}
    }
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
            // Start the server (in production, would spawn as child process)
            info!("API server would be started here (production: spawn subprocess)");
        }
    }
    
    Ok(())
}

/// Start the core engine
#[tauri::command]
async fn start_core_engine(app: AppHandle) -> Result<String, String> {
    info!("Starting core engine...");
    
    // Emit event to frontend
    let _ = app.emit("engine:starting", ());
    
    // TODO: In production, start the browser via API call
    // For now, just signal the frontend
    
    let _ = app.emit("engine:started", ());
    
    Ok("Core engine started".to_string())
}

/// Stop the core engine
#[tauri::command]
async fn stop_core_engine(app: AppHandle) -> Result<String, String> {
    info!("Stopping core engine...");
    
    let _ = app.emit("engine:stopping", ());
    
    // TODO: In production, stop the browser via API call
    
    let _ = app.emit("engine:stopped", ());
    
    Ok("Core engine stopped".to_string())
}

/// Get engine/API status
#[tauri::command]
async fn get_status() -> Result<serde_json::Value, String> {
    // Check API server health
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
        "auto_start": AUTO_START_ENABLED.load(Ordering::SeqCst),
        "api_server": if health { "ok" } else { "unavailable" },
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Toggle auto-start setting
#[tauri::command]
fn toggle_auto_start() -> bool {
    let current = AUTO_START_ENABLED.load(Ordering::SeqCst);
    let new = !current;
    AUTO_START_ENABLED.store(new, Ordering::SeqCst);
    info!("Auto-start set to: {}", new);
    new
}

/// Get recent API logs (placeholder)
#[tauri::command]
async fn get_api_logs() -> Result<Vec<serde_json::Value>, String> {
    // TODO: In production, would fetch from log aggregation
    Ok(vec![
        serde_json::json!({
            "timestamp": "2024-01-01T00:00:00Z",
            "endpoint": "/v1/sense",
            "duration_ms": 150,
            "status": 200
        }),
    ])
}

/// Request human bridge (for captcha/401 scenarios)
#[tauri::command]
async fn request_human_bridge(app: AppHandle, reason: String) -> Result<String, String> {
    warn!("Human bridge requested: {}", reason);
    
    // Emit event to show shadow window
    let _ = app.emit("human_bridge:requested", serde_json::json!({
        "reason": reason,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));
    
    // TODO: In production, show shadow window and wait for completion
    
    Ok("Human intervention requested".to_string())
}
