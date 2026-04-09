use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::client::Sendly;
use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerificationStatus {
    Pending,
    Verified,
    Expired,
    Failed,
}

impl std::fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerificationStatus::Pending => write!(f, "pending"),
            VerificationStatus::Verified => write!(f, "verified"),
            VerificationStatus::Expired => write!(f, "expired"),
            VerificationStatus::Failed => write!(f, "failed"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeliveryStatus {
    Queued,
    Sent,
    Delivered,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub id: String,
    pub status: VerificationStatus,
    pub phone: String,
    pub delivery_status: DeliveryStatus,
    #[serde(default)]
    pub attempts: i32,
    #[serde(default = "default_max_attempts")]
    pub max_attempts: i32,
    pub expires_at: String,
    #[serde(default)]
    pub verified_at: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub sandbox: bool,
    #[serde(default)]
    pub app_name: Option<String>,
    #[serde(default)]
    pub template_id: Option<String>,
    #[serde(default)]
    pub profile_id: Option<String>,
}

fn default_max_attempts() -> i32 {
    3
}

impl Verification {
    pub fn is_pending(&self) -> bool {
        self.status == VerificationStatus::Pending
    }

    pub fn is_verified(&self) -> bool {
        self.status == VerificationStatus::Verified
    }

    pub fn is_expired(&self) -> bool {
        self.status == VerificationStatus::Expired
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SendVerificationRequest {
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_length: Option<i32>,
}

impl SendVerificationRequest {
    pub fn new(to: impl Into<String>) -> Self {
        Self {
            to: to.into(),
            template_id: None,
            profile_id: None,
            app_name: None,
            timeout_secs: None,
            code_length: None,
        }
    }

    pub fn template_id(mut self, id: impl Into<String>) -> Self {
        self.template_id = Some(id.into());
        self
    }

    pub fn profile_id(mut self, id: impl Into<String>) -> Self {
        self.profile_id = Some(id.into());
        self
    }

    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    pub fn timeout_secs(mut self, secs: i32) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    pub fn code_length(mut self, len: i32) -> Self {
        self.code_length = Some(len);
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendVerificationResponse {
    pub id: String,
    pub status: VerificationStatus,
    pub phone: String,
    pub expires_at: String,
    #[serde(default)]
    pub sandbox: bool,
    #[serde(default)]
    pub sandbox_code: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckVerificationRequest {
    pub code: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckVerificationResponse {
    pub id: String,
    pub status: VerificationStatus,
    pub phone: String,
    #[serde(default)]
    pub verified_at: Option<String>,
    #[serde(default)]
    pub remaining_attempts: Option<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct ListVerificationsOptions {
    pub limit: Option<u32>,
    pub status: Option<VerificationStatus>,
}

impl ListVerificationsOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.min(100));
        self
    }

    pub fn status(mut self, status: VerificationStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub(crate) fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(ref status) = self.status {
            params.push(("status".to_string(), status.to_string()));
        }
        params
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerificationList {
    pub verifications: Vec<Verification>,
    #[serde(default)]
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    #[serde(default)]
    pub limit: i32,
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Pending,
    PhoneSubmitted,
    CodeSent,
    Verified,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateSessionRequest {
    pub success_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancel_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl CreateSessionRequest {
    pub fn new(success_url: impl Into<String>) -> Self {
        Self {
            success_url: success_url.into(),
            cancel_url: None,
            brand_name: None,
            brand_color: None,
            metadata: None,
        }
    }

    pub fn cancel_url(mut self, url: impl Into<String>) -> Self {
        self.cancel_url = Some(url.into());
        self
    }

    pub fn brand_name(mut self, name: impl Into<String>) -> Self {
        self.brand_name = Some(name.into());
        self
    }

    pub fn brand_color(mut self, color: impl Into<String>) -> Self {
        self.brand_color = Some(color.into());
        self
    }

    pub fn metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerifySession {
    pub id: String,
    pub url: String,
    pub status: String,
    pub success_url: String,
    #[serde(default)]
    pub cancel_url: Option<String>,
    #[serde(default)]
    pub brand_name: Option<String>,
    #[serde(default)]
    pub brand_color: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub verification_id: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub expires_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidateSessionRequest {
    pub token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ValidateSessionResponse {
    pub valid: bool,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub verified_at: Option<String>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

pub struct SessionsResource<'a> {
    client: &'a Sendly,
}

impl<'a> SessionsResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    pub async fn create(&self, request: CreateSessionRequest) -> Result<VerifySession> {
        let response = self.client.post("/verify/sessions", &request).await?;
        Ok(response.json().await?)
    }

    pub async fn validate(&self, token: &str) -> Result<ValidateSessionResponse> {
        let request = ValidateSessionRequest {
            token: token.to_string(),
        };
        let response = self
            .client
            .post("/verify/sessions/validate", &request)
            .await?;
        Ok(response.json().await?)
    }
}

pub struct VerifyResource<'a> {
    client: &'a Sendly,
}

impl<'a> VerifyResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    pub fn sessions(&self) -> SessionsResource<'a> {
        SessionsResource::new(self.client)
    }

    pub async fn send(&self, request: SendVerificationRequest) -> Result<SendVerificationResponse> {
        let response = self.client.post("/verify", &request).await?;
        Ok(response.json().await?)
    }

    pub async fn resend(&self, id: &str) -> Result<SendVerificationResponse> {
        let response = self
            .client
            .post(&format!("/verify/{}/resend", id), &())
            .await?;
        Ok(response.json().await?)
    }

    pub async fn check(&self, id: &str, code: &str) -> Result<CheckVerificationResponse> {
        let request = CheckVerificationRequest {
            code: code.to_string(),
        };
        let response = self
            .client
            .post(&format!("/verify/{}/check", id), &request)
            .await?;
        Ok(response.json().await?)
    }

    pub async fn get(&self, id: &str) -> Result<Verification> {
        let response = self.client.get(&format!("/verify/{}", id), &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn list(&self, options: ListVerificationsOptions) -> Result<VerificationList> {
        let params = options.to_query_params();
        let response = self.client.get("/verify", &params).await?;
        Ok(response.json().await?)
    }
}
