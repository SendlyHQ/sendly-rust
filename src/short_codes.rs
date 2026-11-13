//! Short Codes Resource: apply for a US short code and track the application
//!
//! A short code is a 5 or 6 digit US sender. Getting one is an application,
//! not a purchase: you fill it in, Sendly reviews it, the carrier forms are
//! signed, Sendly files it, and each carrier certifies the code separately.
//!
//! The application lives at [`ShortCodeApplicationResource`]; every
//! application and code is listed with [`ShortCodesResource::list`]. Reads
//! need the `short_codes:read` scope and writes `short_codes:write`. While
//! short codes are not enabled for the account every call answers 404
//! `short_codes_not_enabled` ([`Error::NotFound`](crate::Error::NotFound)).
//!
//! Statuses, carrier names and charge kinds are plain strings, so a value
//! the API adds later still decodes.

use serde::{Deserialize, Serialize};

use crate::client::Sendly;
use crate::error::Result;
use crate::models::IdempotentRequestOptions;

/// A compliance problem with a short code application, by field.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeIssue {
    /// The field the issue is about, such as `helpResponse` or `documents.loa`.
    pub path: String,
    /// What to change, for people.
    pub message: String,
}

/// A carrier form the application needs, and where it stands.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeDocument {
    /// The form: `order_brief`, `brand_registration`,
    /// `content_provider_registration`, `migration_letter` or `loa`.
    pub kind: String,
    /// Whether Sendly has prepared the form for signing.
    pub template_ready: bool,
    /// When the form was signed.
    pub signed_at: Option<String>,
    /// Who signed it.
    pub signed_by_name: Option<String>,
    /// Whether a signing link is out and has not expired.
    pub awaiting_signature: bool,
}

/// The short code application a workspace is filling in, or has filed.
///
/// With no application yet this is the empty shape: `id` is `None`, the
/// choices show their defaults, and every answer is `None`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeApplication {
    /// The application's id. `None` before the first save.
    pub id: Option<String>,
    /// The assigned code's digits. `None` until Sendly assigns one.
    pub short_code: Option<String>,
    /// Always `US` today.
    pub country_code: String,
    /// Where the code is in provisioning: `requested`, `provisioning`,
    /// `parked`, `active`, `suspended` or `cancelled`. Only `active` can send.
    pub status: String,
    /// Where Sendly's review stands: `draft`, `awaiting_review`,
    /// `changes_requested`, `approved_for_filing`, `filed` or `rejected`.
    pub review_status: String,
    /// Sendly's note from the review, such as what to change.
    pub review_note: Option<String>,
    /// `new` for a new code, `migration` to move one from another provider.
    pub order_type: String,
    /// `random` for any available code, `vanity` for chosen digits.
    pub code_type: String,
    /// Whether the content provider is the brand itself.
    pub content_provider_same_as_brand: bool,
    /// Example messages.
    pub sample_messages: Vec<String>,
    /// What the code will be used for.
    pub use_case: Option<String>,
    /// How subscribers opt in.
    pub opt_in_flow: Option<String>,
    /// The message sent when someone opts in.
    pub opt_in_confirmation: Option<String>,
    /// The reply to HELP.
    pub help_response: Option<String>,
    /// The reply to STOP.
    pub stop_confirmation: Option<String>,
    /// How often subscribers get messages, such as `4 messages per month`.
    pub message_frequency: Option<String>,
    /// The keyword subscribers text to join.
    pub campaign_keyword: Option<String>,
    /// Expected messages a month.
    pub expected_monthly_volume: Option<String>,
    /// Expected messages a day.
    pub expected_daily_volume: Option<String>,
    /// The privacy policy URL.
    pub privacy_policy_url: Option<String>,
    /// The terms and conditions URL.
    pub terms_url: Option<String>,
    /// The digits asked for: a vanity code, or the code being migrated.
    pub requested_digits: Option<String>,
    /// The provider a migrated code is moving from.
    pub losing_provider: Option<String>,
    /// The brand's contact for the Short Code Registry.
    pub brand_contact_name: Option<String>,
    /// The brand contact's email.
    pub brand_contact_email: Option<String>,
    /// The brand contact's phone number.
    pub brand_contact_phone: Option<String>,
    /// The content provider's legal name.
    pub content_provider_legal_name: Option<String>,
    /// The content provider's EIN.
    pub content_provider_ein: Option<String>,
    /// The content provider's contact.
    pub content_provider_contact_name: Option<String>,
    /// The content provider contact's email.
    pub content_provider_contact_email: Option<String>,
    /// The content provider contact's phone number.
    pub content_provider_contact_phone: Option<String>,
    /// Where the brand's Short Code Registry registration stands.
    pub brand_registration_status: String,
    /// Where the content provider's Short Code Registry registration stands.
    pub content_provider_registration_status: String,
    /// When the application was submitted for review.
    pub submitted_at: Option<String>,
    /// When Sendly finished reviewing it.
    pub reviewed_at: Option<String>,
    /// When it was filed with the carriers.
    pub filed_at: Option<String>,
    /// When the code went live.
    pub activated_at: Option<String>,
    /// When the registry registration is next due to be re-vetted.
    pub registry_revetting_due_at: Option<String>,
    /// When the application was created.
    pub created_at: Option<String>,
    /// When the application last changed.
    pub updated_at: Option<String>,
}

/// The business address in [`ShortCodePrefill`]. Only the known parts are set.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodePrefillAddress {
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: Option<String>,
}

/// The business contact in [`ShortCodePrefill`]. Only the known parts are set.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodePrefillContact {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

/// Suggestions from what the workspace has already told Sendly: its 10DLC
/// brand or its toll-free verification. Never saved on the application
/// unless sent back with a save.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodePrefill {
    /// `tendlc`, `verification` or `none`.
    pub source: String,
    pub legal_name: Option<String>,
    pub dba: Option<String>,
    pub ein: Option<String>,
    pub website_url: Option<String>,
    pub address: Option<ShortCodePrefillAddress>,
    pub contact: Option<ShortCodePrefillContact>,
    /// From the toll-free verification.
    pub privacy_policy_url: Option<String>,
    /// From the toll-free verification.
    pub terms_url: Option<String>,
    /// Whether the toll-free verification uses Sendly's hosted legal pages.
    pub uses_hosted_legal_pages: Option<bool>,
}

/// Wording Sendly suggested for the application. Submit refuses it with
/// `short_code_ai_draft_unedited` until a person has edited it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeGeneratedDefaults {
    pub use_case: Option<String>,
    pub opt_in_flow: Option<String>,
}

/// The quoted price of a short code.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeQuote {
    /// `random` or `vanity`.
    pub code_type: String,
    /// The monthly lease, in US dollars.
    pub monthly_usd: i64,
    /// The one-time setup fee, in US dollars.
    pub setup_usd: i64,
    /// Always `USD`.
    pub currency: String,
    /// Always `false`, kept for older clients: `billing` says what is charged.
    pub auto_billed: bool,
    /// The one-time setup fee, in US cents, charged at submit.
    pub setup_cents: i64,
    /// The monthly lease, in US cents, fixed when the application is submitted.
    pub monthly_cents: i64,
    /// The lease runs at least this many months from go-live.
    pub minimum_term_months: i64,
    /// Always `submit`.
    pub setup_charged_at: String,
    /// Always `go_live`.
    pub lease_starts_at: String,
}

/// Where the setup fee stands on a [`ShortCodeBilling`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeSetupFee {
    /// The setup fee, in US cents.
    pub amount_cents: i64,
    /// `unpaid`, `processing`, `requires_action`, `failed`,
    /// `no_payment_method`, `paid`, `waived`, `refund_pending` or `refunded`.
    pub status: String,
    /// Always `submit`. Absent on a listed code.
    pub charged_at: Option<String>,
    /// When the fee was paid.
    pub paid_at: Option<String>,
    pub refunded_cents: i64,
    /// When the fee was refunded.
    pub refunded_at: Option<String>,
}

/// The oldest unpaid lease month.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeLeasePastDue {
    pub charge_id: String,
    pub amount_cents: i64,
    pub status: String,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub first_failed_at: Option<String>,
    /// When sending from the code pauses if the month is still unpaid.
    pub pause_at: Option<String>,
    pub notice_stage: Option<String>,
}

/// The monthly lease of a short code.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeLease {
    /// The monthly lease, in US cents.
    pub monthly_cents: i64,
    pub minimum_term_months: i64,
    /// Always `go_live` on the application; absent on a listed code.
    pub starts_at: Option<String>,
    /// `not_started`, `running`, `past_due`, `paused`, `ending` or `ended`.
    pub state: String,
    /// Whether Sendly bills the lease yet.
    pub billing_enabled: bool,
    pub started_at: Option<String>,
    pub next_charge_at: Option<String>,
    pub paid_through: Option<String>,
    pub term_ends_at: Option<String>,
    /// When a cancelled lease ends.
    pub ends_at: Option<String>,
    pub cancel_requested_at: Option<String>,
    /// `None` when nothing is unpaid.
    pub past_due: Option<ShortCodeLeasePastDue>,
    /// Months used but not yet charged.
    pub due_months: i64,
    /// What those months cost, in US cents.
    pub due_cents: i64,
}

/// One short code charge: the setup fee or a lease month.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeCharge {
    pub id: String,
    /// `setup` or `lease`.
    pub kind: String,
    pub amount_cents: i64,
    /// `paid`, `processing`, `failed`, `waived`, `refunded` or `not_charged`.
    pub status: String,
    pub paid_at: Option<String>,
    pub refunded_cents: i64,
    pub refunded_at: Option<String>,
    pub period_start: Option<String>,
    pub period_end: Option<String>,
    pub created_at: Option<String>,
}

/// The terms on a [`ShortCodeBilling`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeTerms {
    /// The current terms.
    pub version: String,
    /// The terms accepted at submit.
    pub accepted_version: Option<String>,
    pub accepted_at: Option<String>,
}

/// Where the money stands on a short code application.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeBilling {
    pub setup_fee: ShortCodeSetupFee,
    pub lease: ShortCodeLease,
    pub terms: ShortCodeTerms,
    /// Always `full_refund_before_filing`.
    pub refund_policy: String,
    /// `mock` or `enterprise` when the workspace isn't charged by card.
    pub exempt: Option<String>,
    pub charges: Vec<ShortCodeCharge>,
}

/// Where a listed short code's setup fee and lease stand.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeListBilling {
    pub setup_fee: ShortCodeSetupFee,
    pub lease: ShortCodeLease,
}

/// Whether the workspace's business is verified, which submitting needs.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeVerification {
    pub verified: bool,
    /// `verified`, `in_review`, `needs_attention` or `not_started`.
    pub state: String,
    /// The dashboard page for the verification.
    pub href: String,
}

/// One carrier's certification of the code.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeCarrierApproval {
    /// `ATT`, `TMOBILE`, `VERIZON` or `US_CELLULAR`.
    pub carrier: String,
    /// The carrier's name, such as `T-Mobile`.
    pub label: String,
    /// `not_submitted`, `submitted`, `in_review`, `approved` or `rejected`.
    pub status: String,
    pub updated_at: Option<String>,
}

/// Where each carrier's certification stands.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeCarriers {
    pub approved: i64,
    pub total: i64,
    /// `not_submitted`, `in_progress`, `approved` or `rejected`.
    pub overall: String,
    pub by_carrier: Vec<ShortCodeCarrierApproval>,
}

/// The setup fee's status and whether a submit charged it.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodePayment {
    pub status: String,
    /// Whether this call charged the card.
    pub charged: bool,
}

/// The application plus everything needed to act on it, from
/// [`ShortCodeApplicationResource::get`], `update` and `submit`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeApplicationView {
    pub enabled: bool,
    pub application: ShortCodeApplication,
    /// Suggestions from the workspace's business details.
    pub prefill: ShortCodePrefill,
    /// Whether a save can change the application.
    pub editable: bool,
    /// Why the application cannot be edited. `None` while it can.
    pub lock_message: Option<String>,
    /// Whether the application is finished (rejected or cancelled), so the
    /// next save starts a new one.
    pub can_start_new_application: bool,
    /// Wording Sendly suggested. `None` when nothing was suggested.
    pub generated_defaults: Option<ShortCodeGeneratedDefaults>,
    /// The forms this order type needs.
    pub required_documents: Vec<String>,
    /// The required forms not signed yet.
    pub missing_documents: Vec<String>,
    pub documents: Vec<ShortCodeDocument>,
    pub quote: ShortCodeQuote,
    pub billing: ShortCodeBilling,
    pub verification: ShortCodeVerification,
    pub carriers: ShortCodeCarriers,
    /// On submit: always `Some(true)`.
    pub submitted: Option<bool>,
    /// On submit: `Some(true)` when it was already with Sendly, and nothing
    /// was charged.
    pub already_submitted: Option<bool>,
    /// On submit: whether Sendly's review team was told.
    pub notified: Option<bool>,
    /// On submit: the setup fee's status and whether this call charged it.
    pub payment: Option<ShortCodePayment>,
    /// On a save: fields Sendly or the carriers own, which were ignored.
    pub ignored_fields: Option<Vec<String>>,
}

/// What a dry run of the application says, from
/// [`ShortCodeApplicationResource::check`]. Nothing is saved.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodePreflight {
    /// Whether the application can be submitted as it is.
    pub ok: bool,
    /// Every issue found, all at once.
    pub issues: Vec<ShortCodeIssue>,
    pub required_documents: Vec<String>,
    pub missing_documents: Vec<String>,
    pub documents: Vec<ShortCodeDocument>,
    /// Fields Sendly or the carriers own, which were ignored.
    pub ignored_fields: Option<Vec<String>>,
}

/// One of the workspace's short code applications, with its code once
/// Sendly assigns one.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCode {
    /// The application's id.
    pub id: String,
    /// The code's digits. `None` until Sendly assigns a code.
    pub short_code: Option<String>,
    pub country_code: String,
    /// Only `active` can send.
    pub status: String,
    pub review_status: String,
    pub use_case: String,
    pub created_at: String,
    pub billing: ShortCodeListBilling,
}

/// Every short code application of the workspace, from
/// [`ShortCodesResource::list`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct ShortCodeList {
    pub short_codes: Vec<ShortCode>,
}

/// Answers to save on the short code application, for
/// [`ShortCodeApplicationResource::update`] and
/// [`ShortCodeApplicationResource::check`].
///
/// Only the fields you set are sent; a field left out keeps its saved
/// value, and a blank string clears it. Strings are trimmed. Build it with
/// [`ShortCodeApplicationInput::new`] and the setters.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ShortCodeApplicationInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_case: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_in_flow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_in_confirmation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_response: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_confirmation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_frequency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub campaign_keyword: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_monthly_volume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_daily_volume: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_policy_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_digits: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub losing_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_contact_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_contact_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_contact_phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_provider_same_as_brand: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_provider_legal_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_provider_ein: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_provider_contact_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_provider_contact_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_provider_contact_phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sample_messages: Option<Vec<String>>,
}

macro_rules! string_setters {
    ($($(#[$doc:meta])* $name:ident),* $(,)?) => {
        $(
            $(#[$doc])*
            pub fn $name(mut self, value: impl Into<String>) -> Self {
                self.$name = Some(value.into());
                self
            }
        )*
    };
}

impl ShortCodeApplicationInput {
    /// Creates an empty input: nothing is sent, so nothing changes.
    pub fn new() -> Self {
        Self::default()
    }

    string_setters!(
        /// What the code will be used for.
        use_case,
        /// How subscribers opt in.
        opt_in_flow,
        /// The message sent when someone opts in.
        opt_in_confirmation,
        /// The reply to HELP: the program name, a help contact,
        /// `Msg&data rates may apply`, the message frequency and how to stop.
        help_response,
        /// The reply to STOP.
        stop_confirmation,
        /// Specific, such as `4 messages per month`, or `Message frequency varies`.
        message_frequency,
        /// The join keyword: 2 to 20 letters or numbers, not a reserved word.
        campaign_keyword,
        /// Expected messages a month.
        expected_monthly_volume,
        /// Expected messages a day.
        expected_daily_volume,
        /// The privacy policy URL.
        privacy_policy_url,
        /// The terms and conditions URL.
        terms_url,
        /// Only `US`.
        country_code,
        /// `new` or `migration`; any other value answers 422.
        order_type,
        /// `random` or `vanity`; any other value answers 422.
        code_type,
        /// 5 or 6 digits: the vanity code, or the code being migrated.
        requested_digits,
        /// For a migration, the provider the code is moving from.
        losing_provider,
        /// The brand's contact for the Short Code Registry.
        brand_contact_name,
        /// The brand contact's email.
        brand_contact_email,
        /// The brand contact's phone number.
        brand_contact_phone,
        /// The content provider's legal name, when it is not the brand.
        content_provider_legal_name,
        /// The content provider's EIN.
        content_provider_ein,
        /// The content provider's contact.
        content_provider_contact_name,
        /// The content provider contact's email.
        content_provider_contact_email,
        /// The content provider contact's phone number.
        content_provider_contact_phone,
    );

    /// Whether the content provider is the brand itself.
    pub fn content_provider_same_as_brand(mut self, value: bool) -> Self {
        self.content_provider_same_as_brand = Some(value);
        self
    }

    /// Real examples of the messages. An empty list clears them.
    pub fn sample_messages<I, S>(mut self, messages: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.sample_messages = Some(messages.into_iter().map(Into::into).collect());
        self
    }
}

/// The body of [`ShortCodeApplicationResource::submit`]: optional final
/// answers, saved before the check, and acceptance of the price.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SubmitShortCodeApplicationRequest {
    #[serde(flatten)]
    pub application: ShortCodeApplicationInput,
    /// Accepts the $999 setup fee, charged to the workspace's card when you
    /// submit, the monthly lease from go-live and the 3-month minimum.
    /// Submit is refused with `short_code_invalid_application` until this
    /// is `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept_terms: Option<bool>,
}

impl SubmitShortCodeApplicationRequest {
    /// Creates a request with no answers and the terms not yet accepted.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets `acceptTerms`.
    pub fn accept_terms(mut self, accept: bool) -> Self {
        self.accept_terms = Some(accept);
        self
    }

    /// Final answers to save before the submit.
    pub fn application(mut self, application: ShortCodeApplicationInput) -> Self {
        self.application = application;
        self
    }
}

/// The workspace's short code application.
pub struct ShortCodeApplicationResource<'a> {
    client: &'a Sendly,
}

impl<'a> ShortCodeApplicationResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Fetch the workspace's short code application, with the documents it
    /// needs, the quote, and `billing`: where the setup fee and the monthly
    /// lease stand. Requires the `short_codes:read` scope.
    pub async fn get(&self) -> Result<ShortCodeApplicationView> {
        let response = self.client.get("/short_codes/application", &[]).await?;
        Ok(response.json().await?)
    }

    /// Save answers on the application, creating it on the first call.
    ///
    /// Fields Sendly or the carriers own are ignored and named back in
    /// `ignored_fields`. Once the application is with Sendly or the
    /// carriers this answers 409 `short_code_locked`. Requires the
    /// `short_codes:write` scope.
    ///
    /// ```rust,no_run
    /// # use sendly::{Sendly, ShortCodeApplicationInput};
    /// # async fn run(client: Sendly) -> sendly::Result<()> {
    /// client
    ///     .short_codes()
    ///     .application()
    ///     .update(
    ///         ShortCodeApplicationInput::new()
    ///             .use_case("Delivery alerts for Acme orders")
    ///             .message_frequency("4 messages per month"),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(
        &self,
        application: ShortCodeApplicationInput,
    ) -> Result<ShortCodeApplicationView> {
        self.update_with_options(application, IdempotentRequestOptions::new())
            .await
    }

    /// [`update`](Self::update) with per-call options (e.g. your own
    /// idempotency key, so a repeat of the same save returns the first
    /// answer).
    pub async fn update_with_options(
        &self,
        application: ShortCodeApplicationInput,
        options: IdempotentRequestOptions,
    ) -> Result<ShortCodeApplicationView> {
        let response = self
            .client
            .put_with_idempotency(
                "/short_codes/application",
                &application,
                options.idempotency_key.as_deref(),
                true,
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Check the application against the carrier rules without changing
    /// anything. Unsaved answers can be passed in and are checked on top of
    /// what is saved; pass [`ShortCodeApplicationInput::new`] to check only
    /// what is saved. Requires the `short_codes:read` scope.
    pub async fn check(
        &self,
        application: ShortCodeApplicationInput,
    ) -> Result<ShortCodePreflight> {
        let response = self
            .client
            .post("/short_codes/application/preflight", &application)
            .await?;
        Ok(response.json().await?)
    }

    /// Submit the application to Sendly for review.
    ///
    /// Submitting charges the one-time $999 setup fee to the workspace's
    /// card on file, so set `accept_terms(true)` to accept it, the monthly
    /// lease from go-live and the 3-month minimum. The fee is refunded in
    /// full if Sendly rejects the application before filing it. Submitting
    /// twice is safe: the second call reports `already_submitted` and
    /// charges nothing. Requires the `short_codes:write` scope.
    ///
    /// Refusals: 403 `verification_required` until the business is verified
    /// (the answers sent are saved anyway); 422
    /// `short_code_invalid_application` or `short_code_ai_draft_unedited`
    /// with every issue in the body's `errors`; 402
    /// `payment_method_required`, `payment_failed` or
    /// `payment_requires_authentication` when the fee can't be charged.
    ///
    /// ```rust,no_run
    /// # use sendly::{Sendly, SubmitShortCodeApplicationRequest};
    /// # async fn run(client: Sendly) -> sendly::Result<()> {
    /// let result = client
    ///     .short_codes()
    ///     .application()
    ///     .submit(SubmitShortCodeApplicationRequest::new().accept_terms(true))
    ///     .await?;
    /// println!("{}", result.application.review_status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn submit(
        &self,
        request: SubmitShortCodeApplicationRequest,
    ) -> Result<ShortCodeApplicationView> {
        self.submit_with_options(request, IdempotentRequestOptions::new())
            .await
    }

    /// [`submit`](Self::submit) with per-call options (e.g. your own
    /// idempotency key, so a repeat of the same submit returns the first
    /// answer).
    pub async fn submit_with_options(
        &self,
        request: SubmitShortCodeApplicationRequest,
        options: IdempotentRequestOptions,
    ) -> Result<ShortCodeApplicationView> {
        let response = self
            .client
            .post_with_idempotency(
                "/short_codes/application/submit",
                &request,
                options.idempotency_key.as_deref(),
                true,
            )
            .await?;
        Ok(response.json().await?)
    }
}

/// US short codes: list them and manage the workspace's application.
pub struct ShortCodesResource<'a> {
    client: &'a Sendly,
}

impl<'a> ShortCodesResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// The workspace's short code application.
    pub fn application(&self) -> ShortCodeApplicationResource<'a> {
        ShortCodeApplicationResource::new(self.client)
    }

    /// List every short code application of the workspace, newest first,
    /// with its code once Sendly assigns one. Drafts (with `short_code`
    /// `None`), rejected and cancelled applications are listed too, so
    /// filter on `status == "active"` for the codes you can send from.
    /// Requires the `short_codes:read` scope.
    pub async fn list(&self) -> Result<ShortCodeList> {
        let response = self.client.get("/short_codes", &[]).await?;
        Ok(response.json().await?)
    }
}
