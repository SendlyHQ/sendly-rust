<p align="center">
  <img src="https://raw.githubusercontent.com/SendlyHQ/sendly-rust/main/.github/header.svg" alt="Sendly Rust SDK" />
</p>

<p align="center">
  <a href="https://crates.io/crates/sendly"><img src="https://img.shields.io/crates/v/sendly.svg?style=flat-square" alt="crates.io" /></a>
  <a href="https://github.com/SendlyHQ/sendly-rust/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/sendly?style=flat-square" alt="license" /></a>
</p>

# Sendly Rust SDK

Official Rust SDK for the Sendly SMS API.

## Installation

```bash
# cargo
cargo add sendly
```

Or add to your `Cargo.toml`:

```toml
[dependencies]
sendly = "3.29.0"
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
    .base_url("https://sendly.live/api/v1")
    .timeout(Duration::from_secs(60))
    .max_retries(5);

let client = Sendly::with_config("sk_live_v1_xxx", config);
```

## Messages

### Send an SMS

```rust
use sendly::{Sendly, SendMessageRequest};

let client = Sendly::new("sk_live_v1_xxx");

// Marketing message (default)
let message = client.messages()
    .send_to("+15551234567", "Check out our new features!")
    .await?;

// Transactional message (bypasses quiet hours)
let message = client.messages().send(SendMessageRequest {
    to: "+15551234567".to_string(),
    text: "Your verification code is: 123456".to_string(),
    message_type: Some(MessageType::Transactional),
    ..Default::default()
}).await?;

// With custom metadata (max 4KB)
use std::collections::HashMap;
let mut metadata = HashMap::new();
metadata.insert("order_id".to_string(), serde_json::json!("12345"));
metadata.insert("customer_id".to_string(), serde_json::json!("cust_abc"));

let message = client.messages().send(SendMessageRequest {
    to: "+15551234567".to_string(),
    text: "Your order #12345 has shipped!".to_string(),
    metadata: Some(metadata),
    ..Default::default()
}).await?;

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

### Scheduling Messages

```rust
use sendly::{Sendly, ScheduleMessageRequest};

// Schedule a message for future delivery
let scheduled = client.messages().schedule(ScheduleMessageRequest {
    to: "+15551234567".to_string(),
    text: "Your appointment is tomorrow!".to_string(),
    scheduled_at: "2025-01-15T10:00:00Z".to_string(),
    ..Default::default()
}).await?;

println!("Scheduled: {}", scheduled.id);
println!("Will send at: {}", scheduled.scheduled_at);

// List scheduled messages
let result = client.messages().list_scheduled(None).await?;
for msg in &result {
    println!("{}: {}", msg.id, msg.scheduled_at);
}

// Get a specific scheduled message
let msg = client.messages().get_scheduled("sched_xxx").await?;

// Cancel a scheduled message (refunds credits)
let result = client.messages().cancel_scheduled("sched_xxx").await?;
println!("Refunded: {} credits", result.credits_refunded);
```

### Batch Messages

```rust
use sendly::{Sendly, SendBatchRequest, BatchMessageItem};

// Send multiple messages in one API call (up to 1000)
let batch = client.messages().send_batch(SendBatchRequest {
    messages: vec![
        BatchMessageItem { to: "+15551234567".into(), text: "Hello User 1!".into() },
        BatchMessageItem { to: "+15559876543".into(), text: "Hello User 2!".into() },
        BatchMessageItem { to: "+15551112222".into(), text: "Hello User 3!".into() },
    ],
    ..Default::default()
}).await?;

println!("Batch ID: {}", batch.batch_id);
println!("Queued: {}", batch.queued);
println!("Failed: {}", batch.failed);
println!("Credits used: {}", batch.credits_used);

// Get batch status
let status = client.messages().get_batch("batch_xxx").await?;

// List all batches
let batches = client.messages().list_batches(None).await?;

// Preview batch (dry run) - validates without sending
let preview = client.messages().preview_batch(SendBatchRequest {
    messages: vec![
        BatchMessageItem { to: "+15551234567".into(), text: "Hello User 1!".into() },
        BatchMessageItem { to: "+447700900123".into(), text: "Hello UK!".into() },
    ],
    ..Default::default()
}).await?;
println!("Total credits needed: {}", preview.total_credits);
println!("Valid: {}, Invalid: {}", preview.valid, preview.invalid);
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

## Webhooks

```rust
use sendly::{Sendly, CreateWebhookRequest, UpdateWebhookRequest};

// Create a webhook endpoint
let webhook = client.webhooks().create(CreateWebhookRequest {
    url: "https://example.com/webhooks/sendly".to_string(),
    events: vec!["message.delivered".to_string(), "message.failed".to_string()],
}).await?;

println!("Webhook ID: {}", webhook.id);
println!("Secret: {}", webhook.secret); // Store securely!

// List all webhooks
let webhooks = client.webhooks().list().await?;

// Get a specific webhook
let wh = client.webhooks().get("whk_xxx").await?;

// Update a webhook
client.webhooks().update("whk_xxx", UpdateWebhookRequest {
    url: Some("https://new-endpoint.example.com/webhook".to_string()),
    events: Some(vec![
        "message.delivered".to_string(),
        "message.failed".to_string(),
        "message.sent".to_string(),
    ]),
    ..Default::default()
}).await?;

// Test a webhook
let result = client.webhooks().test("whk_xxx").await?;

// Rotate webhook secret
let rotation = client.webhooks().rotate_secret("whk_xxx").await?;

// Delete a webhook
client.webhooks().delete("whk_xxx").await?;

// List available webhook event types
let event_types = client.webhooks().list_event_types().await?;
for event_type in &event_types {
    println!("Event: {}", event_type);
}
```

## Account & Credits

```rust
// Get account information
let account = client.account().get().await?;
println!("Email: {}", account.email);

// Check credit balance
let credits = client.account().get_credits().await?;
println!("Available: {} credits", credits.available_balance);
println!("Reserved: {} credits", credits.reserved_balance);
println!("Total: {} credits", credits.balance);

// View credit transaction history
let transactions = client.account().get_credit_transactions().await?;
for tx in &transactions.data {
    println!("{}: {} credits - {}", tx.tx_type, tx.amount, tx.description);
}

// List API keys
let keys = client.account().list_api_keys().await?;
for key in &keys.data {
    println!("{}: {}*** ({})", key.name, key.prefix, key.key_type);
}

// Get a specific API key
let key = client.account().get_api_key("key_xxx").await?;

// Get API key usage stats
let usage = client.account().get_api_key_usage("key_xxx").await?;
println!("Messages sent: {}", usage.messages_sent);

// Create a new API key
let new_key = client.account().create_api_key(CreateApiKeyRequest {
    name: "Production Key".to_string(),
    key_type: "live".to_string(),
    scopes: Some(vec!["sms:send".to_string(), "sms:read".to_string()]),
}).await?;
println!("New key: {}", new_key.key); // Only shown once!

// Revoke an API key
client.account().revoke_api_key("key_xxx").await?;
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
| Domestic | US, CA | 2 |
| Tier 1 | GB, PL, IN, etc. | 8 |
| Tier 2 | FR, JP, AU, etc. | 12 |
| Tier 3 | DE, IT, MX, etc. | 16 |

## Sandbox Testing

Use test API keys (`sk_test_v1_xxx`) with these test numbers:

| Number | Behavior |
|--------|----------|
| +15005550000 | Success (instant) |
| +15005550001 | Fails: invalid_number |
| +15005550002 | Fails: unroutable_destination |
| +15005550003 | Fails: queue_full |
| +15005550004 | Fails: rate_limit_exceeded |
| +15005550006 | Fails: carrier_violation |

## Features

- Async/await with Tokio
- Automatic retries with exponential backoff
- Rate limit handling
- Strong typing with enums
- Comprehensive error types
- Stream-based pagination

## Enterprise

The Enterprise API lets you programmatically manage workspaces, verification, credits, and API keys for multi-tenant platforms. Requires an enterprise master key (`sk_live_v1_master_*`).

### Quick Provision

Create a fully configured workspace in a single call:

```rust
let client = Sendly::new("sk_live_v1_master_YOUR_KEY");

let options = ProvisionWorkspaceOptions::new("Acme Insurance - Austin")
    .source_workspace_id("ws_verified")
    .credit_amount(5000)
    .credit_source_workspace_id("SOURCE_WORKSPACE_ID")
    .key_name("Production")
    .key_type("live")
    .generate_opt_in_page(true);

let result = client.enterprise().provision(&options).await?;

println!("{}", result.workspace.id);
println!("{}", result.key.as_ref().unwrap().key);
```

Three provisioning modes:

| Mode | Params | Description |
|------|--------|-------------|
| **Inherit** | `.source_workspace_id()` | Shares toll-free number from verified workspace |
| **Inherit + New Number** | `.source_workspace_id()` + `.inherit_with_new_number(true)` | Copies business info, purchases new number |
| **Fresh** | `.verification(VerificationData{...})` | Full business details, new number + carrier approval |

### Workspace Management

```rust
let ws = client.enterprise().workspaces().create("Acme Insurance", None).await?;
let list = client.enterprise().workspaces().list().await?;
let detail = client.enterprise().workspaces().get("ws_xxx").await?;
client.enterprise().workspaces().delete("ws_xxx").await?;
```

### Credits & API Keys

```rust
client.enterprise().workspaces()
    .transfer_credits("ws_dest", "ws_source", 5000).await?;

let key = client.enterprise().workspaces()
    .create_key("ws_xxx", Some("Production"), Some("live")).await?;
println!("{}", key.key);

client.enterprise().workspaces().revoke_key("ws_xxx", "key_abc").await?;
```

### Webhooks & Analytics

```rust
client.enterprise().webhooks().set("https://yourapp.com/webhooks").await?;
let overview = client.enterprise().analytics().overview().await?;
let messages = client.enterprise().analytics().messages(Some("30d"), None).await?;
let delivery = client.enterprise().analytics().delivery().await?;
```

Full enterprise docs: [sendly.live/docs/enterprise](https://sendly.live/docs/enterprise)

---

## License

MIT
