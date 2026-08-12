//! Build-time version info (package version + git commit), shared by `-V`/
//! `--version` (wired via `#[command(version = ...)]` in `cli.rs`) and the
//! `gig version` subcommand, so the two stay identical by construction.

// `built` writes this itself; its lint posture doesn't need to match ours.
#[allow(clippy::pedantic, clippy::nursery, missing_docs)]
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// `"<pkg-version> (<short-commit>[-dirty])"`, or `"<pkg-version> (unknown)"`
/// when no git commit could be determined at build time (source tarball,
/// `cargo install`, or a shallow/gitless checkout).
pub fn string() -> String {
    let pkg_version = built_info::PKG_VERSION;
    let Some(commit) = built_info::GIT_COMMIT_HASH_SHORT else {
        return format!("{pkg_version} (unknown)");
    };
    let dirty_suffix = if built_info::GIT_DIRTY == Some(true) {
        "-dirty"
    } else {
        ""
    };
    format!("{pkg_version} ({commit}{dirty_suffix})")
}

/// `string()`, computed once and cached for `'static` lifetime - clap's
/// `#[command(version = ...)]` needs a `&'static str`, and the derive macro
/// may re-materialize the `Command` more than once per process (e.g. once
/// for parsing, again for `gig completion`'s generation).
pub fn static_str() -> &'static str {
    static VERSION: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    VERSION.get_or_init(string)
}
