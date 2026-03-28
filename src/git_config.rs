//! Thin wrapper around the system `git` binary's own config store, so git-get
//! never parses gitconfig files itself and git's own precedence/include rules
//! keep applying untouched. Shared by every git-get setting (`root-dir` today,
//! category routing later) rather than being specific to any one key.
use anyhow::{Context, Result, bail};
use std::process::{Command, Output};

/// `git config --get <key>`. `None` means the key is unset - that's how `git
/// config --get` itself reports a missing key, via exit code 1. Any other
/// non-zero exit is a genuine failure (bad config file, invalid key, ...) and
/// propagates rather than being silently treated as "unset".
pub fn get(key: &str) -> Result<Option<String>> {
    let output = run(&["config", "--get", key])?;

    if output.status.success() {
        Ok(Some(
            String::from_utf8_lossy(&output.stdout).trim().to_string(),
        ))
    } else if output.status.code() == Some(1) {
        Ok(None)
    } else {
        bail!(
            "git config --get {key} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
}

/// `git config --global <key> <value>`.
pub fn set_global(key: &str, value: &str) -> Result<()> {
    let output = run(&["config", "--global", key, value])?;

    if output.status.success() {
        Ok(())
    } else {
        bail!(
            "git config --global {key} {value} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
}

fn run(args: &[&str]) -> Result<Output> {
    Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("failed to run `git {}`", args.join(" ")))
}
