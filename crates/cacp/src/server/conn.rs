//! An agent's handle on the connected client.

use crate::Peer;
use proto::method::client as method;

/// An agent's handle on the connected client.
#[derive(Clone)]
pub struct ClientConn(pub Peer);

impl ClientConn {
    pub fn session_update(&self, notification: proto::SessionNotification) -> proto::Result<()> {
        self.0.notify(method::SESSION_UPDATE, notification)
    }

    pub async fn request_permission(
        &self,
        request: proto::RequestPermissionRequest,
    ) -> proto::Result<proto::RequestPermissionResponse> {
        self.0
            .request(method::SESSION_REQUEST_PERMISSION, request)
            .await
    }

    pub async fn read_text_file(
        &self,
        request: proto::ReadTextFileRequest,
    ) -> proto::Result<proto::ReadTextFileResponse> {
        self.0.request(method::FS_READ_TEXT_FILE, request).await
    }

    pub async fn write_text_file(
        &self,
        request: proto::WriteTextFileRequest,
    ) -> proto::Result<proto::WriteTextFileResponse> {
        self.0.request(method::FS_WRITE_TEXT_FILE, request).await
    }

    pub async fn create_terminal(
        &self,
        request: proto::CreateTerminalRequest,
    ) -> proto::Result<proto::CreateTerminalResponse> {
        self.0.request(method::TERMINAL_CREATE, request).await
    }

    pub async fn terminal_output(
        &self,
        request: proto::TerminalOutputRequest,
    ) -> proto::Result<proto::TerminalOutputResponse> {
        self.0.request(method::TERMINAL_OUTPUT, request).await
    }

    pub async fn release_terminal(
        &self,
        request: proto::ReleaseTerminalRequest,
    ) -> proto::Result<proto::ReleaseTerminalResponse> {
        self.0.request(method::TERMINAL_RELEASE, request).await
    }

    pub async fn wait_for_terminal_exit(
        &self,
        request: proto::WaitForTerminalExitRequest,
    ) -> proto::Result<proto::WaitForTerminalExitResponse> {
        self.0
            .request(method::TERMINAL_WAIT_FOR_EXIT, request)
            .await
    }

    pub async fn kill_terminal(
        &self,
        request: proto::KillTerminalRequest,
    ) -> proto::Result<proto::KillTerminalResponse> {
        self.0.request(method::TERMINAL_KILL, request).await
    }

    pub async fn create_elicitation(
        &self,
        request: proto::CreateElicitationRequest,
    ) -> proto::Result<proto::CreateElicitationResponse> {
        self.0.request(method::ELICITATION_CREATE, request).await
    }

    pub fn complete_elicitation(
        &self,
        notification: proto::CompleteElicitationNotification,
    ) -> proto::Result<()> {
        self.0.notify(method::ELICITATION_COMPLETE, notification)
    }

    /// Ask the client to open an MCP connection this agent cannot open
    /// itself.
    pub async fn connect_mcp(
        &self,
        request: proto::ConnectMcpRequest,
    ) -> proto::Result<proto::ConnectMcpResponse> {
        self.0.request(method::MCP_CONNECT, request).await
    }

    pub async fn message_mcp(
        &self,
        request: proto::MessageMcpRequest,
    ) -> proto::Result<proto::MessageMcpResponse> {
        self.0.request(method::MCP_MESSAGE, request).await
    }

    pub fn notify_mcp(&self, notification: proto::MessageMcpNotification) -> proto::Result<()> {
        self.0.notify(method::MCP_MESSAGE, notification)
    }

    pub async fn disconnect_mcp(
        &self,
        request: proto::DisconnectMcpRequest,
    ) -> proto::Result<proto::DisconnectMcpResponse> {
        self.0.request(method::MCP_DISCONNECT, request).await
    }

    /// Call a method the spec does not define. Names must start with `_`.
    pub async fn ext_request(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> proto::Result<serde_json::Value> {
        self.0.request(method, params).await
    }

    pub fn ext_notification(&self, method: &str, params: serde_json::Value) -> proto::Result<()> {
        self.0.notify(method, params)
    }
}
