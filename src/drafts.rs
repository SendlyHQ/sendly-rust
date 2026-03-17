use crate::client::Sendly;
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
            return Err(Error::Validation {
                message: "Conversation ID is required".to_string(),
            });
        }
        if request.text.is_empty() {
            return Err(Error::Validation {
                message: "Draft text is required".to_string(),
            });
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
            return Err(Error::Validation {
                message: "Draft ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let path = format!("/drafts/{}", encoded_id);
        let response = self.client.get(&path, &[]).await?;
        let draft: MessageDraft = response.json().await?;

        Ok(draft)
    }

    /// Updates a draft.
    pub async fn update(&self, id: &str, request: UpdateDraftRequest) -> Result<MessageDraft> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Draft ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let path = format!("/drafts/{}", encoded_id);
        let response = self.client.patch(&path, &request).await?;
        let draft: MessageDraft = response.json().await?;

        Ok(draft)
    }

    /// Approves a draft for sending.
    pub async fn approve(&self, id: &str) -> Result<MessageDraft> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Draft ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let path = format!("/drafts/{}/approve", encoded_id);
        let response = self.client.post(&path, &()).await?;
        let draft: MessageDraft = response.json().await?;

        Ok(draft)
    }

    /// Rejects a draft with an optional reason.
    pub async fn reject(&self, id: &str, reason: Option<String>) -> Result<MessageDraft> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Draft ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let path = format!("/drafts/{}/reject", encoded_id);
        let body = RejectDraftRequest { reason };
        let response = self.client.post(&path, &body).await?;
        let draft: MessageDraft = response.json().await?;

        Ok(draft)
    }
}
