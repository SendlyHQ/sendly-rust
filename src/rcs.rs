//! RCS Resource — Brand and agent registration, agent discovery, and
//! recipient capability pre-flight
//!
//! RCS is the branded, rich upgrade to SMS: your verified agent name and
//! logo instead of a bare number, plus tappable suggestion chips and rich
//! cards, on Android and iOS 18+ handsets. Send with
//! [`Messages::send_rcs`](crate::Messages::send_rcs).
//!
//! Messages go out through an RCS agent — the verified sender identity
//! recipients see. Registration is self-serve, from the dashboard or this
//! API, and follows one path:
//!
//! 1. **Brand** — draft your business identity with
//!    [`RcsBrandsResource::create`] ([`RcsDossierResource::get`] prefills
//!    it from details already on file). US businesses only for now.
//! 2. **Agent** — draft the sender identity under that brand with
//!    [`RcsAgentsResource::create`]: name, use case, description, logo,
//!    hero, colour, and policy links.
//! 3. **Submit** — [`RcsAgentsResource::submit`] sends brand and agent to
//!    Sendly for review, then to the carrier network. Poll
//!    [`RcsAgentsResource::get`] or [`RcsRegistrationResource::get`] for
//!    the `customer_stage` as it moves through review.
//! 4. **Test** — once the stage is `testing`, invite your own devices with
//!    [`RcsAgentsResource::set_test_devices`] and fill in the campaign
//!    (message examples, consent) with [`RcsAgentsResource::update`].
//! 5. **Launch** — [`RcsAgentsResource::request_launch`] asks Sendly to
//!    launch the agent with the carrier network. Once the agent is
//!    `sendable`, no other setup is needed.
//!
//! Logo, hero, and call-to-action media must already be public `https://`
//! URLs; uploading assets is dashboard-only. Reads need the `rcs:read`
//! scope and writes `rcs:write`. Every registration write carries an
//! `Idempotency-Key` — generated per call, or your own through the
//! `*_with_options` variants. RCS is rolling out gradually: while it is
//! off for an account these endpoints answer 404 (`rcs_not_enabled`),
//! surfaced as [`Error::NotFound`](crate::Error::NotFound).
//!
//! Text sends fall back to plain SMS on their own when the recipient's
//! device or network doesn't support RCS, so the capability check here is
//! an optional pre-flight (useful for reporting reach, or for choosing
//! between a card and a text before sending). Sending and capability
//! checks require a live API key.
//!
//! See <https://sendly.live/docs/rcs> for the full flow.

use serde::{Deserialize, Serialize};

use crate::client::Sendly;
use crate::error::Result;
use crate::models::IdempotentRequestOptions;

/// Where a registration sits, in customer terms. Reported on brands and
/// agents as `customer_stage`, on [`RcsAgent`] as `stage`, and on
/// [`RcsRegistration`] / the review responses as `stage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RcsCustomerStage {
    /// Being filled in; nothing submitted yet.
    Draft,
    /// Submitted; Sendly is reviewing it.
    InReview,
    /// Sendly asked for changes; edit and resubmit (see `review_note`).
    ChangesRequested,
    /// Sendly declined the registration (see `review_note`).
    Rejected,
    /// Approved by Sendly; the carrier network is verifying the brand.
    BrandVerification,
    /// Brand verified; the carrier network is reviewing the agent.
    AgentReview,
    /// Approved for invited test devices; fill in the campaign, then
    /// request launch.
    Testing,
    /// Launch requested; Sendly is reviewing it.
    LaunchReview,
    /// Sendly asked the carrier network to launch the agent.
    Launching,
    /// The carrier network declined the launch (see `rejection_reason`).
    LaunchRejected,
    /// Launched; the agent can reach every RCS-capable recipient.
    Live,
    /// Sending is currently suspended.
    Suspended,
    /// Registration failed (see `rejection_reason`).
    Failed,
    /// A stage this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

impl RcsCustomerStage {
    /// The stage as the API spells it (e.g. `"in_review"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::InReview => "in_review",
            Self::ChangesRequested => "changes_requested",
            Self::Rejected => "rejected",
            Self::BrandVerification => "brand_verification",
            Self::AgentReview => "agent_review",
            Self::Testing => "testing",
            Self::LaunchReview => "launch_review",
            Self::Launching => "launching",
            Self::LaunchRejected => "launch_rejected",
            Self::Live => "live",
            Self::Suspended => "suspended",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for RcsCustomerStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Review status of a brand or agent, as Sendly tracks it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RcsReviewStatus {
    /// Editable; not submitted.
    Draft,
    /// Submitted; locked while Sendly reviews it.
    AwaitingReview,
    /// Editable again; see `review_note`.
    ChangesRequested,
    /// Approved by Sendly and sent to the carrier network.
    ApprovedForCarrier,
    /// Declined by Sendly; see `review_note`.
    Rejected,
    /// Launch requested; locked while Sendly reviews it.
    LaunchRequested,
    /// Launch sent to the carrier network.
    LaunchSubmitted,
    /// Launch declined by the carrier network; see `rejection_reason`.
    LaunchRejected,
    /// Registration failed; see `rejection_reason`.
    Failed,
    /// A status this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

impl RcsReviewStatus {
    /// The status as the API spells it (e.g. `"awaiting_review"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::AwaitingReview => "awaiting_review",
            Self::ChangesRequested => "changes_requested",
            Self::ApprovedForCarrier => "approved_for_carrier",
            Self::Rejected => "rejected",
            Self::LaunchRequested => "launch_requested",
            Self::LaunchSubmitted => "launch_submitted",
            Self::LaunchRejected => "launch_rejected",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for RcsReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An RCS agent registered for your brand.
#[derive(Debug, Clone, Deserialize)]
pub struct RcsAgent {
    /// Unique agent identifier — pass it as `agent_id` when sending.
    pub id: String,
    /// Agent name recipients see.
    pub name: String,
    /// Lifecycle status. `"testing"` and `"approved"` agents can send —
    /// a `"testing"` agent only reaches invited test numbers.
    pub status: String,
    /// The agent's declared use case; `None` when not set.
    #[serde(default, alias = "useCase")]
    pub use_case: Option<String>,
    /// True when the agent is fully provisioned and its status allows
    /// sending right now.
    #[serde(default)]
    pub sendable: bool,
    /// Where the registration sits, in customer terms; `None` on payloads
    /// that don't report it.
    #[serde(default)]
    pub stage: Option<RcsCustomerStage>,
    /// ISO 8601 timestamp when the agent was registered.
    #[serde(alias = "createdAt")]
    pub created_at: String,
}

/// Response from [`RcsAgentsResource::list`].
#[derive(Debug, Clone, Deserialize)]
pub struct RcsAgentsList {
    #[serde(default)]
    pub agents: Vec<RcsAgent>,
}

/// Response from [`RcsResource::capability`].
#[derive(Debug, Clone, Deserialize)]
pub struct RcsCapability {
    /// The number that was checked, in E.164 format.
    pub to: String,
    /// The agent the check ran against.
    #[serde(alias = "agentId")]
    pub agent_id: String,
    /// True when the recipient's device and network support RCS.
    #[serde(default)]
    pub capable: bool,
    /// RCS features the recipient supports (e.g. "RICHCARD_STANDALONE").
    #[serde(default)]
    pub features: Vec<String>,
}

/// Registered business address on a brand draft. `country_code` must be
/// `US` — RCS registration is available to US businesses for now.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RcsBrandAddressInput {
    /// Street address, first line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line1: Option<String>,
    /// Street address, second line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line2: Option<String>,
    /// City.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub city: Option<String>,
    /// State (two-letter code).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// ZIP / postal code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    /// ISO 3166-1 alpha-2 country code; must be `US`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
}

impl RcsBrandAddressInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn line1(mut self, line1: impl Into<String>) -> Self {
        self.line1 = Some(line1.into());
        self
    }

    pub fn line2(mut self, line2: impl Into<String>) -> Self {
        self.line2 = Some(line2.into());
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

    pub fn country_code(mut self, country_code: impl Into<String>) -> Self {
        self.country_code = Some(country_code.into());
        self
    }
}

/// Business contact on a brand draft — who the carrier network can reach
/// about the registration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RcsBrandContactInput {
    /// Contact's first name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_name: Option<String>,
    /// Contact's last name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    /// Contact's job title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Contact's email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Contact's phone number in E.164 format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<String>,
}

impl RcsBrandContactInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn first_name(mut self, first_name: impl Into<String>) -> Self {
        self.first_name = Some(first_name.into());
        self
    }

    pub fn last_name(mut self, last_name: impl Into<String>) -> Self {
        self.last_name = Some(last_name.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn phone_number(mut self, phone_number: impl Into<String>) -> Self {
        self.phone_number = Some(phone_number.into());
        self
    }
}

/// Brand fields for [`RcsBrandsResource::create`] and
/// [`RcsBrandsResource::update`]; also the shape of
/// [`RcsDossier::brand`], so a dossier can be passed straight to `create`.
///
/// Every field is optional while drafting — required-field checks run at
/// [`RcsAgentsResource::submit`], which reports each gap as a
/// `brand.<field>` entry in the error's `errors`. On update, only the
/// fields you set are changed; `None` leaves a field as it is, and
/// `address` / `contact` may be partial.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RcsBrandInput {
    /// The brand name recipients see.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Legal business name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legal_name: Option<String>,
    /// Legal structure: `LIMITED_LIABILITY_COMPANY`, `SOLE_PROPRIETORSHIP`,
    /// `PARTNERSHIP`, `CORPORATION`, or `S_CORPORATION`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legal_entity_type: Option<String>,
    /// Organization type: `PRIVATE_PROFIT`, `PUBLIC_PROFIT`, `NON_PROFIT`,
    /// `GOVERNMENT`, or `UNKNOWN`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_type: Option<String>,
    /// Business website (https).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website_url: Option<String>,
    /// Employer Identification Number (`123456789` or `12-3456789`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ein: Option<String>,
    /// Stock symbol as `EXCHANGE:TICKER`, for publicly traded businesses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stock_symbol: Option<String>,
    /// Registered business address; `country_code` must be `US`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<RcsBrandAddressInput>,
    /// Business contact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact: Option<RcsBrandContactInput>,
}

impl RcsBrandInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn legal_name(mut self, legal_name: impl Into<String>) -> Self {
        self.legal_name = Some(legal_name.into());
        self
    }

    pub fn legal_entity_type(mut self, legal_entity_type: impl Into<String>) -> Self {
        self.legal_entity_type = Some(legal_entity_type.into());
        self
    }

    pub fn organization_type(mut self, organization_type: impl Into<String>) -> Self {
        self.organization_type = Some(organization_type.into());
        self
    }

    pub fn website_url(mut self, website_url: impl Into<String>) -> Self {
        self.website_url = Some(website_url.into());
        self
    }

    pub fn ein(mut self, ein: impl Into<String>) -> Self {
        self.ein = Some(ein.into());
        self
    }

    pub fn stock_symbol(mut self, stock_symbol: impl Into<String>) -> Self {
        self.stock_symbol = Some(stock_symbol.into());
        self
    }

    pub fn address(mut self, address: RcsBrandAddressInput) -> Self {
        self.address = Some(address);
        self
    }

    pub fn contact(mut self, contact: RcsBrandContactInput) -> Self {
        self.contact = Some(contact);
        self
    }
}

/// Phone contact shown on the agent's info sheet.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RcsAgentPhoneContact {
    /// Phone number in E.164 format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// Label recipients see, e.g. "Call support".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl RcsAgentPhoneContact {
    pub fn new(number: impl Into<String>) -> Self {
        Self {
            number: Some(number.into()),
            label: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Website link shown on the agent's info sheet.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RcsAgentWebsiteContact {
    /// Website URL (https).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Label recipients see, e.g. "Visit our site".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl RcsAgentWebsiteContact {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: Some(url.into()),
            label: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Email contact shown on the agent's info sheet.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RcsAgentEmailContact {
    /// Email address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// Label recipients see, e.g. "Email us".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl RcsAgentEmailContact {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: Some(address.into()),
            label: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Agent identity — what recipients see when they open the agent.
///
/// `logo_url` and `hero_url` must be public `https://` URLs; uploading
/// assets is dashboard-only.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RcsAgentBasicsInput {
    /// The agent name recipients see.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Declared use case: `MULTI_USE`, `PROMOTIONAL`, `TRANSACTIONAL`, or
    /// `OTP`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_case: Option<String>,
    /// What the agent is for, shown on its info sheet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Public https:// URL of the agent's logo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    /// Public https:// URL of the agent's hero image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hero_url: Option<String>,
    /// Brand colour as `#RGB` or `#RRGGBB`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_color: Option<String>,
    /// Privacy policy URL (https).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_policy_url: Option<String>,
    /// Terms and conditions URL (https).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terms_and_conditions_url: Option<String>,
    /// Phone contact on the info sheet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone_number: Option<RcsAgentPhoneContact>,
    /// Website link on the info sheet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<RcsAgentWebsiteContact>,
    /// Email contact on the info sheet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<RcsAgentEmailContact>,
}

impl RcsAgentBasicsInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn use_case(mut self, use_case: impl Into<String>) -> Self {
        self.use_case = Some(use_case.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn logo_url(mut self, logo_url: impl Into<String>) -> Self {
        self.logo_url = Some(logo_url.into());
        self
    }

    pub fn hero_url(mut self, hero_url: impl Into<String>) -> Self {
        self.hero_url = Some(hero_url.into());
        self
    }

    pub fn brand_color(mut self, brand_color: impl Into<String>) -> Self {
        self.brand_color = Some(brand_color.into());
        self
    }

    pub fn privacy_policy_url(mut self, privacy_policy_url: impl Into<String>) -> Self {
        self.privacy_policy_url = Some(privacy_policy_url.into());
        self
    }

    pub fn terms_and_conditions_url(mut self, terms_and_conditions_url: impl Into<String>) -> Self {
        self.terms_and_conditions_url = Some(terms_and_conditions_url.into());
        self
    }

    pub fn phone_number(mut self, phone_number: RcsAgentPhoneContact) -> Self {
        self.phone_number = Some(phone_number);
        self
    }

    pub fn website(mut self, website: RcsAgentWebsiteContact) -> Self {
        self.website = Some(website);
        self
    }

    pub fn email(mut self, email: RcsAgentEmailContact) -> Self {
        self.email = Some(email);
        self
    }
}

/// One kind of conversation the agent has with recipients.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RcsInteraction {
    /// `TRANSACTIONAL_UPDATES`, `CUSTOMER_SUPPORT`, `LOYALTY_OR_REWARD`,
    /// `MARKETING_OR_PROMOTIONAL`, `ACCOUNT_ALERTS`, `TWO_WAY_CONVERSATION`,
    /// or `OTHER`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interaction_type: Option<String>,
    /// What that interaction looks like for your recipients.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl RcsInteraction {
    pub fn new(interaction_type: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            interaction_type: Some(interaction_type.into()),
            description: Some(description.into()),
        }
    }
}

/// One way recipients opt in to the agent's messages.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RcsOptInMethod {
    /// `SMS`, `WEBSITE`, `MOBILE_APP`, `QR_CODE`, `SALE_POINT`, or `OTHER`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method_type: Option<String>,
    /// How the opt-in works on that channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl RcsOptInMethod {
    pub fn new(method_type: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            method_type: Some(method_type.into()),
            description: Some(description.into()),
        }
    }
}

/// How recipients consent to messages, and the standard replies.
///
/// `call_to_action_media_url` must be a public `https://` URL; uploading
/// assets is dashboard-only.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RcsConsentSettings {
    /// Ways recipients opt in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_in_methods: Option<Vec<RcsOptInMethod>>,
    /// The call to action recipients see when opting in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_to_action: Option<String>,
    /// Where the opt-in call to action lives (https).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_to_action_url: Option<String>,
    /// Public https:// URL of a screenshot or image of the opt-in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_to_action_media_url: Option<String>,
    /// Whether recipients confirm their opt-in a second time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub double_opt_in: Option<bool>,
    /// The confirmation message, when `double_opt_in` is true.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub double_opt_in_message: Option<String>,
    /// Message sent when a recipient opts in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_in_message: Option<String>,
    /// Reply to a HELP request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help_response: Option<String>,
    /// Reply to a STOP request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opt_out_response: Option<String>,
}

impl RcsConsentSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn opt_in_methods(mut self, opt_in_methods: Vec<RcsOptInMethod>) -> Self {
        self.opt_in_methods = Some(opt_in_methods);
        self
    }

    pub fn call_to_action(mut self, call_to_action: impl Into<String>) -> Self {
        self.call_to_action = Some(call_to_action.into());
        self
    }

    pub fn call_to_action_url(mut self, call_to_action_url: impl Into<String>) -> Self {
        self.call_to_action_url = Some(call_to_action_url.into());
        self
    }

    pub fn call_to_action_media_url(mut self, call_to_action_media_url: impl Into<String>) -> Self {
        self.call_to_action_media_url = Some(call_to_action_media_url.into());
        self
    }

    pub fn double_opt_in(mut self, double_opt_in: bool) -> Self {
        self.double_opt_in = Some(double_opt_in);
        self
    }

    pub fn double_opt_in_message(mut self, double_opt_in_message: impl Into<String>) -> Self {
        self.double_opt_in_message = Some(double_opt_in_message.into());
        self
    }

    pub fn opt_in_message(mut self, opt_in_message: impl Into<String>) -> Self {
        self.opt_in_message = Some(opt_in_message.into());
        self
    }

    pub fn help_response(mut self, help_response: impl Into<String>) -> Self {
        self.help_response = Some(help_response.into());
        self
    }

    pub fn opt_out_response(mut self, opt_out_response: impl Into<String>) -> Self {
        self.opt_out_response = Some(opt_out_response.into());
        self
    }
}

/// Campaign section of an agent — what it sends and how recipients agreed
/// to it. Optional while drafting; required before
/// [`RcsAgentsResource::request_launch`] (an overview, at least one
/// interaction, at least three message examples, and consent settings).
/// Also the shape of [`RcsAgentDetail::campaign`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RcsCampaign {
    /// What your business does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_overview: Option<String>,
    /// What the agent sends and why.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_overview: Option<String>,
    /// Anything else reviewers should know.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_information: Option<String>,
    /// Kinds of conversations the agent has.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interactions: Option<Vec<RcsInteraction>>,
    /// Example messages the agent sends (at least three at launch).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_examples: Option<Vec<String>>,
    /// How recipients consent, and the standard replies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub consent_settings: Option<RcsConsentSettings>,
}

impl RcsCampaign {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn company_overview(mut self, company_overview: impl Into<String>) -> Self {
        self.company_overview = Some(company_overview.into());
        self
    }

    pub fn agent_overview(mut self, agent_overview: impl Into<String>) -> Self {
        self.agent_overview = Some(agent_overview.into());
        self
    }

    pub fn additional_information(mut self, additional_information: impl Into<String>) -> Self {
        self.additional_information = Some(additional_information.into());
        self
    }

    pub fn interactions(mut self, interactions: Vec<RcsInteraction>) -> Self {
        self.interactions = Some(interactions);
        self
    }

    pub fn message_examples(mut self, message_examples: Vec<String>) -> Self {
        self.message_examples = Some(message_examples);
        self
    }

    pub fn consent_settings(mut self, consent_settings: RcsConsentSettings) -> Self {
        self.consent_settings = Some(consent_settings);
        self
    }
}

/// Testing section of an agent — how reviewers can see the agent in
/// action before launch. Also the shape of [`RcsAgentDetail::testing`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RcsTesting {
    /// URL where reviewers can trigger a test message (required at launch).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_url: Option<String>,
    /// Identifier of a message sent to an invited test device.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    /// Anything else reviewers should know about testing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_information: Option<String>,
}

impl RcsTesting {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn test_url(mut self, test_url: impl Into<String>) -> Self {
        self.test_url = Some(test_url.into());
        self
    }

    pub fn message_id(mut self, message_id: impl Into<String>) -> Self {
        self.message_id = Some(message_id.into());
        self
    }

    pub fn additional_information(mut self, additional_information: impl Into<String>) -> Self {
        self.additional_information = Some(additional_information.into());
        self
    }
}

/// Request body for [`RcsAgentsResource::create`].
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRcsAgentRequest {
    /// The brand this agent belongs to (required).
    pub brand_id: String,
    /// The agent name recipients see; overrides `basics.display_name`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Declared use case (`MULTI_USE`, `PROMOTIONAL`, `TRANSACTIONAL`,
    /// `OTP`); overrides `basics.use_case`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_case: Option<String>,
    /// Agent identity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basics: Option<RcsAgentBasicsInput>,
    /// Campaign section; can be filled in later with
    /// [`RcsAgentsResource::update`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub campaign: Option<RcsCampaign>,
    /// Testing section; can be filled in later with
    /// [`RcsAgentsResource::update`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testing: Option<RcsTesting>,
}

impl CreateRcsAgentRequest {
    pub fn new(brand_id: impl Into<String>) -> Self {
        Self {
            brand_id: brand_id.into(),
            display_name: None,
            use_case: None,
            basics: None,
            campaign: None,
            testing: None,
        }
    }

    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn use_case(mut self, use_case: impl Into<String>) -> Self {
        self.use_case = Some(use_case.into());
        self
    }

    pub fn basics(mut self, basics: RcsAgentBasicsInput) -> Self {
        self.basics = Some(basics);
        self
    }

    pub fn campaign(mut self, campaign: RcsCampaign) -> Self {
        self.campaign = Some(campaign);
        self
    }

    pub fn testing(mut self, testing: RcsTesting) -> Self {
        self.testing = Some(testing);
        self
    }
}

/// Request body for [`RcsAgentsResource::update`]. Only the sections you
/// set are changed: `display_name`, `use_case`, and `basics` merge into
/// the agent identity; `campaign` and `testing` merge section-wise, and
/// [`clear_campaign`](Self::clear_campaign) /
/// [`clear_testing`](Self::clear_testing) send `null` to clear that
/// section.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRcsAgentRequest {
    /// The agent name recipients see.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    /// Declared use case (`MULTI_USE`, `PROMOTIONAL`, `TRANSACTIONAL`,
    /// `OTP`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_case: Option<String>,
    /// Agent identity fields to merge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basics: Option<RcsAgentBasicsInput>,
    /// `None` leaves the section alone; `Some(Some(_))` merges fields;
    /// `Some(None)` is sent as `null` and clears it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub campaign: Option<Option<RcsCampaign>>,
    /// `None` leaves the section alone; `Some(Some(_))` merges fields;
    /// `Some(None)` is sent as `null` and clears it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testing: Option<Option<RcsTesting>>,
}

impl UpdateRcsAgentRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn use_case(mut self, use_case: impl Into<String>) -> Self {
        self.use_case = Some(use_case.into());
        self
    }

    pub fn basics(mut self, basics: RcsAgentBasicsInput) -> Self {
        self.basics = Some(basics);
        self
    }

    /// Merges these campaign fields into the agent's campaign section.
    pub fn campaign(mut self, campaign: RcsCampaign) -> Self {
        self.campaign = Some(Some(campaign));
        self
    }

    /// Clears the campaign section.
    pub fn clear_campaign(mut self) -> Self {
        self.campaign = Some(None);
        self
    }

    /// Merges these testing fields into the agent's testing section.
    pub fn testing(mut self, testing: RcsTesting) -> Self {
        self.testing = Some(Some(testing));
        self
    }

    /// Clears the testing section.
    pub fn clear_testing(mut self) -> Self {
        self.testing = Some(None);
        self
    }
}

/// A device to invite for [`RcsAgentsResource::set_test_devices`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RcsTestDeviceInput {
    /// Device phone number in E.164 format (a formatted 10-digit US number
    /// is also accepted).
    pub phone_number: String,
    /// Friendly label, e.g. "Sam's Pixel".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl RcsTestDeviceInput {
    pub fn new(phone_number: impl Into<String>) -> Self {
        Self {
            phone_number: phone_number.into(),
            label: None,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Request body for [`RcsAgentsResource::request_launch`]. Both fields
/// are optional; when set they are saved to the agent's testing section
/// before the launch request is recorded.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RcsRequestLaunchRequest {
    /// URL where reviewers can trigger a test message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_url: Option<String>,
    /// Anything else reviewers should know about testing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testing_additional_information: Option<String>,
}

impl RcsRequestLaunchRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn test_url(mut self, test_url: impl Into<String>) -> Self {
        self.test_url = Some(test_url.into());
        self
    }

    pub fn testing_additional_information(
        mut self,
        testing_additional_information: impl Into<String>,
    ) -> Self {
        self.testing_additional_information = Some(testing_additional_information.into());
        self
    }
}

/// Registered business address on a brand.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RcsBrandAddress {
    /// Street address, first line (`""` while unset).
    pub line1: String,
    /// Street address, second line.
    pub line2: Option<String>,
    /// City (`""` while unset).
    pub city: String,
    /// State (`""` while unset).
    pub state: String,
    /// ZIP / postal code (`""` while unset).
    pub postal_code: String,
    /// ISO 3166-1 alpha-2 country code (`US`).
    pub country_code: String,
}

/// Business contact on a brand.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RcsBrandContact {
    /// Contact's first name (`""` while unset).
    pub first_name: String,
    /// Contact's last name (`""` while unset).
    pub last_name: String,
    /// Contact's job title.
    pub title: Option<String>,
    /// Contact's email address (`""` while unset).
    pub email: String,
    /// Contact's phone number in E.164 format (`""` while unset).
    pub phone_number: String,
}

/// A brand — the business identity an agent is registered under.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RcsBrand {
    /// Unique brand identifier.
    pub id: String,
    /// Review status, as Sendly tracks it.
    pub review_status: RcsReviewStatus,
    /// Where the brand sits, in customer terms.
    pub customer_stage: RcsCustomerStage,
    /// The brand name recipients see (`""` while unset).
    #[serde(default)]
    pub display_name: String,
    /// Legal business name (`""` while unset).
    #[serde(default)]
    pub legal_name: String,
    /// Legal structure of the business (`""` while unset).
    #[serde(default)]
    pub legal_entity_type: String,
    /// Organization type (`""` while unset).
    #[serde(default)]
    pub organization_type: String,
    /// Stock symbol as `EXCHANGE:TICKER`, for publicly traded businesses.
    #[serde(default)]
    pub stock_symbol: Option<String>,
    /// Business website (`""` while unset).
    #[serde(default)]
    pub website_url: String,
    /// Employer Identification Number (`""` while unset).
    #[serde(default)]
    pub ein: String,
    /// Registered business address.
    #[serde(default)]
    pub address: RcsBrandAddress,
    /// Business contact.
    #[serde(default)]
    pub contact: RcsBrandContact,
    /// Sendly's note from review, when changes were requested or the brand
    /// was declined.
    #[serde(default)]
    pub review_note: Option<String>,
    /// Why the carrier network declined the brand, when it did.
    #[serde(default)]
    pub rejection_reason: Option<String>,
    /// When the brand was submitted for review (ISO 8601), or `None`.
    #[serde(default)]
    pub submitted_for_review_at: Option<String>,
    /// When the brand was sent to the carrier network (ISO 8601), or `None`.
    #[serde(default)]
    pub sent_to_carrier_at: Option<String>,
    /// When the carrier network verified the brand (ISO 8601), or `None`.
    #[serde(default)]
    pub verified_at: Option<String>,
    /// When the brand was created (ISO 8601).
    pub created_at: String,
    /// When the brand was last updated (ISO 8601).
    pub updated_at: String,
}

/// A device invited to test an agent before launch.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RcsTestDevice {
    /// Unique device identifier.
    pub id: String,
    /// Device phone number in E.164 format.
    pub phone_number: String,
    /// Friendly label.
    #[serde(default)]
    pub label: Option<String>,
    /// Invite state reported by the carrier network (e.g. `PENDING`), or
    /// `None` until invited.
    #[serde(default)]
    pub invite_status: Option<String>,
    /// When the device was added (ISO 8601).
    pub created_at: String,
}

/// Agent identity as stored — [`RcsAgentBasicsInput`] with the
/// server-owned fields filled in. Optional fields are `None` until set.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RcsAgentBasics {
    /// The agent name recipients see (`""` while unset).
    pub display_name: String,
    /// Declared use case, or `None` when not set.
    pub use_case: Option<String>,
    /// Hosting region chosen by Sendly, or `None` until provisioned.
    pub hosting_region: Option<String>,
    /// What the agent is for, shown on its info sheet.
    pub description: Option<String>,
    /// Public https:// URL of the agent's logo.
    pub logo_url: Option<String>,
    /// Public https:// URL of the agent's hero image.
    pub hero_url: Option<String>,
    /// Brand colour as `#RGB` or `#RRGGBB`.
    pub brand_color: Option<String>,
    /// Privacy policy URL.
    pub privacy_policy_url: Option<String>,
    /// Terms and conditions URL.
    pub terms_and_conditions_url: Option<String>,
    /// Phone contact on the info sheet.
    pub phone_number: Option<RcsAgentPhoneContact>,
    /// Website link on the info sheet.
    pub website: Option<RcsAgentWebsiteContact>,
    /// Email contact on the info sheet.
    pub email: Option<RcsAgentEmailContact>,
}

/// The full agent record — identity, campaign, testing, review state, and
/// invited devices. [`RcsAgent`] is the lighter shape returned by
/// [`RcsAgentsResource::list`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RcsAgentDetail {
    /// Unique agent identifier — pass it as `agent_id` when sending.
    pub id: String,
    /// The brand this agent belongs to.
    #[serde(default)]
    pub brand_id: Option<String>,
    /// Lifecycle (send) status: `draft`, `submitted`, `testing`,
    /// `approved`, or `suspended`.
    pub status: String,
    /// Review status, as Sendly tracks it.
    pub review_status: RcsReviewStatus,
    /// Where the registration sits, in customer terms.
    pub customer_stage: RcsCustomerStage,
    /// The agent name recipients see (`""` while unset).
    #[serde(default)]
    pub display_name: String,
    /// Declared use case, or `None` when not set.
    #[serde(default)]
    pub use_case: Option<String>,
    /// Hosting region chosen by Sendly, or `None` until provisioned.
    #[serde(default)]
    pub hosting_region: Option<String>,
    /// Agent identity.
    #[serde(default)]
    pub basics: RcsAgentBasics,
    /// Campaign section, or `None` until filled in.
    #[serde(default)]
    pub campaign: Option<RcsCampaign>,
    /// Testing section, or `None` until filled in.
    #[serde(default)]
    pub testing: Option<RcsTesting>,
    /// Sendly's note from review, when changes were requested or the agent
    /// was declined.
    #[serde(default)]
    pub review_note: Option<String>,
    /// Why the carrier network declined the agent or its launch, when it
    /// did.
    #[serde(default)]
    pub rejection_reason: Option<String>,
    /// Devices invited to test the agent.
    #[serde(default)]
    pub test_devices: Vec<RcsTestDevice>,
    /// When the agent was submitted for review (ISO 8601), or `None`.
    #[serde(default)]
    pub submitted_for_review_at: Option<String>,
    /// When the agent identity was sent to the carrier network (ISO 8601),
    /// or `None`.
    #[serde(default)]
    pub basics_submitted_at: Option<String>,
    /// When the launch was sent to the carrier network (ISO 8601), or
    /// `None`.
    #[serde(default)]
    pub launch_submitted_at: Option<String>,
    /// When the agent went live (ISO 8601), or `None`.
    #[serde(default)]
    pub live_at: Option<String>,
    /// When the agent was created (ISO 8601).
    pub created_at: String,
    /// When the agent was last updated (ISO 8601).
    pub updated_at: String,
}

/// Response from [`RcsRegistrationResource::get`] — the workspace's
/// current registration at a glance.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RcsRegistration {
    /// The newest agent's brand, else the newest brand, or `None` when
    /// none exists.
    #[serde(default)]
    pub brand: Option<RcsBrand>,
    /// The newest agent, or `None` when none exists.
    #[serde(default)]
    pub agent: Option<RcsAgentDetail>,
    /// Devices invited to test that agent (empty when there is no agent).
    #[serde(default)]
    pub devices: Vec<RcsTestDevice>,
    /// Where the registration sits, in customer terms (`Draft` when
    /// nothing exists).
    pub stage: RcsCustomerStage,
    /// False when something on file names a non-US country.
    #[serde(default)]
    pub us_eligible: bool,
}

/// Response from [`RcsDossierResource::get`] — business details already
/// on file, shaped as an [`RcsBrandInput`] you can pass straight to
/// [`RcsBrandsResource::create`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RcsDossier {
    /// Prefilled brand fields (only the ones that have a value).
    #[serde(default)]
    pub brand: RcsBrandInput,
    /// False when something on file names a non-US country.
    #[serde(default)]
    pub us_eligible: bool,
    /// Where the details came from: `tendlc` (the workspace's newest 10DLC
    /// brand), `verification` (the active toll-free verification), or
    /// `none` (nothing on file; `brand` is empty).
    pub source: String,
}

/// Response from [`RcsBrandsResource::create`] and
/// [`RcsBrandsResource::update`].
#[derive(Debug, Clone, Deserialize)]
pub struct RcsBrandResponse {
    pub brand: RcsBrand,
}

/// Response from [`RcsAgentsResource::create`] and
/// [`RcsAgentsResource::update`].
#[derive(Debug, Clone, Deserialize)]
pub struct RcsAgentResponse {
    pub agent: RcsAgentDetail,
}

/// Response from [`RcsAgentsResource::get`].
#[derive(Debug, Clone, Deserialize)]
pub struct RcsAgentDetailResponse {
    /// The agent.
    pub agent: RcsAgentDetail,
    /// Devices invited to test the agent (same as `agent.test_devices`).
    #[serde(default)]
    pub devices: Vec<RcsTestDevice>,
    /// Where the registration sits (same as `agent.customer_stage`).
    pub stage: RcsCustomerStage,
}

/// Response from [`RcsAgentsResource::set_test_devices`].
#[derive(Debug, Clone, Deserialize)]
pub struct RcsTestDeviceListResponse {
    /// The full device list after the change.
    #[serde(default)]
    pub devices: Vec<RcsTestDevice>,
}

/// Response from [`RcsAgentsResource::submit`] and
/// [`RcsAgentsResource::request_launch`].
#[derive(Debug, Clone, Deserialize)]
pub struct RcsAgentReviewResponse {
    /// The agent, with its new review status.
    pub agent: RcsAgentDetail,
    /// Where the registration sits now.
    pub stage: RcsCustomerStage,
}

/// The workspace's registration at a glance.
pub struct RcsRegistrationResource<'a> {
    client: &'a Sendly,
}

impl<'a> RcsRegistrationResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Fetch the workspace's registration at a glance: the newest agent,
    /// its brand and test devices, and the overall `stage`.
    ///
    /// Requires the `rcs:read` scope. Answers 404 (`rcs_not_enabled`) while
    /// RCS registration isn't enabled for the account.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{RcsCustomerStage, Sendly};
    ///
    /// # async fn run() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    /// let registration = client.rcs().registration().get().await?;
    /// if registration.stage == RcsCustomerStage::Testing {
    ///     if let Some(agent) = &registration.agent {
    ///         println!("{} is ready to test", agent.display_name);
    ///     }
    /// }
    /// # Ok(()) }
    /// ```
    pub async fn get(&self) -> Result<RcsRegistration> {
        let response = self.client.get("/rcs/registration", &[]).await?;
        Ok(response.json().await?)
    }
}

/// Business details already on file, ready to prefill a brand.
pub struct RcsDossierResource<'a> {
    client: &'a Sendly,
}

impl<'a> RcsDossierResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Fetch business details already on file (from 10DLC or toll-free
    /// verification), shaped for [`RcsBrandsResource::create`].
    ///
    /// Requires the `rcs:read` scope.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn run() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    /// let dossier = client.rcs().dossier().get().await?;
    /// let brand = client
    ///     .rcs()
    ///     .brands()
    ///     .create(dossier.brand.display_name("Acme Coffee"))
    ///     .await?;
    /// println!("{}", brand.brand.id);
    /// # Ok(()) }
    /// ```
    pub async fn get(&self) -> Result<RcsDossier> {
        let response = self.client.get("/rcs/dossier", &[]).await?;
        Ok(response.json().await?)
    }
}

/// Draft and update the brand an agent is registered under.
pub struct RcsBrandsResource<'a> {
    client: &'a Sendly,
}

impl<'a> RcsBrandsResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Draft a brand — step 1 of registering for RCS. Requires the
    /// `rcs:write` scope.
    ///
    /// Every field is optional while drafting; required-field checks run
    /// at [`RcsAgentsResource::submit`]. `address.country_code` must be
    /// `US` (422 `rcs_us_only` otherwise) — RCS registration is available
    /// to US businesses for now.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{RcsBrandAddressInput, RcsBrandContactInput, RcsBrandInput, Sendly};
    ///
    /// # async fn run() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    /// let brand = client
    ///     .rcs()
    ///     .brands()
    ///     .create(
    ///         RcsBrandInput::new()
    ///             .display_name("Acme Coffee")
    ///             .legal_name("Acme Coffee LLC")
    ///             .legal_entity_type("LIMITED_LIABILITY_COMPANY")
    ///             .organization_type("PRIVATE_PROFIT")
    ///             .website_url("https://acme.example")
    ///             .ein("12-3456789")
    ///             .address(
    ///                 RcsBrandAddressInput::new()
    ///                     .line1("100 Main St")
    ///                     .city("Chicago")
    ///                     .state("IL")
    ///                     .postal_code("60601")
    ///                     .country_code("US"),
    ///             )
    ///             .contact(
    ///                 RcsBrandContactInput::new()
    ///                     .first_name("Sam")
    ///                     .last_name("Lee")
    ///                     .email("sam@acme.example")
    ///                     .phone_number("+13125550100"),
    ///             ),
    ///     )
    ///     .await?;
    /// println!("{} ({})", brand.brand.id, brand.brand.review_status);
    /// # Ok(()) }
    /// ```
    pub async fn create(&self, request: RcsBrandInput) -> Result<RcsBrandResponse> {
        self.create_with_options(request, IdempotentRequestOptions::new())
            .await
    }

    /// [`create`](Self::create) with per-call options (e.g. your own
    /// idempotency key).
    pub async fn create_with_options(
        &self,
        request: RcsBrandInput,
        options: IdempotentRequestOptions,
    ) -> Result<RcsBrandResponse> {
        let response = self
            .client
            .post_with_idempotency(
                "/rcs/brands",
                &request,
                options.idempotency_key.as_deref(),
                true,
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Update a brand draft. Requires the `rcs:write` scope.
    ///
    /// Only the fields you set are changed, and `address` / `contact` may
    /// be partial. A brand is locked (409 `rcs_field_locked`) while Sendly
    /// is reviewing it and once the carrier network has registered it.
    /// Field problems come back as 422 `rcs_invalid_content`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{RcsBrandContactInput, RcsBrandInput, Sendly};
    ///
    /// # async fn run() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    /// let brand = client
    ///     .rcs()
    ///     .brands()
    ///     .update(
    ///         "brd_xxx",
    ///         RcsBrandInput::new()
    ///             .website_url("https://acme.example")
    ///             .contact(RcsBrandContactInput::new().title("Head of Support")),
    ///     )
    ///     .await?;
    /// println!("{}", brand.brand.website_url);
    /// # Ok(()) }
    /// ```
    pub async fn update(&self, id: &str, request: RcsBrandInput) -> Result<RcsBrandResponse> {
        self.update_with_options(id, request, IdempotentRequestOptions::new())
            .await
    }

    /// [`update`](Self::update) with per-call options (e.g. your own
    /// idempotency key).
    pub async fn update_with_options(
        &self,
        id: &str,
        request: RcsBrandInput,
        options: IdempotentRequestOptions,
    ) -> Result<RcsBrandResponse> {
        let response = self
            .client
            .patch_with_idempotency(
                &format!("/rcs/brands/{}", id),
                &request,
                options.idempotency_key.as_deref(),
                true,
            )
            .await?;
        Ok(response.json().await?)
    }
}

/// List, draft, submit, test, and launch RCS agents.
pub struct RcsAgentsResource<'a> {
    client: &'a Sendly,
}

impl<'a> RcsAgentsResource<'a> {
    pub fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// List your RCS agents.
    ///
    /// Returns the agents on your workspace, newest first, with lifecycle
    /// status and sendability. An empty list means no agent is registered
    /// yet — draft one with [`create`](Self::create) or from the dashboard.
    pub async fn list(&self) -> Result<RcsAgentsList> {
        let response = self.client.get("/rcs/agents", &[]).await?;
        Ok(response.json().await?)
    }

    /// Draft an agent under a brand — step 2 of registering for RCS.
    /// Requires the `rcs:write` scope.
    ///
    /// `logo_url`, `hero_url`, and `call_to_action_media_url` must be
    /// public `https://` URLs (422 `rcs_invalid_content` otherwise);
    /// uploading assets is dashboard-only. The campaign and testing
    /// sections can be filled in later with [`update`](Self::update).
    /// Answers 404 `rcs_not_found` when the brand isn't in this workspace.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{CreateRcsAgentRequest, RcsAgentBasicsInput, RcsAgentWebsiteContact, Sendly};
    ///
    /// # async fn run() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    /// let agent = client
    ///     .rcs()
    ///     .agents()
    ///     .create(
    ///         CreateRcsAgentRequest::new("brd_xxx")
    ///             .display_name("Acme Coffee")
    ///             .use_case("MULTI_USE")
    ///             .basics(
    ///                 RcsAgentBasicsInput::new()
    ///                     .description("Order updates and support for Acme Coffee customers")
    ///                     .logo_url("https://acme.example/rcs/logo.png")
    ///                     .hero_url("https://acme.example/rcs/hero.png")
    ///                     .brand_color("#0B6E4F")
    ///                     .privacy_policy_url("https://acme.example/privacy")
    ///                     .terms_and_conditions_url("https://acme.example/terms")
    ///                     .website(
    ///                         RcsAgentWebsiteContact::new("https://acme.example")
    ///                             .label("Visit our site"),
    ///                     ),
    ///             ),
    ///     )
    ///     .await?;
    /// println!("{}", agent.agent.id);
    /// # Ok(()) }
    /// ```
    pub async fn create(&self, request: CreateRcsAgentRequest) -> Result<RcsAgentResponse> {
        self.create_with_options(request, IdempotentRequestOptions::new())
            .await
    }

    /// [`create`](Self::create) with per-call options (e.g. your own
    /// idempotency key).
    pub async fn create_with_options(
        &self,
        request: CreateRcsAgentRequest,
        options: IdempotentRequestOptions,
    ) -> Result<RcsAgentResponse> {
        let response = self
            .client
            .post_with_idempotency(
                "/rcs/agents",
                &request,
                options.idempotency_key.as_deref(),
                true,
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Fetch one agent with its review state and invited devices. Poll
    /// this to follow `customer_stage` through review, testing, and
    /// launch.
    ///
    /// Requires the `rcs:read` scope. Answers 404 `rcs_not_found` when the
    /// agent isn't in this workspace.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{RcsCustomerStage, Sendly};
    ///
    /// # async fn run() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    /// let detail = client.rcs().agents().get("rcs_agent_xxx").await?;
    /// if detail.stage == RcsCustomerStage::ChangesRequested {
    ///     println!("{:?}", detail.agent.review_note);
    /// }
    /// # Ok(()) }
    /// ```
    pub async fn get(&self, id: &str) -> Result<RcsAgentDetailResponse> {
        let response = self.client.get(&format!("/rcs/agents/{}", id), &[]).await?;
        Ok(response.json().await?)
    }

    /// Update an agent draft. Requires the `rcs:write` scope.
    ///
    /// Only the sections you set are changed: `display_name`, `use_case`,
    /// and `basics` merge into the identity; `campaign` and `testing`
    /// merge section-wise, and
    /// [`clear_campaign`](UpdateRcsAgentRequest::clear_campaign) /
    /// [`clear_testing`](UpdateRcsAgentRequest::clear_testing) clear
    /// that section. An agent is locked (409 `rcs_field_locked`) while
    /// Sendly is reviewing it; the identity locks once sent to the carrier
    /// network, and the campaign and testing sections lock once the
    /// launch is sent (unless it was declined). Field problems come back
    /// as 422 `rcs_invalid_content`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{
    ///     RcsCampaign, RcsConsentSettings, RcsInteraction, RcsOptInMethod, Sendly,
    ///     UpdateRcsAgentRequest,
    /// };
    ///
    /// # async fn run() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    /// let agent = client
    ///     .rcs()
    ///     .agents()
    ///     .update(
    ///         "rcs_agent_xxx",
    ///         UpdateRcsAgentRequest::new().campaign(
    ///             RcsCampaign::new()
    ///                 .agent_overview("Order confirmations, pickup alerts, and support replies")
    ///                 .interactions(vec![RcsInteraction::new(
    ///                     "TRANSACTIONAL_UPDATES",
    ///                     "Order status",
    ///                 )])
    ///                 .message_examples(vec![
    ///                     "Your order #4821 is being roasted.".to_string(),
    ///                     "Your order #4821 is ready for pickup!".to_string(),
    ///                     "Thanks for visiting — reply HELP for support.".to_string(),
    ///                 ])
    ///                 .consent_settings(
    ///                     RcsConsentSettings::new()
    ///                         .opt_in_methods(vec![RcsOptInMethod::new(
    ///                             "WEBSITE",
    ///                             "Checkout checkbox",
    ///                         )])
    ///                         .call_to_action("Text me order updates")
    ///                         .call_to_action_url("https://acme.example/checkout")
    ///                         .opt_in_message("Welcome to Acme Coffee updates. Reply STOP to opt out.")
    ///                         .help_response("Acme Coffee: email help@acme.example for support.")
    ///                         .opt_out_response("You have been unsubscribed from Acme Coffee updates."),
    ///                 ),
    ///         ),
    ///     )
    ///     .await?;
    /// println!("{}", agent.agent.customer_stage);
    /// # Ok(()) }
    /// ```
    pub async fn update(
        &self,
        id: &str,
        request: UpdateRcsAgentRequest,
    ) -> Result<RcsAgentResponse> {
        self.update_with_options(id, request, IdempotentRequestOptions::new())
            .await
    }

    /// [`update`](Self::update) with per-call options (e.g. your own
    /// idempotency key).
    pub async fn update_with_options(
        &self,
        id: &str,
        request: UpdateRcsAgentRequest,
        options: IdempotentRequestOptions,
    ) -> Result<RcsAgentResponse> {
        let response = self
            .client
            .patch_with_idempotency(
                &format!("/rcs/agents/{}", id),
                &request,
                options.idempotency_key.as_deref(),
                true,
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Replace the agent's test devices (up to 20). Requires the
    /// `rcs:write` scope.
    ///
    /// The list is authoritative: numbers missing from it are removed, new
    /// ones are invited. Devices receive an invite from the carrier network
    /// once the agent reaches the `testing` stage. A bad number comes back
    /// as 422 `rcs_invalid_content` naming `devices.<i>.phone_number`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{RcsTestDeviceInput, Sendly};
    ///
    /// # async fn run() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    /// let devices = client
    ///     .rcs()
    ///     .agents()
    ///     .set_test_devices(
    ///         "rcs_agent_xxx",
    ///         vec![
    ///             RcsTestDeviceInput::new("+13125550100").label("Sam's Pixel"),
    ///             RcsTestDeviceInput::new("+13125550101"),
    ///         ],
    ///     )
    ///     .await?;
    /// println!("{} devices invited", devices.devices.len());
    /// # Ok(()) }
    /// ```
    pub async fn set_test_devices(
        &self,
        id: &str,
        devices: Vec<RcsTestDeviceInput>,
    ) -> Result<RcsTestDeviceListResponse> {
        self.set_test_devices_with_options(id, devices, IdempotentRequestOptions::new())
            .await
    }

    /// [`set_test_devices`](Self::set_test_devices) with per-call options
    /// (e.g. your own idempotency key).
    pub async fn set_test_devices_with_options(
        &self,
        id: &str,
        devices: Vec<RcsTestDeviceInput>,
        options: IdempotentRequestOptions,
    ) -> Result<RcsTestDeviceListResponse> {
        let response = self
            .client
            .put_with_idempotency(
                &format!("/rcs/agents/{}/test-devices", id),
                &serde_json::json!({ "devices": devices }),
                options.idempotency_key.as_deref(),
                true,
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Submit the agent and its brand to Sendly for review — step 3 of
    /// registering for RCS. Requires the `rcs:write` scope.
    ///
    /// Required-field checks run here: the brand and the agent identity
    /// must be complete, and media URLs must be public `https://` (422
    /// `rcs_invalid_content` lists each `brand.<field>` / `agent.<field>`
    /// gap). On success the agent moves to `in_review`; Sendly reviews it,
    /// then the carrier network. Poll [`get`](Self::get) to follow
    /// progress. Answers 409 `rcs_field_locked` when already submitted and
    /// 409 `rcs_brand_not_verified` when the carrier network declined the
    /// brand.
    ///
    /// Pass your own idempotency key through
    /// [`submit_with_options`](Self::submit_with_options) so a retried
    /// call returns the original result instead of notifying reviewers
    /// again.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn run() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    /// let review = client.rcs().agents().submit("rcs_agent_xxx").await?;
    /// println!("{}", review.stage); // in_review
    /// # Ok(()) }
    /// ```
    pub async fn submit(&self, id: &str) -> Result<RcsAgentReviewResponse> {
        self.submit_with_options(id, IdempotentRequestOptions::new())
            .await
    }

    /// [`submit`](Self::submit) with per-call options (e.g. your own
    /// idempotency key).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{IdempotentRequestOptions, Sendly};
    ///
    /// # async fn run() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    /// let review = client
    ///     .rcs()
    ///     .agents()
    ///     .submit_with_options(
    ///         "rcs_agent_xxx",
    ///         IdempotentRequestOptions::new().idempotency_key("rcs-submit-rcs_agent_xxx"),
    ///     )
    ///     .await?;
    /// println!("{}", review.agent.review_status); // awaiting_review
    /// # Ok(()) }
    /// ```
    pub async fn submit_with_options(
        &self,
        id: &str,
        options: IdempotentRequestOptions,
    ) -> Result<RcsAgentReviewResponse> {
        let response = self
            .client
            .post_with_idempotency(
                &format!("/rcs/agents/{}/submit", id),
                &serde_json::json!({}),
                options.idempotency_key.as_deref(),
                true,
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Ask Sendly to launch the agent — step 5, once you've tested it on
    /// an invited device. Requires the `rcs:write` scope.
    ///
    /// The campaign section must be complete (an overview, at least one
    /// interaction, at least three message examples, consent settings) and
    /// the testing section needs a `test_url`, which you can pass here
    /// (422 `rcs_invalid_content` lists each `campaign.<field>` /
    /// `testing.<field>` gap). On success the agent moves to
    /// `launch_review`; Sendly reviews it, then launches it with the
    /// carrier network. Poll [`get`](Self::get) until the stage is `live`.
    /// Answers 409 `rcs_launch_not_ready` before the agent reaches testing
    /// and 409 `rcs_field_locked` while a request is already under review.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{RcsRequestLaunchRequest, Sendly};
    ///
    /// # async fn run() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    /// let review = client
    ///     .rcs()
    ///     .agents()
    ///     .request_launch(
    ///         "rcs_agent_xxx",
    ///         Some(RcsRequestLaunchRequest::new().test_url("https://acme.example/rcs-test")),
    ///     )
    ///     .await?;
    /// println!("{}", review.stage); // launch_review
    /// # Ok(()) }
    /// ```
    pub async fn request_launch(
        &self,
        id: &str,
        request: Option<RcsRequestLaunchRequest>,
    ) -> Result<RcsAgentReviewResponse> {
        self.request_launch_with_options(id, request, IdempotentRequestOptions::new())
            .await
    }

    /// [`request_launch`](Self::request_launch) with per-call options
    /// (e.g. your own idempotency key).
    pub async fn request_launch_with_options(
        &self,
        id: &str,
        request: Option<RcsRequestLaunchRequest>,
        options: IdempotentRequestOptions,
    ) -> Result<RcsAgentReviewResponse> {
        let response = self
            .client
            .post_with_idempotency(
                &format!("/rcs/agents/{}/request-launch", id),
                &request.unwrap_or_default(),
                options.idempotency_key.as_deref(),
                true,
            )
            .await?;
        Ok(response.json().await?)
    }
}

/// RCS resource — register your brand and agent, discover agents, and
/// check recipient capability.
///
/// # Example
///
/// ```rust,no_run
/// use sendly::{
///     CreateRcsAgentRequest, RcsAgentBasicsInput, RcsBrandAddressInput, RcsBrandInput, Sendly,
/// };
///
/// # async fn run() -> Result<(), sendly::Error> {
/// let client = Sendly::new("sk_live_v1_xxx");
///
/// // Register: brand -> agent -> submit (then test and request launch)
/// let brand = client
///     .rcs()
///     .brands()
///     .create(
///         RcsBrandInput::new()
///             .display_name("Acme Coffee")
///             .legal_name("Acme Coffee LLC")
///             .ein("12-3456789")
///             .address(
///                 RcsBrandAddressInput::new()
///                     .line1("100 Main St")
///                     .city("Chicago")
///                     .state("IL")
///                     .postal_code("60601")
///                     .country_code("US"),
///             ),
///     )
///     .await?;
/// let agent = client
///     .rcs()
///     .agents()
///     .create(
///         CreateRcsAgentRequest::new(&brand.brand.id)
///             .display_name("Acme Coffee")
///             .use_case("MULTI_USE")
///             .basics(RcsAgentBasicsInput::new().logo_url("https://acme.example/rcs/logo.png")),
///     )
///     .await?;
/// client.rcs().agents().submit(&agent.agent.id).await?;
///
/// // Once an agent is sendable: find it, optionally pre-flight, then send
/// let agents = client.rcs().agents().list().await?;
/// for agent in &agents.agents {
///     println!("{}: {} (sendable: {})", agent.id, agent.name, agent.sendable);
/// }
/// let capability = client.rcs().capability("+15551234567", None).await?;
/// println!("capable: {} {:?}", capability.capable, capability.features);
/// # Ok(()) }
/// ```
pub struct RcsResource<'a> {
    client: &'a Sendly,
}

impl<'a> RcsResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Returns the agents sub-resource.
    pub fn agents(&self) -> RcsAgentsResource<'a> {
        RcsAgentsResource::new(self.client)
    }

    /// Returns the brands sub-resource.
    pub fn brands(&self) -> RcsBrandsResource<'a> {
        RcsBrandsResource::new(self.client)
    }

    /// Returns the registration sub-resource.
    pub fn registration(&self) -> RcsRegistrationResource<'a> {
        RcsRegistrationResource::new(self.client)
    }

    /// Returns the dossier sub-resource.
    pub fn dossier(&self) -> RcsDossierResource<'a> {
        RcsDossierResource::new(self.client)
    }

    /// Check whether a recipient can receive RCS before sending.
    ///
    /// Pass `agent_id` to pick the agent to check against; `None` works
    /// when the workspace has exactly one sendable agent (with more than
    /// one, the API answers 400 `rcs_agent_ambiguous`). Requires a live API
    /// key.
    ///
    /// This pre-flight is optional: text sends fall back to SMS on their
    /// own for recipients without RCS support.
    pub async fn capability(&self, to: &str, agent_id: Option<&str>) -> Result<RcsCapability> {
        let mut query = vec![("to".to_string(), to.to_string())];
        if let Some(agent_id) = agent_id {
            query.push(("agentId".to_string(), agent_id.to_string()));
        }
        let response = self.client.get("/rcs/capability", &query).await?;
        Ok(response.json().await?)
    }
}
