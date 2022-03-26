use std::net::TcpListener;

use cadence::{NopMetricSink, StatsdClient};
use rusty_resizer::Configuration;

pub fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random system port");
    let port = listener.local_addr().unwrap().port();
    let configuration = Configuration {
        env: String::from("test"),
        allowed_hosts: vec![String::from("raw.githubusercontent.com")],
        cache_expiration: 1,
        default_quality: 85,
    };
    let statsd = StatsdClient::from_sink("rusty.resizer", NopMetricSink);
    let workers = 1;
    let server = rusty_resizer::run(listener, configuration, statsd, workers)
        .expect("Failed to bind address");

    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
