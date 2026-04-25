mod category_config;
mod category_routing;
mod cli;
mod debug_log;
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
    debug_log::init(cli.debug);

    match cli.command {
        Commands::Get(args) => run_get(&args),
        Commands::Config { command } => run_config(command),
        Commands::List => run_list(),
    }
}

/// `get` is the default subcommand, so `gig <url>` must parse the same as
/// `gig get <url>` even though clap has no built-in notion of a default
/// subcommand. Leading global flags (currently just `--debug`, which is valid
/// anywhere via `global = true`) are skipped over first, so `gig --debug <url>`
/// and `gig --debug config ...` both still resolve correctly rather than
/// having `--debug` mistaken for the URL/subcommand itself.
fn normalize_args(mut args: Vec<String>) -> Vec<String> {
    const KNOWN_SUBCOMMANDS: &[&str] = &["get", "config", "list", "ls", "help"];
    const HELP_FLAGS: &[&str] = &["-h", "--help", "-V", "--version"];
    const GLOBAL_FLAGS: &[&str] = &["--debug"];

    let insert_at = args
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, arg)| !GLOBAL_FLAGS.contains(&arg.as_str()))
        .filter(|(_, arg)| {
            !KNOWN_SUBCOMMANDS.contains(&arg.as_str()) && !HELP_FLAGS.contains(&arg.as_str())
        })
        .map(|(index, _)| index);

    if let Some(index) = insert_at {
        args.insert(index, "get".to_string());
    }
    args
}

fn run_get(args: &GetArgs) -> anyhow::Result<()> {
    let root_dir = root_dir()?;
    let parsed = url_parser::parse(&args.url)?;
    let categories = category_config::list()?;

    // Resolved (and validated) unconditionally, even when the existing-clone
    // search below ends up overriding it: every declared category's pattern
    // must compile, and an explicit `--category` must reference an
    // already-declared category, on every invocation - not just the ones
    // that actually need a freshly resolved destination.
    let destination_root =
        resolve_destination_root(&root_dir, &parsed, args.category.as_deref(), &categories)?;

    let destination = find_existing_clone(&root_dir, &parsed, &categories)
        .unwrap_or_else(|| destination::destination_path(&destination_root, &parsed));

    let state = existing_clone_state(&destination);
    debug_log::log(format!(
        "destination {} classified as {state}",
        destination.display()
    ));

    match state {
        // No gig message here by design: git's own pull output (diffstat,
        // "Already up to date.", ...) is what the user should see instead.
        DestinationState::AlreadyCloned if args.pull => git_ops::pull(&destination),
        DestinationState::AlreadyCloned => {
            println!("Already cloned at {}", destination.display());
            note_if_category_resolution_differs(&destination, &destination_root, &parsed);
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
    categories: &[category_config::Category],
) -> anyhow::Result<PathBuf> {
    let matched = category_routing::resolve(categories, &parsed.normalized_path())?;
    debug_log::log(format!(
        "category routing: '{}' against {} declared categories -> {}",
        parsed.normalized_path(),
        categories.len(),
        matched.as_deref().unwrap_or("no match")
    ));

    let category = match category_override {
        Some(name) => {
            if !categories.iter().any(|c| c.name == name) {
                return Err(category_not_declared_error(name));
            }
            debug_log::log(format!(
                "--category {name} overrides automatic category matching"
            ));
            Some(name.to_string())
        }
        None => matched,
    };
    Ok(category.map_or_else(|| root_dir.to_path_buf(), |name| root_dir.join(name)))
}

/// Every location `get` recognizes as a possible existing clone for this
/// URL: the default `root_dir/host/owner/repo` path, plus
/// `root_dir/<category>/host/owner/repo` for every declared category -
/// regardless of whether that category's pattern currently matches this URL,
/// or would even be reachable via `--category`/regex resolution. Declared
/// categories are included unconditionally (even flag-only ones) because a
/// repo may have been cloned there in the past under a rule that has since
/// changed.
fn candidate_destinations(
    root_dir: &Path,
    parsed: &url_parser::ParsedUrl,
    categories: &[category_config::Category],
) -> Vec<PathBuf> {
    let mut candidates = vec![destination::destination_path(root_dir, parsed)];
    candidates.extend(
        categories
            .iter()
            .map(|category| destination::destination_path(&root_dir.join(&category.name), parsed)),
    );
    candidates
}

/// Searches every candidate destination (default path first, then declared
/// categories in declaration order) for one that's already cloned. This is
/// what makes category rule changes non-destructive: a repo cloned before a
/// category existed, or after one was added/removed/reordered, is still
/// found at wherever it actually lives, rather than being duplicated under
/// whatever path current `--category`/regex resolution would otherwise pick.
fn find_existing_clone(
    root_dir: &Path,
    parsed: &url_parser::ParsedUrl,
    categories: &[category_config::Category],
) -> Option<PathBuf> {
    let candidates = candidate_destinations(root_dir, parsed, categories);
    let candidate_count = candidates.len();
    for candidate in candidates {
        if has_git_dir(&candidate) {
            debug_log::log(format!(
                "candidate destination {}: already cloned",
                candidate.display()
            ));
            return Some(candidate);
        }
        debug_log::log(format!(
            "candidate destination {}: not found",
            candidate.display()
        ));
    }
    debug_log::log(format!(
        "no existing clone found among {candidate_count} candidate destination(s)"
    ));
    None
}

/// Informational note printed alongside "Already cloned at ..." when the
/// clone that was actually found lives somewhere other than where current
/// `--category`/regex resolution would place a fresh clone - e.g. a category
/// rule was declared or changed after the repo was already cloned elsewhere.
/// Purely advisory: the existing clone is still what `get`/`--pull` act on.
fn note_if_category_resolution_differs(
    destination: &Path,
    destination_root: &Path,
    parsed: &url_parser::ParsedUrl,
) {
    let resolved = destination::destination_path(destination_root, parsed);
    if resolved != destination {
        println!(
            "Note: current category rules would clone this to {} instead",
            resolved.display()
        );
    }
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

impl std::fmt::Display for DestinationState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Free => "free",
            Self::AlreadyCloned => "already-cloned",
            Self::Occupied => "occupied",
        })
    }
}

/// A destination is "already cloned" only if it contains a `.git`
/// subdirectory; anything else that already exists there (a stray file, an
/// empty dir, unrelated contents) is `Occupied` rather than clonable.
fn existing_clone_state(destination: &Path) -> DestinationState {
    if !destination.exists() {
        DestinationState::Free
    } else if has_git_dir(destination) {
        DestinationState::AlreadyCloned
    } else {
        DestinationState::Occupied
    }
}

/// Whether `path` contains a `.git` subdirectory - the sole signal `gig`
/// uses to recognize an existing clone.
fn has_git_dir(path: &Path) -> bool {
    path.join(".git").is_dir()
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

/// `config category [<name>] [<pattern> ...] [--add] [--flag-only]` - see
/// the `CategoryArgs` doc comment in `cli.rs` for the full set of forms.
fn run_config_category(args: CategoryArgs) -> anyhow::Result<()> {
    let CategoryArgs {
        name,
        patterns,
        flag_only,
        add,
    } = args;
    let Some(name) = name else {
        if flag_only || add || !patterns.is_empty() {
            anyhow::bail!("--add, --flag-only, and <pattern> require a category <name>");
        }
        return run_config_category_list();
    };

    if add && flag_only {
        anyhow::bail!(
            "--add and --flag-only are mutually exclusive - `config category {name}` was given both"
        );
    }
    if flag_only && !patterns.is_empty() {
        anyhow::bail!(
            "<pattern> and --flag-only are mutually exclusive - `config category {name}` was given both"
        );
    }
    if add && patterns.is_empty() {
        anyhow::bail!("at least one pattern is required with --add");
    }

    if add {
        run_config_category_add(&name, &patterns)
    } else if flag_only {
        run_config_category_replace(&name, &[String::new()])
    } else if !patterns.is_empty() {
        run_config_category_replace(&name, &patterns)
    } else {
        run_config_category_view(&name)
    }
}

/// `config category <name> <pattern> ... --add`: appends `patterns` to an
/// already-declared category's list, skipping exact duplicates. Errors,
/// without writing anything, if `name` isn't already declared - `--add`
/// never creates a category.
fn run_config_category_add(name: &str, patterns: &[String]) -> anyhow::Result<()> {
    let existing = category_config::patterns(name)?;
    if existing.is_empty() {
        return Err(category_not_declared_error(name));
    }
    category_config::add(name, &existing, patterns)
}

/// `config category <name> <pattern> ...` / `--flag-only`: replaces `name`'s
/// entire pattern list with `patterns` (a single empty string for
/// `--flag-only`). Warns on stderr when the replace drops at least one
/// previously-declared, non-empty pattern - never for a brand-new category
/// or one that was previously flag-only (i.e. had no *real* pattern to
/// drop).
fn run_config_category_replace(name: &str, patterns: &[String]) -> anyhow::Result<()> {
    let existing = category_config::patterns(name)?;
    if existing
        .iter()
        .any(|pattern| !pattern.is_empty() && !patterns.contains(pattern))
    {
        eprintln!(
            "warning: `config category {name}` replaced its pattern list, dropping previously declared pattern(s)"
        );
    }
    category_config::replace(name, &existing, patterns)
}

/// `config category <name>` (view): every pattern currently declared for
/// `name`, one per line, in declaration/add order. Errors if `name` isn't
/// declared.
fn run_config_category_view(name: &str) -> anyhow::Result<()> {
    let patterns = category_config::patterns(name)?;
    if patterns.is_empty() {
        return Err(category_not_declared_error(name));
    }
    for pattern in patterns {
        println!("{pattern}");
    }
    Ok(())
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
