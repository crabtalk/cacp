# Driving an agent

Implement [`Client`] and hand it to `spawn`. Only `session_update` and
`request_permission` are required; `fs/*`, `terminal/*` and `elicitation/*`
decline until you implement them.

```rust,ignore
{{#include ../../crates/cacp/examples/client.rs}}
```

That is `crates/cacp/examples/client.rs`, included verbatim — `cargo test`
compiles it, so this page cannot drift from working code.

`spawn` runs the agent as a subprocess and takes its stdin and stdout. stderr is
left as you configured it, since a TUI usually wants it captured and a CLI
usually does not. The agent is killed when the returned `Child` drops.

`connect` and `connect_on` are the same thing over a stream you already hold.

## Cancelling

Two different cancellations, easy to confuse:

- **Ending a turn** is `session/cancel`. The agent still answers the prompt, with
  `StopReason::Cancelled` — so keep awaiting the prompt future rather than
  dropping it.
- **Abandoning one request** is what dropping its future does: the peer gets
  `$/cancel_request` and stops working on something nobody is waiting for.

[`Client`]: https://docs.rs/cacp/latest/cacp/trait.Client.html
