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
sendly = "6.0.0"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use sendly::{Sendly, SendMessageRequest};

#[tokio::main]
async fn main() -> sendly::Result<()> {
    let client = Sendly::new("sk_live_v1_your_api_key");

    // Send an SMS
    let message = client.messages()
        .send(SendMessageRequest::new("+14155550123", "Hello from Sendly!"))
        .await?;

    println!("Message sent: {}", message.id);
    Ok(())
}
```

## Prerequisites for Live Messaging

Before sending live SMS messages, you need:

1. **Business Verification** - Complete verification in the [Sendly dashboard](https://sendly.live/dashboard)
   - **International**: Instant approval (just provide Sender ID)
   - **US/Canada**: Requires carrier approval

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
    .max_retries(5)
    .organization_id("org_xxx");

let client = Sendly::with_config("sk_live_v1_xxx", config);
```

The organization id is sent as the `X-Organization-Id` header on every request.
Set it on the config, let the client pick up `SENDLY_ORG_ID` from the
environment, or switch it later with `client.set_organization_id("org_other")`
(that one takes `&mut self`).

### Retries and timeouts

The client retries connect failures and timeouts, up to `max_retries`
(default 3) with exponential backoff (1s, 2s, 4s between attempts), and uses a
30-second request timeout by default. It also retries one HTTP answer: the 429
`too_many_concurrent_verifications`, which means the API was still checking
other requests made with your key from the same address and never ran this
one. The client waits out its `Retry-After` (1 second) and sends the request
again, within `max_retries`. It never waits more than 60 seconds for a 429.
Every other HTTP error, including a 5xx, is returned straight away: another
429 comes back as `Error::RateLimit` carrying `retry_after` and `code` (see
[Rate Limits](#rate-limits)), so you choose whether and when to try again. File
uploads are not sent again after a timeout or a network error.

Every POST carries an `Idempotency-Key`, and every retry the client makes
reuses it. The RCS and voice writes carry one on their PATCH, PUT and DELETE
calls too. `send_batch` carries one only when you pass it, because the API
already deduplicates a batch by its contents. The API deduplicates on sends
(SMS, MMS, group, scheduled, WhatsApp, RCS and conversation replies), batches,
draft approval (which sends), starting a verification, credit transfers, number
purchases, workspace provisioning (single and bulk), enterprise credit deposits
and transfers, WhatsApp signup and template creation, and the RCS and voice
writes, so a retry of one of those that already reached the API is not executed
twice. Other writes, such as creating a webhook or a contact, can run twice if
a timed-out attempt had already reached the API.

## Idempotency

Every send POST carries an automatically generated `Idempotency-Key`, created once per logical request and reused on every retry the client makes (after a timeout, a connection failure or the busy key-check 429), so a retry of a request that already reached the API returns the original result instead of sending again. The API records an answer under the key only when it is final: a 2xx or a 4xx other than 429 is kept for 24 hours and replayed to any request that repeats the key, while a 5xx or a 429 is never recorded, so repeating the key after one of those runs the request. Pass your own key through the `*_with_options` send methods when the guarantee needs to outlive the process, such as a job queue that re-runs after a crash, or when you retry a 5xx yourself. Keys are 1-255 printable ASCII characters; `send_batch` attaches no automatic key because the API already deduplicates identical batches by their contents. RCS registration writes (brand and agent create/update, test devices, submit, request launch) carry a key the same way, including their PATCH and PUT calls, with `*_with_options` variants for your own key. So do the voice and call writes: placing a call, hanging one up, changing a number's voice settings, registering an emergency address, and creating, updating or deleting an agent.

```rust
use sendly::{IdempotentRequestOptions, Sendly, SendMessageRequest};

let client = Sendly::new("sk_live_v1_xxx");

let message = client.messages()
    .send_with_options(
        SendMessageRequest::new("+14155550123", "Your order #4821 has shipped!"),
        IdempotentRequestOptions::new().idempotency_key("order-4821-shipped"),
    )
    .await?;
```

Full details: https://sendly.live/docs/idempotency

## Rate Limits

Requests are counted per API key in a fixed 60-second window that starts with the key's first request:

| Key | Requests per minute |
|-----|---------------------|
| Test (`sk_test_v1_*`) | 60 |
| Live (`sk_live_v1_*`) | 600 |
| Enterprise master key | 3000 |

Going over the limit returns `Error::RateLimit` with `code` set to
`rate_limit_exceeded` and `retry_after` set to the seconds until the window
resets. Not every 429 clears with time, so read the code before you wait:

| `code` | Meaning | `is_retryable()` |
|--------|---------|------------------|
| `rate_limit_exceeded`, `provision_rate_limit` | Too many requests; wait `retry_after` | true |
| `too_many_concurrent_verifications` | The API was still checking other requests made with your key from this address. You see it only after the client's own retries ran out | true |
| `too_many_failed_key_attempts` | Too many requests with a wrong API key from this address; the API refuses keys from it for up to 300 seconds. Fix the key, do not retry | false |
| `max_attempts_exceeded` | A verification took too many wrong codes and has failed | false |
| `quota_exceeded` | The workspace's monthly message quota is used up until the month resets or an admin raises it | false |
| `daily_call_limit` | `calls().create()` hit today's calling limit | false |
| `whatsapp_signup_limit_reached` | Too many WhatsApp connections failed in the last 24 hours | false |

The one-time-password limits on `verify().send()` answer with a `retry_after`
of up to 600 or 86,400 seconds, so check it before you sleep.

```rust
use sendly::Error;

match client.messages().send(request).await {
    Ok(message) => println!("sent {}", message.id),
    Err(Error::RateLimit { code, .. }) if code.as_deref() == Some("too_many_failed_key_attempts") => {
        eprintln!("the API key is wrong; fix it instead of retrying");
    }
    Err(e) if e.is_retryable() => {
        eprintln!("transient ({e}); retry_after: {:?} seconds", e.retry_after());
    }
    Err(e) => return Err(e),
}
```

`err.is_retryable()` encodes the table, so a generic retry loop can rely on it.

## Messages

### Send an SMS

```rust
use sendly::{Sendly, SendMessageRequest, MessageType};

let client = Sendly::new("sk_live_v1_xxx");

// Marketing message (default)
let message = client.messages()
    .send_to("+14155550123", "Check out our new features!")
    .await?;

// Send from one of your owned numbers
let message = client.messages().send(
    SendMessageRequest::new("+14155550123", "Hello from Sendly!")
        .with_from("+15125550188"),
).await?;

// Transactional message (bypasses quiet hours)
let message = client.messages().send(
    SendMessageRequest::new("+14155550123", "Your verification code is: 123456")
        .with_message_type(MessageType::Transactional),
).await?;

// With custom metadata (max 4KB)
use std::collections::HashMap;
let mut metadata = HashMap::new();
metadata.insert("order_id".to_string(), serde_json::json!("12345"));
metadata.insert("customer_id".to_string(), serde_json::json!("cust_abc"));

let message = client.messages().send(
    SendMessageRequest::new("+14155550123", "Your order #12345 has shipped!")
        .with_metadata(metadata),
).await?;

println!("ID: {}", message.id);
println!("Status: {}", message.status);
println!("Credits: {}", message.credits_used);

// A simulated send reached no handset: a test key, a sandbox number, or a live
// key whose account is not yet set up for this destination.
if message.simulated {
    println!("Simulated: {:?} (finish setup at {:?})", message.simulated_reason, message.action_url);
}
```

Text is limited to 1600 characters, counted as characters rather than bytes.

### List Messages

```rust
use sendly::{Sendly, ListMessagesOptions, MessageStatus};

let client = Sendly::new("sk_live_v1_xxx");

// List all
let messages = client.messages().list(None).await?;

for msg in messages.iter() {
    println!("{}: {}", msg.id, msg.to);
}

// Filter by status and recipient, and page with limit (max 100) and offset
let delivered = client.messages().list(Some(
    ListMessagesOptions::new()
        .status(MessageStatus::Delivered)
        .to("+14155550123")
        .limit(50)
        .offset(0)
)).await?;

// `len()` is this page; `total()` counts every match across all pages
println!("{} of {}", delivered.len(), delivered.total());
if let Some(page) = &delivered.pagination {
    println!("page {} of {}, more: {}", page.page, page.total_pages, page.has_more);
}
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

// Schedule a message for future delivery. scheduled_at must be 5 minutes to
// 5 days ahead, so build it from the current time (here with the chrono crate).
let scheduled = client.messages().schedule(ScheduleMessageRequest {
    to: "+14155550123".to_string(),
    text: "Your appointment is tomorrow!".to_string(),
    scheduled_at: (chrono::Utc::now() + chrono::Duration::days(1))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    from: None,
    message_type: None,
    metadata: None,
}).await?;

println!("Scheduled: {}", scheduled.id);
println!("Will send at: {}", scheduled.scheduled_at);

// List scheduled messages
let result = client.messages().list_scheduled(None).await?;
for msg in &result.data {
    println!("{}: {}", msg.id, msg.scheduled_at);
}

// Get a specific scheduled message
let msg = client.messages().get_scheduled("schd_xxx").await?;

// Cancel a scheduled message (refunds credits)
let result = client.messages().cancel_scheduled("schd_xxx").await?;
println!("Refunded: {} credits", result.credits_refunded);
```

### Batch Messages

```rust
use sendly::{Sendly, SendBatchRequest, BatchMessageItem};

// Send multiple messages in one API call (up to 10,000)
let batch = client.messages().send_batch(SendBatchRequest {
    messages: vec![
        BatchMessageItem { to: "+14155550123".into(), text: "Hello User 1!".into(), metadata: None },
        BatchMessageItem { to: "+14155550124".into(), text: "Hello User 2!".into(), metadata: None },
        BatchMessageItem { to: "+14155550125".into(), text: "Hello User 3!".into(), metadata: None },
    ],
    from: None,
    message_type: None,
    metadata: None,
}).await?;

println!("Batch ID: {}", batch.batch_id);
println!("Total: {}", batch.total);
println!("Failed: {}", batch.failed);
println!("Credits used: {}", batch.credits_used);

// Get batch status: delivered counts, and credits reserved and refunded
let status = client.messages().get_batch(&batch.batch_id).await?;
println!(
    "{} delivered, {} reserved, {} refunded",
    status.delivered, status.credits_reserved, status.credits_refunded
);

// List recent batches (one page; `len()` is its size)
let messages = client.messages();
let batches = messages.list_batches(None).await?;
for batch in batches {
    println!("{}: {:?}", batch.batch_id, batch.status);
}

// Preview batch (dry run) - validates without sending
let preview = client.messages().preview_batch(SendBatchRequest {
    messages: vec![
        BatchMessageItem { to: "+14155550123".into(), text: "Hello User 1!".into(), metadata: None },
        BatchMessageItem { to: "+447700900123".into(), text: "Hello UK!".into(), metadata: None },
    ],
    from: None,
    message_type: None,
    metadata: None,
}).await?;
println!("{} of {} will send", preview.will_send, preview.total_messages);
println!("Credits needed: {} (balance {})", preview.credits_needed, preview.current_balance);
println!("Duplicates removed: {}", preview.duplicates);
for blocked in &preview.blocked_messages {
    println!("#{} {} blocked: {}", blocked.index, blocked.to, blocked.reason);
}
for warning in &preview.warnings {
    println!("warning: {}", warning);
}
if !preview.can_send {
    println!("this batch would not send as it stands");
}
```

`can_send` is true when nothing the preview found stops a send: the key has the
`sms:send` scope (`has_write_scope`), the batch has at most 10,000 messages (a
send of more is rejected with 400 `batch_too_large`), at least one message is
sendable, nothing is blocked for a reason other than an opt-out (a live send
rejects the whole batch then), and, for a live key, the balance covers it
(`has_enough_credits`). A test key's send skips the destination and
verification checks, so it can go through while `can_send` is false.

### Iterate All Messages

```rust
use futures::StreamExt;
use tokio::pin;

// Auto-pagination with an async stream. Bind the resource first — the stream
// borrows it — and pin the stream before polling it.
let messages = client.messages();
let stream = messages.iter(None);
pin!(stream);

while let Some(result) = stream.next().await {
    let message = result?;
    println!("{}: {}", message.id, message.to);
}
```

## Webhooks

```rust
use sendly::{Sendly, UpdateWebhookRequest};

// Create a webhook endpoint
let webhook = client.webhooks().create(
    "https://acme.example/webhooks/sendly",
    vec!["message.delivered", "message.failed"],
).await?;

// `create` answers a WebhookCreatedResponse: the signing secret plus the
// webhook itself, which arrives nested on some responses and flattened on
// others — `get_webhook()` reads whichever came back.
if let Some(created) = webhook.get_webhook() {
    println!("Webhook ID: {}", created.id); // whk_…
}
println!("Secret: {}", webhook.secret); // whsec_… — store it securely!

// List all webhooks
let webhooks = client.webhooks().list().await?;

// Get a specific webhook
let wh = client.webhooks().get("whk_xxx").await?;

// Update a webhook
client.webhooks().update("whk_xxx", UpdateWebhookRequest {
    url: Some("https://hooks.acme.example/webhook".to_string()),
    events: Some(vec![
        "message.delivered".to_string(),
        "message.failed".to_string(),
        "message.sent".to_string(),
    ]),
    ..Default::default()
}).await?;

// Send a webhook.test event. An endpoint that accepts it comes back Ok; one
// that refuses it or cannot be reached comes back Err(Error::Validation) with
// the reason in the message.
match client.webhooks().test("whk_xxx").await {
    Ok(result) => println!("{} in {}ms", result.status_code, result.response_time_ms),
    Err(sendly::Error::Validation { message, .. }) => println!("test failed: {}", message),
    Err(e) => return Err(e),
}

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

### Receiving events

`Webhooks::parse_event` verifies the signature and decodes the envelope. Sendly
sends two shapes of payload, and the difference matters:

- **Message events** (`message.*`) carry a message. `WebhookEvent::data` holds
  the typed `WebhookMessageData` view of it.
- **Everything else** — `rcs_*`, `whatsapp_*`, `call.*`, `short_code.*`,
  `brand.*`, `campaign.*`, `assignment.*`, `number.*`, `port*`, `contact*`,
  `contacts.*`, `conversation.*`, `draft.*` and `verification.*` — carries a
  different object entirely. `data` is `None` for those, and the payload is on
  `WebhookEvent::object`: `data.object` exactly as it arrived, for every event
  type. `event.object_as::<T>()` deserializes it into a type of your choosing.

```rust
use sendly::webhooks::{WebhookError, WebhookEventType, WebhookVerificationData, Webhooks};

// `rcs_agent.live` sends { agent_id, name, stage, organization_id }
#[derive(serde::Deserialize)]
struct RcsAgentLive {
    agent_id: String,
    name: String,
    stage: String,
}

fn handle_webhook(body: &str, signature: &str, timestamp: &str) -> Result<(), WebhookError> {
    let secret = std::env::var("SENDLY_WEBHOOK_SECRET").unwrap();
    let event = Webhooks::parse_event(body, signature, &secret, Some(timestamp))?;

    match event.event_type {
        // Message events: the typed message view is populated.
        WebhookEventType::MessageDelivered | WebhookEventType::MessageFailed => {
            if let Some(message) = &event.data {
                println!("{} -> {} ({:?})", message.id, message.to, message.status);
            }
        }

        // Lifecycle event: `data` is `None`, the payload is on `object`.
        WebhookEventType::RcsAgentLive => {
            let agent: RcsAgentLive = event.object_as()?;
            println!("agent {} ({}) is {}", agent.name, agent.agent_id, agent.stage);
        }

        // Verification events decode into the type the crate already ships.
        WebhookEventType::VerificationVerified => {
            let verification: WebhookVerificationData = event.object_as()?;
            println!("{} verified after {} attempts", verification.phone, verification.attempts);
        }

        // Or read one field straight off the raw object, no struct needed.
        // Note `id` here is the *contact* id; the message that triggered the
        // flag is on `message_id`.
        WebhookEventType::ContactAutoFlagged => {
            let contact_id = event.object["id"].as_str().unwrap_or_default();
            let phone = event.object["phone_number"].as_str().unwrap_or_default();
            println!("contact {contact_id} ({phone}) auto-flagged");
        }

        // `WebhookEventType` is `#[non_exhaustive]`, so a wildcard arm is required.
        _ => {}
    }

    Ok(())
}
```

`object_as` needs `serde` with the `derive` feature in your own `Cargo.toml`.
`object` is a `serde_json::Value`, so indexing it needs nothing extra, and an
event type this SDK version doesn't know still parses — it arrives as
`WebhookEventType::Unknown(String)` with its payload intact on `object`.

`Webhooks` is a free helper type, not something you reach through the client:
`Webhooks::verify_signature(payload, signature, secret, timestamp)` is the bare
boolean check behind `parse_event`, and `Webhooks::generate_signature(payload,
secret, timestamp)` signs a payload so you can exercise your handler in tests.
Both take the payload first and the timestamp last. Always pass the
`X-Sendly-Timestamp` header value: Sendly signs `"{timestamp}.{payload}"`, and
`None` makes the helper check a signature over the bare payload instead, so a
real delivery never verifies without it. Signatures are `sha256=<hex>`, compared
in constant time, and a timestamp more than **300 seconds** from now is rejected.

### Delivery log, replays and the circuit breaker

```rust
use sendly::{BackfillOptions, ListDeliveriesOptions, RedeliverOptions};

// Every delivery attempt for one webhook
let deliveries = client.webhooks()
    .list_deliveries("whk_xxx", Some(ListDeliveriesOptions::new().limit(50)))
    .await?;
for d in &deliveries.data {
    println!("{} -> {} (success: {})", d.event_type, d.http_status, d.success);
}

let delivery = client.webhooks().get_delivery("whk_xxx", "del_xxx").await?;
client.webhooks().retry_delivery("whk_xxx", "del_xxx").await?;

// Repeated failures trip a circuit breaker; close it again before replaying
client.webhooks().reset_circuit("whk_xxx").await?;

// Replay recorded deliveries (default: last 24h, failed + cancelled, limit 1000)
client.webhooks().redeliver("whk_xxx", RedeliverOptions::default()).await?;

// ...or choose the window, event types and statuses
client.webhooks().redeliver("whk_xxx", RedeliverOptions {
    since: Some("2026-09-30T00:00:00Z".to_string()),
    until: Some("2026-10-01T00:00:00Z".to_string()),
    event_types: Some(vec!["message.delivered".to_string()]),
    statuses: Some(vec!["failed".to_string()]),
    limit: Some(500),
}).await?;

// Synthesize events that were never recorded at all
client.webhooks().backfill("whk_xxx", BackfillOptions {
    since: Some("2026-09-30T00:00:00Z".to_string()),
    ..Default::default()
}).await?;
```

`redeliver` and `backfill` both answer HTTP 409 while the circuit is open, so
call `reset_circuit` first. The window can span at most 7 days. `statuses` takes
`failed`, `cancelled`, `delivered` and `pending`. A backfilled event carries the
same event id the original dispatch used, so dedupe on `event.id`, not on
`data.object.id`, which a message's sent and delivered events share.

## Account & Credits

```rust
// Get account information
let account = client.account().get().await?;
println!("Email: {}", account.email);
if let Some(workspace) = &account.organization {
    println!("Workspace: {} ({:?})", workspace.id, workspace.name);
}
if let Some(verification) = &account.business_verification {
    println!("Verification: {:?}", verification.status);
}

// Check credit balance
let credits = client.account().credits().await?;
println!("Available: {} credits", credits.available_balance);
println!("Reserved: {} credits", credits.reserved_credits); // held for scheduled and in-flight sends
println!("Total: {} credits", credits.balance);
println!("Billing: {:?}", credits.billing_mode); // Some("prepaid") or Some("pooled")

// View credit transaction history
use sendly::{ListTransactionsOptions, TransactionType};
let transactions = client.account()
    .transactions(Some(ListTransactionsOptions::new().limit(50).transaction_type(TransactionType::Purchase)))
    .await?;
for tx in &transactions.data {
    // `description` is an Option<String>, so print it with {:?}
    println!("{}: {} credits - {:?}", tx.transaction_type, tx.amount, tx.description);
}

// List API keys
let keys = client.account().api_keys().await?;
for key in &keys {
    println!("{}: {}*** {:?} {:?}", key.name, key.prefix, key.key_type, key.scopes);
}

// Get a specific API key
let key = client.account().get_api_key("key_xxx").await?;

// Usage stats, counted over the key's last 100 requests
let usage = client.account().get_api_key_usage("key_xxx").await?;
println!("{} requests, {} credits used", usage.total_requests, usage.credits_used);
let failed = usage.recent_requests.iter()
    .filter(|r| r.status_code.map_or(false, |s| s >= 400))
    .count();
println!("{} of the last {} failed", failed, usage.recent_requests.len());
for endpoint in &usage.endpoint_breakdown {
    println!("{}: {}", endpoint.endpoint, endpoint.count);
}

// Create a new API key. Without a type the API creates a test key; a live key
// needs a verified business and a credit balance, and can only be granted
// scopes the calling key has.
use sendly::CreateApiKeyRequest;
let new_key = client.account()
    .create_api_key_with_options(
        CreateApiKeyRequest::new("Production Key")
            .key_type("live")
            .scopes(vec!["sms:send", "sms:read"]),
    )
    .await?;
println!("New key {:?}: {}", new_key.id, new_key.key); // the key is shown only once

// Revoke an API key
client.account().revoke_api_key("key_xxx").await?;

// Move credits to another workspace you control
use sendly::TransferCreditsRequest;
let transfer = client.account().transfer_credits(TransferCreditsRequest {
    target_organization_id: "org_xxx".to_string(),
    amount: 500,
}).await?;
println!("{} -> {}", transfer.source_balance, transfer.target_balance);

// Rotate an API key — issues a new key and keeps the old one working for a
// grace period (default 24h, 24-168) so you can roll callers over with no downtime
let rotated = client.account().rotate_api_key("key_xxx").await?;
println!("New key: {}", rotated.new_key.secret); // shown once — store it now!
println!("{}", rotated.message);                 // "Old key will expire in 24 hours"

// ...or with a custom grace period
use sendly::RotateApiKeyRequest;
let rotated = client.account()
    .rotate_api_key_with_options("key_xxx", RotateApiKeyRequest::new().grace_period_hours(72))
    .await?;
```

## Numbers

Discover, buy, and manage the phone numbers you own.

```rust
use sendly::{Sendly, ListAvailableNumbersOptions, BuyNumberRequest, UpdateNumberRequest};

let client = Sendly::new("sk_live_v1_xxx");

// Browse countries and search available numbers (already priced for your account)
let countries = client.numbers().list_countries().await?;
let available = client.numbers()
    .list_available(ListAvailableNumbersOptions::new("US", "local"))
    .await?;

// Buy a number. Asynchronous: the response carries a status, not a finished
// number. `provisioning` means it is being set up — poll list() until it is
// active.
let first = &available.numbers[0];
let result = client.numbers().buy(BuyNumberRequest::new(
    &first.phone_number, &first.country, &first.number_type, &first.monthly_cost,
)).await?;
println!("Buy status: {}", result.status);

// `documents_required` / `payment_required` mean a person has to finish
// something on a hosted page first. Hand them the URL and the short code, wait
// for them to complete it, then call buy() again with the SAME body plus the
// completed action's code.
if let Some(action) = result.action {
    println!("Finish at {} (code {})", action.url, action.code);

    let finished = client.numbers().buy(
        BuyNumberRequest::new(
            &first.phone_number, &first.country, &first.number_type, &first.monthly_cost,
        )
        .action_code(action.action_code.unwrap_or_default()),
    ).await?;
    println!("Buy status: {}", finished.status);
}

// List the numbers you own
let owned = client.numbers().list().await?;
for n in &owned.numbers {
    // None for a number with no recorded price, such as the toll-free number
    // that comes with a verification
    let cents = n.monthly_cost_cents.unwrap_or(0);
    println!("{} {} ({} cents/month)", n.phone_number, n.status, cents);
}

// Get one by id (includes `is_default`, which list omits)
let number = client.numbers().get("num_xxx").await?;
println!("default sender: {:?}", number.is_default);

// Make a number the workspace default sender (the number must be active)
let updated = client.numbers().update("num_xxx", UpdateNumberRequest::new().make_default()).await?;

// Cancel a previously scheduled release ("keep this number")
client.numbers().update("num_xxx", UpdateNumberRequest::new().keep()).await?;

// Release a number. A live paid purchase is cancelled at the end of the paid
// period; everything else is released immediately.
let released = client.numbers().release("num_xxx").await?;
if released.scheduled == Some(true) {
    println!("Releases at {:?}", released.scheduled_release_at);
} else {
    println!("Released");
}
```

## Group MMS

Send a group MMS to 2-8 US/Canada recipients. Every recipient sees the others and
replies fan out to the whole group. Requires an MMS-enabled, 10DLC-registered sender.

```rust
use sendly::{Sendly, SendGroupMessageRequest};

let client = Sendly::new("sk_live_v1_xxx");

let group = client.messages().send_group(
    SendGroupMessageRequest::new(vec![
        "+14155550123".to_string(),
        "+14155550124".to_string(),
    ])
    .with_text("Hey team — quick sync at noon?"),
).await?;

println!("Group message: {} ({})", group.id, group.status);
println!("Group id: {:?}", group.group_message_id);

// A live send lists each recipient with its status; a simulated one leaves
// `recipients` empty. `to` holds the phone numbers either way.
for recipient in &group.recipients {
    println!("{}: {:?}", recipient.phone_number, recipient.status);
}
```

## AI Enhance

Rewrite a draft into a single polished SMS segment, with a short explanation.

```rust
use sendly::{Sendly, EnhanceMessageRequest};

let client = Sendly::new("sk_live_v1_xxx");

let result = client.messages().enhance(
    EnhanceMessageRequest::new()
        .with_text("hey come check out our sale this weekend")
        .with_message_type("marketing"),
).await?;

println!("{}", result.enhanced);
println!("{}", result.explanation);
```

## Links

Mint branded short links, list them with click analytics, and disable an individual
link (a per-link kill switch). Gated behind the `url_shortener` rollout flag — while
the flag is off, these calls resolve as `Error::NotFound`.

```rust
use sendly::{Sendly, ListShortLinksOptions};

let client = Sendly::new("sk_live_v1_xxx");

// Shorten a URL
let link = client.links().create("https://acme.example/spring-sale").await?;
println!("{} -> {}", link.short_url, link.destination_url);

// List your links with click counts
let listing = client.links().list(Some(ListShortLinksOptions::new().limit(50))).await?;
for l in &listing.links {
    println!("{} ({} clicks)", l.short_url, l.click_count);
}

// Disable / re-enable a link (its redirect returns 404 while disabled)
client.links().disable(&link.code).await?;
client.links().enable(&link.code).await?;
```

## Contacts and lists

```rust
use sendly::{
    BulkMarkValidRequest, CheckNumbersRequest, CreateContactListRequest, CreateContactRequest,
    ListContactsOptions, Sendly, UpdateContactRequest,
};

let client = Sendly::new("sk_live_v1_xxx");

let page = client.contacts().list(ListContactsOptions::new().limit(50)).await?;
for contact in &page.contacts {
    println!("{} {:?} ({:?})", contact.phone_number, contact.name, contact.line_type);
}

let contact = client.contacts()
    .create(CreateContactRequest::new("+14155550123").name("Sam Lee"))
    .await?;
client.contacts()
    .update(&contact.id, UpdateContactRequest::new().email("sam@acme.example"))
    .await?;

// A carrier lookup auto-flags landlines and unreachable numbers. Clearing a
// flag by hand sticks — later lookups won't re-flag that contact.
client.contacts().check_numbers(CheckNumbersRequest::default()).await?;
client.contacts().mark_valid(&contact.id).await?;
client.contacts().bulk_mark_valid(BulkMarkValidRequest::of_list_id("list_xxx")).await?;

// Lists
let list = client.contacts().lists().create(CreateContactListRequest::new("VIPs")).await?;
client.contacts().lists().add_contacts(&list.id, vec![contact.id.clone()]).await?;
client.contacts().lists().remove_contact(&list.id, &contact.id).await?;
let lists = client.contacts().lists().list().await?;
println!("{} lists", lists.lists.len());
```

`contacts().import(...)` bulk-imports contacts, and `contacts().delete(id)`
removes one. The `Contact` that `update` returns has no `opted_out` or
`created_at` (the update response does not carry them); call
`contacts().get(id)` when you need those. `check_numbers` is asynchronous: if a
lookup is already running for that scope the response comes back with
`already_running` set; wait for the `contacts.lookup_completed` webhook instead
of polling.

## Conversations

```rust
use sendly::{
    ConversationStatus, GetConversationOptions, ListConversationsOptions,
    ReplyToConversationRequest, Sendly, UpdateConversationRequest,
};

let threads = client.conversations()
    .list(Some(ListConversationsOptions::new().limit(50).status(ConversationStatus::Active)))
    .await?;
for thread in &threads.data {
    println!("{} — {} unread", thread.phone_number, thread.unread_count);
}
println!("more: {}", threads.pagination.has_more);

// One thread, with its messages
let thread = client.conversations()
    .get("conv_xxx", Some(GetConversationOptions::new().include_messages(true).message_limit(50)))
    .await?;
if let Some(messages) = &thread.messages {
    for message in &messages.data {
        println!("{:?}: {}", message.direction, message.text);
    }
}

// Reply, then tidy up. A reply takes text, media_urls or both; leave text
// empty for a media-only reply.
let reply = client.conversations().reply("conv_xxx", ReplyToConversationRequest {
    text: "Thanks for reaching out!".to_string(),
    message_type: None,
    metadata: None,
    media_urls: None,
}).await?;

// Labels: add_labels returns every label the thread now has
let labels = client.conversations()
    .add_labels("conv_xxx", vec!["lbl_xxx".to_string()])
    .await?;
println!("{} labels", labels.len());
client.conversations().remove_label("conv_xxx", "lbl_xxx").await?;

client.conversations().update("conv_xxx", UpdateConversationRequest {
    tags: Some(vec!["vip".to_string()]),
    metadata: None,
}).await?;
client.conversations().mark_read("conv_xxx").await?;
client.conversations().close("conv_xxx").await?;
client.conversations().reopen("conv_xxx").await?;

// AI suggestions for the next reply
let suggestions = client.conversations().suggest_replies("conv_xxx").await?;
for s in &suggestions.suggestions {
    println!("[{}] {}", s.tone, s.text);
}
```

`add_labels` returns a `Vec<Label>` and `remove_label` returns `()`; call
`conversations().get(id, None)` when you need the conversation itself.
`get_context(...)` returns an LLM-ready summary of the thread with a token
estimate.

## Labels and rules

```rust
use sendly::{
    CreateLabelRequest, CreateRuleRequest, RuleActions, RuleConditions, Sendly, UpdateRuleRequest,
};

let label = client.labels()
    .create(CreateLabelRequest::new("Urgent").color("#d92d20").description("Needs a human"))
    .await?;
let labels = client.labels().list().await?;
for l in &labels.data {
    println!("{} {}", l.name, l.color);
}

// Rules act on inbound messages by their AI classification. Every condition
// that is set must hold; a rule with no conditions matches every message.
let rule = client.rules().create(CreateRuleRequest {
    name: "Escalate complaints".to_string(),
    conditions: RuleConditions::new()
        .intent(vec!["complaint", "refund"])
        .sentiment("negative")
        .intent_confidence_min(0.7),
    actions: RuleActions::new().add_labels(vec![label.id.clone()]),
    priority: Some(10),
}).await?;

let rules = client.rules().list().await?;
for rule in &rules.data {
    println!("{} (priority {}, enabled {:?})", rule.name, rule.priority, rule.enabled);
}

// Switch a rule off without deleting it
client.rules().update(&rule.id, UpdateRuleRequest {
    enabled: Some(false),
    ..Default::default()
}).await?;
client.rules().delete(&rule.id).await?;
client.labels().delete(&label.id).await?;
```

`conditions` holds `intent` and `sentiment` (one value or a list, any of which
matches) and `intent_confidence_min` / `sentiment_confidence_min` (0 to 1).
`actions` holds `add_labels` (label ids) and `close_conversation`. A rule stored
as a list by an SDK before 6.0.0 reads back as its first entry; the API never
evaluated such a rule, so recreate it.

## Drafts

An approval queue for outbound replies: create a draft, have someone review it,
then approve or reject. Approving sends the message in the same call and
returns the approved draft with the sent message's `message_id`. If the send is
refused with a 4xx (no credits, an opted-out recipient), the call returns that
error and the draft goes back to pending.

```rust
use sendly::{CreateDraftRequest, DraftStatus, ListDraftsOptions, Sendly};

let draft = client.drafts().create(CreateDraftRequest {
    conversation_id: "conv_xxx".to_string(),
    text: "Sorry about that — here's a refund link.".to_string(),
    media_urls: None,
    metadata: None,
    source: None,
}).await?;

let pending = client.drafts()
    .list(Some(ListDraftsOptions::new().status(DraftStatus::Pending).limit(20)))
    .await?;
println!("{} awaiting review", pending.pagination.total);

let approved = client.drafts().approve(&draft.id).await?;
println!("sent as {:?}", approved.message_id);
client.drafts().reject("draft_xxx", Some("Wrong tone".to_string())).await?;
```

## Templates

```rust
use sendly::{CreateTemplateRequest, ListTemplatesOptions, Sendly, UpdateTemplateRequest};

let template = client.templates()
    .create(CreateTemplateRequest::new("welcome", "Hi {{name}}, welcome to {{company}}!"))
    .await?;
println!("{} v{} published: {}", template.name, template.version, template.is_published());

client.templates().update(&template.id, UpdateTemplateRequest::new().text("Hi {{name}}!")).await?;
client.templates().publish(&template.id).await?;

let copy = client.templates().clone_with_name(&template.id, "welcome-v2").await?;
client.templates().delete(&copy.id).await?;
```

`templates().list(ListTemplatesOptions::new()...)` lists presets and your own
templates, `get(id)` fetches one, `clone(id)` copies one under a generated name,
and `generate(...)` drafts a template from a plain-English description.

On `Template`, prefer `text`, `is_preset`, `variable_specs`, `status` and
`is_published()`. The `body`, `template_type`, `locale`, `variables`,
`is_default` and `is_published` **fields** are deprecated and kept only so older
code still compiles. `body`, `template_type`, `variables` and `is_published` are
still filled in, derived from `text`, `is_preset`, `variable_specs` and `status`;
`locale` and `is_default` stay empty because the API no longer returns them.

## Campaigns

```rust
use sendly::{
    CampaignStatus, CreateCampaignRequest, ListCampaignsOptions, ScheduleCampaignRequest, Sendly,
};

let campaign = client.campaigns().create(CreateCampaignRequest::new(
    "Spring sale",
    "Spring sale starts now — 20% off everything.",
    vec!["list_xxx".to_string()],
)).await?;

// Dry-run the audience and the cost before committing
let preview = client.campaigns().preview(&campaign.id).await?;
println!("{} recipients, ~{} credits", preview.recipient_count, preview.estimated_credits);

// Send now. The result is the batch the campaign created; follow it with
// get_batch(). send_from(id, "+15125550188") sends from a number of yours.
let batch = client.campaigns().send(&campaign.id).await?;
let progress = client.messages().get_batch(&batch.batch_id).await?;
println!("{}/{} sent", progress.sent, progress.total);

// ...or schedule another campaign in a timezone, and cancel it before it goes out
client.campaigns().schedule(
    "camp_xxx",
    ScheduleCampaignRequest::new(
        (chrono::Utc::now() + chrono::Duration::days(1)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    )
    .timezone("America/Chicago"),
).await?;
client.campaigns().cancel("camp_xxx").await?;

let campaigns = client.campaigns()
    .list(ListCampaignsOptions::new().status(CampaignStatus::Completed).limit(20))
    .await?;
for c in &campaigns.campaigns {
    println!("{} {} — {}/{} sent", c.id, c.status, c.sent_count, c.recipient_count);
}
```

`send_from` answers 400 `invalid_from_number` when the workspace cannot send
from that number. A campaign that has sent ends in `CampaignStatus::Completed`
(`Sent` and `Paused` are deprecated: the API never gives a campaign either
status). `update`, `delete`, `get` and `clone(id)` do what they say.

## Verify (OTP)

```rust
use sendly::{
    CreateSessionRequest, ListVerificationsOptions, Sendly, SendVerificationRequest,
    VerificationStatus,
};

// Send a code
let sent = client.verify()
    .send(SendVerificationRequest::new("+14155550123").app_name("Acme").code_length(6))
    .await?;
println!("{} expires {}", sent.id, sent.expires_at);
if sent.sandbox {
    println!("sandbox code: {:?}", sent.sandbox_code); // test keys only
}

// Check it. Only a correct code comes back Ok (status Verified); a wrong code
// is Err(Error::Validation) with code `invalid_code` and the attempts left, an
// expired one Error::Api with status 410, and a wrong code with no attempts
// left Error::RateLimit with code `max_attempts_exceeded` (the verification
// has failed; send a new one).
match client.verify().check(&sent.id, "123456").await {
    Ok(check) if check.status == VerificationStatus::Verified => {
        println!("verified at {:?}", check.verified_at);
    }
    Ok(check) => println!("{:?}", check.status),
    Err(e) if e.code() == Some("invalid_code") => {
        println!("wrong code, {:?} attempts left", e.remaining_attempts());
    }
    Err(e) if e.code() == Some("max_attempts_exceeded") => println!("too many wrong codes"),
    Err(e) => return Err(e),
}

client.verify().resend(&sent.id).await?;
let verification = client.verify().get(&sent.id).await?;
let recent = client.verify().list(ListVerificationsOptions::new().limit(20)).await?;
println!("{} verifications", recent.verifications.len());
```

### Hosted verification sessions

Sendly hosts the phone-entry and code-entry pages; you get a URL to send the
user to and a token to validate afterwards.

```rust
let session = client.verify().sessions().create(
    CreateSessionRequest::new("https://acme.example/verified")
        .cancel_url("https://acme.example/cancelled")
        .brand_name("Acme")
        .brand_color("#0B6E4F"),
).await?;
println!("send the user to {}", session.url);

// On your success URL, validate the token you were handed back
let result = client.verify().sessions().validate("tok_xxx").await?;
println!("{} {:?}", result.valid, result.phone);
```

## Media

Upload a JPEG, PNG or GIF image once, then attach its URL to an MMS.

```rust
use sendly::{Sendly, SendMessageRequest};

let file = client.media().upload("./receipt.png").await?;
println!("{} ({}, {} bytes)", file.url, file.content_type, file.size_bytes);

client.messages().send(
    SendMessageRequest::new("+14155550123", "Your receipt")
        .with_media_urls(vec![file.url.clone()]),
).await?;

// Or upload bytes you already have in memory
let file = client.media().upload_bytes(bytes, "receipt.png", "image/png").await?;
```

`upload` guesses the content type from the file extension (jpg/jpeg, png, gif,
webp, mp4, 3gp, pdf, vcf) and falls back to `application/octet-stream`, but the
API accepts only JPEG, PNG and GIF images: it checks both that content type and
the file's bytes, and rejects anything else. An upload is retried only after
the busy key-check 429 (`too_many_concurrent_verifications`), with the same
file and idempotency key; it is not sent again after a timeout or a network
error.

## 10DLC

Register your business so you can text from US local numbers: a brand, then a
campaign under it, then assign the numbers you own. Both steps are reviewed by
the carriers, so both are poll-until-done.

```rust
use sendly::{CreateTenDlcBrandRequest, CreateTenDlcCampaignRequest, Sendly};

let client = Sendly::new("sk_live_v1_xxx");

// 1) Register the brand, then poll until it is verified
let brand = client.ten_dlc().create_brand(
    CreateTenDlcBrandRequest::new("Acme Coffee LLC")
        .ein("12-3456789")
        .entity_type("PRIVATE_PROFIT")
        .vertical("RETAIL")
        .website("https://acme.example")
        .email("sam@acme.example")
        .street("100 Main St")
        .city("Chicago")
        .state("IL")
        .postal_code("60601")
        .country("US"),
).await?;
println!("{} {}", brand.data.id, brand.data.status); // pending

let refreshed = client.ten_dlc().get_brand(&brand.data.id).await?;
println!("{}", refreshed.data.status); // pending -> verified / failed

// 2) Create a campaign under the verified brand (poll it to `active`)
let campaign = client.ten_dlc().create_campaign(
    CreateTenDlcCampaignRequest::new(
        &brand.data.id,
        "MIXED",
        "Order updates and occasional offers for Acme Coffee customers.",
        "Customers opt in at checkout or by texting JOIN to our number.",
        vec![
            "Your order #4821 is ready for pickup!".to_string(),
            "This week only: 20% off beans. Reply STOP to opt out.".to_string(),
        ],
    )
    .opt_out_keywords("STOP,END,QUIT")
    .help_message("Acme Coffee: email help@acme.example for support."),
).await?;

// 3) Assign a number you own to the active campaign
let assignment = client.ten_dlc()
    .assign_number(&campaign.data.id, "+15125550188")
    .await?;
println!("{} {}", assignment.data.phone_number, assignment.data.status);
```

`ten_dlc().qualify(brand_id, use_case)` pre-checks a use case against a brand
before you create the campaign, and `list_brands()`, `list_campaigns()` and
`list_assignments()` each return their rows on `.data`. Assignment `status` is
`Active`, `Under review` or `Action needed`; the number can send once it is
`Active`.

## Business upgrade

`client.business_upgrade()` drives the sole-proprietor → registered-business
upgrade: `preflight(candidate)` checks the details and reports issues before
you commit, `best_prefill()` returns what Sendly already knows about you,
`start(...)` opens the upgrade (optionally with an EIN document),
`status(workspace_id)` follows it, and `cancel`, `resubmit` and
`set_disposition` handle the rest. `start` answers 202 before the new
toll-free number exists: its `StartUpgradeResponse` has `status` set to
`Some("provisioning")` and `toll_free_number` set to `None`, so follow it with
`status()` for the number. Its types live in the `sendly::business_upgrade`
module.

## WhatsApp

Connect a number you own to WhatsApp ($19 one-time, no monthly fee), create
Meta-reviewed templates, and send. Free-form text and media only deliver inside
an open 24-hour customer-service window; an approved template works anytime.

Sends go through `messages().send_whatsapp()` (`POST /v1/messages` with
channel `whatsapp`) and need `sms:send`, not `whatsapp:write`. Reads
(`signup().get()`, templates, the window, senders and sender profiles) need
`whatsapp:read` and accept test keys. Signup, template create/edit/delete and
profile edits need `whatsapp:write` and a live key (otherwise 403
`whatsapp_requires_live_key`). Sends need a live key too. In a team
workspace, connecting and profile edits need an owner or admin
(`settings:write`), and template writes need an owner, admin or member
(`templates:write`). A missing role returns 403 `insufficient_permissions`.

WhatsApp is enabled per person: the user who owns the API key, not the
workspace. While it is off, sends return 403 `whatsapp_not_enabled` and the
`/api/v1/whatsapp/*` management routes return 404 `not_found`.

After the person behind the connect URL links their WhatsApp Business
Account, the signup moves to `Registering`, then `Active`. Activation usually
takes a few minutes but can take hours. If it hasn't finished about 6 hours
after the session began, the session fails with `registration_timeout` and
the fee is refunded. If the connection fails, the $19 fee is refunded
automatically; once a number has connected, a later disconnect gets nothing
back.

Pricing: free-form text or media inside the 24-hour window costs 1 credit
each for the first 1,000 per sending number per calendar month (UTC), then
the destination's utility template price; countries without a listed price
use the default utility price of 12 credits. Templates are priced by category
and destination country; countries without a listed price use 33
(marketing), 12 (utility) and 12 (authentication) credits. A failed send
gives its slot back.

```rust
use sendly::{
    Sendly, SendWhatsAppMessageRequest, CreateWhatsAppTemplateRequest,
    WhatsAppTemplateCategory, WhatsAppTemplateSendParams,
};
use std::collections::HashMap;

let client = Sendly::new("sk_live_v1_xxx");

// Connect a number — the connect URL must be opened by a human, who logs in
// with Facebook to link their WhatsApp Business Account
let signup = client.whatsapp().signup().create("+15125550188").await?;
println!("Have your user open: {}", signup.connect_url);

// Poll until active: Initiated -> Registering (minutes to hours) -> Active
let status = client.whatsapp().signup().get(&signup.id).await?;
println!("{:?} {:?}", status.status, status.failure_reasons);

// List your WhatsApp senders
let senders = client.whatsapp().senders().list().await?;
for s in &senders.senders {
    println!("{} — {:?}", s.phone_number, s.status);
}

// Create a template (Meta reviews it, usually 24-48h)
let mut examples = HashMap::new();
examples.insert("1".to_string(), "Sam".to_string());
examples.insert("2".to_string(), "#4821".to_string());
let template = client.whatsapp().templates().create(
    CreateWhatsAppTemplateRequest::new(
        "+15125550188",
        "order_shipped",
        "en_US",
        WhatsAppTemplateCategory::Utility,
        "Hi {{1}}, your order {{2}} has shipped!",
    )
    .examples(examples),
).await?;

// Check the 24-hour window, then send. The response is exactly
// { open, expiresAt }: no window on record gives open false and expires_at
// None; an expired one gives open false and the past expiry.
let window = client.whatsapp().window("+15125550188", "+14155550123").await?;

if window.open {
    // Free-form text (or media with a caption via .with_media_urls())
    client.messages().send_whatsapp(
        SendWhatsAppMessageRequest::new("+14155550123", "+15125550188")
            .with_text("Your table is ready!"),
    ).await?;
} else {
    // Approved template — works regardless of the window
    let mut variables = HashMap::new();
    variables.insert("1".to_string(), "Sam".to_string());
    variables.insert("2".to_string(), "#4821".to_string());
    let message = client.messages().send_whatsapp(
        SendWhatsAppMessageRequest::new("+14155550123", "+15125550188")
            .with_template(
                WhatsAppTemplateSendParams::new("order_shipped", "en_US")
                    .with_variables(variables),
            ),
    ).await?;
    println!("Kind: {:?}, {} credits", message.whatsapp.kind, message.credits_used);
}
```

How the WhatsApp refusals arrive:

- `signup().create()` answers 503 `whatsapp_unavailable` (`Error::Api`) when
  connections are paused. Only signup returns it; no send does. Nothing is
  charged; its body carries `retryAfter` (3600 seconds), read with
  `err.body()`, and the response a `Retry-After: 3600` header. It answers 429
  `whatsapp_signup_limit_reached` (`Error::RateLimit`, not retryable) after too
  many failed connections in 24 hours.
- A send the carrier refuses is 422 `whatsapp_send_failed`
  (`Error::Validation`): final, do not retry; the API caches it under the
  idempotency key and replays it for 24 hours. A send that could not reach
  the carrier is 502 `whatsapp_send_failed` (`Error::Api`): not sent, safe to
  send again. It is never cached and the client does not retry it, so send it
  again under the same idempotency key; pass your own through
  `send_whatsapp_with_options`. A failed send is refunded either way.
- A send whose outcome is unknown is 409 `whatsapp_send_unconfirmed`
  (`Error::Api`): the message was marked failed and refunded, but it may still
  be delivered, so check before sending it again or it could arrive twice. It
  is cached under the idempotency key, and the client does not retry it.
- `templates().create()` answers 404 `whatsapp_sender_not_connected` first
  when the sender isn't connected, then 400 (`Error::Validation`) for a
  pre-flight refusal: `template_category_invalid`,
  `template_authentication_otp_button_required`,
  `template_authentication_no_links` (a link in the body or a URL button on an
  authentication template) or `template_header_variable_unsupported`. A
  marketing template without an opt-out button only gets a warning.

Every connected sender has a WhatsApp Business profile — the name, photo, and
business details recipients see when they tap your number. Send only the fields
you want to change (`about` is capped at 139 characters, `description` at 512):

```rust
use sendly::UpdateWhatsAppSenderProfileRequest;

let profile = client.whatsapp().senders().get_profile("+15125550188").await?;
println!("{:?}", profile.display_name);

let updated = client.whatsapp().senders().update_profile(
    "+15125550188",
    UpdateWhatsAppSenderProfileRequest::new()
        .about("Fresh bread, daily.")
        .description("Family bakery in Austin since 1998.")
        .email("hello@acme.example")
        .website("https://acme.example"),
).await?;
```

Upload a profile photo (a JPEG or PNG of at most 5 MB, sent as the multipart
field `file`; WhatsApp wants it square and at least 192 pixels wide, 640
recommended) or remove it. Both return the updated profile:

```rust
let profile = client.whatsapp().senders()
    .upload_profile_photo("+15125550188", "logo.png")
    .await?;
println!("{:?}", profile.profile_photo_url);

client.whatsapp().senders().delete_profile_photo("+15125550188").await?;
```

A photo that isn't a JPEG or PNG is 400 `whatsapp_profile_photo_invalid`
(`Error::Validation`), one over 5 MB is 413 `whatsapp_profile_photo_too_large`
(`Error::Api`), and 502 `whatsapp_profile_update_failed` means WhatsApp refused
it or couldn't be reached: fix the image and try again.

Ice breakers are tappable suggestions shown when someone opens a chat with the
business for the first time (up to 4, each at most 80 characters); commands
are shown when the customer types "/" (up to 30; the command is letters,
digits and underscores, at most 32 characters, and the description at most
256). Each list you send replaces the stored one, an empty list clears it, and
a list you leave out is kept. A list over the limits is 400 `invalid_request`
(`Error::Validation`) with a message that says which:

```rust
use sendly::{UpdateWhatsAppConversationalComponentsRequest, WhatsAppCommand};

let current = client.whatsapp().senders()
    .get_conversational_components("+15125550188")
    .await?;
println!("{:?} {:?}", current.ice_breakers, current.commands);

client.whatsapp().senders().update_conversational_components(
    "+15125550188",
    UpdateWhatsAppConversationalComponentsRequest::new()
        .ice_breakers(vec!["Book a repair", "Get a quote"])
        .commands(vec![WhatsAppCommand::new("quote", "Get a price for a job")]),
).await?;
```

Turn on WhatsApp calling and a WhatsApp user calling the number rings like a
phone call, in the dashboard or on the number's AI agent. Voice must be on for
the number first (otherwise 409 `voice_not_enabled`), and WhatsApp only allows
it once the account may message at least 2,000 people a day and the display
name is approved (otherwise 422 `whatsapp_calling_unavailable`). Calls placed
to WhatsApp users are dashboard only; there is no API for them.

```rust
let calling = client.whatsapp().senders().set_calling("+15125550188", true).await?;
println!("on: {}, may call out: {}", calling.calling_enabled, calling.outbound_calling_allowed);
```

`senders().list()` reports each sender's `business_account_id`,
`business_name`, `calling_enabled` and `outbound_calling_allowed`.
`outbound_calling_allowed` is false for every +1 number and for +20, +84 and
+234 numbers, where WhatsApp doesn't allow business-initiated calls.

### Add a number by code

Once one number is connected, add more numbers to the same WhatsApp Business
account without the Facebook step. Pass the account's `business_account_id`
(from `senders().list()`); WhatsApp sends a 6-digit code to the new number by
text (the default) or voice call. It costs the same $19 one-time fee, refunded
automatically if the connection fails.

```rust
use sendly::{CreateWhatsAppSignupRequest, WhatsAppSignupStatus, WhatsAppVerificationMethod};

let signup = client.whatsapp().signup().create_with_options(
    CreateWhatsAppSignupRequest::new("+15125550142")
        .business_account_id("104996582519384")
        .verification_method(WhatsAppVerificationMethod::Sms),
).await?;
assert_eq!(signup.status, WhatsAppSignupStatus::Verifying);

// The code arrives as a text on the number; get() picks it up for you.
let status = client.whatsapp().signup().get(&signup.id).await?;
if let Some(code) = status.verification_code {
    let active = client.whatsapp().signup().verify(&signup.id, &code).await?;
    println!("{:?}", active.status); // Active
}

// No code yet? Ask for another one, at least 30 seconds after the last change.
client.whatsapp().signup()
    .resend(&signup.id, Some(WhatsAppVerificationMethod::Voice))
    .await?;
```

- `create_with_options` answers 404 `whatsapp_business_account_not_found`
  when the account isn't connected in this workspace, 400
  `display_name_required` when no display name can be found (pass
  `.display_name(...)`), and `whatsapp_verification_start_failed` when
  WhatsApp would not send the code: 422 when it refused (final) or 502 when it
  couldn't be reached (start again). The session fails and the fee is refunded
  either way. Calling it again for a number that is already verifying returns
  the same session without charging again.
- `verify` answers 422 `whatsapp_verification_code_invalid` for a wrong code
  (`err.remaining_attempts()` says how many tries are left), 409
  `whatsapp_verification_failed` on the fifth wrong code (the signup fails and
  the fee is refunded), 409 `whatsapp_verification_busy` while another code is
  being checked, 502 `whatsapp_verification_unavailable` when WhatsApp
  couldn't be reached (not counted; try again) and 502
  `whatsapp_activation_pending` when the code was accepted but the connection
  isn't finished yet.
- `resend` answers 429 `whatsapp_verification_resend_too_soon`
  (`Error::RateLimit`; `err.retry_after()` says how long to wait) and 422 or
  502 `whatsapp_verification_resend_failed`. `verify` and `resend` answer 409
  `signup_not_active` once the signup has failed or is more than 3 hours old.
- A signup that fails while verifying has `failure_reasons` such as
  `verification_start_failed`, `verification_failed` or
  `verification_expired`, or `setup_fee_payment_failed` if the fee could not
  be charged; treat the list as open. The `whatsapp_account.connected` and
  `whatsapp_account.failed` webhooks fire for numbers added by code too.

Templates are managed on their own sub-resource:

```rust
use sendly::UpdateWhatsAppTemplateRequest;

let templates = client.whatsapp().templates().list().await?;
for t in &templates.templates {
    println!("{} {} {:?} ({:?})", t.name, t.language, t.status, t.rejection_reason);
}

// Editing a REJECTED template resubmits it — that's the recovery path, because
// Meta locks a deleted template's name for about 30 days.
client.whatsapp().templates()
    .update("tpl_xxx", UpdateWhatsAppTemplateRequest::new().body("Hi {{1}}, your order {{2}} shipped."))
    .await?;
client.whatsapp().templates().delete("tpl_xxx").await?;
```

Template `status` is `Pending`, `Approved`, `Rejected`, `Paused` or `Disabled`,
and `category` is `Authentication`, `Utility` or `Marketing`. Both pass through
from WhatsApp as they come, so a value this SDK does not know (such as
`IN_APPEAL`) decodes as `Unknown` and the rest of the list is kept; match with
a `_` arm. Meta may reclassify a template, and the category on the record is
what drives a template's price. The category is required on create, with no
default, and an update can't change it. A template `header` is fixed text: it cannot
contain `{{n}}` variables (the API refuses one with `Error::Validation`, code
`template_header_variable_unsupported`), so put variables in the body. Note
that Meta has paused delivery of marketing templates to US (+1) numbers.

## RCS

RCS is the branded, rich upgrade to SMS: your verified agent name and logo
instead of a bare number, plus tappable suggestion chips and rich cards, on
Android and iOS 18+ handsets. Messages go out through an RCS agent (the
verified identity recipients see). Registration is self-serve, from the
dashboard or the API: draft a brand and an agent, submit them for review
(Sendly reviews first, then the carrier network), test on invited devices,
then request launch. Sending requires a live API key.

Text sends fall back to plain SMS automatically when the recipient's device or
network doesn't support RCS, so one call covers your whole list. The fallback is
billed as SMS and is visible on the response — `fell_back_to_sms()` is the
direct check. Cards have no SMS form and never fall back.

### Registering an agent

Reads need the `rcs:read` scope and writes `rcs:write`. Every brand and agent
field is optional while drafting; required-field checks run at `submit`, which
lists each gap in the 422 `rcs_invalid_content` response. Logo, hero, and
call-to-action media must be public `https://` URLs; uploading assets is
dashboard-only. RCS registration is available to US businesses for now. Every
registration write carries an `Idempotency-Key` (generated per call, or your
own through the `*_with_options` variants). While RCS registration isn't enabled
for an account, these calls answer 404 (`Error::NotFound`).

```rust
use sendly::{
    CreateRcsAgentRequest, IdempotentRequestOptions, RcsAgentBasicsInput, RcsBrandAddressInput,
    RcsBrandContactInput, RcsCampaign, RcsConsentSettings, RcsCustomerStage, RcsInteraction,
    RcsOptInMethod, RcsRequestLaunchRequest, RcsTestDeviceInput, Sendly, UpdateRcsAgentRequest,
};

let client = Sendly::new("sk_live_v1_xxx");

// 1. Draft a brand - prefill it from business details already on file
let dossier = client.rcs().dossier().get().await?;
let brand = client
    .rcs()
    .brands()
    .create(
        dossier
            .brand
            .display_name("Acme Coffee")
            .legal_name("Acme Coffee LLC")
            .legal_entity_type("LIMITED_LIABILITY_COMPANY")
            .organization_type("PRIVATE_PROFIT")
            .website_url("https://acme.example")
            .ein("12-3456789")
            .address(
                RcsBrandAddressInput::new()
                    .line1("100 Main St")
                    .city("Chicago")
                    .state("IL")
                    .postal_code("60601")
                    .country_code("US"),
            )
            .contact(
                RcsBrandContactInput::new()
                    .first_name("Sam")
                    .last_name("Lee")
                    .email("sam@acme.example")
                    .phone_number("+13125550100"),
            ),
    )
    .await?
    .brand;

// 2. Draft the agent recipients will see
let agent = client
    .rcs()
    .agents()
    .create(
        CreateRcsAgentRequest::new(&brand.id)
            .display_name("Acme Coffee")
            .use_case("MULTI_USE")
            .basics(
                RcsAgentBasicsInput::new()
                    .description("Order updates and support for Acme Coffee customers")
                    .logo_url("https://acme.example/rcs/logo.png") // public https URL
                    .hero_url("https://acme.example/rcs/hero.png")
                    .brand_color("#0B6E4F")
                    .privacy_policy_url("https://acme.example/privacy")
                    .terms_and_conditions_url("https://acme.example/terms"),
            ),
    )
    .await?
    .agent;

// 3. Submit for review - your own idempotency key means a retry never re-notifies reviewers
let review = client
    .rcs()
    .agents()
    .submit_with_options(
        &agent.id,
        IdempotentRequestOptions::new().idempotency_key(format!("rcs-submit-{}", agent.id)),
    )
    .await?;
println!("{}", review.stage); // in_review

// Poll for progress: in_review -> brand_verification -> agent_review -> testing -> ...
let current = client.rcs().agents().get(&agent.id).await?;
println!("{} {:?}", current.stage, current.agent.review_note);

// 4. Once the stage is testing: invite your devices and fill in the campaign
if current.stage == RcsCustomerStage::Testing {
    client
        .rcs()
        .agents()
        .set_test_devices(
            &agent.id,
            vec![RcsTestDeviceInput::new("+13125550100").label("Sam's Pixel")],
        )
        .await?;
    client
        .rcs()
        .agents()
        .update(
            &agent.id,
            UpdateRcsAgentRequest::new().campaign(
                RcsCampaign::new()
                    .agent_overview("Order confirmations, pickup alerts, and support replies")
                    .interactions(vec![RcsInteraction::new("TRANSACTIONAL_UPDATES", "Order status")])
                    .message_examples(vec![
                        "Your order #4821 is being roasted.".to_string(),
                        "Your order #4821 is ready for pickup!".to_string(),
                        "Thanks for visiting. Reply HELP for support.".to_string(),
                    ])
                    .consent_settings(
                        RcsConsentSettings::new()
                            .opt_in_methods(vec![RcsOptInMethod::new("WEBSITE", "Checkout checkbox")])
                            .call_to_action("Text me order updates")
                            .call_to_action_url("https://acme.example/checkout")
                            .opt_in_message("Welcome to Acme Coffee updates. Reply STOP to opt out.")
                            .help_response("Acme Coffee: email help@acme.example for support.")
                            .opt_out_response("You have been unsubscribed from Acme Coffee updates."),
                    ),
            ),
        )
        .await?;

    // 5. Send a test message to an invited device, then request launch
    let launch = client
        .rcs()
        .agents()
        .request_launch(
            &agent.id,
            Some(RcsRequestLaunchRequest::new().test_url("https://acme.example/rcs-test")),
        )
        .await?;
    println!("{}", launch.stage); // launch_review
}

// The whole registration at a glance
let registration = client.rcs().registration().get().await?;
println!("{} (US eligible: {})", registration.stage, registration.us_eligible);
```

### Sending

```rust
use sendly::{RcsCard, RcsSuggestion, SendRcsMessageRequest, Sendly};

let client = Sendly::new("sk_live_v1_xxx");

// Find the agents you can send as
let agents = client.rcs().agents().list().await?;
for agent in &agents.agents {
    println!("{}: {} (sendable: {})", agent.id, agent.name, agent.sendable);
}

// Optional pre-flight — sending handles the fallback on its own.
// Pass Some(agent_id) when the workspace has more than one agent.
let capability = client.rcs().capability("+14155550123", None).await?;
println!("capable: {} {:?}", capability.capable, capability.features);

// Text with tappable chips
let message = client.messages().send_rcs(
    SendRcsMessageRequest::new("+14155550123")
        .with_text("Your order #4821 has shipped!")
        .with_suggestions(vec![
            RcsSuggestion::reply("Track it", "track_4821"),
            RcsSuggestion::action(
                "View receipt",
                "receipt_4821",
                "https://acme.example/receipts/4821",
            ),
        ]),
).await?;

if message.fell_back_to_sms() {
    // Delivered as SMS — chips have no SMS form and were dropped
    println!("fell back to SMS: {}", message.rcs.suggestions_dropped);
} else {
    println!("delivered over RCS from {:?}", message.rcs.agent_name);
}

// A rich card — an unsupported recipient gets a 422
// (rcs_not_supported_for_recipient) rather than an SMS
let card = client.messages().send_rcs(
    SendRcsMessageRequest::new("+14155550123").with_card(
        RcsCard::new(
            "Your table is ready",
            "Head to the host stand — we'll hold it for 10 minutes.",
        )
        .with_media_url("https://acme.example/table.jpg")
        .with_suggestions(vec![RcsSuggestion::reply("On my way", "otw")]),
    ),
).await?;
println!("Kind: {:?}", card.rcs.kind);

// Require RCS delivery — turn the fallback off
client.messages().send_rcs(
    SendRcsMessageRequest::new("+14155550123")
        .with_text("RCS only.")
        .with_fallback_to_sms(false),
).await?;
```

### Registration stages

`RcsCustomerStage` is what you poll — on `registration().get().stage`, on
`agents().get(id).stage`, and on the review responses:

| Stage | Meaning |
|-------|---------|
| `Draft` | Being filled in; nothing submitted |
| `InReview` | Submitted; Sendly is reviewing it |
| `ChangesRequested` | Sendly asked for changes — see `review_note`, edit, resubmit |
| `Rejected` | Sendly declined it (see `review_note`) |
| `BrandVerification` | Approved by Sendly; the carrier network is verifying the brand |
| `AgentReview` | Brand verified; the carrier network is reviewing the agent |
| `Testing` | Approved for invited test devices; fill in the campaign, then request launch |
| `LaunchReview` | Launch requested; Sendly is reviewing it |
| `Launching` | Sendly asked the carrier network to launch the agent |
| `LaunchRejected` | The carrier network declined the launch (see `rejection_reason`) |
| `Live` | Launched; the agent can reach every RCS-capable recipient |
| `Suspended` | Sending is currently suspended |
| `Failed` | Registration failed (see `rejection_reason`) |
| `Unknown` | A stage this SDK build doesn't know yet |

`RcsReviewStatus` is the finer-grained review state Sendly tracks on a brand or
agent (`Draft`, `AwaitingReview`, `ChangesRequested`, `ApprovedForCarrier`,
`Rejected`, `LaunchRequested`, `LaunchSubmitted`, `LaunchRejected`, `Failed`,
`Unknown`), and the agent's own `status` — `draft`, `submitted`, `testing`,
`approved`, `suspended` — is what decides whether it can send. A `testing`
agent only reaches invited test devices.

`brands().update(id, RcsBrandInput)` edits a brand draft, and every registration
write has a `*_with_options` twin that takes your own idempotency key.

## Short codes

This SDK has **no short-code helpers** — short-code programs are applied for and
managed in the dashboard and over REST. The only short-code surface here is the
webhook events, which `Webhooks::parse_event` decodes like any other lifecycle
event: `short_code.action_required`, `short_code.rejected`, `short_code.filed`
and `short_code.live`. Their payload is on `event.object` (`event.data` is
`None`).

The REST endpoints take the same `Authorization: Bearer <key>` header this
client uses:

| Method | Path | Scope |
|--------|------|-------|
| `GET` | `/api/v1/short_codes` | `short_codes:read` |
| `POST` | `/api/v1/short_codes/requests` | `short_codes:write` |
| `GET` | `/api/v1/short_codes/application` | `short_codes:read` |
| `PUT` | `/api/v1/short_codes/application` | `short_codes:write` |
| `POST` | `/api/v1/short_codes/application/preflight` | `short_codes:read` |
| `POST` | `/api/v1/short_codes/application/submit` | `short_codes:write` |

## Voice Calls

Place phone calls that one of your AI agents handles, follow them, end one
early, and fetch recordings. Calls go to US and Canadian numbers from a
workspace number that is voice-enabled and has an emergency address
registered. Set up numbers and agents with `client.voice()` (see
[Configure voice](#configure-voice)) or in the dashboard. Reads need the
`calls:read` scope and writes `calls:write`; placing or ending a call also
needs a live API key.

Calls are billed per started minute, prepaid from your credit balance: 2
credits/min outbound plus 8 credits/min while an AI agent is on the line, so an
agent-handled call is 10 credits/min ($0.10). Unanswered calls cost nothing.
Voice is being enabled workspace by workspace; until it is on for yours, these
calls answer 404 `voice_not_enabled` (`Error::NotFound`).

```rust
use sendly::{CallRecordingStatus, CallStatus, CreateCallRequest, ListCallsOptions, Sendly};

let client = Sendly::new("sk_live_v1_xxx");

// Find a number to call from
let numbers = client.voice().numbers().list().await?;
let from = numbers.data.iter().find(|n| n.voice_enabled).expect("a voice-enabled number");

// Place a call handled by an agent (returns while it is still ringing)
let call = client
    .calls()
    .create(
        CreateCallRequest::new("+15125550123", "3c4d5e6f-7081-4293-a4b5-c6d7e8f90a1b")
            .from_number(&from.phone_number)
            .context("You are calling Jordan to confirm the 3pm appointment on Tuesday.")
            .metadata_entry("crmId", "lead_8812"),
    )
    .await?;
println!("{} {}", call.id, call.status); // ... ringing

// Follow it: agent calls carry a transcript on get()
let call = client.calls().get(&call.id).await?;
println!("{} {:?} {} credits", call.status, call.hangup_class, call.credits_charged);
for line in call.transcript.unwrap_or_default() {
    println!("[{}ms] {}: {}", line.at_ms, line.speaker, line.text);
}

// List completed outbound calls, newest first
let page = client
    .calls()
    .list(Some(ListCallsOptions::new().status(CallStatus::Completed).limit(20)))
    .await?;
println!("{} of {} (more: {})", page.data.len(), page.pagination.total, page.pagination.has_more);

// End a call early (ringing -> cancelled, active -> completed; ended calls come back unchanged)
client.calls().hangup(&call.id).await?;

// Fetch the recording: the URL is signed and valid for five minutes
let recording = client.calls().recording(&call.id).await?;
if recording.status == CallRecordingStatus::Ready {
    println!("{} until {}", recording.url.unwrap(), recording.expires_at.unwrap());
}
```

Refusals map onto the SDK's error variants: 402 `insufficient_credits` is
`Error::InsufficientCredits`; 404s (`voice_not_enabled`, `call_not_found`,
`agent_not_found`, `number_not_found`) are `Error::NotFound`; 400s
(`agent_required`, `invalid_number`, `invalid_metadata`, `from_number_required`,
`from_number_not_supported` for a from number outside the US and Canada,
`destination_not_supported`) are `Error::Validation`; 429s (`rate_limit_exceeded`,
and `daily_call_limit`, which lasts until tomorrow and is not retryable) are
`Error::RateLimit`; everything else arrives as `Error::Api` with `code` set, so
you can match on it:

```rust
use sendly::Error;

match client.calls().create(request).await {
    Ok(call) => println!("ringing {}", call.id),
    Err(Error::InsufficientCredits { message, .. }) => eprintln!("top up first: {}", message),
    Err(Error::Api { code: Some(code), message, .. }) => match code.as_str() {
        "e911_required" => eprintln!("register an emergency address for the from number first: {}", message),
        "lines_busy" => eprintln!("retry shortly: {}", message),
        "agent_disabled" | "no_voice_number" | "live_key_required" => eprintln!("{}: {}", code, message),
        _ => eprintln!("{}: {}", code, message),
    },
    Err(e) => eprintln!("{}", e),
}
```

### Call object

```rust
call.id                // String (uuid)
call.kind              // CallKind: Pstn | Internal
call.channel           // Option<CallChannel>: Phone | WhatsApp | Browser
call.direction         // CallDirection: Inbound | Outbound
call.status            // CallStatus: Ringing | Active | Completed | NoAnswer | Busy | Cancelled | Declined | Failed | Suspended
call.handled_by        // CallHandledBy: Agent | Dashboard
call.agent_id          // Option<String>
call.from_number       // Option<String> (E.164; None on internal calls)
call.to                // Option<String>
call.caller_name       // Option<String>
call.callee_name       // Option<String>
call.started_at        // String (ISO 8601)
call.answered_at       // Option<String>
call.ended_at          // Option<String>
call.duration_secs     // i64 (answered seconds; 0 until ended)
call.credits_charged   // i64
call.billing           // CallBilling: Metered | Settled | Unbilled
call.hangup_class      // Option<String> (why it ended, e.g. "normal", "ring_timeout", "credits_exhausted")
call.recording_status  // Option<CallRecordingStatus>: Recording | Ready | Failed
call.metadata          // HashMap<String, String>
call.transcript        // Option<Vec<CallTranscriptLine>> (get() on agent calls only)

// Helper methods
call.is_live()   // ringing or active
call.is_ended()  // reached a terminal status
```

Every enum has an `Unknown` fallback, so a value added by a later API release
never fails decoding.

`channel` says how the call travelled: the phone network, WhatsApp or the
browser. The `call.started`, `call.completed` and `call.recording.ready`
webhooks carry it too, at `event.object["channel"]` (decode it with
`event.object_as()` into a struct with a `CallChannel` field). Inbound WhatsApp
calls read `Phone` until WhatsApp calls are told apart on the inbound line.

### Configure voice

Set up everything a call depends on from code with `client.voice()`: switch
voice on for a number and choose how it answers, register the number's
emergency address, and create the AI agents that talk on calls. A number is
addressed by its id or its E.164 phone number. Reads need `calls:read`; writes
need `calls:write` and a live API key. In a team workspace, changing a number
also needs a role that can change settings, and managing agents a role that
can manage API keys, because each agent holds its own scoped sending key.

```rust
use sendly::{
    CreateVoiceAgentRequest, RegisterEmergencyAddressRequest, Sendly, UpdateVoiceAgentRequest,
    UpdateVoiceNumberRequest, VoiceAgentToolsInput, VoiceMode,
};

let client = Sendly::new("sk_live_v1_xxx");

// The voices an agent can speak with
let voices = client.voice().voices().list().await?;
for voice in &voices.data {
    println!("{}: {} ({})", voice.id, voice.label, voice.language);
}

// Create an agent (up to 20 per workspace); it answers real callers on any number pointed at it
let agent = client
    .voice()
    .agents()
    .create(
        CreateVoiceAgentRequest::new("Front desk")
            .voice("ashley")
            .greeting("Thanks for calling Acme, how can I help?")
            .instructions("Answer questions about opening hours and take a message for anything else.")
            .tools(VoiceAgentToolsInput::new().send_sms(true)),
    )
    .await?;
println!("{} can text callers: {}", agent.id, agent.can_send_sms);

// Change it later: only the fields you set are sent
client
    .voice()
    .agents()
    .update(&agent.id, UpdateVoiceAgentRequest::new().greeting("Thanks for calling Acme. How can I help today?"))
    .await?;

// Register the emergency address: a US or Canadian number needs one before it
// can place calls, and the first registration adds $1.50 a month to the number
let number = client
    .voice()
    .numbers()
    .register_emergency_address(
        "+15125550188",
        RegisterEmergencyAddressRequest::new("500 Example Ave", "Austin", "TX", "78701").unit("Suite 2"),
    )
    .await?;
println!("{:?}", number.emergency_address.map(|e| e.status)); // Some("provisioning") or Some("active")

// Have the agent answer the number. This changes how real calls to it are answered.
let number = client
    .voice()
    .numbers()
    .update(
        "+15125550188",
        UpdateVoiceNumberRequest::new()
            .voice_enabled(true)
            .voice_mode(VoiceMode::Agent)
            .agent_id(&agent.id),
    )
    .await?;

// Ring the team in the dashboard instead (VoiceMode::None switches voice off)
client
    .voice()
    .numbers()
    .update(&number.id, UpdateVoiceNumberRequest::new().voice_mode(VoiceMode::RingDashboard))
    .await?;

// Every active number with its voice settings and per-minute rates in credits
for n in client.voice().numbers().list().await?.data {
    println!("{} {} {:?} {} credits/min out", n.phone_number, n.voice_mode, n.agent_id, n.rate_per_minute.outbound);
}
```

Refusals map the same way as for calls: `number_not_found` and
`agent_not_found` are `Error::NotFound`; `invalid_request`,
`invalid_voice_mode`, `agent_required`, `e911_not_applicable` and
`invalid_address` (400 for a malformed field, 422 when the address couldn't be
validated) are `Error::Validation`; `forbidden`, `live_key_required`,
`agent_disabled`, `agent_limit`, `agent_in_use`, `voice_attach_failed`,
`carrier_refused` and `voice_unavailable` arrive as `Error::Api` with `code`
set. An agent that still answers a number can't be deleted, so move those
numbers first:

```rust
use sendly::{Error, UpdateVoiceNumberRequest, VoiceMode};

match client.voice().agents().delete(&agent.id).await {
    Ok(deleted) => println!("deleted {}", deleted.id),
    Err(Error::Api { code: Some(code), .. }) if code == "agent_in_use" => {
        for n in client.voice().numbers().list().await?.data {
            if n.voice_mode == VoiceMode::Agent && n.agent_id.as_deref() == Some(agent.id.as_str()) {
                client
                    .voice()
                    .numbers()
                    .update(&n.id, UpdateVoiceNumberRequest::new().voice_mode(VoiceMode::RingDashboard))
                    .await?;
            }
        }
        client.voice().agents().delete(&agent.id).await?;
    }
    Err(e) => return Err(e),
}
```

The number and agent objects:

```rust
number.id                 // String (uuid)
number.phone_number       // String (E.164)
number.phone_number_type  // Option<String>
number.country_code       // Option<String>
number.is_default         // bool
number.voice_enabled      // bool
number.voice_mode         // VoiceMode: None | RingDashboard | Agent
number.agent_id           // Option<String>
number.emergency_address  // Option<VoiceNumberEmergencyAddress> { status, address: Option<EmergencyAddress> }
number.rate_per_minute    // VoiceNumberRates { inbound, outbound, agent } (credits per started minute)

agent.id                  // String (uuid)
agent.name                // String
agent.enabled             // bool
agent.voice               // String (a voice id from voices().list())
agent.voice_label         // String
agent.language            // String
agent.greeting            // String ("" when unset; said when it answers an inbound call)
agent.instructions        // String
agent.tools               // VoiceAgentTools { send_sms, transfer_to }
agent.can_send_sms        // bool (the agent holds its own scoped sending key)
agent.calls_handled       // i64
agent.avg_duration_secs   // i64
agent.created_at          // String (ISO 8601)
agent.updated_at          // String (ISO 8601)
```

`tools.transfer_to` does not transfer calls yet: while it is set, a caller who
asks for a person is told the message will be passed on, and the agent takes
their name and number.

Reads round out the same two sub-resources: `voice().numbers().get(number)`
takes a number's id or its E.164 form (the SDK percent-encodes it), and
`voice().agents().get(id)` and `voice().agents().list()` fetch one agent or all
of them with their call stats.

## Error Handling

```rust
use sendly::{Error, Sendly, SendMessageRequest};

match client.messages().send(request).await {
    Ok(message) => {
        println!("Sent: {}", message.id);
    }
    Err(Error::Authentication { message, .. }) => {
        eprintln!("Invalid API key: {}", message);
    }
    Err(Error::RateLimit { message, retry_after, code, .. }) => {
        eprintln!("Rate limited ({:?}): {}", code, message);
        if let Some(seconds) = retry_after {
            eprintln!("Retry after: {} seconds", seconds);
        }
    }
    Err(Error::InsufficientCredits { message, .. }) => {
        eprintln!("Add more credits: {}", message);
    }
    Err(Error::Validation { message, code, .. }) => {
        eprintln!("Invalid request ({:?}): {}", code, message);
    }
    Err(Error::NotFound { message, .. }) => {
        eprintln!("Not found: {}", message);
    }
    Err(Error::Network { message }) => {
        eprintln!("Network error: {}", message);
    }
    Err(Error::Timeout) => {
        eprintln!("Request timed out");
    }
    Err(Error::Api { message, status_code, code, .. }) => {
        eprintln!("API error {} ({:?}): {}", status_code, code, message);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

`Error` is `#[non_exhaustive]`, so a `match` on it needs a `_` arm. Its
variants gained `code` and `body` fields in 6.0.0; end each pattern in `..` so
it names only the fields it uses. The API's 400 and 422 both arrive as
`Error::Validation`; tell them apart by `code`.

Every error the API returned keeps its machine-readable code and JSON body,
whatever the variant: `err.code()` returns the code (such as `invalid_code`,
`rate_limit_exceeded` or `whatsapp_send_failed`), `err.body()` the body for
fields beyond the message, and `err.remaining_attempts()` the attempts a
verification has left after a wrong code. An error the SDK raised before
sending, such as a client-side `Error::Validation` for an empty id or for an id
of `.` or `..` (which the URL parser would turn into a different path), has
neither code nor body.

`Error` also carries `Http(reqwest::Error)` and `Json(serde_json::Error)` for
transport and decoding failures. `err.is_retryable()` is true for `Network`,
`Timeout` and the `RateLimit`s that clear with time (see
[Rate Limits](#rate-limits)), and `err.retry_after()` gives the seconds to wait
from a 429: the `Retry-After` header, else the `retryAfter` in the body.
`Error::Api` is the catch-all for statuses with no dedicated variant, such as
403, 409, 410 and 5xx.

## Message Object

```rust
message.id            // Unique identifier
message.to            // Recipient phone number
message.from          // Option<String> (sender id or number)
message.text          // Message content
message.status        // MessageStatus enum
message.direction     // MessageDirection: Outbound | Inbound
message.segments      // i32 (SMS segments, defaults to 1)
message.credits_used  // i32 — credits consumed
message.is_sandbox    // bool (read from get() and list(); a send leaves it false)
message.simulated     // bool (send responses: nothing reached a handset)
message.simulated_reason // Option<String> (why a live key's send was simulated)
message.action_url    // Option<String> (where to finish setup after a simulated live send)
message.sender_type   // Option<SenderType>: NumberPool | Alphanumeric | Sandbox | Explicit | Unknown
message.retry_count   // i32
message.metadata      // Option<HashMap<String, serde_json::Value>>
message.ai_metadata   // Option<AiMetadata> (intent/sentiment on inbound)
message.created_at    // Option<String>
message.updated_at    // Option<String>
message.delivered_at  // Option<String>
message.error         // Option<String>
message.error_code    // Option<String>
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
| `Sent` | Message was sent to carrier |
| `Delivered` | Message was delivered |
| `Read` | Recipient read it — RCS and WhatsApp only; SMS never reports a read receipt |
| `Failed` | Message delivery failed |
| `Bounced` | Carrier rejected the message |
| `Retrying` | Being retried after a transient failure |
| `Received` | An inbound message on one of your numbers |
| `Unknown` | A status this SDK build doesn't know; the message still decodes |

`MessageStatus` and `SenderType` are `#[non_exhaustive]`, so match with a `_`
arm. `SenderType`'s `User`, `Api`, `System` and `Campaign` are deprecated: the
API never sends them.

## Pricing Tiers

1 credit = $0.01.

| Tier | Example countries | Credits per SMS |
|------|-------------------|-----------------|
| Domestic | US, CA | 2 |
| Tier 1 | GB, AU, PL, SE, BR | 8 |
| Tier 2 | FR, JP, IT, IN, ES | 12 |
| Tier 3 | DE, NL, MX, BE | 16 |
| Tier 4 | UA, VN, PA, GE | 24 |
| Tier 5 | IL, MY, PH, ID | 48 |

Per-country pricing and the tier each country falls in are served by the API.

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
- Automatic retries for connect failures, timeouts and the busy key-check 429, with backoff
- Automatic idempotency keys on every POST except `send_batch` (which carries one only when you pass it), reused on every retry
- Errors that keep the API's `code` and body, and rate-limit errors that carry `retry_after`
- Strong typing with enums that tolerate values added later (`Unknown` variants)
- Comprehensive error types
- Stream-based pagination (`messages().iter()`)

## Enterprise

The Enterprise API lets you programmatically manage workspaces, verification, credits, and API keys for multi-tenant platforms. It requires an enterprise master key — an ordinary live key (`sk_live_v1_…`) that has been marked as your organization's master key in the dashboard; what distinguishes it is the flag on the key, not the prefix. A non-master key is refused with 403 `enterprise_required`, and a master key whose enterprise account is inactive with 403 `enterprise_inactive`. Master keys also get the higher rate limit of 3,000 requests a minute.

### Quick Provision

Create a fully configured workspace in a single call:

```rust
use sendly::{ProvisionWorkspaceRequest, Sendly};

let client = Sendly::new("sk_live_v1_your_master_key");

let request = ProvisionWorkspaceRequest::new("Acme Insurance - Austin")
    .source_workspace_id("ws_verified")
    .credit_amount(5000)
    .credit_source_workspace_id("SOURCE_WORKSPACE_ID")
    .key_name("Production")
    .key_type("live")
    .generate_opt_in_page(true);

let result = client.enterprise().provision(request).await?;

println!("{:?}", result.workspace);
println!("{:?}", result.key);
if let Some(page) = &result.opt_in_page {
    println!("opt-in page: {:?}", page.url);
}
println!("{:?} {:?}", result.api_base_url, result.dashboard_url);
```

Three provisioning modes:

| Mode | Params | Description |
|------|--------|-------------|
| **Inherit** | `.source_workspace_id()` | Shares toll-free number from verified workspace |
| **Inherit + New Number** | `.source_workspace_id()` + `.inherit_with_new_number(true)` | Copies business info, purchases new number |
| **Fresh** | set the `verification` field to a `SubmitVerificationRequest` | Full business details, new number + carrier approval |

`ProvisionWorkspaceRequest` has no builder method for the verification block —
assign the public field directly (`request.verification = Some(...)`).
`SubmitVerificationRequest` is an alias of `VerificationSubmitInput`.

### Workspace Management

```rust
use sendly::{
    AnalyticsPeriod, CreateWorkspaceKeyRequest, CreateWorkspaceRequest, InheritVerificationOptions,
    ListWorkspacesOptions,
};

let ws = client.enterprise().workspaces().create(CreateWorkspaceRequest::new("Acme Insurance")).await?;

// list() returns the first page; list_with_options() pages, searches, filters and sorts
let list = client.enterprise().workspaces()
    .list_with_options(
        ListWorkspacesOptions::new()
            .search("Acme")
            .status("active")
            .sort("credits_desc")
            .limit(50)
            .page(1),
    )
    .await?;
for w in &list.workspaces {
    println!("{} {} credits, verification {:?}", w.name, w.credit_balance, w.verification_status);
}
println!("{} of {} workspaces used", list.workspaces_used, list.max_workspaces);
if let Some(page) = &list.pagination {
    println!("page {} of {}", page.page, page.total_pages);
}

let detail = client.enterprise().workspaces().get("ws_xxx").await?;

// Copy a verified workspace's business details but order this workspace its
// own toll-free number instead of sharing the source's
let inherited = client.enterprise().workspaces()
    .inherit_verification_with_options(
        &ws.id,
        InheritVerificationOptions::new("ws_verified").purchase_new_number(true),
    )
    .await?;
println!("{} (new number: {})", inherited.status, inherited.new_number);

client.enterprise().workspaces().delete("ws_xxx").await?;
```

`ListWorkspacesOptions::compact(true)` returns every workspace with only its
id, name and balance, unpaginated.

### Credits & API Keys

```rust
client.enterprise().workspaces()
    .transfer_credits("ws_dest", "ws_source", 5000).await?;

let key = client.enterprise().workspaces()
    .create_key("ws_xxx", CreateWorkspaceKeyRequest::new("Production").key_type("live")).await?;
println!("{:?}", key);

client.enterprise().workspaces().revoke_key("ws_xxx", "key_abc").await?;
```

### Webhooks & Analytics

```rust
// The signing secret comes back only from the first set() (later set() calls,
// even after delete(), leave signing_secret None) and from rotate_secret();
// store it then
let webhook = client.enterprise().webhooks().set("https://acme.example/webhooks").await?;
println!("{:?}", webhook.signing_secret);

let overview = client.enterprise().analytics().overview().await?;
let messages = client.enterprise().analytics().messages(Some(AnalyticsPeriod::new().period("30d"))).await?;
let delivery = client.enterprise().analytics().delivery().await?;
let credits = client.enterprise().analytics().credits(None).await?;
println!("{} credits across {} workspaces", credits.total_balance, credits.workspace_count);
```

### The rest of the enterprise surface

`client.enterprise()` also exposes `get_account()`, `generate_business_page()`
and `upload_verification_document()`, plus four more sub-resources:
`settings()` (auto top-up), `billing()` (per-workspace breakdown) and
`credits()` (the pool balance and deposits). On
`workspaces()` there is also verification (`submit_verification`,
`resubmit_verification`, `inherit_verification`, `get_verification`),
`get_credits`, `list_keys`, opt-in pages, per-workspace webhooks, `suspend` /
`resume`, `provision_bulk` (up to 100 workspaces in one call),
`set_custom_domain`, invitations
(`send_invitation`, `list_invitations`, `cancel_invitation`) and quotas
(`get_quota`, `set_quota`).

Full enterprise docs: [sendly.live/docs/enterprise](https://sendly.live/docs/enterprise)

---

## License

MIT
