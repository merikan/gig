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
    Category {
        name: Option<String>,
        pattern: Option<String>,
        #[arg(long = "flag-only")]
        flag_only: bool,
    },
}
