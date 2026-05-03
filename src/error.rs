//! Error types used by `fyers-rs`.

use thiserror::Error;

/// Convenient crate-local result type.
pub type Result<T> = std::result::Result<T, FyersError>;

/// Error type for client configuration, transport, serialization, and broker responses.
#[derive(Debug, Error)]
pub enum FyersError {
    /// A required client configuration value was not provided.
    #[error("missing required configuration: {field}")]
    MissingConfig {
        /// Name of the missing configuration field.
        field: &'static str,
    },

    /// A URL could not be parsed.
    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),

    /// The underlying HTTP client failed.
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    /// JSON serialization or deserialization failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Fyers returned a documented broker/API error envelope or non-success HTTP status.
    #[error("broker error: HTTP {status}, code {code:?}, s {s:?}, message {message:?}")]
    Broker {
        /// HTTP status returned by the broker.
        status: reqwest::StatusCode,
        /// Standard retry delay header documented for order rate limits, when present.
        retry_after: Option<Box<str>>,
        /// Millisecond retry delay header documented for order rate limits, when present.
        retry_after_ms: Option<Box<str>>,
        /// Broker-specific numeric code, when present.
        code: Option<i64>,
        /// Broker status string, when present.
        s: Option<Box<str>>,
        /// Broker message, when present.
        message: Option<Box<str>>,
        /// Raw response body for diagnostics.
        body: Box<str>,
    },

    /// WebSocket transport failed.
    #[error("WebSocket error: {0}")]
    WebSocket(#[source] Box<tokio_tungstenite::tungstenite::Error>),

    /// Builder or request validation failed before a broker call was made.
    #[error("validation error: {0}")]
    Validation(String),
}

impl From<tokio_tungstenite::tungstenite::Error> for FyersError {
    fn from(error: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocket(Box::new(error))
    }
}
