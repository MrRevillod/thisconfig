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

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("No configuration sources configured")]
    NoSourcesConfigured,

    #[error("Current executable directory not found")]
    ExeDirNotFound,
}

impl ConfigError {
    pub const fn interpolation_error(message: String) -> Self {
        Self::InterpolationError { message }
    }

    pub fn key_not_found(key: impl Into<String>) -> Self {
        Self::KeyNotFound { key: key.into() }
    }
}
