//! HTTP interface related errors.

/// An error originating from the SMS HttpClient.
#[derive(thiserror::Error, Debug)]
pub enum HttpError {

    /// Network request failed (connection issues, timeouts, etc.)
    #[error("Network request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    /// Failed to parse the provided URL.
    #[error("Invalid URL: {0}")]
    UrlParseError(#[from] url::ParseError),

    /// Failed to parse JSON response from the API.
    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),

    /// HTTP request returned a non-success status code.
    #[error("HTTP {status}: {message}")]
    HttpStatus {
        /// HTTP status returned in response.
        status: u16,
        /// Full response body as text.
        message: String
    },

    /// API returned success=false with an error message.
    #[error("API responded with success=false: {message}")]
    ApiError {
        /// The error_message key from response.
        message: String
    },

    /// API response missing the expected 'response' field.
    #[error("Missing 'response' field in API response")]
    MissingResponseField,

    /// Modem response missing the expected 'type' field.
    #[error("Missing 'type' field in API response")]
    MissingTypeField,

    /// Modem response missing the expected 'data' field.
    #[error("Missing 'data' field in API response")]
    MissingDataField,

    /// Modem response type doesn't match what was expected.
    #[error("Type mismatch: expected '{expected}', got '{actual}'")]
    ResponseTypeMismatch {
        /// The expected response data-type.
        expected: String,
        /// The actual response data-type.
        actual: String
    }
}

/// Result type alias for HTTP operations.
pub type HttpResult<T> = Result<T, HttpError>;