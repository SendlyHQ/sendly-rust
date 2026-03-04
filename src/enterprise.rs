use crate::client::Sendly;
use crate::error::Result;
use crate::models::{
    AnalyticsOverview, AnalyticsPeriod, AutoTopUpSettings, BillingBreakdown,
    BillingBreakdownOptions, BulkProvisionRequest, BulkProvisionResult, BulkProvisionWorkspace,
    CancelInvitationResponse, CreateOptInPageRequest, CreateOptInPageResponse,
    CreateWorkspaceKeyRequest, CreateWorkspaceRequest, CreditsAnalytics, DeleteOptInPageResponse,
    DeleteWorkspaceResponse, DeliveryByWorkspace, EnterpriseAccount, EnterpriseWebhook,
    EnterpriseWebhookTestResult, EnterpriseWorkspace, EnterpriseWorkspaceDetail,
    InheritVerificationRequest, InheritVerificationResponse, Invitation, MessagesAnalytics,
    OptInPage, ProvisionWorkspaceRequest, ProvisionWorkspaceResponse, QuotaSettings,
    ResumeWorkspaceResponse, RevokeKeyResponse, SendInvitationRequest, SetCustomDomainRequest,
    SetCustomDomainResponse, SetEnterpriseWebhookRequest, SetWorkspaceWebhookRequest,
    SetWorkspaceWebhookResponse, SubmitVerificationRequest, SubmitVerificationResponse,
    SuspendWorkspaceRequest, SuspendWorkspaceResponse, UpdateAutoTopUpRequest,
    UpdateOptInPageRequest, UpdateQuotaRequest, WorkspaceCredits, WorkspaceKey,
    WorkspaceKeyResponse, WorkspaceTransferCreditsRequest, WorkspaceTransferCreditsResponse,
    WorkspaceVerificationStatus, WorkspaceWebhookConfig, WorkspaceWebhookTestResult,
};

pub struct WorkspacesResource<'a> {
    client: &'a Sendly,
}

impl<'a> WorkspacesResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    pub async fn create(&self, request: CreateWorkspaceRequest) -> Result<EnterpriseWorkspace> {
        let response = self.client.post("/enterprise/workspaces", &request).await?;
        Ok(response.json().await?)
    }

    pub async fn list(&self) -> Result<Vec<EnterpriseWorkspaceDetail>> {
        let response = self.client.get("/enterprise/workspaces", &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn get(&self, workspace_id: impl AsRef<str>) -> Result<EnterpriseWorkspaceDetail> {
        let path = format!("/enterprise/workspaces/{}", workspace_id.as_ref());
        let response = self.client.get(&path, &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn delete(&self, workspace_id: impl AsRef<str>) -> Result<DeleteWorkspaceResponse> {
        let path = format!("/enterprise/workspaces/{}", workspace_id.as_ref());
        let response = self.client.delete(&path).await?;
        Ok(response.json().await?)
    }

    pub async fn submit_verification(
        &self,
        workspace_id: impl AsRef<str>,
        request: SubmitVerificationRequest,
    ) -> Result<SubmitVerificationResponse> {
        let path = format!(
            "/enterprise/workspaces/{}/verification/submit",
            workspace_id.as_ref()
        );
        let response = self.client.post(&path, &request).await?;
        Ok(response.json().await?)
    }

    pub async fn inherit_verification(
        &self,
        workspace_id: impl AsRef<str>,
        source_workspace_id: impl Into<String>,
    ) -> Result<InheritVerificationResponse> {
        let path = format!(
            "/enterprise/workspaces/{}/verification/inherit",
            workspace_id.as_ref()
        );
        let request = InheritVerificationRequest {
            source_workspace_id: source_workspace_id.into(),
        };
        let response = self.client.post(&path, &request).await?;
        Ok(response.json().await?)
    }

    pub async fn get_verification(
        &self,
        workspace_id: impl AsRef<str>,
    ) -> Result<WorkspaceVerificationStatus> {
        let path = format!(
            "/enterprise/workspaces/{}/verification",
            workspace_id.as_ref()
        );
        let response = self.client.get(&path, &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn transfer_credits(
        &self,
        workspace_id: impl AsRef<str>,
        source_workspace_id: impl Into<String>,
        amount: i32,
    ) -> Result<WorkspaceTransferCreditsResponse> {
        let path = format!(
            "/enterprise/workspaces/{}/transfer-credits",
            workspace_id.as_ref()
        );
        let request = WorkspaceTransferCreditsRequest {
            source_workspace_id: source_workspace_id.into(),
            amount,
        };
        let response = self.client.post(&path, &request).await?;
        Ok(response.json().await?)
    }

    pub async fn get_credits(&self, workspace_id: impl AsRef<str>) -> Result<WorkspaceCredits> {
        let path = format!("/enterprise/workspaces/{}/credits", workspace_id.as_ref());
        let response = self.client.get(&path, &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn create_key(
        &self,
        workspace_id: impl AsRef<str>,
        request: CreateWorkspaceKeyRequest,
    ) -> Result<WorkspaceKeyResponse> {
        let path = format!("/enterprise/workspaces/{}/keys", workspace_id.as_ref());
        let response = self.client.post(&path, &request).await?;
        Ok(response.json().await?)
    }

    pub async fn list_keys(&self, workspace_id: impl AsRef<str>) -> Result<Vec<WorkspaceKey>> {
        let path = format!("/enterprise/workspaces/{}/keys", workspace_id.as_ref());
        let response = self.client.get(&path, &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn revoke_key(
        &self,
        workspace_id: impl AsRef<str>,
        key_id: impl AsRef<str>,
    ) -> Result<RevokeKeyResponse> {
        let path = format!(
            "/enterprise/workspaces/{}/keys/{}",
            workspace_id.as_ref(),
            key_id.as_ref()
        );
        let response = self.client.delete(&path).await?;
        Ok(response.json().await?)
    }

    pub async fn list_opt_in_pages(&self, workspace_id: impl AsRef<str>) -> Result<Vec<OptInPage>> {
        let path = format!(
            "/enterprise/workspaces/{}/opt-in-pages",
            workspace_id.as_ref()
        );
        let response = self.client.get(&path, &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn create_opt_in_page(
        &self,
        workspace_id: impl AsRef<str>,
        request: CreateOptInPageRequest,
    ) -> Result<CreateOptInPageResponse> {
        let path = format!(
            "/enterprise/workspaces/{}/opt-in-pages",
            workspace_id.as_ref()
        );
        let response = self.client.post(&path, &request).await?;
        Ok(response.json().await?)
    }

    pub async fn update_opt_in_page(
        &self,
        workspace_id: impl AsRef<str>,
        page_id: impl AsRef<str>,
        request: UpdateOptInPageRequest,
    ) -> Result<OptInPage> {
        let path = format!(
            "/enterprise/workspaces/{}/opt-in-pages/{}",
            workspace_id.as_ref(),
            page_id.as_ref()
        );
        let response = self.client.patch(&path, &request).await?;
        Ok(response.json().await?)
    }

    pub async fn delete_opt_in_page(
        &self,
        workspace_id: impl AsRef<str>,
        page_id: impl AsRef<str>,
    ) -> Result<DeleteOptInPageResponse> {
        let path = format!(
            "/enterprise/workspaces/{}/opt-in-pages/{}",
            workspace_id.as_ref(),
            page_id.as_ref()
        );
        let response = self.client.delete(&path).await?;
        Ok(response.json().await?)
    }

    pub async fn set_webhook(
        &self,
        workspace_id: impl AsRef<str>,
        request: SetWorkspaceWebhookRequest,
    ) -> Result<SetWorkspaceWebhookResponse> {
        let path = format!("/enterprise/workspaces/{}/webhooks", workspace_id.as_ref());
        let response = self.client.put(&path, &request).await?;
        Ok(response.json().await?)
    }

    pub async fn list_webhooks(
        &self,
        workspace_id: impl AsRef<str>,
    ) -> Result<Vec<WorkspaceWebhookConfig>> {
        let path = format!("/enterprise/workspaces/{}/webhooks", workspace_id.as_ref());
        let response = self.client.get(&path, &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn delete_webhooks(
        &self,
        workspace_id: impl AsRef<str>,
        webhook_id: Option<impl AsRef<str>>,
    ) -> Result<()> {
        let mut path = format!("/enterprise/workspaces/{}/webhooks", workspace_id.as_ref());
        if let Some(id) = webhook_id {
            path.push_str(&format!("?webhookId={}", id.as_ref()));
        }
        self.client.delete(&path).await?;
        Ok(())
    }

    pub async fn test_webhook(
        &self,
        workspace_id: impl AsRef<str>,
    ) -> Result<WorkspaceWebhookTestResult> {
        let path = format!(
            "/enterprise/workspaces/{}/webhooks/test",
            workspace_id.as_ref()
        );
        let response = self.client.post(&path, &()).await?;
        Ok(response.json().await?)
    }

    pub async fn suspend(
        &self,
        workspace_id: impl AsRef<str>,
        reason: Option<impl Into<String>>,
    ) -> Result<SuspendWorkspaceResponse> {
        let path = format!("/enterprise/workspaces/{}/suspend", workspace_id.as_ref());
        let request = SuspendWorkspaceRequest {
            reason: reason.map(|r| r.into()),
        };
        let response = self.client.post(&path, &request).await?;
        Ok(response.json().await?)
    }

    pub async fn resume(&self, workspace_id: impl AsRef<str>) -> Result<ResumeWorkspaceResponse> {
        let path = format!("/enterprise/workspaces/{}/resume", workspace_id.as_ref());
        let response = self.client.post(&path, &()).await?;
        Ok(response.json().await?)
    }

    pub async fn provision_bulk(
        &self,
        workspaces: Vec<BulkProvisionWorkspace>,
    ) -> Result<BulkProvisionResult> {
        let request = BulkProvisionRequest { workspaces };
        let response = self
            .client
            .post("/enterprise/workspaces/provision/bulk", &request)
            .await?;
        Ok(response.json().await?)
    }

    pub async fn set_custom_domain(
        &self,
        workspace_id: impl AsRef<str>,
        page_id: impl AsRef<str>,
        domain: impl Into<String>,
    ) -> Result<SetCustomDomainResponse> {
        let path = format!(
            "/enterprise/workspaces/{}/pages/{}/domain",
            workspace_id.as_ref(),
            page_id.as_ref()
        );
        let request = SetCustomDomainRequest {
            domain: domain.into(),
        };
        let response = self.client.put(&path, &request).await?;
        Ok(response.json().await?)
    }

    pub async fn send_invitation(
        &self,
        workspace_id: impl AsRef<str>,
        request: SendInvitationRequest,
    ) -> Result<Invitation> {
        let path = format!(
            "/enterprise/workspaces/{}/invitations",
            workspace_id.as_ref()
        );
        let response = self.client.post(&path, &request).await?;
        Ok(response.json().await?)
    }

    pub async fn list_invitations(&self, workspace_id: impl AsRef<str>) -> Result<Vec<Invitation>> {
        let path = format!(
            "/enterprise/workspaces/{}/invitations",
            workspace_id.as_ref()
        );
        let response = self.client.get(&path, &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn cancel_invitation(
        &self,
        workspace_id: impl AsRef<str>,
        invite_id: impl AsRef<str>,
    ) -> Result<CancelInvitationResponse> {
        let path = format!(
            "/enterprise/workspaces/{}/invitations/{}",
            workspace_id.as_ref(),
            invite_id.as_ref()
        );
        let response = self.client.delete(&path).await?;
        Ok(response.json().await?)
    }

    pub async fn get_quota(&self, workspace_id: impl AsRef<str>) -> Result<QuotaSettings> {
        let path = format!("/enterprise/workspaces/{}/quota", workspace_id.as_ref());
        let response = self.client.get(&path, &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn set_quota(
        &self,
        workspace_id: impl AsRef<str>,
        request: UpdateQuotaRequest,
    ) -> Result<QuotaSettings> {
        let path = format!("/enterprise/workspaces/{}/quota", workspace_id.as_ref());
        let response = self.client.put(&path, &request).await?;
        Ok(response.json().await?)
    }
}

pub struct EnterpriseWebhooksResource<'a> {
    client: &'a Sendly,
}

impl<'a> EnterpriseWebhooksResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    pub async fn set(&self, url: impl Into<String>) -> Result<EnterpriseWebhook> {
        let request = SetEnterpriseWebhookRequest { url: url.into() };
        let response = self.client.post("/enterprise/webhooks", &request).await?;
        Ok(response.json().await?)
    }

    pub async fn get(&self) -> Result<EnterpriseWebhook> {
        let response = self.client.get("/enterprise/webhooks", &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn delete(&self) -> Result<()> {
        self.client.delete("/enterprise/webhooks").await?;
        Ok(())
    }

    pub async fn test(&self) -> Result<EnterpriseWebhookTestResult> {
        let response = self.client.post("/enterprise/webhooks/test", &()).await?;
        Ok(response.json().await?)
    }
}

pub struct EnterpriseAnalyticsResource<'a> {
    client: &'a Sendly,
}

impl<'a> EnterpriseAnalyticsResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    pub async fn overview(&self) -> Result<AnalyticsOverview> {
        let response = self
            .client
            .get("/enterprise/analytics/overview", &[])
            .await?;
        Ok(response.json().await?)
    }

    pub async fn messages(&self, options: Option<AnalyticsPeriod>) -> Result<MessagesAnalytics> {
        let query = options.unwrap_or_default().to_query_params();
        let response = self
            .client
            .get("/enterprise/analytics/messages", &query)
            .await?;
        Ok(response.json().await?)
    }

    pub async fn delivery(&self) -> Result<Vec<DeliveryByWorkspace>> {
        let response = self
            .client
            .get("/enterprise/analytics/delivery", &[])
            .await?;
        Ok(response.json().await?)
    }

    pub async fn credits(&self, options: Option<AnalyticsPeriod>) -> Result<CreditsAnalytics> {
        let query = options.unwrap_or_default().to_query_params();
        let response = self
            .client
            .get("/enterprise/analytics/credits", &query)
            .await?;
        Ok(response.json().await?)
    }
}

pub struct EnterpriseSettingsResource<'a> {
    client: &'a Sendly,
}

impl<'a> EnterpriseSettingsResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    pub async fn get_auto_top_up(&self) -> Result<AutoTopUpSettings> {
        let response = self
            .client
            .get("/enterprise/settings/auto-top-up", &[])
            .await?;
        Ok(response.json().await?)
    }

    pub async fn update_auto_top_up(
        &self,
        request: UpdateAutoTopUpRequest,
    ) -> Result<AutoTopUpSettings> {
        let response = self
            .client
            .put("/enterprise/settings/auto-top-up", &request)
            .await?;
        Ok(response.json().await?)
    }
}

pub struct EnterpriseBillingResource<'a> {
    client: &'a Sendly,
}

impl<'a> EnterpriseBillingResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    pub async fn get_breakdown(
        &self,
        options: Option<BillingBreakdownOptions>,
    ) -> Result<BillingBreakdown> {
        let query = options.unwrap_or_default().to_query_params();
        let response = self
            .client
            .get("/enterprise/billing/workspace-breakdown", &query)
            .await?;
        Ok(response.json().await?)
    }
}

pub struct EnterpriseResource<'a> {
    client: &'a Sendly,
}

impl<'a> EnterpriseResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    pub fn workspaces(&self) -> WorkspacesResource<'a> {
        WorkspacesResource::new(self.client)
    }

    pub fn webhooks(&self) -> EnterpriseWebhooksResource<'a> {
        EnterpriseWebhooksResource::new(self.client)
    }

    pub fn analytics(&self) -> EnterpriseAnalyticsResource<'a> {
        EnterpriseAnalyticsResource::new(self.client)
    }

    pub fn settings(&self) -> EnterpriseSettingsResource<'a> {
        EnterpriseSettingsResource::new(self.client)
    }

    pub fn billing(&self) -> EnterpriseBillingResource<'a> {
        EnterpriseBillingResource::new(self.client)
    }

    pub async fn get_account(&self) -> Result<EnterpriseAccount> {
        let response = self.client.get("/enterprise/account", &[]).await?;
        Ok(response.json().await?)
    }

    pub async fn provision(
        &self,
        request: ProvisionWorkspaceRequest,
    ) -> Result<ProvisionWorkspaceResponse> {
        let response = self
            .client
            .post("/enterprise/workspaces/provision", &request)
            .await?;
        Ok(response.json().await?)
    }
}
