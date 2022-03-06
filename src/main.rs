use rusty_resizer::{run, Configuration};
use std::env;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ENV vars
    let port = env::var("PORT").unwrap_or_else(|_| String::from("8080"));
    let env = env::var("ENV").unwrap_or_else(|_| String::from("local"));
    let allowed_hosts = env::var("ALLOWED_HOSTS").expect("ALLOWED_HOSTS must be set!");
    let default_quality = env::var("DEFAULT_QUALITY").ok();
    let cache_expiration = env::var("CACHE_EXPIRATION_HOURS").ok();
    // App Configuration
    let address = format!("127.0.0.1:{}", port);
    let listener =
        TcpListener::bind(address).unwrap_or_else(|_| panic!("Failed to bind to port {}", port));
    let configuration = Configuration::new(env, allowed_hosts, cache_expiration, default_quality);
    // Logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // Start
    run(listener, configuration)?.await
}
