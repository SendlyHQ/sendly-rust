use regex::Regex;
use std::sync::OnceLock;

use crate::client::Sendly;
use crate::error::{Error, Result};
use crate::models::{
    BatchList, BatchMessageResponse, BatchPreviewResponse, CancelScheduledMessageResponse,
    EnhanceMessageRequest, EnhanceMessageResponse, GroupMessageResponse, ListBatchesOptions,
    ListMessagesOptions, ListScheduledMessagesOptions, Message, MessageList, ScheduleMessageRequest,
    ScheduledMessage, ScheduledMessageList, SendBatchRequest, SendGroupMessageRequest,
    SendMessageRequest, SendWhatsAppMessageRequest, WhatsAppMessage,
};

static PHONE_REGEX: OnceLock<Regex> = OnceLock::new();

fn phone_regex() -> &'static Regex {
    PHONE_REGEX.get_or_init(|| Regex::new(r"^\+[1-9]\d{1,14}$").unwrap())
}

const MAX_TEXT_LENGTH: usize = 1600;

/// Messages resource for sending and managing SMS.
#[derive(Debug, Clone)]
pub struct Messages<'a> {
    client: &'a Sendly,
}

impl<'a> Messages<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Sends an SMS message.
    ///
    /// # Arguments
    ///
    /// * `request` - The send message request
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, SendMessageRequest};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let message = client.messages()
    ///     .send(SendMessageRequest::new("+15551234567", "Hello from Sendly!"))
    ///     .await?;
    ///
    /// println!("Sent: {}", message.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send(&self, request: SendMessageRequest) -> Result<Message> {
        validate_phone(&request.to)?;
        let has_media = request.media_urls.as_ref().map_or(false, |u| !u.is_empty());
        if !has_media {
            validate_text(&request.text)?;
        }

        let response = self.client.post("/messages", &request).await?;
        let message: Message = response.json().await?;

        Ok(message)
    }

    /// Sends a WhatsApp message.
    ///
    /// Requires a live API key and a `from` number with an active WhatsApp
    /// connection (see `client.whatsapp().signup()`). Free-form `text` and
    /// media only deliver inside an open 24-hour customer-service window —
    /// outside it, send an approved `template` instead (check with
    /// `client.whatsapp().window(from, to)`).
    ///
    /// # Arguments
    ///
    /// * `request` - WhatsApp message details
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, SendWhatsAppMessageRequest, WhatsAppTemplateSendParams};
    /// use std::collections::HashMap;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// // Free-form reply inside an open 24h window
    /// let message = client.messages().send_whatsapp(
    ///     SendWhatsAppMessageRequest::new("+15551234567", "+15559876543")
    ///         .with_text("Your table is ready!"),
    /// ).await?;
    ///
    /// // Template send — works regardless of the window
    /// let mut variables = HashMap::new();
    /// variables.insert("1".to_string(), "Acme Inc".to_string());
    /// variables.insert("2".to_string(), "#4821".to_string());
    /// let message = client.messages().send_whatsapp(
    ///     SendWhatsAppMessageRequest::new("+15551234567", "+15559876543")
    ///         .with_template(
    ///             WhatsAppTemplateSendParams::new("order_shipped", "en_US")
    ///                 .with_variables(variables),
    ///         ),
    /// ).await?;
    ///
    /// println!("Kind: {:?}", message.whatsapp.kind);
    /// println!("Credits: {}", message.credits_used);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_whatsapp(
        &self,
        request: SendWhatsAppMessageRequest,
    ) -> Result<WhatsAppMessage> {
        validate_phone(&request.to)?;
        validate_phone(&request.from)?;
        let has_media = request.media_urls.as_ref().map_or(false, |u| !u.is_empty());
        let has_text = request.text.as_ref().map_or(false, |t| !t.is_empty());
        if !has_text && !has_media && request.template.is_none() {
            return Err(Error::Validation {
                message: "Provide 'text', 'media_urls', or 'template'".to_string(),
            });
        }

        let response = self.client.post("/messages", &request).await?;
        let message: WhatsAppMessage = response.json().await?;

        Ok(message)
    }

    /// Sends an SMS message with simple parameters.
    ///
    /// # Arguments
    ///
    /// * `to` - Recipient phone number in E.164 format
    /// * `text` - Message content
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let message = client.messages()
    ///     .send_to("+15551234567", "Hello!")
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_to(&self, to: impl Into<String>, text: impl Into<String>) -> Result<Message> {
        self.send(SendMessageRequest {
            to: to.into(),
            text: text.into(),
            from: None,
            message_type: None,
            media_urls: None,
            metadata: None,
        })
        .await
    }

    /// Sends a group MMS to 2-8 recipients (US/Canada only).
    ///
    /// Creates a multi-party MMS conversation: every recipient sees the others,
    /// and replies fan out to all participants. Group messaging is an A2P 10DLC
    /// capability — the sending number must be an MMS-enabled, 10DLC-registered
    /// number you own. Omit `from` to use your workspace's default sender.
    ///
    /// # Arguments
    ///
    /// * `request` - Group message details (2-8 recipients, text and/or media)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, SendGroupMessageRequest};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let group = client.messages().send_group(
    ///     SendGroupMessageRequest::new(vec![
    ///         "+14155551234".to_string(),
    ///         "+14155555678".to_string(),
    ///     ])
    ///     .with_text("Hey team - quick sync at noon?"),
    /// ).await?;
    ///
    /// println!("Group: {:?}", group.group_message_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_group(
        &self,
        request: SendGroupMessageRequest,
    ) -> Result<GroupMessageResponse> {
        if request.to.len() < 2 {
            return Err(Error::Validation {
                message: "Group messaging requires at least 2 recipients".to_string(),
            });
        }
        if request.to.len() > 8 {
            return Err(Error::Validation {
                message: "Group messaging supports at most 8 recipients".to_string(),
            });
        }
        for recipient in &request.to {
            validate_phone(recipient)?;
        }
        let has_media = request.media_urls.as_ref().map_or(false, |u| !u.is_empty());
        let has_text = request.text.as_ref().map_or(false, |t| !t.is_empty());
        if !has_text && !has_media {
            return Err(Error::Validation {
                message: "Provide 'text' or 'media_urls'".to_string(),
            });
        }

        let response = self.client.post("/messages/group", &request).await?;
        let result: GroupMessageResponse = response.json().await?;

        Ok(result)
    }

    /// AI-enhances a draft message for clarity, compliance, and send-readiness.
    ///
    /// Rewrites the supplied text into a single, polished SMS segment (≤160
    /// chars) and returns a short explanation of what changed. Pass
    /// `message_type` to steer the rewrite (e.g. "marketing" vs
    /// "transactional"); with no `text` it generates a suitable message for that
    /// type instead. At least one of `text` or `message_type` is required.
    ///
    /// If AI enhancement is unavailable for the account, the response falls back
    /// to the original text with an empty explanation.
    ///
    /// # Arguments
    ///
    /// * `request` - The draft text and/or a message-type hint
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, EnhanceMessageRequest};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let result = client.messages().enhance(
    ///     EnhanceMessageRequest::new()
    ///         .with_text("hey come check out our sale this weekend")
    ///         .with_message_type("marketing"),
    /// ).await?;
    ///
    /// println!("{}", result.enhanced);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn enhance(
        &self,
        request: EnhanceMessageRequest,
    ) -> Result<EnhanceMessageResponse> {
        let has_text = request.text.as_ref().map_or(false, |t| !t.is_empty());
        let has_type = request
            .message_type
            .as_ref()
            .map_or(false, |t| !t.is_empty());
        if !has_text && !has_type {
            return Err(Error::Validation {
                message: "Provide 'text' or 'message_type'".to_string(),
            });
        }

        let response = self.client.post("/ai/enhance", &request).await?;
        let result: EnhanceMessageResponse = response.json().await?;

        Ok(result)
    }

    /// Lists messages.
    ///
    /// # Arguments
    ///
    /// * `options` - Optional query options
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, ListMessagesOptions, MessageStatus};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// // List all
    /// let messages = client.messages().list(None).await?;
    ///
    /// // With options
    /// let messages = client.messages().list(Some(
    ///     ListMessagesOptions::new()
    ///         .limit(50)
    ///         .status(MessageStatus::Delivered)
    /// )).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self, options: Option<ListMessagesOptions>) -> Result<MessageList> {
        let query = options.map(|o| o.to_query_params()).unwrap_or_default();

        let response = self.client.get("/messages", &query).await?;
        let result: MessageList = response.json().await?;

        Ok(result)
    }

    /// Gets a message by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Message ID
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let message = client.messages().get("msg_abc123").await?;
    /// println!("Status: {}", message.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, id: &str) -> Result<Message> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Message ID is required".to_string(),
            });
        }

        // URL encode the ID to prevent path injection
        let encoded_id = urlencoding::encode(id);
        let path = format!("/messages/{}", encoded_id);
        let response = self.client.get(&path, &[]).await?;
        let message: Message = response.json().await?;

        Ok(message)
    }

    /// Iterates over all messages with automatic pagination.
    ///
    /// # Arguments
    ///
    /// * `options` - Optional query options
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    /// use futures::StreamExt;
    /// use tokio::pin;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    /// let messages = client.messages();
    /// let stream = messages.iter(None);
    /// pin!(stream);
    /// while let Some(result) = stream.next().await {
    ///     let message = result?;
    ///     println!("{}: {}", message.id, message.to);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn iter(
        &self,
        options: Option<ListMessagesOptions>,
    ) -> impl futures::Stream<Item = Result<Message>> + '_ {
        let options = options.unwrap_or_default();
        let mut offset = options.offset.unwrap_or(0);
        let batch_size = options.limit.unwrap_or(100);
        let status = options.status.clone();
        let to = options.to.clone();

        async_stream::try_stream! {
            loop {
                let mut list_opts = ListMessagesOptions::new()
                    .limit(batch_size)
                    .offset(offset);

                // Only apply filters if specified
                if let Some(ref s) = status {
                    list_opts = list_opts.status(s.clone());
                }
                if let Some(ref t) = to {
                    list_opts = list_opts.to(t.clone());
                }

                let page = self.list(Some(list_opts)).await;

                let page = match page {
                    Ok(p) => p,
                    Err(e) => {
                        Err(e)?;
                        return;
                    }
                };

                let page_len = page.len();

                for message in page {
                    yield message;
                }

                // Stop if we got fewer results than requested
                if page_len < batch_size as usize {
                    break;
                }

                offset += batch_size;
            }
        }
    }
}

fn validate_phone(phone: &str) -> Result<()> {
    if !phone_regex().is_match(phone) {
        return Err(Error::Validation {
            message: "Invalid phone number format. Use E.164 format (e.g., +15551234567)"
                .to_string(),
        });
    }
    Ok(())
}

fn validate_text(text: &str) -> Result<()> {
    if text.is_empty() {
        return Err(Error::Validation {
            message: "Message text is required".to_string(),
        });
    }
    if text.len() > MAX_TEXT_LENGTH {
        return Err(Error::Validation {
            message: format!(
                "Message text exceeds maximum length ({} characters)",
                MAX_TEXT_LENGTH
            ),
        });
    }
    Ok(())
}

// ==================== Schedule Methods ====================

impl<'a> Messages<'a> {
    /// Schedules an SMS message for future delivery.
    ///
    /// # Arguments
    ///
    /// * `request` - The schedule message request
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, ScheduleMessageRequest};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let scheduled = client.messages().schedule(ScheduleMessageRequest {
    ///     to: "+15551234567".to_string(),
    ///     text: "Reminder: Your appointment is tomorrow!".to_string(),
    ///     scheduled_at: "2025-01-20T10:00:00Z".to_string(),
    ///     from: None,
    ///     message_type: None,
    ///     metadata: None,
    /// }).await?;
    ///
    /// println!("Scheduled: {}", scheduled.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn schedule(&self, request: ScheduleMessageRequest) -> Result<ScheduledMessage> {
        validate_phone(&request.to)?;
        validate_text(&request.text)?;

        if request.scheduled_at.is_empty() {
            return Err(Error::Validation {
                message: "scheduled_at is required".to_string(),
            });
        }

        let response = self.client.post("/messages/schedule", &request).await?;
        let scheduled: ScheduledMessage = response.json().await?;

        Ok(scheduled)
    }

    /// Lists scheduled messages.
    ///
    /// # Arguments
    ///
    /// * `options` - Optional query options
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, ListScheduledMessagesOptions};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let scheduled = client.messages().list_scheduled(None).await?;
    /// for msg in scheduled {
    ///     println!("{}: {}", msg.id, msg.scheduled_at);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_scheduled(
        &self,
        options: Option<ListScheduledMessagesOptions>,
    ) -> Result<ScheduledMessageList> {
        let query = options.map(|o| o.to_query_params()).unwrap_or_default();

        let response = self.client.get("/messages/scheduled", &query).await?;
        let result: ScheduledMessageList = response.json().await?;

        Ok(result)
    }

    /// Gets a scheduled message by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Scheduled message ID
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let scheduled = client.messages().get_scheduled("sched_abc123").await?;
    /// println!("Status: {:?}", scheduled.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_scheduled(&self, id: &str) -> Result<ScheduledMessage> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Scheduled message ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let path = format!("/messages/scheduled/{}", encoded_id);
        let response = self.client.get(&path, &[]).await?;
        let scheduled: ScheduledMessage = response.json().await?;

        Ok(scheduled)
    }

    /// Cancels a scheduled message.
    ///
    /// # Arguments
    ///
    /// * `id` - Scheduled message ID
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let result = client.messages().cancel_scheduled("sched_abc123").await?;
    /// println!("Refunded {} credits", result.credits_refunded);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cancel_scheduled(&self, id: &str) -> Result<CancelScheduledMessageResponse> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Scheduled message ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let path = format!("/messages/scheduled/{}", encoded_id);
        let response = self.client.delete(&path).await?;
        let result: CancelScheduledMessageResponse = response.json().await?;

        Ok(result)
    }

    // ==================== Batch Methods ====================

    /// Sends multiple SMS messages in a batch.
    ///
    /// # Arguments
    ///
    /// * `request` - The batch send request
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, SendBatchRequest, BatchMessageItem};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let result = client.messages().send_batch(SendBatchRequest {
    ///     messages: vec![
    ///         BatchMessageItem {
    ///             to: "+15551234567".to_string(),
    ///             text: "Hello Alice!".to_string(),
    ///             metadata: None,
    ///         },
    ///         BatchMessageItem {
    ///             to: "+15559876543".to_string(),
    ///             text: "Hello Bob!".to_string(),
    ///             metadata: None,
    ///         },
    ///     ],
    ///     from: None,
    ///     message_type: None,
    ///     metadata: None,
    /// }).await?;
    ///
    /// println!("Batch {}: {} queued", result.batch_id, result.queued);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_batch(&self, request: SendBatchRequest) -> Result<BatchMessageResponse> {
        if request.messages.is_empty() {
            return Err(Error::Validation {
                message: "Messages array is required".to_string(),
            });
        }

        // Validate each message
        for (i, msg) in request.messages.iter().enumerate() {
            validate_phone(&msg.to).map_err(|_| Error::Validation {
                message: format!("Invalid phone number at index {}", i),
            })?;
            validate_text(&msg.text).map_err(|_| Error::Validation {
                message: format!("Invalid message text at index {}", i),
            })?;
        }

        let response = self.client.post("/messages/batch", &request).await?;
        let result: BatchMessageResponse = response.json().await?;

        Ok(result)
    }

    /// Gets batch status by ID.
    ///
    /// # Arguments
    ///
    /// * `batch_id` - Batch ID
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let batch = client.messages().get_batch("batch_abc123").await?;
    /// println!("{}/{} sent", batch.sent, batch.total);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_batch(&self, batch_id: &str) -> Result<BatchMessageResponse> {
        if batch_id.is_empty() {
            return Err(Error::Validation {
                message: "Batch ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(batch_id);
        let path = format!("/messages/batch/{}", encoded_id);
        let response = self.client.get(&path, &[]).await?;
        let result: BatchMessageResponse = response.json().await?;

        Ok(result)
    }

    /// Lists batches.
    ///
    /// # Arguments
    ///
    /// * `options` - Optional query options
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, ListBatchesOptions};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let batches = client.messages().list_batches(None).await?;
    /// for batch in batches {
    ///     println!("{}: {:?}", batch.batch_id, batch.status);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_batches(&self, options: Option<ListBatchesOptions>) -> Result<BatchList> {
        let query = options.map(|o| o.to_query_params()).unwrap_or_default();

        let response = self.client.get("/messages/batches", &query).await?;
        let result: BatchList = response.json().await?;

        Ok(result)
    }

    /// Previews a batch without sending (dry run).
    ///
    /// # Arguments
    ///
    /// * `request` - The batch send request
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, SendBatchRequest, BatchMessageItem};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let preview = client.messages().preview_batch(SendBatchRequest {
    ///     messages: vec![
    ///         BatchMessageItem {
    ///             to: "+15551234567".to_string(),
    ///             text: "Hello Alice!".to_string(),
    ///             metadata: None,
    ///         },
    ///         BatchMessageItem {
    ///             to: "+15559876543".to_string(),
    ///             text: "Hello Bob!".to_string(),
    ///             metadata: None,
    ///         },
    ///     ],
    ///     from: None,
    ///     message_type: None,
    ///     metadata: None,
    /// }).await?;
    ///
    /// println!("Can send: {}", preview.can_send);
    /// println!("Credits needed: {}", preview.credits_needed);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn preview_batch(&self, request: SendBatchRequest) -> Result<BatchPreviewResponse> {
        if request.messages.is_empty() {
            return Err(Error::Validation {
                message: "Messages array is required".to_string(),
            });
        }

        // Validate each message
        for (i, msg) in request.messages.iter().enumerate() {
            validate_phone(&msg.to).map_err(|_| Error::Validation {
                message: format!("Invalid phone number at index {}", i),
            })?;
            validate_text(&msg.text).map_err(|_| Error::Validation {
                message: format!("Invalid message text at index {}", i),
            })?;
        }

        let response = self
            .client
            .post("/messages/batch/preview", &request)
            .await?;
        let result: BatchPreviewResponse = response.json().await?;

        Ok(result)
    }
}
