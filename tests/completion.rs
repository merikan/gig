#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::GigTest;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

/// Loose smoke test by design, not a snapshot of `clap_complete`'s exact
/// output - that's generator-internal and shifts across `clap_complete`
/// version bumps without any change on our side. This only checks that the
/// right shell was wired to the right generator and that something usable
/// came out, per
/// docs/adr/0002-scope-shell-completion-to-bash-zsh-fish-structural-only.md.
#[test]
fn bash_completion_contains_bash_marker() {
    let harness = GigTest::new();
    harness
        .cmd()
        .args(["completion", "bash"])
        .assert()
        .success()
        .stdout(contains("complete"));
}

#[test]
fn zsh_completion_contains_zsh_marker() {
    let harness = GigTest::new();
    harness
        .cmd()
        .args(["completion", "zsh"])
        .assert()
        .success()
        .stdout(contains("#compdef gig"));
}

#[test]
fn fish_completion_contains_fish_marker() {
    let harness = GigTest::new();
    harness
        .cmd()
        .args(["completion", "fish"])
        .assert()
        .success()
        .stdout(contains("complete -c gig"));
}

#[test]
fn unsupported_shell_is_rejected() {
    let harness = GigTest::new();
    harness
        .cmd()
        .args(["completion", "powershell"])
        .assert()
        .failure();
}

#[test]
fn generated_script_mentions_subcommands() {
    let harness = GigTest::new();
    harness
        .cmd()
        .args(["completion", "bash"])
        .assert()
        .success()
        .stdout(
            contains("get")
                .and(contains("config"))
                .and(contains("list")),
        );
}
