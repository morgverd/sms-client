//! A client library for SMS-API, via HTTP and an optional websocket connection.
//! https://github.com/morgverd/sms-api

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic)]

use std::sync::Arc;

pub mod http;

/// Connection configuration
pub struct ClientConnectionConfig {
    /// HTTP base URL. eg: http://192.168.1.2:3000
    pub http_url: String,

    /// Optional HTTP authorization header token.
    pub http_auth: Option<String>
}

/// SMS Client
#[derive(Clone)]
pub struct Client {
    http: Arc<http::HttpClient>
}
impl Client {

    /// Create an SMS client with a connection config.
    pub fn new(config: ClientConnectionConfig) -> Self {
        // TODO: Don't expect here, make this return a Result with an AppError?
        let http = http::HttpClient::new(config.http_url, config.http_auth).expect("HttpClient failed");
        Self {
            http: Arc::new(http)
        }
    }

    /// Borrow the inner HTTP client.
    pub fn http(&self) -> &http::HttpClient {
        &self.http
    }
}