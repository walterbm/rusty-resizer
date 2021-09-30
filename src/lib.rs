extern crate log;

use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{error, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use http::Client;
use img::{Image, ImageError};
use magick_rust::magick_wand_genesis;
use serde::Deserialize;
use std::net::TcpListener;
use std::sync::Once;
use std::time::{Duration, SystemTime};

mod http;
mod img;

static START: Once = Once::new();

#[derive(Clone)]
pub struct Configuration {
    pub env: String,
    pub allowed_hosts: String,
    pub cache_expiration: u64,
    pub default_quality: u8,
}

#[derive(Deserialize)]
struct ResizeOptions {
    source: String,
    height: usize,
    width: usize,
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
async fn resize<'app>(
    options: web::Query<ResizeOptions>,
    configuration: web::Data<Configuration>,
) -> Result<HttpResponse, ImageError> {
    let client = Client::new(&configuration.allowed_hosts);

    let response = client.get(&options.source).await;

    match response {
        Ok(response) => {
            let mut image = Image::from_bytes(&response)?;

            image.resize(options.width, options.height);

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

pub fn run(listener: TcpListener, configuration: Configuration) -> Result<Server, std::io::Error> {
    START.call_once(|| {
        magick_wand_genesis();
    });

    let configuration = Data::new(configuration);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/ping", web::get().to(ping))
            .route("/resize", web::get().to(resize))
            .app_data(configuration.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
