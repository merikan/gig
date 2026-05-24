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

#[test]
fn declaring_a_category_whose_name_contains_a_slash_errors_and_writes_nothing() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args([
            "config",
            "category",
            "../etc/personal",
            r"^github\.com/merikan/",
        ])
        .assert()
        .failure()
        .stderr(contains("../etc/personal"));

    assert!(harness.stub_git.calls().is_empty());
}

#[test]
fn declaring_a_category_whose_name_contains_dot_dot_errors_and_writes_nothing() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "category", "foo..bar", r"^github\.com/merikan/"])
        .assert()
        .failure()
        .stderr(contains("foo..bar"));

    assert!(harness.stub_git.calls().is_empty());
}

#[test]
fn declaring_a_category_whose_name_contains_a_backslash_errors_and_writes_nothing() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "category", r"foo\bar", r"^github\.com/merikan/"])
        .assert()
        .failure()
        .stderr(contains(r"foo\bar"));

    assert!(harness.stub_git.calls().is_empty());
}

#[test]
fn flag_only_with_a_path_unsafe_name_errors_the_same_way_and_writes_nothing() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "category", "../etc", "--flag-only"])
        .assert()
        .failure()
        .stderr(contains("../etc"));

    assert!(harness.stub_git.calls().is_empty());
}

#[test]
fn add_with_a_path_unsafe_name_errors_and_writes_nothing() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args([
            "config",
            "category",
            "../etc",
            r"^github\.com/merikan/",
            "--add",
        ])
        .assert()
        .failure()
        .stderr(contains("../etc"));

    assert!(harness.stub_git.calls().is_empty());
}

#[test]
fn viewing_an_already_declared_category_whose_name_would_now_be_rejected_still_works() {
    // A name only ever becomes unsafe by being written directly to git config
    // outside gig (or before this guard existed) - reading it back must still
    // work, since the guard only applies to declaring/updating a category.
    let harness = GigTest::new();
    harness
        .stub_git
        .seed_config("gig.category.../etc.pattern", r"^github\.com/merikan/");

    harness
        .cmd()
        .args(["config", "category", "../etc"])
        .assert()
        .success()
        .stdout(format!("{}\n", r"^github\.com/merikan/"));
}

#[test]
fn default_alone_on_a_brand_new_name_declares_it_flag_only_and_marks_it_default() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "category", "personal", "--default"])
        .assert()
        .success()
        .stderr(predicates::str::is_empty());

    harness
        .cmd()
        .args(["config", "category"])
        .assert()
        .success()
        .stdout("personal\t\tdefault\n");
}

#[test]
fn default_on_an_already_declared_category_leaves_its_patterns_untouched() {
    let harness = GigTest::new();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");

    harness
        .cmd()
        .args(["config", "category", "personal", "--default"])
        .assert()
        .success();

    harness
        .cmd()
        .args(["config", "category", "personal"])
        .assert()
        .success()
        .stdout(format!("{}\tdefault\n", r"^github\.com/merikan/"));
}

#[test]
fn marking_a_new_category_default_auto_demotes_the_previous_one_with_a_warning() {
    let harness = GigTest::new();
    harness
        .stub_git
        .seed_config("gig.category.work.pattern", "");
    harness
        .stub_git
        .seed_config("gig.category.work.default", "true");
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", "");

    harness
        .cmd()
        .args(["config", "category", "personal", "--default"])
        .assert()
        .success()
        .stderr(contains("work"))
        .stderr(contains("personal"));

    harness
        .cmd()
        .args(["config", "category"])
        .assert()
        .success()
        .stdout("work\t\npersonal\t\tdefault\n");
}

#[test]
fn re_marking_the_current_default_is_a_no_op_and_prints_no_warning() {
    let harness = GigTest::new();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", "");
    harness
        .stub_git
        .seed_config("gig.category.personal.default", "true");

    harness
        .cmd()
        .args(["config", "category", "personal", "--default"])
        .assert()
        .success()
        .stderr(predicates::str::is_empty());
}

#[test]
fn no_default_clears_the_flag_without_touching_patterns() {
    let harness = GigTest::new();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");
    harness
        .stub_git
        .seed_config("gig.category.personal.default", "true");

    harness
        .cmd()
        .args(["config", "category", "personal", "--no-default"])
        .assert()
        .success();

    harness
        .cmd()
        .args(["config", "category", "personal"])
        .assert()
        .success()
        .stdout(format!("{}\n", r"^github\.com/merikan/"));
}

#[test]
fn no_default_on_a_category_that_was_never_default_is_a_silent_no_op() {
    let harness = GigTest::new();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");

    harness
        .cmd()
        .args(["config", "category", "personal", "--no-default"])
        .assert()
        .success()
        .stderr(predicates::str::is_empty());
}

#[test]
fn no_default_on_a_completely_undeclared_name_is_a_silent_no_op() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "category", "ghost", "--no-default"])
        .assert()
        .success()
        .stderr(predicates::str::is_empty());
}

#[test]
fn default_and_no_default_together_error() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args([
            "config",
            "category",
            "personal",
            "--default",
            "--no-default",
        ])
        .assert()
        .failure()
        .stderr(contains("mutually exclusive"));

    assert_nothing_written(&harness.stub_git);
}

#[test]
fn default_combined_with_add_errors() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args([
            "config",
            "category",
            "oss",
            r"^github\.com/tokio-rs/",
            "--add",
            "--default",
        ])
        .assert()
        .failure()
        .stderr(contains("--add"));

    assert!(harness.stub_git.calls().is_empty());
}

#[test]
fn no_default_combined_with_add_errors() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args([
            "config",
            "category",
            "oss",
            r"^github\.com/tokio-rs/",
            "--add",
            "--no-default",
        ])
        .assert()
        .failure()
        .stderr(contains("--add"));

    assert!(harness.stub_git.calls().is_empty());
}

#[test]
fn default_combines_with_flag_only_in_one_call() {
    let harness = GigTest::new();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");

    harness
        .cmd()
        .args(["config", "category", "personal", "--flag-only", "--default"])
        .assert()
        .success()
        .stderr(contains("warning"));

    harness
        .cmd()
        .args(["config", "category", "personal"])
        .assert()
        .success()
        .stdout("\tdefault\n");
}

#[test]
fn default_combines_with_a_replacing_pattern_list_in_one_call() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args([
            "config",
            "category",
            "personal",
            r"^gitlab\.com/",
            "--default",
        ])
        .assert()
        .success();

    harness
        .cmd()
        .args(["config", "category", "personal"])
        .assert()
        .success()
        .stdout(format!("{}\tdefault\n", r"^gitlab\.com/"));
}

#[test]
fn default_with_a_path_unsafe_name_errors_the_same_way_and_writes_nothing() {
    let harness = GigTest::new();

    harness
        .cmd()
        .args(["config", "category", "../etc", "--default"])
        .assert()
        .failure()
        .stderr(contains("../etc"));

    assert!(harness.stub_git.calls().is_empty());
}

#[test]
fn listing_shows_no_default_column_when_no_category_is_default() {
    let harness = GigTest::new();
    harness
        .stub_git
        .seed_config("gig.category.personal.pattern", r"^github\.com/merikan/");

    harness
        .cmd()
        .args(["config", "category"])
        .assert()
        .success()
        .stdout(format!("personal\t{}\n", r"^github\.com/merikan/"));
}
