//! Error Handling Module
//!
//! Provides structured error types and middleware for the API server.

use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::json;
use std::fmt;

/// Application error types
#[derive(Debug)]
pub enum AppError {
    /// Bad request - client error
    BadRequest(String),
    /// Unauthorized
    Unauthorized(String),
    /// Not found
    NotFound(String),
    /// Internal server error
    Internal(String),
    /// Service unavailable
    ServiceUnavailable(String),
    /// Timeout
    Timeout(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal Error: {}", msg),
            AppError::ServiceUnavailable(msg) => write!(f, "Service Unavailable: {}", msg),
            AppError::Timeout(msg) => write!(f, "Timeout: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            AppError::Timeout(msg) => (StatusCode::REQUEST_TIMEOUT, msg),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

/// Result type alias for handlers
pub type AppResult<T> = Result<T, AppError>;

/// Helper functions for creating errors
impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        AppError::BadRequest(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        AppError::NotFound(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        AppError::Internal(msg.into())
    }

    pub fn service_unavailable(msg: impl Into<String>) -> Self {
        AppError::ServiceUnavailable(msg.into())
    }

    pub fn timeout(msg: impl Into<String>) -> Self {
        AppError::Timeout(msg.into())
    }
}

/// Error context for wrapping errors with additional context
pub fn wrap_err<E: std::error::Error>(err: E, context: &str) -> AppError {
    AppError::Internal(format!("{}: {}", context, err))
}

/// Validation helpers
pub mod validation {
    use super::*;

    /// Validate required string field
    pub fn required_field<T: AsRef<str>>(value: Option<T>, field_name: &str) -> AppResult<String> {
        value
            .map(|v| v.as_ref().to_string())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AppError::bad_request(format!("{} is required", field_name)))
    }

    /// Validate positive integer
    pub fn positive_int(value: Option<u32>, field_name: &str) -> AppResult<u32> {
        value
            .filter(|&v| v > 0)
            .ok_or_else(|| AppError::bad_request(format!("{} must be a positive integer", field_name)))
    }

    /// Validate URL
    pub fn url(value: &str) -> AppResult<()> {
        if value.starts_with("http://") || value.starts_with("https://") {
            Ok(())
        } else {
            Err(AppError::bad_request("URL must start with http:// or https://"))
        }
    }
}

/// Recovery strategies for different error types
pub mod recovery {
    use super::*;

    /// Determine if an error is retryable
    pub fn is_retryable(error: &AppError) -> bool {
        matches!(
            error,
            AppError::ServiceUnavailable(_) | AppError::Timeout(_)
        )
    }

    /// Get recommended retry delay
    pub fn retry_delay(attempt: u32) -> u64 {
        // Exponential backoff: 1s, 2s, 4s, 8s, max 30s
        std::cmp::min(2_u64.pow(attempt), 30)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::bad_request("invalid input");
        assert_eq!(err.to_string(), "Bad Request: invalid input");
    }

    #[test]
    fn test_validation() {
        assert!(validation::url("https://example.com").is_ok());
        assert!(validation::url("invalid").is_err());
    }

    #[test]
    fn test_retryable() {
        assert!(recovery::is_retryable(&AppError::service_unavailable("busy")));
        assert!(recovery::is_retryable(&AppError::timeout("slow")));
        assert!(!recovery::is_retryable(&AppError::bad_request("invalid")));
    }
}
