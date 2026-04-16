//! Wraps the system `git` binary's config store for category routing rules,
//! parallel to [`crate::git_config`]'s handling of `root-dir`. A category is
//! declared by the existence of a `gig.category.<name>.pattern` key, even
//! with an empty value - that's what lets a flag-only category (no regex,
//! usable only via `--category`) register for enumeration.
use crate::git_cmd::run;
use crate::git_config;
use anyhow::{Result, bail};

/// The exact `--get-regexp` pattern used to enumerate every declared
/// category, in git-config declaration order - the same order automatic
/// category matching uses for precedence.
const ENUMERATE_PATTERN: &str = r"^gig\.category\..*\.pattern$";

/// A single declared category and its pattern.
pub struct Category {
    pub name: String,
    pub pattern: String,
}

/// `git config --get-regexp` over every declared category, in declaration
/// order. No declared categories yields an empty list rather than an error -
/// that's how `--get-regexp` itself reports "no matches", via exit code 1.
pub fn list() -> Result<Vec<Category>> {
    let output = run(&["config", "--get-regexp", ENUMERATE_PATTERN])?;

    if output.status.success() {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(parse_entry_line)
            .collect()
    } else if output.status.code() == Some(1) {
        Ok(Vec::new())
    } else {
        bail!(
            "git config --get-regexp {ENUMERATE_PATTERN} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
}

/// Parses one `--get-regexp` output line (`<key> <value>`, with `<value>`
/// possibly empty) into its category name and pattern.
fn parse_entry_line(line: &str) -> Result<Category> {
    let (key, pattern) = line.split_once(' ').unwrap_or((line, ""));
    let name = key
        .strip_prefix("gig.category.")
        .and_then(|rest| rest.strip_suffix(".pattern"))
        .ok_or_else(|| anyhow::anyhow!("unexpected `git config --get-regexp` key: {key}"))?;
    Ok(Category {
        name: name.to_string(),
        pattern: pattern.to_string(),
    })
}

/// The current pattern for a declared category, or `None` if `name` was
/// never declared.
pub fn get(name: &str) -> Result<Option<String>> {
    git_config::get(&key(name))
}

/// Declares (or updates) a category with the given pattern. An empty
/// `pattern` declares a flag-only category - usable via `--category`, never
/// auto-matched.
pub fn set(name: &str, pattern: &str) -> Result<()> {
    git_config::set_global(&key(name), pattern)
}

fn key(name: &str) -> String {
    format!("gig.category.{name}.pattern")
}
