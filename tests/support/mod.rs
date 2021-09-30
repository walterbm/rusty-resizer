use std::net::TcpListener;

use rusty_resizer::Configuration;

pub fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random system port");
    let port = listener.local_addr().unwrap().port();
    let configuration = Configuration {
        env: String::from("test"),
        allowed_hosts: String::from("raw.githubusercontent.com"),
        cache_expiration: 1,
        default_quality: 85,
    };
    let server = rusty_resizer::run(listener, configuration).expect("Failed to bind address");

    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
