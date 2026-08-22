//! Every [`Client`] call, forwarded as an [`Event`].

use crate::{Channel, Event, Reply};
use cacp::Client;
use tokio::sync::oneshot;

impl Client for Channel {
    async fn session_update(&self, notification: proto::SessionNotification) {
        self.tell(Event::Update(notification));
    }

    async fn request_permission(
        &self,
        request: proto::RequestPermissionRequest,
    ) -> proto::Result<proto::RequestPermissionResponse> {
        self.ask(|reply| Event::Permission(request, reply)).await
    }

    async fn read_text_file(
        &self,
        request: proto::ReadTextFileRequest,
    ) -> proto::Result<proto::ReadTextFileResponse> {
        self.ask(|reply| Event::ReadTextFile(request, reply)).await
    }

    async fn write_text_file(
        &self,
        request: proto::WriteTextFileRequest,
    ) -> proto::Result<proto::WriteTextFileResponse> {
        self.ask(|reply| Event::WriteTextFile(request, reply)).await
    }

    async fn create_terminal(
        &self,
        request: proto::CreateTerminalRequest,
    ) -> proto::Result<proto::CreateTerminalResponse> {
        self.ask(|reply| Event::CreateTerminal(request, reply))
            .await
    }

    async fn terminal_output(
        &self,
        request: proto::TerminalOutputRequest,
    ) -> proto::Result<proto::TerminalOutputResponse> {
        self.ask(|reply| Event::TerminalOutput(request, reply))
            .await
    }

    async fn release_terminal(
        &self,
        request: proto::ReleaseTerminalRequest,
    ) -> proto::Result<proto::ReleaseTerminalResponse> {
        self.ask(|reply| Event::ReleaseTerminal(request, reply))
            .await
    }

    async fn wait_for_terminal_exit(
        &self,
        request: proto::WaitForTerminalExitRequest,
    ) -> proto::Result<proto::WaitForTerminalExitResponse> {
        self.ask(|reply| Event::WaitForTerminalExit(request, reply))
            .await
    }

    async fn kill_terminal(
        &self,
        request: proto::KillTerminalRequest,
    ) -> proto::Result<proto::KillTerminalResponse> {
        self.ask(|reply| Event::KillTerminal(request, reply)).await
    }

    async fn create_elicitation(
        &self,
        request: proto::CreateElicitationRequest,
    ) -> proto::Result<proto::CreateElicitationResponse> {
        self.ask(|reply| Event::CreateElicitation(request, reply))
            .await
    }

    async fn complete_elicitation(&self, notification: proto::CompleteElicitationNotification) {
        self.tell(Event::CompleteElicitation(notification));
    }

    async fn connect_mcp(
        &self,
        request: proto::ConnectMcpRequest,
    ) -> proto::Result<proto::ConnectMcpResponse> {
        self.ask(|reply| Event::ConnectMcp(request, reply)).await
    }

    async fn message_mcp(
        &self,
        request: proto::MessageMcpRequest,
    ) -> proto::Result<proto::MessageMcpResponse> {
        self.ask(|reply| Event::MessageMcp(request, reply)).await
    }

    async fn notify_mcp(&self, notification: proto::MessageMcpNotification) {
        self.tell(Event::NotifyMcp(notification));
    }

    async fn disconnect_mcp(
        &self,
        request: proto::DisconnectMcpRequest,
    ) -> proto::Result<proto::DisconnectMcpResponse> {
        self.ask(|reply| Event::DisconnectMcp(request, reply)).await
    }

    async fn ext_request(
        &self,
        method: String,
        params: serde_json::Value,
    ) -> proto::Result<serde_json::Value> {
        // The one call whose refusal names the method, matching the default.
        let declined = proto::Error::method_not_found().data(method.clone());
        let (tx, rx) = oneshot::channel();
        let event = Event::ExtRequest {
            method,
            params,
            reply: Reply(tx),
        };
        if self.0.send(event).is_err() {
            return Err(declined);
        }
        rx.await.unwrap_or(Err(declined))
    }

    async fn ext_notification(&self, method: String, params: serde_json::Value) {
        self.tell(Event::ExtNotification { method, params });
    }
}
