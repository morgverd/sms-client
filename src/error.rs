//! Websocket interface related errors.

/// Client-level errors.
#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    /// HTTP client error
    #[error("HTTP client error: {0}")]
    HttpError(#[from] crate::http::error::HttpError),

    /// WebSocket client error
    #[error("WebSocket client error: {0}")]
    WebsocketError(#[from] crate::ws::error::WebsocketError),

    /// No WebSocket URL configured
    #[error("No WebSocket URL configured")]
    MissingConfiguration,

    /// No WebSocket client initialized
    #[error("No WebSocket client initialized")]
    NoWebsocketClient,
}

/// Result type alias for Websocket operations.
pub type ClientResult<T> = Result<T, ClientError>;