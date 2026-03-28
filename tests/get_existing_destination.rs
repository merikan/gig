mod common;

use common::GitGetTest;
use predicates::str::contains;
use std::fs;

#[test]
fn running_get_twice_on_the_same_url_no_ops_on_the_second_run() {
    let harness = GitGetTest::new();
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
fn pull_flag_defaults_to_false_on_a_plain_rerun() {
    let harness = GitGetTest::new();
    harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";

    harness.cmd().args(["get", url]).assert().success();
    harness.cmd().args(["get", url]).assert().success();

    assert!(harness.stub_git.calls_starting_with("-C\t").is_empty());
}

#[test]
fn pull_flag_invokes_git_pull_in_the_destination_directory_not_clone() {
    let harness = GitGetTest::new();
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
    let harness = GitGetTest::new();
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
    let harness = GitGetTest::new();
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
    let harness = GitGetTest::new();
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
    let harness = GitGetTest::new();
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
