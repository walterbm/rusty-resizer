use cadence::{NopMetricSink, StatsdClient, UdpMetricSink, DEFAULT_PORT};
use rusty_resizer::{run, Configuration};
use std::env;
use std::net::TcpListener;
use std::net::UdpSocket;

const DEFAULT_WORKERS: usize = 4;
const DEFAULT_QUALITY: u8 = 85;
const DEFAULT_CACHE_EXPIRATION_HOURS: u64 = 2880;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ENV vars
    let port = env::var("PORT").unwrap_or_else(|_| String::from("8080"));
    let env = env::var("ENV").unwrap_or_else(|_| String::from("local"));
    let allowed_hosts = env::var("ALLOWED_HOSTS").expect("ALLOWED_HOSTS must be set!");
    let workers = env::var("WORKERS")
        .ok()
        .and_then(|workers| workers.parse::<usize>().ok())
        .unwrap_or(DEFAULT_WORKERS);
    let default_quality = env::var("DEFAULT_QUALITY")
        .ok()
        .and_then(|dq| dq.parse::<u8>().ok())
        .unwrap_or(DEFAULT_QUALITY);
    let cache_expiration = env::var("CACHE_EXPIRATION_HOURS")
        .ok()
        .and_then(|ce| ce.parse::<u64>().ok())
        .unwrap_or(DEFAULT_CACHE_EXPIRATION_HOURS);
    let statsd_host = env::var("STATSD_HOST").ok();
    // App Configuration
    let address = format!("0.0.0.0:{}", port);
    let listener =
        TcpListener::bind(address).unwrap_or_else(|_| panic!("Failed to bind to port {}!", port));
    let configuration = Configuration::new(env, allowed_hosts, cache_expiration, default_quality);
    // Logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // Metrics
    let statsd = match statsd_host {
        Some(statsd_host) => {
            let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind to UDP socket!");
            socket
                .set_nonblocking(true)
                .expect("Failed to set UDP socket as non-blocking!");
            let sink = UdpMetricSink::from((statsd_host, DEFAULT_PORT), socket)
                .expect("Failed to bind to UDP port!");
            StatsdClient::from_sink("rusty.resizer", sink)
        }
        None => (StatsdClient::from_sink("rusty.resizer", NopMetricSink)),
    };
    // Start
    run(listener, configuration, statsd, workers)?.await
}
