# cacp

A compact implementation of the [Agent Client Protocol][acp] v1.

ACP is how a code editor and a coding agent talk to each other — JSON-RPC over a
pipe. `cacp` implements both ends: the **agent** an editor drives, and the
**client** that drives it.

## Why

The reference implementation is large. `cacp` covers the same v1 surface — every
method, both roles, every unstable feature area — in two crates and two traits,
with no builder layer and no macros:

- **You write handlers, never a dispatch table.** Routing is a provided trait
  method; you implement the calls you serve and nothing else.
- **Every optional method declines by default.** A peer that asks for something
  you do not serve gets a clean `method not found` instead of waiting forever.
- **Notifications arrive in wire order.** The read loop awaits each one before
  it reads the next frame, so a turn's final update reaches you before its
  result does — you do not have to reassemble the ordering yourself.
- **A slow handler does not stall the stream.** Requests are spawned, so
  blocking on a permission prompt while the user makes up their mind does not
  hold up the updates queued behind it.

## Crates

| crate | what it is | dependencies |
| --- | --- | --- |
| `cacp-proto` | ACP v1 wire types, method names, the protocol error | `serde`, `serde_json` |
| `cacp` | the JSON-RPC connection and both roles | the above, plus `tokio` |

Depend on `cacp-proto` alone if all you do is read and write ACP JSON — it is
data only, and pulls in no async runtime.

```toml
cacp = "0.1"
```

Each role is a feature and both are on by default. Turn off the one you do not
implement:

```toml
cacp = { version = "0.1", default-features = false, features = ["client"] }
```

| feature | gives you | adds |
| --- | --- | --- |
| `client` | `Client`, `AgentConn`, `spawn`, `connect`, `connect_on` | `tokio/process` |
| `server` | `Agent`, `ClientConn`, `serve`, `serve_on`, `serve_on_stdio` | `tokio/io-std` |

## Serving an agent

Implement [`Agent`] and hand it to one of the `serve_*` functions. Only
`initialize`, `new_session` and `prompt` are required — advertise whatever else
you implement in `InitializeResponse::agent_capabilities`.

```rust
use cacp::{Agent, Result, schema};
use std::sync::Arc;

struct Echo;

impl Agent for Echo {
    async fn initialize(&self, _: schema::InitializeRequest) -> Result<schema::InitializeResponse> {
        Ok(schema::InitializeResponse {
            protocol_version: schema::ProtocolVersion::LATEST,
            agent_capabilities: Default::default(),
            auth_methods: Vec::new(),
            agent_info: None,
        })
    }

    async fn new_session(&self, _: schema::NewSessionRequest) -> Result<schema::NewSessionResponse> {
        Ok(schema::NewSessionResponse {
            session_id: "session-1".into(),
            modes: None,
            config_options: None,
        })
    }

    async fn prompt(&self, _: schema::PromptRequest) -> Result<schema::PromptResponse> {
        Ok(schema::PromptResponse::new(schema::StopReason::EndTurn))
    }
}

#[tokio::main]
async fn main() {
    let _client = cacp::serve_on_stdio(Arc::new(Echo));
    // The read and write loops run on the runtime; keep the process alive.
    std::future::pending::<()>().await
}
```

`serve_on_stdio` hands back a `ClientConn` for calling the editor — reporting
progress with `session_update`, asking permission, reading files through it.

## Driving an agent

Implement [`Client`] and hand it to `spawn`. Only `session_update` and
`request_permission` are required; `fs/*`, `terminal/*` and `elicitation/*`
decline until you implement them.

```rust
use cacp::{Client, Result, schema};
use std::sync::Arc;
use tokio::process::Command;

struct Ui;

impl Client for Ui {
    async fn session_update(&self, notification: schema::SessionNotification) {
        if let schema::SessionUpdate::AgentMessageChunk(chunk) = notification.update {
            print!("{:?}", chunk.content);
        }
    }

    async fn request_permission(
        &self,
        request: schema::RequestPermissionRequest,
    ) -> Result<schema::RequestPermissionResponse> {
        Ok(schema::RequestPermissionResponse::selected(
            request.options[0].option_id.clone(),
        ))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let (agent, _child) = cacp::spawn(Command::new("my-agent").arg("--acp"), Arc::new(Ui))?;

    agent
        .initialize(schema::InitializeRequest::new(Default::default()))
        .await?;
    let session = agent
        .new_session(schema::NewSessionRequest::new("/path/to/repo"))
        .await?;
    let done = agent
        .prompt(schema::PromptRequest::new(
            session.session_id,
            vec!["explain this repo".into()],
        ))
        .await?;

    println!("{:?}", done.stop_reason);
    Ok(())
}
```

Both of these are in [`crates/cacp/examples`](crates/cacp/examples), so they are
compiled on every build rather than left to rot.

## Coverage

Every v1 method name, on both sides. Beyond the stable core, all nine areas the
spec still marks unstable are implemented: session fork, LLM providers, plan
operations, next edit suggestions, end-of-turn token usage, tool call names,
auth methods, elicitation, and MCP over ACP.

Not there yet: `_meta` and the `_`-prefixed extension methods. cacp round-trips
drop `_meta`, and there is no hook for serving an extension method.

## License

[Apache-2.0](LICENSE).

[acp]: https://agentclientprotocol.com
[`Agent`]: crates/cacp/src/server/role.rs
[`Client`]: crates/cacp/src/client/role.rs
