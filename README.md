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
- **A newer peer does not break you.** Update kinds, content blocks, tool call
  content, plan payloads and enum values this revision does not know arrive in
  an `Other` variant and round-trip whole, rather than failing the message they
  came in.
- **Dropping a call cancels it.** Walk away from a request future and the peer
  gets `$/cancel_request` instead of finishing work nobody is waiting for.

## Crates

| crate | what it is | dependencies |
| --- | --- | --- |
| `cacp-proto` | ACP v1 wire types, method names, the protocol error | `serde`, `serde_json` |
| `cacp` | the JSON-RPC connection and both roles | the above, plus `tokio` |
| `cacp-events` | the client as a channel instead of a trait | `cacp` with `client` |
| `cacp-agents` | the registry catalog and an installer for it | `serde`, `serde_json`, `anyhow`, `ureq` |

Depend on `cacp-proto` alone if all you do is read and write ACP JSON — it is
data only, and pulls in no async runtime.

```toml
cacp = "0.0.1"
```

Each role is a feature and both are on by default. Turn off the one you do not
implement:

```toml
cacp = { version = "0.0.1", default-features = false, features = ["client"] }
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
            meta: None,
        })
    }

    async fn new_session(&self, _: schema::NewSessionRequest) -> Result<schema::NewSessionResponse> {
        Ok(schema::NewSessionResponse {
            session_id: "session-1".into(),
            modes: None,
            config_options: None,
            meta: None,
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

## Driving an agent from an event loop

A frontend has a loop already, so [`cacp-events`](crates/events) hands it one
rather than a trait to implement — every call from the agent arrives as an
`Event` on a channel.

```rust,ignore
let (client, mut events) = cacp_events::channel();
let (agent, _child) = cacp::spawn(&mut Command::new("my-agent"), client)?;

while let Some(event) = events.recv().await {
    match event {
        Event::Update(notification) => draw(notification.update),
        Event::Permission(request, reply) => reply.send(ask_the_user(request).await),
        _ => {}
    }
}
```

A request you do not match on is declined with `method not found`, the same as
one you never implemented — so `_ => {}` above serves nothing but updates and
permission, exactly as its capabilities should say. It is a layer over the
`Client` trait with no privileged access, which is why it is a separate crate.

## Finding an agent to drive

The protocol publishes a catalog of ACP agents pinned to exact versions.
[`cacp-agents`](crates/agents) reads it and installs from it, so an agent's
build never changes underfoot the way `npx <pkg>@latest` does, and no package
manager sits in the chat path.

```rust,ignore
let catalog = registry::catalog(&cache_dir).expect("a catalog");
let agent = catalog.agents.iter().find(|a| a.id == "claude-acp").unwrap();

let installed = match Installed::find(&data_dir, &agent.id) {
    Some(installed) => installed,
    None => agent.install(&data_dir, |line| println!("{line}"))?,
};

let mut command = Command::new(&installed.command);
command.args(&installed.args).current_dir(&cwd);
let (conn, _child) = cacp::spawn(&mut command, client)?;
```

It carries no runtime and does not depend on `cacp` — the working directory,
the environment and stderr are the caller's, and `cacp::spawn` takes it from
there. Reading the catalog and running `npm` block, so call them off a worker
rather than inside a turn.

## Coverage

Every v1 method name, on both sides. Beyond the stable core, all eight areas the
spec still marks unstable are implemented: session fork, LLM providers, plan
operations, next edit suggestions, end-of-turn token usage, tool call names,
auth methods, and MCP over ACP.

Both extension mechanisms work: `_meta` is a field on every message that carries
it in the spec, read and written untouched, and `_`-prefixed methods reach
`ext_request` / `ext_notification` on either role — which decline by default,
like every other optional method.

Not there yet: the leniency upstream applies field by field, where a malformed
optional field falls back to its default and a bad array item is skipped rather
than failing the message around it. Unknown *shapes* are handled — that is what
the `Other` variants are for — but malformed ones are still an error.

## License

[Apache-2.0](LICENSE).

[acp]: https://agentclientprotocol.com
[`Agent`]: crates/cacp/src/server/role.rs
[`Client`]: crates/cacp/src/client/role.rs
