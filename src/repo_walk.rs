//! Directory-walking module for `list`: recursively finds every
//! `.git`-containing directory under `root-dir` and renders each as a path
//! relative to `root-dir`, sorted. The filesystem is the only I/O boundary
//! here - unlike every other module in this crate, `list` never shells out to
//! `git`.
use crate::debug_log;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Renders a repo-relative path (one of `find_repos`'s results) as a
/// `host/owner/repo` identifier - forward-slash-joined on every platform,
/// matching the convention `gig config category` patterns are written
/// against (see `CategoryArgs`'s docs in `cli.rs`) and the README's
/// `host/owner/repo` framing. Deliberately not `Path::display`, which on
/// Windows would render native `\` separators instead - see
/// `docs/adr/0008-forward-slash-repo-identifiers-even-on-windows.md`.
pub fn display(relative: &Path) -> String {
    relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Every already-cloned repo under `root_dir`, as paths relative to it,
/// sorted. A `root_dir` that doesn't exist on disk yet (nothing cloned)
/// yields an empty list rather than an error. Once a directory is found to
/// contain a `.git` subdirectory, its contents aren't recursed into further -
/// a repo's working tree is never searched for clones nested inside it.
pub fn find_repos(root_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut repos = Vec::new();
    if root_dir.is_dir() {
        walk(root_dir, root_dir, &mut repos)?;
    }
    repos.sort();
    Ok(repos)
}

fn walk(root_dir: &Path, dir: &Path, repos: &mut Vec<PathBuf>) -> Result<()> {
    if dir.join(".git").is_dir() {
        let relative = dir
            .strip_prefix(root_dir)
            .with_context(|| format!("{} is not under {}", dir.display(), root_dir.display()))?;
        debug_log::log(format!("found repo: {}", display(relative)));
        repos.push(relative.to_path_buf());
        return Ok(());
    }

    let entries =
        fs::read_dir(dir).with_context(|| format!("failed to read directory {}", dir.display()))?;
    for entry in entries {
        let entry =
            entry.with_context(|| format!("failed to read an entry in {}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            walk(root_dir, &path, repos)?;
        }
    }
    Ok(())
}
