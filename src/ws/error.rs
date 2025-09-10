//! WebSocket-related error types.

/// Errors that can occur with WebSocket operations.
#[derive(thiserror::Error, Debug)]
pub enum WebsocketError {

    /// Invalid configured websocket connection URL, failed to create request
    #[error("Invalid WebSocket request URL configured")]
    InvalidRequest,

    /// WebSocket connection error
    #[error("WebSocket connection failed: {0}")]
    ConnectionError(#[from] tokio_tungstenite::tungstenite::Error),

    /// Failed to parse URL
    #[error("Invalid WebSocket URL: {0}")]
    UrlParseError(#[from] http::uri::InvalidUri),

    /// HTTP error when establishing WebSocket connection
    #[error("HTTP error: {0}")]
    HttpError(#[from] http::Error),

    /// HTTP authorization header value failure
    #[error("Invalid WebSocket header value: {0}")]
    InvalidHeader(#[from] http::header::InvalidHeaderValue),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Http Unauthorized (401), token is missing or invalid
    #[error("The WebSocket connection was unauthorized")]
    Unauthorized,

    /// Already connected
    #[error("WebSocket is already connected")]
    AlreadyConnected,

    /// Not connected
    #[error("WebSocket is not connected")]
    NotConnected,

    /// Failed to send message
    #[error("Failed to send message to WebSocket")]
    SendError,

    /// Channel communication error
    #[error("Internal channel communication error")]
    ChannelError,

    /// Timeout error
    #[error("Operation timed out")]
    Timeout
}

/// Result type alias for WebSocket operations.
pub type WebsocketResult<T> = Result<T, WebsocketError>;