use axum_config::config;
use serde::Deserialize;
use thisconfig::Config;

#[config(key = "app")]
#[derive(Clone, Deserialize, Default)]
struct AppConfig {
    name: String,
    debug: bool,
}

fn main() {
    let config = Config::from_path("config.toml").expect("Failed to load config file");

    let app_config = config.get_or_panic::<AppConfig>();
    let _ = config.get_or_default::<AppConfig>();

    println!("App Name: {}", app_config.name);
    println!("Debug Mode: {}", app_config.debug);
}
