use crate::{Config, ConfigError, ConfigItem};
use axum::{extract::FromRequestParts, http::request::Parts};
use std::sync::Arc;

pub struct ExtractConfig<T>(pub T);

impl<S, T> FromRequestParts<S> for ExtractConfig<T>
where
    T: ConfigItem,
    S: Send + Sync,
{
    type Rejection = ConfigError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let config = parts
            .extensions
            .get::<Arc<Config>>()
            .ok_or(ConfigError::ExtensionError)?;

        let item = config
            .get::<T>()
            .ok_or(ConfigError::key_not_found(T::key()))?;

        Ok(ExtractConfig(item))
    }
}

pub struct ExtractOptionalConfig<T>(pub Option<T>);

impl<S, T> FromRequestParts<S> for ExtractOptionalConfig<T>
where
    T: ConfigItem,
    S: Send + Sync,
{
    type Rejection = ConfigError;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let config = parts
            .extensions
            .get::<Arc<Config>>()
            .ok_or(ConfigError::ExtensionError)?;

        let item = config.get::<T>();

        Ok(ExtractOptionalConfig(item))
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

        let config = ExtractConfig(Some(mock.clone()));
        assert_eq!(config.0.as_ref().unwrap().value, "test");
    }
}
