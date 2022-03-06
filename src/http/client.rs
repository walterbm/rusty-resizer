use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use awc::{Client as ActixWebClient, Connector};
use openssl::ssl::{SslConnector, SslMethod};
use std::fmt::{Display, Formatter, Result as FmtResult};
use url::Url;

static USER_AGENT: &str = "rusty-resizer/0.1.0";
const MAX_ALLOWED_BYTES: usize = 20_000_000;

pub struct Client<'app> {
    client: ActixWebClient,
    user_agent: &'static str,
    allowed_hosts: &'app Vec<String>,
}

impl<'app> Client<'app> {
    pub fn new(allowed_hosts: &'app Vec<String>) -> Self {
        let user_agent = USER_AGENT;
        let ssl_builder = SslConnector::builder(SslMethod::tls()).unwrap();

        let client = ActixWebClient::builder()
            .connector(Connector::new().openssl(ssl_builder.build()))
            .finish();
        Self {
            client,
            user_agent,
            allowed_hosts,
        }
    }

    pub async fn get(&self, url: &str) -> Result<Bytes, ClientError> {
        self.validate_host(url)?;

        let mut request = self
            .client
            .get(url)
            .append_header(("User-Agent", self.user_agent))
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

    fn validate_host(&self, url: &str) -> Result<(), ClientError> {
        let url = Url::parse(url).map_err(|_| ClientError::InvalidRequest)?;

        let host = url.host_str().unwrap_or("invalid host");

        if self.allowed_hosts.contains(&host.to_string()) {
            return Ok(());
        }

        Err(ClientError::BlockedHost)
    }
}

pub enum ClientError {
    InvalidRequest,
    InvalidPayload,
    NotFound,
    BlockedHost,
    InaccessibleImage,
}

impl ClientError {
    fn message(&self) -> &str {
        match self {
            Self::InvalidRequest => "Invalid Request For Image",
            Self::InvalidPayload => "Invalid Image Payload",
            Self::NotFound => "Image Not Found",
            Self::BlockedHost => "Image Host Is Not Allowed",
            Self::InaccessibleImage => "Inaccessible Image",
        }
    }
}

impl Display for ClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message())
    }
}
