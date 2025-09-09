use thiserror::Error;

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("Network request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Invalid URL: {0}")]
    UrlParseError(#[from] url::ParseError),

    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("HTTP {status}: {message}")]
    HttpStatus { status: u16, message: String },

    #[error("API responded with success=false: {message}")]
    ApiError { message: String },

    #[error("Missing 'response' field in API response")]
    MissingResponseField,

    #[error("Missing 'type' field in API response")]
    MissingTypeField,

    #[error("Missing 'data' field in API response")]
    MissingDataField,

    #[error("Type mismatch: expected '{expected}', got '{actual}'")]
    ResponseTypeMismatch { expected: String, actual: String }
}

pub type HttpResult<T> = Result<T, HttpError>;