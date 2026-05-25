//! Business Upgrade Resource — Entity-Upgrade ("fork-with-new-number")
//!
//! Manages the toll-free business entity upgrade flow: when a customer
//! forms a new legal entity (e.g. an LLC), this resource lets them
//! reserve a new toll-free number under the new entity, submit it for
//! carrier review, and atomically swap to it on approval — without
//! disrupting outbound SMS during the 1-2 week review window.
//!
//! See <https://sendly.live/docs/business-upgrade> for the full flow.

use reqwest::multipart;
use serde::{Deserialize, Serialize};

use crate::client::Sendly;
use crate::error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityType {
    SoleProprietor,
    PrivateProfit,
    PublicProfit,
    NonProfit,
    Government,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::SoleProprietor => "SOLE_PROPRIETOR",
            EntityType::PrivateProfit => "PRIVATE_PROFIT",
            EntityType::PublicProfit => "PUBLIC_PROFIT",
            EntityType::NonProfit => "NON_PROFIT",
            EntityType::Government => "GOVERNMENT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum BrnType {
    Ein,
    Ssn,
    Duns,
    Cra,
    Vat,
    Lei,
    Other,
}

impl BrnType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BrnType::Ein => "EIN",
            BrnType::Ssn => "SSN",
            BrnType::Duns => "DUNS",
            BrnType::Cra => "CRA",
            BrnType::Vat => "VAT",
            BrnType::Lei => "LEI",
            BrnType::Other => "OTHER",
        }
    }
}

/// What to do with the old toll-free number after an entity-upgrade is approved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Disposition {
    /// Keep the old number active under a different workspace owned by the
    /// same user (requires `target_workspace_id`).
    Moved,
    /// Return the number to the carrier pool.
    Released,
}

impl Disposition {
    pub fn as_str(&self) -> &'static str {
        match self {
            Disposition::Moved => "moved",
            Disposition::Released => "released",
        }
    }
}

/// Candidate payload for the preflight validator. Same shape as
/// `StartUpgradeRequest` but every field is advisory — preflight never writes.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightCandidate {
    pub business_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doing_business_as: Option<String>,
    pub brn: String,
    pub brn_type: BrnType,
    pub brn_country: String,
    pub entity_type: EntityType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address1: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address2: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_first_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_last_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_volume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_case: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_case_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_messages: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_in_workflow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_information: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age_gated_content: Option<bool>,
}

impl Default for EntityType {
    fn default() -> Self {
        EntityType::PrivateProfit
    }
}

impl Default for BrnType {
    fn default() -> Self {
        BrnType::Ein
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreflightSeverity {
    Blocker,
    Warning,
    Info,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreflightIssue {
    pub severity: PreflightSeverity,
    pub field: String,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreflightProposedFix {
    pub field: String,
    pub current: serde_json::Value,
    pub proposed: serde_json::Value,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PreflightCountry {
    Ca,
    Us,
    Other,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreflightVerdict {
    Ready,
    Warnings,
    Blocked,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightReport {
    pub verification_id: String,
    #[serde(default)]
    pub business_name: Option<String>,
    pub country: PreflightCountry,
    pub verdict: PreflightVerdict,
    #[serde(default)]
    pub issues: Vec<PreflightIssue>,
    #[serde(default)]
    pub proposed_fixes: Vec<PreflightProposedFix>,
}

/// Best-of prefill snapshot across all the caller's verified workspaces.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BestPrefill {
    #[serde(default)]
    pub monthly_volume: Option<String>,
    #[serde(default)]
    pub use_case: Option<String>,
    #[serde(default)]
    pub use_case_summary: Option<String>,
    #[serde(default)]
    pub sample_messages: Option<String>,
    #[serde(default)]
    pub opt_in_workflow: Option<String>,
    #[serde(default)]
    pub opt_in_image_urls: Option<String>,
    #[serde(default)]
    pub opt_in_source: Option<String>,
    #[serde(default)]
    pub privacy_url: Option<String>,
    #[serde(default)]
    pub terms_url: Option<String>,
    #[serde(default)]
    pub additional_information: Option<String>,
    #[serde(default)]
    pub isv_reseller: Option<String>,
    #[serde(default)]
    pub age_gated_content: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BestPrefillResponse {
    pub prefill: BestPrefill,
    #[serde(default)]
    pub source_workspace_count: i32,
}

/// Required + optional fields for kicking off an entity upgrade.
#[derive(Debug, Clone, Default)]
pub struct StartUpgradeRequest {
    pub business_name: String,
    pub brn: String,
    pub brn_type: BrnType,
    pub brn_country: String,
    pub entity_type: EntityType,
    pub doing_business_as: Option<String>,
    pub website: Option<String>,
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub address_country: Option<String>,
    pub contact_first_name: Option<String>,
    pub contact_last_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub monthly_volume: Option<String>,
    pub use_case: Option<String>,
    pub use_case_summary: Option<String>,
    pub sample_messages: Option<String>,
    pub opt_in_workflow: Option<String>,
    pub privacy_url: Option<String>,
    pub terms_url: Option<String>,
    pub additional_information: Option<String>,
    pub age_gated_content: Option<bool>,
}

impl StartUpgradeRequest {
    pub fn new(
        business_name: impl Into<String>,
        brn: impl Into<String>,
        brn_type: BrnType,
        brn_country: impl Into<String>,
        entity_type: EntityType,
    ) -> Self {
        Self {
            business_name: business_name.into(),
            brn: brn.into(),
            brn_type,
            brn_country: brn_country.into(),
            entity_type,
            ..Self::default()
        }
    }

    fn append_text_fields(&self, mut form: multipart::Form) -> multipart::Form {
        form = form.text("businessName", self.business_name.clone());
        form = form.text("brn", self.brn.clone());
        form = form.text("brnType", self.brn_type.as_str());
        form = form.text("brnCountry", self.brn_country.clone());
        form = form.text("entityType", self.entity_type.as_str());
        form = append_opt(form, "doingBusinessAs", self.doing_business_as.as_deref());
        form = append_opt(form, "website", self.website.as_deref());
        form = append_opt(form, "address1", self.address1.as_deref());
        form = append_opt(form, "address2", self.address2.as_deref());
        form = append_opt(form, "city", self.city.as_deref());
        form = append_opt(form, "state", self.state.as_deref());
        form = append_opt(form, "zip", self.zip.as_deref());
        form = append_opt(form, "addressCountry", self.address_country.as_deref());
        form = append_opt(form, "contactFirstName", self.contact_first_name.as_deref());
        form = append_opt(form, "contactLastName", self.contact_last_name.as_deref());
        form = append_opt(form, "contactEmail", self.contact_email.as_deref());
        form = append_opt(form, "contactPhone", self.contact_phone.as_deref());
        form = append_opt(form, "monthlyVolume", self.monthly_volume.as_deref());
        form = append_opt(form, "useCase", self.use_case.as_deref());
        form = append_opt(form, "useCaseSummary", self.use_case_summary.as_deref());
        form = append_opt(form, "sampleMessages", self.sample_messages.as_deref());
        form = append_opt(form, "optInWorkflow", self.opt_in_workflow.as_deref());
        form = append_opt(form, "privacyUrl", self.privacy_url.as_deref());
        form = append_opt(form, "termsUrl", self.terms_url.as_deref());
        form = append_opt(
            form,
            "additionalInformation",
            self.additional_information.as_deref(),
        );
        if let Some(b) = self.age_gated_content {
            form = form.text("ageGatedContent", b.to_string());
        }
        form
    }
}

/// Partial-update payload used by `resubmit`. Every field is optional — only
/// the ones you send are touched on the pending verification.
#[derive(Debug, Clone, Default)]
pub struct ResubmitUpgradeRequest {
    pub business_name: Option<String>,
    pub brn: Option<String>,
    pub brn_type: Option<BrnType>,
    pub brn_country: Option<String>,
    pub entity_type: Option<EntityType>,
    pub doing_business_as: Option<String>,
    pub website: Option<String>,
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub address_country: Option<String>,
    pub contact_first_name: Option<String>,
    pub contact_last_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub monthly_volume: Option<String>,
    pub use_case: Option<String>,
    pub use_case_summary: Option<String>,
    pub sample_messages: Option<String>,
    pub opt_in_workflow: Option<String>,
    pub privacy_url: Option<String>,
    pub terms_url: Option<String>,
    pub additional_information: Option<String>,
    pub age_gated_content: Option<bool>,
}

impl ResubmitUpgradeRequest {
    pub fn new() -> Self {
        Self::default()
    }

    fn append_text_fields(&self, mut form: multipart::Form) -> multipart::Form {
        form = append_opt(form, "businessName", self.business_name.as_deref());
        form = append_opt(form, "brn", self.brn.as_deref());
        if let Some(bt) = self.brn_type {
            form = form.text("brnType", bt.as_str());
        }
        form = append_opt(form, "brnCountry", self.brn_country.as_deref());
        if let Some(et) = self.entity_type {
            form = form.text("entityType", et.as_str());
        }
        form = append_opt(form, "doingBusinessAs", self.doing_business_as.as_deref());
        form = append_opt(form, "website", self.website.as_deref());
        form = append_opt(form, "address1", self.address1.as_deref());
        form = append_opt(form, "address2", self.address2.as_deref());
        form = append_opt(form, "city", self.city.as_deref());
        form = append_opt(form, "state", self.state.as_deref());
        form = append_opt(form, "zip", self.zip.as_deref());
        form = append_opt(form, "addressCountry", self.address_country.as_deref());
        form = append_opt(form, "contactFirstName", self.contact_first_name.as_deref());
        form = append_opt(form, "contactLastName", self.contact_last_name.as_deref());
        form = append_opt(form, "contactEmail", self.contact_email.as_deref());
        form = append_opt(form, "contactPhone", self.contact_phone.as_deref());
        form = append_opt(form, "monthlyVolume", self.monthly_volume.as_deref());
        form = append_opt(form, "useCase", self.use_case.as_deref());
        form = append_opt(form, "useCaseSummary", self.use_case_summary.as_deref());
        form = append_opt(form, "sampleMessages", self.sample_messages.as_deref());
        form = append_opt(form, "optInWorkflow", self.opt_in_workflow.as_deref());
        form = append_opt(form, "privacyUrl", self.privacy_url.as_deref());
        form = append_opt(form, "termsUrl", self.terms_url.as_deref());
        form = append_opt(
            form,
            "additionalInformation",
            self.additional_information.as_deref(),
        );
        if let Some(b) = self.age_gated_content {
            form = form.text("ageGatedContent", b.to_string());
        }
        form
    }
}

fn append_opt(form: multipart::Form, name: &'static str, value: Option<&str>) -> multipart::Form {
    if let Some(v) = value {
        form.text(name, v.to_string())
    } else {
        form
    }
}

/// Optional IRS-letter / EIN document attached to a `start` or `resubmit`
/// call. The carrier sometimes asks for proof of the new entity's EIN; the
/// file is stored on R2 and auto-deleted on cancel.
#[derive(Debug, Clone)]
pub struct EinDocument {
    pub bytes: Vec<u8>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
}

impl EinDocument {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            filename: None,
            content_type: None,
        }
    }

    pub fn filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    fn into_part(self) -> Result<multipart::Part> {
        let filename = self.filename.unwrap_or_else(|| "ein-doc.pdf".to_string());
        let content_type = self
            .content_type
            .unwrap_or_else(|| "application/pdf".to_string());
        multipart::Part::bytes(self.bytes)
            .file_name(filename)
            .mime_str(&content_type)
            .map_err(|e| Error::Validation {
                message: format!("Invalid EIN doc content type: {}", e),
            })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartUpgradeResponse {
    #[serde(default)]
    pub success: bool,
    pub pending_verification_id: String,
    pub telnyx_verification_id: String,
    pub toll_free_number: String,
    pub telnyx_messaging_profile_id: String,
    #[serde(default)]
    pub ein_doc_stored: bool,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingUpgrade {
    pub id: String,
    pub business_name: String,
    pub status: String,
    #[serde(default)]
    pub entity_type: Option<String>,
    #[serde(default)]
    pub brn_type: Option<String>,
    #[serde(default)]
    pub brn_country: Option<String>,
    #[serde(default)]
    pub toll_free_number: Option<String>,
    #[serde(default)]
    pub rejection_reason: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpgradeStatusResponse {
    #[serde(default)]
    pub pending: Option<PendingUpgrade>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelUpgradeResponse {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub cancelled: bool,
    #[serde(default)]
    pub cancelled_verification_id: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResubmitUpgradeResponse {
    #[serde(default)]
    pub success: bool,
    pub pending_verification_id: String,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DispositionResponse {
    #[serde(default)]
    pub success: bool,
    pub disposition: String,
    pub superseded_verification_id: String,
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SetDispositionRequest {
    pub disposition: Option<Disposition>,
    /// When `disposition` is `Moved`, the workspace that should adopt the
    /// old toll-free number. Required for `Moved`, ignored for `Released`.
    pub target_workspace_id: Option<String>,
}

impl SetDispositionRequest {
    pub fn moved(target_workspace_id: impl Into<String>) -> Self {
        Self {
            disposition: Some(Disposition::Moved),
            target_workspace_id: Some(target_workspace_id.into()),
        }
    }

    pub fn released() -> Self {
        Self {
            disposition: Some(Disposition::Released),
            target_workspace_id: None,
        }
    }
}

#[derive(Debug, Serialize)]
struct DispositionPayload<'a> {
    disposition: &'a str,
    #[serde(rename = "targetOrgId", skip_serializing_if = "Option::is_none")]
    target_org_id: Option<&'a str>,
}

/// Business-upgrade ("fork-with-new-number") resource.
///
/// # Example
///
/// ```rust,no_run
/// use sendly::{Sendly, business_upgrade::{PreflightCandidate, BrnType, EntityType}};
///
/// # async fn run() -> Result<(), sendly::Error> {
/// let client = Sendly::new("sk_live_v1_xxx");
///
/// let report = client.business_upgrade().preflight(PreflightCandidate {
///     business_name: "Acme Holdings LLC".to_string(),
///     brn: "12-3456789".to_string(),
///     brn_type: BrnType::Ein,
///     brn_country: "US".to_string(),
///     entity_type: EntityType::PrivateProfit,
///     ..Default::default()
/// }).await?;
/// println!("Verdict: {:?}", report.verdict);
/// # Ok(()) }
/// ```
pub struct BusinessUpgradeResource<'a> {
    client: &'a Sendly,
}

impl<'a> BusinessUpgradeResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Validate a candidate entity-upgrade payload before submission. Returns
    /// issues + proposed auto-fixes — purely advisory, no writes.
    pub async fn preflight(&self, candidate: PreflightCandidate) -> Result<PreflightReport> {
        let response = self.client.post("/verification/preflight", &candidate).await?;
        Ok(response.json().await?)
    }

    /// Best-of prefill across all the caller's verified workspaces. Returns
    /// the most-recent non-empty value per messaging field — useful for
    /// pre-populating the upgrade form.
    pub async fn best_prefill(&self) -> Result<BestPrefillResponse> {
        let response = self.client.get("/verification/best-prefill", &[]).await?;
        Ok(response.json().await?)
    }

    /// Start an entity upgrade for the given workspace. Auto-provisions a new
    /// toll-free number + messaging profile and submits to the carrier. The
    /// current number keeps sending during the 1-2 week review; on approval
    /// the new number is promoted atomically.
    pub async fn start(
        &self,
        workspace_id: &str,
        request: StartUpgradeRequest,
        ein_doc: Option<EinDocument>,
    ) -> Result<StartUpgradeResponse> {
        let mut form = multipart::Form::new();
        form = request.append_text_fields(form);
        if let Some(doc) = ein_doc {
            form = form.part("einDoc", doc.into_part()?);
        }
        let path = format!(
            "/workspaces/{}/upgrade",
            urlencoding::encode(workspace_id)
        );
        let response = self.client.post_multipart(&path, form).await?;
        Ok(response.json().await?)
    }

    /// Whether the given workspace has a pending entity upgrade. Returns
    /// `{ pending: None }` when no upgrade is in flight.
    pub async fn status(&self, workspace_id: &str) -> Result<UpgradeStatusResponse> {
        let path = format!(
            "/workspaces/{}/upgrade/status",
            urlencoding::encode(workspace_id)
        );
        let response = self.client.get(&path, &[]).await?;
        Ok(response.json().await?)
    }

    /// Cancel a pending entity upgrade — releases the reserved number,
    /// deletes the new messaging profile, removes the stored EIN doc.
    /// Idempotent.
    pub async fn cancel(&self, workspace_id: &str) -> Result<CancelUpgradeResponse> {
        let path = format!(
            "/workspaces/{}/upgrade/cancel",
            urlencoding::encode(workspace_id)
        );
        let response = self.client.post(&path, &serde_json::json!({})).await?;
        Ok(response.json().await?)
    }

    /// Resubmit a rejected (or waiting-for-customer) entity upgrade with
    /// updated fields and optionally a new EIN document.
    pub async fn resubmit(
        &self,
        workspace_id: &str,
        request: ResubmitUpgradeRequest,
        ein_doc: Option<EinDocument>,
    ) -> Result<ResubmitUpgradeResponse> {
        let mut form = multipart::Form::new();
        form = request.append_text_fields(form);
        if let Some(doc) = ein_doc {
            form = form.part("einDoc", doc.into_part()?);
        }
        let path = format!(
            "/workspaces/{}/upgrade/resubmit",
            urlencoding::encode(workspace_id)
        );
        let response = self.client.post_multipart(&path, form).await?;
        Ok(response.json().await?)
    }

    /// After approval, choose what happens to the old number — `Moved` to
    /// another workspace owned by the same user, or `Released` back to the
    /// carrier pool.
    pub async fn set_disposition(
        &self,
        workspace_id: &str,
        request: SetDispositionRequest,
    ) -> Result<DispositionResponse> {
        let disposition = request.disposition.ok_or_else(|| Error::Validation {
            message: "set_disposition requires a disposition".to_string(),
        })?;
        if disposition == Disposition::Moved && request.target_workspace_id.is_none() {
            return Err(Error::Validation {
                message: "Disposition::Moved requires target_workspace_id".to_string(),
            });
        }
        let payload = DispositionPayload {
            disposition: disposition.as_str(),
            target_org_id: request.target_workspace_id.as_deref(),
        };
        let path = format!(
            "/workspaces/{}/upgrade/disposition",
            urlencoding::encode(workspace_id)
        );
        let response = self.client.post(&path, &payload).await?;
        Ok(response.json().await?)
    }
}
