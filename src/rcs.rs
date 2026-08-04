//! RCS Resource — Agent discovery and recipient capability pre-flight
//!
//! RCS is the branded, rich upgrade to SMS: your verified agent name and
//! logo instead of a bare number, plus tappable suggestion chips and rich
//! cards, on Android and iOS 18+ handsets. Send with
//! [`Messages::send_rcs`](crate::Messages::send_rcs).
//!
//! Messages go out through an RCS agent registered for your brand — agents
//! are set up by Sendly support, not self-serve. RCS requires a live API
//! key, and is rolling out gradually: while it is off for an account these
//! endpoints answer 404.
//!
//! Text sends fall back to plain SMS on their own when the recipient's
//! device or network doesn't support RCS, so the capability check here is
//! an optional pre-flight (useful for reporting reach, or for choosing
//! between a card and a text before sending).
//!
//! See <https://sendly.live/docs/rcs> for the full flow.

use serde::Deserialize;

use crate::client::Sendly;
use crate::error::Result;

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

/// List the RCS agents registered for your brand.
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
    /// yet — contact support to set one up for your brand.
    pub async fn list(&self) -> Result<RcsAgentsList> {
        let response = self.client.get("/rcs/agents", &[]).await?;
        Ok(response.json().await?)
    }
}

/// RCS resource — discover agents and check recipient capability.
///
/// # Example
///
/// ```rust,no_run
/// use sendly::Sendly;
///
/// # async fn run() -> Result<(), sendly::Error> {
/// let client = Sendly::new("sk_live_v1_xxx");
///
/// // 1) Find the agents you can send as
/// let agents = client.rcs().agents().list().await?;
/// for agent in &agents.agents {
///     println!("{}: {} (sendable: {})", agent.id, agent.name, agent.sendable);
/// }
///
/// // 2) Optional pre-flight — sending handles the SMS fallback on its own
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
