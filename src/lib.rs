//! # Sendly Rust SDK
//!
//! Official Rust client for the Sendly SMS API.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use sendly::{Sendly, SendMessageRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), sendly::Error> {
//!     let client = Sendly::new("sk_live_v1_your_api_key");
//!
//!     let message = client.messages().send(SendMessageRequest {
//!         to: "+15551234567".to_string(),
//!         text: "Hello from Sendly!".to_string(),
//!         message_type: None,
//!         media_urls: None,
//!         metadata: None,
//!     }).await?;
//!
//!     println!("Message sent: {}", message.id);
//!     Ok(())
//! }
//! ```
//!
//! ## Webhooks Management
//!
//! ```rust,no_run
//! use sendly::Sendly;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), sendly::Error> {
//!     let client = Sendly::new("sk_live_v1_your_api_key");
//!
//!     // Create a webhook
//!     let response = client.webhooks().create(
//!         "https://example.com/webhook",
//!         vec!["message.delivered", "message.failed"],
//!     ).await?;
//!
//!     println!("Webhook secret: {}", response.secret);
//!     Ok(())
//! }
//! ```
//!
//! ## Account & Credits
//!
//! ```rust,no_run
//! use sendly::Sendly;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), sendly::Error> {
//!     let client = Sendly::new("sk_live_v1_your_api_key");
//!
//!     let credits = client.account().credits().await?;
//!     println!("Available credits: {}", credits.available_balance);
//!     Ok(())
//! }
//! ```

mod account_resource;
pub mod business_upgrade;
mod campaigns;
mod client;
mod contacts;
mod conversations;
mod drafts;
mod enterprise;
mod error;
mod labels;
mod media;
mod numbers;
mod rules;
mod messages;
mod models;
mod templates;
mod verify;
mod webhook_resource;

pub mod webhooks;

pub use account_resource::AccountResource;
pub use business_upgrade::BusinessUpgradeResource;
pub use campaigns::*;
pub use client::{Sendly, SendlyConfig};
pub use contacts::*;
pub use conversations::ConversationsResource;
pub use drafts::DraftsResource;
pub use enterprise::EnterpriseResource;
pub use labels::LabelsResource;
pub use rules::RulesResource;
pub use error::{Error, Result};
pub use media::Media;
pub use messages::Messages;
pub use models::*;
pub use numbers::*;
pub use templates::*;
pub use verify::*;
pub use webhook_resource::WebhooksResource;
