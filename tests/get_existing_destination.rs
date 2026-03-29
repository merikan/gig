#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::GigTest;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::fs;

#[test]
fn running_get_twice_on_the_same_url_no_ops_on_the_second_run() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";

    harness.cmd().args(["get", url]).assert().success();
    assert_eq!(harness.stub_git.calls_starting_with("clone\t").len(), 1);

    harness.cmd().args(["get", url]).assert().success();

    // no second clone, and no pull was invoked either
    assert_eq!(harness.stub_git.calls_starting_with("clone\t").len(), 1);
    assert!(harness.stub_git.calls_starting_with("-C\t").is_empty());
}

#[test]
fn rerun_without_pull_prints_the_already_cloned_message_with_the_destination() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";
    let destination = root_dir.join("github.com/owner/repo");

    harness.cmd().args(["get", url]).assert().success();

    harness
        .cmd()
        .args(["get", url])
        .assert()
        .success()
        .stdout(contains(format!(
            "Already cloned at {}",
            destination.display()
        )));
}

#[test]
fn pull_flag_defaults_to_false_on_a_plain_rerun() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";

    harness.cmd().args(["get", url]).assert().success();
    harness.cmd().args(["get", url]).assert().success();

    assert!(harness.stub_git.calls_starting_with("-C\t").is_empty());
}

#[test]
fn pull_prints_no_gig_message_and_shows_gits_own_pull_output() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";

    harness.cmd().args(["get", url]).assert().success();

    harness
        .cmd()
        .args(["get", url, "--pull"])
        .assert()
        .success()
        .stdout(contains("stub-git: already up to date").and(contains("Already cloned at").not()));
}

#[test]
fn pull_flag_invokes_git_pull_in_the_destination_directory_not_clone() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";
    let destination = root_dir.join("github.com/owner/repo");

    harness.cmd().args(["get", url]).assert().success();
    assert_eq!(harness.stub_git.calls_starting_with("clone\t").len(), 1);

    harness
        .cmd()
        .args(["get", url, "--pull"])
        .assert()
        .success();

    assert_eq!(harness.stub_git.calls_starting_with("clone\t").len(), 1);
    assert_eq!(
        harness.stub_git.calls_starting_with("-C\t"),
        vec![format!("-C\t{}\tpull", destination.display())]
    );
}

#[test]
fn bare_url_without_get_keyword_with_pull_on_an_already_cloned_repo_pulls() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";
    let destination = root_dir.join("github.com/owner/repo");

    harness.cmd().arg(url).assert().success();
    assert_eq!(harness.stub_git.calls_starting_with("clone\t").len(), 1);

    harness.cmd().args([url, "--pull"]).assert().success();

    assert_eq!(harness.stub_git.calls_starting_with("clone\t").len(), 1);
    assert_eq!(
        harness.stub_git.calls_starting_with("-C\t"),
        vec![format!("-C\t{}\tpull", destination.display())]
    );
}

#[test]
fn destination_occupied_by_a_non_git_directory_errors_without_cloning() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";
    let destination = root_dir.join("github.com/owner/repo");
    fs::create_dir_all(&destination).unwrap();
    fs::write(destination.join("stray-file.txt"), "not a git repo").unwrap();

    harness
        .cmd()
        .args(["get", url])
        .assert()
        .failure()
        .stderr(contains("already exists and is not a git repository"));

    assert!(harness.stub_git.calls_starting_with("clone\t").is_empty());
    // existing contents are untouched
    assert!(destination.join("stray-file.txt").is_file());
    assert!(!destination.join(".git").exists());
}

#[test]
fn destination_occupied_by_a_stray_file_errors_without_cloning() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";
    let destination = root_dir.join("github.com/owner/repo");
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    fs::write(&destination, "not even a directory").unwrap();

    harness
        .cmd()
        .args(["get", url])
        .assert()
        .failure()
        .stderr(contains("already exists and is not a git repository"));

    assert!(harness.stub_git.calls_starting_with("clone\t").is_empty());
    assert!(destination.is_file());
}

#[test]
fn destination_occupied_by_an_empty_directory_errors_without_cloning() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";
    let destination = root_dir.join("github.com/owner/repo");
    fs::create_dir_all(&destination).unwrap();

    harness
        .cmd()
        .args(["get", url])
        .assert()
        .failure()
        .stderr(contains("already exists and is not a git repository"));

    assert!(harness.stub_git.calls_starting_with("clone\t").is_empty());
}
