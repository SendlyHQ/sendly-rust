use thiserror::Error;

/// Result type for Sendly operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the Sendly SDK.
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid or missing API key.
    #[error("Authentication failed: {message}")]
    Authentication { message: String },

    /// Rate limit exceeded.
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        message: String,
        /// Seconds to wait before retrying.
        retry_after: Option<u64>,
    },

    /// Insufficient credits in account.
    #[error("Insufficient credits: {message}")]
    InsufficientCredits { message: String },

    /// Invalid request parameters.
    #[error("Validation error: {message}")]
    Validation { message: String },

    /// Requested resource not found.
    #[error("Not found: {message}")]
    NotFound { message: String },

    /// Network error.
    #[error("Network error: {message}")]
    Network { message: String },

    /// Request timeout.
    #[error("Request timed out")]
    Timeout,

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP client error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Generic API error.
    #[error("API error ({status_code}): {message}")]
    Api {
        message: String,
        status_code: u16,
        code: Option<String>,
    },
}

impl Error {
    /// Returns true if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::RateLimit { .. } | Error::Network { .. } | Error::Timeout
        )
    }

    /// Returns the retry-after duration in seconds, if applicable.
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Error::RateLimit { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}

/// API error response from the server.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct ApiErrorResponse {
    pub message: Option<String>,
    pub error: Option<String>,
    pub code: Option<String>,
}

impl ApiErrorResponse {
    pub fn message(&self) -> String {
        self.message
            .clone()
            .or_else(|| self.error.clone())
            .unwrap_or_else(|| "Unknown error".to_string())
    }
}
