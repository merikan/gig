mod common;

use common::GitGetTest;

#[test]
fn get_with_explicit_subcommand_parses() {
    let harness = GitGetTest::new();
    harness
        .cmd()
        .args(["get", "git@github.com:owner/repo.git"])
        .assert()
        .success();
}

#[test]
fn get_via_default_subcommand_parses() {
    let harness = GitGetTest::new();
    harness
        .cmd()
        .arg("git@github.com:owner/repo.git")
        .assert()
        .success();
}

#[test]
fn get_with_pull_and_category_flags_parses() {
    let harness = GitGetTest::new();
    harness
        .cmd()
        .args([
            "git@github.com:owner/repo.git",
            "--pull",
            "--category",
            "personal",
        ])
        .assert()
        .success();
}

#[test]
fn config_root_dir_view_parses() {
    let harness = GitGetTest::new();
    harness
        .cmd()
        .args(["config", "root-dir"])
        .assert()
        .success();
}

#[test]
fn config_root_dir_set_parses() {
    let harness = GitGetTest::new();
    harness
        .cmd()
        .args(["config", "root-dir", "/tmp/somewhere"])
        .assert()
        .success();
}

#[test]
fn config_category_set_parses() {
    let harness = GitGetTest::new();
    harness
        .cmd()
        .args(["config", "category", "personal", "^github\\.com/merikan/"])
        .assert()
        .success();
}

#[test]
fn config_category_view_parses() {
    let harness = GitGetTest::new();
    harness
        .cmd()
        .args(["config", "category", "personal"])
        .assert()
        .success();
}

#[test]
fn config_category_flag_only_parses() {
    let harness = GitGetTest::new();
    harness
        .cmd()
        .args(["config", "category", "oneoff", "--flag-only"])
        .assert()
        .success();
}

#[test]
fn config_category_list_all_parses() {
    let harness = GitGetTest::new();
    harness
        .cmd()
        .args(["config", "category"])
        .assert()
        .success();
}

#[test]
fn list_parses() {
    let harness = GitGetTest::new();
    harness.cmd().arg("list").assert().success();
}
