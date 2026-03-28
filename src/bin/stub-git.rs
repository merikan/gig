//! A stand-in for the real `git` binary, injected earlier on `PATH` in acceptance
//! tests so no test ever touches a developer's real gitconfig or network.
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    if let Ok(log_path) = env::var("GIT_GET_STUB_LOG") {
        log_call(&log_path, &args);
    }

    if let Some(forced_exit_code) = env::var("GIT_GET_STUB_EXIT_CODE")
        .ok()
        .and_then(|v| v.parse::<u8>().ok())
    {
        if let Ok(stdout) = env::var("GIT_GET_STUB_STDOUT") {
            print!("{stdout}");
        }
        if let Ok(stderr) = env::var("GIT_GET_STUB_STDERR") {
            eprint!("{stderr}");
        }
        return ExitCode::from(forced_exit_code);
    }

    match args.first().map(String::as_str) {
        Some("config") => handle_config(&args[1..]),
        Some("clone") => handle_clone(&args[1..]),
        Some("-C") => handle_dash_c(&args[1..]),
        _ => ExitCode::FAILURE,
    }
}

/// `git -C <dir> pull`, as issued by `git_ops::pull` to run `pull` against a
/// specific destination rather than the current working directory.
fn handle_dash_c(args: &[String]) -> ExitCode {
    match args {
        [_dir, cmd] if cmd == "pull" => ExitCode::SUCCESS,
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
    env::var_os("GIT_GET_STUB_CONFIG").map(PathBuf::from)
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
    let contents: String = entries.iter().map(|(k, v)| format!("{k}={v}\n")).collect();
    let _ = fs::write(path, contents);
}

/// Handles the two `git config` forms git-get uses today: `--get <key>` and
/// `--global <key> <value>`. `--get-regexp` (needed once category routing lands)
/// isn't implemented yet and falls through to failure like any other unknown form.
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
        [flag, key, value] if flag == "--global" => {
            let mut store = read_store();
            match store.iter_mut().find(|(k, _)| k == key) {
                Some(entry) => entry.1 = value.clone(),
                None => store.push((key.clone(), value.clone())),
            }
            write_store(&store);
            ExitCode::SUCCESS
        }
        _ => ExitCode::FAILURE,
    }
}

/// Simulates a successful `git clone <url> <dest>` by creating a `.git` marker
/// directory at the destination, which is exactly what git-get's own
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
