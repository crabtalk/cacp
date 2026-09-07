//! Small helpers with no home of their own.

use anyhow::{Result, bail};
use std::path::{Component, Path, PathBuf};

/// The first `program` on `PATH`.
pub(crate) fn which(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| dir.join(program))
        .find(|candidate| candidate.is_file())
}

/// `dir` joined with a relative path the registry supplied — at least one
/// component, every one a plain name. An id or a `cmd` that climbs out, or an
/// empty one naming `dir` itself, would aim an install — or the
/// `remove_dir_all` that undoes one — somewhere it was never given.
pub fn contained(dir: &Path, rel: &str) -> Result<PathBuf> {
    let rel = Path::new(rel.trim_start_matches("./"));
    let mut parts = rel.components().peekable();
    if parts.peek().is_none() || !parts.all(|c| matches!(c, Component::Normal(_))) {
        bail!("{} is not a path inside {}", rel.display(), dir.display());
    }
    Ok(dir.join(rel))
}
