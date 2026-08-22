//! An ACP agent as a program on this machine: find it, install it, run it.
//!
//! The registry is the protocol's own catalog, pinned to exact versions — so an
//! agent's build never changes underfoot the way `npx <pkg>@latest` does.
//! Agents install once into a data directory and run straight from there,
//! keeping package managers out of the chat path.
//!
//! ```no_run
//! # fn main() -> anyhow::Result<()> {
//! # let (cache_dir, data_dir) = (std::path::Path::new("cache"), std::path::Path::new("data"));
//! let catalog = cacp_agents::registry::catalog(cache_dir).expect("a catalog");
//! let agent = &catalog.agents[0];
//!
//! let installed = match cacp_agents::Installed::find(data_dir, &agent.id) {
//!     Some(installed) => installed,
//!     None => agent.install(data_dir, |line| println!("{line}"))?,
//! };
//!
//! let launch = cacp_agents::Launch::from(&installed);
//! # let _ = launch;
//! # Ok(()) }
//! ```
//!
//! [`registry`] and [`install`](Agent::install) block: they reach the network
//! and run `npm`. Call them off an async runtime's worker — everything here is
//! a one-shot user action, not part of a turn.

pub use install::{Installed, package_name};
pub use launch::Launch;
pub use registry::{Agent, Distribution, Registry};

pub mod mcp;
pub mod registry;

mod install;
mod launch;
mod utils;
