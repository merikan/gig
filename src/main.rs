mod cli;
mod destination;
mod git_cmd;
mod git_config;
mod git_ops;
mod url_parser;

use clap::Parser;
use cli::{Cli, Commands, ConfigCommand, GetArgs};
use std::path::{Path, PathBuf};

const ROOT_DIR_KEY: &str = "gig.root-dir";
const ROOT_DIR_UNSET_MESSAGE: &str =
    "gig.root-dir is not set. Run `gig config root-dir <path>` to set it.";

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse_from(normalize_args(std::env::args().collect()));

    match cli.command {
        Commands::Get(args) => run_get(&args),
        Commands::Config { command } => run_config(command),
        Commands::List => {
            run_list();
            Ok(())
        }
    }
}

/// `get` is the default subcommand, so `gig <url>` must parse the same as
/// `gig get <url>` even though clap has no built-in notion of a default subcommand.
fn normalize_args(mut args: Vec<String>) -> Vec<String> {
    const KNOWN_SUBCOMMANDS: &[&str] = &["get", "config", "list", "help"];
    const HELP_FLAGS: &[&str] = &["-h", "--help", "-V", "--version"];

    if let Some(first) = args.get(1)
        && !KNOWN_SUBCOMMANDS.contains(&first.as_str())
        && !HELP_FLAGS.contains(&first.as_str())
    {
        args.insert(1, "get".to_string());
    }
    args
}

fn run_get(args: &GetArgs) -> anyhow::Result<()> {
    let root_dir =
        git_config::get(ROOT_DIR_KEY)?.ok_or_else(|| anyhow::anyhow!(ROOT_DIR_UNSET_MESSAGE))?;
    let parsed = url_parser::parse(&args.url)?;
    let destination = destination::destination_path(&PathBuf::from(root_dir), &parsed);

    match existing_clone_state(&destination) {
        // No gig message here by design: git's own pull output (diffstat,
        // "Already up to date.", ...) is what the user should see instead.
        DestinationState::AlreadyCloned if args.pull => git_ops::pull(&destination),
        DestinationState::AlreadyCloned => {
            println!("Already cloned at {}", destination.display());
            Ok(())
        }
        DestinationState::Occupied => anyhow::bail!(
            "{} already exists and is not a git repository (no .git directory) - refusing to clone into it",
            destination.display()
        ),
        DestinationState::Free => {
            println!("Cloning {} into {}...", args.url, destination.display());
            git_ops::clone(&args.url, &destination)?;
            println!("Cloned into {}", destination.display());
            Ok(())
        }
    }
}

/// What, if anything, is already at a computed destination path - drives
/// whether `get` clones, no-ops/pulls, or errors out rather than clobbering
/// something unrelated.
enum DestinationState {
    Free,
    AlreadyCloned,
    Occupied,
}

/// A destination is "already cloned" only if it contains a `.git`
/// subdirectory; anything else that already exists there (a stray file, an
/// empty dir, unrelated contents) is `Occupied` rather than clonable.
fn existing_clone_state(destination: &Path) -> DestinationState {
    if !destination.exists() {
        DestinationState::Free
    } else if destination.join(".git").is_dir() {
        DestinationState::AlreadyCloned
    } else {
        DestinationState::Occupied
    }
}

fn run_config(command: ConfigCommand) -> anyhow::Result<()> {
    match command {
        ConfigCommand::RootDir { path: Some(path) } => git_config::set_global(ROOT_DIR_KEY, &path),
        ConfigCommand::RootDir { path: None } => match git_config::get(ROOT_DIR_KEY)? {
            Some(value) => {
                println!("{value}");
                Ok(())
            }
            None => anyhow::bail!(ROOT_DIR_UNSET_MESSAGE),
        },
        ConfigCommand::Category { .. } => Ok(()),
    }
}

const fn run_list() {}
