#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::GigTest;
use predicates::str::contains;
use std::fs;
use std::path::Path;

/// Seeds a fake clone (a directory containing a `.git` subdirectory) at
/// `root_dir/relative_path`, bypassing `get` entirely - `list` only cares
/// that a `.git` marker exists on disk.
fn seed_fake_clone(root_dir: &Path, relative_path: &str) {
    let repo_dir = common::join(root_dir, relative_path);
    fs::create_dir_all(repo_dir.join(".git")).unwrap();
}

#[test]
fn lists_repos_at_varying_depths_sorted_relative_and_one_per_line() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    seed_fake_clone(&root_dir, "gitlab.com/group/subgroup/repo");
    seed_fake_clone(&root_dir, "github.com/owner/repo");
    seed_fake_clone(&root_dir, "git.sr.ht/~user/repo");

    harness
        .cmd()
        .arg("list")
        .assert()
        .success()
        .stdout("git.sr.ht/~user/repo\ngithub.com/owner/repo\ngitlab.com/group/subgroup/repo\n");
}

#[test]
fn ls_is_an_alias_for_list() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    seed_fake_clone(&root_dir, "github.com/owner/repo");

    harness
        .cmd()
        .arg("ls")
        .assert()
        .success()
        .stdout("github.com/owner/repo\n");
}

#[test]
fn empty_root_dir_produces_empty_output_not_an_error() {
    let harness = GigTest::new();
    harness.seed_root_dir();

    harness.cmd().arg("list").assert().success().stdout("");
}

#[test]
fn root_dir_that_does_not_exist_on_disk_yet_produces_empty_output_not_an_error() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    assert!(!root_dir.exists());

    harness.cmd().arg("list").assert().success().stdout("");
}

#[test]
fn non_git_directories_and_stray_files_are_not_listed() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    seed_fake_clone(&root_dir, "github.com/owner/repo");
    fs::create_dir_all(common::join(&root_dir, "github.com/owner/not-a-repo")).unwrap();
    fs::write(
        common::join(&root_dir, "github.com/owner/stray-file.txt"),
        "hi",
    )
    .unwrap();

    harness
        .cmd()
        .arg("list")
        .assert()
        .success()
        .stdout("github.com/owner/repo\n");
}

#[test]
fn listing_with_root_dir_unset_errors_with_the_command_to_run() {
    let harness = GigTest::new();

    harness
        .cmd()
        .arg("list")
        .assert()
        .failure()
        .stderr(contains("gig config root-dir <path>"));
}
