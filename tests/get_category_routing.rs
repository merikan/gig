#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::GigTest;
use predicates::str::contains;

#[test]
fn a_url_matching_a_declared_category_clones_under_root_dir_category() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");
    let url = "git@github.com:merikan/gig.git";

    harness.cmd().args(["get", url]).assert().success();

    let destination = root_dir.join("personal/github.com/merikan/gig");
    assert_eq!(
        harness.stub_git.calls_starting_with("clone\t"),
        vec![format!("clone\t{url}\t{}", destination.display())]
    );
    assert!(destination.join(".git").is_dir());
}

#[test]
fn a_url_matching_no_declared_category_falls_back_to_the_default_path() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");
    let url = "git@github.com:someone-else/repo.git";

    harness.cmd().args(["get", url]).assert().success();

    let destination = root_dir.join("github.com/someone-else/repo");
    assert_eq!(
        harness.stub_git.calls_starting_with("clone\t"),
        vec![format!("clone\t{url}\t{}", destination.display())]
    );
}

#[test]
fn the_first_declared_of_two_overlapping_categories_wins() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    // Declaration order: "specific" first, then "broad" - both match the URL
    // below, so "specific" must win.
    harness.stub_git.seed_config(
        "gig.category.specific.pattern",
        r"^github\.com/merikan/gig$",
    );
    harness
        .stub_git
        .seed_config("gig.category.broad.pattern", r"^github\.com/merikan/");
    let url = "git@github.com:merikan/gig.git";

    harness.cmd().args(["get", url]).assert().success();

    let destination = root_dir.join("specific/github.com/merikan/gig");
    assert_eq!(
        harness.stub_git.calls_starting_with("clone\t"),
        vec![format!("clone\t{url}\t{}", destination.display())]
    );
}

#[test]
fn a_category_declared_with_an_empty_pattern_is_never_auto_matched() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    harness
        .stub_git
        .seed_config("gig.category.oneoff.pattern", "");
    let url = "git@github.com:merikan/gig.git";

    harness.cmd().args(["get", url]).assert().success();

    let destination = root_dir.join("github.com/merikan/gig");
    assert_eq!(
        harness.stub_git.calls_starting_with("clone\t"),
        vec![format!("clone\t{url}\t{}", destination.display())]
    );
}

#[test]
fn an_invalid_regex_in_a_declared_category_aborts_up_front_naming_it() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    // "broken"'s pattern is invalid and wouldn't have matched this URL even
    // if it did compile - it must still be caught up front.
    harness
        .stub_git
        .seed_config("gig.category.broken.pattern", "(");
    let url = "git@github.com:merikan/gig.git";

    harness
        .cmd()
        .args(["get", url])
        .assert()
        .failure()
        .stderr(contains("broken"));

    assert!(harness.stub_git.calls_starting_with("clone\t").is_empty());
}

#[test]
fn after_a_category_placed_clone_list_shows_it_with_the_category_as_a_prefix() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");
    let url = "git@github.com:merikan/gig.git";

    harness.cmd().args(["get", url]).assert().success();

    harness
        .cmd()
        .arg("list")
        .assert()
        .success()
        .stdout("personal/github.com/merikan/gig\n");
}
