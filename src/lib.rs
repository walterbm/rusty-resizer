extern crate log;

use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{
    dev::Service as _, error, middleware::Logger, web, App, HttpResponse, HttpServer, Responder,
};
use cadence::{CountedExt, StatsdClient, Timed};
use futures_util::future::FutureExt;
use http::Client;
use image::ImageFormat;
use img::{deserialize_image_format_external_enum, ImageError, ResizableImage};
use magick_rust::magick_wand_genesis;
use rand::Rng;
use serde::Deserialize;
use std::net::TcpListener;
use std::sync::{Arc, Once};
use std::time::{Duration, Instant, SystemTime};
use url::Host;

mod http;
mod img;

static START: Once = Once::new();

#[derive(Clone)]
pub struct Configuration {
    pub env: String,
    pub allowed_hosts: Vec<Host>,
    pub cache_expiration: u64,
    pub cache_jitter: u64,
    pub default_quality: u8,
}

impl Configuration {
    /// Create a new Configuration with default values and correctly transformed options
    ///
    /// ```rust
    /// # use url::Host;
    /// # use rusty_resizer::Configuration;
    ///
    /// let config = Configuration::new(String::from("test"), String::from("  x.com,  y.com,z.com"), 2880, 60, 50);
    ///
    /// assert_eq!("test", config.env);
    /// assert_eq!(vec![Host::parse("x.com").unwrap(), Host::parse("y.com").unwrap(), Host::parse("z.com").unwrap()], config.allowed_hosts);
    /// assert_eq!(2880, config.cache_expiration);
    /// assert_eq!(60, config.cache_jitter);
    /// assert_eq!(50, config.default_quality);
    /// ```
    pub fn new(
        env: String,
        allowed_hosts: String,
        cache_expiration: u64,
        cache_jitter: u64,
        default_quality: u8,
    ) -> Self {
        let allowed_hosts = allowed_hosts
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .split(',')
            .map(str::to_string)
            .filter_map(|s| Host::parse(&s).ok())
            .collect::<Vec<Host>>();

        Configuration {
            env,
            allowed_hosts,
            cache_expiration,
            cache_jitter,
            default_quality,
        }
    }
}

#[derive(Deserialize)]
struct ResizeOptions {
    source: String,
    height: Option<f32>,
    width: Option<f32>,
    quality: Option<u8>,
    #[serde(default, deserialize_with = "deserialize_image_format_external_enum")]
    format: Option<ImageFormat>,
}

/// Resize an image
///
/// Accepts five query parameters:
///     - source
///     - height
///     - width
///     - quality
///     - format
///
/// Example request:
///  resize?source=url.jpeg&height=500&width=500&max_quality=85&format=webp
///
async fn resize(
    options: web::Query<ResizeOptions>,
    configuration: web::Data<Configuration>,
) -> Result<HttpResponse, ImageError> {
    let client = Client::new(&configuration.allowed_hosts);

    let response = client.get(&options.source).await;

    match response {
        Ok(response) => {
            let mut image = ResizableImage::from_bytes(&response)?;

            image.resize(
                options.width.map(|f| f.round() as usize),
                options.height.map(|f| f.round() as usize),
            );

            let buffer = image.to_buffer_mut(
                options.quality.unwrap_or(configuration.default_quality),
                options.format.unwrap_or(image.format()?),
            )?;

            let content_type = image.mime_type()?;

            let now = SystemTime::now();
            let jitter = if configuration.cache_jitter > 0 {
                rand::thread_rng().gen_range(0..configuration.cache_jitter)
            } else {
                0
            };
            let expire_time_in_seconds = configuration.cache_expiration * 60 * 60 + jitter;

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
    START.call_once(|| {
        magick_wand_genesis();
    });

    let configuration = Data::new(configuration);

    let statsd = Arc::new(statsd);

    let server = HttpServer::new(move || {
        let statsd = statsd.clone();
        App::new()
            .wrap_fn(move |req, srv| {
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
