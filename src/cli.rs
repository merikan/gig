use clap::{Args, Parser, Subcommand, ValueEnum, ValueHint};

#[derive(Debug, Parser)]
#[command(
    name = "gig",
    version,
    about = "Clone repos into a predictable host/owner/repo workspace"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Print diagnostic detail (git commands run, category routing,
    /// destination resolution) to stderr
    #[arg(long, global = true)]
    pub debug: bool,
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
    /// Print a shell completion script to stdout, for sourcing
    Completion(CompletionArgs),
}

#[derive(Debug, Args)]
pub struct CompletionArgs {
    pub shell: CompletionShell,
}

/// The shells `gig completion` generates a script for. Deliberately narrower
/// than `clap_complete::Shell` (which also offers elvish/powershell) - see
/// `docs/adr/0002-scope-shell-completion-to-bash-zsh-fish-structural-only.md`.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
}

impl From<CompletionShell> for clap_complete::Shell {
    fn from(shell: CompletionShell) -> Self {
        match shell {
            CompletionShell::Bash => Self::Bash,
            CompletionShell::Zsh => Self::Zsh,
            CompletionShell::Fish => Self::Fish,
        }
    }
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
    RootDir {
        #[arg(value_hint = ValueHint::DirPath)]
        path: Option<String>,
    },
    /// View, set, or list category routing rules
    Category(CategoryArgs),
}

/// The five forms `config category` accepts, dispatched by which fields are
/// present:
/// - neither `name` nor `patterns` nor `flag_only` nor `add`: list every
///   declared category (`<name>\t<pattern>`, one per line per pattern,
///   declaration order).
/// - `name` only: print that category's patterns, one per line (error if
///   undeclared).
/// - `name` + `patterns`: declare `name` if new, otherwise **replace** its
///   entire pattern list with the given pattern(s).
/// - `name` + `patterns` + `--add`: **append** the given pattern(s) to
///   `name`'s existing list (errors if `name` isn't already declared;
///   an exact-duplicate pattern is a silent no-op).
/// - `name` + `--flag-only`: replace that category's pattern list with an
///   empty one.
///
/// `patterns` and `--flag-only` are mutually exclusive, as are `--add` and
/// `--flag-only`.
#[derive(Debug, Args)]
#[command(after_help = "\
Pattern examples (RE2 syntax, matched against the `host/owner/repo` path a repo would be cloned to - not the URL):

  gig config category personal '^github\\.com/merikan/'
      Every repo under the `merikan` owner on github.com.

  gig config category work '^gitlab\\.company\\.com/'
      Every repo on a self-hosted GitLab instance, regardless of owner/subgroup.

  gig config category oss '^github\\.com/(rust-lang|tokio-rs)/'
      Every repo under either of two specific owners on github.com.

  gig config category oss '^github\\.com/other-owner/one-off-repo$' --add
      Route one more one-off repo into the existing `oss` category, without
      folding it into the pattern above.")]
pub struct CategoryArgs {
    pub name: Option<String>,
    pub patterns: Vec<String>,
    /// Declare the category with no patterns, usable only via `--category`
    #[arg(long = "flag-only")]
    pub flag_only: bool,
    /// Append the given pattern(s) to the category's existing list instead
    /// of replacing it; errors if the category isn't already declared
    #[arg(long = "add")]
    pub add: bool,
}
