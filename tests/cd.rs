#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::GigTest;
use predicates::str::contains;
use std::fs;
use std::path::Path;

/// Seeds a fake clone (a directory containing a `.git` subdirectory) at
/// `root_dir/relative_path`, bypassing `get` entirely - `cd`, like `list`,
/// only cares that a `.git` marker exists on disk.
fn seed_fake_clone(root_dir: &Path, relative_path: &str) {
    let repo_dir = common::join(root_dir, relative_path);
    fs::create_dir_all(repo_dir.join(".git")).unwrap();
}

/// `assert_cmd` runs the child with stderr as a plain pipe, never a pty, so
/// `gig cd` always hits its own "not a tty" guard before it would try to draw
/// the interactive picker - there is no interactive-selection path this test
/// harness can exercise. That guard is exactly what's under test here: it's
/// what keeps a non-interactive invocation (this test, a CI run, `gig cd`
/// piped into something) from hanging or garbling output instead of failing
/// clearly.
#[test]
fn non_interactive_stderr_is_rejected_with_a_clear_error() {
    let harness = GigTest::new();
    let root_dir = harness.seed_root_dir();
    seed_fake_clone(&root_dir, "github.com/owner/repo");

    harness
        .cmd()
        .arg("cd")
        .assert()
        .failure()
        .stderr(contains("interactive terminal"));
}

#[test]
fn empty_root_dir_errors_instead_of_offering_an_empty_picker() {
    let harness = GigTest::new();
    harness.seed_root_dir();

    harness
        .cmd()
        .arg("cd")
        .assert()
        .failure()
        .stderr(contains("no repos found"));
}

#[test]
fn cd_with_root_dir_unset_errors_with_the_command_to_run() {
    let harness = GigTest::new();

    harness
        .cmd()
        .arg("cd")
        .assert()
        .failure()
        .stderr(contains("gig config root-dir <path>"));
}
