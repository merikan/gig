//! A stand-in for the real `git` binary, injected earlier on `PATH` in acceptance
//! tests so no test ever touches a developer's real gitconfig or network.
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    if let Ok(log_path) = env::var("GIG_STUB_LOG") {
        log_call(&log_path, &args);
    }

    if let Some(forced_exit_code) = env::var("GIG_STUB_EXIT_CODE")
        .ok()
        .and_then(|v| v.parse::<u8>().ok())
        && forced_failure_applies(&args)
    {
        if let Ok(stdout) = env::var("GIG_STUB_STDOUT") {
            print!("{stdout}");
        }
        if let Ok(stderr) = env::var("GIG_STUB_STDERR") {
            eprint!("{stderr}");
        }
        return ExitCode::from(forced_exit_code);
    }

    match args.first().map(String::as_str) {
        Some("config") => handle_config(args.get(1..).unwrap_or_default()),
        Some("clone") => handle_clone(args.get(1..).unwrap_or_default()),
        Some("-C") => handle_dash_c(args.get(1..).unwrap_or_default()),
        _ => ExitCode::FAILURE,
    }
}

/// Whether `GIG_STUB_EXIT_CODE` should apply to this invocation.
/// `GIG_STUB_FAIL_ON`, if set, scopes the forced failure to invocations
/// whose first argument matches it (e.g. `"clone"`), so a test can force just
/// the clone/pull call to fail without also failing the `config --get`
/// call `get` makes first to read `root-dir`. Unset means "every call" -
/// the original, unscoped behavior most tests still rely on.
fn forced_failure_applies(args: &[String]) -> bool {
    env::var("GIG_STUB_FAIL_ON").map_or(true, |target| {
        args.first().map(String::as_str) == Some(target.as_str())
    })
}

/// `git -C <dir> pull`, as issued by `git_ops::pull` to run `pull` against a
/// specific destination rather than the current working directory. Prints a
/// canned line to stdout, standing in for git's own pull output, so tests can
/// assert it passes through gig's inherited-stdio invocation untouched.
fn handle_dash_c(args: &[String]) -> ExitCode {
    match args {
        [_dir, cmd] if cmd == "pull" => {
            println!("stub-git: already up to date");
            ExitCode::SUCCESS
        }
        _ => ExitCode::FAILURE,
    }
}

fn log_call(log_path: &str, args: &[String]) {
    let line = args.join("\t");
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    {
        let _ = writeln!(file, "{line}");
    }
}

fn config_store_path() -> Option<PathBuf> {
    env::var_os("GIG_STUB_CONFIG").map(PathBuf::from)
}

fn read_store() -> Vec<(String, String)> {
    let Some(path) = config_store_path() else {
        return Vec::new();
    };
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };
    contents
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn write_store(entries: &[(String, String)]) {
    let Some(path) = config_store_path() else {
        return;
    };
    let contents = entries.iter().fold(String::new(), |mut acc, (k, v)| {
        let _ = writeln!(acc, "{k}={v}");
        acc
    });
    let _ = fs::write(path, contents);
}

/// Handles the `git config` forms gig uses today: `--get <key>`,
/// `--get-all <key>`, `--global <key> <value>`, `--add --global <key>
/// <value>`, `--replace-all --global <key> <value>`, and `--get-regexp
/// <pattern>`.
fn handle_config(args: &[String]) -> ExitCode {
    match args {
        [flag, key] if flag == "--get" => {
            let store = read_store();
            match store.iter().find(|(k, _)| k == key) {
                Some((_, value)) => {
                    println!("{value}");
                    ExitCode::SUCCESS
                }
                None => ExitCode::FAILURE,
            }
        }
        [flag, key] if flag == "--get-all" => handle_get_all(key),
        [flag, key, value] if flag == "--global" => {
            let mut store = read_store();
            match store.iter_mut().find(|(k, _)| k == key) {
                Some(entry) => entry.1.clone_from(value),
                None => store.push((key.clone(), value.clone())),
            }
            write_store(&store);
            ExitCode::SUCCESS
        }
        [flag1, flag2, key, value] if flag1 == "--add" && flag2 == "--global" => {
            handle_add(key, value)
        }
        [flag1, flag2, key, value] if flag1 == "--replace-all" && flag2 == "--global" => {
            handle_replace_all(key, value)
        }
        [flag, pattern] if flag == "--get-regexp" => handle_get_regexp(pattern),
        _ => ExitCode::FAILURE,
    }
}

/// `git config --get-all <key>` - every value of `key`, one per line, in
/// store (declaration/add) order. Fails closed (exit 1, no output) when
/// `key` has no values at all, matching real git's "no such key" signal.
fn handle_get_all(key: &str) -> ExitCode {
    let store = read_store();
    let matches: Vec<_> = store.iter().filter(|(k, _)| k == key).collect();
    if matches.is_empty() {
        return ExitCode::FAILURE;
    }
    for (_, value) in matches {
        println!("{value}");
    }
    ExitCode::SUCCESS
}

/// `git config --add --global <key> <value>` - appends a new entry for
/// `key`, grouped immediately after its last existing entry (or at the end
/// of the store if `key` has none yet), mirroring how real git keeps all of
/// a key's values together at its first-declared position.
fn handle_add(key: &str, value: &str) -> ExitCode {
    let mut store = read_store();
    let insert_at = store
        .iter()
        .rposition(|(k, _)| k == key)
        .map_or(store.len(), |last| last.saturating_add(1));
    store.insert(insert_at, (key.to_string(), value.to_string()));
    write_store(&store);
    ExitCode::SUCCESS
}

/// `git config --replace-all --global <key> <value>` - drops every existing
/// entry for `key` and writes a single new one in their place, at the
/// position of `key`'s first prior entry (or at the end if it had none) -
/// mirroring real git, which never moves a key to a different position in
/// the file just because its value(s) were replaced.
fn handle_replace_all(key: &str, value: &str) -> ExitCode {
    let mut store = read_store();
    let insert_at = store.iter().position(|(k, _)| k == key);
    store.retain(|(k, _)| k != key);
    store.insert(
        insert_at.unwrap_or(store.len()),
        (key.to_string(), value.to_string()),
    );
    write_store(&store);
    ExitCode::SUCCESS
}

/// `git config --get-regexp <pattern>`. gig only ever asks for the one fixed
/// category-enumeration pattern (`^gig\.category\..*\.pattern$`), so rather
/// than embedding a real regex engine in the stub, this matches that pattern's
/// shape directly (`gig.category.<name>.pattern`) and fails closed - the same
/// unknown-form-fails contract every other unrecognized `git` invocation gets
/// here - for anything else. Matches print as `<key> <value>`, one per line in
/// store (declaration) order, exactly like real git - including the trailing
/// space before the newline real git emits for an empty value.
fn handle_get_regexp(pattern: &str) -> ExitCode {
    if pattern != r"^gig\.category\..*\.pattern$" {
        return ExitCode::FAILURE;
    }

    let store = read_store();
    let matches: Vec<_> = store
        .iter()
        .filter(|(k, _)| k.starts_with("gig.category.") && k.ends_with(".pattern"))
        .collect();

    if matches.is_empty() {
        return ExitCode::FAILURE;
    }
    for (key, value) in matches {
        println!("{key} {value}");
    }
    ExitCode::SUCCESS
}

/// Simulates a successful `git clone <url> <dest>` by creating a `.git` marker
/// directory at the destination, which is exactly what gig's own
/// already-cloned detection looks for.
fn handle_clone(args: &[String]) -> ExitCode {
    let Some(dest) = args.last() else {
        return ExitCode::FAILURE;
    };
    match fs::create_dir_all(PathBuf::from(dest).join(".git")) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
