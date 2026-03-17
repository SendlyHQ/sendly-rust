use crate::client::Sendly;
use crate::error::{Error, Result};
use crate::models::{CreateLabelRequest, Label, LabelListResponse};

/// Labels resource for managing conversation labels.
#[derive(Debug, Clone)]
pub struct LabelsResource<'a> {
    client: &'a Sendly,
}

impl<'a> LabelsResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Creates a new label.
    pub async fn create(&self, request: CreateLabelRequest) -> Result<Label> {
        if request.name.is_empty() {
            return Err(Error::Validation {
                message: "Label name is required".to_string(),
            });
        }

        let response = self.client.post("/labels", &request).await?;
        let label: Label = response.json().await?;

        Ok(label)
    }

    /// Lists all labels.
    pub async fn list(&self) -> Result<LabelListResponse> {
        let response = self.client.get("/labels", &[]).await?;
        let result: LabelListResponse = response.json().await?;

        Ok(result)
    }

    /// Deletes a label by ID.
    pub async fn delete(&self, id: &str) -> Result<()> {
        if id.is_empty() {
            return Err(Error::Validation {
                message: "Label ID is required".to_string(),
            });
        }

        let encoded_id = urlencoding::encode(id);
        let path = format!("/labels/{}", encoded_id);
        self.client.delete(&path).await?;

        Ok(())
    }
}
