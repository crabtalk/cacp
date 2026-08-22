# Introduction

ACP is how a code editor and a coding agent talk to each other — JSON-RPC over a
pipe. Two roles share one connection, and each can call the other:

- the **agent** answers `initialize`, opens sessions, and runs prompt turns
- the **client** — the editor — receives progress, answers permission prompts,
  and serves files and terminals on the agent's behalf

`cacp` implements both ends. Which one you want decides where to start:

| you are writing | you implement | start at |
| --- | --- | --- |
| a coding agent an editor drives | [`Agent`] | [Serving an agent](./serving.md) |
| an editor, TUI or app driving an agent | [`Client`] | [Driving an agent](./driving.md) |
| a frontend with an event loop already | nothing | [An event loop](./events.md) |

Whichever role you serve, only a handful of methods are required. Everything
else answers `method not found` until you implement it, so a half-built peer is
a working peer — it just advertises less.

[`Agent`]: https://docs.rs/cacp/latest/cacp/trait.Agent.html
[`Client`]: https://docs.rs/cacp/latest/cacp/trait.Client.html
