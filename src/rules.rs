use crate::client::{path_id, Sendly};
use crate::error::{Error, Result};
use crate::models::{CreateRuleRequest, Rule, RuleListResponse, UpdateRuleRequest};

/// Rules resource for managing auto-labeling rules.
#[derive(Debug, Clone)]
pub struct RulesResource<'a> {
    client: &'a Sendly,
}

impl<'a> RulesResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Lists all rules.
    pub async fn list(&self) -> Result<RuleListResponse> {
        let response = self.client.get("/rules", &[]).await?;
        let result: RuleListResponse = response.json().await?;

        Ok(result)
    }

    /// Creates a new rule.
    pub async fn create(&self, request: CreateRuleRequest) -> Result<Rule> {
        if request.name.is_empty() {
            return Err(Error::validation("Rule name is required"));
        }

        let response = self.client.post("/rules", &request).await?;
        let rule: Rule = response.json().await?;

        Ok(rule)
    }

    /// Updates a rule.
    pub async fn update(&self, id: &str, request: UpdateRuleRequest) -> Result<Rule> {
        if id.is_empty() {
            return Err(Error::validation("Rule ID is required"));
        }

        let encoded_id = path_id(id)?;
        let path = format!("/rules/{}", encoded_id);
        let response = self.client.patch(&path, &request).await?;
        let rule: Rule = response.json().await?;

        Ok(rule)
    }

    /// Deletes a rule by ID.
    pub async fn delete(&self, id: &str) -> Result<()> {
        if id.is_empty() {
            return Err(Error::validation("Rule ID is required"));
        }

        let encoded_id = path_id(id)?;
        let path = format!("/rules/{}", encoded_id);
        self.client.delete(&path).await?;

        Ok(())
    }
}
