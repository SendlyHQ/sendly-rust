use crate::client::{path_id, Sendly};
use crate::error::{Error, Result};
use crate::models::{
    CreateDraftRequest, DraftListResponse, ListDraftsOptions, MessageDraft, RejectDraftRequest,
    UpdateDraftRequest,
};

/// Drafts resource for managing message drafts.
#[derive(Debug, Clone)]
pub struct DraftsResource<'a> {
    client: &'a Sendly,
}

impl<'a> DraftsResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Creates a new message draft.
    pub async fn create(&self, request: CreateDraftRequest) -> Result<MessageDraft> {
        if request.conversation_id.is_empty() {
            return Err(Error::validation("Conversation ID is required"));
        }
        if request.text.is_empty() {
            return Err(Error::validation("Draft text is required"));
        }

        let response = self.client.post("/drafts", &request).await?;
        let draft: MessageDraft = response.json().await?;

        Ok(draft)
    }

    /// Lists drafts with optional filtering.
    pub async fn list(&self, options: Option<ListDraftsOptions>) -> Result<DraftListResponse> {
        let query = options.map(|o| o.to_query_params()).unwrap_or_default();

        let response = self.client.get("/drafts", &query).await?;
        let result: DraftListResponse = response.json().await?;

        Ok(result)
    }

    /// Gets a draft by ID.
    pub async fn get(&self, id: &str) -> Result<MessageDraft> {
        if id.is_empty() {
            return Err(Error::validation("Draft ID is required"));
        }

        let encoded_id = path_id(id)?;
        let path = format!("/drafts/{}", encoded_id);
        let response = self.client.get(&path, &[]).await?;
        let draft: MessageDraft = response.json().await?;

        Ok(draft)
    }

    /// Updates a draft.
    pub async fn update(&self, id: &str, request: UpdateDraftRequest) -> Result<MessageDraft> {
        if id.is_empty() {
            return Err(Error::validation("Draft ID is required"));
        }

        let encoded_id = path_id(id)?;
        let path = format!("/drafts/{}", encoded_id);
        let response = self.client.patch(&path, &request).await?;
        let draft: MessageDraft = response.json().await?;

        Ok(draft)
    }

    /// Approves a pending draft and sends it to the conversation.
    ///
    /// Returns the approved draft, whose `message_id` is the message that was
    /// sent. If the send is refused with a 4xx error, such as 402
    /// insufficient credits, this returns that error and the draft goes back
    /// to pending.
    pub async fn approve(&self, id: &str) -> Result<MessageDraft> {
        if id.is_empty() {
            return Err(Error::validation("Draft ID is required"));
        }

        let encoded_id = path_id(id)?;
        let path = format!("/drafts/{}/approve", encoded_id);
        let response = self.client.post_empty(&path).await?;
        let draft: MessageDraft = response.json().await?;

        Ok(draft)
    }

    /// Rejects a draft with an optional reason.
    pub async fn reject(&self, id: &str, reason: Option<String>) -> Result<MessageDraft> {
        if id.is_empty() {
            return Err(Error::validation("Draft ID is required"));
        }

        let encoded_id = path_id(id)?;
        let path = format!("/drafts/{}/reject", encoded_id);
        let body = RejectDraftRequest { reason };
        let response = self.client.post(&path, &body).await?;
        let draft: MessageDraft = response.json().await?;

        Ok(draft)
    }
}
