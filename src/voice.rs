//! Voice Configuration Resource: switch voice on for a number and choose how
//! it answers, register the number's emergency address, and manage the AI
//! agents that talk on calls.
//!
//! [`VoiceResource::numbers`] reads and changes each number's voice
//! settings, [`VoiceResource::agents`] creates, edits and deletes AI agents,
//! and [`VoiceResource::voices`] lists the voices an agent can speak with.
//! Place and follow calls with [`CallsResource`](crate::CallsResource).
//!
//! Reads need the `calls:read` scope and writes `calls:write`; every write
//! also needs a live API key (a test key answers 403 `live_key_required`).
//! In a team workspace, changing a number or its emergency address also
//! needs a role that can change settings, and managing agents a role that
//! can manage API keys (each agent holds its own scoped sending key);
//! otherwise the API answers 403 `forbidden`.
//!
//! Voice is being enabled workspace by workspace; until it is on for yours,
//! every route answers 404 `voice_not_enabled`
//! ([`Error::NotFound`](crate::Error::NotFound)). Every write carries an
//! `Idempotency-Key`, generated per call or your own through the
//! `*_with_options` variants.
//!
//! See <https://sendly.live/docs/voice> for the full guide.

use serde::{Deserialize, Serialize};

use crate::client::Sendly;
use crate::error::{Error, Result};
use crate::models::IdempotentRequestOptions;

/// How a number answers phone calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum VoiceMode {
    /// Voice is off for the number.
    None,
    /// Calls ring the team in the dashboard.
    RingDashboard,
    /// An AI agent answers.
    Agent,
    /// A mode this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

impl VoiceMode {
    /// The mode as the API spells it (e.g. `"ring_dashboard"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::RingDashboard => "ring_dashboard",
            Self::Agent => "agent",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for VoiceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A street address registered for emergency calls.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EmergencyAddress {
    /// Street address.
    #[serde(default)]
    pub street: String,
    /// Apartment, suite or floor, when there is one.
    #[serde(default)]
    pub unit: Option<String>,
    /// City.
    #[serde(default)]
    pub city: String,
    /// Two-letter state or province code.
    #[serde(default)]
    pub state: String,
    /// ZIP code (US) or postal code (Canada).
    #[serde(default)]
    pub zip: String,
    /// `"US"` or `"CA"`.
    #[serde(default)]
    pub country: String,
}

/// A number's emergency address registration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct VoiceNumberEmergencyAddress {
    /// `"provisioning"` while the registration is being switched on,
    /// `"active"` once it is in place, otherwise the failure status as
    /// recorded.
    pub status: String,
    /// The registered address, or `None` when none is on file.
    #[serde(default)]
    pub address: Option<EmergencyAddress>,
}

impl VoiceNumberEmergencyAddress {
    /// `true` once the registration is in place.
    pub fn is_active(&self) -> bool {
        self.status == "active"
    }
}

/// Credits charged per started minute on a number (1 credit = $0.01).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct VoiceNumberRates {
    /// An inbound call the team answers in the dashboard.
    #[serde(default)]
    pub inbound: i64,
    /// An outbound call (an agent on the call adds its own per-minute
    /// charge).
    #[serde(default)]
    pub outbound: i64,
    /// An inbound call an AI agent answers, agent included.
    #[serde(default)]
    pub agent: i64,
}

fn default_voice_number_object() -> String {
    "voice_number".to_string()
}

fn default_voice_agent_object() -> String {
    "voice_agent".to_string()
}

/// A number in the workspace with its voice settings.
///
/// Returned by every method on [`VoiceNumbersResource`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct VoiceNumber {
    /// Unique number identifier (a UUID).
    pub id: String,
    /// Always `"voice_number"`.
    #[serde(default = "default_voice_number_object")]
    pub object: String,
    /// The phone number (E.164).
    pub phone_number: String,
    /// The number's type (e.g. `"local"`), when known.
    #[serde(default)]
    pub phone_number_type: Option<String>,
    /// ISO 3166-1 alpha-2 country code, when known.
    #[serde(default)]
    pub country_code: Option<String>,
    /// Whether this is the workspace's default sending number.
    #[serde(default)]
    pub is_default: bool,
    /// Whether the number takes and places phone calls.
    #[serde(default)]
    pub voice_enabled: bool,
    /// How the number answers; always [`VoiceMode::None`] when
    /// `voice_enabled` is false.
    pub voice_mode: VoiceMode,
    /// The agent that answers when `voice_mode` is [`VoiceMode::Agent`]. In
    /// other modes, whichever agent was last stored, or `None`.
    #[serde(default)]
    pub agent_id: Option<String>,
    /// The emergency address registration, or `None` if one was never
    /// registered.
    #[serde(default)]
    pub emergency_address: Option<VoiceNumberEmergencyAddress>,
    /// Credits per started minute on this number.
    #[serde(default)]
    pub rate_per_minute: VoiceNumberRates,
}

/// Response from [`VoiceNumbersResource::list`].
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct VoiceNumberListResponse {
    /// Active numbers in the workspace, in the same order as the dashboard.
    #[serde(default)]
    pub data: Vec<VoiceNumber>,
}

/// Request to change how a number answers phone calls.
///
/// Construct with [`UpdateVoiceNumberRequest::new`] and set only what
/// changes; unset fields are left out of the request. This type is
/// `#[non_exhaustive]`, so external crates must use the constructor rather
/// than a struct literal.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UpdateVoiceNumberRequest {
    /// Switch voice on or off. Turning it on connects the number for phone
    /// calls and answers in `ring_dashboard` mode unless `voice_mode` says
    /// otherwise; turning it off sets `voice_mode` to `none`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_enabled: Option<bool>,
    /// How the number answers. Without `voice_enabled`,
    /// [`VoiceMode::RingDashboard`] or [`VoiceMode::Agent`] switches voice on
    /// and [`VoiceMode::None`] switches it off. When both are set,
    /// `voice_enabled` wins: `false` switches voice off, and `true` with
    /// [`VoiceMode::None`] answers in `ring_dashboard` mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_mode: Option<VoiceMode>,
    /// The agent that answers in `agent` mode: `Some(Some(id))` sets it,
    /// `Some(None)` clears it (sent as `null`), `None` leaves it unchanged.
    /// Required (here or already stored) for `agent` mode, and the agent
    /// must be switched on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<Option<String>>,
}

impl UpdateVoiceNumberRequest {
    /// Creates an empty request that changes nothing until a field is set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Switches voice on or off.
    pub fn voice_enabled(mut self, voice_enabled: bool) -> Self {
        self.voice_enabled = Some(voice_enabled);
        self
    }

    /// Sets how the number answers.
    pub fn voice_mode(mut self, voice_mode: VoiceMode) -> Self {
        self.voice_mode = Some(voice_mode);
        self
    }

    /// Sets the agent that answers in `agent` mode.
    pub fn agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(Some(agent_id.into()));
        self
    }

    /// Clears the stored agent (sends `agentId: null`).
    pub fn clear_agent_id(mut self) -> Self {
        self.agent_id = Some(None);
        self
    }
}

/// Request to register a number's emergency address.
///
/// Construct with [`RegisterEmergencyAddressRequest::new`] and the builder
/// methods. This type is `#[non_exhaustive]`, so external crates must use
/// the constructor rather than a struct literal.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RegisterEmergencyAddressRequest {
    /// Street address.
    pub street: String,
    /// Apartment, suite or floor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// City.
    pub city: String,
    /// Two-letter state or province code, e.g. `"TX"`.
    pub state: String,
    /// Five-digit ZIP (or ZIP+4) in the US, `"A1A 1A1"` in Canada.
    pub zip: String,
    /// `"US"` or `"CA"`; the API defaults to `"US"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
}

impl RegisterEmergencyAddressRequest {
    /// Creates a request for a US address.
    pub fn new(
        street: impl Into<String>,
        city: impl Into<String>,
        state: impl Into<String>,
        zip: impl Into<String>,
    ) -> Self {
        Self {
            street: street.into(),
            unit: None,
            city: city.into(),
            state: state.into(),
            zip: zip.into(),
            country: None,
        }
    }

    /// Sets the apartment, suite or floor.
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Sets the country (`"US"` or `"CA"`).
    pub fn country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }
}

/// What an agent may do on a call.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct VoiceAgentTools {
    /// Whether the agent may text the caller during the call. It confirms
    /// the number with the caller before sending, and can only send when
    /// [`VoiceAgent::can_send_sms`] is true.
    #[serde(default)]
    pub send_sms: bool,
    /// A number (E.164) for callers who need a person, or `None`. Agents
    /// cannot transfer calls yet and never dial or read out this number:
    /// while it is set, a caller who asks for a person is told the message
    /// will be passed on, and the agent takes their name and number.
    #[serde(default)]
    pub transfer_to: Option<String>,
}

/// Tool settings for [`CreateVoiceAgentRequest::tools`] and
/// [`UpdateVoiceAgentRequest::tools`].
///
/// Unset fields are left out of the request: on create `send_sms` defaults
/// to `true` and `transfer_to` to none; on update they keep their current
/// values.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct VoiceAgentToolsInput {
    /// Whether the agent may text the caller during the call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_sms: Option<bool>,
    /// `Some(Some(number))` sets the number for callers who need a person,
    /// `Some(None)` clears it (sent as `null`), `None` leaves it out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transfer_to: Option<Option<String>>,
}

impl VoiceAgentToolsInput {
    /// Creates empty tool settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allows or stops texting the caller during the call.
    pub fn send_sms(mut self, send_sms: bool) -> Self {
        self.send_sms = Some(send_sms);
        self
    }

    /// Sets the number (E.164) for callers who need a person.
    pub fn transfer_to(mut self, number: impl Into<String>) -> Self {
        self.transfer_to = Some(Some(number.into()));
        self
    }

    /// Clears the number for callers who need a person (sends `null`).
    pub fn clear_transfer_to(mut self) -> Self {
        self.transfer_to = Some(None);
        self
    }
}

/// An AI agent that answers and places phone calls.
///
/// Returned by [`VoiceAgentsResource::list`], [`VoiceAgentsResource::create`],
/// [`VoiceAgentsResource::get`] and [`VoiceAgentsResource::update`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct VoiceAgent {
    /// Unique agent identifier (a UUID).
    pub id: String,
    /// Always `"voice_agent"`.
    #[serde(default = "default_voice_agent_object")]
    pub object: String,
    /// The agent's name.
    pub name: String,
    /// `false` when the agent is switched off; a switched-off agent can't
    /// be pointed at a number or put on a call.
    pub enabled: bool,
    /// Voice id, one of [`VoicesResource::list`].
    pub voice: String,
    /// Human-readable voice name, e.g. `"Ashley (US, warm)"`.
    #[serde(default)]
    pub voice_label: String,
    /// Language tag, e.g. `"en-US"`.
    #[serde(default)]
    pub language: String,
    /// What the agent says when it answers an inbound call (`""` when
    /// unset).
    #[serde(default)]
    pub greeting: String,
    /// Business instructions the agent follows (`""` when unset).
    #[serde(default)]
    pub instructions: String,
    /// What the agent may do on a call.
    #[serde(default)]
    pub tools: VoiceAgentTools,
    /// Whether the agent holds its own scoped sending key, so
    /// `tools.send_sms` can send.
    #[serde(default)]
    pub can_send_sms: bool,
    /// Calls this agent has handled.
    #[serde(default)]
    pub calls_handled: i64,
    /// Average answered duration of those calls, in seconds.
    #[serde(default)]
    pub avg_duration_secs: i64,
    /// When the agent was created (ISO 8601).
    pub created_at: String,
    /// When the agent last changed (ISO 8601).
    pub updated_at: String,
}

/// Response from [`VoiceAgentsResource::list`].
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct VoiceAgentListResponse {
    /// The workspace's agents.
    #[serde(default)]
    pub data: Vec<VoiceAgent>,
}

/// Response from [`VoiceAgentsResource::delete`].
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct DeletedVoiceAgent {
    /// The deleted agent's id.
    pub id: String,
    /// Always `"voice_agent"`.
    #[serde(default = "default_voice_agent_object")]
    pub object: String,
    /// Always `true`.
    #[serde(default)]
    pub deleted: bool,
}

/// Request to create an AI agent.
///
/// Construct with [`CreateVoiceAgentRequest::new`] and the builder methods.
/// This type is `#[non_exhaustive]`, so external crates must use the
/// constructor rather than a struct literal.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CreateVoiceAgentRequest {
    /// The agent's name, 1-80 characters.
    pub name: String,
    /// Whether the agent is switched on; the API defaults to `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// A voice id from [`VoicesResource::list`]. An unknown id falls back to
    /// the default voice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    /// Language tag, up to 16 characters; the API defaults to `"en-US"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// What the agent says when it answers an inbound call, up to 500
    /// characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub greeting: Option<String>,
    /// Business instructions the agent follows, up to 4000 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// What the agent may do on a call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<VoiceAgentToolsInput>,
}

impl CreateVoiceAgentRequest {
    /// Creates a request for an agent called `name`.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enabled: None,
            voice: None,
            language: None,
            greeting: None,
            instructions: None,
            tools: None,
        }
    }

    /// Switches the agent on or off.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Sets the voice id.
    pub fn voice(mut self, voice: impl Into<String>) -> Self {
        self.voice = Some(voice.into());
        self
    }

    /// Sets the language tag.
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Sets what the agent says when it answers.
    pub fn greeting(mut self, greeting: impl Into<String>) -> Self {
        self.greeting = Some(greeting.into());
        self
    }

    /// Sets the business instructions.
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Sets what the agent may do on a call.
    pub fn tools(mut self, tools: VoiceAgentToolsInput) -> Self {
        self.tools = Some(tools);
        self
    }
}

/// Request to update an AI agent.
///
/// Construct with [`UpdateVoiceAgentRequest::new`] and set only what
/// changes; unset fields are left out of the request and keep their current
/// values. This type is `#[non_exhaustive]`, so external crates must use the
/// constructor rather than a struct literal.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UpdateVoiceAgentRequest {
    /// The agent's name, 1-80 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Switch the agent on or off.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// A voice id from [`VoicesResource::list`]. An unknown id falls back to
    /// the default voice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    /// Language tag, up to 16 characters; `""` resets it to `"en-US"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Up to 500 characters; `""` clears it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub greeting: Option<String>,
    /// Up to 4000 characters; `""` clears it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Tool settings to change; tools you leave unset keep their values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<VoiceAgentToolsInput>,
}

impl UpdateVoiceAgentRequest {
    /// Creates an empty request that changes nothing until a field is set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Renames the agent.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Switches the agent on or off.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Sets the voice id.
    pub fn voice(mut self, voice: impl Into<String>) -> Self {
        self.voice = Some(voice.into());
        self
    }

    /// Sets the language tag.
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Sets what the agent says when it answers.
    pub fn greeting(mut self, greeting: impl Into<String>) -> Self {
        self.greeting = Some(greeting.into());
        self
    }

    /// Sets the business instructions.
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Sets the tool settings to change.
    pub fn tools(mut self, tools: VoiceAgentToolsInput) -> Self {
        self.tools = Some(tools);
        self
    }
}

/// A voice an agent can speak with.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Voice {
    /// Voice id to pass as `voice` when creating or updating an agent.
    pub id: String,
    /// Human-readable name, e.g. `"Ashley (US, warm)"`.
    #[serde(default)]
    pub label: String,
    /// Language the voice speaks, e.g. `"en"`.
    #[serde(default)]
    pub language: String,
}

/// Response from [`VoicesResource::list`].
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct VoiceListResponse {
    /// Every voice an agent can use.
    #[serde(default)]
    pub data: Vec<Voice>,
}

/// Voice settings and emergency addresses for the workspace's numbers.
///
/// Every method takes the number's id or its E.164 phone number; the SDK
/// percent-encodes it, so `+15555550188` goes out as `%2B15555550188`.
#[derive(Debug, Clone)]
pub struct VoiceNumbersResource<'a> {
    client: &'a Sendly,
}

impl<'a> VoiceNumbersResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Lists the workspace's active numbers with their voice settings, in
    /// the same order as the dashboard. Requires the `calls:read` scope.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let numbers = client.voice().numbers().list().await?;
    /// for number in &numbers.data {
    ///     let emergency = number.emergency_address.as_ref().map(|e| e.status.as_str());
    ///     println!("{} {} {:?}", number.phone_number, number.voice_mode, emergency);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self) -> Result<VoiceNumberListResponse> {
        let response = self.client.get("/voice/numbers", &[]).await?;
        Ok(response.json().await?)
    }

    /// Fetches a number's voice settings. Requires the `calls:read` scope.
    ///
    /// Answers 404 `number_not_found` ([`Error::NotFound`]) when the number
    /// isn't active in this workspace.
    ///
    /// # Arguments
    ///
    /// * `number` - The number's id or its E.164 phone number
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let number = client.voice().numbers().get("+15555550188").await?;
    /// println!("{} {} {} credits/min", number.voice_enabled, number.voice_mode, number.rate_per_minute.outbound);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, number: &str) -> Result<VoiceNumber> {
        let path = format!("/voice/numbers/{}", encode_number(number)?);
        let response = self.client.get(&path, &[]).await?;
        Ok(response.json().await?)
    }

    /// Changes how a number answers phone calls. Requires the `calls:write`
    /// scope and a live API key.
    ///
    /// This changes what happens when real people call the number. A mode
    /// alone is enough: [`VoiceMode::RingDashboard`] or [`VoiceMode::Agent`]
    /// switches voice on and [`VoiceMode::None`] switches it off, while
    /// `voice_enabled(false)` switches it off whatever the mode. Turning voice
    /// on, a mode alone included, connects the number for calls before the
    /// change is saved: 502 `voice_attach_failed` when that fails (try again),
    /// 503 `voice_unavailable` when numbers can't be switched on yet. `agent`
    /// mode needs an agent (400 `agent_required`, 404 `agent_not_found`) that
    /// is switched on (409 `agent_disabled`).
    ///
    /// # Arguments
    ///
    /// * `number` - The number's id or its E.164 phone number
    /// * `request` - What changes (see [`UpdateVoiceNumberRequest`])
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, UpdateVoiceNumberRequest, VoiceMode};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let number = client
    ///     .voice()
    ///     .numbers()
    ///     .update(
    ///         "+15555550188",
    ///         UpdateVoiceNumberRequest::new()
    ///             .voice_enabled(true)
    ///             .voice_mode(VoiceMode::Agent)
    ///             .agent_id("3c4d5e6f-7081-4293-a4b5-c6d7e8f90a1b"),
    ///     )
    ///     .await?;
    /// println!("{}", number.voice_mode); // agent
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(
        &self,
        number: &str,
        request: UpdateVoiceNumberRequest,
    ) -> Result<VoiceNumber> {
        self.update_with_options(number, request, IdempotentRequestOptions::new())
            .await
    }

    /// [`update`](Self::update) with per-call options (e.g. your own
    /// idempotency key).
    pub async fn update_with_options(
        &self,
        number: &str,
        request: UpdateVoiceNumberRequest,
        options: IdempotentRequestOptions,
    ) -> Result<VoiceNumber> {
        let path = format!("/voice/numbers/{}", encode_number(number)?);
        let response = self
            .client
            .patch_with_idempotency(
                &path,
                &request,
                options.idempotency_key.as_deref(),
                true,
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Registers the street address emergency services are sent to when
    /// someone calls them from this number. Requires the `calls:write` scope
    /// and a live API key.
    ///
    /// A US or Canadian number needs one before it can place calls. The
    /// first registration adds $1.50 a month to the number; registering
    /// again replaces the address without adding the charge a second time.
    /// A malformed field answers 400 `invalid_address` and an address that
    /// couldn't be validated 422 `invalid_address` (both
    /// [`Error::Validation`], which carries the message only); a number
    /// outside the US and Canada answers 400 `e911_not_applicable`.
    ///
    /// # Arguments
    ///
    /// * `number` - The number's id or its E.164 phone number
    /// * `request` - The address (see [`RegisterEmergencyAddressRequest`])
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{RegisterEmergencyAddressRequest, Sendly};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let number = client
    ///     .voice()
    ///     .numbers()
    ///     .register_emergency_address(
    ///         "+15555550188",
    ///         RegisterEmergencyAddressRequest::new("500 Example Ave", "Austin", "TX", "78701")
    ///             .unit("Suite 2"),
    ///     )
    ///     .await?;
    /// println!("{:?}", number.emergency_address.map(|e| e.status));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn register_emergency_address(
        &self,
        number: &str,
        request: RegisterEmergencyAddressRequest,
    ) -> Result<VoiceNumber> {
        self.register_emergency_address_with_options(
            number,
            request,
            IdempotentRequestOptions::new(),
        )
        .await
    }

    /// [`register_emergency_address`](Self::register_emergency_address) with
    /// per-call options (e.g. your own idempotency key).
    pub async fn register_emergency_address_with_options(
        &self,
        number: &str,
        request: RegisterEmergencyAddressRequest,
        options: IdempotentRequestOptions,
    ) -> Result<VoiceNumber> {
        let path = format!(
            "/voice/numbers/{}/emergency-address",
            encode_number(number)?
        );
        for (field, value) in [
            ("street", &request.street),
            ("city", &request.city),
            ("state", &request.state),
            ("zip", &request.zip),
        ] {
            if value.trim().is_empty() {
                return Err(Error::Validation {
                    message: format!("{} is required", field),
                });
            }
        }

        let response = self
            .client
            .post_with_idempotency(
                &path,
                &request,
                options.idempotency_key.as_deref(),
                true,
            )
            .await?;
        Ok(response.json().await?)
    }
}

/// The AI agents that answer and place calls.
#[derive(Debug, Clone)]
pub struct VoiceAgentsResource<'a> {
    client: &'a Sendly,
}

impl<'a> VoiceAgentsResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Lists the workspace's AI agents with their call stats. Requires the
    /// `calls:read` scope.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let agents = client.voice().agents().list().await?;
    /// for agent in &agents.data {
    ///     println!("{} ({}) {} calls", agent.name, agent.voice_label, agent.calls_handled);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self) -> Result<VoiceAgentListResponse> {
        let response = self.client.get("/voice/agents", &[]).await?;
        Ok(response.json().await?)
    }

    /// Creates an AI agent. Requires the `calls:write` scope and a live API
    /// key.
    ///
    /// The agent answers real callers on any number pointed at it and talks
    /// on the calls you place with it. Each agent gets its own scoped
    /// sending key so it can text callers; `can_send_sms` says whether it
    /// has one. A workspace can have up to 20 agents (409 `agent_limit`
    /// after that).
    ///
    /// # Arguments
    ///
    /// * `request` - The agent's name, voice, greeting, instructions and tools
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{CreateVoiceAgentRequest, Sendly, VoiceAgentToolsInput};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let agent = client
    ///     .voice()
    ///     .agents()
    ///     .create(
    ///         CreateVoiceAgentRequest::new("Front desk")
    ///             .voice("ashley")
    ///             .greeting("Thanks for calling Acme, how can I help?")
    ///             .instructions("Answer questions about opening hours and take a message for anything else.")
    ///             .tools(VoiceAgentToolsInput::new().send_sms(true)),
    ///     )
    ///     .await?;
    /// println!("{} {}", agent.id, agent.can_send_sms);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, request: CreateVoiceAgentRequest) -> Result<VoiceAgent> {
        self.create_with_options(request, IdempotentRequestOptions::new())
            .await
    }

    /// [`create`](Self::create) with per-call options (e.g. your own
    /// idempotency key, so a retried request never creates a second agent).
    pub async fn create_with_options(
        &self,
        request: CreateVoiceAgentRequest,
        options: IdempotentRequestOptions,
    ) -> Result<VoiceAgent> {
        if request.name.trim().is_empty() {
            return Err(Error::Validation {
                message: "name is required".to_string(),
            });
        }

        let response = self
            .client
            .post_with_idempotency(
                "/voice/agents",
                &request,
                options.idempotency_key.as_deref(),
                true,
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Fetches one agent. Requires the `calls:read` scope.
    ///
    /// Answers 404 `agent_not_found` ([`Error::NotFound`]) when the agent is
    /// not in this workspace.
    ///
    /// # Arguments
    ///
    /// * `id` - The agent id
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let agent = client.voice().agents().get("3c4d5e6f-7081-4293-a4b5-c6d7e8f90a1b").await?;
    /// println!("{} {}", agent.enabled, agent.greeting);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, id: &str) -> Result<VoiceAgent> {
        let path = format!("/voice/agents/{}", encode_agent_id(id)?);
        let response = self.client.get(&path, &[]).await?;
        Ok(response.json().await?)
    }

    /// Updates an agent. Requires the `calls:write` scope and a live API
    /// key.
    ///
    /// Only the fields you set are sent; tools you leave unset keep their
    /// current values. Changes apply to the next call the agent takes.
    ///
    /// # Arguments
    ///
    /// * `id` - The agent id
    /// * `request` - What changes (see [`UpdateVoiceAgentRequest`])
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{Sendly, UpdateVoiceAgentRequest, VoiceAgentToolsInput};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let agent = client
    ///     .voice()
    ///     .agents()
    ///     .update(
    ///         "3c4d5e6f-7081-4293-a4b5-c6d7e8f90a1b",
    ///         UpdateVoiceAgentRequest::new()
    ///             .greeting("Thanks for calling Acme. How can I help today?")
    ///             .tools(VoiceAgentToolsInput::new().send_sms(false)),
    ///     )
    ///     .await?;
    /// println!("{}", agent.updated_at);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(&self, id: &str, request: UpdateVoiceAgentRequest) -> Result<VoiceAgent> {
        self.update_with_options(id, request, IdempotentRequestOptions::new())
            .await
    }

    /// [`update`](Self::update) with per-call options (e.g. your own
    /// idempotency key).
    pub async fn update_with_options(
        &self,
        id: &str,
        request: UpdateVoiceAgentRequest,
        options: IdempotentRequestOptions,
    ) -> Result<VoiceAgent> {
        let path = format!("/voice/agents/{}", encode_agent_id(id)?);
        let response = self
            .client
            .patch_with_idempotency(
                &path,
                &request,
                options.idempotency_key.as_deref(),
                true,
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Deletes an agent and revokes its sending key. Requires the
    /// `calls:write` scope and a live API key.
    ///
    /// An agent that still answers a number can't be deleted: the API
    /// answers 409 `agent_in_use` ([`Error::Api`] with that `code`). Find
    /// those numbers with [`VoiceNumbersResource::list`] (`voice_mode` is
    /// [`VoiceMode::Agent`] and `agent_id` is this agent) and point them at
    /// another agent or back to the team with
    /// [`VoiceNumbersResource::update`] first.
    ///
    /// # Arguments
    ///
    /// * `id` - The agent id
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let deleted = client.voice().agents().delete("3c4d5e6f-7081-4293-a4b5-c6d7e8f90a1b").await?;
    /// println!("{} {}", deleted.id, deleted.deleted);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, id: &str) -> Result<DeletedVoiceAgent> {
        self.delete_with_options(id, IdempotentRequestOptions::new())
            .await
    }

    /// [`delete`](Self::delete) with per-call options (e.g. your own
    /// idempotency key).
    pub async fn delete_with_options(
        &self,
        id: &str,
        options: IdempotentRequestOptions,
    ) -> Result<DeletedVoiceAgent> {
        let path = format!("/voice/agents/{}", encode_agent_id(id)?);
        let response = self
            .client
            .delete_with_idempotency(&path, options.idempotency_key.as_deref(), true)
            .await?;
        Ok(response.json().await?)
    }
}

/// The voices an agent can speak with.
#[derive(Debug, Clone)]
pub struct VoicesResource<'a> {
    client: &'a Sendly,
}

impl<'a> VoicesResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Lists the voices an agent can speak with. Requires the `calls:read`
    /// scope.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let voices = client.voice().voices().list().await?;
    /// for voice in &voices.data {
    ///     println!("{}: {} ({})", voice.id, voice.label, voice.language);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self) -> Result<VoiceListResponse> {
        let response = self.client.get("/voice/voices", &[]).await?;
        Ok(response.json().await?)
    }
}

/// Voice resource: configure numbers, AI agents and voices for phone calls.
///
/// Refusals arrive as [`Error`] variants: 404s (`voice_not_enabled`,
/// `number_not_found`, `agent_not_found`) are [`Error::NotFound`]; 400s
/// (`invalid_request`, `invalid_voice_mode`, `agent_required`,
/// `invalid_address`, `e911_not_applicable`) and 422 `invalid_address` are
/// [`Error::Validation`]; 429s are [`Error::RateLimit`]; everything else
/// (403 `forbidden` / `live_key_required`, 409 `agent_disabled` /
/// `agent_limit` / `agent_in_use`, 502 `voice_attach_failed` /
/// `carrier_refused`, 503 `voice_unavailable`) is [`Error::Api`] with `code`
/// set.
///
/// # Example
///
/// ```rust,no_run
/// use sendly::{
///     CreateVoiceAgentRequest, RegisterEmergencyAddressRequest, Sendly, UpdateVoiceNumberRequest,
///     VoiceMode,
/// };
///
/// # async fn example() -> sendly::Result<()> {
/// let client = Sendly::new("sk_live_v1_xxx");
///
/// let voices = client.voice().voices().list().await?;
/// let agent = client
///     .voice()
///     .agents()
///     .create(
///         CreateVoiceAgentRequest::new("Front desk")
///             .voice(&voices.data[0].id)
///             .greeting("Thanks for calling Acme, how can I help?"),
///     )
///     .await?;
///
/// client
///     .voice()
///     .numbers()
///     .register_emergency_address(
///         "+15555550188",
///         RegisterEmergencyAddressRequest::new("500 Example Ave", "Austin", "TX", "78701"),
///     )
///     .await?;
/// let number = client
///     .voice()
///     .numbers()
///     .update(
///         "+15555550188",
///         UpdateVoiceNumberRequest::new()
///             .voice_enabled(true)
///             .voice_mode(VoiceMode::Agent)
///             .agent_id(&agent.id),
///     )
///     .await?;
/// println!("{} {}", number.phone_number, number.voice_mode); // ... agent
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct VoiceResource<'a> {
    client: &'a Sendly,
}

impl<'a> VoiceResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Voice settings and emergency addresses for the workspace's numbers.
    pub fn numbers(&self) -> VoiceNumbersResource<'a> {
        VoiceNumbersResource::new(self.client)
    }

    /// The AI agents that answer and place calls.
    pub fn agents(&self) -> VoiceAgentsResource<'a> {
        VoiceAgentsResource::new(self.client)
    }

    /// The voices an agent can speak with.
    pub fn voices(&self) -> VoicesResource<'a> {
        VoicesResource::new(self.client)
    }
}

fn encode_number(number: &str) -> Result<String> {
    if number.trim().is_empty() {
        return Err(Error::Validation {
            message: "Number is required: its id or E.164 phone number".to_string(),
        });
    }
    Ok(urlencoding::encode(number).into_owned())
}

fn encode_agent_id(id: &str) -> Result<String> {
    if id.trim().is_empty() {
        return Err(Error::Validation {
            message: "Agent id is required".to_string(),
        });
    }
    Ok(urlencoding::encode(id).into_owned())
}
