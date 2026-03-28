#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::GitGetTest;
use predicates::str::contains;

#[test]
fn setting_root_dir_invokes_git_config_global() {
    let harness = GitGetTest::new();

    harness
        .cmd()
        .args(["config", "root-dir", "/tmp/somewhere"])
        .assert()
        .success();

    assert_eq!(
        harness.stub_git.calls(),
        vec!["config\t--global\tgit-get.root-dir\t/tmp/somewhere"]
    );
}

#[test]
fn viewing_root_dir_prints_the_configured_value() {
    let harness = GitGetTest::new();
    harness
        .stub_git
        .seed_config("git-get.root-dir", "/tmp/somewhere");

    harness
        .cmd()
        .args(["config", "root-dir"])
        .assert()
        .success()
        .stdout("/tmp/somewhere\n");
}

#[test]
fn viewing_unset_root_dir_errors_with_the_command_to_run() {
    let harness = GitGetTest::new();

    harness
        .cmd()
        .args(["config", "root-dir"])
        .assert()
        .failure()
        .stderr(contains("git-get config root-dir <path>"));
}

#[test]
fn viewing_root_dir_propagates_a_real_git_config_error() {
    let harness = GitGetTest::new();

    harness
        .cmd()
        .env("GIT_GET_STUB_EXIT_CODE", "2")
        .env(
            "GIT_GET_STUB_STDERR",
            "fatal: bad config line 3 in file .gitconfig",
        )
        .args(["config", "root-dir"])
        .assert()
        .failure()
        .stderr(contains("fatal: bad config line 3 in file .gitconfig"));
}

#[test]
fn setting_root_dir_propagates_a_git_config_error() {
    let harness = GitGetTest::new();

    harness
        .cmd()
        .env("GIT_GET_STUB_EXIT_CODE", "1")
        .env("GIT_GET_STUB_STDERR", "fatal: could not lock config file")
        .args(["config", "root-dir", "/tmp/somewhere"])
        .assert()
        .failure()
        .stderr(contains("fatal: could not lock config file"));
}
