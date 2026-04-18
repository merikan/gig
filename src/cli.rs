use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "gig",
    version,
    about = "Clone repos into a predictable host/owner/repo workspace"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Clone a repo by URL, or update it if already cloned
    Get(GetArgs),
    /// View or set gig configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// List every repo already cloned under root-dir
    #[command(alias = "ls")]
    List,
}

#[derive(Debug, Args)]
pub struct GetArgs {
    pub url: String,
    #[arg(long)]
    pub pull: bool,
    #[arg(long)]
    pub category: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// View or set the root-dir where repos are cloned
    RootDir { path: Option<String> },
    /// View, set, or list category routing rules
    Category(CategoryArgs),
}

/// The four forms `config category` accepts, dispatched by which fields are
/// present:
/// - neither `name` nor `pattern` nor `flag_only`: list every declared
///   category (`<name>\t<pattern>`, one per line, declaration order).
/// - `name` only: print that category's pattern (error if undeclared).
/// - `name` + `pattern`: declare/update that category with the given pattern.
/// - `name` + `--flag-only`: declare that category with an empty pattern.
///
/// `pattern` and `flag_only` are mutually exclusive.
#[derive(Debug, Args)]
pub struct CategoryArgs {
    pub name: Option<String>,
    pub pattern: Option<String>,
    /// Declare the category with no pattern, usable only via `--category`
    #[arg(long = "flag-only")]
    pub flag_only: bool,
}
