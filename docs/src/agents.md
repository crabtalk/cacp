# Finding an agent to drive

The protocol publishes a catalog of ACP agents pinned to exact versions.
`cacp-agents` reads it and installs from it, so an agent's build never changes
underfoot the way `npx <pkg>@latest` does, and no package manager sits in the
chat path.

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

It carries no runtime and does not depend on `cacp`. The working directory, the
environment and stderr are yours to set, and `cacp::spawn` takes it from there.

`registry::catalog` serves a cache under a day old as-is, tries the network
otherwise, and falls back to a stale cache when that fails — so a catalog opened
offline is out of date rather than empty.

Everything here blocks: it reaches the network and runs `npm`. Call it off a
worker rather than inside a turn.

## MCP servers

The same module shape covers the MCP registry, for handing an agent servers it
can reach: `mcp::search` queries it live, and `Server::install` places an npm
package or hands back `None` for a remote server that needs no install.

Remote servers only work against an agent that advertises
`mcp_capabilities.http`. Check before offering one, rather than sending an entry
it will fail to dial.
