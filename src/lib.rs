mod config;
mod env;
mod error;
mod extract;

use serde::de::DeserializeOwned;

pub use config::Config;
pub use error::ConfigError;
pub use extract::*;

pub use axum_config_macros::config;

/// Trait for configuration section types.
///
/// Types implementing this trait can be used with `Config::get()` to extract
/// and deserialize specific sections from the configuration table.
///
/// # Example
///
/// ```ignore
/// use axum_config::config;
/// use serde::Deserialize;
///
/// #[config(key = "database")]
/// #[derive(Debug, Clone, Deserialize)]
/// pub struct DatabaseConfig {
///     pub host: String,
///     pub port: u16,
/// }
/// ```
pub trait ConfigItem: DeserializeOwned + Clone + Send + Sync + 'static {
    /// Returns the TOML section key for this configuration type.
    fn key() -> &'static str;
}
