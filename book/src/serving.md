# Serving an agent

Implement [`Agent`] and hand it to one of the `serve_*` functions. Only
`initialize`, `new_session` and `prompt` are required — advertise whatever else
you implement in `InitializeResponse::agent_capabilities`.

```rust,ignore
{{#include ../../crates/cacp/examples/agent.rs}}
```

That is `crates/cacp/examples/agent.rs`, included verbatim — `cargo test`
compiles it, so this page cannot drift from working code.

`serve_on_stdio` hands back a `ClientConn` for calling the editor — reporting
progress with `session_update`, asking permission, reading files through it.

Three ways in, depending on what you already hold:

| function | takes |
| --- | --- |
| `serve_on_stdio` | this process's own stdin and stdout |
| `serve` | one duplex stream — a socket, an in-memory pipe |
| `serve_on` | a reader and a writer owned separately |

[`Agent`]: https://docs.rs/cacp/latest/cacp/trait.Agent.html
