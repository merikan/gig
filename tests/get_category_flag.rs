#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::GigTest;
use predicates::str::contains;

#[test]
fn category_flag_overrides_automatic_matching_for_a_declared_category() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    // "personal" would auto-match this URL - the flag must win anyway.
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");
    harness
        .stub_git
        .seed_config("gig.category.work.pattern", "");
    let url = "git@github.com:merikan/gig.git";

    harness
        .cmd()
        .args(["get", url, "--category", "work"])
        .assert()
        .success();

    let destination = root_dir.join("work/github.com/merikan/gig");
    assert_eq!(
        harness.stub_git.calls_starting_with("clone\t"),
        vec![format!("clone\t{url}\t{}", destination.display())]
    );
    assert!(destination.join(".git").is_dir());
}

#[test]
fn category_flag_with_an_undeclared_name_errors_and_attempts_no_clone() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    let url = "git@github.com:merikan/gig.git";

    harness
        .cmd()
        .args(["get", url, "--category", "ghost"])
        .assert()
        .failure()
        .stderr(contains("ghost"));

    assert!(harness.stub_git.calls_starting_with("clone\t").is_empty());
}

#[test]
fn category_flag_is_the_only_way_to_reach_a_flag_only_category() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    harness
        .stub_git
        .seed_config("gig.category.oneoff.pattern", "");
    let url = "git@github.com:merikan/gig.git";

    harness
        .cmd()
        .args(["get", url, "--category", "oneoff"])
        .assert()
        .success();

    let destination = root_dir.join("oneoff/github.com/merikan/gig");
    assert_eq!(
        harness.stub_git.calls_starting_with("clone\t"),
        vec![format!("clone\t{url}\t{}", destination.display())]
    );
}
