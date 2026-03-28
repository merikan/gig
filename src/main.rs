mod cli;
mod destination;
mod git_cmd;
mod git_config;
mod git_ops;
mod url_parser;

use clap::Parser;
use cli::{Cli, Commands, ConfigCommand, GetArgs};
use std::path::PathBuf;

const ROOT_DIR_KEY: &str = "git-get.root-dir";
const ROOT_DIR_UNSET_MESSAGE: &str =
    "git-get.root-dir is not set. Run `git-get config root-dir <path>` to set it.";

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse_from(normalize_args(std::env::args().collect()));

    match cli.command {
        Commands::Get(args) => run_get(args),
        Commands::Config { command } => run_config(command),
        Commands::List => run_list(),
    }
}

/// `get` is the default subcommand, so `git-get <url>` must parse the same as
/// `git-get get <url>` even though clap has no built-in notion of a default subcommand.
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

fn run_get(args: GetArgs) -> anyhow::Result<()> {
    let root_dir =
        git_config::get(ROOT_DIR_KEY)?.ok_or_else(|| anyhow::anyhow!(ROOT_DIR_UNSET_MESSAGE))?;
    let parsed = url_parser::parse(&args.url)?;
    let destination = destination::destination_path(&PathBuf::from(root_dir), &parsed);
    git_ops::clone(&args.url, &destination)
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

fn run_list() -> anyhow::Result<()> {
    Ok(())
}
