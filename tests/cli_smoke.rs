#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::GigTest;

#[test]
fn get_with_explicit_subcommand_parses() {
    let harness = GigTest::new();
    let root_dir = harness.home_dir.join("root");
    harness
        .stub_git
        .seed_config("gig.root-dir", root_dir.to_str().unwrap());
    harness
        .cmd()
        .args(["get", "git@github.com:owner/repo.git"])
        .assert()
        .success();
}

#[test]
fn get_via_default_subcommand_parses() {
    let harness = GigTest::new();
    let root_dir = harness.home_dir.join("root");
    harness
        .stub_git
        .seed_config("gig.root-dir", root_dir.to_str().unwrap());
    harness
        .cmd()
        .arg("git@github.com:owner/repo.git")
        .assert()
        .success();
}

#[test]
fn get_with_pull_and_category_flags_parses() {
    let harness = GigTest::new();
    let root_dir = harness.home_dir.join("root");
    harness
        .stub_git
        .seed_config("gig.root-dir", root_dir.to_str().unwrap());
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
    // root-dir is unset in a fresh harness, so this is expected to fail with a
    // business error, not a clap parse error - the "set" and "value already set"
    // cases are covered precisely by tests/config_root_dir.rs.
    let harness = GigTest::new();
    harness
        .cmd()
        .args(["config", "root-dir"])
        .assert()
        .failure();
}

#[test]
fn config_root_dir_set_parses() {
    let harness = GigTest::new();
    harness
        .cmd()
        .args(["config", "root-dir", "/tmp/somewhere"])
        .assert()
        .success();
}

#[test]
fn config_category_set_parses() {
    let harness = GigTest::new();
    harness
        .cmd()
        .args(["config", "category", "personal", "^github\\.com/merikan/"])
        .assert()
        .success();
}

#[test]
fn config_category_view_parses() {
    let harness = GigTest::new();
    harness
        .cmd()
        .args(["config", "category", "personal"])
        .assert()
        .success();
}

#[test]
fn config_category_flag_only_parses() {
    let harness = GigTest::new();
    harness
        .cmd()
        .args(["config", "category", "oneoff", "--flag-only"])
        .assert()
        .success();
}

#[test]
fn config_category_list_all_parses() {
    let harness = GigTest::new();
    harness
        .cmd()
        .args(["config", "category"])
        .assert()
        .success();
}

#[test]
fn list_parses() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    harness.cmd().arg("list").assert().success();
}
