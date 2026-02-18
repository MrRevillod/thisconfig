use crate::env::expand_env_variables;
use crate::{ConfigError, ConfigItem};

use serde::de::{DeserializeOwned, IntoDeserializer};
use std::{env::current_exe, fs, path::Path, str::FromStr, sync::Arc};
use toml::{Table, Value};

#[cfg(feature = "validation")]
use validator::Validate;

const CONFIG_ENV_VAR: &str = "CONFIG_FILE_PATH";
const DEFAULT_CONFIG_PATH: &str = "config/config.toml";

/// Configuration loaded from TOML files.
///
/// Loads configuration with the following priority:
/// 1. Explicit path via `from_path()`
/// 2. `CONFIG_FILE_PATH` environment variable
/// 3. `config/config.toml`
/// 4. `<executable_dir>/config/config.toml`
#[derive(Debug, Clone, Default)]
pub struct Config {
    inner: Arc<Table>,
}

impl Config {
    /// Loads configuration using default priority order.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if no configuration file is found or contains invalid TOML.
    pub fn new() -> Result<Self, ConfigError> {
        let content = Self::load_config_file(None)?;

        let expanded = expand_env_variables(&content).map_err(ConfigError::interpolation_error)?;

        Ok(Self {
            inner: Arc::new(Table::from_str(&expanded)?),
        })
    }

    /// Loads configuration from a specific file path without fallbacks.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::FileNotFound` if the file doesn't exist.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = Self::load_config_file(Some(path.as_ref()))?;

        let expanded = expand_env_variables(&content).map_err(ConfigError::interpolation_error)?;

        Ok(Self {
            inner: Arc::new(Table::from_str(&expanded)?),
        })
    }

    /// Retrieves and deserializes a configuration section.
    ///
    /// Type `T` must implement `ConfigItem` via `#[config(key = "section")]` macro.
    pub fn get<T: DeserializeOwned + ConfigItem>(&self) -> Option<T> {
        let key = T::key();

        let item = self.inner.get(key).cloned()?;
        let value = Value::into_deserializer(item);

        T::deserialize(value).ok()
    }

    #[cfg(feature = "validation")]
    pub fn get_validated<T>(&self) -> Result<T, ConfigError>
    where
        T: DeserializeOwned + ConfigItem + Validate,
    {
        let key = T::key();

        let item = self
            .inner
            .get(key)
            .cloned()
            .ok_or_else(|| ConfigError::KeyNotFound {
                key: key.to_string(),
            })?;

        let value = Value::into_deserializer(item);

        let deserialized: T = T::deserialize(value)?;

        deserialized
            .validate()
            .map_err(|e| ConfigError::ValidationError {
                message: format!("Validation failed for '{key}': {e}"),
            })?;

        Ok(deserialized)
    }

    /// Retrieves a configuration section, panicking if not found or invalid.
    pub fn get_or_panic<T: DeserializeOwned + ConfigItem>(&self) -> T {
        self.get::<T>()
            .unwrap_or_else(|| panic!("Failed to load configuration for key '{}'", T::key()))
    }

    /// Retrieves a configuration section, returning default if not found or invalid.
    pub fn get_or_default<T: DeserializeOwned + ConfigItem + Default>(&self) -> T {
        self.get::<T>().unwrap_or_default()
    }

    fn load_config_file(path: Option<&Path>) -> Result<String, ConfigError> {
        if let Some(p) = path {
            if p.exists() {
                return Ok(fs::read_to_string(p)?);
            }

            return Err(ConfigError::FileNotFound(
                p.to_str().unwrap_or_default().to_string(),
            ));
        }

        if let Ok(env_path) = std::env::var(CONFIG_ENV_VAR) {
            let env_path = Path::new(&env_path);

            if env_path.exists() {
                return Ok(fs::read_to_string(env_path)?);
            }

            eprintln!(
                "Warning: {} is set to '{}' but file does not exist. Falling back to default paths.",
                CONFIG_ENV_VAR,
                env_path.display()
            );
        }

        let default_path = Path::new(DEFAULT_CONFIG_PATH);

        if default_path.exists() {
            return Ok(fs::read_to_string(default_path)?);
        }

        Self::load_from_exe_directory()
    }

    fn load_from_exe_directory() -> Result<String, ConfigError> {
        let exe_path = current_exe().map_err(|_| ConfigError::ExeDirNotFound)?;
        let exe_dir = exe_path.parent().ok_or(ConfigError::ExeDirNotFound)?;

        let fallback_path = exe_dir.join(DEFAULT_CONFIG_PATH);

        if !fallback_path.exists() {
            return Err(ConfigError::FileNotFound(
                fallback_path.to_str().unwrap_or_default().to_string(),
            ));
        }

        Ok(fs::read_to_string(fallback_path)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Clone, Deserialize, PartialEq)]
    struct TestConfig {
        name: String,
        port: u16,
    }

    impl ConfigItem for TestConfig {
        fn key() -> &'static str {
            "test"
        }
    }

    #[test]
    fn test_from_path_valid_toml() {
        let temp_file = tempfile::NamedTempFile::new().expect("failed to create temp file");
        let path = temp_file.path();
        fs::write(path, "[test]\nname = \"myapp\"\nport = 8080").expect("failed to write");

        let config = Config::from_path(path).expect("failed to load config");
        let test_config = config
            .get::<TestConfig>()
            .expect("failed to get test config");

        assert_eq!(test_config.name, "myapp");
        assert_eq!(test_config.port, 8080);
    }

    #[test]
    fn test_from_path_nonexistent_file() {
        let result = Config::from_path("/nonexistent/path/config.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_missing_key() {
        let temp_file = tempfile::NamedTempFile::new().expect("failed to create temp file");
        let path = temp_file.path();
        fs::write(path, "[other]\nvalue = 1").expect("failed to write");

        let config = Config::from_path(path).expect("failed to load config");
        let result = config.get::<TestConfig>();

        assert!(result.is_none());
    }

    #[test]
    fn test_macro_config() {
        use crate::ConfigItem;

        #[derive(Debug, Clone, Deserialize)]
        struct MacroConfig {
            value: String,
        }

        impl ConfigItem for MacroConfig {
            fn key() -> &'static str {
                "macro_test"
            }
        }

        let temp_file = tempfile::NamedTempFile::new().expect("failed to create temp file");
        let path = temp_file.path();
        fs::write(path, "[macro_test]\nvalue = \"works\"").expect("failed to write");

        let config = Config::from_path(path).expect("failed to load config");
        let macro_config = config
            .get::<MacroConfig>()
            .expect("failed to get macro config");

        assert_eq!(macro_config.value, "works");
    }

    #[cfg(feature = "validation")]
    #[test]
    fn test_get_validated_success() {
        use crate::ConfigItem;
        use validator::Validate;

        #[derive(Debug, Clone, Deserialize, Validate)]
        struct ValidConfigTest {
            #[validate(length(min = 1))]
            name: String,
            #[validate(range(min = 1, max = 100))]
            value: u32,
        }

        impl ConfigItem for ValidConfigTest {
            fn key() -> &'static str {
                "valid_test"
            }
        }

        #[derive(Debug, Clone, Deserialize, Validate)]
        struct ValidConfig {
            #[validate(length(min = 1))]
            name: String,
            #[validate(range(min = 1, max = 100))]
            value: u32,
        }

        impl ConfigItem for ValidConfig {
            fn key() -> &'static str {
                "valid_test"
            }
        }

        let temp_file = tempfile::NamedTempFile::new().expect("failed to create temp file");
        let path = temp_file.path();
        fs::write(path, "[valid_test]\nname = \"test\"\nvalue = 50").expect("failed to write");

        let config = Config::from_path(path).expect("failed to load config");
        let valid_config = config
            .get_validated::<ValidConfigTest>()
            .expect("failed to get validated config");

        assert_eq!(valid_config.name, "test");
        assert_eq!(valid_config.value, 50);
    }

    #[cfg(feature = "validation")]
    #[test]
    fn test_get_validated_failure() {
        use crate::ConfigItem;
        use validator::Validate;

        #[derive(Debug, Clone, Deserialize, Validate)]
        struct InvalidConfigTest {
            #[validate(length(min = 3))]
            name: String,
        }

        impl ConfigItem for InvalidConfigTest {
            fn key() -> &'static str {
                "invalid_test"
            }
        }

        #[derive(Debug, Clone, Deserialize, Validate)]
        struct InvalidConfig {
            #[validate(length(min = 3))]
            name: String,
        }

        impl ConfigItem for InvalidConfig {
            fn key() -> &'static str {
                "invalid_test"
            }
        }

        let temp_file = tempfile::NamedTempFile::new().expect("failed to create temp file");
        let path = temp_file.path();
        fs::write(path, "[invalid_test]\nname = \"a\"").expect("failed to write"); // name too short

        let config = Config::from_path(path).expect("failed to load config");
        let result = config.get_validated::<InvalidConfigTest>();

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Validation failed"));
        }
    }
}
