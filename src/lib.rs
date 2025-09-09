#![warn(missing_docs)]

pub mod http;

pub struct ClientConnectionConfig {
    pub http_url: String,
    pub http_auth: Option<String>
}

pub struct Client {
    http: http::HttpClient
}
impl Client {
    pub fn new(config: ClientConnectionConfig) -> Self {
        // TODO: Don't expect here, make this return a Result with an AppError?
        let http = http::HttpClient::new(config.http_url, config.http_auth).expect("HttpClient failed");
        Self {
            http
        }
    }

    pub fn borrow_http(&self) -> &http::HttpClient {
        &self.http
    }
}