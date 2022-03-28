use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{
    dev::Service as _, error, middleware::Logger, web, App, HttpResponse, HttpServer, Responder,
};
use cadence::{CountedExt, StatsdClient, Timed};
use futures_util::future::FutureExt;
use http::Client;
use img::{Image, ImageError};
use magick_rust::magick_wand_genesis;
use serde::Deserialize;
use std::net::TcpListener;
use std::sync::{Arc, Once};
use std::time::{Duration, Instant, SystemTime};

mod http;
mod img;

static START: Once = Once::new();

#[derive(Clone)]
pub struct Configuration {
    pub env: String,
    pub allowed_hosts: Vec<String>,
    pub cache_expiration: u64,
    pub default_quality: u8,
}

impl Configuration {
    /// Create a new Configuration with default values and correctly transformed options
    ///
    /// ```rust
    /// # use rusty_resizer::Configuration;
    ///
    /// let config = Configuration::new(String::from("test"), String::from("  x.com,  y.com,z.com"), None, Some(String::from("50")));
    ///
    /// assert_eq!("test", config.env);
    /// assert_eq!(vec!["x.com","y.com","z.com"], config.allowed_hosts);
    /// assert_eq!(2880, config.cache_expiration);
    /// assert_eq!(50, config.default_quality);
    /// ```
    pub fn new(
        env: String,
        allowed_hosts: String,
        cache_expiration: Option<String>,
        default_quality: Option<String>,
    ) -> Self {
        // I see two collect calls in one chain and that sets off alarms on efficiency.
        // Since this is not a critical part of the code for optimization, I wouldn't worry.
        let allowed_hosts = allowed_hosts
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .split(',')
            .map(str::to_string)
            .collect::<Vec<String>>();

        // I noticed you use a different approach with cache expiration and default quality.
        // You take in an option and do the defaulting in the run function instead of like
        // the others where you do the defaulting in main and send a non-Option value as
        // the arguments to run. Up to you, but I think the approach of doing the optional
        // unwrapping and defaulting in main makes the intent more obvious and erases the
        // Option wrapper type from the run function signature.
        let cache_expiration: u64 = cache_expiration
            .and_then(|ce| ce.parse::<u64>().ok())
            // Did you mean to provide both 2800 and 2880 or just one?
            .unwrap_or(2800u64);

        let default_quality = default_quality
            .and_then(|dq| dq.parse::<u8>().ok())
            .unwrap_or(85u8);

        Configuration {
            env,
            allowed_hosts,
            cache_expiration,
            default_quality,
        }
    }
}

#[derive(Deserialize)]
struct ResizeOptions {
    // This might be better typed as a more concrete type instead of any string.
    // It's the source for client - what are client's type constraints? If the String
    // is an invalid URI wouldn't this fail at runtime?
    source: String,
    height: Option<f32>,
    width: Option<f32>,
    quality: Option<u8>,
}

/// Resize an image
///
/// Accepts four query parameters:
///     - source
///     - height
///     - width
///     - quality
///
/// Example request:
///  resize?source=url.jpeg&height=500&width=500&max_quality=85
///
async fn resize(
    options: web::Query<ResizeOptions>,
    configuration: web::Data<Configuration>,
) -> Result<HttpResponse, ImageError> {
    let client = Client::new(&configuration.allowed_hosts);

    let response = client.get(&options.source).await;

    match response {
        Ok(response) => {
            let mut image = Image::from_bytes(&response)?;

            image.resize(
                options.width.map(|f| f.round() as usize),
                options.height.map(|f| f.round() as usize),
            );

            let buffer =
                image.to_buffer_mut(options.quality.unwrap_or(configuration.default_quality))?;

            let content_type = image.mime_type()?;

            let now = SystemTime::now();
            let expire_time_in_seconds = configuration.cache_expiration * 60 * 60;

            let response = HttpResponse::Ok()
                .content_type(content_type)
                .append_header(("Last-Modified", httpdate::fmt_http_date(now)))
                .append_header((
                    "Cache-Control",
                    format!("max-age={}", expire_time_in_seconds),
                ))
                .append_header((
                    "Expires",
                    httpdate::fmt_http_date(now + Duration::from_secs(expire_time_in_seconds)),
                ))
                .body(buffer);

            Ok(response)
        }
        Err(err) => Ok(HttpResponse::BadRequest().body(err.to_string())),
    }
}

impl error::ResponseError for ImageError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().body(self.to_string())
    }
}

/// Health check
pub async fn ping() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

pub fn run(
    listener: TcpListener,
    configuration: Configuration,
    statsd: StatsdClient,
    workers: usize,
) -> Result<Server, std::io::Error> {
    // Curious why you needed to use a Once synchronization for initialization
    START.call_once(|| {
        magick_wand_genesis();
    });

    let configuration = Data::new(configuration);

    // I don't understand why moving this value into the closure without being
    // clonable is a problem since you have no intent to use this reference
    // again. It is correct, but I don't understand why.
    // Did you ever learn why you had to use an Arc here? I see you keep moving the statsd value.
    // I wonder if this API is only intended for values that could be taken ownership of? If so,
    // maybe the intent is to provide a client like statsd through that interface? Like app_data?
    let statsd = Arc::new(statsd);
    let server = HttpServer::new(move || {
        let statsd = statsd.clone();
        App::new()
            .wrap_fn(move|req, srv| {
                let statsd = statsd.clone();
                let now = Instant::now();

                srv.call(req).map(move |res| {
                    match &res {
                        Ok(res) => {
                            if res.request().path() == "/resize" {
                                statsd
                                    .time_with_tags("resize", now.elapsed())
                                    .with_tag("status", res.response().status().as_str())
                                    .try_send()
                                    .ok();
                            }
                        }
                        Err(_) => {
                            statsd.incr("unknown.error").ok();
                        }
                    }

                    res
                })
            })
            .wrap(Logger::default().exclude("/ping"))
            .route("/ping", web::get().to(ping))
            .route("/resize", web::get().to(resize))
            .app_data(configuration.clone())
    })
    .listen(listener)?
    .workers(workers)
    .run();

    Ok(server)
}
