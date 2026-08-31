use serde::{Deserialize, Serialize};

use crate::client::Sendly;
use crate::error::{Error, Result};

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
pub struct TemplateVariable {
    pub key: String,
    #[serde(default, rename = "type")]
    pub variable_type: Option<String>,
    #[serde(default, deserialize_with = "deserialize_fallback")]
    pub fallback: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum FallbackWire {
    Text(String),
    Number(serde_json::Number),
}

fn deserialize_fallback<'de, D>(deserializer: D) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(
        Option::<FallbackWire>::deserialize(deserializer)?.map(|value| match value {
            FallbackWire::Text(text) => text,
            FallbackWire::Number(number) => number.to_string(),
        }),
    )
}

#[derive(Deserialize)]
#[serde(untagged)]
enum VariableWire {
    Key(String),
    Spec(TemplateVariable),
}

fn deserialize_variables<'de, D>(
    deserializer: D,
) -> std::result::Result<Vec<TemplateVariable>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<Vec<VariableWire>>::deserialize(deserializer)?
        .unwrap_or_default()
        .into_iter()
        .map(|entry| match entry {
            VariableWire::Key(key) => TemplateVariable {
                key,
                variable_type: None,
                fallback: None,
            },
            VariableWire::Spec(spec) => spec,
        })
        .collect())
}

#[derive(Debug, Clone, Deserialize)]
struct TemplateWire {
    id: String,
    name: String,
    #[serde(alias = "body")]
    text: String,
    #[serde(default, deserialize_with = "deserialize_variables")]
    variables: Vec<TemplateVariable>,
    #[serde(default, alias = "type")]
    template_type: TemplateType,
    #[serde(default, alias = "isPreset")]
    is_preset: bool,
    #[serde(default, alias = "presetSlug")]
    preset_slug: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    version: i32,
    #[serde(default, alias = "publishedAt")]
    published_at: Option<String>,
    #[serde(default)]
    locale: Option<String>,
    #[serde(default, alias = "isDefault")]
    is_default: bool,
    #[serde(default, alias = "isPublished")]
    is_published: Option<bool>,
    #[serde(default, alias = "createdAt")]
    created_at: Option<String>,
    #[serde(default, alias = "updatedAt")]
    updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "TemplateWire")]
pub struct Template {
    pub id: String,
    pub name: String,
    pub text: String,
    #[serde(rename = "variables")]
    pub variable_specs: Vec<TemplateVariable>,
    pub is_preset: bool,
    pub preset_slug: Option<String>,
    pub status: Option<String>,
    pub version: i32,
    pub published_at: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,

    #[deprecated(note = "Renamed to `text` to match the API; this field carries the same value.")]
    #[serde(skip)]
    pub body: String,
    #[deprecated(note = "Use the `is_preset` field or `is_custom()`.")]
    #[serde(skip)]
    pub template_type: TemplateType,
    #[deprecated(
        note = "The templates API no longer returns a locale; this stays `None` unless the payload being decoded still carries one. Filter by locale with `ListTemplatesOptions::locale`."
    )]
    #[serde(skip)]
    pub locale: Option<String>,
    #[deprecated(note = "Use `variable_specs`, which also carries each variable's type and fallback; this field holds the keys only.")]
    #[serde(skip)]
    pub variables: Vec<String>,
    #[deprecated(
        note = "The templates API no longer returns a default flag; this stays `false` unless the payload being decoded still carries one. Use the `is_preset` field to identify built-in templates."
    )]
    #[serde(skip)]
    pub is_default: bool,
    #[deprecated(note = "Use the `status` field or the `is_published()` method.")]
    #[serde(skip)]
    pub is_published: bool,
}

impl From<TemplateWire> for Template {
    #[allow(deprecated)]
    fn from(wire: TemplateWire) -> Self {
        let is_preset = wire.is_preset || wire.template_type == TemplateType::Preset;
        let template_type = if is_preset {
            TemplateType::Preset
        } else {
            TemplateType::Custom
        };
        let is_published = wire
            .is_published
            .unwrap_or(wire.status.as_deref() == Some("published"));
        let variables = wire.variables.iter().map(|v| v.key.clone()).collect();

        Self {
            id: wire.id,
            name: wire.name,
            body: wire.text.clone(),
            text: wire.text,
            variable_specs: wire.variables,
            is_preset,
            preset_slug: wire.preset_slug,
            status: wire.status,
            version: wire.version,
            published_at: wire.published_at,
            created_at: wire.created_at,
            updated_at: wire.updated_at,
            template_type,
            locale: wire.locale,
            variables,
            is_default: wire.is_default,
            is_published,
        }
    }
}

impl Template {
    pub fn is_custom(&self) -> bool {
        !self.is_preset
    }

    #[allow(deprecated)]
    pub fn is_published(&self) -> bool {
        self.status.as_deref() == Some("published") || self.is_published
    }

    #[deprecated(note = "Use the `is_preset` field.")]
    pub fn is_preset(&self) -> bool {
        self.is_preset
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "isPublished")]
    pub is_published: Option<bool>,
    #[deprecated(
        note = "Renamed to `text` to match the API. `new()` keeps this in step with `text`; assigning to it directly has no effect on the request, so set `text` instead."
    )]
    #[serde(skip)]
    pub body: String,
}

impl CreateTemplateRequest {
    #[allow(deprecated)]
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            name: name.into(),
            body: text.clone(),
            text,
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
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "isPublished")]
    pub is_published: Option<bool>,
    #[deprecated(
        note = "Renamed to `text` to match the API. `text()` and `body()` both keep this in step with `text`; assigning to it directly has no effect on the request, so set `text` instead."
    )]
    #[serde(skip)]
    pub body: Option<String>,
}

impl UpdateTemplateRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    #[allow(deprecated)]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        let text = text.into();
        self.body = Some(text.clone());
        self.text = Some(text);
        self
    }

    #[deprecated(
        note = "Renamed to `text`; call `text()` instead. `body()` and `text()` set the same value, so whichever is called last wins."
    )]
    #[allow(deprecated)]
    pub fn body(mut self, body: impl Into<String>) -> Self {
        let body = body.into();
        self.body = Some(body.clone());
        self.text = Some(body);
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
        let response = self.client.get("/templates", &params).await?;
        Ok(response.json().await?)
    }

    pub async fn get(&self, id: &str) -> Result<Template> {
        let response = self.client.get(&format!("/templates/{}", id), &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn create(&self, request: CreateTemplateRequest) -> Result<Template> {
        let response = self.client.post("/templates", &request).await?;
        Ok(response.json().await?)
    }

    pub async fn update(&self, id: &str, request: UpdateTemplateRequest) -> Result<Template> {
        let response = self
            .client
            .patch(&format!("/templates/{}", id), &request)
            .await?;
        Ok(response.json().await?)
    }

    pub async fn delete(&self, id: &str) -> Result<DeleteTemplateResponse> {
        let response = self.client.delete(&format!("/templates/{}", id)).await?;
        let body = response.text().await?;
        if body.trim().is_empty() {
            return Ok(DeleteTemplateResponse {
                success: true,
                message: None,
            });
        }
        Ok(serde_json::from_str(&body)?)
    }

    pub async fn publish(&self, id: &str) -> Result<Template> {
        let response = self
            .client
            .post(&format!("/templates/{}/publish", id), &serde_json::json!({}))
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

    pub async fn generate(
        &self,
        request: crate::models::GenerateTemplateRequest,
    ) -> Result<crate::models::GeneratedTemplate> {
        if request.description.is_empty() {
            return Err(Error::Validation {
                message: "Description is required".to_string(),
            });
        }

        let response = self.client.post("/templates/generate", &request).await?;
        Ok(response.json().await?)
    }
}
