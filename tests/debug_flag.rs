#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::GigTest;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

#[test]
fn without_debug_no_diagnostic_lines_are_printed() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";

    harness
        .cmd()
        .args(["get", url])
        .assert()
        .success()
        .stderr(contains("[debug]").not());

    assert!(root_dir.join("github.com/owner/repo/.git").is_dir());
}

#[test]
fn debug_reveals_the_git_commands_run_during_a_clone() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";
    let destination = root_dir.join("github.com/owner/repo");

    harness
        .cmd()
        .args(["get", url, "--debug"])
        .assert()
        .success()
        .stderr(
            contains("[debug] running: git config --get-regexp")
                .and(contains(format!(
                    "[debug] running: git clone {url} {}",
                    destination.display()
                )))
                .and(contains(format!(
                    "[debug] destination {} classified as free",
                    destination.display()
                ))),
        );
}

#[test]
fn debug_reveals_candidate_destination_probing_and_classification_for_an_already_cloned_repo() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";
    let destination = root_dir.join("github.com/owner/repo");

    harness.cmd().args(["get", url]).assert().success();

    harness
        .cmd()
        .args(["get", url, "--debug"])
        .assert()
        .success()
        .stderr(
            contains(format!(
                "[debug] candidate destination {}: already cloned",
                destination.display()
            ))
            .and(contains(format!(
                "[debug] destination {} classified as already-cloned",
                destination.display()
            ))),
        );
}

#[test]
fn debug_reveals_which_category_pattern_matched() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");
    let url = "git@github.com:merikan/gig.git";

    harness
        .cmd()
        .args(["get", url, "--debug"])
        .assert()
        .success()
        .stderr(contains(
            "[debug] category routing: 'github.com/merikan/gig' against 1 declared categories -> personal",
        ));
}

#[test]
fn debug_reveals_a_category_flag_override() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    harness
        .stub_git
        .seed_config("gig.category.work.pattern", "");
    let url = "git@github.com:merikan/gig.git";

    harness
        .cmd()
        .args(["get", url, "--category", "work", "--debug"])
        .assert()
        .success()
        .stderr(contains(
            "[debug] --category work overrides automatic category matching",
        ));
}

#[test]
fn debug_before_the_implicit_get_default_subcommand_still_resolves_the_url_not_as_a_flag() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";

    harness
        .cmd()
        .args(["--debug", url])
        .assert()
        .success()
        .stderr(contains("[debug] running: git clone"));

    assert!(root_dir.join("github.com/owner/repo/.git").is_dir());
}

#[test]
fn debug_before_an_explicit_subcommand_still_resolves_it_correctly() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["--debug", "config", "root-dir", "/tmp/somewhere"])
        .assert()
        .success()
        .stderr(contains(
            "[debug] running: git config --global gig.root-dir /tmp/somewhere",
        ));
}

#[test]
fn debug_does_not_change_stdout_output() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";
    let destination = root_dir.join("github.com/owner/repo");

    harness
        .cmd()
        .args(["get", url, "--debug"])
        .assert()
        .success()
        .stdout(
            contains(format!("Cloning {url} into {}...", destination.display()))
                .and(contains(format!("Cloned into {}", destination.display()))),
        );
}
