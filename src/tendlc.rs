//! 10DLC Resource — Local-Number Texting Registration
//!
//! Register your business for carrier review so you can text from local
//! (10-digit) US numbers. The flow has three steps:
//!
//! 1. **Brand** — register your business identity. Starts `pending`; poll
//!    [`TenDlcResource::get_brand`] until it becomes `verified` (or `failed`,
//!    with `failure_reasons` explaining why).
//! 2. **Campaign** — describe your messaging use case under a verified brand
//!    and submit it for carrier review. Starts `pending`; poll
//!    [`TenDlcResource::get_campaign`] until it becomes `active`.
//!    [`TenDlcResource::qualify`] pre-checks a use case before you create
//!    the campaign.
//! 3. **Assign** — attach a number you own to the active campaign with
//!    [`TenDlcResource::assign_number`]. Once the assignment is `Active`,
//!    the number can send.
//!
//! Brand, campaign, and number-assignment writes require a live API key
//! (`sk_live_v1_xxx`) with the `tendlc:write` scope.
//!
//! See <https://sendly.live/docs/10dlc> for the full flow.

use serde::{Deserialize, Serialize};

use crate::client::Sendly;
use crate::error::Result;

/// A business identity registered for carrier review.
#[derive(Debug, Clone, Deserialize)]
pub struct TenDlcBrand {
    /// Unique brand identifier.
    pub id: String,
    /// Legal business name.
    #[serde(alias = "legalName")]
    pub legal_name: String,
    /// "Doing business as" name, if different from the legal name.
    #[serde(default)]
    pub dba: Option<String>,
    /// Business entity type (e.g. `PRIVATE_PROFIT`, `SOLE_PROPRIETOR`).
    #[serde(alias = "entityType")]
    pub entity_type: String,
    /// Business registration number (e.g. EIN).
    #[serde(default)]
    pub ein: Option<String>,
    /// Industry vertical.
    #[serde(default)]
    pub vertical: Option<String>,
    /// Business website URL.
    #[serde(default)]
    pub website: Option<String>,
    /// Carrier-review status: `pending`, `verified`, or `failed`.
    pub status: String,
    /// Identity-verification detail from the carrier review, when available.
    #[serde(default, alias = "identityStatus")]
    pub identity_status: Option<String>,
    /// Why the review failed, when `status` is `failed`.
    #[serde(default, alias = "failureReasons")]
    pub failure_reasons: Option<Vec<String>>,
    /// When the brand was created (ISO 8601).
    #[serde(alias = "createdAt")]
    pub created_at: String,
    /// When the brand was last updated (ISO 8601).
    #[serde(alias = "updatedAt")]
    pub updated_at: String,
}

/// Response from [`TenDlcResource::list_brands`].
#[derive(Debug, Clone, Deserialize)]
pub struct TenDlcBrandListResponse {
    #[serde(default)]
    pub data: Vec<TenDlcBrand>,
}

/// Response from [`TenDlcResource::create_brand`] and
/// [`TenDlcResource::get_brand`].
#[derive(Debug, Clone, Deserialize)]
pub struct TenDlcBrandResponse {
    pub data: TenDlcBrand,
}

/// Request body for [`TenDlcResource::create_brand`].
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTenDlcBrandRequest {
    /// Legal business name.
    pub legal_name: String,
    /// "Doing business as" name, if different from the legal name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dba: Option<String>,
    /// Business registration number (e.g. EIN).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ein: Option<String>,
    /// Business entity type (e.g. `PRIVATE_PROFIT`, `SOLE_PROPRIETOR`);
    /// defaults to `PRIVATE_PROFIT`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    /// Industry vertical.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical: Option<String>,
    /// Business website URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    /// Business contact email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Business phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    /// Business mobile phone number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile_phone: Option<String>,
    /// Street address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street: Option<String>,
    /// City.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// State or region.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// Postal code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    /// ISO 3166-1 alpha-2 country code; defaults to `US`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    /// Existing Sendly verification to prefill business details from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_id: Option<String>,
}

impl CreateTenDlcBrandRequest {
    pub fn new(legal_name: impl Into<String>) -> Self {
        Self {
            legal_name: legal_name.into(),
            dba: None,
            ein: None,
            entity_type: None,
            vertical: None,
            website: None,
            email: None,
            phone: None,
            mobile_phone: None,
            street: None,
            city: None,
            state: None,
            postal_code: None,
            country: None,
            verification_id: None,
        }
    }

    pub fn dba(mut self, dba: impl Into<String>) -> Self {
        self.dba = Some(dba.into());
        self
    }

    pub fn ein(mut self, ein: impl Into<String>) -> Self {
        self.ein = Some(ein.into());
        self
    }

    pub fn entity_type(mut self, entity_type: impl Into<String>) -> Self {
        self.entity_type = Some(entity_type.into());
        self
    }

    pub fn vertical(mut self, vertical: impl Into<String>) -> Self {
        self.vertical = Some(vertical.into());
        self
    }

    pub fn website(mut self, website: impl Into<String>) -> Self {
        self.website = Some(website.into());
        self
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    pub fn mobile_phone(mut self, mobile_phone: impl Into<String>) -> Self {
        self.mobile_phone = Some(mobile_phone.into());
        self
    }

    pub fn street(mut self, street: impl Into<String>) -> Self {
        self.street = Some(street.into());
        self
    }

    pub fn city(mut self, city: impl Into<String>) -> Self {
        self.city = Some(city.into());
        self
    }

    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    pub fn postal_code(mut self, postal_code: impl Into<String>) -> Self {
        self.postal_code = Some(postal_code.into());
        self
    }

    pub fn country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    pub fn verification_id(mut self, verification_id: impl Into<String>) -> Self {
        self.verification_id = Some(verification_id.into());
        self
    }
}

/// Throughput detail for a campaign or use-case qualification.
#[derive(Debug, Clone, Deserialize)]
pub struct TenDlcThroughput {
    /// Throughput tier granted by the carrier network (`High volume`,
    /// `Standard`, or `Low volume`).
    pub tier: String,
    /// How many carriers have accepted the campaign so far.
    #[serde(alias = "carriersReady")]
    pub carriers_ready: i64,
}

/// Result of a use-case qualification pre-check.
#[derive(Debug, Clone, Deserialize)]
pub struct TenDlcQualifyResult {
    /// The use-case code that was checked (e.g. `MIXED`, `MARKETING`).
    #[serde(alias = "useCase")]
    pub use_case: String,
    /// Whether the use case qualifies for this brand.
    pub qualified: bool,
    /// Why the use case does not qualify, when `qualified` is false.
    #[serde(default)]
    pub reason: Option<String>,
    /// Expected throughput, when the carrier network reports it.
    #[serde(default)]
    pub throughput: Option<TenDlcThroughput>,
}

/// Response from [`TenDlcResource::qualify`].
#[derive(Debug, Clone, Deserialize)]
pub struct TenDlcQualifyResponse {
    pub data: TenDlcQualifyResult,
}

/// A messaging campaign registered for carrier review.
#[derive(Debug, Clone, Deserialize)]
pub struct TenDlcCampaign {
    /// Unique campaign identifier.
    pub id: String,
    /// The brand this campaign belongs to.
    #[serde(alias = "brandId")]
    pub brand_id: String,
    /// Primary use-case code (e.g. `MIXED`, `MARKETING`).
    #[serde(alias = "useCase")]
    pub use_case: String,
    /// Sub-use-case codes.
    #[serde(default, alias = "subUseCases")]
    pub sub_use_cases: Vec<String>,
    /// What the campaign sends and why.
    #[serde(default)]
    pub description: Option<String>,
    /// Carrier-review status: `pending`, `active`, `failed`, `suspended`,
    /// or `expired`.
    pub status: String,
    /// Example messages the campaign sends.
    #[serde(default, alias = "sampleMessages")]
    pub sample_messages: Vec<String>,
    /// Granted throughput, once carriers approve.
    #[serde(default)]
    pub throughput: Option<TenDlcThroughput>,
    /// Why the review failed, when `status` is `failed`.
    #[serde(default, alias = "failureReasons")]
    pub failure_reasons: Option<Vec<String>>,
    /// When the campaign was created (ISO 8601).
    #[serde(alias = "createdAt")]
    pub created_at: String,
    /// When the campaign was last updated (ISO 8601).
    #[serde(alias = "updatedAt")]
    pub updated_at: String,
}

/// Response from [`TenDlcResource::list_campaigns`].
#[derive(Debug, Clone, Deserialize)]
pub struct TenDlcCampaignListResponse {
    #[serde(default)]
    pub data: Vec<TenDlcCampaign>,
}

/// Response from [`TenDlcResource::create_campaign`] and
/// [`TenDlcResource::get_campaign`].
#[derive(Debug, Clone, Deserialize)]
pub struct TenDlcCampaignResponse {
    pub data: TenDlcCampaign,
}

/// Request body for [`TenDlcResource::create_campaign`].
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTenDlcCampaignRequest {
    /// The verified brand to create the campaign under.
    pub brand_id: String,
    /// Primary use-case code (e.g. `MIXED`, `MARKETING`).
    pub use_case: String,
    /// What the campaign sends and why.
    pub description: String,
    /// How recipients opt in to receive messages.
    pub message_flow: String,
    /// Example messages the campaign sends (the first 5 are used).
    pub sample_messages: Vec<String>,
    /// Sub-use-case codes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_use_cases: Option<Vec<String>>,
    /// Comma-separated keywords that opt a recipient in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_in_keywords: Option<String>,
    /// Comma-separated keywords that opt a recipient out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_out_keywords: Option<String>,
    /// Comma-separated keywords that request help.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_keywords: Option<String>,
    /// Auto-reply sent on opt-in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_in_message: Option<String>,
    /// Auto-reply sent on opt-out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_out_message: Option<String>,
    /// Auto-reply sent on a help request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_message: Option<String>,
    /// Whether messages may contain links; defaults to true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedded_link: Option<bool>,
    /// Whether messages may contain phone numbers; defaults to false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedded_phone: Option<bool>,
}

impl CreateTenDlcCampaignRequest {
    pub fn new(
        brand_id: impl Into<String>,
        use_case: impl Into<String>,
        description: impl Into<String>,
        message_flow: impl Into<String>,
        sample_messages: Vec<String>,
    ) -> Self {
        Self {
            brand_id: brand_id.into(),
            use_case: use_case.into(),
            description: description.into(),
            message_flow: message_flow.into(),
            sample_messages,
            sub_use_cases: None,
            opt_in_keywords: None,
            opt_out_keywords: None,
            help_keywords: None,
            opt_in_message: None,
            opt_out_message: None,
            help_message: None,
            embedded_link: None,
            embedded_phone: None,
        }
    }

    pub fn sub_use_cases(mut self, sub_use_cases: Vec<String>) -> Self {
        self.sub_use_cases = Some(sub_use_cases);
        self
    }

    pub fn opt_in_keywords(mut self, opt_in_keywords: impl Into<String>) -> Self {
        self.opt_in_keywords = Some(opt_in_keywords.into());
        self
    }

    pub fn opt_out_keywords(mut self, opt_out_keywords: impl Into<String>) -> Self {
        self.opt_out_keywords = Some(opt_out_keywords.into());
        self
    }

    pub fn help_keywords(mut self, help_keywords: impl Into<String>) -> Self {
        self.help_keywords = Some(help_keywords.into());
        self
    }

    pub fn opt_in_message(mut self, opt_in_message: impl Into<String>) -> Self {
        self.opt_in_message = Some(opt_in_message.into());
        self
    }

    pub fn opt_out_message(mut self, opt_out_message: impl Into<String>) -> Self {
        self.opt_out_message = Some(opt_out_message.into());
        self
    }

    pub fn help_message(mut self, help_message: impl Into<String>) -> Self {
        self.help_message = Some(help_message.into());
        self
    }

    pub fn embedded_link(mut self, embedded_link: bool) -> Self {
        self.embedded_link = Some(embedded_link);
        self
    }

    pub fn embedded_phone(mut self, embedded_phone: bool) -> Self {
        self.embedded_phone = Some(embedded_phone);
        self
    }
}

/// A phone number assigned to a campaign.
#[derive(Debug, Clone, Deserialize)]
pub struct TenDlcAssignment {
    /// Unique assignment identifier.
    pub id: String,
    /// The campaign the number is assigned to.
    #[serde(alias = "campaignId")]
    pub campaign_id: String,
    /// The assigned phone number in E.164 format.
    #[serde(alias = "phoneNumber")]
    pub phone_number: String,
    /// Assignment status (`Active`, `Under review`, or `Action needed`);
    /// the number can send once `Active`.
    pub status: String,
    /// When the assignment completed (ISO 8601), or `None` while in progress.
    #[serde(default, alias = "assignedAt")]
    pub assigned_at: Option<String>,
}

/// Response from [`TenDlcResource::assign_number`].
#[derive(Debug, Clone, Deserialize)]
pub struct TenDlcAssignmentResponse {
    pub data: TenDlcAssignment,
}

/// Response from [`TenDlcResource::list_assignments`].
#[derive(Debug, Clone, Deserialize)]
pub struct TenDlcAssignmentListResponse {
    #[serde(default)]
    pub data: Vec<TenDlcAssignment>,
}

/// 10DLC resource — register your business for carrier review and text from
/// local US numbers.
///
/// # Example
///
/// ```rust,no_run
/// use sendly::{Sendly, CreateTenDlcBrandRequest, CreateTenDlcCampaignRequest};
///
/// # async fn run() -> Result<(), sendly::Error> {
/// let client = Sendly::new("sk_live_v1_xxx");
///
/// // 1) Register a brand and poll until it's verified
/// let brand = client
///     .ten_dlc()
///     .create_brand(
///         CreateTenDlcBrandRequest::new("Acme Holdings LLC")
///             .ein("12-3456789")
///             .website("https://acme.example")
///             .email("ops@acme.example"),
///     )
///     .await?;
/// // ...poll client.ten_dlc().get_brand(&brand.data.id) until status == "verified"
///
/// // 2) Pre-check the use case, then create a campaign
/// let check = client.ten_dlc().qualify(&brand.data.id, "MIXED").await?;
/// if check.data.qualified {
///     let campaign = client
///         .ten_dlc()
///         .create_campaign(CreateTenDlcCampaignRequest::new(
///             &brand.data.id,
///             "MIXED",
///             "Order updates and support replies for Acme customers",
///             "Customers opt in at checkout on acme.example",
///             vec!["Your order #123 has shipped!".to_string()],
///         ))
///         .await?;
///     // ...poll client.ten_dlc().get_campaign(&campaign.data.id) until status == "active"
///
///     // 3) Assign a number you own
///     client
///         .ten_dlc()
///         .assign_number(&campaign.data.id, "+15551234567")
///         .await?;
/// }
/// # Ok(()) }
/// ```
pub struct TenDlcResource<'a> {
    client: &'a Sendly,
}

impl<'a> TenDlcResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// List the brands registered for carrier review.
    pub async fn list_brands(&self) -> Result<TenDlcBrandListResponse> {
        let response = self.client.get("/tendlc/brands", &[]).await?;
        Ok(response.json().await?)
    }

    /// Register a brand for carrier review — step 1 of enabling local-number
    /// texting. Requires a live API key.
    ///
    /// The brand starts `pending`. Poll [`get_brand`](Self::get_brand) until
    /// it becomes `verified` before creating a campaign.
    pub async fn create_brand(
        &self,
        request: CreateTenDlcBrandRequest,
    ) -> Result<TenDlcBrandResponse> {
        let response = self.client.post("/tendlc/brands", &request).await?;
        Ok(response.json().await?)
    }

    /// Fetch one brand. Also refreshes its carrier-review status, so polling
    /// this method shows progress (`pending` → `verified`/`failed`).
    pub async fn get_brand(&self, id: &str) -> Result<TenDlcBrandResponse> {
        let response = self
            .client
            .get(&format!("/tendlc/brands/{}", id), &[])
            .await?;
        Ok(response.json().await?)
    }

    /// Pre-check whether a use case (e.g. `MIXED`, `MARKETING`) qualifies for
    /// a brand on the carrier network before creating a campaign.
    pub async fn qualify(&self, brand_id: &str, use_case: &str) -> Result<TenDlcQualifyResponse> {
        let response = self
            .client
            .get(
                &format!("/tendlc/brands/{}/qualify/{}", brand_id, use_case),
                &[],
            )
            .await?;
        Ok(response.json().await?)
    }

    /// List your messaging campaigns.
    pub async fn list_campaigns(&self) -> Result<TenDlcCampaignListResponse> {
        let response = self.client.get("/tendlc/campaigns", &[]).await?;
        Ok(response.json().await?)
    }

    /// Create a messaging campaign under a verified brand and submit it for
    /// carrier review. Requires a live API key.
    ///
    /// The campaign starts `pending`. Poll [`get_campaign`](Self::get_campaign)
    /// until it becomes `active` before assigning numbers.
    pub async fn create_campaign(
        &self,
        request: CreateTenDlcCampaignRequest,
    ) -> Result<TenDlcCampaignResponse> {
        let response = self.client.post("/tendlc/campaigns", &request).await?;
        Ok(response.json().await?)
    }

    /// Fetch one campaign. Also refreshes its carrier-review status, so
    /// polling this method shows progress (`pending` → `active`) including
    /// throughput once carriers approve.
    pub async fn get_campaign(&self, id: &str) -> Result<TenDlcCampaignResponse> {
        let response = self
            .client
            .get(&format!("/tendlc/campaigns/{}", id), &[])
            .await?;
        Ok(response.json().await?)
    }

    /// Assign a phone number you own to an active (carrier-approved)
    /// campaign, making the number sendable. Requires a live API key.
    ///
    /// Idempotent — re-assigning the same number to the same campaign returns
    /// the existing assignment.
    pub async fn assign_number(
        &self,
        campaign_id: &str,
        phone_number: &str,
    ) -> Result<TenDlcAssignmentResponse> {
        let response = self
            .client
            .post(
                &format!("/tendlc/campaigns/{}/assign", campaign_id),
                &serde_json::json!({ "phoneNumber": phone_number }),
            )
            .await?;
        Ok(response.json().await?)
    }

    /// List your number-to-campaign assignments.
    pub async fn list_assignments(&self) -> Result<TenDlcAssignmentListResponse> {
        let response = self.client.get("/tendlc/assignments", &[]).await?;
        Ok(response.json().await?)
    }
}
