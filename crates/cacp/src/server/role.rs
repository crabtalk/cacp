//! What an ACP agent implements.

use crate::handler::{decode, encode};
use proto::method::agent as method;
use std::future::Future;

/// What an ACP agent serves.
///
/// Only [`initialize`](Agent::initialize), [`new_session`](Agent::new_session)
/// and [`prompt`](Agent::prompt) are required. Every other method defaults to
/// replying "method not found", so an agent that does not implement a capability
/// declines it instead of leaving the client waiting — advertise what you do
/// implement in [`proto::InitializeResponse::agent_capabilities`].
pub trait Agent: Send + Sync + 'static {
    fn initialize(
        &self,
        request: proto::InitializeRequest,
    ) -> impl Future<Output = proto::Result<proto::InitializeResponse>> + Send;

    fn new_session(
        &self,
        request: proto::NewSessionRequest,
    ) -> impl Future<Output = proto::Result<proto::NewSessionResponse>> + Send;

    /// Run a turn. Report progress with
    /// [`ClientConn::session_update`](crate::ClientConn::session_update) as it goes.
    fn prompt(
        &self,
        request: proto::PromptRequest,
    ) -> impl Future<Output = proto::Result<proto::PromptResponse>> + Send;

    /// Stop the session's current turn. The in-flight `prompt` should still
    /// return, with [`StopReason::Cancelled`](proto::StopReason::Cancelled).
    fn cancel(&self, notification: proto::CancelNotification) -> impl Future<Output = ()> + Send {
        let _ = notification;
        async {}
    }

    fn authenticate(
        &self,
        request: proto::AuthenticateRequest,
    ) -> impl Future<Output = proto::Result<proto::AuthenticateResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    fn logout(
        &self,
        request: proto::LogoutRequest,
    ) -> impl Future<Output = proto::Result<proto::LogoutResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    fn load_session(
        &self,
        request: proto::LoadSessionRequest,
    ) -> impl Future<Output = proto::Result<proto::LoadSessionResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    fn resume_session(
        &self,
        request: proto::ResumeSessionRequest,
    ) -> impl Future<Output = proto::Result<proto::ResumeSessionResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    /// Branch a session. The response carries the *new* session's id; the
    /// one in the request keeps running untouched.
    fn fork_session(
        &self,
        request: proto::ForkSessionRequest,
    ) -> impl Future<Output = proto::Result<proto::ForkSessionResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    fn close_session(
        &self,
        request: proto::CloseSessionRequest,
    ) -> impl Future<Output = proto::Result<proto::CloseSessionResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    fn list_sessions(
        &self,
        request: proto::ListSessionsRequest,
    ) -> impl Future<Output = proto::Result<proto::ListSessionsResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    fn delete_session(
        &self,
        request: proto::DeleteSessionRequest,
    ) -> impl Future<Output = proto::Result<proto::DeleteSessionResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    fn set_session_mode(
        &self,
        request: proto::SetSessionModeRequest,
    ) -> impl Future<Output = proto::Result<proto::SetSessionModeResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    fn set_session_config_option(
        &self,
        request: proto::SetSessionConfigOptionRequest,
    ) -> impl Future<Output = proto::Result<proto::SetSessionConfigOptionResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    /// The LLM endpoints this agent can be pointed at, and where each one
    /// points right now.
    fn list_providers(
        &self,
        request: proto::ListProvidersRequest,
    ) -> impl Future<Output = proto::Result<proto::ListProvidersResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    fn set_provider(
        &self,
        request: proto::SetProviderRequest,
    ) -> impl Future<Output = proto::Result<proto::SetProviderResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    fn disable_provider(
        &self,
        request: proto::DisableProviderRequest,
    ) -> impl Future<Output = proto::Result<proto::DisableProviderResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    /// Open a next-edit session. Its id is its own — it does not name a chat
    /// session, and the two lifecycles are unrelated.
    fn start_nes(
        &self,
        request: proto::StartNesRequest,
    ) -> impl Future<Output = proto::Result<proto::StartNesResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    /// Predict the user's next edit. Called on every keystroke a client
    /// decides is worth asking about, so it should be cheap to decline.
    fn suggest_nes(
        &self,
        request: proto::SuggestNesRequest,
    ) -> impl Future<Output = proto::Result<proto::SuggestNesResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    fn close_nes(
        &self,
        request: proto::CloseNesRequest,
    ) -> impl Future<Output = proto::Result<proto::CloseNesResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    /// The user took a suggestion.
    fn accept_nes(
        &self,
        notification: proto::AcceptNesNotification,
    ) -> impl Future<Output = ()> + Send {
        let _ = notification;
        async {}
    }

    /// The user did not take a suggestion — see
    /// [`NesRejectReason`](proto::NesRejectReason) for why.
    fn reject_nes(
        &self,
        notification: proto::RejectNesNotification,
    ) -> impl Future<Output = ()> + Send {
        let _ = notification;
        async {}
    }

    fn did_open_document(
        &self,
        notification: proto::DidOpenDocumentNotification,
    ) -> impl Future<Output = ()> + Send {
        let _ = notification;
        async {}
    }

    fn did_change_document(
        &self,
        notification: proto::DidChangeDocumentNotification,
    ) -> impl Future<Output = ()> + Send {
        let _ = notification;
        async {}
    }

    fn did_close_document(
        &self,
        notification: proto::DidCloseDocumentNotification,
    ) -> impl Future<Output = ()> + Send {
        let _ = notification;
        async {}
    }

    fn did_save_document(
        &self,
        notification: proto::DidSaveDocumentNotification,
    ) -> impl Future<Output = ()> + Send {
        let _ = notification;
        async {}
    }

    fn did_focus_document(
        &self,
        notification: proto::DidFocusDocumentNotification,
    ) -> impl Future<Output = ()> + Send {
        let _ = notification;
        async {}
    }

    /// An MCP message coming *back* up the tunnel — a server the client holds
    /// on this agent's behalf is asking the agent something.
    fn message_mcp(
        &self,
        request: proto::MessageMcpRequest,
    ) -> impl Future<Output = proto::Result<proto::MessageMcpResponse>> + Send {
        let _ = request;
        async { Err(proto::Error::method_not_found()) }
    }

    /// The same tunnel as [`message_mcp`](Agent::message_mcp), for MCP
    /// messages that want no reply.
    fn notify_mcp(
        &self,
        notification: proto::MessageMcpNotification,
    ) -> impl Future<Output = ()> + Send {
        let _ = notification;
        async {}
    }

    /// Answer a method the spec does not define — anything the client sends
    /// with a `_`-prefixed name. Declines by default.
    fn ext_request(
        &self,
        method: String,
        params: serde_json::Value,
    ) -> impl Future<Output = proto::Result<serde_json::Value>> + Send {
        let _ = params;
        async move { Err(proto::Error::method_not_found().data(method)) }
    }

    /// The same, for a notification. Ignored by default.
    fn ext_notification(
        &self,
        method: String,
        params: serde_json::Value,
    ) -> impl Future<Output = ()> + Send {
        let _ = (method, params);
        async {}
    }

    /// Route one incoming call. Provided — do not override.
    fn dispatch(
        &self,
        method: String,
        params: serde_json::Value,
    ) -> impl Future<Output = proto::Result<serde_json::Value>> + Send {
        async move {
            match method.as_str() {
                method::INITIALIZE => encode(self.initialize(decode(params)?).await?),
                method::AUTHENTICATE => encode(self.authenticate(decode(params)?).await?),
                method::LOGOUT => encode(self.logout(decode(params)?).await?),
                method::SESSION_NEW => encode(self.new_session(decode(params)?).await?),
                method::SESSION_LOAD => encode(self.load_session(decode(params)?).await?),
                method::SESSION_RESUME => encode(self.resume_session(decode(params)?).await?),
                method::SESSION_FORK => encode(self.fork_session(decode(params)?).await?),
                method::SESSION_CLOSE => encode(self.close_session(decode(params)?).await?),
                method::SESSION_LIST => encode(self.list_sessions(decode(params)?).await?),
                method::SESSION_DELETE => encode(self.delete_session(decode(params)?).await?),
                method::SESSION_SET_MODE => encode(self.set_session_mode(decode(params)?).await?),
                method::SESSION_SET_CONFIG_OPTION => {
                    encode(self.set_session_config_option(decode(params)?).await?)
                }
                method::SESSION_PROMPT => encode(self.prompt(decode(params)?).await?),
                method::PROVIDERS_LIST => encode(self.list_providers(decode(params)?).await?),
                method::PROVIDERS_SET => encode(self.set_provider(decode(params)?).await?),
                method::PROVIDERS_DISABLE => encode(self.disable_provider(decode(params)?).await?),
                method::NES_START => encode(self.start_nes(decode(params)?).await?),
                method::NES_SUGGEST => encode(self.suggest_nes(decode(params)?).await?),
                method::NES_CLOSE => encode(self.close_nes(decode(params)?).await?),
                method::MCP_MESSAGE => encode(self.message_mcp(decode(params)?).await?),
                _ => self.ext_request(method, params).await,
            }
        }
    }

    /// Route one incoming notification. Provided — do not override.
    fn dispatch_notification(
        &self,
        method: String,
        params: serde_json::Value,
    ) -> impl Future<Output = ()> + Send {
        async move {
            match method.as_str() {
                method::SESSION_CANCEL => {
                    if let Ok(notification) = decode(params) {
                        self.cancel(notification).await;
                    }
                }
                method::NES_ACCEPT => {
                    if let Ok(notification) = decode(params) {
                        self.accept_nes(notification).await;
                    }
                }
                method::NES_REJECT => {
                    if let Ok(notification) = decode(params) {
                        self.reject_nes(notification).await;
                    }
                }
                method::DOCUMENT_DID_OPEN => {
                    if let Ok(notification) = decode(params) {
                        self.did_open_document(notification).await;
                    }
                }
                method::DOCUMENT_DID_CHANGE => {
                    if let Ok(notification) = decode(params) {
                        self.did_change_document(notification).await;
                    }
                }
                method::DOCUMENT_DID_CLOSE => {
                    if let Ok(notification) = decode(params) {
                        self.did_close_document(notification).await;
                    }
                }
                method::DOCUMENT_DID_SAVE => {
                    if let Ok(notification) = decode(params) {
                        self.did_save_document(notification).await;
                    }
                }
                method::DOCUMENT_DID_FOCUS => {
                    if let Ok(notification) = decode(params) {
                        self.did_focus_document(notification).await;
                    }
                }
                method::MCP_MESSAGE => {
                    if let Ok(notification) = decode(params) {
                        self.notify_mcp(notification).await;
                    }
                }
                _ => self.ext_notification(method, params).await,
            }
        }
    }
}
