//! Thin wrapper around the system `git` binary's clone/pull operations,
//! invoked by `get`. No libgit2, no other git implementation - shells out via
//! [`crate::git_cmd`], same as [`crate::git_config`].
//!
//! Both operations inherit stdio rather than capturing it, so git's own
//! output streams directly to the user - see
//! `docs/adr/0001-inherit-stdio-for-clone-and-pull.md`. This means failures
//! here can't quote git's stderr text (it was already shown live); the error
//! just names the operation and git's exit code.
use crate::git_cmd::run_inherited;
use anyhow::{Result, bail};
use std::path::Path;
use std::process::ExitStatus;

/// `git clone <url> <destination>`.
pub fn clone(url: &str, destination: &Path) -> Result<()> {
    let destination = destination.to_string_lossy();
    let status = run_inherited(&["clone", url, &destination])?;

    if status.success() {
        Ok(())
    } else {
        bail!(
            "git clone {url} {destination} failed (exit code {})",
            exit_code_display(status)
        );
    }
}

/// `git -C <destination> pull`, refreshing an already-cloned repo in place.
pub fn pull(destination: &Path) -> Result<()> {
    let destination = destination.to_string_lossy();
    let status = run_inherited(&["-C", &destination, "pull"])?;

    if status.success() {
        Ok(())
    } else {
        bail!(
            "git pull in {destination} failed (exit code {})",
            exit_code_display(status)
        );
    }
}

/// A process's exit code as display text, or "unknown" for the rare case a
/// process has none (e.g. killed by a signal on Unix).
fn exit_code_display(status: ExitStatus) -> String {
    status
        .code()
        .map_or_else(|| "unknown".to_string(), |c| c.to_string())
}
