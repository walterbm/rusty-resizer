extern crate log;

use actix_web::{error, get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use http::Client;
use img::{Image, ImageError};
use magick_rust::magick_wand_genesis;
use serde::Deserialize;
use std::env;
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

#[get("/ping")]
pub async fn ping() -> impl Responder {
    HttpResponse::Ok().body("pong")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    START.call_once(|| {
        magick_wand_genesis();
    });

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let port = env::var("PORT").unwrap_or(String::from("8080"));

    HttpServer::new(|| {
        App::new()
            .service(ping)
            .service(resize)
            .wrap(Logger::default())
    })
    .bind(format!("127.0.0.1:{}", port))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{body::Body, test, App};

    #[actix_rt::test]
    async fn test_ping() {
        let mut app = test::init_service(App::new().service(ping)).await;

        let req = test::TestRequest::get().uri("/ping").to_request();
        let mut resp = test::call_service(&mut app, req).await;

        let body = resp.take_body();
        let body = body.as_ref().unwrap();

        assert!(resp.status().is_success());
        assert_eq!(&Body::from("pong"), body);
    }
}
