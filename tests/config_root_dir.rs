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
