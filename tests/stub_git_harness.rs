#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::StubGit;

#[test]
fn records_every_invocation() {
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());

    stub.command()
        .args(["config", "--global", "gig.root-dir", "/tmp/repos"])
        .status()
        .expect("run stub git");

    assert_eq!(
        stub.calls(),
        vec!["config\t--global\tgig.root-dir\t/tmp/repos"]
    );
}

#[test]
fn config_get_and_global_set_round_trip() {
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());

    let set_status = stub
        .command()
        .args(["config", "--global", "gig.root-dir", "/tmp/repos"])
        .status()
        .expect("run stub git set");
    assert!(set_status.success());

    let output = stub
        .command()
        .args(["config", "--get", "gig.root-dir"])
        .output()
        .expect("run stub git get");
    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "/tmp/repos");
}

#[test]
fn config_get_on_unset_key_fails() {
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());

    let status = stub
        .command()
        .args(["config", "--get", "gig.root-dir"])
        .status()
        .expect("run stub git get");
    assert!(!status.success());
}

#[test]
fn clone_creates_git_marker_directory() {
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());
    let dest = temp.path().join("dest-repo");

    let status = stub
        .command()
        .args([
            "clone",
            "git@github.com:owner/repo.git",
            dest.to_str().unwrap(),
        ])
        .status()
        .expect("run stub git clone");

    assert!(status.success());
    assert!(dest.join(".git").is_dir());
}

#[test]
fn dash_c_pull_succeeds_and_is_logged_with_the_directory() {
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());

    let status = stub
        .command()
        .args(["-C", "/some/dest", "pull"])
        .status()
        .expect("run stub git -C pull");

    assert!(status.success());
    assert_eq!(stub.calls(), vec!["-C\t/some/dest\tpull"]);
}

#[test]
fn path_lookup_resolves_to_stub_git() {
    // The other tests here invoke the stub by its full path via `StubGit::command`.
    // This is the one test proving `path_env` actually shadows the real `git` when
    // something (eventually gig itself) resolves "git" through `PATH` alone.
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());

    let status = std::process::Command::new("git")
        .env("PATH", stub.path_env())
        .env("GIG_STUB_LOG", &stub.log_file)
        .env("GIG_STUB_CONFIG", &stub.config_file)
        .args(["config", "--global", "gig.root-dir", "/tmp/repos"])
        .status()
        .expect("resolve git via PATH");

    assert!(status.success());
    assert_eq!(
        stub.calls(),
        vec!["config\t--global\tgig.root-dir\t/tmp/repos"]
    );
}

#[test]
fn get_all_returns_every_value_of_a_multi_valued_key_in_add_order() {
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());
    stub.command()
        .args([
            "config",
            "--add",
            "--global",
            "gig.category.oss.pattern",
            "a",
        ])
        .status()
        .expect("seed first value");
    stub.command()
        .args([
            "config",
            "--add",
            "--global",
            "gig.category.oss.pattern",
            "b",
        ])
        .status()
        .expect("seed second value");

    let output = stub
        .command()
        .args(["config", "--get-all", "gig.category.oss.pattern"])
        .output()
        .expect("run stub git get-all");

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "a\nb\n");
}

#[test]
fn get_all_on_an_unset_key_fails() {
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());

    let status = stub
        .command()
        .args(["config", "--get-all", "gig.category.ghost.pattern"])
        .status()
        .expect("run stub git get-all");

    assert!(!status.success());
}

#[test]
fn add_global_appends_without_disturbing_other_keys_declared_in_between() {
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());
    stub.command()
        .args([
            "config",
            "--add",
            "--global",
            "gig.category.a.pattern",
            "p1",
        ])
        .status()
        .expect("seed a p1");
    stub.command()
        .args([
            "config",
            "--add",
            "--global",
            "gig.category.b.pattern",
            "q1",
        ])
        .status()
        .expect("seed b q1");

    let status = stub
        .command()
        .args([
            "config",
            "--add",
            "--global",
            "gig.category.a.pattern",
            "p2",
        ])
        .status()
        .expect("add a p2");

    assert!(status.success());
    let output = stub
        .command()
        .args(["config", "--get-regexp", r"^gig\.category\..*\.pattern$"])
        .output()
        .expect("enumerate");
    // "a"'s two values stay grouped together, ahead of "b" - matching real
    // git's behavior of keeping a key's values at its first-declared position.
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "gig.category.a.pattern p1\ngig.category.a.pattern p2\ngig.category.b.pattern q1\n"
    );
}

#[test]
fn replace_all_global_clears_prior_values_and_keeps_the_keys_original_position() {
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());
    stub.command()
        .args([
            "config",
            "--add",
            "--global",
            "gig.category.a.pattern",
            "p1",
        ])
        .status()
        .expect("seed a p1");
    stub.command()
        .args([
            "config",
            "--add",
            "--global",
            "gig.category.b.pattern",
            "q1",
        ])
        .status()
        .expect("seed b q1");
    stub.command()
        .args([
            "config",
            "--add",
            "--global",
            "gig.category.a.pattern",
            "p2",
        ])
        .status()
        .expect("seed a p2");

    let status = stub
        .command()
        .args([
            "config",
            "--replace-all",
            "--global",
            "gig.category.a.pattern",
            "p3",
        ])
        .status()
        .expect("replace-all a");

    assert!(status.success());
    let output = stub
        .command()
        .args(["config", "--get-regexp", r"^gig\.category\..*\.pattern$"])
        .output()
        .expect("enumerate");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "gig.category.a.pattern p3\ngig.category.b.pattern q1\n"
    );
}

#[test]
fn unset_global_removes_the_key_and_reports_success() {
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());
    stub.command()
        .args([
            "config",
            "--global",
            "gig.category.personal.default",
            "true",
        ])
        .status()
        .expect("seed default flag");

    let status = stub
        .command()
        .args([
            "config",
            "--unset",
            "--global",
            "gig.category.personal.default",
        ])
        .status()
        .expect("run stub git unset");

    assert!(status.success());
    let output = stub
        .command()
        .args(["config", "--get", "gig.category.personal.default"])
        .output()
        .expect("run stub git get");
    assert!(!output.status.success());
}

#[test]
fn unset_global_on_a_key_that_was_never_set_fails_with_exit_code_5() {
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());

    let status = stub
        .command()
        .args([
            "config",
            "--unset",
            "--global",
            "gig.category.ghost.default",
        ])
        .status()
        .expect("run stub git unset");

    assert_eq!(status.code(), Some(5));
}

#[test]
fn configurable_exit_code_and_output() {
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());

    let output = stub
        .command()
        .env("GIG_STUB_EXIT_CODE", "17")
        .env("GIG_STUB_STDERR", "boom")
        .args(["clone", "url", "dest"])
        .output()
        .expect("run stub git");

    assert_eq!(output.status.code(), Some(17));
    assert_eq!(String::from_utf8_lossy(&output.stderr), "boom");
}
