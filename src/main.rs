use rusty_resizer::{run, Configuration};
use std::env;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ENV vars
    let port = env::var("PORT").unwrap_or_else(|_| String::from("8080"));
    let env = env::var("ENV").unwrap_or_else(|_| String::from("local"));
    let allowed_hosts =
        env::var("ALLOWED_HOSTS").unwrap_or_else(|_| panic!("ALLOWED_HOSTS must be set!"));
    let default_quality = env::var("DEFAULT_QUALITY")
        .unwrap_or_else(|_| String::from("85"))
        .parse()
        .unwrap_or(85);
    let cache_expiration: u64 = env::var("CACHE_EXPIRATION_HOURS")
        .unwrap_or_else(|_| String::from("2880"))
        .parse()
        .unwrap_or(2800);
    // App Configuration
    let address = format!("127.0.0.1:{}", port);
    let listener =
        TcpListener::bind(address).unwrap_or_else(|_| panic!("Failed to bind to port {}", port));
    let configuration = Configuration {
        env,
        allowed_hosts,
        cache_expiration,
        default_quality,
    };
    // Logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // Start
    run(listener, configuration)?.await
}
