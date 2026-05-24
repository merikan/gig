//! Thin wrapper around the system `git` binary's own config store, so gig
//! never parses gitconfig files itself and git's own precedence/include rules
//! keep applying untouched. Shared by every gig setting (`root-dir` today,
//! category routing later) rather than being specific to any one key.
use crate::git_cmd::run;
use anyhow::{Result, bail};

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

/// `git config --get-all <key>` - every value of a (possibly multi-valued)
/// key, in declaration/add order. An empty vec means the key is unset -
/// that's how `--get-all` itself reports "no such key", via exit code 1.
pub fn get_all(key: &str) -> Result<Vec<String>> {
    let output = run(&["config", "--get-all", key])?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_string)
            .collect())
    } else if output.status.code() == Some(1) {
        Ok(Vec::new())
    } else {
        bail!(
            "git config --get-all {key} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
}

/// `git config --add --global <key> <value>` - appends a new value to a
/// (possibly already multi-valued) key without touching its existing values.
pub fn add_global(key: &str, value: &str) -> Result<()> {
    let output = run(&["config", "--add", "--global", key, value])?;

    if output.status.success() {
        Ok(())
    } else {
        bail!(
            "git config --add --global {key} {value} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
}

/// `git config --unset --global <key>` - removes `key` if present. A
/// missing key isn't an error: real git reports "nothing to unset" via a
/// non-zero exit (1 or 5, depending on version) rather than success, but the
/// end state the caller wants (`key` absent) already holds either way, so
/// this treats both the same as [`get`] treats a missing key.
pub fn unset_global(key: &str) -> Result<()> {
    let output = run(&["config", "--unset", "--global", key])?;

    if output.status.success() || matches!(output.status.code(), Some(1 | 5)) {
        Ok(())
    } else {
        bail!(
            "git config --unset --global {key} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
}

/// `git config --replace-all --global <key> <value>` - clears every existing
/// value of `key` and writes `value` as its sole remaining one. Used instead
/// of plain [`set_global`] whenever a key might already hold (or is about to
/// be given) more than one value - `git config --global` itself refuses to
/// overwrite a multi-valued key with a single value.
pub fn replace_all_global(key: &str, value: &str) -> Result<()> {
    let output = run(&["config", "--replace-all", "--global", key, value])?;

    if output.status.success() {
        Ok(())
    } else {
        bail!(
            "git config --replace-all --global {key} {value} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
}
