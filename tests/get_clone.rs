mod common;

use common::GitGetTest;
use predicates::str::contains;
use std::path::Path;

/// Asserts `get` invoked exactly one clone, of `url` into `destination`, and
/// that the stub git's `.git` marker landed there.
fn assert_cloned(harness: &GitGetTest, url: &str, destination: &Path) {
    assert_eq!(
        harness.stub_git.calls_starting_with("clone\t"),
        vec![format!("clone\t{url}\t{}", destination.display())]
    );
    assert!(destination.join(".git").is_dir());
}

#[test]
fn scp_style_ssh_url_clones_into_host_owner_repo() {
    let harness = GitGetTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";

    harness.cmd().args(["get", url]).assert().success();

    assert_cloned(&harness, url, &root_dir.join("github.com/owner/repo"));
}

#[test]
fn https_url_with_nested_subgroups_clones_into_the_full_nested_path() {
    let harness = GitGetTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "https://gitlab.com/group/subgroup/repo";

    harness.cmd().args(["get", url]).assert().success();

    assert_cloned(
        &harness,
        url,
        &root_dir.join("gitlab.com/group/subgroup/repo"),
    );
}

#[test]
fn sourcehut_tilde_user_url_clones_into_the_tilde_prefixed_path() {
    let harness = GitGetTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@git.sr.ht:~user/repo";

    harness.cmd().args(["get", url]).assert().success();

    assert_cloned(&harness, url, &root_dir.join("git.sr.ht/~user/repo"));
}

#[test]
fn bare_url_without_get_keyword_behaves_like_explicit_get() {
    let harness = GitGetTest::new();
    let root_dir = harness.seed_root_dir();
    let url = "git@github.com:owner/repo.git";

    harness.cmd().arg(url).assert().success();

    assert_cloned(&harness, url, &root_dir.join("github.com/owner/repo"));
}

#[test]
fn unsupported_url_form_errors_without_attempting_a_clone() {
    let harness = GitGetTest::new();
    harness.seed_root_dir();

    harness
        .cmd()
        .args(["get", "ssh://git@github.com/owner/repo.git"])
        .assert()
        .failure()
        .stderr(contains("unsupported URL form"));

    assert!(harness.stub_git.calls_starting_with("clone\t").is_empty());
}
