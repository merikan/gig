#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::GigTest;
use predicates::str::contains;

#[test]
fn setting_a_category_invokes_git_config_global() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "category", "personal", r"^github\.com/merikan/"])
        .assert()
        .success();

    assert_eq!(
        harness.stub_git.calls(),
        vec![format!(
            "config\t--global\tgig.category.personal.pattern\t{}",
            r"^github\.com/merikan/"
        )]
    );
}

#[test]
fn viewing_a_category_reads_back_the_pattern_that_was_set() {
    let harness = GigTest::new();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");

    harness
        .cmd()
        .args(["config", "category", "personal"])
        .assert()
        .success()
        .stdout(format!("{}\n", r"^github\.com/merikan/"));
}

#[test]
fn flag_only_declares_an_empty_pattern_category_that_appears_in_enumeration() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "category", "oneoff", "--flag-only"])
        .assert()
        .success();

    assert_eq!(
        harness.stub_git.calls(),
        vec!["config\t--global\tgig.category.oneoff.pattern\t"]
    );

    harness
        .cmd()
        .args(["config", "category"])
        .assert()
        .success()
        .stdout("oneoff\t\n");
}

#[test]
fn viewing_an_undeclared_category_errors_clearly() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "category", "ghost"])
        .assert()
        .failure()
        .stderr(contains("ghost"))
        .stderr(contains("not declared"));
}

#[test]
fn pattern_and_flag_only_together_error() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args([
            "config",
            "category",
            "personal",
            "some-pattern",
            "--flag-only",
        ])
        .assert()
        .failure()
        .stderr(contains("mutually exclusive"));

    // Neither form should have reached the stub - both are rejected up front.
    assert!(harness.stub_git.calls().is_empty());
}

#[test]
fn listing_with_no_name_lists_every_declared_category_tab_separated_in_declaration_order() {
    let harness = GigTest::new();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");
    harness
        .stub_git
        .seed_config("gig.category.oneoff.pattern", "");

    harness
        .cmd()
        .args(["config", "category"])
        .assert()
        .success()
        .stdout(format!(
            "personal\t{}\noneoff\t\n",
            r"^github\.com/merikan/"
        ));
}
