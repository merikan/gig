#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::GigTest;
use predicates::str::contains;
use std::fs;

#[test]
fn a_repo_cloned_at_the_default_path_before_a_matching_category_existed_is_found_there_not_duplicated()
 {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:merikan/gig.git";

    // Cloned before any category rule existed - lands at the default path.
    harness.cmd().args(["get", url]).assert().success();
    let default_destination = root_dir.join("github.com/merikan/gig");
    assert_eq!(harness.stub_git.calls_starting_with("clone\t").len(), 1);

    // A category rule is declared after the fact that would now match this URL.
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");

    // Re-running `get` finds the existing clone at the default path...
    harness
        .cmd()
        .args(["get", url])
        .assert()
        .success()
        .stdout(contains(format!(
            "Already cloned at {}",
            default_destination.display()
        )));

    // ...no duplicate clone under the new category path.
    assert_eq!(harness.stub_git.calls_starting_with("clone\t").len(), 1);
    assert!(!root_dir.join("personal/github.com/merikan/gig").exists());

    // --pull pulls at the original (default) location, not the category path.
    harness
        .cmd()
        .args(["get", url, "--pull"])
        .assert()
        .success();
    assert_eq!(
        harness.stub_git.calls_starting_with("-C\t"),
        vec![format!("-C\t{}\tpull", default_destination.display())]
    );
}

#[test]
fn a_repo_cloned_at_the_default_path_is_still_found_there_after_a_category_is_declared_then_removed_or_reordered()
 {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:merikan/gig.git";
    let default_destination = root_dir.join("github.com/merikan/gig");

    // Cloned before any category rule existed.
    harness.cmd().args(["get", url]).assert().success();
    assert_eq!(harness.stub_git.calls_starting_with("clone\t").len(), 1);

    // A category is declared that would now match this URL, then removed
    // again - editing the stub's config store directly (rather than through
    // `gig config`, which has no "unset") to drop the
    // `gig.category.personal.pattern` line entirely.
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");
    let store = fs::read_to_string(&harness.stub_git.config_file).unwrap();
    let without_personal_category = store
        .lines()
        .filter(|line| !line.starts_with("gig.category.personal.pattern="))
        .fold(String::new(), |mut acc, line| {
            acc.push_str(line);
            acc.push('\n');
            acc
        });
    fs::write(&harness.stub_git.config_file, without_personal_category).unwrap();

    // A different, unrelated category is declared ahead of where "personal"
    // would have sorted - simulating reordering churn in the category config.
    harness
        .stub_git
        .seed_config("gig.category.work.pattern", r"^gitlab\.company\.com/");

    // Through all of that churn, the repo was never anywhere but the default
    // path - re-running `get` must keep finding it there, never duplicating.
    harness
        .cmd()
        .args(["get", url])
        .assert()
        .success()
        .stdout(contains(format!(
            "Already cloned at {}",
            default_destination.display()
        )));

    assert_eq!(harness.stub_git.calls_starting_with("clone\t").len(), 1);
    assert!(!root_dir.join("personal/github.com/merikan/gig").exists());
}

#[test]
fn a_genuinely_new_url_with_no_existing_clone_anywhere_lands_at_the_resolved_path() {
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
}

#[test]
fn an_invalid_regex_in_an_unrelated_category_still_aborts_up_front_even_when_the_repo_is_already_cloned()
 {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:merikan/gig.git";

    // Already cloned at the default path before "broken" was ever declared.
    harness.cmd().args(["get", url]).assert().success();
    assert_eq!(harness.stub_git.calls_starting_with("clone\t").len(), 1);

    // "broken"'s pattern is invalid and wouldn't have matched this URL even
    // if it did compile - the existing-clone search finding this repo
    // elsewhere must not let that invalid regex go uncaught.
    harness
        .stub_git
        .seed_config("gig.category.broken.pattern", "(");

    harness
        .cmd()
        .args(["get", url])
        .assert()
        .failure()
        .stderr(contains("broken"));

    // No pull was attempted either - validation aborts before any git-ops call.
    assert!(harness.stub_git.calls_starting_with("-C\t").is_empty());
    assert!(!root_dir.join("broken/github.com/merikan/gig").exists());
}

#[test]
fn an_undeclared_category_flag_still_errors_even_when_the_repo_is_already_cloned_elsewhere() {
    let harness = GigTest::new();
    harness.seed_root_dir();
    let url = "git@github.com:merikan/gig.git";

    // Already cloned at the default path.
    harness.cmd().args(["get", url]).assert().success();
    assert_eq!(harness.stub_git.calls_starting_with("clone\t").len(), 1);

    // `--category nope` was never declared - must still error, not silently
    // fall through to the existing clone found at the default path.
    harness
        .cmd()
        .args(["get", url, "--category", "nope"])
        .assert()
        .failure()
        .stderr(contains("'nope' is not declared"));

    assert!(harness.stub_git.calls_starting_with("-C\t").is_empty());
}
