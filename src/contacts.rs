use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::client::Sendly;
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    #[serde(alias = "phoneNumber")]
    pub phone_number: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    #[serde(default, alias = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactList {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, alias = "contactCount")]
    pub contact_count: i32,
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<String>,
    #[serde(default, alias = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContactListResponse {
    pub contacts: Vec<Contact>,
    #[serde(default)]
    pub total: i32,
    #[serde(default)]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContactListsResponse {
    pub lists: Vec<ContactList>,
    #[serde(default)]
    pub total: i32,
    #[serde(default)]
    pub limit: i32,
    #[serde(default)]
    pub offset: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateContactRequest {
    #[serde(rename = "phone_number")]
    pub phone_number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl CreateContactRequest {
    pub fn new(phone_number: impl Into<String>) -> Self {
        Self {
            phone_number: phone_number.into(),
            name: None,
            email: None,
            metadata: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateContactRequest {
    #[serde(skip_serializing_if = "Option::is_none", rename = "phone_number")]
    pub phone_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl UpdateContactRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn phone_number(mut self, phone_number: impl Into<String>) -> Self {
        self.phone_number = Some(phone_number.into());
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListContactsOptions {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub search: Option<String>,
    pub list_id: Option<String>,
}

impl ListContactsOptions {
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

    pub fn search(mut self, search: impl Into<String>) -> Self {
        self.search = Some(search.into());
        self
    }

    pub fn list_id(mut self, list_id: impl Into<String>) -> Self {
        self.list_id = Some(list_id.into());
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
        if let Some(ref search) = self.search {
            params.push(("search".to_string(), search.clone()));
        }
        if let Some(ref list_id) = self.list_id {
            params.push(("list_id".to_string(), list_id.clone()));
        }
        params
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateContactListRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CreateContactListRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
        }
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateContactListRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl UpdateContactListRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AddContactsRequest {
    #[serde(rename = "contact_ids")]
    pub contact_ids: Vec<String>,
}

pub struct ContactsResource<'a> {
    client: &'a Sendly,
}

impl<'a> ContactsResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    pub fn lists(&self) -> ContactListsResource<'a> {
        ContactListsResource::new(self.client)
    }

    pub async fn list(&self, options: ListContactsOptions) -> Result<ContactListResponse> {
        let params = options.to_query_params();
        let response = self.client.get("/contacts", &params).await?;
        Ok(response.json().await?)
    }

    pub async fn get(&self, id: &str) -> Result<Contact> {
        let response = self.client.get(&format!("/contacts/{}", id), &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn create(&self, request: CreateContactRequest) -> Result<Contact> {
        let response = self.client.post("/contacts", &request).await?;
        Ok(response.json().await?)
    }

    pub async fn update(&self, id: &str, request: UpdateContactRequest) -> Result<Contact> {
        let response = self
            .client
            .patch(&format!("/contacts/{}", id), &request)
            .await?;
        Ok(response.json().await?)
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("/contacts/{}", id)).await?;
        Ok(())
    }
}

pub struct ContactListsResource<'a> {
    client: &'a Sendly,
}

impl<'a> ContactListsResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    pub async fn list(&self) -> Result<ContactListsResponse> {
        let response = self.client.get("/contact-lists", &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn get(&self, id: &str) -> Result<ContactList> {
        let response = self
            .client
            .get(&format!("/contact-lists/{}", id), &[])
            .await?;
        Ok(response.json().await?)
    }

    pub async fn create(&self, request: CreateContactListRequest) -> Result<ContactList> {
        let response = self.client.post("/contact-lists", &request).await?;
        Ok(response.json().await?)
    }

    pub async fn update(&self, id: &str, request: UpdateContactListRequest) -> Result<ContactList> {
        let response = self
            .client
            .patch(&format!("/contact-lists/{}", id), &request)
            .await?;
        Ok(response.json().await?)
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client
            .delete(&format!("/contact-lists/{}", id))
            .await?;
        Ok(())
    }

    pub async fn add_contacts(&self, list_id: &str, contact_ids: Vec<String>) -> Result<()> {
        let request = AddContactsRequest { contact_ids };
        self.client
            .post(&format!("/contact-lists/{}/contacts", list_id), &request)
            .await?;
        Ok(())
    }

    pub async fn remove_contact(&self, list_id: &str, contact_id: &str) -> Result<()> {
        self.client
            .delete(&format!(
                "/contact-lists/{}/contacts/{}",
                list_id, contact_id
            ))
            .await?;
        Ok(())
    }
}
