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
//!     timestamp: &str,
//! ) -> Result<WebhookEvent, &'static str> {
//!     let secret = std::env::var("WEBHOOK_SECRET").unwrap();
//!
//!     match Webhooks::parse_event(&body, signature, &secret, Some(timestamp)) {
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
use serde_json::Value;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

const SIGNATURE_TOLERANCE_SECONDS: u64 = 300;

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
    #[serde(rename = "message.bounced")]
    MessageBounced,
    #[serde(rename = "message.retrying")]
    MessageRetrying,
    #[serde(rename = "message.received")]
    MessageReceived,
    #[serde(rename = "message.opt_out")]
    MessageOptOut,
    #[serde(rename = "message.opt_in")]
    MessageOptIn,
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
    Bounced,
    Retrying,
    Received,
    Undelivered,
}

/// Data payload for message webhook events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookMessageData {
    /// The message ID
    pub id: String,
    /// Current message status
    pub status: WebhookMessageStatus,
    /// Recipient phone number
    pub to: String,
    /// Sender ID or phone number
    pub from: String,
    /// Message direction
    #[serde(default = "default_direction")]
    pub direction: String,
    /// Organization ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,
    /// Message text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Error message if status is 'failed' or 'undelivered'
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Error code if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// When the message was delivered
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delivered_at: Option<Value>,
    /// When the message failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_at: Option<Value>,
    /// When the message was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<Value>,
    /// Number of SMS segments
    #[serde(default = "default_segments")]
    pub segments: i32,
    /// Credits charged
    #[serde(default)]
    pub credits_used: i32,
    /// Message format (sms or mms)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_format: Option<String>,
    /// Media URLs for MMS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_urls: Option<Vec<String>>,
}

impl WebhookMessageData {
    /// Backwards-compatible alias for id
    pub fn message_id(&self) -> &str {
        &self.id
    }
}

fn default_direction() -> String {
    "outbound".to_string()
}

fn default_segments() -> i32 {
    1
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
    #[serde(skip)]
    pub data: WebhookMessageData,
    /// When the event was created (unix timestamp)
    #[serde(default)]
    pub created: Value,
    /// API version
    #[serde(default = "default_api_version")]
    pub api_version: String,
    /// Whether this is a live (production) event
    #[serde(default)]
    pub livemode: bool,
}

fn default_api_version() -> String {
    "2024-01".to_string()
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
    /// Verify webhook signature from Sendly.
    /// Pass `None` for timestamp to skip timestamp verification (backwards compat).
    pub fn verify_signature(
        payload: &str,
        signature: &str,
        secret: &str,
        timestamp: Option<&str>,
    ) -> bool {
        if payload.is_empty() || signature.is_empty() || secret.is_empty() {
            return false;
        }

        let signed_payload = match timestamp {
            Some(ts) if !ts.is_empty() => {
                if let Ok(ts_val) = ts.parse::<u64>() {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    if now.abs_diff(ts_val) > SIGNATURE_TOLERANCE_SECONDS {
                        return false;
                    }
                }
                format!("{}.{}", ts, payload)
            }
            _ => payload.to_string(),
        };

        let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
            Ok(mac) => mac,
            Err(_) => return false,
        };

        mac.update(signed_payload.as_bytes());
        let result = mac.finalize();
        let expected = format!("sha256={}", hex::encode(result.into_bytes()));

        constant_time_compare(signature, &expected)
    }

    /// Parse and validate a webhook event.
    /// Pass `None` for timestamp to skip timestamp verification.
    pub fn parse_event(
        payload: &str,
        signature: &str,
        secret: &str,
        timestamp: Option<&str>,
    ) -> Result<WebhookEvent, WebhookError> {
        if !Self::verify_signature(payload, signature, secret, timestamp) {
            return Err(WebhookError::InvalidSignature);
        }

        let raw: Value =
            serde_json::from_str(payload).map_err(|e| WebhookError::ParseError(e.to_string()))?;

        let id = raw["id"]
            .as_str()
            .ok_or(WebhookError::InvalidStructure)?
            .to_string();
        let event_type: WebhookEventType = serde_json::from_value(raw["type"].clone())
            .map_err(|e| WebhookError::ParseError(e.to_string()))?;

        let data_val = &raw["data"];
        let obj_val = if data_val["object"].is_object() {
            &data_val["object"]
        } else {
            data_val
        };

        let mut msg_data: WebhookMessageData = serde_json::from_value(obj_val.clone())
            .map_err(|e| WebhookError::ParseError(e.to_string()))?;

        if msg_data.id.is_empty() {
            if let Some(mid) = obj_val["message_id"].as_str() {
                msg_data.id = mid.to_string();
            }
        }

        let created = if !raw["created"].is_null() {
            raw["created"].clone()
        } else {
            raw["created_at"].clone()
        };

        Ok(WebhookEvent {
            id,
            event_type,
            data: msg_data,
            created,
            api_version: raw["api_version"]
                .as_str()
                .unwrap_or("2024-01")
                .to_string(),
            livemode: raw["livemode"].as_bool().unwrap_or(false),
        })
    }

    /// Generate a webhook signature for testing purposes.
    /// Pass `None` for timestamp to skip timestamp in signature.
    pub fn generate_signature(payload: &str, secret: &str, timestamp: Option<&str>) -> String {
        let signed_payload = match timestamp {
            Some(ts) if !ts.is_empty() => format!("{}.{}", ts, payload),
            _ => payload.to_string(),
        };

        let mut mac =
            HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
        mac.update(signed_payload.as_bytes());
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
    fn test_verify_signature_with_timestamp() {
        let payload = r#"{"id":"evt_123","type":"message.delivered"}"#;
        let secret = "test_secret";
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();
        let signature = Webhooks::generate_signature(payload, secret, Some(&ts));

        assert!(Webhooks::verify_signature(payload, &signature, secret, Some(&ts)));
        assert!(!Webhooks::verify_signature(payload, "invalid", secret, Some(&ts)));
    }

    #[test]
    fn test_verify_signature_without_timestamp() {
        let payload = r#"{"id":"evt_123","type":"message.delivered"}"#;
        let secret = "test_secret";
        let signature = Webhooks::generate_signature(payload, secret, None);

        assert!(Webhooks::verify_signature(payload, &signature, secret, None));
    }

    #[test]
    fn test_generate_signature() {
        let payload = "test";
        let secret = "secret";
        let signature = Webhooks::generate_signature(payload, secret, None);

        assert!(signature.starts_with("sha256="));
        assert_eq!(signature.len(), 71); // "sha256=" + 64 hex chars
    }
}
