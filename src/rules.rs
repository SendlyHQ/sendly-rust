use crate::client::Sendly;
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
            return Err(Error::Validation {
                message: "Rule name is required".to_string(),
            });
        }
        if request.conditions.is_empty() {
            return Err(Error::Validation {
                message: "Rule conditions are required".to_string(),
            });
        }
        if request.actions.is_empty() {
            return Err(Error::Validation {
                message: "Rule actions are required".to_string(),
            });
        }

        let response = self.client.post("/rules", &request).await?;
        let rule: Rule = response.json().await?;

        Ok(rule)
    }

    /// Updates a rule.
    pub async fn update(&self, id: &str, request: UpdateRuleRequest) -> Result<Rule> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Rule ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let path = format!("/rules/{}", encoded_id);
        let response = self.client.patch(&path, &request).await?;
        let rule: Rule = response.json().await?;

        Ok(rule)
    }

    /// Deletes a rule by ID.
    pub async fn delete(&self, id: &str) -> Result<()> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Rule ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let path = format!("/rules/{}", encoded_id);
        self.client.delete(&path).await?;

        Ok(())
    }
}
