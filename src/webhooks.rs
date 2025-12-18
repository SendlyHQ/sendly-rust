//! Sendly Webhook Helpers
//!
//! Utilities for verifying and parsing webhook events from Sendly.
//!
//! # Example
//!
//! ```rust
//! use sendly::webhooks::{Webhooks, WebhookEvent};
//!
//! // In your webhook handler (e.g., Actix-web)
//! async fn handle_webhook(
//!     body: String,
//!     signature: &str,
//! ) -> Result<WebhookEvent, &'static str> {
//!     let secret = std::env::var("WEBHOOK_SECRET").unwrap();
//!
//!     match Webhooks::parse_event(&body, signature, &secret) {
//!         Ok(event) => {
//!             println!("Received event: {:?}", event.event_type);
//!             Ok(event)
//!         }
//!         Err(_) => Err("Invalid signature"),
//!     }
//! }
//! ```

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

/// Webhook event types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    #[serde(rename = "message.queued")]
    MessageQueued,
    #[serde(rename = "message.sent")]
    MessageSent,
    #[serde(rename = "message.delivered")]
    MessageDelivered,
    #[serde(rename = "message.failed")]
    MessageFailed,
    #[serde(rename = "message.undelivered")]
    MessageUndelivered,
}

/// Message status in webhook events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebhookMessageStatus {
    Queued,
    Sent,
    Delivered,
    Failed,
    Undelivered,
}

/// Data payload for message webhook events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookMessageData {
    /// The message ID
    pub message_id: String,
    /// Current message status
    pub status: WebhookMessageStatus,
    /// Recipient phone number
    pub to: String,
    /// Sender ID or phone number
    pub from: String,
    /// Error message if status is 'failed' or 'undelivered'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Error code if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// When the message was delivered (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivered_at: Option<String>,
    /// When the message failed (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_at: Option<String>,
    /// Number of SMS segments
    pub segments: i32,
    /// Credits charged
    pub credits_used: i32,
}

/// Webhook event from Sendly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    /// Unique event ID
    pub id: String,
    /// Event type
    #[serde(rename = "type")]
    pub event_type: WebhookEventType,
    /// Event data
    pub data: WebhookMessageData,
    /// When the event was created (ISO 8601)
    pub created_at: String,
    /// API version
    #[serde(default = "default_api_version")]
    pub api_version: String,
}

fn default_api_version() -> String {
    "2024-01-01".to_string()
}

/// Error type for webhook signature verification failures
#[derive(Error, Debug)]
pub enum WebhookError {
    #[error("Invalid webhook signature")]
    InvalidSignature,
    #[error("Failed to parse webhook payload: {0}")]
    ParseError(String),
    #[error("Invalid event structure")]
    InvalidStructure,
}

/// Webhook utilities for verifying and parsing Sendly webhook events
pub struct Webhooks;

impl Webhooks {
    /// Verify webhook signature from Sendly
    ///
    /// # Arguments
    ///
    /// * `payload` - Raw request body as string
    /// * `signature` - X-Sendly-Signature header value
    /// * `secret` - Your webhook secret from dashboard
    ///
    /// # Returns
    ///
    /// `true` if signature is valid, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// use sendly::webhooks::Webhooks;
    ///
    /// let is_valid = Webhooks::verify_signature(
    ///     &raw_body,
    ///     &signature,
    ///     &secret,
    /// );
    /// ```
    pub fn verify_signature(payload: &str, signature: &str, secret: &str) -> bool {
        if payload.is_empty() || signature.is_empty() || secret.is_empty() {
            return false;
        }

        let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
            Ok(mac) => mac,
            Err(_) => return false,
        };

        mac.update(payload.as_bytes());
        let result = mac.finalize();
        let expected = format!("sha256={}", hex::encode(result.into_bytes()));

        // Constant-time comparison
        constant_time_compare(signature, &expected)
    }

    /// Parse and validate a webhook event
    ///
    /// # Arguments
    ///
    /// * `payload` - Raw request body as string
    /// * `signature` - X-Sendly-Signature header value
    /// * `secret` - Your webhook secret from dashboard
    ///
    /// # Returns
    ///
    /// Parsed and validated `WebhookEvent` or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// use sendly::webhooks::Webhooks;
    ///
    /// match Webhooks::parse_event(&raw_body, &signature, &secret) {
    ///     Ok(event) => {
    ///         println!("Event type: {:?}", event.event_type);
    ///         println!("Message ID: {}", event.data.message_id);
    ///     }
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub fn parse_event(
        payload: &str,
        signature: &str,
        secret: &str,
    ) -> Result<WebhookEvent, WebhookError> {
        if !Self::verify_signature(payload, signature, secret) {
            return Err(WebhookError::InvalidSignature);
        }

        let event: WebhookEvent =
            serde_json::from_str(payload).map_err(|e| WebhookError::ParseError(e.to_string()))?;

        // Basic validation
        if event.id.is_empty() || event.created_at.is_empty() {
            return Err(WebhookError::InvalidStructure);
        }

        Ok(event)
    }

    /// Generate a webhook signature for testing purposes
    ///
    /// # Arguments
    ///
    /// * `payload` - The payload to sign
    /// * `secret` - The secret to use for signing
    ///
    /// # Returns
    ///
    /// The signature in the format "sha256=..."
    ///
    /// # Example
    ///
    /// ```rust
    /// use sendly::webhooks::Webhooks;
    ///
    /// let signature = Webhooks::generate_signature(&test_payload, "test_secret");
    /// ```
    pub fn generate_signature(payload: &str, secret: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        format!("sha256={}", hex::encode(result.into_bytes()))
    }
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_signature() {
        let payload = r#"{"id":"evt_123","type":"message.delivered"}"#;
        let secret = "test_secret";
        let signature = Webhooks::generate_signature(payload, secret);

        assert!(Webhooks::verify_signature(payload, &signature, secret));
        assert!(!Webhooks::verify_signature(payload, "invalid", secret));
    }

    #[test]
    fn test_generate_signature() {
        let payload = "test";
        let secret = "secret";
        let signature = Webhooks::generate_signature(payload, secret);

        assert!(signature.starts_with("sha256="));
        assert_eq!(signature.len(), 71); // "sha256=" + 64 hex chars
    }
}
