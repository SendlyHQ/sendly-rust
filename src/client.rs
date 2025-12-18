use reqwest::{Client, Response, StatusCode};
use std::time::Duration;

use crate::error::{ApiErrorResponse, Error, Result};
use crate::messages::Messages;

/// Default API base URL.
pub const DEFAULT_BASE_URL: &str = "https://sendly.live/api/v1";

/// SDK version.
pub const VERSION: &str = "0.9.5";

/// Configuration for the Sendly client.
#[derive(Debug, Clone)]
pub struct SendlyConfig {
    /// API base URL.
    pub base_url: String,
    /// Request timeout.
    pub timeout: Duration,
    /// Maximum retry attempts.
    pub max_retries: u32,
}

impl Default for SendlyConfig {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}

impl SendlyConfig {
    /// Creates a new configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the base URL.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Sets the timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the max retries.
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }
}

/// Sendly API client.
#[derive(Debug, Clone)]
pub struct Sendly {
    api_key: String,
    config: SendlyConfig,
    client: Client,
}

impl Sendly {
    /// Creates a new Sendly client with default configuration.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your Sendly API key
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// let client = Sendly::new("sk_live_v1_your_api_key");
    /// ```
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_config(api_key, SendlyConfig::default())
    }

    /// Creates a new Sendly client with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your Sendly API key
    /// * `config` - Client configuration
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, SendlyConfig};
    /// use std::time::Duration;
    ///
    /// let config = SendlyConfig::new()
    ///     .timeout(Duration::from_secs(60))
    ///     .max_retries(5);
    ///
    /// let client = Sendly::with_config("sk_live_v1_xxx", config);
    /// ```
    pub fn with_config(api_key: impl Into<String>, config: SendlyConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            api_key: api_key.into(),
            config,
            client,
        }
    }

    /// Returns the Messages resource.
    pub fn messages(&self) -> Messages {
        Messages::new(self)
    }

    /// Makes a GET request.
    pub(crate) async fn get(&self, path: &str, query: &[(String, String)]) -> Result<Response> {
        self.request_with_retry(|| async {
            let url = format!("{}{}", self.config.base_url, path);

            self.client
                .get(&url)
                .query(query)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Accept", "application/json")
                .header("User-Agent", format!("sendly-rs/{}", VERSION))
                .send()
                .await
        })
        .await
    }

    /// Makes a POST request.
    pub(crate) async fn post<T: serde::Serialize>(&self, path: &str, body: &T) -> Result<Response> {
        self.request_with_retry(|| async {
            let url = format!("{}{}", self.config.base_url, path);

            self.client
                .post(&url)
                .json(body)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .header("User-Agent", format!("sendly-rs/{}", VERSION))
                .send()
                .await
        })
        .await
    }

    /// Makes a DELETE request.
    pub(crate) async fn delete(&self, path: &str) -> Result<Response> {
        self.request_with_retry(|| async {
            let url = format!("{}{}", self.config.base_url, path);

            self.client
                .delete(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Accept", "application/json")
                .header("User-Agent", format!("sendly-rs/{}", VERSION))
                .send()
                .await
        })
        .await
    }

    /// Executes a request with retries.
    async fn request_with_retry<F, Fut>(&self, request_fn: F) -> Result<Response>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = std::result::Result<Response, reqwest::Error>>,
    {
        let mut last_error: Option<Error> = None;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                let delay = Duration::from_secs(2u64.pow(attempt - 1));
                tokio::time::sleep(delay).await;
            }

            match request_fn().await {
                Ok(response) => {
                    return self.handle_response(response).await;
                }
                Err(e) => {
                    if e.is_timeout() {
                        last_error = Some(Error::Timeout);
                    } else if e.is_connect() {
                        last_error = Some(Error::Network {
                            message: e.to_string(),
                        });
                    } else {
                        return Err(Error::Http(e));
                    }
                }
            }
        }

        Err(last_error.unwrap_or(Error::Network {
            message: "Request failed after retries".to_string(),
        }))
    }

    /// Handles the response and converts errors.
    async fn handle_response(&self, response: Response) -> Result<Response> {
        let status = response.status();

        if status.is_success() {
            return Ok(response);
        }

        let retry_after = response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok());

        let error_body: ApiErrorResponse = response.json().await.unwrap_or(ApiErrorResponse {
            message: None,
            error: None,
            code: None,
        });

        let message = error_body.message();

        Err(match status {
            StatusCode::UNAUTHORIZED => Error::Authentication { message },
            StatusCode::PAYMENT_REQUIRED => Error::InsufficientCredits { message },
            StatusCode::NOT_FOUND => Error::NotFound { message },
            StatusCode::TOO_MANY_REQUESTS => Error::RateLimit {
                message,
                retry_after,
            },
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY => {
                Error::Validation { message }
            }
            _ => Error::Api {
                message,
                status_code: status.as_u16(),
                code: error_body.code,
            },
        })
    }
}
