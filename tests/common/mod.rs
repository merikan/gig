// Each integration-test binary compiles this module separately and only uses a
// subset of it, so per-binary dead-code warnings here are false positives.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use assert_cmd::Command;

/// The stub `git` binary, installed as `git` in its own bin directory so it can be
/// prepended onto `PATH` and intercept every invocation of the real `git`.
pub struct StubGit {
    bin_dir: PathBuf,
    pub log_file: PathBuf,
    pub config_file: PathBuf,
}

impl StubGit {
    pub fn install(temp_dir: &Path) -> Self {
        let bin_dir = temp_dir.join("bin");
        fs::create_dir_all(&bin_dir).expect("create stub-git bin dir");

        let stub_git_bin = assert_cmd::cargo::cargo_bin("stub-git");
        let git_path = bin_dir.join("git");
        fs::copy(&stub_git_bin, &git_path).expect("copy stub-git binary");
        set_executable(&git_path);

        Self {
            bin_dir,
            log_file: temp_dir.join("stub-git.log"),
            config_file: temp_dir.join("stub-git.config"),
        }
    }

    /// `PATH` with the stub-git directory prepended, so anything shelling out to
    /// `git` finds the stub instead of whatever real git is installed.
    pub fn path_env(&self) -> String {
        let existing = std::env::var("PATH").unwrap_or_default();
        format!("{}:{existing}", self.bin_dir.display())
    }

    /// Every call the stub recorded, oldest first, as tab-joined argv.
    pub fn calls(&self) -> Vec<String> {
        fs::read_to_string(&self.log_file)
            .unwrap_or_default()
            .lines()
            .map(String::from)
            .collect()
    }

    /// Invoke the stub directly (bypassing git-get) for harness-level tests.
    pub fn command(&self) -> StdCommand {
        let mut cmd = StdCommand::new(self.bin_dir.join("git"));
        cmd.env("GIT_GET_STUB_LOG", &self.log_file)
            .env("GIT_GET_STUB_CONFIG", &self.config_file);
        cmd
    }
}

#[cfg(unix)]
fn set_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)
        .expect("stat stub-git copy")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).expect("chmod stub-git copy");
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) {}

/// A `git-get` invocation wired to the stub-git harness with an isolated
/// HOME/working directory - no test ever touches the developer's real gitconfig.
pub struct GitGetTest {
    pub stub_git: StubGit,
    pub home_dir: PathBuf,
    #[allow(dead_code)]
    // keeps the TempDir (and its on-disk contents) alive for the test's duration
    temp_dir: tempfile::TempDir,
}

impl GitGetTest {
    pub fn new() -> Self {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let home_dir = temp_dir.path().join("home");
        fs::create_dir_all(&home_dir).expect("create home dir");
        let stub_git = StubGit::install(temp_dir.path());

        Self {
            stub_git,
            home_dir,
            temp_dir,
        }
    }

    pub fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("git-get").expect("locate git-get binary");
        cmd.env("PATH", self.stub_git.path_env())
            .env("HOME", &self.home_dir)
            .env("GIT_GET_STUB_LOG", &self.stub_git.log_file)
            .env("GIT_GET_STUB_CONFIG", &self.stub_git.config_file)
            .current_dir(&self.home_dir);
        cmd
    }
}
