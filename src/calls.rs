//! Voice Calls Resource: place phone calls handled by your AI agents, list
//! and inspect calls, end one early, and fetch recordings.
//!
//! A workspace phone number can take and place phone calls. Over the API a
//! call is placed to a US or Canadian number and answered by one of the
//! workspace's AI agents (configured in the dashboard under Calls → Agents).
//! Reads need the `calls:read` scope and writes `calls:write`; placing or
//! ending a call also needs a live API key (a test key answers 403
//! `live_key_required`).
//!
//! Calls are billed per started minute in credits (1 credit = $0.01),
//! prepaid from the workspace balance: an outbound call costs 2 credits/min,
//! plus 8 credits/min while an AI agent is on the line, so an agent-handled
//! outbound call is 10 credits/min. Unanswered calls cost nothing. The
//! `from` number must be voice-enabled in the dashboard (Calls → Settings)
//! and have an emergency address registered before it can place calls.
//!
//! Voice is being enabled workspace by workspace; until it is on for yours,
//! every call route answers 404 `voice_not_enabled`
//! ([`Error::NotFound`](crate::Error::NotFound)). Every `POST` carries an
//! `Idempotency-Key`, generated per call or your own through the
//! `*_with_options` variants.
//!
//! See <https://sendly.live/docs/voice> for the full guide.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::client::Sendly;
use crate::error::{Error, Result};
use crate::models::IdempotentRequestOptions;

/// Where a call is in its lifecycle.
///
/// `Ringing` and `Active` are live; every other known variant is terminal.
/// `Suspended` can appear on an internal (browser-to-browser) call whose
/// media dropped and may recover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallStatus {
    /// The far end is being rung; nothing has been charged yet.
    Ringing,
    /// Answered and in progress.
    Active,
    /// Ended after being answered.
    Completed,
    /// Nobody answered before the ring deadline.
    NoAnswer,
    /// The far end was busy.
    Busy,
    /// Cancelled before it was answered (see `hangup_class`).
    Cancelled,
    /// The far end declined the call.
    Declined,
    /// The call could not be set up or was cut short by a failure.
    Failed,
    /// An internal call whose media dropped and may recover.
    Suspended,
    /// A status this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

impl CallStatus {
    /// The status as the API spells it (e.g. `"no_answer"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ringing => "ringing",
            Self::Active => "active",
            Self::Completed => "completed",
            Self::NoAnswer => "no_answer",
            Self::Busy => "busy",
            Self::Cancelled => "cancelled",
            Self::Declined => "declined",
            Self::Failed => "failed",
            Self::Suspended => "suspended",
            Self::Unknown => "unknown",
        }
    }

    /// `true` while the call is ringing or active.
    pub fn is_live(&self) -> bool {
        matches!(self, Self::Ringing | Self::Active)
    }
}

impl std::fmt::Display for CallStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Which way the call went.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallDirection {
    /// Someone called one of your numbers.
    Inbound,
    /// Your workspace placed the call.
    Outbound,
    /// A direction this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

impl CallDirection {
    /// The direction as the API spells it.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Inbound => "inbound",
            Self::Outbound => "outbound",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for CallDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What kind of call this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallKind {
    /// A phone call to or from a phone number.
    Pstn,
    /// A browser-to-browser call between teammates (free, no numbers).
    Internal,
    /// A kind this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

impl CallKind {
    /// The kind as the API spells it.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pstn => "pstn",
            Self::Internal => "internal",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for CallKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Who answered (or placed) the call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallHandledBy {
    /// One of the workspace's AI agents.
    Agent,
    /// A teammate in the dashboard.
    Dashboard,
    /// A handler this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

impl CallHandledBy {
    /// The handler as the API spells it.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Dashboard => "dashboard",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for CallHandledBy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// How the call is billed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallBilling {
    /// A phone call in progress, charged per started minute.
    Metered,
    /// Ended; `credits_charged` is final.
    Settled,
    /// Never charged (internal calls, or rows from before metering).
    Unbilled,
    /// A billing state this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

impl CallBilling {
    /// The billing state as the API spells it.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Metered => "metered",
            Self::Settled => "settled",
            Self::Unbilled => "unbilled",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for CallBilling {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// State of a call's recording.
///
/// On [`Call::recording_status`] the API sends `null` when there is no
/// recording, so that field is an `Option` and `None` (the variant) never
/// appears there. [`CallRecording::status`] uses the full set, including
/// `None` for a call that has no recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallRecordingStatus {
    /// No recording for this call (recording off, or never answered).
    None,
    /// The call is being recorded.
    Recording,
    /// The recording can be fetched.
    Ready,
    /// Recording failed.
    Failed,
    /// A state this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

impl CallRecordingStatus {
    /// The state as the API spells it.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Recording => "recording",
            Self::Ready => "ready",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for CallRecordingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Who said a transcript line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CallTranscriptSpeaker {
    /// The person on the phone.
    Caller,
    /// The AI agent.
    Agent,
    /// A speaker this SDK version doesn't know yet.
    #[serde(other)]
    Unknown,
}

impl CallTranscriptSpeaker {
    /// The speaker as the API spells it.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Caller => "caller",
            Self::Agent => "agent",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for CallTranscriptSpeaker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One line of an agent-handled call's transcript.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CallTranscriptLine {
    /// Who spoke.
    pub speaker: CallTranscriptSpeaker,
    /// What was said.
    pub text: String,
    /// Milliseconds from the start of the call.
    #[serde(default)]
    pub at_ms: i64,
}

fn default_call_object() -> String {
    "call".to_string()
}

/// A phone call (or a browser-to-browser call between teammates).
///
/// Returned by every method on [`CallsResource`]. `transcript` is only
/// present on [`CallsResource::get`] for agent-handled calls; it is `None`
/// everywhere else.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Call {
    /// Unique call identifier (a UUID).
    pub id: String,
    /// Always `"call"`.
    #[serde(default = "default_call_object")]
    pub object: String,
    /// A phone call or an internal call.
    pub kind: CallKind,
    /// Inbound or outbound.
    pub direction: CallDirection,
    /// Where the call is in its lifecycle.
    pub status: CallStatus,
    /// Who answered (or placed) it: an AI agent or the team in the dashboard.
    pub handled_by: CallHandledBy,
    /// The AI agent on the call, when `handled_by` is `Agent`.
    #[serde(default)]
    pub agent_id: Option<String>,
    /// The number the call was placed from (E.164). `None` on internal calls.
    #[serde(default, rename = "from")]
    pub from_number: Option<String>,
    /// The number that was called (E.164). `None` on internal calls.
    #[serde(default)]
    pub to: Option<String>,
    /// Display name of the calling side, when known.
    #[serde(default)]
    pub caller_name: Option<String>,
    /// Display name of the called side, when known.
    #[serde(default)]
    pub callee_name: Option<String>,
    /// When the call started ringing (ISO 8601).
    pub started_at: String,
    /// When the call was answered (ISO 8601), or `None` if it never was.
    #[serde(default)]
    pub answered_at: Option<String>,
    /// When the call ended (ISO 8601), or `None` while it is live.
    #[serde(default)]
    pub ended_at: Option<String>,
    /// Answered seconds; `0` until the call has ended.
    #[serde(default)]
    pub duration_secs: i64,
    /// Credits charged so far (final once `billing` is `Settled`).
    #[serde(default)]
    pub credits_charged: i64,
    /// Whether the call is being metered, is settled, or was never charged.
    pub billing: CallBilling,
    /// Why the call ended (e.g. `"normal"`, `"ring_timeout"`,
    /// `"credits_exhausted"`), or `None` while it is live. See the API
    /// reference for the full vocabulary; anything unrecognised is
    /// reported as `"ended"`.
    #[serde(default)]
    pub hangup_class: Option<String>,
    /// Recording state, or `None` when there is no recording.
    #[serde(default)]
    pub recording_status: Option<CallRecordingStatus>,
    /// The key/value pairs attached on create; empty when none.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// What was said, for agent-handled calls fetched with
    /// [`CallsResource::get`]. `None` on every other read.
    #[serde(default)]
    pub transcript: Option<Vec<CallTranscriptLine>>,
}

impl Call {
    /// `true` while the call is ringing or active.
    pub fn is_live(&self) -> bool {
        self.status.is_live()
    }

    /// `true` once the call has reached a terminal status. `false` for
    /// `suspended` (media may recover) and for `Unknown` (a status this
    /// version doesn't recognise is not known to be terminal).
    pub fn is_ended(&self) -> bool {
        matches!(
            self.status,
            CallStatus::Completed
                | CallStatus::NoAnswer
                | CallStatus::Busy
                | CallStatus::Cancelled
                | CallStatus::Declined
                | CallStatus::Failed
        )
    }
}

/// Page metadata returned by [`CallsResource::list`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CallPagination {
    /// Calls matching the filters, across all pages.
    #[serde(default)]
    pub total: i64,
    /// Page size that was applied.
    #[serde(default)]
    pub limit: i64,
    /// Offset that was applied.
    #[serde(default)]
    pub offset: i64,
    /// Whether another page follows.
    #[serde(default)]
    pub has_more: bool,
}

/// One page of calls, newest first.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct CallListResponse {
    /// The calls on this page.
    #[serde(default)]
    pub data: Vec<Call>,
    /// Page metadata.
    #[serde(default)]
    pub pagination: CallPagination,
}

/// A call's recording, from [`CallsResource::recording`].
///
/// `url`, `expires_at` and `content_type` are `Some` only when `status` is
/// [`CallRecordingStatus::Ready`]. The URL is signed and valid for five
/// minutes from the moment of the request. Recordings are Ogg/Opus; agent
/// calls are recorded dual-channel (caller left, agent right).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CallRecording {
    /// The call this recording belongs to.
    pub call_id: String,
    /// Whether there is a recording and whether it can be fetched yet.
    pub status: CallRecordingStatus,
    /// Signed download URL, valid until `expires_at`.
    #[serde(default)]
    pub url: Option<String>,
    /// When `url` stops working (ISO 8601).
    #[serde(default)]
    pub expires_at: Option<String>,
    /// `"audio/ogg"` when ready.
    #[serde(default)]
    pub content_type: Option<String>,
}

/// Request to place a call handled by an AI agent.
///
/// Construct with [`CreateCallRequest::new`] and the builder methods. This
/// type is `#[non_exhaustive]`, so external crates must use the constructor
/// rather than a struct literal.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CreateCallRequest {
    /// Number to call, E.164 (US or Canada).
    pub to: String,
    /// The AI agent that talks on the call.
    pub agent_id: String,
    /// A voice-enabled number in your workspace to call from. Required when
    /// the workspace has more than one voice-enabled number.
    #[serde(rename = "from", skip_serializing_if = "Option::is_none")]
    pub from_number: Option<String>,
    /// Extra instructions for this call only (up to 2000 characters).
    /// Appended to the agent's instructions; not echoed back.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Up to 20 string pairs stored on the call and echoed on every read
    /// and `call.*` webhook. Keys are 1-40 characters matching
    /// `[A-Za-z0-9_.:-]`; values up to 500 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

impl CreateCallRequest {
    /// Creates a request to call `to` with the agent `agent_id`.
    pub fn new(to: impl Into<String>, agent_id: impl Into<String>) -> Self {
        Self {
            to: to.into(),
            agent_id: agent_id.into(),
            from_number: None,
            context: None,
            metadata: None,
        }
    }

    /// Sets the number to call from.
    pub fn from_number(mut self, from_number: impl Into<String>) -> Self {
        self.from_number = Some(from_number.into());
        self
    }

    /// Sets extra instructions for this call only.
    pub fn context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Replaces the metadata map.
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Adds one metadata pair.
    pub fn metadata_entry(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }
}

/// Filters and paging for [`CallsResource::list`].
#[derive(Debug, Clone, Default)]
pub struct ListCallsOptions {
    /// Page size (default 50, max 100).
    pub limit: Option<u32>,
    /// Calls to skip.
    pub offset: Option<u32>,
    /// Only calls in this status.
    pub status: Option<CallStatus>,
    /// Only inbound or only outbound calls.
    pub direction: Option<CallDirection>,
    /// Only phone calls or only internal calls.
    pub kind: Option<CallKind>,
    /// Only calls handled by this agent.
    pub agent_id: Option<String>,
    /// Only calls to this number (E.164, exact match).
    pub to: Option<String>,
    /// Only calls from this number (E.164, exact match).
    pub from_number: Option<String>,
}

impl ListCallsOptions {
    /// Creates new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the page size (clamped to 100).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit.min(100));
        self
    }

    /// Sets the offset.
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Filters by status.
    pub fn status(mut self, status: CallStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Filters by direction.
    pub fn direction(mut self, direction: CallDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Filters by kind.
    pub fn kind(mut self, kind: CallKind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// Filters by the agent that handled the call.
    pub fn agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    /// Filters by the called number.
    pub fn to(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
        self
    }

    /// Filters by the calling number.
    pub fn from_number(mut self, from_number: impl Into<String>) -> Self {
        self.from_number = Some(from_number.into());
        self
    }

    pub(crate) fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        if let Some(limit) = self.limit {
            params.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(offset) = self.offset {
            params.push(("offset".to_string(), offset.to_string()));
        }
        if let Some(status) = self.status {
            params.push(("status".to_string(), status.as_str().to_string()));
        }
        if let Some(direction) = self.direction {
            params.push(("direction".to_string(), direction.as_str().to_string()));
        }
        if let Some(kind) = self.kind {
            params.push(("kind".to_string(), kind.as_str().to_string()));
        }
        if let Some(ref agent_id) = self.agent_id {
            params.push(("agentId".to_string(), agent_id.clone()));
        }
        if let Some(ref to) = self.to {
            params.push(("to".to_string(), to.clone()));
        }
        if let Some(ref from_number) = self.from_number {
            params.push(("from".to_string(), from_number.clone()));
        }

        params
    }
}

/// Calls resource: phone calls handled by your AI agents.
///
/// Place a call ([`create`](Self::create)), follow it
/// ([`list`](Self::list), [`get`](Self::get)), end it early
/// ([`hangup`](Self::hangup)) and fetch the recording
/// ([`recording`](Self::recording)). Configuration (switching voice on for
/// a number, choosing how it answers, registering an emergency address,
/// creating agents) is done in the dashboard; use
/// [`NumbersResource::list`](crate::NumbersResource::list) and read
/// `voice_enabled` / `voice_mode` to find a number to call from.
///
/// Refusals arrive as [`Error`] variants: 402 `insufficient_credits` is
/// [`Error::InsufficientCredits`], 404s (`voice_not_enabled`,
/// `call_not_found`, `agent_not_found`, `number_not_found`) are
/// [`Error::NotFound`], 400s (`agent_required`, `invalid_number`,
/// `invalid_metadata`, `from_number_required`, `destination_not_supported`)
/// are [`Error::Validation`], 429s are [`Error::RateLimit`], and everything
/// else (403 `live_key_required`, 409 `agent_disabled` / `no_voice_number` /
/// `lines_busy`, 428 `e911_required`, 503 `voice_unavailable`) is
/// [`Error::Api`] with `code` set.
#[derive(Debug, Clone)]
pub struct CallsResource<'a> {
    client: &'a Sendly,
}

impl<'a> CallsResource<'a> {
    pub(crate) fn new(client: &'a Sendly) -> Self {
        Self { client }
    }

    /// Places a call that one of your AI agents handles. Requires the
    /// `calls:write` scope and a live API key.
    ///
    /// Returns the call while it is still `Ringing`; poll
    /// [`get`](Self::get) or subscribe to the `call.started` /
    /// `call.completed` webhooks to follow it. Needs at least one minute of
    /// credit at the agent-outbound rate (10 credits) up front
    /// ([`Error::InsufficientCredits`] otherwise) and a `from` number with
    /// an emergency address registered (428 `e911_required` otherwise).
    ///
    /// # Arguments
    ///
    /// * `request` - Who to call and which agent talks (see [`CreateCallRequest`])
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{CreateCallRequest, Sendly};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let call = client
    ///     .calls()
    ///     .create(
    ///         CreateCallRequest::new("+15555550123", "3c4d5e6f-7081-4293-a4b5-c6d7e8f90a1b")
    ///             .from_number("+15555550188")
    ///             .context("You are calling Jordan to confirm the 3pm appointment on Tuesday.")
    ///             .metadata_entry("crmId", "lead_8812"),
    ///     )
    ///     .await?;
    /// println!("{} {}", call.id, call.status); // ... ringing
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, request: CreateCallRequest) -> Result<Call> {
        self.create_with_options(request, IdempotentRequestOptions::new())
            .await
    }

    /// [`create`](Self::create) with per-call options (e.g. your own
    /// idempotency key, so a retried request never places a second call).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{CreateCallRequest, IdempotentRequestOptions, Sendly};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let call = client
    ///     .calls()
    ///     .create_with_options(
    ///         CreateCallRequest::new("+15555550123", "3c4d5e6f-7081-4293-a4b5-c6d7e8f90a1b"),
    ///         IdempotentRequestOptions::new().idempotency_key("call-lead_8812-2026-09-12"),
    ///     )
    ///     .await?;
    /// println!("{}", call.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_with_options(
        &self,
        request: CreateCallRequest,
        options: IdempotentRequestOptions,
    ) -> Result<Call> {
        if request.to.trim().is_empty() {
            return Err(Error::Validation {
                message: "to is required".to_string(),
            });
        }
        if request.agent_id.trim().is_empty() {
            return Err(Error::Validation {
                message: "agentId is required".to_string(),
            });
        }

        let response = self
            .client
            .post_with_idempotency(
                "/calls",
                &request,
                options.idempotency_key.as_deref(),
                true,
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Lists calls, newest first. Requires the `calls:read` scope.
    ///
    /// Live calls are reconciled before they are returned, so a ring past
    /// its deadline shows as `NoAnswer`.
    ///
    /// # Arguments
    ///
    /// * `options` - Optional filters and paging (see [`ListCallsOptions`])
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{CallStatus, ListCallsOptions, Sendly};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let page = client
    ///     .calls()
    ///     .list(Some(ListCallsOptions::new().status(CallStatus::Completed).limit(20)))
    ///     .await?;
    /// for call in &page.data {
    ///     println!("{} {}s {} credits", call.id, call.duration_secs, call.credits_charged);
    /// }
    /// if page.pagination.has_more {
    ///     println!("{} more", page.pagination.total - page.data.len() as i64);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self, options: Option<ListCallsOptions>) -> Result<CallListResponse> {
        let query = options.map(|o| o.to_query_params()).unwrap_or_default();

        let response = self.client.get("/calls", &query).await?;
        Ok(response.json().await?)
    }

    /// Fetches one call. Requires the `calls:read` scope.
    ///
    /// For agent-handled calls the response carries the `transcript`.
    /// Answers 404 `call_not_found` ([`Error::NotFound`]) when the call is
    /// not in this workspace.
    ///
    /// # Arguments
    ///
    /// * `id` - The call id
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let call = client.calls().get("6f1c2d3e-4a5b-4c6d-8e9f-0a1b2c3d4e5f").await?;
    /// println!("{} {:?}", call.status, call.hangup_class);
    /// for line in call.transcript.unwrap_or_default() {
    ///     println!("[{}] {}: {}", line.at_ms, line.speaker, line.text);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, id: &str) -> Result<Call> {
        let path = format!("/calls/{}", encode_call_id(id)?);
        let response = self.client.get(&path, &[]).await?;
        Ok(response.json().await?)
    }

    /// Ends a call. Requires the `calls:write` scope and a live API key.
    ///
    /// A `Ringing` call becomes `Cancelled` (`hangup_class`
    /// `"caller_cancelled"`); an `Active` call becomes `Completed`
    /// (`"normal"`). A call that has already ended is returned unchanged.
    ///
    /// # Arguments
    ///
    /// * `id` - The call id
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::Sendly;
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let call = client.calls().hangup("6f1c2d3e-4a5b-4c6d-8e9f-0a1b2c3d4e5f").await?;
    /// println!("{}", call.status); // cancelled or completed
    /// # Ok(())
    /// # }
    /// ```
    pub async fn hangup(&self, id: &str) -> Result<Call> {
        self.hangup_with_options(id, IdempotentRequestOptions::new())
            .await
    }

    /// [`hangup`](Self::hangup) with per-call options (e.g. your own
    /// idempotency key).
    pub async fn hangup_with_options(
        &self,
        id: &str,
        options: IdempotentRequestOptions,
    ) -> Result<Call> {
        let path = format!("/calls/{}/hangup", encode_call_id(id)?);
        let response = self
            .client
            .post_with_idempotency(
                &path,
                &serde_json::json!({}),
                options.idempotency_key.as_deref(),
                true,
            )
            .await?;
        Ok(response.json().await?)
    }

    /// Fetches a call's recording. Requires the `calls:read` scope.
    ///
    /// `url` is `Some` only when `status` is
    /// [`CallRecordingStatus::Ready`]; it is signed and valid for five
    /// minutes (`expires_at`). Download it straight away rather than storing
    /// the URL.
    ///
    /// # Arguments
    ///
    /// * `id` - The call id
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use sendly::{CallRecordingStatus, Sendly};
    ///
    /// # async fn example() -> sendly::Result<()> {
    /// let client = Sendly::new("sk_live_v1_xxx");
    ///
    /// let recording = client.calls().recording("6f1c2d3e-4a5b-4c6d-8e9f-0a1b2c3d4e5f").await?;
    /// if recording.status == CallRecordingStatus::Ready {
    ///     println!("{} until {}", recording.url.unwrap(), recording.expires_at.unwrap());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn recording(&self, id: &str) -> Result<CallRecording> {
        let path = format!("/calls/{}/recording", encode_call_id(id)?);
        let response = self.client.get(&path, &[]).await?;
        Ok(response.json().await?)
    }
}

fn encode_call_id(id: &str) -> Result<String> {
    if id.trim().is_empty() {
        return Err(Error::Validation {
            message: "Call id is required".to_string(),
        });
    }
    Ok(urlencoding::encode(id).into_owned())
}
