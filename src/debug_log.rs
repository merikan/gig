//! Diagnostic output for `--debug`: short-lived, human-readable lines to
//! stderr describing what gig is doing under the hood (git commands run,
//! category routing, destination resolution) - never persisted, never
//! structured, and orthogonal to `get`'s normal stdout output.
//!
//! [`init`] latches whether `--debug` was passed exactly once, at the very
//! start of `main`, before any other gig logic runs. From then on it's
//! read-only for the rest of the process, so [`log`] can be called from
//! anywhere (including deep call chains like `git_cmd::run`) without
//! threading a flag through every function signature in between.
use std::sync::OnceLock;

static ENABLED: OnceLock<bool> = OnceLock::new();

/// Latches whether `--debug` was passed. Must be called exactly once, before
/// any other gig logic runs - see `main`. A second call is silently ignored
/// rather than treated as an error; nothing in gig makes one.
pub fn init(enabled: bool) {
    let _ = ENABLED.set(enabled);
}

/// Prints `message` to stderr, prefixed with `[debug]`, if `--debug` was
/// passed - a no-op otherwise (including if [`init`] was never called, e.g.
/// from a unit test that never goes through `main`).
pub fn log(message: impl std::fmt::Display) {
    if ENABLED.get().copied().unwrap_or(false) {
        eprintln!("[debug] {message}");
    }
}
