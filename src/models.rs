use serde::{Deserialize, Serialize};

/// Message delivery status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MessageStatus {
    /// Message is queued for delivery.
    Queued,
    /// Message was sent to carrier.
    Sent,
    /// Message was delivered.
    Delivered,
    /// Recipient read the message. Read receipts exist on RCS and WhatsApp
    /// only — SMS never reports one.
    Read,
    /// Message delivery failed.
    Failed,
    /// Message bounced (carrier rejected).
    Bounced,
    /// Message is being retried after a transient failure.
    Retrying,
    /// An inbound message received on one of your numbers.
    Received,
    /// A status this build does not know; the message still deserialises.
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for MessageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageStatus::Queued => write!(f, "queued"),
            MessageStatus::Sent => write!(f, "sent"),
            MessageStatus::Delivered => write!(f, "delivered"),
            MessageStatus::Read => write!(f, "read"),
            MessageStatus::Failed => write!(f, "failed"),
            MessageStatus::Bounced => write!(f, "bounced"),
            MessageStatus::Retrying => write!(f, "retrying"),
            MessageStatus::Received => write!(f, "received"),
            MessageStatus::Unknown => write!(f, "unknown"),
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

/// The kind of sender a live send went out from. A simulated send reports
/// none, so its [`Message::sender_type`] is `None`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SenderType {
    /// A toll-free number from Sendly's number pool.
    NumberPool,
    /// Your alphanumeric sender ID.
    Alphanumeric,
    /// The sandbox sender.
    Sandbox,
    /// A number of yours: the `from` you passed, or the number your workspace
    /// is authorized to send from.
    Explicit,
    /// Never sent by the API.
    #[deprecated(note = "The API never reports this sender type.")]
    User,
    /// Never sent by the API.
    #[deprecated(note = "The API never reports this sender type.")]
    Api,
    /// Never sent by the API.
    #[deprecated(note = "The API never reports this sender type.")]
    System,
    /// Never sent by the API.
    #[deprecated(note = "The API never reports this sender type.")]
    Campaign,
    /// A sender type this build does not know; the message still decodes.
    #[serde(other)]
    Unknown,
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
    /// Whether sent in sandbox mode. Read from `get` and `list`; the send
    /// response does not report it, so a send leaves it `false` (see
    /// [`simulated`](Self::simulated)).
    #[serde(default, alias = "isSandbox")]
    pub is_sandbox: bool,
    /// True when a send was simulated and nothing reached a handset: with a
    /// test key, to a sandbox number, or from a live key whose account is not
    /// yet set up to send to the destination. Set on send responses only.
    #[serde(default)]
    pub simulated: bool,
    /// Why a live key's send was simulated instead of sent.
    #[serde(default, alias = "simulatedReason")]
    pub simulated_reason: Option<String>,
    /// Where to finish setting up the account so the next send is real, on a
    /// simulated live-key send.
    #[serde(default, alias = "actionUrl")]
    pub action_url: Option<String>,
    /// Type of sender, on a live send.
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

/// Per-call options accepted by the `*_with_options` send methods.
///
/// Construct with [`IdempotentRequestOptions::new`] and the builder methods.
#[derive(Debug, Clone, Default)]
pub struct IdempotentRequestOptions {
    /// Idempotency key for this operation (1-255 printable ASCII characters).
    ///
    /// The SDK already generates a key per logical request automatically, so
    /// the server can dedupe the SDK's own timeout retries. Supply your own
    /// key when you need idempotency across process restarts or your own
    /// retry loops — repeating a request with the same key within 24 hours
    /// returns the original response instead of executing again.
    ///
    /// Note: a response is cached under the key once the original attempt
    /// completes, including error responses — retrying a failed request with
    /// the same key returns the recorded failure; use a fresh key to
    /// re-execute.
    pub idempotency_key: Option<String>,
}

impl IdempotentRequestOptions {
    /// Creates new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the idempotency key.
    pub fn idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }
}

/// Request to send an SMS message.
///
/// Construct with [`SendMessageRequest::new`] and the `with_*` builder methods.
/// This type is `#[non_exhaustive]`, so external crates must use the constructor
/// rather than a struct literal.
#[derive(Debug, Clone, Default, Serialize)]
#[non_exhaustive]
pub struct SendMessageRequest {
    /// Recipient phone number in E.164 format.
    pub to: String,
    /// Message content (max 1600 characters).
    pub text: String,
    /// Sender ID or phone number (optional). Only sent to the API when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
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

impl SendMessageRequest {
    /// Creates a new send request for the given recipient and message text.
    pub fn new(to: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            to: to.into(),
            text: text.into(),
            ..Default::default()
        }
    }

    /// Sets the sender ID or phone number.
    pub fn with_from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Sets the message type (marketing or transactional).
    pub fn with_message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = Some(message_type);
        self
    }

    /// Sets the media URLs for an MMS message.
    pub fn with_media_urls(mut self, media_urls: Vec<String>) -> Self {
        self.media_urls = Some(media_urls);
        self
    }

    /// Sets custom metadata to attach to the message.
    pub fn with_metadata(
        mut self,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Request to send a group MMS to 2-8 recipients (US/Canada only).
///
/// Group messaging is an A2P 10DLC capability: the sending number must be an
/// MMS-enabled, 10DLC-registered number you own. Construct with
/// [`SendGroupMessageRequest::new`] and the `with_*` builder methods. This type
/// is `#[non_exhaustive]`, so external crates must use the constructor rather
/// than a struct literal.
#[derive(Debug, Clone, Default, Serialize)]
#[non_exhaustive]
pub struct SendGroupMessageRequest {
    /// 2-8 recipient phone numbers in E.164 format (US/CA MMS-capable mobiles).
    pub to: Vec<String>,
    /// Message body. Required unless `media_urls` is provided.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Sending number (E.164). Omit to use your workspace's default sender.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// HTTPS media URLs to attach. Required unless `text` is provided.
    #[serde(skip_serializing_if = "Option::is_none", rename = "mediaUrls")]
    pub media_urls: Option<Vec<String>>,
    /// Message type: defaults to "transactional" for group MMS; pass
    /// "marketing" to apply quiet-hours rules.
    #[serde(skip_serializing_if = "Option::is_none", rename = "messageType")]
    pub message_type: Option<MessageType>,
}

impl SendGroupMessageRequest {
    /// Creates a new group MMS request for the given recipients.
    pub fn new(to: Vec<String>) -> Self {
        Self {
            to,
            ..Default::default()
        }
    }

    /// Sets the message body.
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Sets the sending number.
    pub fn with_from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Sets the media URLs to attach.
    pub fn with_media_urls(mut self, media_urls: Vec<String>) -> Self {
        self.media_urls = Some(media_urls);
        self
    }

    /// Sets the message type (marketing or transactional).
    pub fn with_message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = Some(message_type);
        self
    }
}

/// Response from sending a group MMS.
#[derive(Debug, Clone, Deserialize)]
#[serde(from = "RawGroupMessageResponse")]
pub struct GroupMessageResponse {
    /// Message identifier (matches the `id` in delivery webhooks).
    pub id: String,
    /// Delivery status ("sent" on a live send, "delivered" when simulated).
    pub status: MessageStatus,
    /// Phone numbers the group message was sent to.
    pub to: Vec<String>,
    /// Each recipient with its status. Live sends carry it; simulated sends
    /// leave it empty.
    pub recipients: Vec<GroupRecipient>,
    /// Identifier for the group conversation (present on live sends).
    pub group_message_id: Option<String>,
    /// True when the send was simulated and nothing was sent to the carrier.
    pub simulated: Option<bool>,
    /// Human-readable note, present on simulated sends.
    pub message: Option<String>,
}

/// One recipient of a live group send.
#[derive(Debug, Clone, Deserialize)]
pub struct GroupRecipient {
    /// The recipient's phone number in E.164 format.
    #[serde(default, rename = "phoneNumber")]
    pub phone_number: String,
    /// The recipient's status when the send was accepted, for example
    /// `"queued"`.
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawGroupTo {
    Number(String),
    Recipient(GroupRecipient),
}

#[derive(Deserialize)]
struct RawGroupMessageResponse {
    id: String,
    status: MessageStatus,
    #[serde(default)]
    to: Vec<RawGroupTo>,
    #[serde(default, alias = "groupMessageId")]
    group_message_id: Option<String>,
    #[serde(default)]
    simulated: Option<bool>,
    #[serde(default)]
    message: Option<String>,
}

impl From<RawGroupMessageResponse> for GroupMessageResponse {
    fn from(raw: RawGroupMessageResponse) -> Self {
        let mut to = Vec::with_capacity(raw.to.len());
        let mut recipients = Vec::new();
        for entry in raw.to {
            match entry {
                RawGroupTo::Number(number) => to.push(number),
                RawGroupTo::Recipient(recipient) => {
                    to.push(recipient.phone_number.clone());
                    recipients.push(recipient);
                }
            }
        }
        Self {
            id: raw.id,
            status: raw.status,
            to,
            recipients,
            group_message_id: raw.group_message_id,
            simulated: raw.simulated,
            message: raw.message,
        }
    }
}

/// Request to AI-enhance a draft message. Provide `text`, `message_type`, or
/// both — at least one is required.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EnhanceMessageRequest {
    /// Draft message text to rewrite. Optional if `message_type` is provided
    /// (the model then generates a suitable message for that type). Only the
    /// first 500 characters are considered; the result is trimmed to one SMS
    /// segment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Hint about the kind of message so the rewrite is targeted (e.g.
    /// "marketing", "transactional"). Optional if `text` is provided.
    #[serde(skip_serializing_if = "Option::is_none", rename = "messageType")]
    pub message_type: Option<String>,
}

impl EnhanceMessageRequest {
    /// Creates a new, empty enhancement request.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the draft text to rewrite.
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Sets the message-type hint.
    pub fn with_message_type(mut self, message_type: impl Into<String>) -> Self {
        self.message_type = Some(message_type.into());
        self
    }
}

/// Result of an AI message enhancement.
#[derive(Debug, Clone, Deserialize)]
pub struct EnhanceMessageResponse {
    /// The rewritten message, capped at 160 characters (one SMS segment). When
    /// AI enhancement is unavailable, this falls back to the original text.
    pub enhanced: String,
    /// Short explanation of what changed. An empty string on the fallback path.
    #[serde(default)]
    pub explanation: String,
    /// The model that produced the enhancement, when available.
    #[serde(default)]
    pub model: Option<String>,
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

/// Page metadata returned by [`Messages::list`](crate::Messages::list).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MessagePagination {
    /// Messages matching the filters, across all pages.
    #[serde(default)]
    pub total: i64,
    /// Page size that was applied.
    #[serde(default)]
    pub limit: i64,
    /// Offset that was applied.
    #[serde(default)]
    pub offset: i64,
    /// Page number, starting at 1.
    #[serde(default)]
    pub page: i64,
    /// Number of pages at this page size.
    #[serde(default)]
    pub total_pages: i64,
    /// Whether another page follows.
    #[serde(default)]
    pub has_more: bool,
}

/// Paginated list of messages.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageList {
    /// Messages in this page.
    pub data: Vec<Message>,
    /// Number of messages in this page. For the number matching the query,
    /// use [`total`](Self::total).
    #[serde(default)]
    pub count: i32,
    /// Paging details, including the total across all pages.
    #[serde(default)]
    pub pagination: Option<MessagePagination>,
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

    /// Returns the number of messages matching the query across all pages,
    /// or the page size when the response carries no pagination.
    pub fn total(&self) -> i32 {
        self.pagination
            .as_ref()
            .map_or(self.count, |p| i32::try_from(p.total).unwrap_or(i32::MAX))
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
#[non_exhaustive]
pub enum ScheduledMessageStatus {
    /// Message is scheduled for future delivery.
    Scheduled,
    /// Message was sent.
    Sent,
    /// Message was delivered.
    Delivered,
    /// Message was cancelled.
    Cancelled,
    /// Message failed to send.
    Failed,
    /// Message bounced (carrier rejected).
    Bounced,
    /// A status this build does not know; the message still deserialises.
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for ScheduledMessageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScheduledMessageStatus::Scheduled => write!(f, "scheduled"),
            ScheduledMessageStatus::Sent => write!(f, "sent"),
            ScheduledMessageStatus::Delivered => write!(f, "delivered"),
            ScheduledMessageStatus::Cancelled => write!(f, "cancelled"),
            ScheduledMessageStatus::Failed => write!(f, "failed"),
            ScheduledMessageStatus::Bounced => write!(f, "bounced"),
            ScheduledMessageStatus::Unknown => write!(f, "unknown"),
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

    /// Returns true if the message was sent, including once it is delivered.
    pub fn is_sent(&self) -> bool {
        matches!(
            self.status,
            ScheduledMessageStatus::Sent | ScheduledMessageStatus::Delivered
        )
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
            if *status != ScheduledMessageStatus::Unknown {
                params.push(("status".to_string(), status.to_string()));
            }
        }

        params
    }
}

/// Paginated list of scheduled messages.
#[derive(Debug, Clone, Deserialize)]
pub struct ScheduledMessageList {
    /// Scheduled messages in this page.
    pub data: Vec<ScheduledMessage>,
    /// Number of scheduled messages in this page. The API reports no total
    /// across pages.
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

    /// Returns the number of scheduled messages in this page.
    #[deprecated(
        note = "The API reports no total across pages; this is the page size, the same as `len()`."
    )]
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
    /// The message ID (absent for failed messages).
    #[serde(default)]
    pub id: Option<String>,
    /// Recipient phone number.
    pub to: String,
    /// Message status.
    pub status: String,
    /// Error message if failed.
    #[serde(default)]
    pub error: Option<String>,
    /// When the message was created.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// When the message was delivered.
    #[serde(default, alias = "deliveredAt")]
    pub delivered_at: Option<String>,
}

/// A message batch: the result of sending one, or its status from
/// [`get_batch`](crate::Messages::get_batch) and
/// [`list_batches`](crate::Messages::list_batches).
#[derive(Debug, Clone, Deserialize)]
pub struct BatchMessageResponse {
    /// Unique batch identifier. The send response calls it `batchId` and the
    /// status responses `id`.
    #[serde(alias = "batchId", alias = "id")]
    pub batch_id: String,
    /// Batch status.
    pub status: BatchStatus,
    /// Total messages in batch.
    pub total: i32,
    /// Messages queued (absent on the batch-create response).
    #[serde(default)]
    pub queued: i32,
    /// Messages sent.
    pub sent: i32,
    /// Messages delivered (absent on the batch-create response).
    #[serde(default)]
    pub delivered: i32,
    /// Messages failed.
    pub failed: i32,
    /// Credits reserved for the batch (absent on the batch-create response).
    #[serde(default, alias = "creditsReserved")]
    pub credits_reserved: i32,
    /// Total credits used.
    #[serde(default, alias = "creditsUsed")]
    pub credits_used: i32,
    /// Credits refunded for messages that failed.
    #[serde(default, alias = "creditsRefunded")]
    pub credits_refunded: i32,
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

/// A message the batch preview found it cannot send.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BatchPreviewBlockedMessage {
    /// Position of the message in the request.
    #[serde(default)]
    pub index: i32,
    /// Recipient phone number.
    #[serde(default)]
    pub to: String,
    /// Why it cannot be sent.
    #[serde(default)]
    pub reason: String,
}

#[derive(Deserialize)]
struct BatchPreviewWire {
    #[serde(default, alias = "canSend")]
    can_send: Option<bool>,
    #[serde(default)]
    total: Option<i32>,
    #[serde(default, alias = "totalMessages")]
    total_messages: Option<i32>,
    #[serde(default)]
    sendable: Option<i32>,
    #[serde(default, alias = "willSend")]
    will_send: Option<i32>,
    #[serde(default)]
    blocked: i32,
    #[serde(default)]
    duplicates: i32,
    #[serde(default, alias = "creditsNeeded")]
    credits_needed: i32,
    #[serde(default, rename = "creditBalance")]
    credit_balance: Option<i32>,
    #[serde(default, alias = "currentBalance")]
    current_balance: Option<i32>,
    #[serde(default, rename = "hasSufficientCredits")]
    has_sufficient_credits: Option<bool>,
    #[serde(default, alias = "hasEnoughCredits")]
    has_enough_credits: Option<bool>,
    #[serde(default, alias = "keyType")]
    key_type: Option<String>,
    #[serde(default, alias = "hasWriteScope")]
    has_write_scope: bool,
    #[serde(default)]
    pooled: bool,
    #[serde(default, alias = "blockedMessages")]
    blocked_messages: Vec<BatchPreviewBlockedMessage>,
    #[serde(default)]
    compliance: Option<serde_json::Value>,
    #[serde(default)]
    warnings: Vec<String>,
    #[serde(default)]
    messages: Vec<BatchPreviewItem>,
    #[serde(default, alias = "blockReasons")]
    block_reasons: Option<std::collections::HashMap<String, i32>>,
}

/// Response from previewing a batch (dry run).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "BatchPreviewWire")]
pub struct BatchPreviewResponse {
    /// Whether nothing the preview found stops a send: the key can send, the
    /// batch has at most 10,000 messages, at least one message is sendable,
    /// nothing is blocked for a reason other than an opt-out (a live send
    /// rejects the whole batch then), and, for a live key, the balance covers
    /// it. A test key's send skips the destination and verification checks,
    /// so it can go through while `can_send` is false.
    pub can_send: bool,
    /// Total number of messages.
    pub total_messages: i32,
    /// Number of messages that will be sent.
    pub will_send: i32,
    /// Number of messages that are blocked.
    pub blocked: i32,
    /// Number of duplicate recipients, which the send removes.
    pub duplicates: i32,
    /// Total credits needed.
    pub credits_needed: i32,
    /// Current credit balance.
    pub current_balance: i32,
    /// Whether there are enough credits.
    pub has_enough_credits: bool,
    /// `"test"` or `"live"`: the kind of key the preview ran with.
    pub key_type: Option<String>,
    /// Whether the key has the `sms:send` scope a send needs.
    pub has_write_scope: bool,
    /// Whether the balance is an enterprise credit pool.
    pub pooled: bool,
    /// The messages that cannot be sent, and why.
    pub blocked_messages: Vec<BatchPreviewBlockedMessage>,
    /// Warnings about the batch that do not block it.
    pub warnings: Vec<String>,
    /// Preview for each message.
    #[deprecated(
        note = "The API does not preview each message; this is always empty. Read `blocked_messages` for the ones it cannot send."
    )]
    pub messages: Vec<BatchPreviewItem>,
    /// Count of block reasons.
    #[deprecated(
        note = "The API does not send this; it is always `None`. Read `blocked_messages`."
    )]
    pub block_reasons: Option<std::collections::HashMap<String, i32>>,
}

impl From<BatchPreviewWire> for BatchPreviewResponse {
    #[allow(deprecated)]
    fn from(wire: BatchPreviewWire) -> Self {
        let current_balance = wire.credit_balance.or(wire.current_balance).unwrap_or(0);
        let has_enough_credits = wire
            .has_sufficient_credits
            .or(wire.has_enough_credits)
            .unwrap_or(false);
        let will_send = wire.sendable.or(wire.will_send).unwrap_or(0);
        let opted_out = wire
            .compliance
            .as_ref()
            .and_then(|c| c.get("optedOutBlocked"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let total_messages = wire.total.or(wire.total_messages).unwrap_or(0);
        let can_send = wire.can_send.unwrap_or_else(|| {
            wire.has_write_scope
                && total_messages <= 10_000
                && will_send > 0
                && i64::from(wire.blocked) <= opted_out
                && (wire.key_type.as_deref() == Some("test") || has_enough_credits)
        });
        Self {
            can_send,
            total_messages,
            will_send,
            blocked: wire.blocked,
            duplicates: wire.duplicates,
            credits_needed: wire.credits_needed,
            current_balance,
            has_enough_credits,
            key_type: wire.key_type,
            has_write_scope: wire.has_write_scope,
            pooled: wire.pooled,
            blocked_messages: wire.blocked_messages,
            warnings: wire.warnings,
            messages: wire.messages,
            block_reasons: wire.block_reasons,
        }
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
    /// Number of batches in this page. The API reports no total across
    /// pages.
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

    /// Returns the number of batches in this page.
    #[deprecated(
        note = "The API reports no total across pages; this is the page size, the same as `len()`."
    )]
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

// ==================== URL Shortener (branded links) ====================

/// Request to mint a branded short link.
#[derive(Debug, Clone, Serialize)]
pub struct CreateShortLinkRequest {
    /// Destination URL to shorten (http/https only).
    pub url: String,
}

/// A newly minted branded short link.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateShortLinkResponse {
    /// Short code (the segment after the domain, e.g. "Ab3xY7").
    pub code: String,
    /// Full branded short URL to share (e.g. "https://sendly.live/l/Ab3xY7").
    #[serde(alias = "shortUrl")]
    pub short_url: String,
    /// The destination the short link redirects to.
    #[serde(alias = "destinationUrl")]
    pub destination_url: String,
}

/// A short link with click analytics, as returned by [`ShortLinkListResponse`].
#[derive(Debug, Clone, Deserialize)]
pub struct ShortLink {
    /// Short code (the segment after the domain).
    pub code: String,
    /// Full branded short URL.
    #[serde(alias = "shortUrl")]
    pub short_url: String,
    /// The destination the short link redirects to.
    #[serde(alias = "destinationUrl")]
    pub destination_url: String,
    /// Workspace brand slug segment, or `None` when unbranded.
    #[serde(default, alias = "brandSlug")]
    pub brand_slug: Option<String>,
    /// Total human clicks recorded (link-preview bots are excluded).
    #[serde(default, alias = "clickCount")]
    pub click_count: i64,
    /// Whether the link is disabled (the redirect then returns 404).
    #[serde(default)]
    pub disabled: bool,
    /// ISO 3166-1 alpha-2 country of the most recent click, or `None`.
    #[serde(default, alias = "lastCountry")]
    pub last_country: Option<String>,
    /// When the link was last clicked (ISO 8601), or `None`.
    #[serde(default, alias = "lastClickedAt")]
    pub last_clicked_at: Option<String>,
    /// When the link was created (ISO 8601).
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// 14-day daily click histogram, oldest first (today last).
    #[serde(default)]
    pub spark: Vec<i64>,
}

/// Response from listing short links.
#[derive(Debug, Clone, Deserialize)]
pub struct ShortLinkListResponse {
    /// The workspace's short links, newest first.
    #[serde(default)]
    pub links: Vec<ShortLink>,
    /// Total number of short links in the workspace.
    #[serde(default)]
    pub total: i32,
}

/// Request to enable or disable a short link.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateShortLinkRequest {
    /// New disabled state.
    pub disabled: bool,
}

/// Response from enabling or disabling a short link.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateShortLinkResponse {
    /// Short code that was updated.
    pub code: String,
    /// New disabled state.
    pub disabled: bool,
}

/// Options for listing short links.
#[derive(Debug, Clone, Default)]
pub struct ListShortLinksOptions {
    /// Maximum links to return (default: 50, max: 200).
    pub limit: Option<u32>,
    /// Number of links to skip.
    pub offset: Option<u32>,
}

impl ListShortLinksOptions {
    /// Creates new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the limit.
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.min(200));
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
    #[deprecated(
        note = "The API reports no total; this is always 0. A page shorter than the limit you asked for is the last one."
    )]
    #[serde(default)]
    pub total: i32,
    /// Whether there are more deliveries.
    #[deprecated(
        note = "The API does not report this; it is always false. A page shorter than the limit you asked for is the last one."
    )]
    #[serde(default, alias = "hasMore")]
    pub has_more: bool,
}

#[derive(Deserialize)]
struct WebhookTestResultWire {
    #[serde(default)]
    success: bool,
    #[serde(default, alias = "statusCode")]
    status_code: Option<i32>,
    #[serde(default, alias = "responseTimeMs")]
    response_time_ms: Option<i32>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    delivery: Option<serde_json::Value>,
}

/// Result from testing a webhook.
#[derive(Debug, Clone, Deserialize)]
#[serde(from = "WebhookTestResultWire")]
pub struct WebhookTestResult {
    /// Whether the test was successful.
    pub success: bool,
    /// HTTP status code from the endpoint.
    pub status_code: i32,
    /// Response time in milliseconds.
    pub response_time_ms: i32,
    /// Error message if the test failed.
    pub error: Option<String>,
    /// What happened, in words (for example `Test webhook delivered
    /// successfully in 87ms`).
    pub message: Option<String>,
    /// The test delivery the API recorded (`id`, `status`, `status_code`,
    /// `response_time`, `response_body`, ...).
    pub delivery: Option<serde_json::Value>,
}

impl From<WebhookTestResultWire> for WebhookTestResult {
    fn from(wire: WebhookTestResultWire) -> Self {
        let delivery_field = |key: &str| {
            wire.delivery
                .as_ref()
                .and_then(|d| d.get(key))
                .and_then(|v| v.as_i64())
                .and_then(|n| i32::try_from(n).ok())
        };
        let status_code = wire
            .status_code
            .or_else(|| delivery_field("status_code"))
            .unwrap_or(0);
        let response_time_ms = wire
            .response_time_ms
            .or_else(|| delivery_field("response_time"))
            .unwrap_or(0);
        let error = wire.error.or_else(|| {
            wire.delivery
                .as_ref()
                .and_then(|d| d.get("error"))
                .and_then(|v| v.as_str())
                .map(str::to_string)
        });
        Self {
            success: wire.success,
            status_code,
            response_time_ms,
            error,
            message: wire.message,
            delivery: wire.delivery,
        }
    }
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
    #[deprecated(note = "The API has no pending balance; this is always 0.")]
    #[serde(default, alias = "pendingCredits")]
    pub pending_credits: i32,
    /// Credits reserved for messages that are scheduled or still sending.
    #[serde(default, alias = "reservedBalance", alias = "reservedCredits")]
    pub reserved_credits: i32,
    /// `"prepaid"` for a workspace's own balance, or `"pooled"` when it draws
    /// on an enterprise credit pool.
    #[serde(default, alias = "billingMode")]
    pub billing_mode: Option<String>,
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
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum TransactionType {
    /// Credit purchase, including an auto-recharge.
    Purchase,
    /// Credit usage (sending messages).
    Usage,
    /// Credit refund.
    Refund,
    /// Bonus credits.
    Bonus,
    /// Credits moved between workspaces.
    Transfer,
    /// Credits Sendly added to the account by hand.
    AdminGrant,
    /// Test credits Sendly seeded into the account.
    AdminSeed,
    /// Manual adjustment.
    #[deprecated(note = "The API never records an adjustment; Sendly's grants are `AdminGrant`.")]
    Adjustment,
    /// A type this build does not know; the transaction still deserialises.
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for TransactionType {
    #[allow(deprecated)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Purchase => write!(f, "purchase"),
            TransactionType::Usage => write!(f, "usage"),
            TransactionType::Refund => write!(f, "refund"),
            TransactionType::Bonus => write!(f, "bonus"),
            TransactionType::Transfer => write!(f, "transfer"),
            TransactionType::AdminGrant => write!(f, "admin_grant"),
            TransactionType::AdminSeed => write!(f, "admin_seed"),
            TransactionType::Adjustment => write!(f, "adjustment"),
            TransactionType::Unknown => write!(f, "unknown"),
        }
    }
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
    #[deprecated(
        note = "The API reports no total; this is always 0. A page shorter than the limit you asked for is the last one."
    )]
    #[serde(default)]
    pub total: i32,
    /// Whether there are more transactions.
    #[deprecated(
        note = "The API does not report this; it is always false. A page shorter than the limit you asked for is the last one."
    )]
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
            if *t != TransactionType::Unknown {
                params.push(("type".to_string(), t.to_string()));
            }
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
    #[serde(default, alias = "keyPrefix")]
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
    /// `"test"` or `"live"`.
    #[serde(default, rename = "type")]
    pub key_type: Option<String>,
    /// The scopes the key grants, such as `sms:send`.
    #[serde(default)]
    pub scopes: Option<Vec<String>>,
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
    /// The created key's ID.
    #[serde(default)]
    pub id: Option<String>,
    /// The created key's display name.
    #[serde(default)]
    pub name: Option<String>,
    /// The created key's prefix.
    #[serde(default, alias = "keyPrefix")]
    pub key_prefix: Option<String>,
    /// `"test"` or `"live"`.
    #[serde(default, rename = "type")]
    pub key_type: Option<String>,
}

/// Request to create an API key.
///
/// Build with [`CreateApiKeyRequest::new`] and the builder methods.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateApiKeyRequest {
    /// Display name for the API key.
    pub name: String,
    /// Optional expiration date (ISO 8601, in the future).
    #[serde(skip_serializing_if = "Option::is_none", rename = "expiresAt")]
    pub expires_at: Option<String>,
    /// `"test"` or `"live"`. Omitted, the API creates a test key.
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub key_type: Option<String>,
    /// Scopes to grant, such as `sms:send`; each must be one the calling key
    /// has. Omitted, the new key gets the calling key's scopes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

impl CreateApiKeyRequest {
    /// Creates a request for a key with this display name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Sets the key type: `"test"` or `"live"`.
    pub fn key_type(mut self, key_type: impl Into<String>) -> Self {
        self.key_type = Some(key_type.into());
        self
    }

    /// Sets the scopes to grant.
    pub fn scopes(mut self, scopes: Vec<impl Into<String>>) -> Self {
        self.scopes = Some(scopes.into_iter().map(|s| s.into()).collect());
        self
    }

    /// Sets when the key expires (ISO 8601, in the future).
    pub fn expires_at(mut self, expires_at: impl Into<String>) -> Self {
        self.expires_at = Some(expires_at.into());
        self
    }
}

/// Request to rotate an API key.
///
/// Build with [`RotateApiKeyRequest::new`] and optionally
/// [`RotateApiKeyRequest::grace_period_hours`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct RotateApiKeyRequest {
    /// Hours the old key keeps working after rotation, 24-168 inclusive.
    /// Omit to use the API default of 24.
    #[serde(skip_serializing_if = "Option::is_none", rename = "gracePeriodHours")]
    pub grace_period_hours: Option<u32>,
}

impl RotateApiKeyRequest {
    /// Creates a new rotation request with the default grace period.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the grace period (24-168 hours) the old key keeps working.
    pub fn grace_period_hours(mut self, hours: u32) -> Self {
        self.grace_period_hours = Some(hours);
        self
    }
}

/// A rotated (newly issued) API key: every [`ApiKey`] field plus the one-time
/// raw secret and a caution to store it.
#[derive(Debug, Clone, Deserialize)]
pub struct RotatedApiKey {
    /// All standard API key fields (id, name, prefix, timestamps, …).
    #[serde(flatten)]
    pub key_info: ApiKey,
    /// The raw new secret (`sk_…`). Shown only once — store it now.
    #[serde(rename = "key")]
    pub secret: String,
    /// Human-readable caution about the one-time secret.
    #[serde(default)]
    pub warning: String,
}

/// Response from rotating an API key (see [`AccountResource::rotate_api_key`]).
#[derive(Debug, Clone, Deserialize)]
pub struct RotateApiKeyResponse {
    /// The newly issued key, including its one-time raw secret and a warning.
    #[serde(alias = "newKey")]
    pub new_key: RotatedApiKey,
    /// The predecessor key, now counting down its grace period.
    #[serde(alias = "oldKey")]
    pub old_key: ApiKey,
    /// Human-readable summary (e.g. when the old key expires).
    #[serde(default)]
    pub message: String,
}

/// Account verification status.
#[deprecated(
    note = "The API reports no email, phone or identity verification; read `Account::business_verification`."
)]
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

#[allow(deprecated)]
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
    #[deprecated(
        note = "The API does not report a per-second limit; this is always 10. Read `messages_per_minute`."
    )]
    #[serde(default = "default_mps", alias = "messagesPerSecond")]
    pub messages_per_second: i32,
    /// Maximum messages per day: 100 with a test key, 10,000 with a live
    /// one.
    #[serde(default = "default_mpd", alias = "messagesPerDay")]
    pub messages_per_day: i32,
    /// Maximum batch size.
    #[deprecated(
        note = "The API does not report this; it is always 1000. A batch takes up to 10,000 messages."
    )]
    #[serde(default = "default_batch", alias = "maxBatchSize")]
    pub max_batch_size: i32,
    /// Maximum messages per minute.
    #[serde(default, alias = "messagesPerMinute")]
    pub messages_per_minute: Option<i32>,
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
    #[allow(deprecated)]
    fn default() -> Self {
        Self {
            messages_per_second: 10,
            messages_per_day: 10000,
            max_batch_size: 1000,
            messages_per_minute: None,
        }
    }
}

/// The workspace an API key belongs to.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AccountOrganization {
    /// Workspace ID, the `organization_id` webhook payloads carry.
    pub id: String,
    /// Workspace name.
    #[serde(default)]
    pub name: Option<String>,
    /// Whether this is the user's personal workspace.
    #[serde(default)]
    pub is_personal: bool,
}

/// The business verification that decides where an account can send.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AccountBusinessVerification {
    /// Status, such as `verified`, `processing`, `action_required` or
    /// `rejected`.
    #[serde(default)]
    pub status: Option<String>,
    /// Verification type, such as `toll_free`.
    #[serde(default, rename = "type")]
    pub verification_type: Option<String>,
    /// Region the verification covers.
    #[serde(default)]
    pub region: Option<String>,
    /// When it was submitted.
    #[serde(default)]
    pub submitted_at: Option<String>,
    /// When it last changed.
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Account information.
#[allow(deprecated)]
#[derive(Debug, Clone, Deserialize)]
pub struct Account {
    /// Unique account identifier (the user ID).
    pub id: String,
    /// Account email address.
    #[serde(default)]
    pub email: String,
    /// Account holder name.
    #[deprecated(note = "The API does not send this; it is always `None`.")]
    #[serde(default)]
    pub name: Option<String>,
    /// Company name.
    #[deprecated(note = "The API does not send this; it is always `None`.")]
    #[serde(default, alias = "companyName")]
    pub company_name: Option<String>,
    /// Verification status.
    #[deprecated(
        note = "The API reports no email, phone or identity verification; read `business_verification`."
    )]
    #[serde(default)]
    pub verification: AccountVerification,
    /// Rate limits.
    #[serde(default)]
    pub limits: AccountLimits,
    /// Account creation timestamp.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// The workspace the API key belongs to; `None` for a key that belongs
    /// to no workspace.
    #[serde(default)]
    pub organization: Option<AccountOrganization>,
    /// The business verification, or `None` when the account has not
    /// submitted one.
    #[serde(default)]
    pub business_verification: Option<AccountBusinessVerification>,
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

/// One workspace in [`EnterpriseWorkspaceList`]. A compact list carries
/// only `id`, `name` and `credit_balance`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EnterpriseWorkspaceListItem {
    /// Workspace ID.
    pub id: String,
    /// Workspace name.
    #[serde(default)]
    pub name: String,
    /// Workspace slug.
    #[serde(default)]
    pub slug: Option<String>,
    /// `active` or `suspended`.
    #[serde(default)]
    pub status: Option<String>,
    /// When the workspace was suspended.
    #[serde(default)]
    pub suspended_at: Option<String>,
    /// Why the workspace was suspended.
    #[serde(default)]
    pub suspend_reason: Option<String>,
    /// Verification status, or `None` before one is submitted.
    #[serde(default)]
    pub verification_status: Option<String>,
    /// Why the verification was rejected.
    #[serde(default)]
    pub rejection_reason: Option<String>,
    /// Verification type, such as `toll_free`.
    #[serde(default)]
    pub verification_type: Option<String>,
    /// The workspace's verified toll-free number.
    #[serde(default)]
    pub toll_free_number: Option<String>,
    /// The workspace's credit balance.
    #[serde(default)]
    pub credit_balance: i64,
    /// Number of active API keys.
    #[serde(default)]
    pub key_count: i64,
    /// Messages sent in the last 30 days.
    #[serde(default, rename = "messages30d")]
    pub messages_30d: i64,
    /// Messages delivered in the last 30 days.
    #[serde(default, rename = "delivered30d")]
    pub delivered_30d: i64,
    /// Messages that failed in the last 30 days.
    #[serde(default, rename = "failed30d")]
    pub failed_30d: i64,
    /// Monthly message quota, or `None` for no quota.
    #[serde(default)]
    pub monthly_message_quota: Option<i64>,
    /// Messages sent this month.
    #[serde(default)]
    pub messages_this_month: i64,
    /// When the monthly count resets.
    #[serde(default)]
    pub quota_reset_at: Option<String>,
    /// When the workspace was created.
    #[serde(default)]
    pub created_at: Option<String>,
    /// The workspace's tags.
    #[serde(default)]
    pub tags: Vec<serde_json::Value>,
}

/// Page metadata for [`EnterpriseWorkspaceList`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EnterpriseWorkspacePagination {
    /// Workspaces matching the filters, across all pages.
    #[serde(default)]
    pub total: i64,
    /// Page size that was applied.
    #[serde(default)]
    pub limit: i64,
    /// Page number, starting at 1.
    #[serde(default)]
    pub page: i64,
    /// Number of pages at this page size.
    #[serde(default)]
    pub total_pages: i64,
    /// Whether another page follows.
    #[serde(default)]
    pub has_more: bool,
}

/// A page of the enterprise's workspaces.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EnterpriseWorkspaceList {
    /// The workspaces on this page.
    #[serde(default)]
    pub workspaces: Vec<EnterpriseWorkspaceListItem>,
    /// Page metadata; `None` for a compact list, which is not paginated.
    #[serde(default)]
    pub pagination: Option<EnterpriseWorkspacePagination>,
    /// Totals across all workspaces: credits, verifications pending and a
    /// breakdown by verification status.
    #[serde(default)]
    pub summary: Option<serde_json::Value>,
    /// How many workspaces the enterprise plan allows.
    #[serde(default)]
    pub max_workspaces: i64,
    /// How many workspaces exist.
    #[serde(default)]
    pub workspaces_used: i64,
}

/// Options for listing enterprise workspaces.
#[derive(Debug, Clone, Default)]
pub struct ListWorkspacesOptions {
    /// Page number, starting at 1.
    pub page: Option<u32>,
    /// Workspaces per page (default 50, max 100). A limit of 0 is not sent.
    pub limit: Option<u32>,
    /// Only workspaces whose name contains this text.
    pub search: Option<String>,
    /// `all` (the default), `active` or `suspended`.
    pub status: Option<String>,
    /// Only workspaces in this verification state: `unverified`, or a status
    /// such as `verified` or `rejected`.
    pub verification: Option<String>,
    /// `name_asc`, `name_desc`, `created_asc`, `created_desc` (the default),
    /// `credits_asc` or `credits_desc`.
    pub sort: Option<String>,
    /// Only workspaces with any of these tag IDs.
    pub tags: Option<Vec<String>>,
    /// Every workspace with only its ID, name and balance, unpaginated; the
    /// other options do not apply.
    pub compact: bool,
}

impl ListWorkspacesOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.clamp(1, 100));
        self
    }

    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn verification(mut self, verification: impl Into<String>) -> Self {
        self.verification = Some(verification.into());
        self
    }

    pub fn sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    pub fn tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        self.tags = Some(tags.into_iter().map(|t| t.into()).collect());
        self
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    pub(crate) fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(page) = self.page {
            params.push(("page".to_string(), page.to_string()));
        }
        if let Some(limit) = self.limit.filter(|&limit| limit > 0) {
            params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(ref search) = self.search {
            params.push(("search".to_string(), search.clone()));
        }
        if let Some(ref status) = self.status {
            params.push(("status".to_string(), status.clone()));
        }
        if let Some(ref verification) = self.verification {
            params.push(("verification".to_string(), verification.clone()));
        }
        if let Some(ref sort) = self.sort {
            params.push(("sort".to_string(), sort.clone()));
        }
        if let Some(ref tags) = self.tags {
            params.push(("tags".to_string(), tags.join(",")));
        }
        if self.compact {
            params.push(("compact".to_string(), "true".to_string()));
        }
        params
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteWorkspaceResponse {
    #[serde(default)]
    pub success: bool,
    #[serde(default, alias = "deletedId")]
    pub deleted_id: Option<String>,
}

/// Verification submit/resubmit payload. All fields optional for resubmits
/// (server merges with existing record). For initial provision via
/// `submit_verification` (no existing record), the server validator requires:
/// `business_name`, `website`, `address`, `contact`, `use_case`,
/// `use_case_summary`, `sample_messages`, `opt_in_workflow`.
///
/// For sole proprietors, leave `brn`, `brn_type`, `brn_country` as `None` —
/// the server strips them before forwarding to the carrier.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationSubmitInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doing_business_as: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<VerificationAddress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<VerificationContact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brn_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brn_country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_case: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_case_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_messages: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_in_workflow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_in_image_urls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_volume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_information: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_gated_content: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isv_reseller: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_url: Option<String>,
}

/// Backwards-compatible alias. New code should use `VerificationSubmitInput`.
pub type SubmitVerificationRequest = VerificationSubmitInput;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationAddress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerificationContact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
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

/// Options for `enterprise().workspaces().inherit_verification_with_options()`.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct InheritVerificationOptions {
    /// The workspace whose verification to copy.
    #[serde(rename = "sourceWorkspaceId")]
    pub source_workspace_id: String,
    /// Copy only the business details and order the workspace its own
    /// toll-free number, which is submitted for verification on its own,
    /// instead of sharing the source workspace's number.
    #[serde(
        rename = "purchaseNewNumber",
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub purchase_new_number: bool,
}

impl InheritVerificationOptions {
    /// Inherits from this source workspace, sharing its verified number.
    pub fn new(source_workspace_id: impl Into<String>) -> Self {
        Self {
            source_workspace_id: source_workspace_id.into(),
            purchase_new_number: false,
        }
    }

    /// Orders the workspace its own toll-free number instead of sharing the
    /// source workspace's.
    pub fn purchase_new_number(mut self, purchase_new_number: bool) -> Self {
        self.purchase_new_number = purchase_new_number;
        self
    }
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
    /// True when the workspace was given its own new toll-free number
    /// instead of sharing the source workspace's.
    #[serde(default, alias = "newNumber")]
    pub new_number: bool,
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

/// A page provisioning generated for a workspace, or why it could not.
#[derive(Debug, Clone, Default, Deserialize)]
#[non_exhaustive]
pub struct ProvisionedPage {
    /// The page's ID.
    #[serde(default)]
    pub id: Option<String>,
    /// The page's slug.
    #[serde(default)]
    pub slug: Option<String>,
    /// The hosted page's URL.
    #[serde(default)]
    pub url: Option<String>,
    /// Why the page could not be generated; the rest of provisioning still
    /// went ahead.
    #[serde(default)]
    pub error: Option<String>,
}

/// The privacy policy and terms pages provisioning generated, or why it
/// could not.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ProvisionedLegalPages {
    /// The hosted privacy policy's URL.
    #[serde(default)]
    pub privacy_url: Option<String>,
    /// The hosted terms of service's URL.
    #[serde(default)]
    pub terms_url: Option<String>,
    /// The privacy policy page's ID.
    #[serde(default)]
    pub privacy_page_id: Option<String>,
    /// The terms of service page's ID.
    #[serde(default)]
    pub terms_page_id: Option<String>,
    /// Why the pages could not be generated; the rest of provisioning still
    /// went ahead.
    #[serde(default)]
    pub error: Option<String>,
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
    /// The hosted opt-in page, when one was generated or failed to be.
    #[serde(default, alias = "optInPage")]
    pub opt_in_page: Option<ProvisionedPage>,
    /// The hosted privacy policy and terms pages, when generated or failed.
    #[serde(default, alias = "legalPages")]
    pub legal_pages: Option<ProvisionedLegalPages>,
    /// The hosted business page, when one was generated or failed to be.
    #[serde(default, alias = "businessPage")]
    pub business_page: Option<ProvisionedPage>,
    /// The API base URL the new workspace's key works against.
    #[serde(default, alias = "apiBaseUrl")]
    pub api_base_url: Option<String>,
    /// The new workspace in the Sendly dashboard.
    #[serde(default, alias = "dashboardUrl")]
    pub dashboard_url: Option<String>,
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
    /// The event types the webhook receives; `None` means all of them.
    #[serde(default)]
    pub events: Option<Vec<String>>,
    /// The workspaces whose events the webhook receives; `None` means all of
    /// them.
    #[serde(default)]
    pub workspaces: Option<Vec<String>>,
    /// The signing secret. Returned only when it is new: by the first
    /// `set()` and by `rotate_secret()`. Store it; it is not shown again.
    #[serde(default, alias = "signingSecret", alias = "secret")]
    pub signing_secret: Option<String>,
    /// When the secret was rotated, on a rotation.
    #[serde(default, alias = "rotatedAt")]
    pub rotated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnterpriseWebhookTestResult {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default, alias = "statusCode")]
    pub status_code: Option<i32>,
    /// The HTTP status text the endpoint answered with.
    #[serde(default, alias = "statusText")]
    pub status_text: Option<String>,
    /// Why the test event could not be delivered, when the request failed.
    #[serde(default)]
    pub error: Option<String>,
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

#[deprecated(note = "The credits analytics endpoint reports totals, not a daily series.")]
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

/// Credit totals across the enterprise's workspaces.
#[allow(deprecated)]
#[derive(Debug, Clone, Deserialize)]
pub struct CreditsAnalytics {
    #[serde(default)]
    pub period: String,
    #[deprecated(
        note = "The API reports totals, not a daily series; this is always empty. Read `total_balance`, `total_lifetime`, `total_used` and `workspace_count`."
    )]
    #[serde(default)]
    pub data: Vec<CreditDataPoint>,
    /// Credits the workspaces hold now.
    #[serde(default, alias = "totalBalance")]
    pub total_balance: i64,
    /// Credits the workspaces have ever received.
    #[serde(default, alias = "totalLifetime")]
    pub total_lifetime: i64,
    /// Credits the workspaces have used: lifetime minus balance.
    #[serde(default, alias = "totalUsed")]
    pub total_used: i64,
    /// Number of workspaces counted.
    #[serde(default, alias = "workspaceCount")]
    pub workspace_count: i64,
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

#[derive(Deserialize)]
struct WorkspaceWebhookTestResultWire {
    #[serde(default)]
    success: bool,
    #[serde(default)]
    message: Option<String>,
    #[serde(default, alias = "statusCode")]
    status_code: Option<i32>,
    #[serde(default)]
    delivery: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "WorkspaceWebhookTestResultWire")]
pub struct WorkspaceWebhookTestResult {
    pub success: bool,
    pub message: Option<String>,
    /// The HTTP status the endpoint answered the test event with.
    pub status_code: Option<i32>,
    /// The test delivery the API recorded (`id`, `status`, `status_code`,
    /// `response_time`, `error`, ...).
    pub delivery: Option<serde_json::Value>,
}

impl From<WorkspaceWebhookTestResultWire> for WorkspaceWebhookTestResult {
    fn from(wire: WorkspaceWebhookTestResultWire) -> Self {
        let status_code = wire.status_code.or_else(|| {
            wire.delivery
                .as_ref()
                .and_then(|d| d.get("status_code"))
                .and_then(|v| v.as_i64())
                .and_then(|n| i32::try_from(n).ok())
        });
        Self {
            success: wire.success,
            message: wire.message,
            status_code,
            delivery: wire.delivery,
        }
    }
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

/// Request to reply to a conversation. Provide `text`, `media_urls` or
/// both.
#[derive(Debug, Clone, Serialize)]
pub struct ReplyToConversationRequest {
    /// Message content; may be empty when `media_urls` is set.
    #[serde(skip_serializing_if = "String::is_empty")]
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
// Conversation Context
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ConversationContextResponse {
    pub context: String,
    pub conversation: ConversationContextInfo,
    #[serde(default, alias = "tokenEstimate")]
    pub token_estimate: i64,
    #[serde(default)]
    pub business: Option<ConversationContextBusiness>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConversationContextInfo {
    pub id: String,
    #[serde(alias = "phoneNumber")]
    pub phone_number: String,
    pub status: String,
    #[serde(default, alias = "messageCount")]
    pub message_count: i32,
    #[serde(default, alias = "unreadCount")]
    pub unread_count: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConversationContextBusiness {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, alias = "useCase")]
    pub use_case: Option<String>,
}

// ============================================================================
// Rules
// ============================================================================

/// A value a rule condition matches: one value, or any of several.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RuleMatch {
    /// Matches this value.
    One(String),
    /// Matches any of these values.
    Any(Vec<String>),
}

impl From<&str> for RuleMatch {
    fn from(value: &str) -> Self {
        RuleMatch::One(value.to_string())
    }
}

impl From<String> for RuleMatch {
    fn from(value: String) -> Self {
        RuleMatch::One(value)
    }
}

impl<S: Into<String>> From<Vec<S>> for RuleMatch {
    fn from(values: Vec<S>) -> Self {
        RuleMatch::Any(values.into_iter().map(Into::into).collect())
    }
}

/// What a rule matches in an inbound message's AI classification. Every
/// condition that is set must hold; a rule with none matches every message.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleConditions {
    /// The classified intent, such as `complaint`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<RuleMatch>,
    /// The classified sentiment, such as `negative`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sentiment: Option<RuleMatch>,
    /// The lowest intent confidence (0-1) the rule accepts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent_confidence_min: Option<f64>,
    /// The lowest sentiment confidence (0-1) the rule accepts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sentiment_confidence_min: Option<f64>,
}

impl RuleConditions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intent(mut self, intent: impl Into<RuleMatch>) -> Self {
        self.intent = Some(intent.into());
        self
    }

    pub fn sentiment(mut self, sentiment: impl Into<RuleMatch>) -> Self {
        self.sentiment = Some(sentiment.into());
        self
    }

    pub fn intent_confidence_min(mut self, min: f64) -> Self {
        self.intent_confidence_min = Some(min);
        self
    }

    pub fn sentiment_confidence_min(mut self, min: f64) -> Self {
        self.sentiment_confidence_min = Some(min);
        self
    }
}

/// What a rule does to the conversation of a message it matches.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleActions {
    /// Label IDs to add to the conversation.
    #[serde(default)]
    pub add_labels: Vec<String>,
    /// Whether to close the conversation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close_conversation: Option<bool>,
}

impl RuleActions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_labels(mut self, label_ids: Vec<impl Into<String>>) -> Self {
        self.add_labels = label_ids.into_iter().map(|id| id.into()).collect();
        self
    }

    pub fn close_conversation(mut self, close: bool) -> Self {
        self.close_conversation = Some(close);
        self
    }
}

fn deserialize_rule_part<'de, D, T>(deserializer: D) -> std::result::Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::de::DeserializeOwned + Default,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Array(items) => Ok(items
            .into_iter()
            .next()
            .and_then(|first| serde_json::from_value(first).ok())
            .unwrap_or_default()),
        serde_json::Value::Null => Ok(T::default()),
        object => serde_json::from_value(object).map_err(serde::de::Error::custom),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    /// What the rule matches. A rule stored by an SDK that sent a list reads
    /// back as its first entry; the API never evaluated such a rule.
    #[serde(deserialize_with = "deserialize_rule_part")]
    pub conditions: RuleConditions,
    /// What the rule does. A rule stored by an SDK that sent a list reads
    /// back as its first entry; the API never applied such a rule.
    #[serde(deserialize_with = "deserialize_rule_part")]
    pub actions: RuleActions,
    #[serde(default)]
    pub priority: i32,
    /// Whether the rule runs. A new rule is enabled.
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    #[serde(default, alias = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuleListResponse {
    #[serde(default)]
    pub data: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateRuleRequest {
    pub name: String,
    pub conditions: RuleConditions,
    pub actions: RuleActions,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateRuleRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditions: Option<RuleConditions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<RuleActions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    /// `false` switches the rule off; `true` switches it back on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

// ============================================================================
// Generated Template
// ============================================================================

#[derive(Debug, Clone, Serialize)]
pub struct GenerateTemplateRequest {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GeneratedTemplate {
    pub name: String,
    pub text: String,
    #[serde(default)]
    pub variables: Vec<String>,
    pub category: String,
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

/// A single AI-generated suggested reply.
#[derive(Debug, Clone, Deserialize)]
pub struct SuggestedReply {
    /// The suggested reply text.
    pub text: String,
    /// The tone of the suggested reply.
    pub tone: String,
}

/// Response from `conversations.suggest_replies()`.
#[derive(Debug, Clone, Deserialize)]
pub struct SuggestRepliesResponse {
    /// The suggested replies.
    pub suggestions: Vec<SuggestedReply>,
    /// The message ID the suggestions were generated against.
    #[serde(default, alias = "basedOnMessageId")]
    pub based_on_message_id: Option<String>,
    /// The model that produced the suggestions.
    #[serde(default)]
    pub model: Option<String>,
}

/// Variable values for one dynamic-URL button on an approved WhatsApp
/// template.
#[derive(Debug, Clone, Serialize)]
pub struct WhatsAppTemplateButtonVariables {
    /// Zero-based index of the button on the approved template.
    pub index: u32,
    /// Values for the button's URL placeholders, keyed by placeholder
    /// number: `{ "1": "4821" }`.
    pub variables: std::collections::HashMap<String, String>,
}

/// The approved WhatsApp template to send, with its variable values.
#[derive(Debug, Clone, Serialize)]
pub struct WhatsAppTemplateSendParams {
    /// Template name as approved (e.g. "order_shipped").
    pub name: String,
    /// Template language code (e.g. "en_US") — must match the approved
    /// template's language exactly.
    pub language: String,
    /// Body variable values keyed by placeholder number:
    /// `{ "1": "Acme Inc", "2": "#4821" }`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<std::collections::HashMap<String, String>>,
    /// Variable values for dynamic-URL buttons.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<WhatsAppTemplateButtonVariables>>,
}

impl WhatsAppTemplateSendParams {
    /// Creates template send params for the given approved template.
    pub fn new(name: impl Into<String>, language: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            language: language.into(),
            variables: None,
            buttons: None,
        }
    }

    /// Sets the body variable values.
    pub fn with_variables(mut self, variables: std::collections::HashMap<String, String>) -> Self {
        self.variables = Some(variables);
        self
    }

    /// Sets the variable values for dynamic-URL buttons.
    pub fn with_buttons(mut self, buttons: Vec<WhatsAppTemplateButtonVariables>) -> Self {
        self.buttons = Some(buttons);
        self
    }
}

/// Request to send a WhatsApp message.
///
/// Provide exactly one of:
/// - `text` — free-form text; only deliverable inside an open 24-hour
///   customer-service window (the recipient messaged you in the last 24h)
/// - `media_urls` — a single media attachment (optional `text` becomes its
///   caption); also window-bound
/// - `template` — an approved template; works regardless of the window
///
/// WhatsApp sends require the `sms:send` scope, a live API key and a `from`
/// number that has been connected to WhatsApp (see
/// `client.whatsapp().signup()`).
///
/// Construct with [`SendWhatsAppMessageRequest::new`] and the `with_*`
/// builder methods. This type is `#[non_exhaustive]`, so external crates
/// must use the constructor rather than a struct literal.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct SendWhatsAppMessageRequest {
    channel: &'static str,
    /// Destination phone number in E.164 format (e.g., +15551234567).
    pub to: String,
    /// Sending number in E.164 format. Required — must be one of your
    /// numbers with an active WhatsApp connection.
    pub from: String,
    /// Free-form message text (max 4096 bytes), or the caption when
    /// `media_urls` is provided (max 1024 bytes). Requires an open 24-hour
    /// window — outside it the API responds 422 `whatsapp_window_closed`;
    /// send a `template` instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Media attachment URL. WhatsApp accepts exactly one per message.
    /// Must be a publicly accessible HTTPS URL.
    #[serde(skip_serializing_if = "Option::is_none", rename = "mediaUrls")]
    pub media_urls: Option<Vec<String>>,
    /// Approved template to send. Works regardless of the 24-hour window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<WhatsAppTemplateSendParams>,
    /// Custom JSON metadata to attach to the message (max 4KB).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
}

impl SendWhatsAppMessageRequest {
    /// Creates a new WhatsApp send request for the given recipient and
    /// WhatsApp-connected sending number.
    pub fn new(to: impl Into<String>, from: impl Into<String>) -> Self {
        Self {
            channel: "whatsapp",
            to: to.into(),
            from: from.into(),
            text: None,
            media_urls: None,
            template: None,
            metadata: None,
        }
    }

    /// Sets the free-form text (or the media caption when `media_urls` is
    /// also set).
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Sets the media attachment URL (WhatsApp accepts exactly one).
    pub fn with_media_urls(mut self, media_urls: Vec<String>) -> Self {
        self.media_urls = Some(media_urls);
        self
    }

    /// Sets the approved template to send.
    pub fn with_template(mut self, template: WhatsAppTemplateSendParams) -> Self {
        self.template = Some(template);
        self
    }

    /// Sets custom metadata to attach to the message.
    pub fn with_metadata(
        mut self,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// What kind of WhatsApp message was sent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum WhatsAppMessageKind {
    /// Free-form text.
    Text,
    /// Media attachment (with optional caption).
    Media,
    /// Approved template.
    Template,
    /// A kind this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

/// Billing category of a sent WhatsApp template (Meta reviews and may
/// reclassify templates; the category on the send response is what was
/// billed).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum WhatsAppMessageCategory {
    Marketing,
    Utility,
    Authentication,
    /// A category this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

/// The template that was sent (template sends only).
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppMessageTemplateInfo {
    /// Template name.
    pub name: String,
    /// Template language code.
    pub language: String,
    /// Billed category.
    pub category: WhatsAppMessageCategory,
}

/// WhatsApp-specific details on a sent message.
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppMessageDetails {
    /// What was sent: free-form text, media, or a template.
    pub kind: WhatsAppMessageKind,
    /// The template that was sent (template sends only).
    #[serde(default)]
    pub template: Option<WhatsAppMessageTemplateInfo>,
    /// WhatsApp message id — `None` until the first delivery report lands;
    /// populated on the message record afterwards.
    #[serde(default, alias = "messageId")]
    pub message_id: Option<String>,
}

/// A sent WhatsApp message.
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppMessage {
    /// Unique message identifier.
    pub id: String,
    /// Always "whatsapp".
    #[serde(default)]
    pub channel: String,
    /// Always "whatsapp".
    #[serde(default)]
    pub message_format: String,
    /// Destination phone number.
    pub to: String,
    /// Sending number.
    pub from: String,
    /// Body text for free-form text sends; `None` for template and media
    /// sends.
    #[serde(default)]
    pub text: Option<String>,
    /// Current delivery status.
    pub status: MessageStatus,
    /// Always 1 — WhatsApp has no segment concept.
    #[serde(default = "default_segments")]
    pub segments: i32,
    /// Credits charged for this message. Free-form text or media inside the
    /// 24-hour window: 1 credit each for the first 1,000 per sending number
    /// per calendar month (UTC), then the destination's utility template
    /// price; countries without a listed price use the default utility price
    /// of 12 credits. Templates are priced by category and destination
    /// country; countries without a listed price use 33 (marketing), 12
    /// (utility) and 12 (authentication) credits. A failed send gives its
    /// slot back.
    #[serde(default, alias = "creditsUsed")]
    pub credits_used: i32,
    /// WhatsApp-specific details.
    pub whatsapp: WhatsAppMessageDetails,
    /// ISO 8601 timestamp when the message was created.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// Custom JSON metadata attached to the message.
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// A tappable chip that sends a reply back when tapped.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RcsSuggestedReply {
    /// Chip label — what the recipient sees, and what they send back.
    pub text: String,
    /// Machine-readable payload returned on the inbound reply.
    pub postback_data: String,
}

/// A tappable chip that opens a URL when tapped.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RcsSuggestedAction {
    /// Chip label.
    pub text: String,
    /// Machine-readable payload reported when the chip is tapped.
    pub postback_data: String,
    /// Link opened when the chip is tapped.
    pub url: String,
}

/// A tappable chip on an RCS message: either a reply or a URL action.
///
/// Build one with [`RcsSuggestion::reply`] or [`RcsSuggestion::action`].
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RcsSuggestion {
    /// Sends `text` back as a reply, carrying `postback_data`.
    Reply(RcsSuggestedReply),
    /// Opens `url`.
    Action(RcsSuggestedAction),
}

impl RcsSuggestion {
    /// Creates a reply chip.
    pub fn reply(text: impl Into<String>, postback_data: impl Into<String>) -> Self {
        RcsSuggestion::Reply(RcsSuggestedReply {
            text: text.into(),
            postback_data: postback_data.into(),
        })
    }

    /// Creates a chip that opens a URL.
    pub fn action(
        text: impl Into<String>,
        postback_data: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        RcsSuggestion::Action(RcsSuggestedAction {
            text: text.into(),
            postback_data: postback_data.into(),
            url: url.into(),
        })
    }
}

/// Layout of an RCS rich card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RcsCardOrientation {
    /// Media above the text (the default).
    Vertical,
    /// Media beside the text.
    Horizontal,
}

/// A standalone RCS rich card: a title and description with an optional
/// image and tappable chips.
///
/// Construct with [`RcsCard::new`] and the `with_*` builder methods.
#[derive(Debug, Clone, Serialize)]
pub struct RcsCard {
    /// Card title. Required.
    pub title: String,
    /// Card body text. Required.
    pub description: String,
    /// Public JPEG, PNG, or GIF image URL shown on the card.
    #[serde(skip_serializing_if = "Option::is_none", rename = "mediaUrl")]
    pub media_url: Option<String>,
    /// Card layout; defaults to vertical.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<RcsCardOrientation>,
    /// Tappable chips on the card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestions: Option<Vec<RcsSuggestion>>,
}

impl RcsCard {
    /// Creates a card with the required title and description.
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            media_url: None,
            orientation: None,
            suggestions: None,
        }
    }

    /// Sets the card image (public JPEG, PNG, or GIF URL).
    pub fn with_media_url(mut self, media_url: impl Into<String>) -> Self {
        self.media_url = Some(media_url.into());
        self
    }

    /// Sets the card layout.
    pub fn with_orientation(mut self, orientation: RcsCardOrientation) -> Self {
        self.orientation = Some(orientation);
        self
    }

    /// Sets the card's tappable chips.
    pub fn with_suggestions(mut self, suggestions: Vec<RcsSuggestion>) -> Self {
        self.suggestions = Some(suggestions);
        self
    }
}

/// Request to send an RCS message.
///
/// Provide exactly one of:
/// - `text` — rich text; optional tappable `suggestions` ride along
/// - `card` — a standalone rich card (title and description, with an
///   optional image and card-level chips)
///
/// When the recipient's device or network doesn't support RCS, text sends
/// fall back to plain SMS (billed as SMS) unless `fallback_to_sms` is set to
/// `false`; suggestions have no SMS form and are dropped on the fallback.
/// Cards have no SMS form and never fall back — an unsupported recipient
/// gets a 422 `rcs_not_supported_for_recipient`.
///
/// RCS sends require a live API key and a sendable RCS agent on the
/// workspace (see `client.rcs().agents()`).
///
/// Construct with [`SendRcsMessageRequest::new`] and the `with_*` builder
/// methods. This type is `#[non_exhaustive]`, so external crates must use
/// the constructor rather than a struct literal.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct SendRcsMessageRequest {
    channel: &'static str,
    /// Destination phone number in E.164 format (e.g., +15551234567).
    pub to: String,
    /// RCS agent to send as. Optional when the workspace has exactly one
    /// sendable agent; required when it has more than one.
    #[serde(skip_serializing_if = "Option::is_none", rename = "agentId")]
    pub agent_id: Option<String>,
    /// Message text. Exactly one of `text` or `card` is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Tappable chips riding on a text message. Not valid with `card` —
    /// put card buttons in [`RcsCard::with_suggestions`] instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestions: Option<Vec<RcsSuggestion>>,
    /// Rich card to send. Exactly one of `text` or `card` is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub card: Option<RcsCard>,
    /// Whether a text send may fall back to SMS for a recipient without RCS
    /// support. Defaults to `true` when omitted.
    #[serde(skip_serializing_if = "Option::is_none", rename = "fallbackToSms")]
    pub fallback_to_sms: Option<bool>,
    /// Custom JSON metadata to attach to the message (max 4KB).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
}

impl SendRcsMessageRequest {
    /// Creates a new RCS send request for the given recipient.
    pub fn new(to: impl Into<String>) -> Self {
        Self {
            channel: "rcs",
            to: to.into(),
            agent_id: None,
            text: None,
            suggestions: None,
            card: None,
            fallback_to_sms: None,
            metadata: None,
        }
    }

    /// Sets the message text.
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Sets the tappable chips riding on a text message.
    pub fn with_suggestions(mut self, suggestions: Vec<RcsSuggestion>) -> Self {
        self.suggestions = Some(suggestions);
        self
    }

    /// Sets the rich card to send.
    pub fn with_card(mut self, card: RcsCard) -> Self {
        self.card = Some(card);
        self
    }

    /// Sets the RCS agent to send as.
    pub fn with_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    /// Turns the SMS fallback off (`false`) or explicitly on (`true`).
    pub fn with_fallback_to_sms(mut self, fallback_to_sms: bool) -> Self {
        self.fallback_to_sms = Some(fallback_to_sms);
        self
    }

    /// Sets custom metadata to attach to the message.
    pub fn with_metadata(
        mut self,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// What was sent over RCS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RcsMessageKind {
    /// Rich text (with optional chips).
    Text,
    /// Standalone rich card.
    Card,
}

/// RCS-specific details on a sent message.
///
/// On an RCS delivery, `kind` and `agent_name` are set. On an SMS fallback,
/// `requested_channel` is `"rcs"` and `suggestions_dropped` reports whether
/// chips were dropped on the way out.
#[derive(Debug, Clone, Deserialize)]
pub struct RcsMessageDetails {
    /// What was sent over RCS; `None` on an SMS fallback.
    #[serde(default)]
    pub kind: Option<RcsMessageKind>,
    /// The RCS agent the send was routed through.
    #[serde(alias = "agentId")]
    pub agent_id: String,
    /// Agent name recipients see; `None` on an SMS fallback.
    #[serde(default, alias = "agentName")]
    pub agent_name: Option<String>,
    /// `"rcs"` on an SMS fallback — the channel requested before the
    /// recipient's RCS support was probed; `None` on an RCS delivery.
    #[serde(default, alias = "requestedChannel")]
    pub requested_channel: Option<String>,
    /// True when the message fell back to SMS and its chips were dropped
    /// (suggestions have no SMS form).
    #[serde(default, alias = "suggestionsDropped")]
    pub suggestions_dropped: bool,
}

/// A sent RCS message — or the SMS it fell back to.
///
/// `channel` reports the leg that actually delivered: `"rcs"` when the
/// message went out over RCS, `"sms"` when it fell back for a recipient
/// without RCS support. [`fell_back_to_sms`](Self::fell_back_to_sms) is the
/// direct check; billing follows the leg (`credits_used` is SMS pricing on
/// a fallback).
#[derive(Debug, Clone, Deserialize)]
pub struct RcsMessage {
    /// Unique message identifier.
    pub id: String,
    /// `"rcs"` on an RCS delivery, `"sms"` on a fallback.
    #[serde(default)]
    pub channel: String,
    /// `"sms"` when the message fell back to SMS; `None` on an RCS delivery.
    #[serde(default, alias = "fellBackTo")]
    pub fell_back_to: Option<String>,
    /// Matches `channel`: `"rcs"` or `"sms"`.
    #[serde(default)]
    pub message_format: String,
    /// Destination phone number.
    pub to: String,
    /// RCS agent name, or the SMS sender on a fallback.
    pub from: String,
    /// Message text for text sends; `None` for card sends.
    #[serde(default)]
    pub text: Option<String>,
    /// Current delivery status.
    pub status: MessageStatus,
    /// 1 on an RCS delivery; the SMS segment count on a fallback.
    #[serde(default = "default_segments")]
    pub segments: i32,
    /// Credits charged — RCS pricing on an RCS delivery, SMS pricing on a
    /// fallback.
    #[serde(default, alias = "creditsUsed")]
    pub credits_used: i32,
    /// RCS-specific details.
    pub rcs: RcsMessageDetails,
    /// ISO 8601 timestamp when the message was created.
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    /// Custom JSON metadata attached to the message.
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, serde_json::Value>>,
}

impl RcsMessage {
    /// True when the recipient had no RCS support and the message was
    /// delivered as plain SMS instead (billed as SMS).
    pub fn fell_back_to_sms(&self) -> bool {
        self.fell_back_to.as_deref() == Some("sms")
    }
}
