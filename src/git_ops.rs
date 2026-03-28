//! Thin wrapper around the system `git` binary's clone operation, invoked by
//! `get`. No libgit2, no other git implementation - shells out via
//! [`crate::git_cmd`], same as [`crate::git_config`].
use crate::git_cmd::run;
use anyhow::{Result, bail};
use std::path::Path;

/// `git clone <url> <destination>`.
pub fn clone(url: &str, destination: &Path) -> Result<()> {
    let destination = destination.to_string_lossy();
    let output = run(&["clone", url, &destination])?;

    if output.status.success() {
        Ok(())
    } else {
        bail!(
            "git clone {url} {destination} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
}
