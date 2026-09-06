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
sendly = "3.36.0"
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
        .send(SendMessageRequest::new("+15551234567", "Hello from Sendly!"))
        .await?;

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

## Idempotency

Every send POST carries an automatically generated `Idempotency-Key`, created once per logical request and reused across the SDK's own retries, so a retry of a request that already reached the API returns the original result instead of sending again. Pass your own key through the `*_with_options` send methods when the guarantee needs to outlive the process, such as a job queue that re-runs after a crash. Repeating a request with the same key within 24 hours returns the original response instead of executing again. Keys are 1-255 printable ASCII characters; `send_batch` attaches no automatic key because the API already deduplicates identical batches by their contents. RCS registration writes (brand and agent create/update, test devices, submit, request launch) carry a key the same way, including their PATCH and PUT calls, with `*_with_options` variants for your own key.

```rust
use sendly::{IdempotentRequestOptions, Sendly, SendMessageRequest};

let client = Sendly::new("sk_live_v1_xxx");

let message = client.messages()
    .send_with_options(
        SendMessageRequest::new("+15551234567", "Your order #4821 has shipped!"),
        IdempotentRequestOptions::new().idempotency_key("order-4821-shipped"),
    )
    .await?;
```

Full details: https://sendly.live/docs/idempotency

## Messages

### Send an SMS

```rust
use sendly::{Sendly, SendMessageRequest, MessageType};

let client = Sendly::new("sk_live_v1_xxx");

// Marketing message (default)
let message = client.messages()
    .send_to("+15551234567", "Check out our new features!")
    .await?;

// Send from one of your owned numbers
let message = client.messages().send(
    SendMessageRequest::new("+15551234567", "Hello from Sendly!")
        .with_from("+447111111111"),
).await?;

// Transactional message (bypasses quiet hours)
let message = client.messages().send(
    SendMessageRequest::new("+15551234567", "Your verification code is: 123456")
        .with_message_type(MessageType::Transactional),
).await?;

// With custom metadata (max 4KB)
use std::collections::HashMap;
let mut metadata = HashMap::new();
metadata.insert("order_id".to_string(), serde_json::json!("12345"));
metadata.insert("customer_id".to_string(), serde_json::json!("cust_abc"));

let message = client.messages().send(
    SendMessageRequest::new("+15551234567", "Your order #12345 has shipped!")
        .with_metadata(metadata),
).await?;

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
let messages = client.messages();
let batches = messages.list_batches(None).await?;
for batch in batches {
    println!("{}: {:?}", batch.batch_id, batch.status);
}

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
use sendly::{Sendly, UpdateWebhookRequest};

// Create a webhook endpoint
let webhook = client.webhooks().create(
    "https://example.com/webhooks/sendly",
    vec!["message.delivered", "message.failed"],
).await?;

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
let credits = client.account().credits().await?;
println!("Available: {} credits", credits.available_balance);
println!("Reserved: {} credits", credits.reserved_balance);
println!("Total: {} credits", credits.balance);

// View credit transaction history
let transactions = client.account().transactions(None).await?;
for tx in &transactions.data {
    println!("{:?}: {} credits - {}", tx.transaction_type, tx.amount, tx.description);
}

// List API keys
let keys = client.account().api_keys().await?;
for key in &keys {
    println!("{}: {}***", key.name, key.prefix);
}

// Get a specific API key
let key = client.account().get_api_key("key_xxx").await?;

// Get API key usage stats
let usage = client.account().get_api_key_usage("key_xxx").await?;
println!("Messages sent: {}", usage.messages_sent);

// Create a new API key
let new_key = client.account().create_api_key("Production Key").await?;
println!("New key: {:?}", new_key.key); // Only shown once!

// Revoke an API key
client.account().revoke_api_key("key_xxx").await?;

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

// Buy a number (asynchronous — see the numbers docs for documents/payment hand-offs)
let first = &available.numbers[0];
let result = client.numbers().buy(BuyNumberRequest::new(
    &first.phone_number, &first.country, &first.number_type, &first.monthly_cost,
)).await?;
println!("Buy status: {}", result.status);

// List the numbers you own
let owned = client.numbers().list().await?;
for n in &owned.numbers {
    println!("{} — {}", n.phone_number, n.status);
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
        "+14155551234".to_string(),
        "+14155555678".to_string(),
    ])
    .with_text("Hey team — quick sync at noon?"),
).await?;

println!("Group message: {} ({})", group.id, group.status);
println!("Group id: {:?}", group.group_message_id);
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
let link = client.links().create("https://example.com/spring-sale").await?;
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

## WhatsApp

Connect a number you own to WhatsApp ($19 one-time, no monthly fee), create
Meta-reviewed templates, and send. Free-form text and media only deliver inside
an open 24-hour customer-service window; an approved template works anytime.

```rust
use sendly::{
    Sendly, SendWhatsAppMessageRequest, CreateWhatsAppTemplateRequest,
    WhatsAppTemplateCategory, WhatsAppTemplateSendParams,
};
use std::collections::HashMap;

let client = Sendly::new("sk_live_v1_xxx");

// Connect a number — the connect URL must be opened by a human, who logs in
// with Facebook to link their WhatsApp Business Account
let signup = client.whatsapp().signup().create("+15559876543").await?;
println!("Have your user open: {}", signup.connect_url);

// Poll until active
let status = client.whatsapp().signup().get(&signup.id).await?;

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
        "+15559876543",
        "order_shipped",
        "en_US",
        WhatsAppTemplateCategory::Utility,
        "Hi {{1}}, your order {{2}} has shipped!",
    )
    .examples(examples),
).await?;

// Check the 24-hour window, then send
let window = client.whatsapp().window("+15559876543", "+15551234567").await?;

if window.open {
    // Free-form text (or media with a caption via .with_media_urls())
    client.messages().send_whatsapp(
        SendWhatsAppMessageRequest::new("+15551234567", "+15559876543")
            .with_text("Your table is ready!"),
    ).await?;
} else {
    // Approved template — works regardless of the window
    let mut variables = HashMap::new();
    variables.insert("1".to_string(), "Sam".to_string());
    variables.insert("2".to_string(), "#4821".to_string());
    let message = client.messages().send_whatsapp(
        SendWhatsAppMessageRequest::new("+15551234567", "+15559876543")
            .with_template(
                WhatsAppTemplateSendParams::new("order_shipped", "en_US")
                    .with_variables(variables),
            ),
    ).await?;
    println!("Kind: {:?}", message.whatsapp.kind);
}
```

Every connected sender has a WhatsApp Business profile — the name, photo, and
business details recipients see when they tap your number. Send only the fields
you want to change (`about` is capped at 139 characters, `description` at 512):

```rust
use sendly::UpdateWhatsAppSenderProfileRequest;

let profile = client.whatsapp().senders().get_profile("+15559876543").await?;
println!("{:?}", profile.display_name);

let updated = client.whatsapp().senders().update_profile(
    "+15559876543",
    UpdateWhatsAppSenderProfileRequest::new()
        .about("Fresh bread, daily.")
        .description("Family bakery in Austin since 1998.")
        .email("hello@example.com")
        .website("https://example.com"),
).await?;
```

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
let capability = client.rcs().capability("+15551234567", None).await?;
println!("capable: {} {:?}", capability.capable, capability.features);

// Text with tappable chips
let message = client.messages().send_rcs(
    SendRcsMessageRequest::new("+15551234567")
        .with_text("Your order #4821 has shipped!")
        .with_suggestions(vec![
            RcsSuggestion::reply("Track it", "track_4821"),
            RcsSuggestion::action(
                "View receipt",
                "receipt_4821",
                "https://example.com/receipts/4821",
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
    SendRcsMessageRequest::new("+15551234567").with_card(
        RcsCard::new(
            "Your table is ready",
            "Head to the host stand — we'll hold it for 10 minutes.",
        )
        .with_media_url("https://example.com/table.jpg")
        .with_suggestions(vec![RcsSuggestion::reply("On my way", "otw")]),
    ),
).await?;
println!("Kind: {:?}", card.rcs.kind);

// Require RCS delivery — turn the fallback off
client.messages().send_rcs(
    SendRcsMessageRequest::new("+15551234567")
        .with_text("RCS only.")
        .with_fallback_to_sms(false),
).await?;
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
message.created_at   // Option<String>
message.updated_at   // Option<String>
message.delivered_at // Option<String>
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
use sendly::ProvisionWorkspaceRequest;

let client = Sendly::new("sk_live_v1_master_YOUR_KEY");

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
```

Three provisioning modes:

| Mode | Params | Description |
|------|--------|-------------|
| **Inherit** | `.source_workspace_id()` | Shares toll-free number from verified workspace |
| **Inherit + New Number** | `.source_workspace_id()` + `.inherit_with_new_number(true)` | Copies business info, purchases new number |
| **Fresh** | `.verification(VerificationData{...})` | Full business details, new number + carrier approval |

### Workspace Management

```rust
use sendly::{CreateWorkspaceRequest, CreateWorkspaceKeyRequest, AnalyticsPeriod};

let ws = client.enterprise().workspaces().create(CreateWorkspaceRequest::new("Acme Insurance")).await?;
let list = client.enterprise().workspaces().list().await?;
let detail = client.enterprise().workspaces().get("ws_xxx").await?;
client.enterprise().workspaces().delete("ws_xxx").await?;
```

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
client.enterprise().webhooks().set("https://yourapp.com/webhooks").await?;
let overview = client.enterprise().analytics().overview().await?;
let messages = client.enterprise().analytics().messages(Some(AnalyticsPeriod::new().period("30d"))).await?;
let delivery = client.enterprise().analytics().delivery().await?;
```

Full enterprise docs: [sendly.live/docs/enterprise](https://sendly.live/docs/enterprise)

---

## License

MIT
