//! Websocket interface related errors.

/// Client-level errors.
#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    /// HTTP client error
    #[cfg(feature = "http")]
    #[error("{0}")]
    HttpError(#[from] crate::http::error::HttpError),

    /// WebSocket client error
    #[cfg(feature = "websocket")]
    #[error("{0}")]
    WebsocketError(#[from] crate::ws::error::WebsocketError),

    /// Missing/invalid configuration value
    #[error("Missing/invalid required configuration: {0}")]
    ConfigError(&'static str),

    /// No WebSocket client initialized
    #[cfg(feature = "websocket")]
    #[error("No WebSocket client initialized")]
    NoWebsocketClient
}

/// Result type alias for Websocket operations.
pub type ClientResult<T> = Result<T, ClientError>;