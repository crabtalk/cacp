# Crates

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

`cacp-events` and `cacp-agents` are layers, not dependencies of the core: each
reaches only what `cacp` makes public, which is why each is its own crate rather
than a feature.
