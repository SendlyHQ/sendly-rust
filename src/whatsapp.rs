//! WhatsApp Resource — Connect senders, manage templates, check windows
//!
//! WhatsApp is a first-class Sendly channel: connect a number you own,
//! create Meta-reviewed message templates, and send via
//! [`Messages::send_whatsapp`](crate::Messages::send_whatsapp).
//!
//! Connecting a number is a one-time $19 setup (no monthly fee). The first
//! number ends with a human step: [`WhatsAppSignupResource::create`] returns
//! a `connect_url` that a person must open in a browser and log in with
//! Facebook to link their WhatsApp Business Account. Hand the URL to your
//! user: the connection cannot be completed programmatically. Further
//! numbers can be added to that connected account by verification code,
//! with no Facebook step: [`WhatsAppSignupResource::create_with_options`]
//! with a `business_account_id`, then [`WhatsAppSignupResource::verify`].
//!
//! Connected senders can also have a profile photo, ice breakers and
//! commands, and WhatsApp calling (see [`WhatsAppSendersResource`]).
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
//! Pricing: free-form text or media inside the 24-hour window costs 1 credit
//! each for the first 1,000 per sending number per calendar month (UTC),
//! then the destination's utility template price; countries without a listed
//! price use the default utility price of 12 credits. Templates are priced
//! by category and destination country; countries without a listed price
//! use 33 (marketing), 12 (utility) and 12 (authentication) credits. A failed
//! send gives its slot back.
//!
//! Scopes and keys: sends go through
//! [`Messages::send_whatsapp`](crate::Messages::send_whatsapp) and need
//! `sms:send`, not `whatsapp:write`, and a live key. Reads (signup status,
//! templates, the window, senders and sender profiles) need `whatsapp:read`
//! and accept test keys. Signup, template create/edit/delete and profile
//! edits need `whatsapp:write` and a live key (otherwise 403
//! `whatsapp_requires_live_key`). In a team workspace, connecting and profile
//! edits need an owner or admin (`settings:write`), and template writes need
//! an owner, admin or member (`templates:write`); a missing role returns 403
//! `insufficient_permissions`.
//!
//! WhatsApp is enabled per person: the user who owns the API key, not the
//! workspace. While it is off, sends return 403 `whatsapp_not_enabled` and
//! every method on this resource gets 404 `not_found`.
//!
//! See <https://sendly.live/docs/whatsapp> for the full flow.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use reqwest::multipart;

use crate::client::{path_id, Sendly};
use crate::error::{Error, Result};
use crate::media::mime_from_extension;

/// Lifecycle status of a WhatsApp signup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum WhatsAppSignupStatus {
    /// Created; waiting for a human to complete the connect URL.
    Initiated,
    /// The business account is linked; activation in progress. Activation
    /// usually takes a few minutes but can take hours. If it hasn't finished
    /// about 6 hours after the session began, the session fails with
    /// `registration_timeout` and the fee is refunded.
    Registering,
    /// A number being added to an already-connected WhatsApp Business
    /// account is waiting for its verification code: submit it with
    /// [`WhatsAppSignupResource::verify`].
    Verifying,
    /// Connected; the number can send and receive on WhatsApp.
    Active,
    /// The connection failed (see `failure_reasons`).
    Failed,
    /// The connect URL expired before it was completed.
    Expired,
    /// A status this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

/// How WhatsApp delivers the verification code for a number added to an
/// already-connected WhatsApp Business account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum WhatsAppVerificationMethod {
    /// A text message to the number (the default).
    Sms,
    /// A voice call to the number.
    Voice,
    /// A method this SDK version doesn't know yet; not valid in a request.
    #[serde(other)]
    Unknown,
}

/// Response from [`WhatsAppSignupResource::create`] and
/// [`WhatsAppSignupResource::create_with_options`].
///
/// For the Facebook connection, hand `connect_url` to a human: they open it
/// in a browser and log in with Facebook to link their WhatsApp Business
/// Account. Poll [`WhatsAppSignupResource::get`] with `id` until the status
/// is `Active`.
///
/// When a number is added to an already-connected account (a non-empty
/// `business_account_id` was given), the status is `Verifying`,
/// `connect_url` is empty, and the remaining fields are set.
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppSignupSession {
    /// Unique signup identifier — use with [`WhatsAppSignupResource::get`].
    pub id: String,
    /// Hosted connect page URL. A person must open this in a browser. Empty
    /// when a number is added to an already-connected account.
    #[serde(default, alias = "connectUrl")]
    pub connect_url: String,
    /// Current signup status.
    pub status: WhatsAppSignupStatus,
    /// The number being added, in E.164 format. `None` on a Facebook
    /// connection response.
    #[serde(default, alias = "phoneNumber")]
    pub phone_number: Option<String>,
    /// The WhatsApp Business Account the number is being added to. `None` on
    /// a Facebook connection response.
    #[serde(default, alias = "businessAccountId")]
    pub business_account_id: Option<String>,
    /// How the verification code is delivered, while the status is
    /// `Verifying`.
    #[serde(default, alias = "verificationMethod")]
    pub verification_method: Option<WhatsAppVerificationMethod>,
    /// Wrong codes still allowed before the attempt fails, while the status
    /// is `Verifying`.
    #[serde(default, alias = "verificationAttemptsRemaining")]
    pub verification_attempts_remaining: Option<u32>,
    /// ISO 8601 timestamp of the last status change. `None` on a Facebook
    /// connection response.
    #[serde(default, alias = "updatedAt")]
    pub updated_at: Option<String>,
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
    /// Why the signup failed, when status is `Failed`; `None` otherwise. If
    /// the connection fails, the $19 fee is refunded automatically. Adding a
    /// number by code can fail with `verification_start_failed` (WhatsApp
    /// would not send a code), `verification_failed` (too many wrong codes)
    /// or `verification_expired` (the code was never entered). Other values
    /// may appear; treat the list as open.
    #[serde(default, alias = "failureReasons")]
    pub failure_reasons: Option<Vec<String>>,
    /// ISO 8601 timestamp of the last status change.
    #[serde(alias = "updatedAt")]
    pub updated_at: String,
    /// How the verification code is delivered; set only while the status
    /// is `Verifying`.
    #[serde(default, alias = "verificationMethod")]
    pub verification_method: Option<WhatsAppVerificationMethod>,
    /// Wrong codes still allowed before the attempt fails; set only while
    /// the status is `Verifying`.
    #[serde(default, alias = "verificationAttemptsRemaining")]
    pub verification_attempts_remaining: Option<u32>,
    /// The 6-digit code, once WhatsApp's text has arrived on the number
    /// (read from your workspace's inbound messages). Set only by
    /// [`WhatsAppSignupResource::get`] while the status is `Verifying`, and
    /// `None` until a code arrives. Until a code has been submitted, it is
    /// the newest code that has arrived since the signup started, so after
    /// a resend it still shows the earlier code until the new one arrives.
    /// Once WhatsApp has checked a code, only a code that arrived after the
    /// last submission or resend is returned. A submission answered with
    /// 502 `whatsapp_verification_unavailable` is not counted, so the same
    /// unchecked code can come back, and submitting it again with
    /// [`WhatsAppSignupResource::verify`] is safe.
    #[serde(default, alias = "verificationCode")]
    pub verification_code: Option<String>,
}

/// Request body for [`WhatsAppSignupResource::create_with_options`].
///
/// With only a phone number, or with no `business_account_id`, this starts
/// the Facebook connection, like [`WhatsAppSignupResource::create`]. With a
/// non-empty `business_account_id` it adds the number to that
/// already-connected WhatsApp Business account by verification code, with
/// no Facebook step. A `business_account_id` that is present but empty or
/// whitespace-only is refused with [`Error::Validation`] before anything is
/// sent.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CreateWhatsAppSignupRequest {
    /// The number to connect, in E.164 format.
    #[serde(rename = "phoneNumber")]
    pub phone_number: String,
    /// The WhatsApp Business Account id (as in
    /// [`WhatsAppSender::business_account_id`]) to add the number to. Leave
    /// it unset to start the Facebook connection instead.
    #[serde(skip_serializing_if = "Option::is_none", rename = "businessAccountId")]
    pub business_account_id: Option<String>,
    /// How WhatsApp sends the code: by text (the default) or by voice call.
    /// Only read with a non-empty `business_account_id`.
    #[serde(skip_serializing_if = "Option::is_none", rename = "verificationMethod")]
    pub verification_method: Option<WhatsAppVerificationMethod>,
    /// The name WhatsApp shows for the number (at most 512 characters).
    /// Defaults to the account's existing sender display name, else its
    /// business name. Only read with a non-empty `business_account_id`.
    #[serde(skip_serializing_if = "Option::is_none", rename = "displayName")]
    pub display_name: Option<String>,
}

impl CreateWhatsAppSignupRequest {
    pub fn new(phone_number: impl Into<String>) -> Self {
        Self {
            phone_number: phone_number.into(),
            ..Self::default()
        }
    }

    pub fn business_account_id(mut self, business_account_id: impl Into<String>) -> Self {
        self.business_account_id = Some(business_account_id.into());
        self
    }

    pub fn verification_method(mut self, verification_method: WhatsAppVerificationMethod) -> Self {
        self.verification_method = Some(verification_method);
        self
    }

    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }
}

/// Connection status of a WhatsApp sender.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum WhatsAppSenderStatus {
    /// The connection is still in progress; not sendable yet.
    Pending,
    /// Connected; the number can send and receive on WhatsApp.
    Active,
    /// Sending is currently suspended for this number.
    Suspended,
    /// A status this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
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
    /// The WhatsApp Business Account id the number belongs to; pass it to
    /// [`CreateWhatsAppSignupRequest::business_account_id`] to add another
    /// number to the same account. `None` while the status is `Pending`.
    #[serde(default, alias = "businessAccountId")]
    pub business_account_id: Option<String>,
    /// The business name on that account. `None` while the status is
    /// `Pending`, or when the account has no business name on record.
    #[serde(default, alias = "businessName")]
    pub business_name: Option<String>,
    /// Whether WhatsApp calling is on for the number (see
    /// [`WhatsAppSendersResource::set_calling`]).
    #[serde(default, alias = "callingEnabled")]
    pub calling_enabled: Option<bool>,
    /// Whether WhatsApp lets the business call people from this number.
    /// False for every +1 number and for +20, +84 and +234 numbers.
    #[serde(default, alias = "outboundCallingAllowed")]
    pub outbound_calling_allowed: Option<bool>,
}

/// Response from [`WhatsAppSendersResource::list`].
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppSendersList {
    #[serde(default)]
    pub senders: Vec<WhatsAppSender>,
}

/// The WhatsApp Business profile recipients see for a connected sender —
/// the details behind your name in the chat.
#[derive(Debug, Clone, Deserialize)]
pub struct WhatsAppSenderProfile {
    /// The sender, in E.164 format.
    #[serde(alias = "phoneNumber")]
    pub phone_number: String,
    /// The name recipients see; `None` until set.
    #[serde(default, alias = "displayName")]
    pub display_name: Option<String>,
    /// Profile photo URL; `None` until set. Change it with
    /// [`WhatsAppSendersResource::upload_profile_photo`] and
    /// [`WhatsAppSendersResource::delete_profile_photo`].
    #[serde(default, alias = "profilePhotoUrl")]
    pub profile_photo_url: Option<String>,
    /// Business category; `None` until set.
    #[serde(default)]
    pub category: Option<String>,
    /// Short line under the profile name (max 139 characters); `None`
    /// until set.
    #[serde(default)]
    pub about: Option<String>,
    /// Longer business description (max 512 characters); `None` until set.
    #[serde(default)]
    pub description: Option<String>,
    /// Public contact email; `None` until set.
    #[serde(default)]
    pub email: Option<String>,
    /// Public website URL; `None` until set.
    #[serde(default)]
    pub website: Option<String>,
    /// Public business address; `None` until set.
    #[serde(default)]
    pub address: Option<String>,
}

/// Request body for [`WhatsAppSendersResource::update_profile`]. Supply only
/// the fields to change — at least one is required; omitted fields keep
/// their current value.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateWhatsAppSenderProfileRequest {
    /// Replacement display name.
    #[serde(skip_serializing_if = "Option::is_none", rename = "displayName")]
    pub display_name: Option<String>,
    /// Replacement short line under the profile name (max 139 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    /// Replacement business description (max 512 characters).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Replacement business category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Replacement public contact email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Replacement public website URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    /// Replacement public business address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

impl UpdateWhatsAppSenderProfileRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn about(mut self, about: impl Into<String>) -> Self {
        self.about = Some(about.into());
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn website(mut self, website: impl Into<String>) -> Self {
        self.website = Some(website.into());
        self
    }

    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }
}

/// A command a customer can pick by typing "/" in the chat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WhatsAppCommand {
    /// The command, without the "/": letters, digits and underscores, at
    /// most 32 characters. A leading "/" is stripped by the API.
    pub command: String,
    /// What the command does, at most 256 characters.
    #[serde(default)]
    pub description: String,
}

impl WhatsAppCommand {
    pub fn new(command: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            description: description.into(),
        }
    }
}

/// A sender's conversation starters: the ice breakers shown when someone
/// opens a chat with the business for the first time, and the commands
/// shown when they type "/".
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct WhatsAppConversationalComponents {
    /// The sender, in E.164 format.
    #[serde(alias = "phoneNumber")]
    pub phone_number: String,
    /// Tappable suggestions shown in a first chat.
    #[serde(default, alias = "iceBreakers")]
    pub ice_breakers: Vec<String>,
    /// Commands shown when the customer types "/".
    #[serde(default)]
    pub commands: Vec<WhatsAppCommand>,
}

/// Request body for
/// [`WhatsAppSendersResource::update_conversational_components`]. Set at
/// least one list. Each list given replaces the stored one, an empty list
/// clears it, and a list left unset is kept as is.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateWhatsAppConversationalComponentsRequest {
    /// Up to 4 ice breakers, each 1 to 80 characters, no two the same
    /// (ignoring case).
    #[serde(skip_serializing_if = "Option::is_none", rename = "iceBreakers")]
    pub ice_breakers: Option<Vec<String>>,
    /// Up to 30 commands, no two the same (ignoring case).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<WhatsAppCommand>>,
}

impl UpdateWhatsAppConversationalComponentsRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ice_breakers<I, S>(mut self, ice_breakers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.ice_breakers = Some(ice_breakers.into_iter().map(Into::into).collect());
        self
    }

    pub fn commands(mut self, commands: Vec<WhatsAppCommand>) -> Self {
        self.commands = Some(commands);
        self
    }
}

/// Response from [`WhatsAppSendersResource::set_calling`].
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct WhatsAppSenderCalling {
    /// The sender, in E.164 format.
    #[serde(alias = "phoneNumber")]
    pub phone_number: String,
    /// Whether WhatsApp calling is now on for the number.
    #[serde(alias = "callingEnabled")]
    pub calling_enabled: bool,
    /// Whether WhatsApp lets the business call people from this number.
    /// False for every +1 number and for +20, +84 and +234 numbers.
    #[serde(alias = "outboundCallingAllowed")]
    pub outbound_calling_allowed: bool,
}

/// Template category. Meta reviews every template and may reclassify it —
/// the category on the record is authoritative and drives per-message
/// pricing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[non_exhaustive]
pub enum WhatsAppTemplateCategory {
    Authentication,
    Utility,
    Marketing,
    /// A category this SDK version doesn't know yet; not valid in a request.
    #[serde(other)]
    Unknown,
}

/// Review status of a template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
#[non_exhaustive]
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
    /// A status this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
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
    /// Template category — drives Meta review rules and pricing. Required,
    /// with no default; the server uppercases it, and an update can't change
    /// it.
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
    /// When the window closes (ISO 8601). After it closes this is the past
    /// expiry, with `open` false. `None` when Sendly has no window on record
    /// for the pair (then `open` is false too).
    #[serde(default, alias = "expiresAt")]
    pub expires_at: Option<String>,
}

/// Connect numbers to WhatsApp. Starting a signup returns a `connect_url`
/// a human must complete in a browser; further numbers can be added to a
/// connected account by verification code.
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
    /// Requires a live API key with the `whatsapp:write` scope (a test key
    /// gets 403 `whatsapp_requires_live_key`) and, in a team workspace, an
    /// owner or admin (`settings:write`). After the Facebook step the signup
    /// stays `Registering` while WhatsApp activates the number. Activation
    /// usually takes a few minutes but can take hours. If it hasn't finished
    /// about 6 hours after the session began, the session fails with
    /// `registration_timeout` and the fee is refunded. If the connection
    /// fails, the $19 fee is refunded automatically; once the number has
    /// connected there is no refund, and a later disconnect gets nothing
    /// back.
    ///
    /// A number that is being added by code answers 409
    /// `whatsapp_verification_in_progress`, with that session's `id` in the
    /// body.
    ///
    /// While WhatsApp connections are unavailable this answers 503
    /// `whatsapp_unavailable` before charging anything, with `retryAfter`
    /// 3600 in the body and a `Retry-After: 3600` header. Only signup
    /// returns it; no send does.
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

    /// Start connecting a number, with options. Without a
    /// `business_account_id` this is [`create`](Self::create): it starts
    /// the Facebook connection.
    ///
    /// With a non-empty `business_account_id` it adds the number to that
    /// WhatsApp Business account, which must already be connected in this
    /// workspace with at least one active number, by verification code and
    /// with no Facebook step. It charges the same $19 one-time fee (refunded
    /// automatically if the connection fails), asks WhatsApp to send a
    /// 6-digit code to the number by text or voice call, and returns a
    /// session with status `Verifying` (and no `connect_url`). Poll
    /// [`get`](Self::get) for `verification_code` (filled in when the code
    /// arrives as a text on the number), then submit it with
    /// [`verify`](Self::verify). Calling again for a number that is already
    /// verifying returns that session without charging again or sending a
    /// second code.
    ///
    /// Requires a live API key with the `whatsapp:write` scope and, in a
    /// team workspace, an owner or admin (`settings:write`). The number
    /// must pass the same checks as a Facebook connection. Refusals: 404
    /// `whatsapp_business_account_not_found` (`Error::NotFound`); 400
    /// `display_name_required` or `invalid_request` (`Error::Validation`);
    /// 402 `payment_method_required` or `payment_failed`
    /// (`Error::InsufficientCredits`); 409 `whatsapp_already_enabled`,
    /// `whatsapp_signup_in_progress` (a Facebook connection for the number
    /// is under way; the body has its `id`) or
    /// `whatsapp_verification_in_progress` (`Error::Api`); 429
    /// `whatsapp_signup_limit_reached` (`Error::RateLimit`, not retryable);
    /// and `whatsapp_verification_start_failed` when WhatsApp would not send
    /// the code, a 422 when it refused (`Error::Validation`, final) or a 502
    /// when it couldn't be reached (`Error::Api`, start again). Either way
    /// the session fails and the fee is refunded.
    ///
    /// Starting the Facebook connection for a number that is being added by
    /// code answers 409 `whatsapp_verification_in_progress` with the
    /// session's `id` in the body.
    ///
    /// With a non-empty `business_account_id` the request is sent once: the
    /// client never retries it, not after a 5xx, a timeout or a network
    /// error, because a failed start has already failed the session and
    /// refunded the fee, and a new attempt is a new $19 session. Calling
    /// again is safe while the number is verifying (you get the same session
    /// back). No `business_account_id` starts the Facebook connection,
    /// which is retried like [`create`](Self::create). A
    /// `business_account_id` that is present but empty or whitespace-only
    /// is refused with [`Error::Validation`] before anything is sent.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{CreateWhatsAppSignupRequest, Sendly, WhatsAppVerificationMethod};
    ///
    /// # async fn run() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    /// let signup = client
    ///     .whatsapp()
    ///     .signup()
    ///     .create_with_options(
    ///         CreateWhatsAppSignupRequest::new("+14155550142")
    ///             .business_account_id("104996582519384")
    ///             .verification_method(WhatsAppVerificationMethod::Sms),
    ///     )
    ///     .await?;
    /// println!("{} {:?}", signup.id, signup.status);
    /// # Ok(()) }
    /// ```
    pub async fn create_with_options(
        &self,
        request: CreateWhatsAppSignupRequest,
    ) -> Result<WhatsAppSignupSession> {
        if matches!(request.business_account_id.as_deref(), Some(id) if id.trim().is_empty()) {
            return Err(Error::validation(
                "businessAccountId must be a non-empty string",
            ));
        }
        let by_code = matches!(request.business_account_id.as_deref(), Some(id) if !id.is_empty());
        let response = if by_code {
            self.client.post_once("/whatsapp/signup", &request).await?
        } else {
            self.client.post("/whatsapp/signup", &request).await?
        };
        Ok(response.json().await?)
    }

    /// Get the status of a WhatsApp signup. Needs the `whatsapp:read` scope;
    /// test keys work.
    ///
    /// While a number added by code is `Verifying`, the signup also carries
    /// `verification_method`, `verification_attempts_remaining` and
    /// `verification_code`: the code once WhatsApp's text has arrived on the
    /// number, ready to pass to [`verify`](Self::verify). An unknown id is
    /// 404 `signup_not_found` (`Error::NotFound`).
    pub async fn get(&self, id: &str) -> Result<WhatsAppSignup> {
        let response = self
            .client
            .get(&format!("/whatsapp/signup/{}", path_id(id)?), &[])
            .await?;
        Ok(response.json().await?)
    }

    /// Submit the 6-digit code WhatsApp sent to a number being added by
    /// code. Spaces and dashes in the code are ignored. On success the
    /// signup is `Active` and the `whatsapp_account.connected` webhook
    /// fires; a signup that is already `Active` comes back as is.
    ///
    /// Requires a live API key with the `whatsapp:write` scope and, in a
    /// team workspace, an owner or admin (`settings:write`). Refusals:
    ///
    /// - 400 `invalid_verification_code`: not 6 digits (`Error::Validation`).
    /// - 422 `whatsapp_verification_code_invalid`: a wrong code
    ///   (`Error::Validation`); [`Error::remaining_attempts`] says how many
    ///   tries are left.
    /// - 409 `whatsapp_verification_failed`: the fifth wrong code. The
    ///   signup fails and the fee is refunded; start again.
    /// - 409 `whatsapp_verification_busy`: another code for the number is
    ///   being checked; try again in a moment.
    /// - 409 `signup_not_active`: the signup isn't waiting for a code (it
    ///   failed, or is more than 3 hours old).
    /// - 502 `whatsapp_verification_unavailable`: WhatsApp couldn't be
    ///   reached; the attempt isn't counted, so try again.
    /// - 502 `whatsapp_activation_pending`: the code was accepted but the
    ///   connection couldn't be finished yet; check back with
    ///   [`get`](Self::get).
    /// - 404 `signup_not_found` (`Error::NotFound`).
    ///
    /// The 409s and 502s are `Error::Api` with `code` set.
    ///
    /// Every submission uses up one of the five attempts, so the code is sent
    /// once: the client never retries it, not after a 5xx, a timeout or a
    /// network error. Check with [`get`](Self::get) before submitting again.
    pub async fn verify(&self, id: &str, code: &str) -> Result<WhatsAppSignup> {
        let response = self
            .client
            .post_once(
                &format!("/whatsapp/signup/{}/verify", path_id(id)?),
                &serde_json::json!({ "code": code }),
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Ask WhatsApp to send a new code to a number being added by code, by
    /// text (`None` or `Sms`) or voice call. Wrong-code attempts are not
    /// reset.
    ///
    /// Requires a live API key with the `whatsapp:write` scope and, in a
    /// team workspace, an owner or admin (`settings:write`). Codes are at
    /// least 30 seconds apart, counted from the last change to the signup
    /// (including the first code and every submission): sooner is 429
    /// `whatsapp_verification_resend_too_soon` (`Error::RateLimit`), and
    /// [`Error::retry_after`] says how many seconds to wait. WhatsApp
    /// refusing to send another code is 422
    /// `whatsapp_verification_resend_failed` (`Error::Validation`; wait a
    /// few minutes), or a 502 with the same code when it couldn't be
    /// reached (`Error::Api`). A signup that isn't waiting for a code is
    /// 409 `signup_not_active`; one that is already `Active` comes back as
    /// is.
    pub async fn resend(
        &self,
        id: &str,
        verification_method: Option<WhatsAppVerificationMethod>,
    ) -> Result<WhatsAppSignup> {
        let body = match verification_method {
            Some(method) => serde_json::json!({ "verificationMethod": method }),
            None => serde_json::json!({}),
        };
        let response = self
            .client
            .post(&format!("/whatsapp/signup/{}/resend", path_id(id)?), &body)
            .await?;
        Ok(response.json().await?)
    }
}

/// List the numbers connected (or connecting) to WhatsApp, and manage their
/// business profiles, profile photos, ice breakers and commands, and
/// WhatsApp calling.
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
    /// yet — start one with [`WhatsAppSignupResource::create`]. Needs the
    /// `whatsapp:read` scope; test keys work.
    pub async fn list(&self) -> Result<WhatsAppSendersList> {
        let response = self.client.get("/whatsapp/senders", &[]).await?;
        Ok(response.json().await?)
    }

    /// Get the WhatsApp Business profile recipients see for a connected
    /// sender (E.164). The number must have an active WhatsApp connection.
    /// Needs the `whatsapp:read` scope; test keys work.
    pub async fn get_profile(&self, phone_number: &str) -> Result<WhatsAppSenderProfile> {
        let response = self
            .client
            .get(
                &format!("/whatsapp/senders/{}/profile", path_id(phone_number)?),
                &[],
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Update a connected sender's WhatsApp Business profile and return the
    /// updated profile.
    ///
    /// Supply only the fields to change — at least one is required.
    /// `about` is capped at 139 characters and `description` at 512.
    /// Requires a live API key with the `whatsapp:write` scope and, in a
    /// team workspace, an owner or admin (`settings:write`).
    pub async fn update_profile(
        &self,
        phone_number: &str,
        request: UpdateWhatsAppSenderProfileRequest,
    ) -> Result<WhatsAppSenderProfile> {
        let response = self
            .client
            .patch(
                &format!("/whatsapp/senders/{}/profile", path_id(phone_number)?),
                &request,
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Upload a connected sender's profile photo from a file and return the
    /// updated profile. The content type is taken from the file extension.
    ///
    /// See [`upload_profile_photo_bytes`](Self::upload_profile_photo_bytes)
    /// for the rules and refusals.
    pub async fn upload_profile_photo(
        &self,
        phone_number: &str,
        file_path: &str,
    ) -> Result<WhatsAppSenderProfile> {
        let path = std::path::Path::new(file_path);
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("photo")
            .to_string();
        let content_type = mime_from_extension(path);
        let data = tokio::fs::read(file_path)
            .await
            .map_err(|e| Error::validation(format!("Failed to read file: {e}")))?;
        self.upload_profile_photo_bytes(phone_number, data, &filename, &content_type)
            .await
    }

    /// Upload a connected sender's profile photo and return the updated
    /// profile. The photo is sent as the multipart field `file`.
    ///
    /// It must be a JPEG or PNG (checked from the bytes) of at most 5 MB.
    /// WhatsApp wants it square and at least 192 pixels wide (640
    /// recommended). Requires a live API key with the `whatsapp:write`
    /// scope and, in a team workspace, an owner or admin (`settings:write`).
    /// Refusals: 400 `file_required`, `whatsapp_profile_photo_invalid` (not
    /// a JPEG or PNG) or `invalid_request` (`Error::Validation`); 413
    /// `whatsapp_profile_photo_too_large` (`Error::Api`); 404
    /// `whatsapp_sender_not_connected` (`Error::NotFound`); 502
    /// `whatsapp_profile_update_failed` (`Error::Api`) when WhatsApp refused
    /// the photo or couldn't be reached, which can be retried after fixing
    /// the image. The photo is sent once: the client never retries it after
    /// a 5xx, a timeout or a network error.
    pub async fn upload_profile_photo_bytes(
        &self,
        phone_number: &str,
        data: Vec<u8>,
        filename: &str,
        content_type: &str,
    ) -> Result<WhatsAppSenderProfile> {
        let path = format!("/whatsapp/senders/{}/profile/photo", path_id(phone_number)?);
        let build_form = || {
            let part = multipart::Part::bytes(data.clone())
                .file_name(filename.to_string())
                .mime_str(content_type)
                .map_err(|e| Error::validation(format!("Invalid content type: {e}")))?;
            Ok(multipart::Form::new().part("file", part))
        };
        let response = self.client.post_multipart(&path, build_form).await?;
        Ok(response.json().await?)
    }

    /// Remove a connected sender's profile photo and return the updated
    /// profile. Requires a live API key with the `whatsapp:write` scope and,
    /// in a team workspace, an owner or admin (`settings:write`). 502
    /// `whatsapp_profile_update_failed` (`Error::Api`) when WhatsApp
    /// couldn't remove it; try again shortly.
    pub async fn delete_profile_photo(&self, phone_number: &str) -> Result<WhatsAppSenderProfile> {
        let response = self
            .client
            .delete(&format!(
                "/whatsapp/senders/{}/profile/photo",
                path_id(phone_number)?
            ))
            .await?;
        Ok(response.json().await?)
    }

    /// Get a connected sender's ice breakers and commands. Needs the
    /// `whatsapp:read` scope; test keys work. 502
    /// `whatsapp_conversational_components_fetch_failed` (`Error::Api`) when
    /// WhatsApp couldn't be reached.
    pub async fn get_conversational_components(
        &self,
        phone_number: &str,
    ) -> Result<WhatsAppConversationalComponents> {
        let response = self
            .client
            .get(
                &format!(
                    "/whatsapp/senders/{}/conversational_components",
                    path_id(phone_number)?
                ),
                &[],
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Replace a connected sender's ice breakers, commands, or both, and
    /// return what is now stored.
    ///
    /// Each list given replaces the stored one and an empty list clears it.
    /// Requires a live API key with the `whatsapp:write` scope and, in a
    /// team workspace, an owner or admin (`settings:write`). A list over the
    /// limits (see [`UpdateWhatsAppConversationalComponentsRequest`]) is 400
    /// `invalid_request` (`Error::Validation`) with a message that says
    /// which; 502 `whatsapp_conversational_components_update_failed`
    /// (`Error::Api`) when WhatsApp couldn't save them.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, UpdateWhatsAppConversationalComponentsRequest, WhatsAppCommand};
    ///
    /// # async fn run() -> Result<(), sendly::Error> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    /// client
    ///     .whatsapp()
    ///     .senders()
    ///     .update_conversational_components(
    ///         "+14155550123",
    ///         UpdateWhatsAppConversationalComponentsRequest::new()
    ///             .ice_breakers(vec!["Book a repair", "Get a quote"])
    ///             .commands(vec![WhatsAppCommand::new("quote", "Get a price for a job")]),
    ///     )
    ///     .await?;
    /// # Ok(()) }
    /// ```
    pub async fn update_conversational_components(
        &self,
        phone_number: &str,
        request: UpdateWhatsAppConversationalComponentsRequest,
    ) -> Result<WhatsAppConversationalComponents> {
        let response = self
            .client
            .patch(
                &format!(
                    "/whatsapp/senders/{}/conversational_components",
                    path_id(phone_number)?
                ),
                &request,
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Turn WhatsApp calling on or off for a connected sender. Once it is
    /// on, a WhatsApp user calling the number rings like a phone call (in
    /// the dashboard or on the number's AI agent, per its voice mode).
    ///
    /// Requires a live API key with the `whatsapp:write` scope and, in a
    /// team workspace, an owner or admin (`settings:write`). Turning it on
    /// needs voice switched on for the number first, else 409
    /// `voice_not_enabled` (`Error::Api`). 422 `whatsapp_calling_unavailable`
    /// (`Error::Validation`) when WhatsApp didn't allow it: the account must
    /// be able to message at least 2,000 people a day and the display name
    /// must be approved. 502 `whatsapp_calling_update_failed` (`Error::Api`)
    /// can be retried.
    pub async fn set_calling(
        &self,
        phone_number: &str,
        enabled: bool,
    ) -> Result<WhatsAppSenderCalling> {
        let response = self
            .client
            .patch(
                &format!("/whatsapp/senders/{}/calling", path_id(phone_number)?),
                &serde_json::json!({ "enabled": enabled }),
            )
            .await?;
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

    /// List your WhatsApp templates. Needs the `whatsapp:read` scope; test
    /// keys work.
    pub async fn list(&self) -> Result<WhatsAppTemplateListResponse> {
        let response = self.client.get("/whatsapp/templates", &[]).await?;
        Ok(response.json().await?)
    }

    /// Create a template and submit it to Meta for review.
    ///
    /// Review usually takes 24-48h; the template is usable once its status
    /// is `Approved`. Requires a live API key with the `whatsapp:write` scope
    /// and, in a team workspace, an owner, admin or member
    /// (`templates:write`). The category is required, with no default.
    ///
    /// A sender that isn't connected gets 404 `whatsapp_sender_not_connected`,
    /// checked first. A template that fails pre-flight checks gets a 400
    /// (`Error::Validation`): `template_category_invalid` (category missing
    /// or not one of the three), `template_authentication_otp_button_required`,
    /// `template_authentication_no_links` (a link in the body or a URL button
    /// on an authentication template) or `template_header_variable_unsupported`.
    /// A marketing template without an opt-out button only gets a warning.
    pub async fn create(&self, request: CreateWhatsAppTemplateRequest) -> Result<WhatsAppTemplate> {
        let response = self.client.post("/whatsapp/templates", &request).await?;
        Ok(response.json().await?)
    }

    /// Edit an APPROVED or REJECTED template and resubmit it for review.
    ///
    /// This is the recovery path for rejections: template names are locked
    /// for ~30 days after deletion, so editing a rejected template (rather
    /// than deleting and re-creating it) is the way to fix it. The updated
    /// template goes back to `Pending` review. The category can't be changed.
    /// Requires a live API key with the `whatsapp:write` scope and, in a team
    /// workspace, an owner, admin or member (`templates:write`).
    pub async fn update(
        &self,
        id: &str,
        request: UpdateWhatsAppTemplateRequest,
    ) -> Result<WhatsAppTemplate> {
        let response = self
            .client
            .patch(&format!("/whatsapp/templates/{}", path_id(id)?), &request)
            .await?;
        Ok(response.json().await?)
    }

    /// Delete a template.
    ///
    /// Meta locks a deleted template's name for ~30 days — re-creating it
    /// fails with `template_name_locked` until the lock lifts. To fix a
    /// rejected template, prefer [`update`](Self::update). Requires a live
    /// API key with the `whatsapp:write` scope and, in a team workspace, an
    /// owner, admin or member (`templates:write`).
    pub async fn delete(&self, id: &str) -> Result<WhatsAppTemplateDeletedResponse> {
        let response = self
            .client
            .delete(&format!("/whatsapp/templates/{}", path_id(id)?))
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
    /// inbound message). Outside a window, send an approved template. Needs
    /// the `whatsapp:read` scope; test keys work.
    ///
    /// The response is exactly `{ open, expiresAt }`: with no window on
    /// record `open` is false and `expires_at` is `None`; after a window has
    /// expired `open` is false and `expires_at` is the past expiry.
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
