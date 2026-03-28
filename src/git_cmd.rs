//! Shared plumbing for shelling out to the system `git` binary. Every
//! git-get module that wraps a `git` invocation (`git_config`, `git_ops`, and
//! eventually category-config) runs it through here rather than each calling
//! `Command::new("git")` itself.
use anyhow::{Context, Result};
use std::process::{Command, Output};

pub fn run(args: &[&str]) -> Result<Output> {
    Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("failed to run `git {}`", args.join(" ")))
}
