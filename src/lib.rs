extern crate log;

use actix_http::header;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::HttpRequest;
use actix_web::{error, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use cadence::StatsdClient;
use http::middleware::statsd::StatsD;
use http::Client;
use image::ImageFormat;
use img::{ImageError, ResizableImage, ResizeImageFormat};
use magick_rust::magick_wand_genesis;
use rand::Rng;
use serde::Deserialize;
use std::collections::HashSet;
use std::net::TcpListener;
use std::sync::{Arc, Once};
use std::time::{Duration, SystemTime};
use url::Host;

mod http;
mod img;

static START: Once = Once::new();
const ACCEPTS_WEBP_HEADER: &[u8; 10] = b"image/webp";

#[derive(Clone)]
pub struct Configuration {
    pub env: String,
    pub allowed_hosts: HashSet<Host>,
    pub cache_expiration: u64,
    pub cache_jitter: u64,
    pub default_quality: u8,
}

impl Configuration {
    /// Create a new Configuration with default values and correctly transformed options
    ///
    /// ```rust
    /// # use url::Host;
    /// # use std::collections::HashSet;
    /// # use rusty_resizer::Configuration;
    ///
    /// let config = Configuration::new(String::from("test"), String::from("  x.com,  y.com,z.com"), 2880, 60, 50);
    ///
    /// assert_eq!("test", config.env);
    /// assert_eq!(HashSet::from_iter(vec![Host::parse("x.com").unwrap(), Host::parse("y.com").unwrap(), Host::parse("z.com").unwrap()]), config.allowed_hosts);
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
            .collect::<HashSet<Host>>();

        Configuration {
            env,
            allowed_hosts,
            cache_expiration,
            cache_jitter,
            default_quality,
        }
    }
}

fn supports_webp(request: &HttpRequest) -> bool {
    match request.headers().get(header::ACCEPT) {
        Some(accept_header) => accept_header
            .as_bytes()
            .windows(ACCEPTS_WEBP_HEADER.len())
            .any(move |sub_slice| sub_slice == ACCEPTS_WEBP_HEADER),
        None => false,
    }
}

#[derive(Deserialize)]
struct ResizeOptions {
    source: String,
    height: Option<f32>,
    width: Option<f32>,
    quality: Option<u8>,
    format: Option<ResizeImageFormat>,
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
    request: HttpRequest,
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

            let format = options.format.and_then(|request_format| {
                // If automatic content negotiation is enabled
                // attempt to convert to WebP when that is supported by incoming request
                if request_format == ResizeImageFormat::Auto && supports_webp(&request) {
                    Some(ImageFormat::WebP)
                } else {
                    request_format.into()
                }
            });

            let buffer = image.to_buffer_mut(
                options.quality.unwrap_or(configuration.default_quality),
                format.unwrap_or(image.format()?),
            )?;

            let content_type = image.mime_type()?;

            let now = SystemTime::now();
            let jitter = rand::thread_rng().gen_range(0..=configuration.cache_jitter);
            let expire_time_in_seconds = configuration.cache_expiration * 60 * 60 + jitter;

            let mut builder = HttpResponse::Ok();

            builder
                .content_type(content_type)
                .insert_header((header::LAST_MODIFIED, httpdate::fmt_http_date(now)))
                .insert_header((
                    header::CACHE_CONTROL,
                    format!("max-age={}", expire_time_in_seconds),
                ))
                .insert_header((
                    header::EXPIRES,
                    httpdate::fmt_http_date(now + Duration::from_secs(expire_time_in_seconds)),
                ));

            if options.format == Some(ResizeImageFormat::Auto) {
                builder.insert_header((header::VARY, "Accept"));
            }

            let response = builder.body(buffer);

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
        App::new()
            .wrap(StatsD::new(statsd.clone()).exclude("/ping"))
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
