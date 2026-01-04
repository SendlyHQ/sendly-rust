use serde::{Deserialize, Serialize};

/// Message delivery status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    /// Message is queued for delivery.
    Queued,
    /// Message was sent to carrier.
    Sent,
    /// Message was delivered.
    Delivered,
    /// Message delivery failed.
    Failed,
}

impl std::fmt::Display for MessageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageStatus::Queued => write!(f, "queued"),
            MessageStatus::Sent => write!(f, "sent"),
            MessageStatus::Delivered => write!(f, "delivered"),
            MessageStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Message direction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageDirection {
    /// Outbound message (sent by you).
    Outbound,
    /// Inbound message (received from recipient).
    Inbound,
}

impl Default for MessageDirection {
    fn default() -> Self {
        MessageDirection::Outbound
    }
}

/// Sender type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SenderType {
    /// Sent by user via dashboard.
    User,
    /// Sent via API.
    Api,
    /// System-generated message.
    System,
    /// Campaign message.
    Campaign,
}

/// An SMS message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier.
    pub id: String,
    /// Recipient phone number in E.164 format.
    pub to: String,
    /// Sender ID or phone number.
    #[serde(default)]
    pub from: Option<String>,
    /// Message content.
    pub text: String,
    /// Delivery status.
    pub status: MessageStatus,
    /// Message direction.
    #[serde(default)]
    pub direction: MessageDirection,
    /// Number of SMS segments.
    #[serde(default = "default_segments")]
    pub segments: i32,
    /// Credits consumed.
    #[serde(default, alias = "creditsUsed")]
    pub credits_used: i32,
    /// Whether sent in sandbox mode.
    #[serde(default, alias = "isSandbox")]
    pub is_sandbox: bool,
    /// Type of sender.
    #[serde(default, alias = "senderType")]
    pub sender_type: Option<SenderType>,
    /// Telnyx message ID for carrier tracking.
    #[serde(default, alias = "telnyxMessageId")]
    pub telnyx_message_id: Option<String>,
    /// Warning message if any.
    #[serde(default)]
    pub warning: Option<String>,
    /// Optional note from the sender.
    #[serde(default, alias = "senderNote")]
    pub sender_note: Option<String>,
    /// Error message (if failed).
    #[serde(default)]
    pub error: Option<String>,
    /// Error code (if failed).
    #[serde(default, alias = "errorCode")]
    pub error_code: Option<String>,
    /// Error message (if failed).
    #[serde(default, alias = "errorMessage")]
    pub error_message: Option<String>,
    /// Creation timestamp.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// Last update timestamp.
    #[serde(default, alias = "updatedAt")]
    pub updated_at: Option<String>,
    /// Delivery timestamp (if delivered).
    #[serde(default, alias = "deliveredAt")]
    pub delivered_at: Option<String>,
}

fn default_segments() -> i32 {
    1
}

impl Message {
    /// Returns true if the message was delivered.
    pub fn is_delivered(&self) -> bool {
        self.status == MessageStatus::Delivered
    }

    /// Returns true if the message failed.
    pub fn is_failed(&self) -> bool {
        self.status == MessageStatus::Failed
    }

    /// Returns true if the message is pending.
    pub fn is_pending(&self) -> bool {
        matches!(self.status, MessageStatus::Queued | MessageStatus::Sent)
    }
}

/// Message type for compliance handling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    /// Marketing message (subject to quiet hours restrictions).
    Marketing,
    /// Transactional message (24/7 delivery, bypasses quiet hours).
    Transactional,
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageType::Marketing => write!(f, "marketing"),
            MessageType::Transactional => write!(f, "transactional"),
        }
    }
}

/// Request to send an SMS message.
#[derive(Debug, Clone, Serialize)]
pub struct SendMessageRequest {
    /// Recipient phone number in E.164 format.
    pub to: String,
    /// Message content (max 1600 characters).
    pub text: String,
    /// Message type: "marketing" (default, subject to quiet hours) or "transactional" (24/7).
    #[serde(skip_serializing_if = "Option::is_none", rename = "messageType")]
    pub message_type: Option<MessageType>,
}

/// Options for listing messages.
#[derive(Debug, Clone, Default)]
pub struct ListMessagesOptions {
    /// Maximum messages to return (default: 20, max: 100).
    pub limit: Option<u32>,
    /// Number of messages to skip.
    pub offset: Option<u32>,
    /// Filter by status.
    pub status: Option<MessageStatus>,
    /// Filter by recipient phone number.
    pub to: Option<String>,
}

impl ListMessagesOptions {
    /// Creates new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.min(100));
        self
    }

    /// Sets the offset.
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Sets the status filter.
    pub fn status(mut self, status: MessageStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the to filter.
    pub fn to(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
        self
    }

    pub(crate) fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(limit) = self.limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(offset) = self.offset {
            params.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(ref status) = self.status {
            params.push(("status".to_string(), status.to_string()));
        }
        if let Some(ref to) = self.to {
            params.push(("to".to_string(), to.clone()));
        }

        params
    }
}

/// Paginated list of messages.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageList {
    /// Messages in this page.
    pub data: Vec<Message>,
    /// Total count of messages matching the query.
    #[serde(default)]
    pub count: i32,
}

impl MessageList {
    /// Returns the number of messages in this page.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the total count of messages.
    pub fn total(&self) -> i32 {
        self.count
    }

    /// Returns the first message.
    pub fn first(&self) -> Option<&Message> {
        self.data.first()
    }

    /// Returns the last message.
    pub fn last(&self) -> Option<&Message> {
        self.data.last()
    }

    /// Returns an iterator over messages.
    pub fn iter(&self) -> impl Iterator<Item = &Message> {
        self.data.iter()
    }
}

impl IntoIterator for MessageList {
    type Item = Message;
    type IntoIter = std::vec::IntoIter<Message>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

// ==================== Scheduled Messages ====================

/// Status of a scheduled message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScheduledMessageStatus {
    /// Message is scheduled for future delivery.
    Scheduled,
    /// Message was sent.
    Sent,
    /// Message was cancelled.
    Cancelled,
    /// Message failed to send.
    Failed,
}

impl std::fmt::Display for ScheduledMessageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScheduledMessageStatus::Scheduled => write!(f, "scheduled"),
            ScheduledMessageStatus::Sent => write!(f, "sent"),
            ScheduledMessageStatus::Cancelled => write!(f, "cancelled"),
            ScheduledMessageStatus::Failed => write!(f, "failed"),
        }
    }
}

/// A scheduled SMS message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledMessage {
    /// Unique scheduled message identifier.
    pub id: String,
    /// Recipient phone number in E.164 format.
    pub to: String,
    /// Sender ID or phone number.
    #[serde(default)]
    pub from: Option<String>,
    /// Message content.
    pub text: String,
    /// When the message is scheduled to be sent (ISO 8601).
    #[serde(alias = "scheduledAt")]
    pub scheduled_at: String,
    /// Scheduled message status.
    pub status: ScheduledMessageStatus,
    /// Credits reserved for this message.
    #[serde(default, alias = "creditsReserved")]
    pub credits_reserved: i32,
    /// Creation timestamp.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// When the message was sent.
    #[serde(default, alias = "sentAt")]
    pub sent_at: Option<String>,
    /// When the message was cancelled.
    #[serde(default, alias = "cancelledAt")]
    pub cancelled_at: Option<String>,
    /// Message ID after sending.
    #[serde(default, alias = "messageId")]
    pub message_id: Option<String>,
}

impl ScheduledMessage {
    /// Returns true if the message is still scheduled.
    pub fn is_scheduled(&self) -> bool {
        self.status == ScheduledMessageStatus::Scheduled
    }

    /// Returns true if the message was sent.
    pub fn is_sent(&self) -> bool {
        self.status == ScheduledMessageStatus::Sent
    }

    /// Returns true if the message was cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.status == ScheduledMessageStatus::Cancelled
    }
}

/// Request to schedule an SMS message.
#[derive(Debug, Clone, Serialize)]
pub struct ScheduleMessageRequest {
    /// Recipient phone number in E.164 format.
    pub to: String,
    /// Message content (max 1600 characters).
    pub text: String,
    /// When to send the message (ISO 8601).
    #[serde(rename = "scheduledAt")]
    pub scheduled_at: String,
    /// Sender ID or phone number (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Message type: "marketing" (default, subject to quiet hours) or "transactional" (24/7).
    #[serde(skip_serializing_if = "Option::is_none", rename = "messageType")]
    pub message_type: Option<MessageType>,
}

/// Options for listing scheduled messages.
#[derive(Debug, Clone, Default)]
pub struct ListScheduledMessagesOptions {
    /// Maximum messages to return (default: 20, max: 100).
    pub limit: Option<u32>,
    /// Number of messages to skip.
    pub offset: Option<u32>,
    /// Filter by status.
    pub status: Option<ScheduledMessageStatus>,
}

impl ListScheduledMessagesOptions {
    /// Creates new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.min(100));
        self
    }

    /// Sets the offset.
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Sets the status filter.
    pub fn status(mut self, status: ScheduledMessageStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub(crate) fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(limit) = self.limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(offset) = self.offset {
            params.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(ref status) = self.status {
            params.push(("status".to_string(), status.to_string()));
        }

        params
    }
}

/// Paginated list of scheduled messages.
#[derive(Debug, Clone, Deserialize)]
pub struct ScheduledMessageList {
    /// Scheduled messages in this page.
    pub data: Vec<ScheduledMessage>,
    /// Total count of scheduled messages.
    #[serde(default)]
    pub count: i32,
}

impl ScheduledMessageList {
    /// Returns the number of scheduled messages in this page.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the total count.
    pub fn total(&self) -> i32 {
        self.count
    }
}

impl IntoIterator for ScheduledMessageList {
    type Item = ScheduledMessage;
    type IntoIter = std::vec::IntoIter<ScheduledMessage>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

/// Response from cancelling a scheduled message.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelScheduledMessageResponse {
    /// Scheduled message ID.
    pub id: String,
    /// New status (cancelled).
    pub status: ScheduledMessageStatus,
    /// Credits refunded.
    #[serde(default, alias = "creditsRefunded")]
    pub credits_refunded: i32,
}

// ==================== Batch Messages ====================

/// Status of a message batch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchStatus {
    /// Batch is being processed.
    Processing,
    /// Batch completed successfully.
    Completed,
    /// Some messages in batch failed.
    PartialFailure,
    /// Batch failed.
    Failed,
}

impl std::fmt::Display for BatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatchStatus::Processing => write!(f, "processing"),
            BatchStatus::Completed => write!(f, "completed"),
            BatchStatus::PartialFailure => write!(f, "partial_failure"),
            BatchStatus::Failed => write!(f, "failed"),
        }
    }
}

/// A single message in a batch request.
#[derive(Debug, Clone, Serialize)]
pub struct BatchMessageItem {
    /// Recipient phone number in E.164 format.
    pub to: String,
    /// Message content (max 1600 characters).
    pub text: String,
}

/// Request to send batch messages.
#[derive(Debug, Clone, Serialize)]
pub struct SendBatchRequest {
    /// Messages to send.
    pub messages: Vec<BatchMessageItem>,
    /// Sender ID or phone number (optional, applies to all).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Message type: "marketing" (default, subject to quiet hours) or "transactional" (24/7).
    #[serde(skip_serializing_if = "Option::is_none", rename = "messageType")]
    pub message_type: Option<MessageType>,
}

/// Result of a single message in a batch.
#[derive(Debug, Clone, Deserialize)]
pub struct BatchMessageResult {
    /// Recipient phone number.
    pub to: String,
    /// Message ID if successful.
    #[serde(default, alias = "messageId")]
    pub message_id: Option<String>,
    /// Message status.
    pub status: String,
    /// Error message if failed.
    #[serde(default)]
    pub error: Option<String>,
}

/// Response from sending batch messages.
#[derive(Debug, Clone, Deserialize)]
pub struct BatchMessageResponse {
    /// Unique batch identifier.
    #[serde(alias = "batchId")]
    pub batch_id: String,
    /// Batch status.
    pub status: BatchStatus,
    /// Total messages in batch.
    pub total: i32,
    /// Messages queued.
    pub queued: i32,
    /// Messages sent.
    pub sent: i32,
    /// Messages failed.
    pub failed: i32,
    /// Total credits used.
    #[serde(default, alias = "creditsUsed")]
    pub credits_used: i32,
    /// Results for each message.
    #[serde(default)]
    pub messages: Vec<BatchMessageResult>,
    /// Creation timestamp.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// Completion timestamp.
    #[serde(default, alias = "completedAt")]
    pub completed_at: Option<String>,
}

impl BatchMessageResponse {
    /// Returns true if the batch is still processing.
    pub fn is_processing(&self) -> bool {
        self.status == BatchStatus::Processing
    }

    /// Returns true if the batch completed.
    pub fn is_completed(&self) -> bool {
        self.status == BatchStatus::Completed
    }

    /// Returns true if the batch failed.
    pub fn is_failed(&self) -> bool {
        self.status == BatchStatus::Failed
    }
}

/// A single message in a batch preview.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPreviewItem {
    /// Recipient phone number.
    pub to: String,
    /// Message content.
    pub text: String,
    /// Number of SMS segments.
    #[serde(default = "default_segments")]
    pub segments: i32,
    /// Credits needed for this message.
    #[serde(default)]
    pub credits: i32,
    /// Whether this message can be sent.
    #[serde(default, alias = "canSend")]
    pub can_send: bool,
    /// Reason if message is blocked.
    #[serde(default, alias = "blockReason")]
    pub block_reason: Option<String>,
    /// Destination country code.
    #[serde(default)]
    pub country: Option<String>,
    /// Pricing tier for this message.
    #[serde(default, alias = "pricingTier")]
    pub pricing_tier: Option<String>,
}

/// Response from previewing a batch (dry run).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPreviewResponse {
    /// Whether the entire batch can be sent.
    #[serde(alias = "canSend")]
    pub can_send: bool,
    /// Total number of messages.
    #[serde(default, alias = "totalMessages")]
    pub total_messages: i32,
    /// Number of messages that will be sent.
    #[serde(default, alias = "willSend")]
    pub will_send: i32,
    /// Number of messages that are blocked.
    #[serde(default)]
    pub blocked: i32,
    /// Total credits needed.
    #[serde(default, alias = "creditsNeeded")]
    pub credits_needed: i32,
    /// Current credit balance.
    #[serde(default, alias = "currentBalance")]
    pub current_balance: i32,
    /// Whether there are enough credits.
    #[serde(default, alias = "hasEnoughCredits")]
    pub has_enough_credits: bool,
    /// Preview for each message.
    #[serde(default)]
    pub messages: Vec<BatchPreviewItem>,
    /// Count of block reasons.
    #[serde(default, alias = "blockReasons")]
    pub block_reasons: Option<std::collections::HashMap<String, i32>>,
}

/// Options for listing batches.
#[derive(Debug, Clone, Default)]
pub struct ListBatchesOptions {
    /// Maximum batches to return (default: 20, max: 100).
    pub limit: Option<u32>,
    /// Number of batches to skip.
    pub offset: Option<u32>,
    /// Filter by status.
    pub status: Option<BatchStatus>,
}

impl ListBatchesOptions {
    /// Creates new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.min(100));
        self
    }

    /// Sets the offset.
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Sets the status filter.
    pub fn status(mut self, status: BatchStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub(crate) fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(limit) = self.limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(offset) = self.offset {
            params.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(ref status) = self.status {
            params.push(("status".to_string(), status.to_string()));
        }

        params
    }
}

/// Paginated list of batches.
#[derive(Debug, Clone, Deserialize)]
pub struct BatchList {
    /// Batches in this page.
    pub data: Vec<BatchMessageResponse>,
    /// Total count of batches.
    #[serde(default)]
    pub count: i32,
}

impl BatchList {
    /// Returns the number of batches in this page.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the total count.
    pub fn total(&self) -> i32 {
        self.count
    }
}

impl IntoIterator for BatchList {
    type Item = BatchMessageResponse;
    type IntoIter = std::vec::IntoIter<BatchMessageResponse>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

// ==================== Webhook Types ====================

/// Circuit breaker state for webhooks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CircuitState {
    /// Circuit is closed (healthy).
    Closed,
    /// Circuit is open (failing).
    Open,
    /// Circuit is half-open (testing).
    HalfOpen,
}

impl Default for CircuitState {
    fn default() -> Self {
        CircuitState::Closed
    }
}

/// Webhook mode for event filtering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookMode {
    /// Receive both test and live events.
    All,
    /// Only receive sandbox/test events.
    Test,
    /// Only receive production events (requires verification).
    Live,
}

impl Default for WebhookMode {
    fn default() -> Self {
        WebhookMode::All
    }
}

/// A webhook configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    /// Unique webhook identifier.
    pub id: String,
    /// URL to receive webhook events.
    pub url: String,
    /// List of subscribed event types.
    #[serde(default)]
    pub events: Vec<String>,
    /// Event mode filter (all, test, live).
    #[serde(default)]
    pub mode: WebhookMode,
    /// Whether the webhook is active.
    #[serde(default = "default_true", alias = "isActive")]
    pub is_active: bool,
    /// Number of consecutive failures.
    #[serde(default, alias = "failureCount")]
    pub failure_count: i32,
    /// Circuit breaker state.
    #[serde(default, alias = "circuitState")]
    pub circuit_state: CircuitState,
    /// API version for webhook payloads.
    #[serde(default, alias = "apiVersion")]
    pub api_version: Option<String>,
    /// Total number of delivery attempts.
    #[serde(default, alias = "totalDeliveries")]
    pub total_deliveries: i32,
    /// Number of successful deliveries.
    #[serde(default, alias = "successfulDeliveries")]
    pub successful_deliveries: i32,
    /// Success rate percentage.
    #[serde(default, alias = "successRate")]
    pub success_rate: f64,
    /// Timestamp of last delivery attempt.
    #[serde(default, alias = "lastDeliveryAt")]
    pub last_delivery_at: Option<String>,
    /// Creation timestamp.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// Last update timestamp.
    #[serde(default, alias = "updatedAt")]
    pub updated_at: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Webhook {
    /// Returns true if the webhook is healthy (active and circuit closed).
    pub fn is_healthy(&self) -> bool {
        self.is_active && self.circuit_state == CircuitState::Closed
    }

    /// Returns true if the circuit breaker is open.
    pub fn is_circuit_open(&self) -> bool {
        self.circuit_state == CircuitState::Open
    }
}

/// Response from creating a webhook (includes secret).
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookCreatedResponse {
    /// The created webhook.
    #[serde(default)]
    pub webhook: Option<Webhook>,
    /// The webhook secret for signature verification.
    #[serde(default)]
    pub secret: String,
    // Flatten webhook fields for direct responses
    #[serde(flatten)]
    pub data: Option<Webhook>,
}

impl WebhookCreatedResponse {
    /// Gets the webhook, checking both nested and flattened forms.
    pub fn get_webhook(&self) -> Option<&Webhook> {
        self.webhook.as_ref().or(self.data.as_ref())
    }
}

/// Request to create a webhook.
#[derive(Debug, Clone, Serialize)]
pub struct CreateWebhookRequest {
    /// URL to receive webhook events.
    pub url: String,
    /// List of event types to subscribe to.
    pub events: Vec<String>,
    /// Event mode filter (all, test, live). Live requires verification.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<WebhookMode>,
    /// API version for webhook payloads.
    #[serde(skip_serializing_if = "Option::is_none", rename = "apiVersion")]
    pub api_version: Option<String>,
}

/// Request to update a webhook.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateWebhookRequest {
    /// New URL to receive webhook events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// New list of event types to subscribe to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,
    /// Whether the webhook is active.
    #[serde(skip_serializing_if = "Option::is_none", rename = "is_active")]
    pub is_active: Option<bool>,
    /// Event mode filter (all, test, live).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<WebhookMode>,
}

/// A webhook delivery attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDelivery {
    /// Unique delivery identifier.
    pub id: String,
    /// Webhook ID this delivery belongs to.
    #[serde(alias = "webhookId")]
    pub webhook_id: String,
    /// Event type that triggered this delivery.
    #[serde(alias = "eventType")]
    pub event_type: String,
    /// HTTP status code from the endpoint.
    #[serde(default, alias = "httpStatus")]
    pub http_status: i32,
    /// Whether the delivery was successful.
    #[serde(default)]
    pub success: bool,
    /// Attempt number (1-based).
    #[serde(default = "default_one", alias = "attemptNumber")]
    pub attempt_number: i32,
    /// Error message if the delivery failed.
    #[serde(default, alias = "errorMessage")]
    pub error_message: Option<String>,
    /// Response time in milliseconds.
    #[serde(default, alias = "responseTimeMs")]
    pub response_time_ms: i32,
    /// Timestamp of the delivery attempt.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
}

fn default_one() -> i32 {
    1
}

/// List of webhook deliveries.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookDeliveryList {
    /// Deliveries in this page.
    #[serde(default, alias = "deliveries")]
    pub data: Vec<WebhookDelivery>,
    /// Total count of deliveries.
    #[serde(default)]
    pub total: i32,
    /// Whether there are more deliveries.
    #[serde(default, alias = "hasMore")]
    pub has_more: bool,
}

/// Result from testing a webhook.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookTestResult {
    /// Whether the test was successful.
    #[serde(default)]
    pub success: bool,
    /// HTTP status code from the endpoint.
    #[serde(default, alias = "statusCode")]
    pub status_code: i32,
    /// Response time in milliseconds.
    #[serde(default, alias = "responseTimeMs")]
    pub response_time_ms: i32,
    /// Error message if the test failed.
    #[serde(default)]
    pub error: Option<String>,
}

/// Response from rotating a webhook secret.
#[derive(Debug, Clone, Deserialize)]
pub struct WebhookSecretRotation {
    /// The new webhook secret.
    #[serde(default)]
    pub secret: String,
    /// Timestamp of the rotation.
    #[serde(default, alias = "rotatedAt")]
    pub rotated_at: Option<String>,
}

/// Options for listing webhook deliveries.
#[derive(Debug, Clone, Default)]
pub struct ListDeliveriesOptions {
    /// Maximum deliveries to return.
    pub limit: Option<u32>,
    /// Number of deliveries to skip.
    pub offset: Option<u32>,
}

impl ListDeliveriesOptions {
    /// Creates new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.min(100));
        self
    }

    /// Sets the offset.
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub(crate) fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(limit) = self.limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(offset) = self.offset {
            params.push(("offset".to_string(), offset.to_string()));
        }

        params
    }
}

// ==================== Account Types ====================

/// Credit balance information.
#[derive(Debug, Clone, Deserialize)]
pub struct Credits {
    /// Total credit balance.
    #[serde(default)]
    pub balance: i32,
    /// Available credits for use.
    #[serde(default, alias = "availableBalance")]
    pub available_balance: i32,
    /// Credits pending from purchases.
    #[serde(default, alias = "pendingCredits")]
    pub pending_credits: i32,
    /// Credits reserved for scheduled messages.
    #[serde(default, alias = "reservedCredits")]
    pub reserved_credits: i32,
    /// Currency code.
    #[serde(default = "default_currency")]
    pub currency: String,
}

fn default_currency() -> String {
    "USD".to_string()
}

impl Credits {
    /// Returns true if there are credits available.
    pub fn has_credits(&self) -> bool {
        self.available_balance > 0
    }
}

/// Credit transaction type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    /// Credit purchase.
    Purchase,
    /// Credit usage (sending messages).
    Usage,
    /// Credit refund.
    Refund,
    /// Bonus credits.
    Bonus,
    /// Manual adjustment.
    Adjustment,
}

/// A credit transaction.
#[derive(Debug, Clone, Deserialize)]
pub struct CreditTransaction {
    /// Unique transaction identifier.
    pub id: String,
    /// Transaction type.
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    /// Amount (positive for credits, negative for debits).
    #[serde(default)]
    pub amount: i32,
    /// Balance after this transaction.
    #[serde(default, alias = "balanceAfter")]
    pub balance_after: i32,
    /// Transaction description.
    #[serde(default)]
    pub description: Option<String>,
    /// Reference ID (e.g., message ID, order ID).
    #[serde(default, alias = "referenceId")]
    pub reference_id: Option<String>,
    /// Transaction timestamp.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
}

impl CreditTransaction {
    /// Returns true if this is a credit (positive amount).
    pub fn is_credit(&self) -> bool {
        self.amount > 0
    }

    /// Returns true if this is a debit (negative amount).
    pub fn is_debit(&self) -> bool {
        self.amount < 0
    }
}

/// List of credit transactions.
#[derive(Debug, Clone, Deserialize)]
pub struct CreditTransactionList {
    /// Transactions in this page.
    #[serde(default, alias = "transactions")]
    pub data: Vec<CreditTransaction>,
    /// Total count of transactions.
    #[serde(default)]
    pub total: i32,
    /// Whether there are more transactions.
    #[serde(default, alias = "hasMore")]
    pub has_more: bool,
}

/// Options for listing transactions.
#[derive(Debug, Clone, Default)]
pub struct ListTransactionsOptions {
    /// Maximum transactions to return.
    pub limit: Option<u32>,
    /// Number of transactions to skip.
    pub offset: Option<u32>,
    /// Filter by transaction type.
    pub transaction_type: Option<TransactionType>,
}

impl ListTransactionsOptions {
    /// Creates new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.min(100));
        self
    }

    /// Sets the offset.
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Sets the transaction type filter.
    pub fn transaction_type(mut self, t: TransactionType) -> Self {
        self.transaction_type = Some(t);
        self
    }

    pub(crate) fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(limit) = self.limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(offset) = self.offset {
            params.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(ref t) = self.transaction_type {
            let type_str = match t {
                TransactionType::Purchase => "purchase",
                TransactionType::Usage => "usage",
                TransactionType::Refund => "refund",
                TransactionType::Bonus => "bonus",
                TransactionType::Adjustment => "adjustment",
            };
            params.push(("type".to_string(), type_str.to_string()));
        }

        params
    }
}

/// An API key.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ApiKey {
    /// Unique API key identifier.
    pub id: String,
    /// Display name for the API key.
    #[serde(default)]
    pub name: String,
    /// Key prefix for identification.
    #[serde(default)]
    pub prefix: String,
    /// Last time the key was used.
    #[serde(default, alias = "lastUsedAt")]
    pub last_used_at: Option<String>,
    /// Creation timestamp.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// Expiration timestamp.
    #[serde(default, alias = "expiresAt")]
    pub expires_at: Option<String>,
    /// Whether the key is active.
    #[serde(default = "default_true", alias = "isActive")]
    pub is_active: bool,
}

/// Response from creating an API key.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateApiKeyResponse {
    /// The created API key.
    #[serde(default, alias = "apiKey")]
    pub api_key: Option<ApiKey>,
    /// The full API key value (only shown once).
    #[serde(default)]
    pub key: String,
}

/// Request to create an API key.
#[derive(Debug, Clone, Serialize)]
pub struct CreateApiKeyRequest {
    /// Display name for the API key.
    pub name: String,
    /// Optional expiration date.
    #[serde(skip_serializing_if = "Option::is_none", rename = "expires_at")]
    pub expires_at: Option<String>,
}

/// Account verification status.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AccountVerification {
    /// Whether email is verified.
    #[serde(default, alias = "emailVerified")]
    pub email_verified: bool,
    /// Whether phone is verified.
    #[serde(default, alias = "phoneVerified")]
    pub phone_verified: bool,
    /// Whether identity is verified.
    #[serde(default, alias = "identityVerified")]
    pub identity_verified: bool,
}

impl AccountVerification {
    /// Returns true if fully verified.
    pub fn is_fully_verified(&self) -> bool {
        self.email_verified && self.phone_verified && self.identity_verified
    }
}

/// Account rate limits.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountLimits {
    /// Maximum messages per second.
    #[serde(default = "default_mps", alias = "messagesPerSecond")]
    pub messages_per_second: i32,
    /// Maximum messages per day.
    #[serde(default = "default_mpd", alias = "messagesPerDay")]
    pub messages_per_day: i32,
    /// Maximum batch size.
    #[serde(default = "default_batch", alias = "maxBatchSize")]
    pub max_batch_size: i32,
}

fn default_mps() -> i32 {
    10
}
fn default_mpd() -> i32 {
    10000
}
fn default_batch() -> i32 {
    1000
}

impl Default for AccountLimits {
    fn default() -> Self {
        Self {
            messages_per_second: 10,
            messages_per_day: 10000,
            max_batch_size: 1000,
        }
    }
}

/// Account information.
#[derive(Debug, Clone, Deserialize)]
pub struct Account {
    /// Unique account identifier.
    pub id: String,
    /// Account email address.
    #[serde(default)]
    pub email: String,
    /// Account holder name.
    #[serde(default)]
    pub name: Option<String>,
    /// Company name.
    #[serde(default, alias = "companyName")]
    pub company_name: Option<String>,
    /// Verification status.
    #[serde(default)]
    pub verification: AccountVerification,
    /// Rate limits.
    #[serde(default)]
    pub limits: AccountLimits,
    /// Account creation timestamp.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
}
