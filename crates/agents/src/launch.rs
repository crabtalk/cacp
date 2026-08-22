//! An agent as a command to run.

use crate::Installed;
use anyhow::{Context, Result};
use cacp::{AgentConn, Client};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
use tokio::process::{Child, Command};

/// How to start an agent.
///
/// [`From<&Installed>`](Installed) covers an agent this crate installed; build
/// one by hand for an agent the user configured themselves.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Launch {
    pub command: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
}

impl Launch {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            ..Default::default()
        }
    }

    /// Start the agent and connect `client` to it. The agent exits when the
    /// returned [`Child`] is dropped.
    ///
    /// Build the [`Command`] yourself when you want stderr or a working
    /// directory set first, and hand it to [`cacp::spawn`].
    pub fn spawn<C: Client>(&self, client: Arc<C>) -> Result<(AgentConn, Child)> {
        cacp::spawn(&mut Command::from(self), client)
            .with_context(|| format!("spawning {}", self.command))
    }
}

impl From<&Installed> for Launch {
    fn from(installed: &Installed) -> Self {
        Self {
            command: installed.command.clone(),
            args: installed.args.clone(),
            env: BTreeMap::new(),
        }
    }
}

impl From<&Launch> for Command {
    fn from(launch: &Launch) -> Self {
        let mut command = Command::new(&launch.command);
        command.args(&launch.args).envs(&launch.env);
        command
    }
}
