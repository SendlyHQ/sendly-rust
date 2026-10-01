use thiserror::Error;

/// Result type for Sendly operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the Sendly SDK.
///
/// An error the API returned carries its machine-readable `code` (such as
/// `invalid_code`, `max_attempts_exceeded` or `rate_limit_exceeded`) and the
/// JSON `body` it sent; read them with [`Error::code`] and [`Error::body`]
/// whatever the variant. An error raised before a request is sent, such as a
/// client-side [`Error::Validation`], has neither.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Invalid or missing API key.
    #[error("Authentication failed: {message}")]
    Authentication {
        message: String,
        /// The API's machine-readable error code, when it sent one.
        code: Option<String>,
        /// The JSON body the API answered with.
        body: Option<serde_json::Value>,
    },

    /// Rate limit exceeded.
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        message: String,
        /// Seconds to wait before retrying: the `Retry-After` header, else the
        /// `retryAfter` the API put in the body.
        retry_after: Option<u64>,
        /// The API's machine-readable error code, such as
        /// `rate_limit_exceeded`, `provision_rate_limit`,
        /// `too_many_concurrent_verifications`, `too_many_failed_key_attempts`,
        /// `max_attempts_exceeded`, `quota_exceeded`, `daily_call_limit`,
        /// `hold_limit_reached`, `whatsapp_signup_limit_reached` or
        /// `whatsapp_verification_resend_too_soon`. The list is not
        /// exhaustive.
        code: Option<String>,
        /// The JSON body the API answered with.
        body: Option<serde_json::Value>,
    },

    /// Insufficient credits in account.
    #[error("Insufficient credits: {message}")]
    InsufficientCredits {
        message: String,
        /// The API's machine-readable error code, when it sent one.
        code: Option<String>,
        /// The JSON body the API answered with.
        body: Option<serde_json::Value>,
    },

    /// Invalid request parameters.
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        /// The API's machine-readable error code, such as `invalid_code`;
        /// `None` when the SDK rejected the input before sending it.
        code: Option<String>,
        /// The JSON body the API answered with; `None` when the SDK rejected
        /// the input before sending it.
        body: Option<serde_json::Value>,
    },

    /// Requested resource not found.
    #[error("Not found: {message}")]
    NotFound {
        message: String,
        /// The API's machine-readable error code, when it sent one.
        code: Option<String>,
        /// The JSON body the API answered with.
        body: Option<serde_json::Value>,
    },

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
        /// The JSON body the API answered with.
        body: Option<serde_json::Value>,
    },
}

impl Error {
    pub(crate) fn validation(message: impl Into<String>) -> Self {
        Error::Validation {
            message: message.into(),
            code: None,
            body: None,
        }
    }

    /// Returns true if this error is retryable.
    ///
    /// Network errors, timeouts and rate limits are, except a rate limit whose
    /// code says waiting cannot clear it: `too_many_failed_key_attempts` (the
    /// API key is wrong), `max_attempts_exceeded` (the verification has
    /// failed), `quota_exceeded` (the workspace's monthly message quota is
    /// used up), `daily_call_limit` (today's calling limit is reached),
    /// `hold_limit_reached` (too many numbers are on hold) and
    /// `whatsapp_signup_limit_reached`. Check
    /// [`retry_after`](Self::retry_after) before waiting: the
    /// one-time-password limits answer with up to a day.
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::RateLimit { code, .. } => !matches!(
                code.as_deref(),
                Some(
                    "too_many_failed_key_attempts"
                        | "max_attempts_exceeded"
                        | "quota_exceeded"
                        | "daily_call_limit"
                        | "hold_limit_reached"
                        | "whatsapp_signup_limit_reached"
                )
            ),
            Error::Network { .. } | Error::Timeout => true,
            _ => false,
        }
    }

    /// Returns the retry-after duration in seconds, if applicable.
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Error::RateLimit { retry_after, .. } => *retry_after,
            _ => None,
        }
    }

    /// Returns the API's machine-readable error code, such as
    /// `invalid_code` or `rate_limit_exceeded`, or `None` when the API sent
    /// none or the error did not come from the API.
    pub fn code(&self) -> Option<&str> {
        match self {
            Error::Authentication { code, .. }
            | Error::RateLimit { code, .. }
            | Error::InsufficientCredits { code, .. }
            | Error::Validation { code, .. }
            | Error::NotFound { code, .. }
            | Error::Api { code, .. } => code.as_deref(),
            _ => None,
        }
    }

    /// Returns the JSON body the API answered with, for fields an error
    /// carries beyond its message and code.
    pub fn body(&self) -> Option<&serde_json::Value> {
        match self {
            Error::Authentication { body, .. }
            | Error::RateLimit { body, .. }
            | Error::InsufficientCredits { body, .. }
            | Error::Validation { body, .. }
            | Error::NotFound { body, .. }
            | Error::Api { body, .. } => body.as_ref(),
            _ => None,
        }
    }

    /// Returns how many attempts a verification has left after a wrong code.
    ///
    /// [`VerifyResource::check`](crate::VerifyResource::check) reports this
    /// only on its 400 `invalid_code` error, never on a successful check.
    /// [`WhatsAppSignupResource::verify`](crate::WhatsAppSignupResource::verify)
    /// reports it on its 422 `whatsapp_verification_code_invalid` error.
    pub fn remaining_attempts(&self) -> Option<i32> {
        let body = self.body()?;
        body.get("remaining_attempts")
            .or_else(|| body.get("attemptsRemaining"))?
            .as_i64()
            .and_then(|n| i32::try_from(n).ok())
    }
}

/// API error response from the server.
#[derive(Debug, Default, serde::Deserialize)]
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

    /// The machine-readable code: an explicit `code`, else the `error`
    /// field when the body also carries a human `message` (the
    /// `{ error, message }` shape).
    pub fn code(&self) -> Option<String> {
        self.code
            .clone()
            .or_else(|| match (&self.error, &self.message) {
                (Some(error), Some(_)) => Some(error.clone()),
                _ => None,
            })
    }
}
