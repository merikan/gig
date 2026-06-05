#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::GigTest;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

#[test]
fn bash_and_zsh_get_the_same_posix_function() {
    let harness = GigTest::new();
    let bash = harness
        .cmd()
        .args(["shellenv", "bash"])
        .output()
        .unwrap()
        .stdout;
    let zsh = harness
        .cmd()
        .args(["shellenv", "zsh"])
        .output()
        .unwrap()
        .stdout;
    assert_eq!(bash, zsh);
}

#[test]
fn bash_shellenv_defines_a_gig_function_that_shells_out_to_the_real_binary() {
    let harness = GigTest::new();
    harness
        .cmd()
        .args(["shellenv", "bash"])
        .assert()
        .success()
        .stdout(contains("gig() {").and(contains("command gig")));
}

#[test]
fn fish_shellenv_uses_fish_function_syntax() {
    let harness = GigTest::new();
    harness
        .cmd()
        .args(["shellenv", "fish"])
        .assert()
        .success()
        .stdout(contains("function gig").and(contains("command gig")));
}

#[test]
fn unsupported_shell_is_rejected() {
    let harness = GigTest::new();
    harness
        .cmd()
        .args(["shellenv", "powershell"])
        .assert()
        .failure();
}
