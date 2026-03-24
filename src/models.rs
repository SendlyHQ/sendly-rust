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
    /// Message bounced (carrier rejected).
    Bounced,
    /// Message is being retried after a transient failure.
    Retrying,
}

impl std::fmt::Display for MessageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageStatus::Queued => write!(f, "queued"),
            MessageStatus::Sent => write!(f, "sent"),
            MessageStatus::Delivered => write!(f, "delivered"),
            MessageStatus::Failed => write!(f, "failed"),
            MessageStatus::Bounced => write!(f, "bounced"),
            MessageStatus::Retrying => write!(f, "retrying"),
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
    /// Carrier message ID for tracking.
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
    /// Number of delivery retry attempts.
    #[serde(default, alias = "retryCount")]
    pub retry_count: i32,
    /// Creation timestamp.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// Last update timestamp.
    #[serde(default, alias = "updatedAt")]
    pub updated_at: Option<String>,
    /// Delivery timestamp (if delivered).
    #[serde(default, alias = "deliveredAt")]
    pub delivered_at: Option<String>,
    /// Custom metadata attached to the message.
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    /// AI classification metadata for inbound messages.
    #[serde(default, rename = "aiMetadata")]
    pub ai_metadata: Option<AiMetadata>,
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

/// AI classification metadata for an inbound message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMetadata {
    /// Classified intent of the message.
    pub intent: String,
    /// Confidence score for the intent (0-1).
    #[serde(rename = "intentConfidence")]
    pub intent_confidence: f64,
    /// Classified sentiment of the message.
    pub sentiment: String,
    /// Confidence score for the sentiment (0-1).
    #[serde(rename = "sentimentConfidence")]
    pub sentiment_confidence: f64,
    /// ISO 8601 timestamp of when classification occurred.
    #[serde(rename = "classifiedAt")]
    pub classified_at: String,
    /// AI model used for classification.
    pub model: String,
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
    /// Media URLs for MMS messages.
    #[serde(skip_serializing_if = "Option::is_none", rename = "mediaUrls")]
    pub media_urls: Option<Vec<String>>,
    /// Custom metadata to attach to the message (max 4KB).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// An uploaded media file.
#[derive(Debug, Clone, Deserialize)]
pub struct MediaFile {
    pub id: String,
    pub url: String,
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "sizeBytes")]
    pub size_bytes: i64,
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
    /// Custom metadata to attach to the message (max 4KB).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
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
    /// Per-message metadata (max 4KB, merged with batch metadata).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
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
    /// Shared metadata for all messages in the batch (max 4KB).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
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

#[derive(Debug, Clone, Serialize)]
pub struct TransferCreditsRequest {
    #[serde(rename = "targetOrganizationId")]
    pub target_organization_id: String,
    pub amount: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TransferCreditsResponse {
    pub success: bool,
    pub amount: i32,
    #[serde(alias = "sourceBalance")]
    pub source_balance: i32,
    #[serde(alias = "targetBalance")]
    pub target_balance: i32,
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

// ==================== Enterprise Types ====================

#[derive(Debug, Clone, Deserialize)]
pub struct EnterpriseWorkspaceSummary {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default, alias = "verificationStatus")]
    pub verification_status: Option<String>,
    #[serde(default, alias = "verificationType")]
    pub verification_type: Option<String>,
    #[serde(default, alias = "tollFreeNumber")]
    pub toll_free_number: Option<String>,
    #[serde(default, alias = "creditBalance")]
    pub credit_balance: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnterpriseAccount {
    pub id: String,
    #[serde(default, alias = "maxWorkspaces")]
    pub max_workspaces: i32,
    #[serde(default, alias = "workspaceCount")]
    pub workspace_count: i32,
    #[serde(default)]
    pub workspaces: Vec<EnterpriseWorkspaceSummary>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CreateWorkspaceRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnterpriseWorkspace {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnterpriseWorkspaceVerification {
    #[serde(default)]
    pub status: String,
    #[serde(default, rename = "type")]
    pub verification_type: Option<String>,
    #[serde(default, alias = "tollFreeNumber")]
    pub toll_free_number: Option<String>,
    #[serde(default, alias = "businessName")]
    pub business_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnterpriseWorkspaceDetail {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    #[serde(default)]
    pub verification: Option<EnterpriseWorkspaceVerification>,
    #[serde(default)]
    pub credits: i64,
    #[serde(default, alias = "keyCount")]
    pub key_count: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteWorkspaceResponse {
    #[serde(default)]
    pub success: bool,
    #[serde(default, alias = "deletedId")]
    pub deleted_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubmitVerificationRequest {
    #[serde(rename = "businessName")]
    pub business_name: String,
    pub website: String,
    pub address: VerificationAddress,
    pub contact: VerificationContact,
    #[serde(rename = "useCase")]
    pub use_case: String,
    #[serde(rename = "useCaseSummary")]
    pub use_case_summary: String,
    #[serde(rename = "sampleMessages")]
    pub sample_messages: String,
    #[serde(rename = "optInWorkflow")]
    pub opt_in_workflow: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "doingBusinessAs")]
    pub doing_business_as: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "brnType")]
    pub brn_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "brnCountry")]
    pub brn_country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "optInImageUrls")]
    pub opt_in_image_urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "monthlyVolume")]
    pub monthly_volume: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "additionalInformation"
    )]
    pub additional_information: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "ageGatedContent")]
    pub age_gated_content: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "isvReseller")]
    pub isv_reseller: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "privacyUrl")]
    pub privacy_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "termsUrl")]
    pub terms_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "entityType")]
    pub entity_type: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerificationAddress {
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address2: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerificationContact {
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    pub email: String,
    pub phone: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubmitVerificationResponse {
    #[serde(default, alias = "verificationId")]
    pub verification_id: Option<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default, alias = "tollFreeNumber")]
    pub toll_free_number: Option<String>,
    #[serde(default, alias = "businessName")]
    pub business_name: Option<String>,
    #[serde(default, alias = "telnyxProfileId")]
    pub telnyx_profile_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InheritVerificationRequest {
    #[serde(rename = "sourceWorkspaceId")]
    pub source_workspace_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InheritVerificationResponse {
    #[serde(default, alias = "verificationId")]
    pub verification_id: Option<String>,
    #[serde(default)]
    pub status: String,
    #[serde(default, rename = "type")]
    pub verification_type: Option<String>,
    #[serde(default, alias = "tollFreeNumber")]
    pub toll_free_number: Option<String>,
    #[serde(default, alias = "inheritedFrom")]
    pub inherited_from: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceVerificationStatus {
    #[serde(default)]
    pub status: String,
    #[serde(default, alias = "verificationId")]
    pub verification_id: Option<String>,
    #[serde(default, rename = "type")]
    pub verification_type: Option<String>,
    #[serde(default, alias = "tollFreeNumber")]
    pub toll_free_number: Option<String>,
    #[serde(default, alias = "businessName")]
    pub business_name: Option<String>,
    #[serde(default, alias = "submittedAt")]
    pub submitted_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorkspaceTransferCreditsRequest {
    #[serde(rename = "sourceWorkspaceId")]
    pub source_workspace_id: String,
    pub amount: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceTransferCreditsResponse {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub amount: i32,
    #[serde(default, alias = "sourceBalance")]
    pub source_balance: i64,
    #[serde(default, alias = "targetBalance")]
    pub target_balance: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceCredits {
    #[serde(default)]
    pub balance: i64,
    #[serde(default, alias = "lifetimeCredits")]
    pub lifetime_credits: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateWorkspaceKeyRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub key_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

impl CreateWorkspaceKeyRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            key_type: None,
            scopes: None,
        }
    }

    pub fn key_type(mut self, key_type: impl Into<String>) -> Self {
        self.key_type = Some(key_type.into());
        self
    }

    pub fn scopes(mut self, scopes: Vec<impl Into<String>>) -> Self {
        self.scopes = Some(scopes.into_iter().map(|s| s.into()).collect());
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceKeyResponse {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub key: String,
    #[serde(default, alias = "keyPrefix")]
    pub key_prefix: String,
    #[serde(default, rename = "type")]
    pub key_type: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceKey {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, alias = "keyPrefix")]
    pub key_prefix: String,
    #[serde(default, rename = "type")]
    pub key_type: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default, alias = "lastUsedAt")]
    pub last_used_at: Option<String>,
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RevokeKeyResponse {
    #[serde(default)]
    pub success: bool,
    #[serde(default, alias = "revokedId")]
    pub revoked_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProvisionWorkspaceRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "sourceWorkspaceId")]
    pub source_workspace_id: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "inheritWithNewNumber"
    )]
    pub inherit_with_new_number: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<SubmitVerificationRequest>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "creditAmount")]
    pub credit_amount: Option<i32>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "creditSourceWorkspaceId"
    )]
    pub credit_source_workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "keyName")]
    pub key_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "keyType")]
    pub key_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "webhookUrl")]
    pub webhook_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "generateOptInPage")]
    pub generate_opt_in_page: Option<bool>,
}

impl ProvisionWorkspaceRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source_workspace_id: None,
            inherit_with_new_number: None,
            verification: None,
            credit_amount: None,
            credit_source_workspace_id: None,
            key_name: None,
            key_type: None,
            webhook_url: None,
            generate_opt_in_page: None,
        }
    }

    pub fn source_workspace_id(mut self, id: impl Into<String>) -> Self {
        self.source_workspace_id = Some(id.into());
        self
    }

    pub fn key_name(mut self, name: impl Into<String>) -> Self {
        self.key_name = Some(name.into());
        self
    }

    pub fn key_type(mut self, key_type: impl Into<String>) -> Self {
        self.key_type = Some(key_type.into());
        self
    }

    pub fn webhook_url(mut self, url: impl Into<String>) -> Self {
        self.webhook_url = Some(url.into());
        self
    }

    pub fn credit_amount(mut self, amount: i32) -> Self {
        self.credit_amount = Some(amount);
        self
    }

    pub fn credit_source_workspace_id(mut self, id: impl Into<String>) -> Self {
        self.credit_source_workspace_id = Some(id.into());
        self
    }

    pub fn inherit_with_new_number(mut self, value: bool) -> Self {
        self.inherit_with_new_number = Some(value);
        self
    }

    pub fn generate_opt_in_page(mut self, value: bool) -> Self {
        self.generate_opt_in_page = Some(value);
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProvisionWorkspaceResponse {
    #[serde(default)]
    pub workspace: Option<EnterpriseWorkspace>,
    #[serde(default)]
    pub verification: Option<serde_json::Value>,
    #[serde(default)]
    pub credits: Option<serde_json::Value>,
    #[serde(default)]
    pub key: Option<serde_json::Value>,
    #[serde(default)]
    pub webhook: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerateBusinessPageRequest {
    #[serde(rename = "businessName")]
    pub business_name: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "useCase")]
    pub use_case: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "useCaseSummary")]
    pub use_case_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "contactEmail")]
    pub contact_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "contactPhone")]
    pub contact_phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "businessAddress")]
    pub business_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "socialUrl")]
    pub social_url: Option<String>,
}

impl GenerateBusinessPageRequest {
    pub fn new(business_name: impl Into<String>) -> Self {
        Self {
            business_name: business_name.into(),
            use_case: None,
            use_case_summary: None,
            contact_email: None,
            contact_phone: None,
            business_address: None,
            social_url: None,
        }
    }

    pub fn use_case(mut self, use_case: impl Into<String>) -> Self {
        self.use_case = Some(use_case.into());
        self
    }

    pub fn use_case_summary(mut self, summary: impl Into<String>) -> Self {
        self.use_case_summary = Some(summary.into());
        self
    }

    pub fn contact_email(mut self, email: impl Into<String>) -> Self {
        self.contact_email = Some(email.into());
        self
    }

    pub fn contact_phone(mut self, phone: impl Into<String>) -> Self {
        self.contact_phone = Some(phone.into());
        self
    }

    pub fn business_address(mut self, address: impl Into<String>) -> Self {
        self.business_address = Some(address.into());
        self
    }

    pub fn social_url(mut self, url: impl Into<String>) -> Self {
        self.social_url = Some(url.into());
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GenerateBusinessPageResponse {
    pub slug: String,
    pub url: String,
    #[serde(alias = "pageId")]
    pub page_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerificationDocumentUploadResponse {
    pub url: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetEnterpriseWebhookRequest {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnterpriseWebhook {
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnterpriseWebhookTestResult {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default, alias = "statusCode")]
    pub status_code: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnalyticsOverview {
    #[serde(default, alias = "totalMessages")]
    pub total_messages: i64,
    #[serde(default, alias = "deliveredMessages")]
    pub delivered_messages: i64,
    #[serde(default, alias = "failedMessages")]
    pub failed_messages: i64,
    #[serde(default, alias = "deliveryRate")]
    pub delivery_rate: f64,
    #[serde(default, alias = "totalCreditsUsed")]
    pub total_credits_used: i64,
    #[serde(default, alias = "activeWorkspaces")]
    pub active_workspaces: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageDataPoint {
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub sent: i64,
    #[serde(default)]
    pub delivered: i64,
    #[serde(default)]
    pub failed: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessagesAnalytics {
    #[serde(default)]
    pub period: String,
    #[serde(default)]
    pub data: Vec<MessageDataPoint>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeliveryByWorkspace {
    #[serde(default, alias = "workspaceId")]
    pub workspace_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub sent: i64,
    #[serde(default)]
    pub delivered: i64,
    #[serde(default)]
    pub failed: i64,
    #[serde(default)]
    pub rate: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreditDataPoint {
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub used: i64,
    #[serde(default)]
    pub transferred: i64,
    #[serde(default)]
    pub purchased: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreditsAnalytics {
    #[serde(default)]
    pub period: String,
    #[serde(default)]
    pub data: Vec<CreditDataPoint>,
}

#[derive(Debug, Clone, Default)]
pub struct AnalyticsPeriod {
    pub period: Option<String>,
    pub workspace_id: Option<String>,
}

impl AnalyticsPeriod {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn period(mut self, period: impl Into<String>) -> Self {
        self.period = Some(period.into());
        self
    }

    pub fn workspace_id(mut self, id: impl Into<String>) -> Self {
        self.workspace_id = Some(id.into());
        self
    }

    pub(crate) fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(ref period) = self.period {
            params.push(("period".to_string(), period.clone()));
        }
        if let Some(ref id) = self.workspace_id {
            params.push(("workspaceId".to_string(), id.clone()));
        }
        params
    }
}

// ==================== Enterprise Opt-In Pages ====================

#[derive(Debug, Clone, Deserialize)]
pub struct OptInPage {
    pub id: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub url: String,
    #[serde(default, alias = "businessName")]
    pub business_name: String,
    #[serde(default, alias = "useCase")]
    pub use_case: Option<String>,
    #[serde(default = "default_true", alias = "isActive")]
    pub is_active: bool,
    #[serde(default, alias = "viewCount")]
    pub view_count: i32,
    #[serde(default, alias = "logoUrl")]
    pub logo_url: Option<String>,
    #[serde(default, alias = "headerColor")]
    pub header_color: Option<String>,
    #[serde(default, alias = "buttonColor")]
    pub button_color: Option<String>,
    #[serde(default, alias = "customHeadline")]
    pub custom_headline: Option<String>,
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateOptInPageRequest {
    #[serde(rename = "businessName")]
    pub business_name: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "useCase")]
    pub use_case: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "useCaseSummary")]
    pub use_case_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "sampleMessages")]
    pub sample_messages: Option<String>,
}

impl CreateOptInPageRequest {
    pub fn new(business_name: impl Into<String>) -> Self {
        Self {
            business_name: business_name.into(),
            use_case: None,
            use_case_summary: None,
            sample_messages: None,
        }
    }

    pub fn use_case(mut self, use_case: impl Into<String>) -> Self {
        self.use_case = Some(use_case.into());
        self
    }

    pub fn use_case_summary(mut self, summary: impl Into<String>) -> Self {
        self.use_case_summary = Some(summary.into());
        self
    }

    pub fn sample_messages(mut self, messages: impl Into<String>) -> Self {
        self.sample_messages = Some(messages.into());
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateOptInPageResponse {
    pub id: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub url: String,
    #[serde(default, alias = "businessName")]
    pub business_name: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateOptInPageRequest {
    #[serde(skip_serializing_if = "Option::is_none", rename = "logoUrl")]
    pub logo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "headerColor")]
    pub header_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "buttonColor")]
    pub button_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "customHeadline")]
    pub custom_headline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "customBenefits")]
    pub custom_benefits: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteOptInPageResponse {
    #[serde(default)]
    pub success: bool,
}

// ==================== Enterprise Workspace Webhooks ====================

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceWebhookConfig {
    pub id: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default = "default_true", alias = "isActive")]
    pub is_active: bool,
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetWorkspaceWebhookRequest {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl SetWorkspaceWebhookRequest {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            events: None,
            description: None,
        }
    }

    pub fn events(mut self, events: Vec<impl Into<String>>) -> Self {
        self.events = Some(events.into_iter().map(|e| e.into()).collect());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetWorkspaceWebhookResponse {
    pub id: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub secret: Option<String>,
    #[serde(default)]
    pub created: Option<bool>,
    #[serde(default)]
    pub updated: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceWebhookTestResult {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default, alias = "statusCode")]
    pub status_code: Option<i32>,
}

// ==================== Enterprise Suspend/Resume ====================

#[derive(Debug, Clone, Serialize, Default)]
pub struct SuspendWorkspaceRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SuspendWorkspaceResponse {
    pub id: String,
    #[serde(default)]
    pub status: String,
    #[serde(default, alias = "suspendedAt")]
    pub suspended_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResumeWorkspaceResponse {
    pub id: String,
    #[serde(default)]
    pub status: String,
}

// ==================== Enterprise Pool Credits ====================

#[derive(Debug, Clone, Deserialize)]
pub struct PoolCredits {
    #[serde(default)]
    pub balance: i64,
    #[serde(default, alias = "lifetimeCredits")]
    pub lifetime_credits: i64,
    #[serde(default, alias = "reservedBalance")]
    pub reserved_balance: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DepositCreditsRequest {
    pub amount: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

// ==================== Enterprise Auto Top-Up ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoTopUpSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub threshold: i32,
    #[serde(default)]
    pub amount: i32,
    #[serde(default, alias = "sourceWorkspaceId")]
    pub source_workspace_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateAutoTopUpRequest {
    pub enabled: bool,
    pub threshold: i32,
    pub amount: i32,
    #[serde(skip_serializing_if = "Option::is_none", rename = "sourceWorkspaceId")]
    pub source_workspace_id: Option<String>,
}

// ==================== Enterprise Billing ====================

#[derive(Debug, Clone, Default)]
pub struct BillingBreakdownOptions {
    pub period: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl BillingBreakdownOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn period(mut self, period: impl Into<String>) -> Self {
        self.period = Some(period.into());
        self
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub(crate) fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(ref period) = self.period {
            params.push(("period".to_string(), period.clone()));
        }
        if let Some(page) = self.page {
            params.push(("page".to_string(), page.to_string()));
        }
        if let Some(limit) = self.limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        params
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceBillingItem {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, alias = "creditsUsed")]
    pub credits_used: i64,
    #[serde(default, alias = "creditsPurchased")]
    pub credits_purchased: i64,
    #[serde(default, alias = "creditsTransferredIn")]
    pub credits_transferred_in: i64,
    #[serde(default, alias = "creditsTransferredOut")]
    pub credits_transferred_out: i64,
    #[serde(default, alias = "messagesSent")]
    pub messages_sent: i64,
    #[serde(default, alias = "messagesDelivered")]
    pub messages_delivered: i64,
    #[serde(default, alias = "workspaceFee")]
    pub workspace_fee: f64,
    #[serde(default, alias = "allocatedPlatformFee")]
    pub allocated_platform_fee: f64,
    #[serde(default, alias = "totalCost")]
    pub total_cost: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BillingBreakdownSummary {
    #[serde(default, alias = "platformFee")]
    pub platform_fee: f64,
    #[serde(default, alias = "totalWorkspaceFees")]
    pub total_workspace_fees: f64,
    #[serde(default, alias = "totalCreditsUsed")]
    pub total_credits_used: i64,
    #[serde(default, alias = "totalCost")]
    pub total_cost: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BillingBreakdown {
    #[serde(default)]
    pub period: String,
    #[serde(default)]
    pub summary: Option<BillingBreakdownSummary>,
    #[serde(default)]
    pub workspaces: Vec<WorkspaceBillingItem>,
}

// ==================== Enterprise Bulk Provision ====================

#[derive(Debug, Clone, Serialize)]
pub struct BulkProvisionWorkspace {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "sourceWorkspaceId")]
    pub source_workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "creditAmount")]
    pub credit_amount: Option<i32>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "creditSourceWorkspaceId"
    )]
    pub credit_source_workspace_id: Option<String>,
}

impl BulkProvisionWorkspace {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source_workspace_id: None,
            credit_amount: None,
            credit_source_workspace_id: None,
        }
    }

    pub fn source_workspace_id(mut self, id: impl Into<String>) -> Self {
        self.source_workspace_id = Some(id.into());
        self
    }

    pub fn credit_amount(mut self, amount: i32) -> Self {
        self.credit_amount = Some(amount);
        self
    }

    pub fn credit_source_workspace_id(mut self, id: impl Into<String>) -> Self {
        self.credit_source_workspace_id = Some(id.into());
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkProvisionRequest {
    pub workspaces: Vec<BulkProvisionWorkspace>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkProvisionResultItem {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub status: String,
    #[serde(default, alias = "workspaceId")]
    pub workspace_id: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub warning: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkProvisionSummary {
    #[serde(default)]
    pub total: i32,
    #[serde(default)]
    pub succeeded: i32,
    #[serde(default)]
    pub failed: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkProvisionResult {
    #[serde(default)]
    pub results: Vec<BulkProvisionResultItem>,
    #[serde(default)]
    pub summary: Option<BulkProvisionSummary>,
}

// ==================== Enterprise Custom Domain ====================

#[derive(Debug, Clone, Serialize)]
pub struct SetCustomDomainRequest {
    pub domain: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DnsRecord {
    #[serde(default, rename = "type")]
    pub record_type: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DnsInstructions {
    #[serde(default)]
    pub cname: Option<DnsRecord>,
    #[serde(default)]
    pub txt: Option<DnsRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetCustomDomainResponse {
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub verified: bool,
    #[serde(default, alias = "dnsInstructions")]
    pub dns_instructions: Option<DnsInstructions>,
}

// ==================== Enterprise Invitations ====================

#[derive(Debug, Clone, Serialize)]
pub struct SendInvitationRequest {
    pub email: String,
    pub role: String,
}

impl SendInvitationRequest {
    pub fn new(email: impl Into<String>, role: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            role: role.into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Invitation {
    pub id: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub status: String,
    #[serde(default, alias = "expiresAt")]
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancelInvitationResponse {
    #[serde(default)]
    pub success: bool,
}

// ==================== Enterprise Quota ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaSettings {
    #[serde(default, alias = "monthlyMessageQuota")]
    pub monthly_message_quota: Option<i64>,
    #[serde(default, alias = "messagesThisMonth")]
    pub messages_this_month: i64,
    #[serde(default, alias = "quotaResetAt")]
    pub quota_reset_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateQuotaRequest {
    #[serde(rename = "monthlyMessageQuota")]
    pub monthly_message_quota: Option<i64>,
}

// ==================== Conversations ====================

/// Conversation status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConversationStatus {
    /// Conversation is active.
    Active,
    /// Conversation is closed.
    Closed,
}

impl std::fmt::Display for ConversationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversationStatus::Active => write!(f, "active"),
            ConversationStatus::Closed => write!(f, "closed"),
        }
    }
}

/// An SMS conversation thread.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique conversation identifier.
    pub id: String,
    /// Phone number of the contact.
    #[serde(alias = "phoneNumber")]
    pub phone_number: String,
    /// Conversation status.
    pub status: ConversationStatus,
    /// Number of unread messages.
    #[serde(default, alias = "unreadCount")]
    pub unread_count: i32,
    /// Total number of messages.
    #[serde(default, alias = "messageCount")]
    pub message_count: i32,
    /// Text of the last message.
    #[serde(default, alias = "lastMessageText")]
    pub last_message_text: Option<String>,
    /// When the last message was sent/received.
    #[serde(default, alias = "lastMessageAt")]
    pub last_message_at: Option<String>,
    /// Direction of the last message.
    #[serde(default, alias = "lastMessageDirection")]
    pub last_message_direction: Option<String>,
    /// Custom metadata.
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    /// Conversation tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Associated contact ID.
    #[serde(default, alias = "contactId")]
    pub contact_id: Option<String>,
    /// When the conversation was created.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// When the conversation was last updated.
    #[serde(default, alias = "updatedAt")]
    pub updated_at: Option<String>,
}

/// Pagination info for conversation lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationPagination {
    /// Total number of conversations.
    #[serde(default)]
    pub total: i64,
    /// Maximum number of conversations returned.
    #[serde(default)]
    pub limit: i32,
    /// Number of conversations skipped.
    #[serde(default)]
    pub offset: i32,
    /// Whether more conversations are available.
    #[serde(default, alias = "hasMore")]
    pub has_more: bool,
}

/// Response from listing conversations.
#[derive(Debug, Clone, Deserialize)]
pub struct ConversationListResponse {
    /// List of conversations.
    #[serde(default)]
    pub data: Vec<Conversation>,
    /// Pagination info.
    pub pagination: ConversationPagination,
}

/// Messages within a conversation.
#[derive(Debug, Clone, Deserialize)]
pub struct ConversationMessages {
    /// List of messages.
    #[serde(default)]
    pub data: Vec<Message>,
    /// Pagination info.
    pub pagination: ConversationPagination,
}

/// A conversation with its messages.
#[derive(Debug, Clone, Deserialize)]
pub struct ConversationWithMessages {
    /// Unique conversation identifier.
    pub id: String,
    /// Phone number of the contact.
    #[serde(alias = "phoneNumber")]
    pub phone_number: String,
    /// Conversation status.
    pub status: ConversationStatus,
    /// Number of unread messages.
    #[serde(default, alias = "unreadCount")]
    pub unread_count: i32,
    /// Total number of messages.
    #[serde(default, alias = "messageCount")]
    pub message_count: i32,
    /// Text of the last message.
    #[serde(default, alias = "lastMessageText")]
    pub last_message_text: Option<String>,
    /// When the last message was sent/received.
    #[serde(default, alias = "lastMessageAt")]
    pub last_message_at: Option<String>,
    /// Direction of the last message.
    #[serde(default, alias = "lastMessageDirection")]
    pub last_message_direction: Option<String>,
    /// Custom metadata.
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    /// Conversation tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Associated contact ID.
    #[serde(default, alias = "contactId")]
    pub contact_id: Option<String>,
    /// When the conversation was created.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// When the conversation was last updated.
    #[serde(default, alias = "updatedAt")]
    pub updated_at: Option<String>,
    /// Conversation messages (when include_messages=true).
    #[serde(default)]
    pub messages: Option<ConversationMessages>,
}

/// Options for listing conversations.
#[derive(Debug, Clone, Default)]
pub struct ListConversationsOptions {
    /// Maximum number of conversations to return.
    pub limit: Option<i32>,
    /// Number of conversations to skip.
    pub offset: Option<i32>,
    /// Filter by conversation status.
    pub status: Option<ConversationStatus>,
}

impl ListConversationsOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: i32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn status(mut self, status: ConversationStatus) -> Self {
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

/// Options for getting a single conversation.
#[derive(Debug, Clone, Default)]
pub struct GetConversationOptions {
    /// Include messages in the response.
    pub include_messages: Option<bool>,
    /// Maximum number of messages to return.
    pub message_limit: Option<i32>,
    /// Number of messages to skip.
    pub message_offset: Option<i32>,
}

impl GetConversationOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn include_messages(mut self, include: bool) -> Self {
        self.include_messages = Some(include);
        self
    }

    pub fn message_limit(mut self, limit: i32) -> Self {
        self.message_limit = Some(limit);
        self
    }

    pub fn message_offset(mut self, offset: i32) -> Self {
        self.message_offset = Some(offset);
        self
    }

    pub(crate) fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(true) = self.include_messages {
            params.push(("include_messages".to_string(), "true".to_string()));
        }
        if let Some(limit) = self.message_limit {
            params.push(("message_limit".to_string(), limit.to_string()));
        }
        if let Some(offset) = self.message_offset {
            params.push(("message_offset".to_string(), offset.to_string()));
        }
        params
    }
}

/// Request to update a conversation.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateConversationRequest {
    /// New metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    /// New tags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Request to reply to a conversation.
#[derive(Debug, Clone, Serialize)]
pub struct ReplyToConversationRequest {
    /// Message content.
    pub text: String,
    /// Message type for compliance.
    #[serde(skip_serializing_if = "Option::is_none", rename = "messageType")]
    pub message_type: Option<String>,
    /// Custom JSON metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    /// Media URLs to attach.
    #[serde(skip_serializing_if = "Option::is_none", rename = "mediaUrls")]
    pub media_urls: Option<Vec<String>>,
}

/// Request to add labels to a conversation.
#[derive(Debug, Clone, Serialize)]
pub struct AddLabelsRequest {
    /// Label IDs to add.
    #[serde(rename = "labelIds")]
    pub label_ids: Vec<String>,
}

// ============================================================================
// Labels
// ============================================================================

/// A conversation label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// Unique label identifier.
    pub id: String,
    /// Label name.
    pub name: String,
    /// Label color.
    pub color: String,
    /// Label description.
    #[serde(default)]
    pub description: Option<String>,
    /// When the label was created.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
}

/// Response from listing labels.
#[derive(Debug, Clone, Deserialize)]
pub struct LabelListResponse {
    /// List of labels.
    #[serde(default)]
    pub data: Vec<Label>,
}

/// Request to create a label.
#[derive(Debug, Clone, Serialize)]
pub struct CreateLabelRequest {
    /// Label name.
    pub name: String,
    /// Label color.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Label description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CreateLabelRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            color: None,
            description: None,
        }
    }

    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

// ============================================================================
// Drafts
// ============================================================================

/// Draft status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DraftStatus {
    /// Draft is awaiting review.
    Pending,
    /// Draft has been approved.
    Approved,
    /// Draft has been rejected.
    Rejected,
    /// Draft has been sent.
    Sent,
    /// Draft failed to send.
    Failed,
}

impl std::fmt::Display for DraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DraftStatus::Pending => write!(f, "pending"),
            DraftStatus::Approved => write!(f, "approved"),
            DraftStatus::Rejected => write!(f, "rejected"),
            DraftStatus::Sent => write!(f, "sent"),
            DraftStatus::Failed => write!(f, "failed"),
        }
    }
}

/// A message draft.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDraft {
    /// Unique draft identifier.
    pub id: String,
    /// Associated conversation ID.
    #[serde(alias = "conversationId")]
    pub conversation_id: String,
    /// Draft message content.
    pub text: String,
    /// Media URLs.
    #[serde(default, alias = "mediaUrls")]
    pub media_urls: Option<Vec<String>>,
    /// Custom metadata.
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    /// Draft status.
    pub status: DraftStatus,
    /// Origin of the draft.
    #[serde(default)]
    pub source: Option<String>,
    /// User who created the draft.
    #[serde(default, alias = "createdBy")]
    pub created_by: Option<String>,
    /// User who reviewed the draft.
    #[serde(default, alias = "reviewedBy")]
    pub reviewed_by: Option<String>,
    /// When the draft was reviewed.
    #[serde(default, alias = "reviewedAt")]
    pub reviewed_at: Option<String>,
    /// Reason for rejection.
    #[serde(default, alias = "rejectionReason")]
    pub rejection_reason: Option<String>,
    /// ID of the sent message (if sent).
    #[serde(default, alias = "messageId")]
    pub message_id: Option<String>,
    /// When the draft was created.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// When the draft was last updated.
    #[serde(default, alias = "updatedAt")]
    pub updated_at: Option<String>,
}

/// Pagination info for draft lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPagination {
    /// Total number of drafts.
    #[serde(default)]
    pub total: i64,
}

/// Response from listing drafts.
#[derive(Debug, Clone, Deserialize)]
pub struct DraftListResponse {
    /// List of drafts.
    #[serde(default)]
    pub data: Vec<MessageDraft>,
    /// Pagination info.
    pub pagination: DraftPagination,
}

/// Request to create a draft.
#[derive(Debug, Clone, Serialize)]
pub struct CreateDraftRequest {
    /// Associated conversation ID.
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    /// Draft message content.
    pub text: String,
    /// Media URLs.
    #[serde(skip_serializing_if = "Option::is_none", rename = "mediaUrls")]
    pub media_urls: Option<Vec<String>>,
    /// Custom metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
    /// Origin of the draft.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Request to update a draft.
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateDraftRequest {
    /// Updated draft message content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Updated media URLs.
    #[serde(skip_serializing_if = "Option::is_none", rename = "mediaUrls")]
    pub media_urls: Option<Vec<String>>,
    /// Updated custom metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Request to reject a draft.
#[derive(Debug, Clone, Serialize)]
pub struct RejectDraftRequest {
    /// Rejection reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Options for listing drafts.
#[derive(Debug, Clone, Default)]
pub struct ListDraftsOptions {
    /// Filter by conversation ID.
    pub conversation_id: Option<String>,
    /// Filter by draft status.
    pub status: Option<DraftStatus>,
    /// Maximum number of drafts to return.
    pub limit: Option<i32>,
    /// Number of drafts to skip.
    pub offset: Option<i32>,
}

impl ListDraftsOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn conversation_id(mut self, id: impl Into<String>) -> Self {
        self.conversation_id = Some(id.into());
        self
    }

    pub fn status(mut self, status: DraftStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: i32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub(crate) fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(ref id) = self.conversation_id {
            params.push(("conversation_id".to_string(), id.clone()));
        }
        if let Some(ref status) = self.status {
            params.push(("status".to_string(), status.to_string()));
        }
        if let Some(limit) = self.limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(offset) = self.offset {
            params.push(("offset".to_string(), offset.to_string()));
        }
        params
    }
}
