mod category_config;
mod category_routing;
mod cli;
mod destination;
mod git_cmd;
mod git_config;
mod git_ops;
mod repo_walk;
mod url_parser;

use clap::Parser;
use cli::{CategoryArgs, Cli, Commands, ConfigCommand, GetArgs};
use std::path::{Path, PathBuf};

const ROOT_DIR_KEY: &str = "gig.root-dir";
const ROOT_DIR_UNSET_MESSAGE: &str =
    "gig.root-dir is not set. Run `gig config root-dir <path>` to set it.";

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse_from(normalize_args(std::env::args().collect()));

    match cli.command {
        Commands::Get(args) => run_get(&args),
        Commands::Config { command } => run_config(command),
        Commands::List => run_list(),
    }
}

/// `get` is the default subcommand, so `gig <url>` must parse the same as
/// `gig get <url>` even though clap has no built-in notion of a default subcommand.
fn normalize_args(mut args: Vec<String>) -> Vec<String> {
    const KNOWN_SUBCOMMANDS: &[&str] = &["get", "config", "list", "ls", "help"];
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
    let root_dir = root_dir()?;
    let parsed = url_parser::parse(&args.url)?;
    let destination_root = resolve_destination_root(&root_dir, &parsed, args.category.as_deref())?;
    let destination = destination::destination_path(&destination_root, &parsed);

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

/// The root a clone's destination is built under: `root-dir/<category>` if
/// a category applies, `root-dir` itself otherwise. Every declared
/// category's pattern is compiled here regardless of whether it ends up
/// matching or is even reachable this invocation - an invalid regex in any
/// of them aborts the run, naming that category.
///
/// Precedence: an explicit `category_override` (the `--category` flag) wins
/// outright and must reference an already-declared category - otherwise
/// this errors and no clone is attempted. Absent that, `parsed`'s normalized
/// `host/owner/repo` string is matched against every declared category's
/// pattern (first match in git-config declaration order wins).
fn resolve_destination_root(
    root_dir: &Path,
    parsed: &url_parser::ParsedUrl,
    category_override: Option<&str>,
) -> anyhow::Result<PathBuf> {
    let categories = category_config::list()?;
    let matched = category_routing::resolve(&categories, &parsed.normalized_path())?;

    let category = match category_override {
        Some(name) => {
            if !categories.iter().any(|c| c.name == name) {
                return Err(category_not_declared_error(name));
            }
            Some(name.to_string())
        }
        None => matched,
    };
    Ok(category.map_or_else(|| root_dir.to_path_buf(), |name| root_dir.join(name)))
}

/// The shared "you must declare a category before using it" error, raised
/// both by `--category <name>` on `get` and by `config category <name>`
/// (view) on an undeclared name.
fn category_not_declared_error(name: &str) -> anyhow::Error {
    anyhow::anyhow!(
        "category '{name}' is not declared. Run `gig config category {name} <pattern>` to declare it."
    )
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
        ConfigCommand::Category(args) => run_config_category(args),
    }
}

/// `config category [<name>] [<pattern>] [--flag-only]` - see the
/// `CategoryArgs` doc comment in `cli.rs` for the full set of forms.
fn run_config_category(args: CategoryArgs) -> anyhow::Result<()> {
    let CategoryArgs {
        name,
        pattern,
        flag_only,
    } = args;
    let Some(name) = name else {
        if flag_only || pattern.is_some() {
            anyhow::bail!("--flag-only and <pattern> require a category <name>");
        }
        return run_config_category_list();
    };

    match (pattern, flag_only) {
        (Some(_), true) => anyhow::bail!(
            "<pattern> and --flag-only are mutually exclusive - `config category {name}` was given both"
        ),
        (Some(pattern), false) => category_config::set(&name, &pattern),
        (None, true) => category_config::set(&name, ""),
        (None, false) => category_config::get(&name)?.map_or_else(
            || Err(category_not_declared_error(&name)),
            |pattern| {
                println!("{pattern}");
                Ok(())
            },
        ),
    }
}

/// `config category` with no name: every declared category, one per line,
/// `<name>\t<pattern>`, in git-config declaration order.
fn run_config_category_list() -> anyhow::Result<()> {
    for category in category_config::list()? {
        println!("{}\t{}", category.name, category.pattern);
    }
    Ok(())
}

fn run_list() -> anyhow::Result<()> {
    for repo in repo_walk::find_repos(&root_dir()?)? {
        println!("{}", repo.display());
    }
    Ok(())
}

/// The configured `root-dir`, or the actionable "not set" error - the
/// precondition every `get`/`list` invocation shares.
fn root_dir() -> anyhow::Result<PathBuf> {
    git_config::get(ROOT_DIR_KEY)?
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!(ROOT_DIR_UNSET_MESSAGE))
}
