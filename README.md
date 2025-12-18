# Sendly Rust SDK

Official Rust SDK for the Sendly SMS API.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sendly = "0.9"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use sendly::{Sendly, SendMessageRequest};

#[tokio::main]
async fn main() -> sendly::Result<()> {
    let client = Sendly::new("sk_live_v1_your_api_key");

    // Send an SMS
    let message = client.messages().send(SendMessageRequest {
        to: "+15551234567".to_string(),
        text: "Hello from Sendly!".to_string(),
    }).await?;

    println!("Message sent: {}", message.id);
    Ok(())
}
```

## Prerequisites for Live Messaging

Before sending live SMS messages, you need:

1. **Business Verification** - Complete verification in the [Sendly dashboard](https://sendly.live/dashboard)
   - **International**: Instant approval (just provide Sender ID)
   - **US/Canada**: Requires carrier approval (3-7 business days)

2. **Credits** - Add credits to your account
   - Test keys (`sk_test_*`) work without credits (sandbox mode)
   - Live keys (`sk_live_*`) require credits for each message

3. **Live API Key** - Generate after verification + credits
   - Dashboard → API Keys → Create Live Key

### Test vs Live Keys

| Key Type | Prefix | Credits Required | Verification Required | Use Case |
|----------|--------|------------------|----------------------|----------|
| Test | `sk_test_v1_*` | No | No | Development, testing |
| Live | `sk_live_v1_*` | Yes | Yes | Production messaging |

> **Note**: You can start development immediately with a test key. Messages to sandbox test numbers are free and don't require verification.

## Configuration

```rust
use sendly::{Sendly, SendlyConfig};
use std::time::Duration;

let config = SendlyConfig::new()
    .base_url("https://api.sendly.live/v1")
    .timeout(Duration::from_secs(60))
    .max_retries(5);

let client = Sendly::with_config("sk_live_v1_xxx", config);
```

## Messages

### Send an SMS

```rust
use sendly::{Sendly, SendMessageRequest};

let client = Sendly::new("sk_live_v1_xxx");

// With request struct
let message = client.messages().send(SendMessageRequest {
    to: "+15551234567".to_string(),
    text: "Hello from Sendly!".to_string(),
}).await?;

// Convenience method
let message = client.messages()
    .send_to("+15551234567", "Hello!")
    .await?;

println!("ID: {}", message.id);
println!("Status: {}", message.status);
println!("Credits: {}", message.credits_used);
```

### List Messages

```rust
use sendly::{Sendly, ListMessagesOptions, MessageStatus};

let client = Sendly::new("sk_live_v1_xxx");

// List all
let messages = client.messages().list(None).await?;

for msg in &messages {
    println!("{}: {}", msg.id, msg.to);
}

// With options
let messages = client.messages().list(Some(
    ListMessagesOptions::new()
        .limit(50)
        .offset(0)
        .status(MessageStatus::Delivered)
        .to("+15551234567")
)).await?;

// Pagination info
println!("Total: {}", messages.total());
println!("Has more: {}", messages.has_more());
```

### Get a Message

```rust
let message = client.messages().get("msg_abc123").await?;

println!("To: {}", message.to);
println!("Text: {}", message.text);
println!("Status: {}", message.status);
println!("Delivered: {:?}", message.delivered_at);
```

### Iterate All Messages

```rust
use futures::StreamExt;

// Auto-pagination with async stream
let mut stream = client.messages().iter(None);

while let Some(result) = stream.next().await {
    let message = result?;
    println!("{}: {}", message.id, message.to);
}
```

## Error Handling

```rust
use sendly::{Error, Sendly, SendMessageRequest};

match client.messages().send(request).await {
    Ok(message) => {
        println!("Sent: {}", message.id);
    }
    Err(Error::Authentication { message }) => {
        eprintln!("Invalid API key: {}", message);
    }
    Err(Error::RateLimit { message, retry_after }) => {
        eprintln!("Rate limited: {}", message);
        if let Some(seconds) = retry_after {
            eprintln!("Retry after: {} seconds", seconds);
        }
    }
    Err(Error::InsufficientCredits { message }) => {
        eprintln!("Add more credits: {}", message);
    }
    Err(Error::Validation { message }) => {
        eprintln!("Invalid request: {}", message);
    }
    Err(Error::NotFound { message }) => {
        eprintln!("Not found: {}", message);
    }
    Err(Error::Network { message }) => {
        eprintln!("Network error: {}", message);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

## Message Object

```rust
message.id           // Unique identifier
message.to           // Recipient phone number
message.text         // Message content
message.status       // MessageStatus enum
message.credits_used // Credits consumed
message.created_at   // DateTime<Utc>
message.updated_at   // DateTime<Utc>
message.delivered_at // Option<DateTime<Utc>>
message.error_code   // Option<String>
message.error_message // Option<String>

// Helper methods
message.is_delivered() // bool
message.is_failed()    // bool
message.is_pending()   // bool
```

## Message Status

| Status | Description |
|--------|-------------|
| `Queued` | Message is queued for delivery |
| `Sending` | Message is being sent |
| `Sent` | Message was sent to carrier |
| `Delivered` | Message was delivered |
| `Failed` | Message delivery failed |

## Pricing Tiers

| Tier | Countries | Credits per SMS |
|------|-----------|-----------------|
| Domestic | US, CA | 1 |
| Tier 1 | GB, PL, IN, etc. | 8 |
| Tier 2 | FR, JP, AU, etc. | 12 |
| Tier 3 | DE, IT, MX, etc. | 16 |

## Sandbox Testing

Use test API keys (`sk_test_v1_xxx`) with these test numbers:

| Number | Behavior |
|--------|----------|
| +15550001234 | Success |
| +15550001001 | Invalid number |
| +15550001002 | Carrier rejected |
| +15550001003 | No credits |
| +15550001004 | Rate limited |

## Features

- Async/await with Tokio
- Automatic retries with exponential backoff
- Rate limit handling
- Strong typing with enums
- Comprehensive error types
- Stream-based pagination

## License

MIT
