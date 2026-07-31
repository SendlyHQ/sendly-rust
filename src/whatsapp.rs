//! WhatsApp Resource — Connect senders, manage templates, check windows
//!
//! WhatsApp is a first-class Sendly channel: connect a number you own,
//! create Meta-reviewed message templates, and send via
//! [`Messages::send_whatsapp`](crate::Messages::send_whatsapp).
//!
//! Connecting a number is a one-time $19 setup (no monthly fee) and always
//! ends with a human step: [`WhatsAppSignupResource::create`] returns a
//! `connect_url` that a person must open in a browser and log in with
//! Facebook to link their WhatsApp Business Account. Hand the URL to your
//! user — the connection cannot be completed programmatically.
//!
//! Two ways to reach a recipient:
//!
//! - **Inside a 24-hour window** (the recipient messaged you in the last
//!   24h): free-form text and media are allowed. Check with
//!   [`WhatsAppResource::window`].
//! - **Anytime**: an approved template. Templates are reviewed by Meta
//!   (typically 24-48h) and categorized as authentication, utility, or
//!   marketing — pricing follows the category and destination country.
//!   Note: Meta has paused marketing template delivery to US (+1) numbers.
//!
//! See <https://sendly.live/docs/whatsapp> for the full flow.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::client::Sendly;
use crate::error::Result;

/// Lifecycle status of a WhatsApp signup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WhatsAppSignupStatus {
    /// Created; waiting for a human to complete the connect URL.
    Initiated,
    /// The business account is linked; activation in progress.
    Registering,
    /// Connected; the number can send and receive on WhatsApp.
    Active,
    /// The connection failed (see `failure_reasons`).
    Failed,
    /// The connect URL expired before it was completed.
    Expired,
}

/// Response from [`WhatsAppSignupResource::create`].
///
/// Hand `connect_url` to a human — they open it in a browser and log in
/// with Facebook to link their WhatsApp Business Account. Poll
/// [`WhatsAppSignupResource::get`] with `id` until the status is `Active`.
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppSignupSession {
    /// Unique signup identifier — use with [`WhatsAppSignupResource::get`].
    pub id: String,
    /// Hosted connect page URL. A person must open this in a browser.
    #[serde(alias = "connectUrl")]
    pub connect_url: String,
    /// Current signup status.
    pub status: WhatsAppSignupStatus,
}

/// Response from [`WhatsAppSignupResource::get`].
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppSignup {
    /// Unique signup identifier.
    pub id: String,
    /// Current signup status.
    pub status: WhatsAppSignupStatus,
    /// The number being connected, in E.164 format.
    #[serde(alias = "phoneNumber")]
    pub phone_number: String,
    /// The customer's WhatsApp Business Account id, once linked; `None`
    /// before the human completes the connect step.
    #[serde(default, alias = "businessAccountId")]
    pub business_account_id: Option<String>,
    /// Why the signup failed, when status is `Failed`; `None` otherwise.
    #[serde(default, alias = "failureReasons")]
    pub failure_reasons: Option<Vec<String>>,
    /// ISO 8601 timestamp of the last status change.
    #[serde(alias = "updatedAt")]
    pub updated_at: String,
}

/// Connection status of a WhatsApp sender.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WhatsAppSenderStatus {
    /// The connection is still in progress; not sendable yet.
    Pending,
    /// Connected; the number can send and receive on WhatsApp.
    Active,
    /// Sending is currently suspended for this number.
    Suspended,
}

/// A number connected (or connecting) to WhatsApp.
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppSender {
    /// The sender, in E.164 format.
    #[serde(alias = "phoneNumber")]
    pub phone_number: String,
    /// The name recipients see — chosen during the connect flow and
    /// reviewed by Meta; `None` until set.
    #[serde(default, alias = "displayName")]
    pub display_name: Option<String>,
    /// Connection status.
    pub status: WhatsAppSenderStatus,
    /// Meta quality rating (e.g. "GREEN"), or `None` before first rating.
    #[serde(default, alias = "qualityRating")]
    pub quality_rating: Option<String>,
    /// ISO 8601 timestamp when the sender was connected.
    #[serde(alias = "createdAt")]
    pub created_at: String,
}

/// Response from [`WhatsAppSendersResource::list`].
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppSendersList {
    #[serde(default)]
    pub senders: Vec<WhatsAppSender>,
}

/// Template category. Meta reviews every template and may reclassify it —
/// the category on the record is authoritative and drives per-message
/// pricing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum WhatsAppTemplateCategory {
    Authentication,
    Utility,
    Marketing,
}

/// Review status of a template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum WhatsAppTemplateStatus {
    /// Submitted; Meta review usually takes 24-48h.
    Pending,
    /// Usable in template sends.
    Approved,
    /// Not usable; edit it with [`WhatsAppTemplatesResource::update`] to
    /// resubmit (template names are locked for ~30 days after deletion, so
    /// editing is the way out).
    Rejected,
    /// Quality-suspended by Meta.
    Paused,
    /// Quality-suspended by Meta.
    Disabled,
}

/// Button type on a template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WhatsAppTemplateButtonType {
    /// Link button; `url` is required and may contain a `{{1}}`
    /// placeholder (supply `example` values for review).
    Url,
    /// Tap-to-reply button (e.g. a "Stop promotions" opt-out, recommended
    /// on marketing templates).
    QuickReply,
    /// Copy-code button; required on AUTHENTICATION templates.
    Otp,
}

/// A button on a template.
#[derive(Debug, Clone, Serialize)]
pub struct WhatsAppTemplateButton {
    /// Button type.
    #[serde(rename = "type")]
    pub button_type: WhatsAppTemplateButtonType,
    /// Button label.
    pub text: String,
    /// Link target (url buttons only); may contain a `{{1}}` placeholder.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Example values for a url placeholder, for Meta review.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<Vec<String>>,
}

/// Request body for [`WhatsAppTemplatesResource::create`].
#[derive(Debug, Clone, Serialize)]
pub struct CreateWhatsAppTemplateRequest {
    /// The WhatsApp-connected sending number this template belongs to, in
    /// E.164 format.
    pub sender: String,
    /// Template name: lowercase letters, digits, and underscores (e.g.
    /// "order_shipped").
    pub name: String,
    /// Template language code (e.g. "en_US").
    pub language: String,
    /// Template category — drives Meta review rules and pricing.
    pub category: WhatsAppTemplateCategory,
    /// Body text. Use `{{1}}`, `{{2}}`, ... for variables; every
    /// placeholder needs an example value in `examples`.
    pub body: String,
    /// Optional footer line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<String>,
    /// Optional text header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    /// Optional buttons.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<WhatsAppTemplateButton>>,
    /// Example values for body placeholders, keyed by placeholder number:
    /// `{ "1": "Acme Inc", "2": "#4821" }`. Required when the body has
    /// variables — Meta reviews templates with these examples filled in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<HashMap<String, String>>,
}

impl CreateWhatsAppTemplateRequest {
    pub fn new(
        sender: impl Into<String>,
        name: impl Into<String>,
        language: impl Into<String>,
        category: WhatsAppTemplateCategory,
        body: impl Into<String>,
    ) -> Self {
        Self {
            sender: sender.into(),
            name: name.into(),
            language: language.into(),
            category,
            body: body.into(),
            footer: None,
            header: None,
            buttons: None,
            examples: None,
        }
    }

    pub fn footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    pub fn header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }

    pub fn buttons(mut self, buttons: Vec<WhatsAppTemplateButton>) -> Self {
        self.buttons = Some(buttons);
        self
    }

    pub fn examples(mut self, examples: HashMap<String, String>) -> Self {
        self.examples = Some(examples);
        self
    }
}

/// Request body for [`WhatsAppTemplatesResource::update`]. Supply only the
/// fields to change; omitted fields keep their current value.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateWhatsAppTemplateRequest {
    /// Replacement body text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Replacement footer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer: Option<String>,
    /// Replacement text header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<String>,
    /// Replacement buttons.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buttons: Option<Vec<WhatsAppTemplateButton>>,
    /// Replacement example values for body placeholders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<HashMap<String, String>>,
}

impl UpdateWhatsAppTemplateRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    pub fn footer(mut self, footer: impl Into<String>) -> Self {
        self.footer = Some(footer.into());
        self
    }

    pub fn header(mut self, header: impl Into<String>) -> Self {
        self.header = Some(header.into());
        self
    }

    pub fn buttons(mut self, buttons: Vec<WhatsAppTemplateButton>) -> Self {
        self.buttons = Some(buttons);
        self
    }

    pub fn examples(mut self, examples: HashMap<String, String>) -> Self {
        self.examples = Some(examples);
        self
    }
}

/// A WhatsApp message template.
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppTemplate {
    /// Unique template identifier.
    pub id: String,
    /// Template name.
    pub name: String,
    /// Template language code.
    pub language: String,
    /// Category (Meta may reclassify; this value drives pricing).
    pub category: WhatsAppTemplateCategory,
    /// Review status.
    pub status: WhatsAppTemplateStatus,
    /// Meta quality rating (e.g. "GREEN"), or `None` before first rating.
    #[serde(default, alias = "qualityRating")]
    pub quality_rating: Option<String>,
    /// Why Meta rejected the template, when status is `Rejected`.
    #[serde(default, alias = "rejectionReason")]
    pub rejection_reason: Option<String>,
    /// ISO 8601 timestamp when the template was created.
    #[serde(alias = "createdAt")]
    pub created_at: String,
    /// ISO 8601 timestamp when the template was last updated.
    #[serde(alias = "updatedAt")]
    pub updated_at: String,
    /// Non-blocking submission warnings (e.g. an unapproved display name,
    /// or a marketing template without an opt-out button). Present on
    /// create responses when applicable.
    #[serde(default)]
    pub warnings: Option<Vec<String>>,
}

/// Response from [`WhatsAppTemplatesResource::list`].
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppTemplateListResponse {
    #[serde(default)]
    pub templates: Vec<WhatsAppTemplate>,
}

/// Response from [`WhatsAppTemplatesResource::delete`].
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppTemplateDeletedResponse {
    /// The deleted template's id.
    pub id: String,
    /// Always true.
    pub deleted: bool,
}

/// Response from [`WhatsAppResource::window`].
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppWindow {
    /// True when a 24-hour customer-service window is currently open.
    pub open: bool,
    /// When the window closes (ISO 8601), or `None` when no window is open.
    #[serde(default, alias = "expiresAt")]
    pub expires_at: Option<String>,
}

/// Connect numbers to WhatsApp. Starting a signup returns a `connect_url`
/// a human must complete in a browser.
pub struct WhatsAppSignupResource<'a> {
    client: &'a Sendly,
}

impl<'a> WhatsAppSignupResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Start connecting a number to WhatsApp.
    ///
    /// Charges a one-time $19 setup fee (no monthly fee) and returns a
    /// `connect_url`. Completing the connection requires a human: hand the
    /// URL to your user — they open it in a browser and log in with
    /// Facebook to link their WhatsApp Business Account. Then poll
    /// [`get`](Self::get) until the status is `Active`.
    ///
    /// Calling again for a number with an in-flight signup returns the
    /// existing signup (same `connect_url`) without charging again.
    /// Requires a live API key.
    pub async fn create(&self, phone_number: &str) -> Result<WhatsAppSignupSession> {
        let response = self
            .client
            .post(
                "/whatsapp/signup",
                &serde_json::json!({ "phoneNumber": phone_number }),
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Get the status of a WhatsApp signup.
    pub async fn get(&self, id: &str) -> Result<WhatsAppSignup> {
        let response = self
            .client
            .get(&format!("/whatsapp/signup/{}", urlencoding::encode(id)), &[])
            .await?;
        Ok(response.json().await?)
    }
}

/// List the numbers connected (or connecting) to WhatsApp.
pub struct WhatsAppSendersResource<'a> {
    client: &'a Sendly,
}

impl<'a> WhatsAppSendersResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// List your WhatsApp senders.
    ///
    /// Returns the numbers connected (or connecting) to WhatsApp on your
    /// workspace, newest first. An empty list means no number is connected
    /// yet — start one with [`WhatsAppSignupResource::create`].
    pub async fn list(&self) -> Result<WhatsAppSendersList> {
        let response = self.client.get("/whatsapp/senders", &[]).await?;
        Ok(response.json().await?)
    }
}

/// Manage Meta-reviewed message templates.
pub struct WhatsAppTemplatesResource<'a> {
    client: &'a Sendly,
}

impl<'a> WhatsAppTemplatesResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// List your WhatsApp templates.
    pub async fn list(&self) -> Result<WhatsAppTemplateListResponse> {
        let response = self.client.get("/whatsapp/templates", &[]).await?;
        Ok(response.json().await?)
    }

    /// Create a template and submit it to Meta for review.
    ///
    /// Review usually takes 24-48h; the template is usable once its status
    /// is `Approved`. Requires a live API key.
    pub async fn create(&self, request: CreateWhatsAppTemplateRequest) -> Result<WhatsAppTemplate> {
        let response = self.client.post("/whatsapp/templates", &request).await?;
        Ok(response.json().await?)
    }

    /// Edit an APPROVED or REJECTED template and resubmit it for review.
    ///
    /// This is the recovery path for rejections: template names are locked
    /// for ~30 days after deletion, so editing a rejected template (rather
    /// than deleting and re-creating it) is the way to fix it. The updated
    /// template goes back to `Pending` review. Requires a live API key.
    pub async fn update(
        &self,
        id: &str,
        request: UpdateWhatsAppTemplateRequest,
    ) -> Result<WhatsAppTemplate> {
        let response = self
            .client
            .patch(
                &format!("/whatsapp/templates/{}", urlencoding::encode(id)),
                &request,
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Delete a template.
    ///
    /// Meta locks a deleted template's name for ~30 days — re-creating it
    /// fails with `template_name_locked` until the lock lifts. To fix a
    /// rejected template, prefer [`update`](Self::update). Requires a live
    /// API key.
    pub async fn delete(&self, id: &str) -> Result<WhatsAppTemplateDeletedResponse> {
        let response = self
            .client
            .delete(&format!("/whatsapp/templates/{}", urlencoding::encode(id)))
            .await?;
        Ok(response.json().await?)
    }
}

/// WhatsApp resource — connect senders, manage templates, and check
/// 24-hour windows.
///
/// # Example
///
/// ```rust,no_run
/// use sendly::{Sendly, CreateWhatsAppTemplateRequest, WhatsAppTemplateCategory};
/// use std::collections::HashMap;
///
/// # async fn run() -> Result<(), sendly::Error> {
/// let client = Sendly::new("sk_live_v1_xxx");
///
/// // 1) Connect a number ($19 one-time, no monthly fee). The connect URL
/// //    must be opened by a human — they log in with Facebook in a
/// //    browser to link their WhatsApp Business Account.
/// let signup = client.whatsapp().signup().create("+15559876543").await?;
/// println!("Have your user open: {}", signup.connect_url);
///
/// // 2) Poll until active
/// let status = client.whatsapp().signup().get(&signup.id).await?;
///
/// // 3) Create a template (Meta reviews it, usually 24-48h)
/// let mut examples = HashMap::new();
/// examples.insert("1".to_string(), "Sam".to_string());
/// examples.insert("2".to_string(), "#4821".to_string());
/// client
///     .whatsapp()
///     .templates()
///     .create(
///         CreateWhatsAppTemplateRequest::new(
///             "+15559876543",
///             "order_shipped",
///             "en_US",
///             WhatsAppTemplateCategory::Utility,
///             "Hi {{1}}, your order {{2}} has shipped!",
///         )
///         .examples(examples),
///     )
///     .await?;
///
/// // 4) Send — free-form inside an open 24h window, template anytime
/// let window = client.whatsapp().window("+15559876543", "+15551234567").await?;
/// if !window.open {
///     // Use a template send instead of free-form text
/// }
/// # Ok(()) }
/// ```
pub struct WhatsAppResource<'a> {
    client: &'a Sendly,
}

impl<'a> WhatsAppResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Returns the signup sub-resource.
    pub fn signup(&self) -> WhatsAppSignupResource<'a> {
        WhatsAppSignupResource::new(self.client)
    }

    /// Returns the senders sub-resource.
    pub fn senders(&self) -> WhatsAppSendersResource<'a> {
        WhatsAppSendersResource::new(self.client)
    }

    /// Returns the templates sub-resource.
    pub fn templates(&self) -> WhatsAppTemplatesResource<'a> {
        WhatsAppTemplatesResource::new(self.client)
    }

    /// Check whether a 24-hour customer-service window is open between one
    /// of your WhatsApp senders and a recipient.
    ///
    /// Free-form text and media only deliver while a window is open (it
    /// opens when the recipient messages you and lasts 24h from their last
    /// inbound message). Outside a window, send an approved template.
    pub async fn window(&self, from: &str, to: &str) -> Result<WhatsAppWindow> {
        let response = self
            .client
            .get(
                "/whatsapp/window",
                &[
                    ("from".to_string(), from.to_string()),
                    ("to".to_string(), to.to_string()),
                ],
            )
            .await?;
        Ok(response.json().await?)
    }
}
