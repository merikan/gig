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

    let destination = common::join(&root_dir, "personal/github.com/merikan/gig");
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

    let destination = common::join(&root_dir, "github.com/someone-else/repo");
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

    let destination = common::join(&root_dir, "specific/github.com/merikan/gig");
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

    let destination = common::join(&root_dir, "github.com/merikan/gig");
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
fn a_category_declared_with_two_patterns_auto_matches_a_url_matching_either_one() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    harness
        .stub_git
        .seed_config("gig.category.oss.pattern", r"^github\.com/rust-lang/");
    harness
        .stub_git
        .seed_config_add("gig.category.oss.pattern", r"^github\.com/tokio-rs/");

    harness
        .cmd()
        .args(["get", "git@github.com:rust-lang/regex.git"])
        .assert()
        .success();
    harness
        .cmd()
        .args(["get", "git@github.com:tokio-rs/tokio.git"])
        .assert()
        .success();

    assert!(common::join(&root_dir, "oss/github.com/rust-lang/regex/.git").is_dir());
    assert!(common::join(&root_dir, "oss/github.com/tokio-rs/tokio/.git").is_dir());
}

#[test]
fn a_pattern_appended_via_config_category_add_routes_alongside_the_original() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    harness
        .stub_git
        .seed_config("gig.category.oss.pattern", r"^github\.com/rust-lang/");

    harness
        .cmd()
        .args([
            "config",
            "category",
            "oss",
            r"^github\.com/tokio-rs/",
            "--add",
        ])
        .assert()
        .success();

    harness
        .cmd()
        .args(["get", "git@github.com:rust-lang/regex.git"])
        .assert()
        .success();
    harness
        .cmd()
        .args(["get", "git@github.com:tokio-rs/tokio.git"])
        .assert()
        .success();

    assert!(common::join(&root_dir, "oss/github.com/rust-lang/regex/.git").is_dir());
    assert!(common::join(&root_dir, "oss/github.com/tokio-rs/tokio/.git").is_dir());
}

#[test]
fn an_invalid_regex_among_several_patterns_in_a_category_still_aborts_up_front_naming_it() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    harness
        .stub_git
        .seed_config("gig.category.mixed.pattern", r"^github\.com/merikan/");
    // Second pattern is invalid - it must still be caught even though the
    // first pattern already compiles fine.
    harness
        .stub_git
        .seed_config_add("gig.category.mixed.pattern", "(");
    let url = "git@github.com:merikan/gig.git";

    harness
        .cmd()
        .args(["get", url])
        .assert()
        .failure()
        .stderr(contains("mixed"));

    assert!(harness.stub_git.calls_starting_with("clone\t").is_empty());
}

#[test]
fn a_url_matching_no_category_routes_to_the_declared_default_category() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", "");
    harness
        .stub_git
        .seed_config("gig.category.personal.default", "true");
    let url = "git@github.com:someone-else/repo.git";

    harness.cmd().args(["get", url]).assert().success();

    let destination = common::join(&root_dir, "personal/github.com/someone-else/repo");
    assert_eq!(
        harness.stub_git.calls_starting_with("clone\t"),
        vec![format!("clone\t{url}\t{}", destination.display())]
    );
}

#[test]
fn a_url_matching_a_declared_categorys_pattern_still_wins_over_the_default() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    harness
        .stub_git
        .seed_config("gig.category.work.pattern", r"^gitlab\.company\.com/");
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", "");
    harness
        .stub_git
        .seed_config("gig.category.personal.default", "true");
    let url = "https://gitlab.company.com/team/service";

    harness.cmd().args(["get", url]).assert().success();

    let destination = common::join(&root_dir, "work/gitlab.company.com/team/service");
    assert_eq!(
        harness.stub_git.calls_starting_with("clone\t"),
        vec![format!("clone\t{url}\t{}", destination.display())]
    );
}

#[test]
fn the_category_flag_still_overrides_the_declared_default() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", "");
    harness
        .stub_git
        .seed_config("gig.category.personal.default", "true");
    harness
        .stub_git
        .seed_config("gig.category.work.pattern", "");
    let url = "git@github.com:someone-else/repo.git";

    harness
        .cmd()
        .args(["get", url, "--category", "work"])
        .assert()
        .success();

    let destination = common::join(&root_dir, "work/github.com/someone-else/repo");
    assert_eq!(
        harness.stub_git.calls_starting_with("clone\t"),
        vec![format!("clone\t{url}\t{}", destination.display())]
    );
}

#[test]
fn a_default_category_with_its_own_patterns_matches_them_normally_and_still_catches_the_rest() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");
    harness
        .stub_git
        .seed_config("gig.category.personal.default", "true");

    // Matches "personal"'s own pattern.
    harness
        .cmd()
        .args(["get", "git@github.com:merikan/gig.git"])
        .assert()
        .success();
    // Matches nothing - falls back to "personal" anyway, since it's default.
    harness
        .cmd()
        .args(["get", "git@github.com:someone-else/repo.git"])
        .assert()
        .success();

    assert!(common::join(&root_dir, "personal/github.com/merikan/gig/.git").is_dir());
    assert!(common::join(&root_dir, "personal/github.com/someone-else/repo/.git").is_dir());
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
