use regex::Regex;
use std::sync::OnceLock;

use crate::client::Sendly;
use crate::error::{Error, Result};
use crate::models::{
    BatchList, BatchMessageItem, BatchMessageResponse, CancelScheduledMessageResponse,
    ListBatchesOptions, ListMessagesOptions, ListScheduledMessagesOptions, Message, MessageList,
    ScheduleMessageRequest, ScheduledMessage, ScheduledMessageList, SendBatchRequest,
    SendMessageRequest,
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
    /// let message = client.messages().send(SendMessageRequest {
    ///     to: "+15551234567".to_string(),
    ///     text: "Hello from Sendly!".to_string(),
    /// }).await?;
    ///
    /// println!("Sent: {}", message.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send(&self, request: SendMessageRequest) -> Result<Message> {
        validate_phone(&request.to)?;
        validate_text(&request.text)?;

        let response = self.client.post("/messages", &request).await?;
        let message: Message = response.json().await?;

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
        })
        .await
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
    ///         },
    ///         BatchMessageItem {
    ///             to: "+15559876543".to_string(),
    ///             text: "Hello Bob!".to_string(),
    ///         },
    ///     ],
    ///     from: None,
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
}
