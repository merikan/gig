// Each integration-test binary compiles this module separately and only uses a
// subset of it, so per-binary dead-code warnings here are false positives.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use assert_cmd::Command;

/// Joins each `/`-delimited segment of `relative` onto `base` individually.
/// Tests build expected destination paths from `/`-joined literals mirroring
/// gig's own `host/owner/repo` on-disk layout; a single
/// `base.join("host/owner/repo")` call embeds the literal `/` bytes into one
/// path component instead of splitting on them. That's harmless for actual
/// filesystem calls (Windows accepts `/` interchangeably with `\`), but
/// produces a string that differs from gig's own output when compared via
/// `.display()`: gig always joins one path segment at a time (see
/// `src/destination.rs`), so it always prints pure native separators, never
/// an embedded `/` on Windows.
pub fn join(base: &Path, relative: &str) -> PathBuf {
    relative
        .split('/')
        .fold(base.to_path_buf(), |acc, segment| acc.join(segment))
}

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
        let git_path = bin_dir.join(GIT_FILENAME);
        link_stub_git(&stub_git_bin, &git_path);

        Self {
            bin_dir,
            log_file: temp_dir.join("stub-git.log"),
            config_file: temp_dir.join("stub-git.config"),
        }
    }

    /// `PATH` with the stub-git directory prepended, so anything shelling out to
    /// `git` finds the stub instead of whatever real git is installed.
    ///
    /// Built with `std::env::join_paths` rather than a hardcoded `:` - on
    /// Windows the separator is `;`, and joining with the wrong one merges
    /// our stub dir into the following PATH entry into one invalid,
    /// nonexistent directory, silently dropping the stub from the search
    /// entirely and letting `Command::new("git")` fall through to whatever
    /// real git is further down PATH.
    pub fn path_env(&self) -> String {
        let existing = std::env::var_os("PATH").unwrap_or_default();
        let entries = std::iter::once(self.bin_dir.clone()).chain(std::env::split_paths(&existing));
        std::env::join_paths(entries)
            .expect("join PATH entries")
            .to_str()
            .expect("PATH is valid UTF-8")
            .to_string()
    }

    /// Every call the stub recorded, oldest first, as tab-joined argv.
    pub fn calls(&self) -> Vec<String> {
        fs::read_to_string(&self.log_file)
            .unwrap_or_default()
            .lines()
            .map(String::from)
            .collect()
    }

    /// Just the calls whose tab-joined argv starts with `prefix` (e.g. `"clone\t"`
    /// or `"-C\t"`) - lets a test isolate the operation it's asserting on from
    /// incidental setup noise like the `config` calls made while seeding
    /// `root-dir` or by `get` reading it back.
    pub fn calls_starting_with(&self, prefix: &str) -> Vec<String> {
        self.calls()
            .into_iter()
            .filter(|call| call.starts_with(prefix))
            .collect()
    }

    /// Invoke the stub directly (bypassing gig) for harness-level tests.
    pub fn command(&self) -> StdCommand {
        let mut cmd = StdCommand::new(self.bin_dir.join(GIT_FILENAME));
        cmd.env("GIG_STUB_LOG", &self.log_file)
            .env("GIG_STUB_CONFIG", &self.config_file);
        cmd
    }

    /// Pre-populates a config key/value directly in the stub's store, bypassing
    /// gig - for tests that need a value to already be "set" before the
    /// invocation under test runs.
    pub fn seed_config(&self, key: &str, value: &str) {
        let status = self
            .command()
            .args(["config", "--global", key, value])
            .status()
            .expect("seed stub git config");
        assert!(status.success(), "failed to seed stub git config");
    }

    /// Appends another value to a (possibly already-seeded) key directly in
    /// the stub's store, bypassing gig - for seeding a multi-valued key (e.g.
    /// a category declared with more than one pattern) before the invocation
    /// under test runs.
    pub fn seed_config_add(&self, key: &str, value: &str) {
        let status = self
            .command()
            .args(["config", "--add", "--global", key, value])
            .status()
            .expect("seed stub git config (add)");
        assert!(status.success(), "failed to seed stub git config (add)");
    }
}

/// The stub binary's filename within its bin dir. `gig` itself finds `git`
/// via a bare `Command::new("git")`, resolved by the OS through `PATH` - on
/// Windows that resolution only ever tries appending `.exe` (not the full
/// `PATHEXT` list, and only when the name has no path component), so the
/// file must actually be named `git.exe` there or it's silently skipped in
/// favor of any real git further down `PATH`. Unix has no such requirement.
#[cfg(windows)]
const GIT_FILENAME: &str = "git.exe";
#[cfg(not(windows))]
const GIT_FILENAME: &str = "git";

/// Links `dst` to the already-built, already-executable `src` binary.
///
/// On Unix this symlinks rather than copies: copying bytes and then exec'ing
/// the destination immediately can transiently fail with ETXTBSY ("Text file
/// busy") - observed on GitHub Actions' overlayfs - because the fresh write's
/// close hasn't fully settled before the exec. Symlinking to the pre-existing,
/// already-stable binary sidesteps that race instead of retrying around it.
#[cfg(unix)]
fn link_stub_git(src: &Path, dst: &Path) {
    std::os::unix::fs::symlink(src, dst).expect("symlink stub-git binary");
}

#[cfg(not(unix))]
fn link_stub_git(src: &Path, dst: &Path) {
    fs::copy(src, dst).expect("copy stub-git binary");
}

/// A `gig` invocation wired to the stub-git harness with an isolated
/// HOME/working directory - no test ever touches the developer's real gitconfig.
pub struct GigTest {
    pub stub_git: StubGit,
    pub home_dir: PathBuf,
    #[allow(dead_code)]
    // keeps the TempDir (and its on-disk contents) alive for the test's duration
    temp_dir: tempfile::TempDir,
}

impl GigTest {
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
        let mut cmd = Command::cargo_bin("gig").expect("locate gig binary");
        cmd.env("PATH", self.stub_git.path_env())
            .env("HOME", &self.home_dir)
            .env("GIG_STUB_LOG", &self.stub_git.log_file)
            .env("GIG_STUB_CONFIG", &self.stub_git.config_file)
            .current_dir(&self.home_dir);
        cmd
    }

    /// Seeds `gig.root-dir` to `<home>/root` directly in the stub's store,
    /// bypassing `gig config root-dir`, and returns that path - the common
    /// precondition every `get`-exercising test needs before it can run.
    pub fn seed_root_dir(&self) -> PathBuf {
        let root_dir = self.home_dir.join("root");
        self.stub_git
            .seed_config("gig.root-dir", root_dir.to_str().unwrap());
        root_dir
    }
}
