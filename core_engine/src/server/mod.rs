//! API Server Module
//! 
//! Exposes the AI Browser Interface via REST/WebSocket endpoints.
//! Blueprint 4: ABI Interface Implementation
//!
//! Endpoints:
//! - GET  /health          - Health check
//! - GET  /v1/sense        - Get semantic page representation
//! - POST /v1/act          - Perform action on page
//! - POST /v1/human_bridge - Request human intervention
//! - POST /v1/browser/start - Start browser
//! - POST /v1/browser/stop  - Stop browser
//! - GET  /v1/status       - Get current status

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
use tracing::{error, info, warn};

use crate::{CoreError, EngineConfig};
use crate::parser::SemanticNode;

/// Shared application state
pub struct AppState {
    pub config: EngineConfig,
    pub parser: RwLock<crate::parser::SemanticParser>,
    pub inspector: RwLock<crate::inspector::TrafficInspector>,
    pub cdp_client: RwLock<crate::cdp::CdpClient>,
    pub is_running: RwLock<bool>,
}

/// ============ Request/Response Types ============

/// Sense response - Blueprint 4.1
#[derive(Debug, Serialize)]
pub struct SenseResponse {
    pub status: String,
    pub url: String,
    pub page_title: String,
    pub semantic_tree: Vec<SemanticNode>,
}

/// Act request - Blueprint 4.2
#[derive(Debug, Deserialize)]
pub struct ActRequest {
    pub action: String,
    pub target_id: u32,
    #[serde(default)]
    pub value: String,
}

/// Supported actions
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionType {
    Click,
    Type,
    Scroll,
    Wait,
    Hover,
    Select,
}

impl ActionType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "click" => Some(ActionType::Click),
            "type" | "fill" => Some(ActionType::Type),
            "scroll" => Some(ActionType::Scroll),
            "wait" => Some(ActionType::Wait),
            "hover" => Some(ActionType::Hover),
            "select" => Some(ActionType::Select),
            _ => None,
        }
    }
}

/// Act response
#[derive(Debug, Serialize)]
pub struct ActResponse {
    pub success: bool,
    pub message: String,
    pub new_state: Option<InspectorStateResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Inspector state for API responses
#[derive(Debug, Serialize)]
pub struct InspectorStateResponse {
    pub active_requests: u32,
    pub is_ready: bool,
}

/// Browser start request
#[derive(Debug, Deserialize)]
pub struct BrowserStartRequest {
    #[serde(default)]
    pub headless: bool,
    #[serde(default)]
    pub user_data_dir: Option<String>,
}

/// Status response
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub browser_running: bool,
    pub inspector: InspectorStateResponse,
    pub url: Option<String>,
}

// ============ Handlers ============

/// Health check handler
async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "ai-native-browser-core",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Sense handler - Blueprint 4.1
/// GET /v1/sense
/// Logic: wait_until_ready -> get DOM -> parse -> return
async fn sense_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SenseResponse>, (StatusCode, String)> {
    info!("sense endpoint called");
    
    // Check browser status
    let is_running = *state.is_running.read();
    if !is_running {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Browser not running. Start with POST /v1/browser/start".to_string(),
        ));
    }
    
    // Get inspector and CDP client
    let inspector = state.inspector.read();
    
    // Get page info from CDP (placeholder)
    let cdp = state.cdp_client.read();
    let url = cdp.get_url().unwrap_or_default();
    let title = cdp.get_title().unwrap_or_default();
    let content = cdp.content().unwrap_or_default();
    
    // Update inspector state
    inspector.set_page_info(Some(url.clone()), Some(title.clone()));
    
    // Parse HTML (placeholder - returns empty for now)
    let mut parser = state.parser.write();
    let semantic_tree = parser.parse_html(&content);
    
    info!("sense returned: {} nodes from {}", semantic_tree.len(), url);
    
    let response = SenseResponse {
        status: "ready".to_string(),
        url,
        page_title: title,
        semantic_tree,
    };
    
    Ok(Json(response))
}

/// Act handler - Blueprint 4.2
/// POST /v1/act
/// Logic: find element by ID -> execute CDP action -> wait -> return
async fn act_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ActRequest>,
) -> Result<Json<ActResponse>, (StatusCode, String)> {
    info!("act endpoint: action={}, target_id={}, value={}", req.action, req.target_id, req.value);
    
    // Check browser status
    let is_running = *state.is_running.read();
    if !is_running {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Browser not running".to_string(),
        ));
    }
    
    // Parse action type
    let action_type = ActionType::from_str(&req.action);
    if action_type.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Invalid action: {}. Valid: click, type, scroll, wait, hover", req.action),
        ));
    }
    
    // Execute action via CDP (simplified)
    let cdp = state.cdp_client.read();
    let _result = match action_type.unwrap() {
        ActionType::Click => {
            let selector = format!("[data-id=\"{}\"]", req.target_id);
            cdp.click(&selector).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            "clicked"
        }
        ActionType::Type => {
            let selector = format!("[data-id=\"{}\"]", req.target_id);
            cdp.type_text(&selector, &req.value).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            "typed"
        }
        ActionType::Scroll => {
            cdp.evaluate(&format!("window.scrollBy(0, {})", req.value)).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
            "scrolled"
        }
        ActionType::Wait | ActionType::Hover | ActionType::Select => {
            "not implemented"
        }
    };
    
    // Notify inspector of DOM change
    let inspector = state.inspector.read();
    inspector.dom_mutated();
    inspector.mark_ready();
    
    let inspector_state = inspector.get_state();
    
    let response = ActResponse {
        success: true,
        message: format!("Action '{}' executed (placeholder)", req.action),
        new_state: Some(InspectorStateResponse {
            active_requests: inspector_state.active_requests,
            is_ready: inspector_state.is_ready,
        }),
        error: None,
    };
    
    info!("act completed: {}", response.message);
    Ok(Json(response))
}

/// Human bridge handler - Blueprint 4.3
/// POST /v1/human_bridge
async fn human_bridge_handler(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    warn!("human_bridge called: {:?}", req);
    
    // TODO: Real implementation:
    // 1. Signal Tauri to show shadow window
    // 2. Wait for human completion
    // 3. Return success
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Human intervention requested. Please interact with the browser."
    })))
}

/// Browser start handler
async fn browser_start_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BrowserStartRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!("browser_start called: headless={}", req.headless);
    
    // Check if already running
    if *state.is_running.read() {
        return Err((
            StatusCode::CONFLICT,
            "Browser already running".to_string(),
        ));
    }
    
    // TODO: Real implementation:
    // 1. Configure CDP client with user_data_dir if provided
    // 2. Launch browser
    // 3. Set up network/DOM event listeners
    
    // Configure and launch browser
    let config = crate::cdp::BrowserConfig {
        headless: req.headless,
        user_data_dir: req.user_data_dir.clone(),
        browser_path: None,
        port: 9222,
    };
    
    // Create and launch CDP client
    let mut cdp = crate::cdp::CdpClient::new(config);
    let launch_result = cdp.launch();
    
    match launch_result {
        Ok(_) => {
            // Update state
            *state.cdp_client.write() = cdp;
            *state.is_running.write() = true;
            
            // Reset inspector
            state.inspector.read().reset();
            
            info!("Browser started successfully");
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Browser started"
            })))
        }
        Err(e) => {
            error!("Failed to start browser: {}", e);
            Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to start browser: {}", e)))
        }
    }
}

/// Browser stop handler
async fn browser_stop_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!("browser_stop called");
    
    // Check if running
    if !*state.is_running.read() {
        return Err((
            StatusCode::CONFLICT,
            "Browser not running".to_string(),
        ));
    }
    
    // TODO: Real implementation:
    // 1. Close CDP client
    // 2. Clean up
    
    *state.is_running.write() = false;
    
    info!("Browser stopped");
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Browser stopped"
    })))
}

/// Status handler
async fn status_handler(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let inspector = state.inspector.read();
    let inspector_state = inspector.get_state();
    
    Json(serde_json::json!({
        "browser_running": *state.is_running.read(),
        "inspector": {
            "active_requests": inspector_state.active_requests,
            "is_ready": inspector_state.is_ready,
        },
        "url": inspector_state.url,
        "title": inspector_state.title,
    }))
}

// ============ Router Setup ============

/// Create the router with all endpoints
pub fn create_router(config: &EngineConfig) -> Router {
    let app_state = Arc::new(AppState {
        config: config.clone(),
        parser: RwLock::new(crate::parser::SemanticParser::default_parser()),
        inspector: RwLock::new(crate::inspector::TrafficInspector::default_inspector()),
        cdp_client: RwLock::new(crate::cdp::CdpClient::new(crate::cdp::BrowserConfig::default())),
        is_running: RwLock::new(false),
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Health
        .route("/health", get(health_check))
        // API v1
        .route("/v1/sense", get(sense_handler))
        .route("/v1/act", post(act_handler))
        .route("/v1/human_bridge", post(human_bridge_handler))
        .route("/v1/browser/start", post(browser_start_handler))
        .route("/v1/browser/stop", post(browser_stop_handler))
        .route("/v1/status", get(status_handler))
        // Layers
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(app_state)
}

/// Start the API server
pub async fn start_server(config: EngineConfig) -> Result<(), CoreError> {
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let router = create_router(&config);
    
    info!("===========================================");
    info!("AI Native Browser API Server");
    info!("Listening on: http://{}", addr);
    info!("Endpoints:");
    info!("  GET  /health           - Health check");
    info!("  GET  /v1/sense         - Get semantic page");
    info!("  POST /v1/act           - Perform action");
    info!("  POST /v1/human_bridge  - Human intervention");
    info!("  POST /v1/browser/start - Start browser");
    info!("  POST /v1/browser/stop  - Stop browser");
    info!("  GET  /v1/status        - Get status");
    info!("===========================================");
    
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
        let _router = create_router(&config);
        // Router is created successfully (no panic)
    }

    #[test]
    fn test_action_type_parsing() {
        assert!(ActionType::from_str("click").is_some());
        assert!(ActionType::from_str("type").is_some());
        assert!(ActionType::from_str("fill").is_some());
        assert!(ActionType::from_str("scroll").is_some());
        assert!(ActionType::from_str("wait").is_some());
        assert!(ActionType::from_str("hover").is_some());
        assert!(ActionType::from_str("select").is_some());
        assert!(ActionType::from_str("invalid").is_none());
    }

    #[test]
    fn test_action_type_case_insensitive() {
        assert!(ActionType::from_str("CLICK").is_some());
        assert!(ActionType::from_str("Click").is_some());
        assert!(ActionType::from_str("TYPE").is_some());
    }
}
