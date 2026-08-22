//! Putting an agent on disk and finding what it left there.

use crate::{
    registry::{Agent, Distribution},
    utils,
};
use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

/// A local installation: the command to spawn and its arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Installed {
    pub id: String,
    pub version: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

impl Agent {
    /// Install under `data_dir`, streaming installer output to `on_line`.
    /// Replaces any previous install of the same agent.
    pub fn install(&self, data_dir: &Path, on_line: impl FnMut(&str)) -> Result<Installed> {
        let Distribution::Npm { package, args } = &self.distribution else {
            bail!("{} cannot be installed yet", self.name);
        };
        let command = install_npm(&agent_dir(data_dir, &self.id), package, on_line)?;

        let record = Installed {
            id: self.id.clone(),
            version: self.version.clone(),
            command,
            args: args.clone(),
        };
        std::fs::write(
            record_file(data_dir, &self.id),
            serde_json::to_string_pretty(&record)?,
        )
        .context("recording the installation")?;
        Ok(record)
    }
}

impl Installed {
    /// The recorded installation for `id`, if the agent is installed and its
    /// command still exists.
    pub fn find(data_dir: &Path, id: &str) -> Option<Self> {
        let body = std::fs::read_to_string(record_file(data_dir, id)).ok()?;
        let record: Self = serde_json::from_str(&body).ok()?;
        Path::new(&record.command).exists().then_some(record)
    }

    /// Delete an installed agent. Succeeds whether or not it was there.
    pub fn remove(data_dir: &Path, id: &str) -> Result<()> {
        let dir = agent_dir(data_dir, id);
        if dir.exists() {
            std::fs::remove_dir_all(&dir).with_context(|| format!("removing {}", dir.display()))?;
        }
        Ok(())
    }
}

fn agent_dir(data_dir: &Path, id: &str) -> PathBuf {
    data_dir.join("agents").join(id)
}

fn record_file(data_dir: &Path, id: &str) -> PathBuf {
    agent_dir(data_dir, id).join("install.json")
}

/// The package name in a spec like `@scope/name@1.2.3` — the version separator
/// is the last `@` that isn't the scope's leading one.
pub fn package_name(spec: &str) -> &str {
    match spec.rfind('@') {
        Some(ix) if ix > 0 => &spec[..ix],
        _ => spec,
    }
}

/// Install one npm package into `dir` (replacing whatever was there), streaming
/// installer output to `on_line`. Returns the executable path.
pub(crate) fn install_npm(
    dir: &Path,
    package: &str,
    mut on_line: impl FnMut(&str),
) -> Result<String> {
    if utils::which("npm").is_none() {
        bail!("npm was not found on PATH — install Node.js to add agents");
    }
    let _ = std::fs::remove_dir_all(dir);
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;

    on_line(&format!("npm install {package}"));
    let mut child = Command::new("npm")
        .arg("install")
        .arg("--prefix")
        .arg(dir)
        .args(["--no-fund", "--no-audit", "--loglevel", "http"])
        .arg(package)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("running npm")?;

    // npm splits progress across both streams; merge them so the caller sees
    // output in the order it arrives.
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    for stream in [
        child
            .stdout
            .take()
            .map(|s| Box::new(s) as Box<dyn std::io::Read + Send>),
        child
            .stderr
            .take()
            .map(|s| Box::new(s) as Box<dyn std::io::Read + Send>),
    ]
    .into_iter()
    .flatten()
    {
        let tx = tx.clone();
        std::thread::spawn(move || {
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
    }
    drop(tx);
    for line in rx {
        let line = line.trim_end();
        if !line.is_empty() {
            on_line(line);
        }
    }

    let status = child.wait().context("waiting for npm")?;
    if !status.success() {
        bail!("npm install failed ({status})");
    }
    let command = binary_path(dir, package)?;
    on_line("installed");
    Ok(command)
}

/// The executable npm linked for `package`, read from the installed package's
/// own `bin` field.
fn binary_path(dir: &Path, package: &str) -> Result<String> {
    let name = package_name(package);
    let manifest = dir.join("node_modules").join(name).join("package.json");
    let body = std::fs::read_to_string(&manifest)
        .with_context(|| format!("reading {}", manifest.display()))?;
    let manifest: serde_json::Value = serde_json::from_str(&body)?;

    let bin_name = match &manifest["bin"] {
        serde_json::Value::String(_) => name.rsplit('/').next().unwrap_or(name).to_owned(),
        serde_json::Value::Object(map) => map
            .keys()
            .next()
            .cloned()
            .ok_or_else(|| anyhow!("{name} declares no executable"))?,
        _ => bail!("{name} declares no executable"),
    };

    let bin = dir.join("node_modules").join(".bin").join(&bin_name);
    if !bin.exists() {
        bail!("{} is missing after install", bin.display());
    }
    Ok(bin.display().to_string())
}
