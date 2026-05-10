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

/// Every pattern currently declared for `name`, in declaration/add order.
/// An empty vec means `name` isn't declared at all - a flag-only category
/// still yields one entry (a single empty string), matching how it's stored.
pub fn patterns(name: &str) -> Result<Vec<String>> {
    git_config::get_all(&key(name))
}

/// Declares/replaces `name`'s entire pattern list with `patterns` (must be
/// non-empty - a flag-only declaration is represented by `[""]`). `existing`
/// is `name`'s pattern list *before* this call, as already read by the
/// caller (e.g. to decide whether to print the destructive-replace warning) -
/// passed in rather than re-read here to avoid a redundant `git config`
/// invocation.
///
/// Plain `git config --global` can only ever hold one value, and errors if
/// the key it's asked to overwrite already holds more than one - so once
/// either the previous or the new list has more than one entry, this falls
/// back to `--replace-all` (clear every existing value, write the first new
/// one) followed by `--add` for the rest. The plain single-value set is used
/// whenever that's safe, preserving the exact invocation single-pattern
/// categories have always used.
pub fn replace(name: &str, existing: &[String], patterns: &[String]) -> Result<()> {
    let key = key(name);
    match (existing.len() <= 1, patterns) {
        (true, [only]) => git_config::set_global(&key, only),
        (_, [first, rest @ ..]) => {
            git_config::replace_all_global(&key, first)?;
            for pattern in rest {
                git_config::add_global(&key, pattern)?;
            }
            Ok(())
        }
        (_, []) => bail!("replace requires at least one pattern"),
    }
}

/// Appends `patterns` to `name`'s existing list (`existing`, as already read
/// by the caller - see [`replace`] for why it's passed in rather than
/// re-read). Skips any pattern already present verbatim, so re-running the
/// same `add` twice doesn't grow the list. Assumes the caller has already
/// verified `name` is declared (`existing` non-empty) - `add` never creates a
/// category.
pub fn add(name: &str, existing: &[String], patterns: &[String]) -> Result<()> {
    let key = key(name);
    let mut current = existing.to_vec();
    for pattern in patterns {
        if !current.contains(pattern) {
            git_config::add_global(&key, pattern)?;
            current.push(pattern.clone());
        }
    }
    Ok(())
}

fn key(name: &str) -> String {
    format!("gig.category.{name}.pattern")
}

/// Rejects a category `name` that could escape `root-dir` when interpolated
/// as a raw path segment (`root-dir/<name>/host/owner/repo`, see
/// [`crate::category_routing`]) - `/`, `\`, or `..` anywhere in `name` would
/// let a category's clone target step outside `root-dir`. Only meant to gate
/// declaring/updating a category ([`add`]/[`replace`]'s callers); reading or
/// listing an already-declared category never calls this, so a name that
/// predates this guard still reads back fine.
pub fn validate_name(name: &str) -> Result<()> {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        bail!("category name '{name}' is not path-safe - it must not contain `/`, `\\`, or `..`");
    }
    Ok(())
}
