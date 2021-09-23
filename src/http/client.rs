use actix_web::client::{Client as ActixWebClient, Connector};
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use openssl::ssl::{SslConnector, SslMethod};
use std::fmt::{Display, Formatter, Result as FmtResult};

const MAX_ALLOWED_BYTES: usize = 20_000_000;

pub struct Client {
    client: ActixWebClient,
    user_agent: &'static str,
}

impl Client {
    pub fn new(user_agent: &'static str) -> Self {
        let ssl_builder = SslConnector::builder(SslMethod::tls()).unwrap();

        let client = ActixWebClient::builder()
            .connector(Connector::new().ssl(ssl_builder.build()).finish())
            .finish();
        return Self { client, user_agent };
    }

    pub async fn get(&self, url: &str) -> Result<Bytes, ClientError> {
        let mut request = self
            .client
            .get(url)
            .header("User-Agent", self.user_agent)
            .send()
            .await
            .map_err(|_| ClientError::InvalidRequest)?;

        match request.status() {
            StatusCode::OK => request
                .body()
                .limit(MAX_ALLOWED_BYTES)
                .await
                .map_err(|_| ClientError::InvalidPayload),
            StatusCode::NOT_FOUND => Err(ClientError::NotFound),
            StatusCode::FORBIDDEN => Err(ClientError::InaccessibleImage),
            _ => Err(ClientError::InvalidRequest),
        }
    }
}

pub enum ClientError {
    InvalidRequest,
    InvalidPayload,
    NotFound,
    InaccessibleImage,
}

impl ClientError {
    fn message(&self) -> &str {
        match self {
            Self::InvalidRequest => "Invalid Request For Image",
            Self::InvalidPayload => "Invalid Image Payload",
            Self::NotFound => "Image Not Found",
            Self::InaccessibleImage => "Inaccessible Image",
        }
    }
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}
