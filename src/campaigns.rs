use serde::{Deserialize, Serialize};

use crate::client::Sendly;
use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CampaignStatus {
    Draft,
    Scheduled,
    Sending,
    Sent,
    Paused,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    pub id: String,
    pub name: String,
    pub text: String,
    #[serde(default, alias = "templateId")]
    pub template_id: Option<String>,
    #[serde(default, alias = "contactListIds")]
    pub contact_list_ids: Vec<String>,
    pub status: String,
    #[serde(default, alias = "recipientCount")]
    pub recipient_count: i32,
    #[serde(default, alias = "sentCount")]
    pub sent_count: i32,
    #[serde(default, alias = "deliveredCount")]
    pub delivered_count: i32,
    #[serde(default, alias = "failedCount")]
    pub failed_count: i32,
    #[serde(default, alias = "estimatedCredits")]
    pub estimated_credits: Option<f64>,
    #[serde(default, alias = "creditsUsed")]
    pub credits_used: Option<f64>,
    #[serde(default, alias = "scheduledAt")]
    pub scheduled_at: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default, alias = "startedAt")]
    pub started_at: Option<String>,
    #[serde(default, alias = "completedAt")]
    pub completed_at: Option<String>,
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    #[serde(default, alias = "updatedAt")]
    pub updated_at: Option<String>,
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
            let status_str = match status {
                CampaignStatus::Draft => "draft",
                CampaignStatus::Scheduled => "scheduled",
                CampaignStatus::Sending => "sending",
                CampaignStatus::Sent => "sent",
                CampaignStatus::Paused => "paused",
                CampaignStatus::Cancelled => "cancelled",
                CampaignStatus::Failed => "failed",
            };
            params.push(("status".to_string(), status_str.to_string()));
        }
        params
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ScheduleCampaignRequest {
    #[serde(rename = "scheduled_at")]
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
        let response = self.client.get(&format!("/campaigns/{}", id), &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn create(&self, request: CreateCampaignRequest) -> Result<Campaign> {
        let response = self.client.post("/campaigns", &request).await?;
        Ok(response.json().await?)
    }

    pub async fn update(&self, id: &str, request: UpdateCampaignRequest) -> Result<Campaign> {
        let response = self
            .client
            .patch(&format!("/campaigns/{}", id), &request)
            .await?;
        Ok(response.json().await?)
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("/campaigns/{}", id)).await?;
        Ok(())
    }

    pub async fn preview(&self, id: &str) -> Result<CampaignPreview> {
        let response = self
            .client
            .get(&format!("/campaigns/{}/preview", id), &[])
            .await?;
        Ok(response.json().await?)
    }

    pub async fn send(&self, id: &str) -> Result<Campaign> {
        let response = self
            .client
            .post(&format!("/campaigns/{}/send", id), &())
            .await?;
        Ok(response.json().await?)
    }

    pub async fn schedule(&self, id: &str, request: ScheduleCampaignRequest) -> Result<Campaign> {
        let response = self
            .client
            .post(&format!("/campaigns/{}/schedule", id), &request)
            .await?;
        Ok(response.json().await?)
    }

    pub async fn cancel(&self, id: &str) -> Result<Campaign> {
        let response = self
            .client
            .post(&format!("/campaigns/{}/cancel", id), &())
            .await?;
        Ok(response.json().await?)
    }

    pub async fn clone(&self, id: &str) -> Result<Campaign> {
        let response = self
            .client
            .post(&format!("/campaigns/{}/clone", id), &())
            .await?;
        Ok(response.json().await?)
    }
}
