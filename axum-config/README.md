# Axum Config

Configuration management for Axum applications. Load configuration from TOML files with environment variable support.

## Features

- Load configuration from TOML files
- Environment variable interpolation (`${VAR}` and `${VAR:default}`)
- Integration with Axum as extractor

## Installation

Note: `thisconfig` is also required for auto generated code by `config` macro. You can add both dependencies with:

```bash
cargo add axum-config thisconfig
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
use axum_config::{Config, ExtractConfig};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = Config::builder()
        .add_file("config/config.toml")
        .build()
        .expect("Failed to load config");

    let app = Router::new()
        .route("/", get(handler))
        .layer(Extension(config));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

// If your handlers are separated into another modules,
// it's recommended to rename `ExtractConfig` to `Config` for clarity.
// use axum_config::ExtractConfig as Config;

async fn handler(
    ExtractConfig(db_config): ExtractConfig<DatabaseConfig>,
) -> impl IntoResponse {
    format!("DB: {}:{}", db_config.host, db_config.port)
}
```

## Configuration Loading

Use `Config::builder()` to specify configuration files. Files can be optional (loaded if present) or required (must exist).

```rust
let config = Config::builder()
    .add_file("config/config.toml")  // optional
    .add_required_file("secrets.toml")  // required
    .build()?;
```

Files are merged in order, with later files overriding earlier ones.

## Environment variables

Supports interpolation in TOML with default values:

```toml
[server]
host = "${HOST:0.0.0.0}"      # Uses HOST or defaults to "0.0.0.0"
port = "${PORT}"              # Requires PORT to be defined
database_url = "${DATABASE_URL}/my_database"  # Requires DATABASE_URL and appends "/my_database"
```

## Config Methods

| Method                | Description                                      |
| --------------------- | ------------------------------------------------ |
| `get<T>()`            | Returns the configuration section as `Option<T>` |
| `get_or_default<T>()` | Returns the config section or default if missing |
| `require<T>()`        | Returns the config section or panics if missing  |

## Extractor Methods

| Extractor                   | Description                                | Feature      |
| --------------------------- | ------------------------------------------ | ------------ |
| `ExtractConfig<T>`          | Extracts config section without validation | -            |
| `ExtractOptionalConfig<T>`  | Extracts config section as `Option<T>`     | -            |
| `ExtractValidatedConfig<T>` | Extracts and validates config section      | `validation` |

> **Note**: Enable the `validation` feature in your `Cargo.toml` for `ExtractValidatedConfig`: `axum-config = { version = "*", features = ["validation"] }`
