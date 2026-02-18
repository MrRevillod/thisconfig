use serde::Deserialize;
use thisconfig::{Config, config};

#[config(key = "app")]
#[derive(Clone, Deserialize, Default)]
struct AppConfig {
    name: String,
    debug: bool,
}

fn main() {
    dotenv::from_filename(".env").ok();

    let config = Config::builder()
        .add_file("nonexistent.toml")
        .add_required_file("config.toml")
        .add_toml_str("[extra]\nversion = \"1.0\"")
        .build()
        .expect("Failed to load config file");

    let app_config = config.expect::<AppConfig>();
    let _ = config.get_or_default::<AppConfig>();

    println!("App Name: {}", app_config.name);
    println!("Debug Mode: {}", app_config.debug);
}
