//! Version and capability negotiation, and authentication.

use crate::{
    AuthMethodId, ClientNesCapabilities, ElicitationCapabilities, NesCapabilities,
    PositionEncodingKind,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The protocol revision. This crate speaks [`ProtocolVersion::V1`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProtocolVersion(pub u16);

impl ProtocolVersion {
    pub const V0: Self = Self(0);
    pub const V1: Self = Self(1);
    pub const LATEST: Self = Self::V1;
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::LATEST
    }
}

/// An empty object whose presence is itself the capability flag.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capability {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeRequest {
    pub protocol_version: ProtocolVersion,
    #[serde(default)]
    pub client_capabilities: ClientCapabilities,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_info: Option<Implementation>,
}

impl InitializeRequest {
    pub fn new(client_capabilities: ClientCapabilities) -> Self {
        Self {
            protocol_version: ProtocolVersion::LATEST,
            client_capabilities,
            client_info: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResponse {
    pub protocol_version: ProtocolVersion,
    #[serde(default)]
    pub agent_capabilities: AgentCapabilities,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub auth_methods: Vec<AuthMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_info: Option<Implementation>,
}

/// Name and version of the program on either end.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Implementation {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    #[serde(default)]
    pub fs: FileSystemCapabilities,
    #[serde(default)]
    pub terminal: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<ClientSessionCapabilities>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elicitation: Option<ElicitationCapabilities>,
    /// Whether the client understands the `plan_update` and `plan_removed`
    /// session updates, as opposed to whole-plan `plan` reports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan: Option<Capability>,
    #[serde(default)]
    pub auth: ClientAuthCapabilities,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nes: Option<ClientNesCapabilities>,
    /// Column encodings the client accepts in a
    /// [`Position`](crate::Position), best first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub position_encodings: Vec<PositionEncodingKind>,
}

/// Which [`AuthMethod`] variants the agent may offer. A client that cannot
/// run a terminal must not be handed [`AuthMethod::Terminal`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientAuthCapabilities {
    #[serde(default)]
    pub terminal: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSystemCapabilities {
    #[serde(default)]
    pub read_text_file: bool,
    #[serde(default)]
    pub write_text_file: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientSessionCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_options: Option<SessionConfigOptionsCapabilities>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionConfigOptionsCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boolean: Option<Capability>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilities {
    #[serde(default)]
    pub load_session: bool,
    #[serde(default)]
    pub prompt_capabilities: PromptCapabilities,
    #[serde(default)]
    pub mcp_capabilities: McpCapabilities,
    #[serde(default)]
    pub session_capabilities: SessionCapabilities,
    #[serde(default)]
    pub auth: AgentAuthCapabilities,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Capability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nes: Option<NesCapabilities>,
    /// The one encoding the agent picked from the client's
    /// [`position_encodings`](ClientCapabilities::position_encodings).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position_encoding: Option<PositionEncodingKind>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptCapabilities {
    #[serde(default)]
    pub image: bool,
    #[serde(default)]
    pub audio: bool,
    #[serde(default)]
    pub embedded_context: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpCapabilities {
    #[serde(default)]
    pub http: bool,
    #[serde(default)]
    pub sse: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub list: Option<Capability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delete: Option<Capability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub additional_directories: Option<Capability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fork: Option<Capability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume: Option<Capability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub close: Option<Capability>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentAuthCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logout: Option<Capability>,
}

/// A way to sign in. [`Agent`](AuthMethod::Agent) is the untagged fallback:
/// methods that predate the `type` tag still arrive without one, and they
/// mean "the agent handles this itself".
///
/// Only offer [`EnvVar`](AuthMethod::EnvVar) or
/// [`Terminal`](AuthMethod::Terminal) to a client that advertised
/// [`ClientAuthCapabilities`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthMethod {
    EnvVar(AuthMethodEnvVar),
    Terminal(AuthMethodTerminal),
    #[serde(untagged)]
    Agent(AuthMethodAgent),
}

impl AuthMethod {
    pub fn id(&self) -> &AuthMethodId {
        match self {
            Self::EnvVar(method) => &method.id,
            Self::Terminal(method) => &method.id,
            Self::Agent(method) => &method.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::EnvVar(method) => &method.name,
            Self::Terminal(method) => &method.name,
            Self::Agent(method) => &method.name,
        }
    }

    pub fn description(&self) -> Option<&str> {
        match self {
            Self::EnvVar(method) => method.description.as_deref(),
            Self::Terminal(method) => method.description.as_deref(),
            Self::Agent(method) => method.description.as_deref(),
        }
    }
}

/// The agent signs in on its own — an API key already in its environment, or
/// an OAuth flow it opens a browser for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethodAgent {
    pub id: AuthMethodId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// The client collects values from the user and passes them to the agent as
/// environment variables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethodEnvVar {
    pub id: AuthMethodId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub vars: Vec<AuthEnvVar>,
    /// Where the user can go to obtain the credentials.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthEnvVar {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Secret by default — a client that guesses wrong echoes an API key to
    /// the screen.
    #[serde(default = "yes", skip_serializing_if = "is_yes")]
    pub secret: bool,
    #[serde(default, skip_serializing_if = "is_no")]
    pub optional: bool,
}

fn yes() -> bool {
    true
}

fn is_yes(value: &bool) -> bool {
    *value
}

fn is_no(value: &bool) -> bool {
    !*value
}

/// The client runs a terminal for the user to sign in interactively.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethodTerminal {
    pub id: AuthMethodId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateRequest {
    pub method_id: AuthMethodId,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthenticateResponse {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogoutRequest {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogoutResponse {}
