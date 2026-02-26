//! AI Native Browser Gateway Library
//!
//! This library provides the core functionality for the gateway application.

use std::sync::atomic::{AtomicBool, Ordering};

static AUTO_START_ENABLED: AtomicBool = AtomicBool::new(false);

pub fn get_auto_start() -> bool {
    AUTO_START_ENABLED.load(Ordering::SeqCst)
}

pub fn toggle_auto_start() -> bool {
    let current = AUTO_START_ENABLED.load(Ordering::SeqCst);
    let new = !current;
    AUTO_START_ENABLED.store(new, Ordering::SeqCst);
    new
}

/// Get engine/API status
pub async fn get_status() -> serde_json::Value {
    let client = reqwest::Client::new();
    let health = client
        .get("http://127.0.0.1:9222/health")
        .send()
        .await
        .ok()
        .map(|r| r.status().is_success())
        .unwrap_or(false);
    
    serde_json::json!({
        "status": if health { "running" } else { "stopped" },
        "auto_start": get_auto_start(),
        "api_server": if health { "ok" } else { "unavailable" }
    })
}
