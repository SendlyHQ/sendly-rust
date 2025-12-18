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
//!     }).await?;
//!
//!     println!("Message sent: {}", message.id);
//!     Ok(())
//! }
//! ```

mod client;
mod error;
mod messages;
mod models;

pub use client::{Sendly, SendlyConfig};
pub use error::{Error, Result};
pub use messages::Messages;
pub use models::*;
