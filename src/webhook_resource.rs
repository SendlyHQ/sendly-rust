//! Webhooks resource for managing webhook endpoints.

use crate::client::Sendly;
use crate::error::Result;
use crate::models::{
    CreateWebhookRequest, ListDeliveriesOptions, UpdateWebhookRequest, Webhook,
    WebhookCreatedResponse, WebhookDelivery, WebhookDeliveryList, WebhookSecretRotation,
    WebhookTestResult,
};
use serde::Deserialize;

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
        let request = CreateWebhookRequest {
            url: url.into(),
            events: events.into_iter().map(|e| e.into()).collect(),
            api_version: None,
        };

        self.create_with_options(request).await
    }

    /// Creates a new webhook with full options.
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
        let result: WebhookListResponse = response.json().await?;

        Ok(result.webhooks.or(result.data).unwrap_or_default())
    }

    /// Gets a webhook by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Webhook ID
    pub async fn get(&self, id: impl AsRef<str>) -> Result<Webhook> {
        let path = format!("/webhooks/{}", id.as_ref());
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
                is_active: true,
                failure_count: 0,
                circuit_state: crate::models::CircuitState::Closed,
                api_version: None,
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
        let path = format!("/webhooks/{}", id.as_ref());
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
                is_active: true,
                failure_count: 0,
                circuit_state: crate::models::CircuitState::Closed,
                api_version: None,
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
        let path = format!("/webhooks/{}", id.as_ref());
        self.client.delete(&path).await?;
        Ok(())
    }

    /// Tests a webhook endpoint.
    ///
    /// # Arguments
    ///
    /// * `id` - Webhook ID
    pub async fn test(&self, id: impl AsRef<str>) -> Result<WebhookTestResult> {
        let path = format!("/webhooks/{}/test", id.as_ref());
        let response = self.client.post(&path, &()).await?;
        let result: WebhookTestResult = response.json().await?;
        Ok(result)
    }

    /// Rotates a webhook's secret.
    ///
    /// # Arguments
    ///
    /// * `id` - Webhook ID
    pub async fn rotate_secret(&self, id: impl AsRef<str>) -> Result<WebhookSecretRotation> {
        let path = format!("/webhooks/{}/rotate-secret", id.as_ref());
        let response = self.client.post(&path, &()).await?;
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
        let path = format!("/webhooks/{}/deliveries", id.as_ref());
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
            webhook_id.as_ref(),
            delivery_id.as_ref()
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
            webhook_id.as_ref(),
            delivery_id.as_ref()
        );
        let response = self.client.post(&path, &()).await?;
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
}
