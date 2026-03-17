use crate::client::Sendly;
use crate::error::{Error, Result};
use crate::models::{
    AddLabelsRequest, Conversation, ConversationListResponse, ConversationWithMessages,
    GetConversationOptions, ListConversationsOptions, Message, ReplyToConversationRequest,
    UpdateConversationRequest,
};

/// Conversations resource for managing SMS conversation threads.
#[derive(Debug, Clone)]
pub struct ConversationsResource<'a> {
    client: &'a Sendly,
}

impl<'a> ConversationsResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Lists conversations.
    ///
    /// # Arguments
    ///
    /// * `options` - Optional query options
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, ListConversationsOptions, ConversationStatus};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// // List all
    /// let conversations = client.conversations().list(None).await?;
    ///
    /// // With options
    /// let conversations = client.conversations().list(Some(
    ///     ListConversationsOptions::new()
    ///         .limit(50)
    ///         .status(ConversationStatus::Active)
    /// )).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(
        &self,
        options: Option<ListConversationsOptions>,
    ) -> Result<ConversationListResponse> {
        let query = options.map(|o| o.to_query_params()).unwrap_or_default();

        let response = self.client.get("/conversations", &query).await?;
        let result: ConversationListResponse = response.json().await?;

        Ok(result)
    }

    /// Gets a conversation by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - Conversation ID
    /// * `options` - Optional query options (include messages, etc.)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, GetConversationOptions};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let conversation = client.conversations().get("conv_abc123", None).await?;
    ///
    /// // With messages
    /// let conversation = client.conversations().get("conv_abc123", Some(
    ///     GetConversationOptions::new()
    ///         .include_messages(true)
    ///         .message_limit(50)
    /// )).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(
        &self,
        id: &str,
        options: Option<GetConversationOptions>,
    ) -> Result<ConversationWithMessages> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Conversation ID is required".to_string(),
            });
        }

        let query = options.map(|o| o.to_query_params()).unwrap_or_default();

        let encoded_id = urlencoding::encode(id);
        let path = format!("/conversations/{}", encoded_id);
        let response = self.client.get(&path, &query).await?;
        let result: ConversationWithMessages = response.json().await?;

        Ok(result)
    }

    /// Replies to a conversation.
    ///
    /// # Arguments
    ///
    /// * `id` - Conversation ID
    /// * `request` - The reply request
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, ReplyToConversationRequest};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let message = client.conversations().reply("conv_abc123", ReplyToConversationRequest {
    ///     text: "Thanks for reaching out!".to_string(),
    ///     message_type: None,
    ///     metadata: None,
    ///     media_urls: None,
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reply(
        &self,
        id: &str,
        request: ReplyToConversationRequest,
    ) -> Result<Message> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Conversation ID is required".to_string(),
            });
        }
        if request.text.is_empty() {
            return Err(Error::Validation {
                message: "Message text is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let path = format!("/conversations/{}/messages", encoded_id);
        let response = self.client.post(&path, &request).await?;
        let message: Message = response.json().await?;

        Ok(message)
    }

    /// Updates a conversation's metadata or tags.
    ///
    /// # Arguments
    ///
    /// * `id` - Conversation ID
    /// * `request` - The update request
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, UpdateConversationRequest};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let conversation = client.conversations().update("conv_abc123", UpdateConversationRequest {
    ///     metadata: None,
    ///     tags: Some(vec!["vip".to_string(), "support".to_string()]),
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(
        &self,
        id: &str,
        request: UpdateConversationRequest,
    ) -> Result<Conversation> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Conversation ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let path = format!("/conversations/{}", encoded_id);
        let response = self.client.patch(&path, &request).await?;
        let result: Conversation = response.json().await?;

        Ok(result)
    }

    /// Closes a conversation.
    ///
    /// # Arguments
    ///
    /// * `id` - Conversation ID
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let conversation = client.conversations().close("conv_abc123").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn close(&self, id: &str) -> Result<Conversation> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Conversation ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let path = format!("/conversations/{}/close", encoded_id);
        let response = self.client.post(&path, &()).await?;
        let result: Conversation = response.json().await?;

        Ok(result)
    }

    /// Reopens a closed conversation.
    ///
    /// # Arguments
    ///
    /// * `id` - Conversation ID
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let conversation = client.conversations().reopen("conv_abc123").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reopen(&self, id: &str) -> Result<Conversation> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Conversation ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let path = format!("/conversations/{}/reopen", encoded_id);
        let response = self.client.post(&path, &()).await?;
        let result: Conversation = response.json().await?;

        Ok(result)
    }

    /// Marks a conversation as read.
    ///
    /// # Arguments
    ///
    /// * `id` - Conversation ID
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let conversation = client.conversations().mark_read("conv_abc123").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn mark_read(&self, id: &str) -> Result<Conversation> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Conversation ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let path = format!("/conversations/{}/mark-read", encoded_id);
        let response = self.client.post(&path, &()).await?;
        let result: Conversation = response.json().await?;

        Ok(result)
    }

    /// Adds labels to a conversation.
    ///
    /// # Arguments
    ///
    /// * `id` - Conversation ID
    /// * `label_ids` - Label IDs to add
    pub async fn add_labels(&self, id: &str, label_ids: Vec<String>) -> Result<Conversation> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Conversation ID is required".to_string(),
            });
        }
        if label_ids.is_empty() {
            return Err(Error::Validation {
                message: "At least one label ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let path = format!("/conversations/{}/labels", encoded_id);
        let body = AddLabelsRequest { label_ids };
        let response = self.client.post(&path, &body).await?;
        let result: Conversation = response.json().await?;

        Ok(result)
    }

    /// Removes a label from a conversation.
    ///
    /// # Arguments
    ///
    /// * `id` - Conversation ID
    /// * `label_id` - Label ID to remove
    pub async fn remove_label(&self, id: &str, label_id: &str) -> Result<Conversation> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Conversation ID is required".to_string(),
            });
        }
        if label_id.is_empty() {
            return Err(Error::Validation {
                message: "Label ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let encoded_label_id = urlencoding::encode(label_id);
        let path = format!("/conversations/{}/labels/{}", encoded_id, encoded_label_id);
        let response = self.client.delete(&path).await?;
        let result: Conversation = response.json().await?;

        Ok(result)
    }
}
