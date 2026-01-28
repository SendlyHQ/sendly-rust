use serde::{Deserialize, Serialize};

use crate::client::Sendly;
use crate::error::Result;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateType {
    Preset,
    Custom,
}

impl Default for TemplateType {
    fn default() -> Self {
        TemplateType::Custom
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub body: String,
    #[serde(default, alias = "type")]
    pub template_type: TemplateType,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub variables: Vec<String>,
    #[serde(default, alias = "isDefault")]
    pub is_default: bool,
    #[serde(default, alias = "isPublished")]
    pub is_published: bool,
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    #[serde(default, alias = "updatedAt")]
    pub updated_at: Option<String>,
}

impl Template {
    pub fn is_preset(&self) -> bool {
        self.template_type == TemplateType::Preset
    }

    pub fn is_custom(&self) -> bool {
        self.template_type == TemplateType::Custom
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "isPublished")]
    pub is_published: Option<bool>,
}

impl CreateTemplateRequest {
    pub fn new(name: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            body: body.into(),
            locale: None,
            is_published: None,
        }
    }

    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = Some(locale.into());
        self
    }

    pub fn published(mut self, published: bool) -> Self {
        self.is_published = Some(published);
        self
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateTemplateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "isPublished")]
    pub is_published: Option<bool>,
}

impl UpdateTemplateRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = Some(locale.into());
        self
    }

    pub fn published(mut self, published: bool) -> Self {
        self.is_published = Some(published);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListTemplatesOptions {
    pub limit: Option<u32>,
    pub template_type: Option<TemplateType>,
    pub locale: Option<String>,
}

impl ListTemplatesOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.min(100));
        self
    }

    pub fn template_type(mut self, t: TemplateType) -> Self {
        self.template_type = Some(t);
        self
    }

    pub fn locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = Some(locale.into());
        self
    }

    pub(crate) fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(ref t) = self.template_type {
            let type_str = match t {
                TemplateType::Preset => "preset",
                TemplateType::Custom => "custom",
            };
            params.push(("type".to_string(), type_str.to_string()));
        }
        if let Some(ref locale) = self.locale {
            params.push(("locale".to_string(), locale.clone()));
        }
        params
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateList {
    pub templates: Vec<Template>,
    #[serde(default)]
    pub pagination: Option<TemplatePagination>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplatePagination {
    #[serde(default)]
    pub limit: i32,
    #[serde(default, alias = "hasMore")]
    pub has_more: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteTemplateResponse {
    pub success: bool,
    #[serde(default)]
    pub message: Option<String>,
}

pub struct TemplatesResource<'a> {
    client: &'a Sendly,
}

impl<'a> TemplatesResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    pub async fn list(&self, options: ListTemplatesOptions) -> Result<TemplateList> {
        let params = options.to_query_params();
        let response = self.client.get("/verify/templates", &params).await?;
        Ok(response.json().await?)
    }

    pub async fn get(&self, id: &str) -> Result<Template> {
        let response = self
            .client
            .get(&format!("/verify/templates/{}", id), &[])
            .await?;
        Ok(response.json().await?)
    }

    pub async fn create(&self, request: CreateTemplateRequest) -> Result<Template> {
        let response = self.client.post("/verify/templates", &request).await?;
        Ok(response.json().await?)
    }

    pub async fn update(&self, id: &str, request: UpdateTemplateRequest) -> Result<Template> {
        let response = self
            .client
            .patch(&format!("/verify/templates/{}", id), &request)
            .await?;
        Ok(response.json().await?)
    }

    pub async fn delete(&self, id: &str) -> Result<DeleteTemplateResponse> {
        let response = self
            .client
            .delete(&format!("/verify/templates/{}", id))
            .await?;
        Ok(response.json().await?)
    }

    pub async fn publish(&self, id: &str) -> Result<Template> {
        let response = self
            .client
            .post(&format!("/verify/templates/{}/publish", id), &())
            .await?;
        Ok(response.json().await?)
    }

    pub async fn unpublish(&self, id: &str) -> Result<Template> {
        let response = self
            .client
            .post(&format!("/verify/templates/{}/unpublish", id), &())
            .await?;
        Ok(response.json().await?)
    }

    pub async fn clone(&self, id: &str) -> Result<Template> {
        let response = self
            .client
            .post(&format!("/templates/{}/clone", id), &())
            .await?;
        Ok(response.json().await?)
    }

    pub async fn clone_with_name(&self, id: &str, name: impl Into<String>) -> Result<Template> {
        #[derive(serde::Serialize)]
        struct CloneRequest {
            name: String,
        }
        let request = CloneRequest { name: name.into() };
        let response = self
            .client
            .post(&format!("/templates/{}/clone", id), &request)
            .await?;
        Ok(response.json().await?)
    }
}
