mod cli;

use clap::Parser;
use cli::{Cli, Commands, ConfigCommand, GetArgs};

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

fn run_get(_args: GetArgs) -> anyhow::Result<()> {
    Ok(())
}

fn run_config(_command: ConfigCommand) -> anyhow::Result<()> {
    Ok(())
}

fn run_list() -> anyhow::Result<()> {
    Ok(())
}
