extern crate log;

use actix_web::dev::Server;
use actix_web::{error, get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use http::Client;
use img::{Image, ImageError};
use magick_rust::magick_wand_genesis;
use serde::Deserialize;
use std::net::TcpListener;
use std::sync::Once;

mod http;
mod img;

static START: Once = Once::new();

static USER_AGENT: &str = "rusty-resizer/0.1.0";

#[derive(Deserialize)]
struct ResizeOptions {
    source: String,
    height: usize,
    width: usize,
    quality: Option<u8>,
}

// /resize?source=url.jpeg&height=500&width=500&max_quality=85
#[get("/resize")]
async fn resize(options: web::Query<ResizeOptions>) -> Result<HttpResponse, ImageError> {
    let client = Client::new(USER_AGENT);

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

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    START.call_once(|| {
        magick_wand_genesis();
    });

    let server = HttpServer::new(|| {
        App::new()
            .route("/ping", web::get().to(ping))
            .service(resize)
            .wrap(Logger::default())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
