#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::GigTest;
use predicates::str::is_match;

/// The commit portion varies per checkout, so this pins the shape rather
/// than an exact hash: `<pkg-version> (<short-commit>[-dirty])`, or
/// `<pkg-version> (unknown)` for a build with no git info available (see
/// `src/version.rs`).
const VERSION_PATTERN: &str = r"^gig \d+\.\d+\.\d+ \((?:[0-9a-f]{7,40}(-dirty)?|unknown)\)\n$";

#[test]
fn long_flag_prints_version_with_commit() {
    let harness = GigTest::new();
    harness
        .cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(is_match(VERSION_PATTERN).unwrap());
}

#[test]
fn short_flag_prints_version_with_commit() {
    let harness = GigTest::new();
    harness
        .cmd()
        .arg("-V")
        .assert()
        .success()
        .stdout(is_match(VERSION_PATTERN).unwrap());
}

#[test]
fn version_subcommand_prints_version_with_commit() {
    let harness = GigTest::new();
    harness
        .cmd()
        .arg("version")
        .assert()
        .success()
        .stdout(is_match(VERSION_PATTERN).unwrap());
}

#[test]
fn version_subcommand_matches_long_flag_output_exactly() {
    let harness = GigTest::new();
    let flag_output = harness.cmd().arg("--version").output().unwrap();
    let subcommand_output = harness.cmd().arg("version").output().unwrap();

    assert_eq!(flag_output.stdout, subcommand_output.stdout);
}
