# Axum Config

Configuration management for Axum applications. Load configuration from TOML files with environment variable support.

## Features

- Load configuration from TOML files
- Environment variable interpolation (`${VAR}` and `${VAR:default}`)
- Integration with Axum as extractor

## Installation

```bash
cargo add axum-config
```

## Usage

### 1. Define configuration structure

```rust
use serde::Deserialize;
use axum_config::config;

#[config(key = "database")]
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
}
```

### 2. Create configuration file

`config/config.toml`:

```toml
[database]
host = "${DB_HOST:localhost}"
port = 5432
user = "${DB_USER:admin}"
```

### 3. Use in your application

```rust
use axum::{Router, routing::get, Extension};
use axum_config::{ApplicationConfig, Config};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = ApplicationConfig::new().expect("Failed to load config");

    let app = Router::new()
        .route("/", get(handler))
        .layer(Extension(Arc::new(config)));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

async fn handler(
    Config(db_config): Config<DatabaseConfig>,
) -> String {
    format!("DB: {}:{}", db_config.host, db_config.port)
}
```

## Configuration priority

1. Explicit path: `ApplicationConfig::from_path("path/to/config.toml")`
2. Environment variable: `CONFIG_FILE_PATH`
3. Default path: `config/config.toml`
4. Executable directory: `<exe_dir>/config/config.toml`

## Environment variables

Supports interpolation in TOML with default values:

```toml
[server]
host = "${HOST:0.0.0.0}"      # Uses HOST or defaults to "0.0.0.0"
port = "${PORT}"              # Requires PORT to be defined
```

## Available methods

```rust
let config = ApplicationConfig::new()?;

// Get configuration
let db: DatabaseConfig = config.get()?;

// With default if missing
let db: DatabaseConfig = config.get_or_default();

// Panic if missing (not recommended)
let db: DatabaseConfig = config.get_or_panic();
```
