use crate::{ApplicationConfig, ConfigError, ConfigItem};
use axum::{extract::FromRequestParts, http::request::Parts};
use std::sync::Arc;

pub struct Config<T>(pub T);

impl<S, T> FromRequestParts<S> for Config<T>
where
    T: ConfigItem,
    S: Send + Sync,
{
    type Rejection = ConfigError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let config = parts
            .extensions
            .get::<Arc<ApplicationConfig>>()
            .ok_or(ConfigError::ExtensionError)?;

        let item = config.get::<T>()?;

        Ok(Config(item))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Clone, Deserialize)]
    struct MockConfig {
        value: String,
    }

    impl ConfigItem for MockConfig {
        fn key() -> &'static str {
            "mock"
        }
    }

    #[test]
    fn test_config_wrapper() {
        let mock = MockConfig {
            value: "test".to_string(),
        };
        let config = Config(mock);
        assert_eq!(config.0.value, "test");
    }
}
