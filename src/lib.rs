#![warn(missing_docs)]

mod http;
mod types;

struct ClientConnectionConfig {
    pub http_url: String
}

struct Client {
    http: http::HttpClient
}
impl Client {
    pub fn new(config: ClientConnectionConfig) -> Self {

        // TODO: Remove unwrap for client failure.
        Self {
            http: http::HttpClient::new(config.http_url.as_str()).unwrap()
        }
    }
}
