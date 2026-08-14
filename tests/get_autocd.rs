#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::GigTest;
use predicates::str::contains;
use std::fs;
use std::path::PathBuf;

/// A `GIG_CD_FILE` path that doesn't exist yet - `get` is expected to create
/// it via `std::fs::write` when auto-cd applies, same as `gig shellenv`'s
/// wrapper would hand it a fresh `mktemp` path.
fn cd_file_path(tmp: &tempfile::TempDir) -> PathBuf {
    tmp.path().join("cd-file")
}

#[test]
fn fresh_clone_writes_the_destination_to_the_cd_file_by_default() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let tmp = tempfile::tempdir().unwrap();
    let cd_file = cd_file_path(&tmp);
    let url = "git@github.com:owner/repo.git";

    harness
        .cmd()
        .env("GIG_CD_FILE", &cd_file)
        .args(["get", url])
        .assert()
        .success();

    let destination = common::join(&root_dir, "github.com/owner/repo");
    assert_eq!(
        fs::read_to_string(&cd_file).unwrap(),
        destination.display().to_string()
    );
}

#[test]
fn already_cloned_without_pull_writes_the_destination_to_the_cd_file_by_default() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";
    harness.cmd().args(["get", url]).assert().success();

    let tmp = tempfile::tempdir().unwrap();
    let cd_file = cd_file_path(&tmp);
    harness
        .cmd()
        .env("GIG_CD_FILE", &cd_file)
        .args(["get", url])
        .assert()
        .success();

    let destination = common::join(&root_dir, "github.com/owner/repo");
    assert_eq!(
        fs::read_to_string(&cd_file).unwrap(),
        destination.display().to_string()
    );
}

#[test]
fn pull_writes_the_destination_to_the_cd_file_by_default() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";
    harness.cmd().args(["get", url]).assert().success();

    let tmp = tempfile::tempdir().unwrap();
    let cd_file = cd_file_path(&tmp);
    harness
        .cmd()
        .env("GIG_CD_FILE", &cd_file)
        .args(["get", url, "--pull"])
        .assert()
        .success();

    let destination = common::join(&root_dir, "github.com/owner/repo");
    assert_eq!(
        fs::read_to_string(&cd_file).unwrap(),
        destination.display().to_string()
    );
}

#[test]
fn bare_url_shorthand_also_writes_the_cd_file() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let tmp = tempfile::tempdir().unwrap();
    let cd_file = cd_file_path(&tmp);
    let url = "git@github.com:owner/repo.git";

    harness
        .cmd()
        .env("GIG_CD_FILE", &cd_file)
        .arg(url)
        .assert()
        .success();

    let destination = common::join(&root_dir, "github.com/owner/repo");
    assert_eq!(
        fs::read_to_string(&cd_file).unwrap(),
        destination.display().to_string()
    );
}

#[test]
fn without_the_wrapper_env_var_get_behaves_exactly_as_before() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";

    // No GIG_CD_FILE set at all - the wrapper isn't active.
    harness.cmd().args(["get", url]).assert().success();
}

#[test]
fn autocd_into_false_skips_writing_the_cd_file() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    harness.stub_git.seed_config("gig.autocd-into", "false");
    let tmp = tempfile::tempdir().unwrap();
    let cd_file = cd_file_path(&tmp);
    let url = "git@github.com:owner/repo.git";

    harness
        .cmd()
        .env("GIG_CD_FILE", &cd_file)
        .args(["get", url])
        .assert()
        .success();

    assert!(!cd_file.exists());
}

#[test]
fn no_cd_flag_skips_writing_the_cd_file_even_when_config_is_enabled() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    let tmp = tempfile::tempdir().unwrap();
    let cd_file = cd_file_path(&tmp);
    let url = "git@github.com:owner/repo.git";

    harness
        .cmd()
        .env("GIG_CD_FILE", &cd_file)
        .args(["get", url, "--no-cd"])
        .assert()
        .success();

    assert!(!cd_file.exists());
}

#[test]
fn cd_flag_forces_writing_the_cd_file_even_when_config_is_disabled() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    harness.stub_git.seed_config("gig.autocd-into", "false");
    let tmp = tempfile::tempdir().unwrap();
    let cd_file = cd_file_path(&tmp);
    let url = "git@github.com:owner/repo.git";

    harness
        .cmd()
        .env("GIG_CD_FILE", &cd_file)
        .args(["get", url, "--cd"])
        .assert()
        .success();

    let destination = common::join(&root_dir, "github.com/owner/repo");
    assert_eq!(
        fs::read_to_string(&cd_file).unwrap(),
        destination.display().to_string()
    );
}

#[test]
fn cd_and_no_cd_together_is_a_mutually_exclusive_error() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";

    harness
        .cmd()
        .args(["get", url, "--cd", "--no-cd"])
        .assert()
        .failure()
        .stderr(contains("--cd and --no-cd are mutually exclusive"));

    assert!(harness.stub_git.calls_starting_with("clone\t").is_empty());
}

#[test]
fn occupied_destination_does_not_write_the_cd_file() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";
    let destination = common::join(&root_dir, "github.com/owner/repo");
    fs::create_dir_all(&destination).unwrap();
    fs::write(destination.join("stray-file.txt"), "not a git repo").unwrap();
    let tmp = tempfile::tempdir().unwrap();
    let cd_file = cd_file_path(&tmp);

    harness
        .cmd()
        .env("GIG_CD_FILE", &cd_file)
        .args(["get", url])
        .assert()
        .failure();

    assert!(!cd_file.exists());
}

#[test]
fn cd_file_write_failure_warns_but_the_invocation_still_succeeds() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";
    // A path under a directory that doesn't exist - std::fs::write fails.
    let unwritable_cd_file = tempfile::tempdir().unwrap().path().join("gone/cd-file");

    harness
        .cmd()
        .env("GIG_CD_FILE", &unwritable_cd_file)
        .args(["get", url])
        .assert()
        .success()
        .stderr(contains("warning: failed to write auto-cd file"));

    // the clone itself still happened
    assert_eq!(harness.stub_git.calls_starting_with("clone\t").len(), 1);
}

#[test]
fn autocd_into_config_can_be_set_and_viewed() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "autocd-into", "false"])
        .assert()
        .success();
    assert_eq!(
        harness.stub_git.calls(),
        vec!["config\t--global\tgig.autocd-into\tfalse"]
    );

    harness
        .cmd()
        .args(["config", "autocd-into"])
        .assert()
        .success()
        .stdout("false\n");
}

#[test]
fn autocd_into_defaults_to_true_when_unset() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "autocd-into"])
        .assert()
        .success()
        .stdout("true\n");
}

#[test]
fn autocd_into_rejects_anything_other_than_true_or_false() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "autocd-into", "yes"])
        .assert()
        .failure()
        .stderr(contains(
            "invalid value 'yes' for gig.autocd-into - must be 'true' or 'false'",
        ));

    // never wrote the invalid value
    assert!(
        harness
            .stub_git
            .calls_starting_with("config\t--global\tgig.autocd-into")
            .is_empty()
    );
}
