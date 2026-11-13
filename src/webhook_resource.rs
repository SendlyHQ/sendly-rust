//! Webhooks resource for managing webhook endpoints.

use crate::client::{path_id, Sendly};
use crate::error::Result;
use crate::models::{
    CreateWebhookRequest, ListDeliveriesOptions, UpdateWebhookRequest, Webhook,
    WebhookCreatedResponse, WebhookDelivery, WebhookDeliveryList, WebhookSecretRotation,
    WebhookTestResult,
};
use serde::{Deserialize, Serialize};

/// Options for [`WebhooksResource::redeliver`]. Use
/// `RedeliverOptions::default()` to accept server defaults (last 24h,
/// statuses `["failed", "cancelled"]`, limit 1000).
#[derive(Debug, Default, Clone, Serialize)]
pub struct RedeliverOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "event_types")]
    pub event_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// Options for [`WebhooksResource::backfill`]. Use
/// `BackfillOptions::default()` to accept server defaults.
#[derive(Debug, Default, Clone, Serialize)]
pub struct BackfillOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "event_types")]
    pub event_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// Options for [`WebhooksResource::recovery_preview`]. Use
/// [`RecoveryPreviewOptions::new`] for the last 24 hours and every event
/// type, and the builder methods to narrow it. This type is
/// `#[non_exhaustive]`, so external crates must use the constructor rather
/// than a struct literal.
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct RecoveryPreviewOptions {
    /// The window's start, ISO-8601. Default: now − 24h.
    pub since: Option<String>,
    /// The window's end, ISO-8601. Default: now.
    pub until: Option<String>,
    /// Event types to count, sent comma-separated. Default: all.
    pub event_types: Option<Vec<String>>,
}

impl RecoveryPreviewOptions {
    /// Creates options for the last 24 hours and every event type.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the window's start (ISO 8601).
    pub fn since(mut self, since: impl Into<String>) -> Self {
        self.since = Some(since.into());
        self
    }

    /// Sets the window's end (ISO 8601).
    pub fn until(mut self, until: impl Into<String>) -> Self {
        self.until = Some(until.into());
        self
    }

    /// Counts only these event types.
    pub fn event_types<E>(mut self, event_types: impl IntoIterator<Item = E>) -> Self
    where
        E: Into<String>,
    {
        self.event_types = Some(event_types.into_iter().map(Into::into).collect());
        self
    }

    fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(ref since) = self.since {
            params.push(("since".to_string(), since.clone()));
        }
        if let Some(ref until) = self.until {
            params.push(("until".to_string(), until.clone()));
        }
        if let Some(ref event_types) = self.event_types {
            if !event_types.is_empty() {
                params.push(("event_types".to_string(), event_types.join(",")));
            }
        }
        params
    }
}

/// What a webhook recovery would do for a window, without sending anything.
/// Returned by [`WebhooksResource::recovery_preview`]. The API sends its
/// keys in camelCase, unlike the other webhook responses.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WebhookRecoveryPreview {
    /// The circuit breaker: `closed`, `open` or `half_open`. More values may
    /// be added.
    pub circuit_state: String,
    /// When the circuit opened, if it has.
    #[serde(default)]
    pub circuit_opened_at: Option<String>,
    /// Failed deliveries in the current streak.
    #[serde(default)]
    pub failure_count: u64,
    /// Failed and cancelled deliveries in the window: what
    /// [`redeliver`](WebhooksResource::redeliver) would send.
    pub audit_failures: WebhookRecoveryAuditFailures,
    /// Message events with no delivery in the window: what
    /// [`backfill`](WebhooksResource::backfill) would send.
    pub missing_from_messages: WebhookRecoveryMissing,
    /// The window in up to 24 equal buckets, oldest first.
    #[serde(default)]
    pub histogram: Vec<WebhookRecoveryHistogramBucket>,
    /// Up to 5 affected events: failed deliveries first, then missing
    /// message events, each newest first.
    #[serde(default)]
    pub samples: Vec<WebhookRecoverySample>,
    /// The outage window, when the circuit has an opening time.
    #[serde(default)]
    pub window_suggestion: Option<WebhookRecoveryWindow>,
    /// The window's start.
    pub since: String,
    /// The window's end.
    pub until: String,
}

/// Failed and cancelled deliveries in a recovery preview's window.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WebhookRecoveryAuditFailures {
    /// How many deliveries.
    pub total: u64,
    /// Counts by delivery status, such as `failed` and `cancelled`.
    #[serde(default)]
    pub by_status: std::collections::HashMap<String, u64>,
    /// Counts by event type, such as `message.delivered`.
    #[serde(default)]
    pub by_event_type: std::collections::HashMap<String, u64>,
}

/// Message events with no delivery in a recovery preview's window.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WebhookRecoveryMissing {
    /// How many events.
    pub total: u64,
    /// Counts by event type, such as `message.sent`.
    #[serde(default)]
    pub by_event_type: std::collections::HashMap<String, u64>,
}

/// One bucket of a recovery preview's histogram.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct WebhookRecoveryHistogramBucket {
    /// The bucket's start.
    pub bucket: String,
    /// Missing message events in the bucket.
    pub missing: u64,
    /// Failed deliveries in the bucket.
    pub failed: u64,
}

/// One affected event in a recovery preview.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WebhookRecoverySample {
    /// The event's object id, such as a message id.
    pub id: String,
    /// The recipient, masked, or `***` when there is none.
    pub to_masked: String,
    /// The event type.
    pub event_type: String,
    /// When the delivery or message was created.
    pub occurred_at: String,
}

/// The suggested outage window in a recovery preview.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct WebhookRecoveryWindow {
    /// When the circuit opened.
    pub since: String,
    /// Now.
    pub until: String,
}

/// Webhooks resource for managing webhook endpoints.
pub struct WebhooksResource<'a> {
    client: &'a Sendly,
}

#[derive(Debug, Deserialize)]
struct WebhookResponse {
    #[serde(default)]
    webhook: Option<Webhook>,
    #[serde(default)]
    data: Option<Webhook>,
    #[serde(flatten)]
    flat: Option<Webhook>,
}

#[derive(Debug, Deserialize)]
struct WebhookListResponse {
    #[serde(default)]
    webhooks: Option<Vec<Webhook>>,
    #[serde(default)]
    data: Option<Vec<Webhook>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum WebhookListWire {
    Bare(Vec<Webhook>),
    Wrapped(WebhookListResponse),
}

#[derive(Debug, Deserialize)]
struct DeliveryResponse {
    #[serde(default)]
    delivery: Option<WebhookDelivery>,
    #[serde(default)]
    data: Option<WebhookDelivery>,
}

impl<'a> WebhooksResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Creates a new webhook.
    ///
    /// # Arguments
    ///
    /// * `url` - URL to receive webhook events
    /// * `events` - List of event types to subscribe to
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let response = client.webhooks().create(
    ///     "https://example.com/webhook",
    ///     vec!["message.delivered", "message.failed"],
    /// ).await?;
    ///
    /// println!("Webhook created: {:?}", response.get_webhook());
    /// println!("Secret: {}", response.secret);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(
        &self,
        url: impl Into<String>,
        events: Vec<impl Into<String>>,
    ) -> Result<WebhookCreatedResponse> {
        self.create_with_options(CreateWebhookRequest::new(url, events))
            .await
    }

    /// Creates a new webhook with full options: a description, the event
    /// mode and your own metadata.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{CreateWebhookRequest, Sendly, WebhookMode};
    ///
    /// # async fn example() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let response = client.webhooks().create_with_options(
    ///     CreateWebhookRequest::new(
    ///         "https://example.com/webhook",
    ///         ["message.delivered", "message.failed"],
    ///     )
    ///     .description("Delivery receipts")
    ///     .mode(WebhookMode::Live)
    ///     .metadata_entry("team", "billing"),
    /// ).await?;
    ///
    /// println!("Secret: {}", response.secret);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_with_options(
        &self,
        request: CreateWebhookRequest,
    ) -> Result<WebhookCreatedResponse> {
        let response = self.client.post("/webhooks", &request).await?;
        let result: WebhookCreatedResponse = response.json().await?;
        Ok(result)
    }

    /// Lists all webhooks.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let webhooks = client.webhooks().list().await?;
    /// for webhook in webhooks {
    ///     println!("Webhook: {} -> {}", webhook.id, webhook.url);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self) -> Result<Vec<Webhook>> {
        let response = self.client.get("/webhooks", &[]).await?;
        let result: WebhookListWire = response.json().await?;

        Ok(match result {
            WebhookListWire::Bare(webhooks) => webhooks,
            WebhookListWire::Wrapped(wrapped) => {
                wrapped.webhooks.or(wrapped.data).unwrap_or_default()
            }
        })
    }

    /// Gets a webhook by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Webhook ID
    pub async fn get(&self, id: impl AsRef<str>) -> Result<Webhook> {
        let path = format!("/webhooks/{}", path_id(id.as_ref())?);
        let response = self.client.get(&path, &[]).await?;
        let result: WebhookResponse = response.json().await?;

        Ok(result
            .webhook
            .or(result.data)
            .or(result.flat)
            .unwrap_or_else(|| Webhook {
                id: String::new(),
                url: String::new(),
                events: Vec::new(),
                description: None,
                mode: crate::models::WebhookMode::All,
                is_active: true,
                failure_count: 0,
                circuit_state: crate::models::CircuitState::Closed,
                last_failure_at: None,
                circuit_opened_at: None,
                api_version: None,
                metadata: None,
                total_deliveries: 0,
                successful_deliveries: 0,
                success_rate: 0.0,
                last_delivery_at: None,
                created_at: None,
                updated_at: None,
            }))
    }

    /// Updates a webhook.
    ///
    /// # Arguments
    ///
    /// * `id` - Webhook ID
    /// * `request` - Update options
    pub async fn update(
        &self,
        id: impl AsRef<str>,
        request: UpdateWebhookRequest,
    ) -> Result<Webhook> {
        let path = format!("/webhooks/{}", path_id(id.as_ref())?);
        let response = self.client.patch(&path, &request).await?;
        let result: WebhookResponse = response.json().await?;

        Ok(result
            .webhook
            .or(result.data)
            .or(result.flat)
            .unwrap_or_else(|| Webhook {
                id: String::new(),
                url: String::new(),
                events: Vec::new(),
                description: None,
                mode: crate::models::WebhookMode::All,
                is_active: true,
                failure_count: 0,
                circuit_state: crate::models::CircuitState::Closed,
                last_failure_at: None,
                circuit_opened_at: None,
                api_version: None,
                metadata: None,
                total_deliveries: 0,
                successful_deliveries: 0,
                success_rate: 0.0,
                last_delivery_at: None,
                created_at: None,
                updated_at: None,
            }))
    }

    /// Deletes a webhook.
    ///
    /// # Arguments
    ///
    /// * `id` - Webhook ID
    pub async fn delete(&self, id: impl AsRef<str>) -> Result<()> {
        let path = format!("/webhooks/{}", path_id(id.as_ref())?);
        self.client.delete(&path).await?;
        Ok(())
    }

    /// Tests a webhook endpoint by sending it a `webhook.test` event.
    ///
    /// Returns the result when the endpoint accepted the event. When it did
    /// not, the API answers 400 and this returns
    /// [`Error::Validation`](crate::Error::Validation), whose message says
    /// why.
    ///
    /// # Arguments
    ///
    /// * `id` - Webhook ID
    pub async fn test(&self, id: impl AsRef<str>) -> Result<WebhookTestResult> {
        let path = format!("/webhooks/{}/test", path_id(id.as_ref())?);
        let response = self.client.post_empty(&path).await?;
        let result: WebhookTestResult = response.json().await?;
        Ok(result)
    }

    /// Resets the circuit breaker for a webhook.
    ///
    /// # Arguments
    ///
    /// * `id` - Webhook ID
    pub async fn reset_circuit(&self, id: impl AsRef<str>) -> Result<serde_json::Value> {
        let path = format!("/webhooks/{}/reset-circuit", path_id(id.as_ref())?);
        let response = self.client.post_empty(&path).await?;
        let result: serde_json::Value = response.json().await?;
        Ok(result)
    }

    /// Replay failed or cancelled webhook deliveries from the audit log.
    ///
    /// Use after a customer endpoint has recovered from an outage to re-fire
    /// deliveries we recorded but couldn't deliver. Each replay creates a
    /// new delivery row preserving the original `event_id` so customers can
    /// dedupe. Rejects with HTTP 409 if the circuit is currently open —
    /// call [`reset_circuit`](Self::reset_circuit) first.
    ///
    /// # Arguments
    ///
    /// * `id` - Webhook ID
    /// * `options` - Window and filter options; `RedeliverOptions::default()`
    ///   uses server defaults (last 24h, statuses `["failed", "cancelled"]`,
    ///   limit 1000).
    pub async fn redeliver(
        &self,
        id: impl AsRef<str>,
        options: RedeliverOptions,
    ) -> Result<serde_json::Value> {
        let path = format!("/webhooks/{}/redeliver", path_id(id.as_ref())?);
        let response = self.client.post(&path, &options).await?;
        let result: serde_json::Value = response.json().await?;
        Ok(result)
    }

    /// Backfill missed webhook events from the underlying message log.
    ///
    /// Use when a circuit-breaker outage left events with no audit row (the
    /// case [`redeliver`](Self::redeliver) cannot recover). Synthesized
    /// message events carry the same event id the original dispatch used,
    /// so dedupe on `event.id`. Do not dedupe on `data.object.id`: a
    /// message's sent and delivered events share it. Rejects with HTTP 409 if
    /// the circuit is currently open — call
    /// [`reset_circuit`](Self::reset_circuit) first.
    ///
    /// # Arguments
    ///
    /// * `id` - Webhook ID
    /// * `options` - Window and filter options; `BackfillOptions::default()`
    ///   uses server defaults (last 24h, all subscribed message events,
    ///   limit 1000).
    pub async fn backfill(
        &self,
        id: impl AsRef<str>,
        options: BackfillOptions,
    ) -> Result<serde_json::Value> {
        let path = format!("/webhooks/{}/backfill", path_id(id.as_ref())?);
        let response = self.client.post(&path, &options).await?;
        let result: serde_json::Value = response.json().await?;
        Ok(result)
    }

    /// Preview a recovery: count what [`redeliver`](Self::redeliver) and
    /// [`backfill`](Self::backfill) would send for a window, without
    /// sending anything. The window is at most 7 days long.
    ///
    /// # Arguments
    ///
    /// * `id` - Webhook ID
    /// * `options` - Window and event types; `RecoveryPreviewOptions::new()`
    ///   counts the last 24 hours and every event type.
    pub async fn recovery_preview(
        &self,
        id: impl AsRef<str>,
        options: RecoveryPreviewOptions,
    ) -> Result<WebhookRecoveryPreview> {
        let path = format!("/webhooks/{}/recovery-preview", path_id(id.as_ref())?);
        let query = options.to_query_params();
        let response = self.client.get(&path, &query).await?;
        let result: WebhookRecoveryPreview = response.json().await?;
        Ok(result)
    }

    /// Rotates a webhook's secret.
    ///
    /// # Arguments
    ///
    /// * `id` - Webhook ID
    pub async fn rotate_secret(&self, id: impl AsRef<str>) -> Result<WebhookSecretRotation> {
        let path = format!("/webhooks/{}/rotate-secret", path_id(id.as_ref())?);
        let response = self.client.post_empty(&path).await?;
        let result: WebhookSecretRotation = response.json().await?;
        Ok(result)
    }

    /// Lists delivery attempts for a webhook.
    ///
    /// # Arguments
    ///
    /// * `id` - Webhook ID
    /// * `options` - Query options
    pub async fn list_deliveries(
        &self,
        id: impl AsRef<str>,
        options: Option<ListDeliveriesOptions>,
    ) -> Result<WebhookDeliveryList> {
        let path = format!("/webhooks/{}/deliveries", path_id(id.as_ref())?);
        let query = options.unwrap_or_default().to_query_params();
        let response = self.client.get(&path, &query).await?;
        let result: WebhookDeliveryList = response.json().await?;
        Ok(result)
    }

    /// Gets a specific delivery attempt.
    ///
    /// # Arguments
    ///
    /// * `webhook_id` - Webhook ID
    /// * `delivery_id` - Delivery ID
    pub async fn get_delivery(
        &self,
        webhook_id: impl AsRef<str>,
        delivery_id: impl AsRef<str>,
    ) -> Result<WebhookDelivery> {
        let path = format!(
            "/webhooks/{}/deliveries/{}",
            path_id(webhook_id.as_ref())?,
            path_id(delivery_id.as_ref())?
        );
        let response = self.client.get(&path, &[]).await?;
        let result: DeliveryResponse = response.json().await?;

        Ok(result
            .delivery
            .or(result.data)
            .unwrap_or_else(|| WebhookDelivery {
                id: String::new(),
                webhook_id: String::new(),
                event_type: String::new(),
                http_status: 0,
                success: false,
                attempt_number: 1,
                error_message: None,
                response_time_ms: 0,
                created_at: None,
            }))
    }

    /// Retries a failed delivery.
    ///
    /// # Arguments
    ///
    /// * `webhook_id` - Webhook ID
    /// * `delivery_id` - Delivery ID
    pub async fn retry_delivery(
        &self,
        webhook_id: impl AsRef<str>,
        delivery_id: impl AsRef<str>,
    ) -> Result<WebhookDelivery> {
        let path = format!(
            "/webhooks/{}/deliveries/{}/retry",
            path_id(webhook_id.as_ref())?,
            path_id(delivery_id.as_ref())?
        );
        let response = self.client.post_empty(&path).await?;
        let result: DeliveryResponse = response.json().await?;

        Ok(result
            .delivery
            .or(result.data)
            .unwrap_or_else(|| WebhookDelivery {
                id: String::new(),
                webhook_id: String::new(),
                event_type: String::new(),
                http_status: 0,
                success: false,
                attempt_number: 1,
                error_message: None,
                response_time_ms: 0,
                created_at: None,
            }))
    }

    /// Lists available webhook event types.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let event_types = client.webhooks().list_event_types().await?;
    /// for event_type in event_types {
    ///     println!("Event type: {}", event_type);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_event_types(&self) -> Result<Vec<String>> {
        #[derive(Debug, Deserialize)]
        struct EventType {
            #[serde(rename = "type")]
            event_type: String,
        }

        #[derive(Debug, Deserialize)]
        struct EventTypesResponse {
            #[serde(default)]
            events: Vec<EventType>,
        }

        let response = self.client.get("/webhooks/event-types", &[]).await?;
        let result: EventTypesResponse = response.json().await?;

        Ok(result.events.into_iter().map(|e| e.event_type).collect())
    }
}
