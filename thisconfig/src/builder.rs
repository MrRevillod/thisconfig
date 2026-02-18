use crate::{env::expand_env_variables, Config, ConfigError};
use std::{fs, path::PathBuf, sync::Arc};
use toml::Table;
use tracing::{error, warn};

#[derive(Debug)]
enum Source {
    File { path: PathBuf, required: bool },
    TomlString { content: String },
}

#[derive(Debug, Default)]
pub struct ConfigBuilder {
    sources: Vec<Source>,
}

impl ConfigBuilder {
    pub fn add_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.sources.push(Source::File {
            path: path.into(),
            required: false,
        });

        self
    }

    pub fn add_required_file<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.sources.push(Source::File {
            path: path.into(),
            required: true,
        });

        self
    }

    pub fn add_toml_str(mut self, toml: &str) -> Self {
        self.sources.push(Source::TomlString {
            content: toml.to_string(),
        });

        self
    }

    fn load(sources: Vec<Source>) -> Result<Config, ConfigError> {
        let mut merged = Table::new();

        for source in sources {
            match source {
                Source::File { path, required } => {
                    if path.exists() {
                        let content = fs::read_to_string(&path)?;
                        let expanded = expand_env_variables(&content)
                            .map_err(ConfigError::interpolation_error)?;

                        let table: Table = toml::from_str::<Table>(&expanded).inspect_err(|e| {
                            error!("Failed to parse TOML from {}: {}", path.display(), e);
                        })?;

                        Self::merge_tables(&mut merged, table);
                    } else if required {
                        error!("Config file not found (required): {}", path.display());

                        return Err(ConfigError::FileNotFound(
                            path.to_str().unwrap_or_default().to_string(),
                        ));
                    } else {
                        warn!("Config file not found (optional): {}", path.display());
                    }
                }
                Source::TomlString { content } => {
                    let expanded =
                        expand_env_variables(&content).map_err(ConfigError::interpolation_error)?;

                    let table: Table = toml::from_str::<Table>(&expanded).inspect_err(|e| {
                        error!("Failed to parse TOML string: {}", e);
                    })?;

                    Self::merge_tables(&mut merged, table);
                }
            }
        }

        Ok(Config {
            inner: Arc::new(merged),
        })
    }

    fn merge_tables(base: &mut Table, other: Table) {
        for (key, value) in other {
            match base.get_mut(&key) {
                Some(existing)
                    if matches!(existing, toml::Value::Table(_))
                        && matches!(value, toml::Value::Table(_)) =>
                {
                    if let (toml::Value::Table(base_table), toml::Value::Table(other_table)) =
                        (existing, value)
                    {
                        Self::merge_tables(base_table, other_table);
                    }
                }
                _ => {
                    base.insert(key, value);
                }
            }
        }
    }

    /// Builds the configuration from added sources.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if no sources, files missing, or parsing fails.
    pub fn build(self) -> Result<Config, ConfigError> {
        if self.sources.is_empty() {
            return Err(ConfigError::NoSourcesConfigured);
        }

        Self::load(self.sources)
    }
}
