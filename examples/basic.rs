use std::sync::Arc;

use axum::{Extension, Router, response::IntoResponse, routing::get};
use axum_config::{Config, ExtractConfig, config};
use serde::Deserialize;
use tokio::net::TcpListener;

#[config(key = "server")]
#[derive(Clone, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
}

async fn app_info(ExtractConfig(config): ExtractConfig<ServerConfig>) -> impl IntoResponse {
    format!("App running on {}:{}", config.host, config.port)
}

#[tokio::main]
async fn main() {
    let app_config = Config::from_path("examples/config.toml").expect("Failed to load config file");

    let server_config = app_config.get_or_panic::<ServerConfig>();

    let app = Router::new()
        .route("/", get(app_info))
        .layer(Extension(Arc::new(app_config)));

    let listener = TcpListener::bind(format!("{}:{}", server_config.host, server_config.port))
        .await
        .expect("Failed to bind to address");

    println!(
        "Server running at http://{}:{}",
        server_config.host, server_config.port
    );

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
