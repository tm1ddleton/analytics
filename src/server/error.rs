//! Error types for the REST API server

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use uuid::Uuid;

/// API error types
#[derive(Debug)]
pub enum ApiError {
    /// Asset not found in database
    AssetNotFound(String),
    /// Invalid parameter in request
    InvalidParameter(String),
    /// Invalid date range
    InvalidDateRange(String),
    /// Analytics computation failed
    ComputationFailed(String),
    /// Replay session not found
    SessionNotFound(Uuid),
    /// Too many concurrent sessions
    SessionLimitReached,
    /// Internal server error
    InternalError(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::AssetNotFound(asset) => write!(f, "Asset not found: {}", asset),
            ApiError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            ApiError::InvalidDateRange(msg) => write!(f, "Invalid date range: {}", msg),
            ApiError::ComputationFailed(msg) => write!(f, "Computation failed: {}", msg),
            ApiError::SessionNotFound(id) => write!(f, "Session not found: {}", id),
            ApiError::SessionLimitReached => write!(f, "Session limit reached"),
            ApiError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            ApiError::AssetNotFound(asset) => (
                StatusCode::NOT_FOUND,
                "AssetNotFound",
                format!("Asset '{}' not found in database", asset),
            ),
            ApiError::InvalidParameter(msg) => (
                StatusCode::BAD_REQUEST,
                "InvalidParameter",
                msg.clone(),
            ),
            ApiError::InvalidDateRange(msg) => (
                StatusCode::BAD_REQUEST,
                "InvalidDateRange",
                msg.clone(),
            ),
            ApiError::ComputationFailed(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ComputationFailed",
                msg.clone(),
            ),
            ApiError::SessionNotFound(id) => (
                StatusCode::NOT_FOUND,
                "SessionNotFound",
                format!("Replay session '{}' not found", id),
            ),
            ApiError::SessionLimitReached => (
                StatusCode::SERVICE_UNAVAILABLE,
                "SessionLimitReached",
                "Maximum number of concurrent sessions reached".to_string(),
            ),
            ApiError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                msg.clone(),
            ),
        };

        let body = Json(json!({
            "error": error_type,
            "message": message,
        }));

        (status, body).into_response()
    }
}

// Conversions from other error types

impl From<crate::dag::DagError> for ApiError {
    fn from(err: crate::dag::DagError) -> Self {
        match err {
            crate::dag::DagError::NodeNotFound(msg) => ApiError::InvalidParameter(msg),
            crate::dag::DagError::DataProviderError(msg) => {
                if msg.contains("not found") {
                    ApiError::AssetNotFound(msg)
                } else {
                    ApiError::ComputationFailed(msg)
                }
            }
            _ => ApiError::ComputationFailed(err.to_string()),
        }
    }
}

impl From<crate::time_series::DataProviderError> for ApiError {
    fn from(err: crate::time_series::DataProviderError) -> Self {
        match err {
            crate::time_series::DataProviderError::AssetNotFound => {
                ApiError::AssetNotFound("Asset not found".to_string())
            }
            _ => ApiError::InternalError(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::InvalidParameter(format!("JSON error: {}", err))
    }
}

impl From<chrono::ParseError> for ApiError {
    fn from(err: chrono::ParseError) -> Self {
        ApiError::InvalidDateRange(format!("Date parse error: {}", err))
    }
}

