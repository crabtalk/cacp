//! Wire shapes that are not obvious from the type definitions: tagged enums
//! with an untagged fallback, nested tags, and the renames that do not follow
//! the container's rule.

use cacp_proto as proto;
use serde_json::json;

fn round<T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug>(
    value: T,
    expected: serde_json::Value,
) {
    assert_eq!(serde_json::to_value(&value).unwrap(), expected);
    assert_eq!(serde_json::from_value::<T>(expected).unwrap(), value);
}

#[test]
fn auth_method_agent_carries_no_tag() {
    round(
        proto::AuthMethod::Agent(proto::AuthMethodAgent {
            id: "oauth".into(),
            name: "Sign in".into(),
            description: None,
        }),
        json!({"id": "oauth", "name": "Sign in"}),
    );
}

#[test]
fn auth_method_tags_the_rest() {
    round(
        proto::AuthMethod::Terminal(proto::AuthMethodTerminal {
            id: "tui".into(),
            name: "Terminal".into(),
            description: None,
            args: vec!["login".into()],
            env: Default::default(),
        }),
        json!({"type": "terminal", "id": "tui", "name": "Terminal", "args": ["login"]}),
    );
}

#[test]
fn auth_env_var_is_secret_unless_told_otherwise() {
    let var: proto::AuthEnvVar = serde_json::from_value(json!({"name": "OPENAI_API_KEY"})).unwrap();
    assert!(var.secret);
    assert!(!var.optional);
    // Secret is the default, so it stays off the wire.
    assert_eq!(
        serde_json::to_value(&var).unwrap(),
        json!({"name": "OPENAI_API_KEY"})
    );
}

#[test]
fn mcp_server_acp_is_tagged_and_stdio_still_is_not() {
    round(
        proto::McpServer::Acp(proto::McpServerAcp {
            name: "editor".into(),
            server_id: "s1".into(),
        }),
        json!({"type": "acp", "name": "editor", "serverId": "s1"}),
    );

    let stdio: proto::McpServer =
        serde_json::from_value(json!({"name": "local", "command": "/bin/srv"})).unwrap();
    assert!(matches!(stdio, proto::McpServer::Stdio(_)));
}

#[test]
fn plan_update_nests_a_tag_inside_the_session_update_tag() {
    round(
        proto::SessionUpdate::PlanUpdate(proto::PlanUpdate {
            plan: proto::PlanUpdateContent::Items(proto::PlanItems {
                plan_id: "p1".into(),
                entries: vec![],
            }),
        }),
        json!({
            "sessionUpdate": "plan_update",
            "plan": {"type": "items", "planId": "p1", "entries": []},
        }),
    );

    round(
        proto::SessionUpdate::PlanRemoved(proto::PlanRemoved {
            plan_id: "p1".into(),
        }),
        json!({"sessionUpdate": "plan_removed", "planId": "p1"}),
    );
}

#[test]
fn nes_suggestions_tag_on_kind_in_camel_case() {
    round(
        proto::NesSuggestion::SearchAndReplace(proto::NesSearchAndReplaceSuggestion {
            id: "s1".into(),
            uri: "file:///a.rs".into(),
            search: "foo".into(),
            replace: "bar".into(),
            is_regex: None,
        }),
        json!({
            "kind": "searchAndReplace",
            "id": "s1",
            "uri": "file:///a.rs",
            "search": "foo",
            "replace": "bar",
        }),
    );
}

#[test]
fn position_encodings_keep_their_hyphens() {
    round(proto::PositionEncodingKind::Utf16, json!("utf-16"));
    round(proto::PositionEncodingKind::Utf8, json!("utf-8"));
}

#[test]
fn unknown_llm_protocols_survive_as_themselves() {
    round(proto::LlmProtocol::OpenAi, json!("openai"));
    round(proto::LlmProtocol::Other("groq".into()), json!("groq"));
}

#[test]
fn fork_is_advertised_as_a_bare_presence_flag() {
    let capabilities = proto::SessionCapabilities {
        fork: Some(proto::Capability {}),
        ..Default::default()
    };
    assert_eq!(
        serde_json::to_value(capabilities).unwrap(),
        json!({"fork": {}})
    );
}

#[test]
fn end_of_turn_usage_is_optional() {
    round(
        proto::PromptResponse::new(proto::StopReason::EndTurn),
        json!({"stopReason": "end_turn"}),
    );
    round(
        proto::PromptResponse {
            stop_reason: proto::StopReason::EndTurn,
            usage: Some(proto::Usage {
                total_tokens: 30,
                input_tokens: 10,
                output_tokens: 20,
                ..Default::default()
            }),
        },
        json!({
            "stopReason": "end_turn",
            "usage": {"totalTokens": 30, "inputTokens": 10, "outputTokens": 20},
        }),
    );
}
