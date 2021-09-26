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

mod http;
mod img;

static START: Once = Once::new();

#[derive(Clone)]
pub struct Configuration {
    pub env: String,
    pub allowed_hosts: String,
}

#[derive(Deserialize)]
struct ResizeOptions {
    source: String,
    height: usize,
    width: usize,
    quality: Option<u8>,
}

// /resize?source=url.jpeg&height=500&width=500&max_quality=85
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

            let buffer = image.to_buffer(options.quality.unwrap_or(85))?;

            let content_type = image.mime_type()?;

            return Ok(HttpResponse::Ok().content_type(content_type).body(buffer));
        }
        Err(err) => Ok(HttpResponse::BadRequest().body(err.to_string())),
    }
}

impl error::ResponseError for ImageError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().body(self.to_string())
    }
}

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
