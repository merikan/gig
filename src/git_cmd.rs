//! Shared plumbing for shelling out to the system `git` binary. Every
//! gig module that wraps a `git` invocation (`git_config`, `git_ops`,
//! `category_config`) runs it through here rather than each calling
//! `Command::new("git")` itself.
use crate::debug_log;
use anyhow::{Context, Result};
use std::process::{Command, ExitStatus, Output};

pub fn run(args: &[&str]) -> Result<Output> {
    debug_log::log(format!("running: git {}", args.join(" ")));
    Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("failed to run `git {}`", args.join(" ")))
}

/// Like [`run`], but inherits stdio instead of capturing it - for git
/// invocations whose own output (clone progress, pull diffstat, ...) should
/// stream directly to the user rather than being swallowed. See
/// `docs/adr/0001-inherit-stdio-for-clone-and-pull.md`.
pub fn run_inherited(args: &[&str]) -> Result<ExitStatus> {
    debug_log::log(format!("running: git {}", args.join(" ")));
    Command::new("git")
        .args(args)
        .status()
        .with_context(|| format!("failed to run `git {}`", args.join(" ")))
}
