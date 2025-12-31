use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found at {0}")]
    FileNotFound(String),

    #[error("Failed to read configuration file: {source}")]
    ReadError {
        #[from]
        source: std::io::Error,
    },

    #[error("Environment variable interpolation error: {message}")]
    InterpolationError { message: String },

    #[error("Configuration key '{key}' not found")]
    KeyNotFound { key: String },

    #[error("Deserialization error: {source}")]
    DeserializeError {
        #[from]
        source: toml::de::Error,
    },

    #[error("Current executable directory not found")]
    ExeDirNotFound,

    #[error("Missing application configuration in request extensions")]
    ExtensionError,
}

impl ConfigError {
    pub fn interpolation_error(message: String) -> Self {
        ConfigError::InterpolationError { message }
    }

    pub fn key_not_found(key: impl Into<String>) -> Self {
        ConfigError::KeyNotFound { key: key.into() }
    }
}

impl IntoResponse for ConfigError {
    fn into_response(self) -> Response {
        tracing::error!("Configuration error: {}", self);

        let json = json!({
            "code": 500,
            "success": false,
            "message": "Internal Server Error",
        });

        (StatusCode::INTERNAL_SERVER_ERROR, Json(json)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_response_status() {
        let error = ConfigError::FileNotFound("/test".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
