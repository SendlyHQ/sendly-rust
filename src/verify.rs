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
    Pending,
    Sent,
    Delivered,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    Sms,
    Whatsapp,
    Email,
}

impl Default for Channel {
    fn default() -> Self {
        Channel::Sms
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verification {
    pub id: String,
    pub status: VerificationStatus,
    pub phone: String,
    #[serde(alias = "deliveryStatus")]
    pub delivery_status: DeliveryStatus,
    #[serde(default)]
    pub attempts: i32,
    #[serde(default = "default_max_attempts", alias = "maxAttempts")]
    pub max_attempts: i32,
    #[serde(default)]
    pub channel: Channel,
    #[serde(alias = "expiresAt")]
    pub expires_at: String,
    #[serde(default, alias = "verifiedAt")]
    pub verified_at: Option<String>,
    #[serde(alias = "createdAt")]
    pub created_at: String,
    #[serde(default)]
    pub sandbox: bool,
    #[serde(default, alias = "appName")]
    pub app_name: Option<String>,
    #[serde(default, alias = "templateId")]
    pub template_id: Option<String>,
    #[serde(default, alias = "profileId")]
    pub profile_id: Option<String>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
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
    pub phone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<Channel>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "codeLength")]
    pub code_length: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "expiresIn")]
    pub expires_in: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxAttempts")]
    pub max_attempts: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "templateId")]
    pub template_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "profileId")]
    pub profile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "appName")]
    pub app_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl SendVerificationRequest {
    pub fn new(phone: impl Into<String>) -> Self {
        Self {
            phone: phone.into(),
            channel: None,
            code_length: None,
            expires_in: None,
            max_attempts: None,
            template_id: None,
            profile_id: None,
            app_name: None,
            locale: None,
            metadata: None,
        }
    }

    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = Some(channel);
        self
    }

    pub fn code_length(mut self, len: i32) -> Self {
        self.code_length = Some(len);
        self
    }

    pub fn expires_in(mut self, secs: i32) -> Self {
        self.expires_in = Some(secs);
        self
    }

    pub fn app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = Some(name.into());
        self
    }

    pub fn template_id(mut self, id: impl Into<String>) -> Self {
        self.template_id = Some(id.into());
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendVerificationResponse {
    pub verification: Verification,
    #[serde(default)]
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckVerificationRequest {
    pub code: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CheckVerificationResponse {
    pub valid: bool,
    pub status: VerificationStatus,
    #[serde(default)]
    pub verification: Option<Verification>,
}

#[derive(Debug, Clone, Default)]
pub struct ListVerificationsOptions {
    pub limit: Option<u32>,
    pub status: Option<VerificationStatus>,
    pub phone: Option<String>,
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

    pub fn phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
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
        if let Some(ref phone) = self.phone {
            params.push(("phone".to_string(), phone.clone()));
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
    #[serde(default, alias = "hasMore")]
    pub has_more: bool,
}

pub struct VerifyResource<'a> {
    client: &'a Sendly,
}

impl<'a> VerifyResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
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
