#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{GigTest, StubGit};
use predicates::str::contains;

/// No `git config` call that actually mutates the store (`--global`,
/// `--add --global`, or `--replace-all --global`) was made. Reads (`--get`,
/// `--get-all`, `--get-regexp`) are fine - some error paths (e.g. "category
/// isn't declared") only find that out by reading first.
fn assert_nothing_written(stub_git: &StubGit) {
    assert!(
        stub_git
            .calls_starting_with("config\t--global\t")
            .is_empty()
    );
    assert!(
        stub_git
            .calls_starting_with("config\t--add\t--global\t")
            .is_empty()
    );
    assert!(
        stub_git
            .calls_starting_with("config\t--replace-all\t--global\t")
            .is_empty()
    );
}

#[test]
fn setting_a_category_invokes_git_config_global() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "category", "personal", r"^github\.com/merikan/"])
        .assert()
        .success();

    // A brand-new single-pattern declaration still uses the same plain
    // `--global` set single-pattern categories have always used - filtered
    // from the preceding `--get-all` read the destructive-replace-warning
    // check now always makes first.
    assert_eq!(
        harness.stub_git.calls_starting_with("config\t--global\t"),
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
        harness.stub_git.calls_starting_with("config\t--global\t"),
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

#[test]
fn adding_a_pattern_appends_to_an_already_declared_categorys_list() {
    let harness = GigTest::new();
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

    assert_eq!(
        harness
            .stub_git
            .calls_starting_with("config\t--add\t--global\t"),
        vec![format!(
            "config\t--add\t--global\tgig.category.oss.pattern\t{}",
            r"^github\.com/tokio-rs/"
        )]
    );
    harness
        .cmd()
        .args(["config", "category", "oss"])
        .assert()
        .success()
        .stdout(format!(
            "{}\n{}\n",
            r"^github\.com/rust-lang/", r"^github\.com/tokio-rs/"
        ));
}

#[test]
fn adding_to_an_undeclared_category_errors_and_writes_nothing() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "category", "ghost", "some-pattern", "--add"])
        .assert()
        .failure()
        .stderr(contains("ghost"))
        .stderr(contains("not declared"));

    assert_nothing_written(&harness.stub_git);
}

#[test]
fn adding_an_exact_duplicate_pattern_is_a_silent_no_op() {
    let harness = GigTest::new();
    harness
        .stub_git
        .seed_config("gig.category.oss.pattern", r"^github\.com/rust-lang/");

    harness
        .cmd()
        .args([
            "config",
            "category",
            "oss",
            r"^github\.com/rust-lang/",
            "--add",
        ])
        .assert()
        .success();

    assert!(
        harness
            .stub_git
            .calls_starting_with("config\t--add\t--global\t")
            .is_empty()
    );
    harness
        .cmd()
        .args(["config", "category", "oss"])
        .assert()
        .success()
        .stdout(format!("{}\n", r"^github\.com/rust-lang/"));
}

#[test]
fn replacing_a_populated_categorys_patterns_prints_the_destructive_replace_warning() {
    let harness = GigTest::new();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");

    harness
        .cmd()
        .args(["config", "category", "personal", r"^gitlab\.com/merikan/"])
        .assert()
        .success()
        .stderr(contains("personal"))
        .stderr(contains("warning"));

    harness
        .cmd()
        .args(["config", "category", "personal"])
        .assert()
        .success()
        .stdout(format!("{}\n", r"^gitlab\.com/merikan/"));
}

#[test]
fn flag_only_replace_of_a_populated_category_prints_the_destructive_replace_warning() {
    let harness = GigTest::new();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");

    harness
        .cmd()
        .args(["config", "category", "personal", "--flag-only"])
        .assert()
        .success()
        .stderr(contains("personal"))
        .stderr(contains("warning"));

    harness
        .cmd()
        .args(["config", "category"])
        .assert()
        .success()
        .stdout("personal\t\n");
}

#[test]
fn replacing_a_brand_new_categorys_patterns_prints_no_warning() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "category", "personal", r"^github\.com/merikan/"])
        .assert()
        .success()
        .stderr(predicates::str::is_empty());
}

#[test]
fn replacing_a_category_that_already_had_zero_patterns_prints_no_warning() {
    let harness = GigTest::new();
    // "oneoff" is flag-only: zero real patterns, even though it's already
    // declared (one empty-string storage entry).
    harness
        .stub_git
        .seed_config("gig.category.oneoff.pattern", "");

    harness
        .cmd()
        .args(["config", "category", "oneoff", r"^github\.com/merikan/"])
        .assert()
        .success()
        .stderr(predicates::str::is_empty());
}

#[test]
fn viewing_a_multi_pattern_category_prints_one_pattern_per_line_in_order() {
    let harness = GigTest::new();
    harness
        .stub_git
        .seed_config("gig.category.oss.pattern", r"^github\.com/rust-lang/");
    harness
        .stub_git
        .seed_config_add("gig.category.oss.pattern", r"^github\.com/tokio-rs/");

    harness
        .cmd()
        .args(["config", "category", "oss"])
        .assert()
        .success()
        .stdout(format!(
            "{}\n{}\n",
            r"^github\.com/rust-lang/", r"^github\.com/tokio-rs/"
        ));
}

#[test]
fn listing_all_prints_one_line_per_pattern_repeating_a_multi_pattern_categorys_name() {
    let harness = GigTest::new();
    harness
        .stub_git
        .seed_config("gig.category.oss.pattern", r"^github\.com/rust-lang/");
    harness
        .stub_git
        .seed_config_add("gig.category.oss.pattern", r"^github\.com/tokio-rs/");
    harness
        .stub_git
        .seed_config("gig.category.oneoff.pattern", "");

    harness
        .cmd()
        .args(["config", "category"])
        .assert()
        .success()
        .stdout(format!(
            "oss\t{}\noss\t{}\noneoff\t\n",
            r"^github\.com/rust-lang/", r"^github\.com/tokio-rs/"
        ));
}

#[test]
fn add_combined_with_flag_only_errors_and_writes_nothing() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args([
            "config",
            "category",
            "personal",
            "some-pattern",
            "--add",
            "--flag-only",
        ])
        .assert()
        .failure()
        .stderr(contains("mutually exclusive"));

    assert!(harness.stub_git.calls().is_empty());
}

#[test]
fn add_with_no_pattern_argument_errors_and_writes_nothing() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "category", "personal", "--add"])
        .assert()
        .failure()
        .stderr(contains("at least one pattern"));

    assert!(harness.stub_git.calls().is_empty());
}
