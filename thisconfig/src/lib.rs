mod builder;
mod config;
mod env;
mod error;
mod utils;

pub use builder::ConfigBuilder;
pub use config::Config;
pub use error::ConfigError;
pub use thisconfig_macros::config;

#[cfg(feature = "byte-unit")]
pub use utils::byte_unit::ByteConfig;

#[cfg(feature = "time-unit")]
pub use utils::time_unit::TimeConfig;

use serde::de::DeserializeOwned;

/// Trait for configuration section types.
///
/// Types implementing this trait can be used with `Config::get()` to extract
/// and deserialize specific sections from the configuration table.
pub trait ConfigItem: DeserializeOwned + Clone + Send + Sync + 'static {
    /// Returns the TOML section key for this configuration type.
    ///
    /// # Example
    ///
    /// ```toml
    /// [database]
    /// host = "localhost"
    /// ```
    /// In this example, the `key()` method for `DatabaseConfig` would return `"database"`.
    fn key() -> &'static str;
}
