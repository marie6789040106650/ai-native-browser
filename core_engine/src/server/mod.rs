//! API Server Module
//! 
//! Exposes the AI Browser Interface via REST/WebSocket endpoints.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use parking_lot::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{error, info};

use crate::{CoreError, EngineConfig};
use crate::parser::SemanticNode;

/// Shared application state
pub struct AppState {
    pub config: EngineConfig,
    pub parser: RwLock<crate::parser::SemanticParser>,
    pub inspector: RwLock<crate::inspector::TrafficInspector>,
    // Note: In real implementation, we'd have CDP client here
}

/// Sense response
#[derive(Debug, Serialize)]
pub struct SenseResponse {
    pub status: String,
    pub url: String,
    pub page_title: String,
    pub semantic_tree: Vec<SemanticNode>,
}

/// Act request
#[derive(Debug, Deserialize)]
pub struct ActRequest {
    pub action: String,
    pub target_id: u32,
    #[serde(default)]
    pub value: String,
}

/// Act response
#[derive(Debug, Serialize)]
pub struct ActResponse {
    pub success: bool,
    pub message: String,
    pub new_state: Option<InspectorState>,
}

/// Inspector state for API responses
#[derive(Debug, Serialize)]
pub struct InspectorState {
    pub active_requests: u32,
    pub is_ready: bool,
}

/// Health check handler
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "ai-native-browser-core",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Sense handler - Get semantic page representation
async fn sense_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SenseResponse>, (StatusCode, String)> {
    info!("sense endpoint called");
    
    // In real implementation:
    // 1. Call wait_until_ready on inspector
    // 2. Get full DOM via CDP
    // 3. Parse with semantic parser
    // 4. Return result
    
    let inspector = state.inspector.read();
    let _ready_state = inspector.get_state();
    
    // Placeholder response
    let response = SenseResponse {
        status: "ready".to_string(),
        url: "https://example.com".to_string(),
        page_title: "Example".to_string(),
        semantic_tree: vec![],
    };
    
    Ok(Json(response))
}

/// Act handler - Perform action on page
async fn act_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ActRequest>,
) -> Result<Json<ActResponse>, (StatusCode, String)> {
    info!("act endpoint called: action={}, target_id={}", req.action, req.target_id);
    
    // In real implementation:
    // 1. Find element by ID in semantic tree
    // 2. Get CSS selector
    // 3. Execute CDP action (click, type, etc.)
    // 4. Wait for ready
    // 5. Return new state
    
    let inspector = state.inspector.read();
    let inspector_state = inspector.get_state();
    
    let response = ActResponse {
        success: true,
        message: format!("Action '{}' executed successfully", req.action),
        new_state: Some(InspectorState {
            active_requests: inspector_state.active_requests,
            is_ready: inspector_state.is_ready,
        }),
    };
    
    Ok(Json(response))
}

/// Human bridge handler - Request human intervention
async fn human_bridge_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    error!("human_bridge called: {:?}", req);
    
    // In real implementation:
    // 1. Signal Tauri to show shadow window
    // 2. Wait for human completion
    // 3. Return success
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Human intervention requested"
    })))
}

/// Create the router
pub fn create_router(config: &EngineConfig) -> Router {
    let app_state = Arc::new(AppState {
        config: config.clone(),
        parser: RwLock::new(crate::parser::SemanticParser::default_parser()),
        inspector: RwLock::new(crate::inspector::TrafficInspector::default_inspector()),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health_check))
        .route("/v1/sense", get(sense_handler))
        .route("/v1/act", post(act_handler))
        .route("/v1/human_bridge", post(human_bridge_handler))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(app_state)
}

/// Start the API server
pub async fn start_server(config: EngineConfig) -> Result<(), CoreError> {
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let router = create_router(&config);
    
    info!("Starting API server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| CoreError::Server(e.to_string()))?;
    
    axum::serve(listener, router)
        .await
        .map_err(|e| CoreError::Server(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let config = EngineConfig::default();
        let router = create_router(&config);
        assert!(router.is_ok());
    }
}
