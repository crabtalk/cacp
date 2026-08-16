//! Next edit suggestions: the agent predicts the user's next edit as they
//! type. NES sessions are their own thing — they do not share ids or history
//! with the chat sessions in [`session`](crate::session).

use crate::{Capability, NesSuggestionId, SessionId};
use serde::{Deserialize, Serialize};

/// How the client counts columns in a [`Position`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionEncodingKind {
    #[serde(rename = "utf-16")]
    Utf16,
    #[serde(rename = "utf-32")]
    Utf32,
    #[serde(rename = "utf-8")]
    Utf8,
}

/// Zero-based, with `character` counted in the negotiated
/// [`PositionEncodingKind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// What the agent wants: which document events to receive, and which kinds of
/// context to be given with each request.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<NesEventCapabilities>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<NesContextCapabilities>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesEventCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub document: Option<NesDocumentEventCapabilities>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesDocumentEventCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub did_open: Option<Capability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub did_change: Option<NesDidChangeCapabilities>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub did_close: Option<Capability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub did_save: Option<Capability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub did_focus: Option<Capability>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesDidChangeCapabilities {
    pub sync_kind: TextDocumentSyncKind,
}

/// Whether [`DidChangeDocumentNotification`] carries the whole document or
/// just the ranges that changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextDocumentSyncKind {
    Full,
    Incremental,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesContextCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recent_files: Option<NesCountCapability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_snippets: Option<Capability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edit_history: Option<NesCountCapability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_actions: Option<NesCountCapability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_files: Option<Capability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Capability>,
}

/// A context kind the agent wants, capped at however many entries it can use.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesCountCapability {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_count: Option<u32>,
}

/// Which suggestion kinds the client can actually apply. An agent must not
/// send a kind the client did not advertise.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientNesCapabilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jump: Option<Capability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rename: Option<Capability>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_and_replace: Option<Capability>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartNesRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace_folders: Option<Vec<WorkspaceFolder>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<NesRepository>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFolder {
    pub uri: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesRepository {
    pub name: String,
    pub owner: String,
    pub remote_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartNesResponse {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseNesRequest {
    pub session_id: SessionId,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloseNesResponse {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidOpenDocumentNotification {
    pub session_id: SessionId,
    pub uri: String,
    pub language_id: String,
    pub version: i64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeDocumentNotification {
    pub session_id: SessionId,
    pub uri: String,
    pub version: i64,
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

/// A `range` of `None` means `text` replaces the whole document — that is the
/// [`TextDocumentSyncKind::Full`] shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentContentChangeEvent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidCloseDocumentNotification {
    pub session_id: SessionId,
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidSaveDocumentNotification {
    pub session_id: SessionId,
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidFocusDocumentNotification {
    pub session_id: SessionId,
    pub uri: String,
    pub version: i64,
    pub position: Position,
    pub visible_range: Range,
}

/// Why the client is asking now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NesTriggerKind {
    Automatic,
    Diagnostic,
    Manual,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestNesRequest {
    pub session_id: SessionId,
    pub uri: String,
    pub version: i64,
    pub position: Position,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection: Option<Range>,
    pub trigger_kind: NesTriggerKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<NesSuggestContext>,
}

/// Only the kinds the agent asked for in [`NesContextCapabilities`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesSuggestContext {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recent_files: Option<Vec<NesRecentFile>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_snippets: Option<Vec<NesRelatedSnippet>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edit_history: Option<Vec<NesEditHistoryEntry>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_actions: Option<Vec<NesUserAction>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_files: Option<Vec<NesOpenFile>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<NesDiagnostic>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesRecentFile {
    pub uri: String,
    pub language_id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesRelatedSnippet {
    pub uri: String,
    pub excerpts: Vec<NesExcerpt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesExcerpt {
    pub start_line: u32,
    pub end_line: u32,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesEditHistoryEntry {
    pub uri: String,
    pub diff: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesUserAction {
    pub action: String,
    pub uri: String,
    pub position: Position,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesOpenFile {
    pub uri: String,
    pub language_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visible_range: Option<Range>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_focused_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesDiagnostic {
    pub uri: String,
    pub range: Range,
    pub severity: NesDiagnosticSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NesDiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestNesResponse {
    pub suggestions: Vec<NesSuggestion>,
}

/// What the agent thinks should happen next. The client applies at most one,
/// then says which with [`AcceptNesNotification`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NesSuggestion {
    Edit(NesEditSuggestion),
    Jump(NesJumpSuggestion),
    Rename(NesRenameSuggestion),
    SearchAndReplace(NesSearchAndReplaceSuggestion),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesEditSuggestion {
    pub id: NesSuggestionId,
    pub uri: String,
    pub edits: Vec<NesTextEdit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor_position: Option<Position>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesTextEdit {
    pub range: Range,
    pub new_text: String,
}

/// Move the cursor somewhere else — the next edit belongs in another place,
/// not in different text here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesJumpSuggestion {
    pub id: NesSuggestionId,
    pub uri: String,
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesRenameSuggestion {
    pub id: NesSuggestionId,
    pub uri: String,
    pub position: Position,
    pub new_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NesSearchAndReplaceSuggestion {
    pub id: NesSuggestionId,
    pub uri: String,
    pub search: String,
    pub replace: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_regex: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptNesNotification {
    pub session_id: SessionId,
    pub id: NesSuggestionId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RejectNesNotification {
    pub session_id: SessionId,
    pub id: NesSuggestionId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<NesRejectReason>,
}

/// `Ignored` means the user simply kept typing; `Replaced` means a newer
/// suggestion superseded this one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NesRejectReason {
    Rejected,
    Ignored,
    Replaced,
    Cancelled,
}
