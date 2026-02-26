//! AI Native Browser - Core Engine
//! 
//! This crate provides the core functionality for browser automation,
//! including CDP control, semantic parsing, traffic inspection, and API server.

pub mod cdp;
pub mod parser;
pub mod inspector;
pub mod server;
pub mod error;

pub use anyhow::Result;
pub use thiserror::Error;

/// Core engine error types
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Browser error: {0}")]
    Browser(String),
    
    #[error("CDP error: {0}")]
    Cdp(String),
    
    #[error("Parser error: {0}")]
    Parser(String),
    
    #[error("Inspector error: {0}")]
    Inspector(String),
    
    #[error("Server error: {0}")]
    Server(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}

impl serde::Serialize for CoreError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Engine configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EngineConfig {
    /// Browser launch options
    pub browser_path: Option<String>,
    pub user_data_dir: Option<String>,
    pub headless: bool,
    pub sandbox: bool,
    
    /// Server options
    pub host: String,
    pub port: u16,
    
    /// Inspector options  
    pub dom_idle_threshold_ms: u64,
    pub network_idle_threshold_ms: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            browser_path: None,
            user_data_dir: None,
            headless: true,
            sandbox: true,
            host: "127.0.0.1".to_string(),
            port: 9222,
            dom_idle_threshold_ms: 500,
            network_idle_threshold_ms: 1000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_config_default() {
        let config = EngineConfig::default();
        assert!(config.headless);
        assert!(config.sandbox);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 9222);
    }

    #[test]
    fn test_engine_config_custom() {
        let config = EngineConfig {
            browser_path: Some("/usr/bin/chrome".to_string()),
            user_data_dir: Some("/tmp/profile".to_string()),
            headless: false,
            sandbox: false,
            host: "0.0.0.0".to_string(),
            port: 8080,
            dom_idle_threshold_ms: 1000,
            network_idle_threshold_ms: 2000,
        };
        
        assert!(!config.headless);
        assert!(!config.sandbox);
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "0.0.0.0");
    }

    #[test]
    fn test_core_error_message() {
        let err = CoreError::Browser("test error".to_string());
        assert!(err.to_string().contains("Browser"));
        assert!(err.to_string().contains("test error"));
    }
}
