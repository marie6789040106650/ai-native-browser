//! End-to-End API Tests
//!
//! Tests for the AI Native Browser API server.

use reqwest;
use serde_json::json;

// Test configuration
const API_BASE: &str = "http://127.0.0.1:9222";

/// Test health endpoint
#[tokio::test]
async fn test_health_check() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health", API_BASE))
        .send()
        .await?;
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await?;
    assert_eq!(body["status"], "ok");
    
    Ok(())
}

/// Test status endpoint
#[tokio::test]
async fn test_status() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/v1/status", API_BASE))
        .send()
        .await?;
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await?;
    assert!(body["engine"].is_object());
    
    Ok(())
}

/// Test browser start endpoint
#[tokio::test]
async fn test_browser_start() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/v1/browser/start", API_BASE))
        .send()
        .await?;
    
    // Should return 200 or 409 (already running)
    assert!(response.status() == 200 || response.status() == 409);
    
    Ok(())
}

/// Test sense endpoint (requires browser running)
#[tokio::test]
async fn test_sense() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    // First ensure browser is started
    let _ = client
        .post(&format!("{}/v1/browser/start", API_BASE))
        .send()
        .await;
    
    // Wait for browser to start
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Now test sense
    let response = client
        .get(&format!("{}/v1/sense?url=https://example.com", API_BASE))
        .send()
        .await?;
    
    // Should return 200 or 400 (if no browser)
    if response.status() == 200 {
        let body: serde_json::Value = response.json().await?;
        assert!(body["semantic_tree"].is_array() || body["error"].is_string());
    }
    
    Ok(())
}

/// Test act endpoint (requires browser running)
#[tokio::test]
async fn test_act_click() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    // First ensure browser is started
    let _ = client
        .post(&format!("{}/v1/browser/start", API_BASE))
        .send()
        .await;
    
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Test click action
    let response = client
        .post(&format!("{}/v1/act", API_BASE))
        .json(&json!({
            "action": "click",
            "target_id": 1
        }))
        .send()
        .await?;
    
    // Should return 200 or 400 (invalid target)
    assert!(response.status() == 200 || response.status() == 400);
    
    Ok(())
}

/// Test error handling - invalid action
#[tokio::test]
async fn test_act_invalid_action() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(&format!("{}/v1/act", API_BASE))
        .json(&json!({
            "action": "invalid_action",
            "target_id": 1
        }))
        .send()
        .await?;
    
    // Should return 400 Bad Request
    assert_eq!(response.status(), 400);
    
    Ok(())
}

/// Test error handling - missing fields
#[tokio::test]
async fn test_act_missing_fields() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(&format!("{}/v1/act", API_BASE))
        .json(&json!({
            "action": "click"
            // missing target_id
        }))
        .send()
        .await?;
    
    // Should return 400 Bad Request
    assert_eq!(response.status(), 400);
    
    Ok(())
}

/// Test human bridge endpoint
#[tokio::test]
async fn test_human_bridge() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(&format!("{}/v1/human_bridge", API_BASE))
        .json(&json!({
            "reason": "captcha"
        }))
        .send()
        .await?;
    
    // Should return 200 (accepted)
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await?;
    assert!(body["request_id"].is_string());
    
    Ok(())
}
