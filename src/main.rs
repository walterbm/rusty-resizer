use cadence::{NopMetricSink, StatsdClient, UdpMetricSink, DEFAULT_PORT};
use rusty_resizer::{run, Configuration};
use std::env;
use std::net::TcpListener;
use std::net::UdpSocket;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // ENV vars
    let port = env::var("PORT").unwrap_or( String::from("8080"));
    let env = env::var("ENV").unwrap_or( String::from("local"));
    // I always find it annoying that Result is so strict about the Error type, but its right to be that way. If you want to manage the possible VarError cases differently, you can create a custom wrapper error type and use map_err to have the Result E types coalesce.
    let workers = env::var("WORKERS")
        .ok()
        .and_then(|workers| workers.parse::<usize>().ok())
        .unwrap_or(4);
    let allowed_hosts = env::var("ALLOWED_HOSTS").expect("ALLOWED_HOSTS must be set!");
    let default_quality = env::var("DEFAULT_QUALITY").ok();
    let cache_expiration = env::var("CACHE_EXPIRATION_HOURS").ok();
    let statsd_host = env::var("STATSD_HOST").ok();
    // App Configuration
    let address = format!("0.0.0.0:{}", port);
    let listener =
        TcpListener::bind(address).expect(&format!("Failed to bind to port {}!", port));
    let configuration = Configuration::new(env, allowed_hosts, cache_expiration, default_quality);
    // Logging (Not necessary, but helpful to readers to see a use statement for env_logger so they know "where the module comes from")
    use env_logger;
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
