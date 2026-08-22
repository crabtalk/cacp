//! Everything an agent asks of a client, as one value.

use crate::Reply;

/// One inbound call from the agent.
///
/// A variant carrying a [`Reply`] is a request: answer it, or drop the reply to
/// decline. The rest are notifications.
#[derive(Debug)]
#[non_exhaustive]
pub enum Event {
    Update(proto::SessionNotification),
    Permission(
        proto::RequestPermissionRequest,
        Reply<proto::RequestPermissionResponse>,
    ),
    ReadTextFile(
        proto::ReadTextFileRequest,
        Reply<proto::ReadTextFileResponse>,
    ),
    WriteTextFile(
        proto::WriteTextFileRequest,
        Reply<proto::WriteTextFileResponse>,
    ),
    CreateTerminal(
        proto::CreateTerminalRequest,
        Reply<proto::CreateTerminalResponse>,
    ),
    TerminalOutput(
        proto::TerminalOutputRequest,
        Reply<proto::TerminalOutputResponse>,
    ),
    ReleaseTerminal(
        proto::ReleaseTerminalRequest,
        Reply<proto::ReleaseTerminalResponse>,
    ),
    WaitForTerminalExit(
        proto::WaitForTerminalExitRequest,
        Reply<proto::WaitForTerminalExitResponse>,
    ),
    KillTerminal(
        proto::KillTerminalRequest,
        Reply<proto::KillTerminalResponse>,
    ),
    CreateElicitation(
        proto::CreateElicitationRequest,
        Reply<proto::CreateElicitationResponse>,
    ),
    CompleteElicitation(proto::CompleteElicitationNotification),
    ConnectMcp(proto::ConnectMcpRequest, Reply<proto::ConnectMcpResponse>),
    MessageMcp(proto::MessageMcpRequest, Reply<proto::MessageMcpResponse>),
    NotifyMcp(proto::MessageMcpNotification),
    DisconnectMcp(
        proto::DisconnectMcpRequest,
        Reply<proto::DisconnectMcpResponse>,
    ),
    ExtRequest {
        method: String,
        params: serde_json::Value,
        reply: Reply<serde_json::Value>,
    },
    ExtNotification {
        method: String,
        params: serde_json::Value,
    },
}
