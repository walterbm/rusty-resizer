use rusty_resizer::run;
use std::env;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = env::var("PORT").unwrap_or(String::from("8080"));
    let address = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(address).expect(&format!("Failed to bind to port {}", port));
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    run(listener)?.await
}
