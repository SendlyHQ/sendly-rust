use serde::{Deserialize, Serialize};

use crate::client::{path_id, Sendly};
use crate::error::Result;
use crate::models::BatchMessageResponse;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum CampaignStatus {
    Draft,
    Scheduled,
    Sending,
    /// The campaign finished sending.
    Completed,
    #[deprecated(
        note = "The API never gives a campaign this status; a sent campaign is `Completed`."
    )]
    Sent,
    #[deprecated(note = "The API never gives a campaign this status.")]
    Paused,
    Cancelled,
    Failed,
}

impl std::fmt::Display for CampaignStatus {
    #[allow(deprecated)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CampaignStatus::Draft => write!(f, "draft"),
            CampaignStatus::Scheduled => write!(f, "scheduled"),
            CampaignStatus::Sending => write!(f, "sending"),
            CampaignStatus::Completed => write!(f, "completed"),
            CampaignStatus::Sent => write!(f, "sent"),
            CampaignStatus::Paused => write!(f, "paused"),
            CampaignStatus::Cancelled => write!(f, "cancelled"),
            CampaignStatus::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Deserialize)]
struct CampaignWire {
    id: String,
    name: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default, rename = "messageText")]
    message_text: Option<String>,
    #[serde(default)]
    template_id: Option<String>,
    #[serde(default, rename = "templateId")]
    template_id_camel: Option<String>,
    #[serde(default)]
    contact_list_ids: Option<Vec<String>>,
    #[serde(default, rename = "contactListIds")]
    contact_list_ids_camel: Option<Vec<String>>,
    #[serde(default, rename = "targetListId")]
    target_list_id: Option<String>,
    status: String,
    #[serde(default)]
    recipient_count: Option<i32>,
    #[serde(default, rename = "recipientCount")]
    recipient_count_camel: Option<i32>,
    #[serde(default, rename = "totalRecipients")]
    total_recipients: Option<i32>,
    #[serde(default)]
    sent_count: Option<i32>,
    #[serde(default, rename = "sentCount")]
    sent_count_camel: Option<i32>,
    #[serde(default)]
    delivered_count: Option<i32>,
    #[serde(default, rename = "deliveredCount")]
    delivered_count_camel: Option<i32>,
    #[serde(default)]
    failed_count: Option<i32>,
    #[serde(default, rename = "failedCount")]
    failed_count_camel: Option<i32>,
    #[serde(default)]
    estimated_credits: Option<f64>,
    #[serde(default, rename = "estimatedCredits")]
    estimated_credits_camel: Option<f64>,
    #[serde(default)]
    credits_used: Option<f64>,
    #[serde(default, rename = "creditsUsed")]
    credits_used_camel: Option<f64>,
    #[serde(default)]
    scheduled_at: Option<String>,
    #[serde(default, rename = "scheduledAt")]
    scheduled_at_camel: Option<String>,
    #[serde(default)]
    timezone: Option<String>,
    #[serde(default)]
    started_at: Option<String>,
    #[serde(default, rename = "startedAt")]
    started_at_camel: Option<String>,
    #[serde(default, rename = "sentAt")]
    sent_at: Option<String>,
    #[serde(default)]
    completed_at: Option<String>,
    #[serde(default, rename = "completedAt")]
    completed_at_camel: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default, rename = "createdAt")]
    created_at_camel: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
    #[serde(default, rename = "updatedAt")]
    updated_at_camel: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "CampaignWire")]
pub struct Campaign {
    pub id: String,
    pub name: String,
    pub text: String,
    pub template_id: Option<String>,
    /// The contact list the campaign targets; a campaign targets one list.
    pub contact_list_ids: Vec<String>,
    pub status: String,
    /// Recipients the campaign targets.
    pub recipient_count: i32,
    pub sent_count: i32,
    pub delivered_count: i32,
    pub failed_count: i32,
    pub estimated_credits: Option<f64>,
    pub credits_used: Option<f64>,
    pub scheduled_at: Option<String>,
    pub timezone: Option<String>,
    /// When the campaign started sending.
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<CampaignWire> for Campaign {
    fn from(wire: CampaignWire) -> Self {
        Self {
            id: wire.id,
            name: wire.name,
            text: wire.text.or(wire.message_text).unwrap_or_default(),
            template_id: wire.template_id.or(wire.template_id_camel),
            contact_list_ids: wire
                .contact_list_ids
                .or(wire.contact_list_ids_camel)
                .or_else(|| wire.target_list_id.map(|id| vec![id]))
                .unwrap_or_default(),
            status: wire.status,
            recipient_count: wire
                .recipient_count
                .or(wire.recipient_count_camel)
                .or(wire.total_recipients)
                .unwrap_or_default(),
            sent_count: wire
                .sent_count
                .or(wire.sent_count_camel)
                .unwrap_or_default(),
            delivered_count: wire
                .delivered_count
                .or(wire.delivered_count_camel)
                .unwrap_or_default(),
            failed_count: wire
                .failed_count
                .or(wire.failed_count_camel)
                .unwrap_or_default(),
            estimated_credits: wire.estimated_credits.or(wire.estimated_credits_camel),
            credits_used: wire.credits_used.or(wire.credits_used_camel),
            scheduled_at: wire.scheduled_at.or(wire.scheduled_at_camel),
            timezone: wire.timezone,
            started_at: wire.started_at.or(wire.started_at_camel).or(wire.sent_at),
            completed_at: wire.completed_at.or(wire.completed_at_camel),
            created_at: wire.created_at.or(wire.created_at_camel),
            updated_at: wire.updated_at.or(wire.updated_at_camel),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CampaignListResponse {
    pub campaigns: Vec<Campaign>,
    #[serde(default)]
    pub total: i32,
    #[serde(default)]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CampaignPreview {
    #[serde(alias = "recipientCount")]
    pub recipient_count: i32,
    #[serde(alias = "estimatedCredits")]
    pub estimated_credits: f64,
    #[serde(default, alias = "estimatedCost")]
    pub estimated_cost: f64,
    #[serde(default, alias = "blockedCount")]
    pub blocked_count: Option<i32>,
    #[serde(default, alias = "sendableCount")]
    pub sendable_count: Option<i32>,
    #[serde(default)]
    pub warnings: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub text: String,
    #[serde(rename = "contact_list_ids")]
    pub contact_list_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "template_id")]
    pub template_id: Option<String>,
}

impl CreateCampaignRequest {
    pub fn new(
        name: impl Into<String>,
        text: impl Into<String>,
        contact_list_ids: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            text: text.into(),
            contact_list_ids,
            template_id: None,
        }
    }

    pub fn template_id(mut self, template_id: impl Into<String>) -> Self {
        self.template_id = Some(template_id.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateCampaignRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "contact_list_ids")]
    pub contact_list_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "template_id")]
    pub template_id: Option<String>,
}

impl UpdateCampaignRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn contact_list_ids(mut self, ids: Vec<String>) -> Self {
        self.contact_list_ids = Some(ids);
        self
    }

    pub fn template_id(mut self, template_id: impl Into<String>) -> Self {
        self.template_id = Some(template_id.into());
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListCampaignsOptions {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub status: Option<CampaignStatus>,
}

impl ListCampaignsOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.min(100));
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn status(mut self, status: CampaignStatus) -> Self {
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

#[derive(Debug, Clone, Serialize)]
pub struct ScheduleCampaignRequest {
    #[serde(rename = "scheduledAt")]
    pub scheduled_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

impl ScheduleCampaignRequest {
    pub fn new(scheduled_at: impl Into<String>) -> Self {
        Self {
            scheduled_at: scheduled_at.into(),
            timezone: None,
        }
    }

    pub fn timezone(mut self, timezone: impl Into<String>) -> Self {
        self.timezone = Some(timezone.into());
        self
    }
}

pub struct CampaignsResource<'a> {
    client: &'a Sendly,
}

impl<'a> CampaignsResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    pub async fn list(&self, options: ListCampaignsOptions) -> Result<CampaignListResponse> {
        let params = options.to_query_params();
        let response = self.client.get("/campaigns", &params).await?;
        Ok(response.json().await?)
    }

    pub async fn get(&self, id: &str) -> Result<Campaign> {
        let response = self
            .client
            .get(&format!("/campaigns/{}", path_id(id)?), &[])
            .await?;
        Ok(response.json().await?)
    }

    pub async fn create(&self, request: CreateCampaignRequest) -> Result<Campaign> {
        let response = self.client.post("/campaigns", &request).await?;
        Ok(response.json().await?)
    }

    pub async fn update(&self, id: &str, request: UpdateCampaignRequest) -> Result<Campaign> {
        let response = self
            .client
            .patch(&format!("/campaigns/{}", path_id(id)?), &request)
            .await?;
        Ok(response.json().await?)
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client
            .delete(&format!("/campaigns/{}", path_id(id)?))
            .await?;
        Ok(())
    }

    pub async fn preview(&self, id: &str) -> Result<CampaignPreview> {
        let response = self
            .client
            .get(&format!("/campaigns/{}/preview", path_id(id)?), &[])
            .await?;
        Ok(response.json().await?)
    }

    /// Sends a draft or scheduled campaign now, from the workspace's default
    /// sender, and returns the batch it created. Follow the batch with
    /// [`Messages::get_batch`](crate::Messages::get_batch).
    pub async fn send(&self, id: &str) -> Result<BatchMessageResponse> {
        let response = self
            .client
            .post_empty(&format!("/campaigns/{}/send", path_id(id)?))
            .await?;
        Ok(response.json().await?)
    }

    /// Sends a campaign now from `from`, a number of yours, and returns the
    /// batch it created. A number the workspace cannot send from answers 400
    /// `invalid_from_number`.
    pub async fn send_from(
        &self,
        id: &str,
        from: impl Into<String>,
    ) -> Result<BatchMessageResponse> {
        #[derive(Serialize)]
        struct SendCampaignRequest {
            from: String,
        }
        let response = self
            .client
            .post(
                &format!("/campaigns/{}/send", path_id(id)?),
                &SendCampaignRequest { from: from.into() },
            )
            .await?;
        Ok(response.json().await?)
    }

    pub async fn schedule(&self, id: &str, request: ScheduleCampaignRequest) -> Result<Campaign> {
        let response = self
            .client
            .post(&format!("/campaigns/{}/schedule", path_id(id)?), &request)
            .await?;
        Ok(response.json().await?)
    }

    pub async fn cancel(&self, id: &str) -> Result<Campaign> {
        let response = self
            .client
            .post_empty(&format!("/campaigns/{}/cancel", path_id(id)?))
            .await?;
        Ok(response.json().await?)
    }

    pub async fn clone(&self, id: &str) -> Result<Campaign> {
        let response = self
            .client
            .post_empty(&format!("/campaigns/{}/clone", path_id(id)?))
            .await?;
        Ok(response.json().await?)
    }
}
