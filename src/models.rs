use serde::{Deserialize, Serialize};

/// Message delivery status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    /// Message is queued for delivery.
    Queued,
    /// Message is being sent.
    Sending,
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
            MessageStatus::Sending => write!(f, "sending"),
            MessageStatus::Sent => write!(f, "sent"),
            MessageStatus::Delivered => write!(f, "delivered"),
            MessageStatus::Failed => write!(f, "failed"),
        }
    }
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
    /// Error message (if failed).
    #[serde(default)]
    pub error: Option<String>,
    /// Number of SMS segments.
    #[serde(default)]
    pub segments: i32,
    /// Credits consumed.
    #[serde(default, rename = "creditsUsed")]
    pub credits_used: i32,
    /// Whether sent in sandbox mode.
    #[serde(default, rename = "isSandbox")]
    pub is_sandbox: bool,
    /// Creation timestamp.
    #[serde(default, rename = "createdAt")]
    pub created_at: Option<String>,
    /// Delivery timestamp (if delivered).
    #[serde(default, rename = "deliveredAt")]
    pub delivered_at: Option<String>,
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
        matches!(
            self.status,
            MessageStatus::Queued | MessageStatus::Sending | MessageStatus::Sent
        )
    }
}

/// Request to send an SMS message.
#[derive(Debug, Clone, Serialize)]
pub struct SendMessageRequest {
    /// Recipient phone number in E.164 format.
    pub to: String,
    /// Message content (max 1600 characters).
    pub text: String,
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
    #[serde(rename = "scheduledAt")]
    pub scheduled_at: String,
    /// Scheduled message status.
    pub status: ScheduledMessageStatus,
    /// Credits reserved for this message.
    #[serde(default, rename = "creditsReserved")]
    pub credits_reserved: i32,
    /// Creation timestamp.
    #[serde(default, rename = "createdAt")]
    pub created_at: Option<String>,
    /// When the message was sent.
    #[serde(default, rename = "sentAt")]
    pub sent_at: Option<String>,
    /// When the message was cancelled.
    #[serde(default, rename = "cancelledAt")]
    pub cancelled_at: Option<String>,
    /// Message ID after sending.
    #[serde(default, rename = "messageId")]
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
    #[serde(default, rename = "creditsRefunded")]
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
    PartiallyCompleted,
    /// Batch failed.
    Failed,
}

impl std::fmt::Display for BatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BatchStatus::Processing => write!(f, "processing"),
            BatchStatus::Completed => write!(f, "completed"),
            BatchStatus::PartiallyCompleted => write!(f, "partially_completed"),
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
}

/// Result of a single message in a batch.
#[derive(Debug, Clone, Deserialize)]
pub struct BatchMessageResult {
    /// Recipient phone number.
    pub to: String,
    /// Message ID if successful.
    #[serde(default, rename = "messageId")]
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
    #[serde(rename = "batchId")]
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
    #[serde(default, rename = "creditsUsed")]
    pub credits_used: i32,
    /// Results for each message.
    #[serde(default)]
    pub messages: Vec<BatchMessageResult>,
    /// Creation timestamp.
    #[serde(default, rename = "createdAt")]
    pub created_at: Option<String>,
    /// Completion timestamp.
    #[serde(default, rename = "completedAt")]
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
