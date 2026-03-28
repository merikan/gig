mod common;

use common::StubGit;

#[test]
fn records_every_invocation() {
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());

    stub.command()
        .args(["config", "--global", "git-get.root-dir", "/tmp/repos"])
        .status()
        .expect("run stub git");

    assert_eq!(
        stub.calls(),
        vec!["config\t--global\tgit-get.root-dir\t/tmp/repos"]
    );
}

#[test]
fn config_get_and_global_set_round_trip() {
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());

    let set_status = stub
        .command()
        .args(["config", "--global", "git-get.root-dir", "/tmp/repos"])
        .status()
        .expect("run stub git set");
    assert!(set_status.success());

    let output = stub
        .command()
        .args(["config", "--get", "git-get.root-dir"])
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
        .args(["config", "--get", "git-get.root-dir"])
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
    // something (eventually git-get itself) resolves "git" through `PATH` alone.
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());

    let status = std::process::Command::new("git")
        .env("PATH", stub.path_env())
        .env("GIT_GET_STUB_LOG", &stub.log_file)
        .env("GIT_GET_STUB_CONFIG", &stub.config_file)
        .args(["config", "--global", "git-get.root-dir", "/tmp/repos"])
        .status()
        .expect("resolve git via PATH");

    assert!(status.success());
    assert_eq!(
        stub.calls(),
        vec!["config\t--global\tgit-get.root-dir\t/tmp/repos"]
    );
}

#[test]
fn configurable_exit_code_and_output() {
    let temp = tempfile::tempdir().unwrap();
    let stub = StubGit::install(temp.path());

    let output = stub
        .command()
        .env("GIT_GET_STUB_EXIT_CODE", "17")
        .env("GIT_GET_STUB_STDERR", "boom")
        .args(["clone", "url", "dest"])
        .output()
        .expect("run stub git");

    assert_eq!(output.status.code(), Some(17));
    assert_eq!(String::from_utf8_lossy(&output.stderr), "boom");
}
