# gig

A Rust CLI that clones repos into a predictable `root-dir/host/owner/repo` workspace, and recognizes repos already cloned there instead of re-cloning or clobbering them.

If you've ever ended up with three different local copies of the same repo because you `git clone`d it from three different directories, `gig` is for that. Give it a `root-dir` once, and every repo you fetch through it lands at a deterministic path derived from its URL - `github.com/owner/repo`, `gitlab.company.com/team/service`, and so on - so "do I already have this cloned?" always has one obvious answer.

Authentication is fully delegated to your system's existing `ssh-agent`/git credential setup - `gig` shells out to your own `git` binary for the actual clone/pull, it doesn't reimplement git.

## Features

- **Predictable clone paths** - `gig get <url>` always resolves to `root-dir/host/owner/repo`, regardless of your current directory.
- **Clone-or-update, never clobber** - re-running `gig get` on an already-cloned repo no-ops (or pulls, with `--pull`); it refuses to touch a destination that exists but isn't a clone.
- **Auto-cd** - with `gig shellenv` set up, `gig get` `cd`s you into the destination on success, whether it was freshly cloned or already there - configurable via `gig config autocd-into`.
- **Category routing** - declare regex rules that route clones under a different subtree (e.g. work repos under `~/root-dir/work`,  oss ones under `~/root-dir/oss`) instead of the default `root-dir` root, with an optional default category to catch anything unmatched.
- **Interactive cd** - `gig cd` opens a fuzzy-searchable picker of every repo already cloned under `root-dir` and, with `gig shellenv` set up, `cd`s into the one you pick.
- **Shell completion** - structural completion (subcommand/flag names) for bash, zsh, and fish.

## Installation

There's no published crate or release binary yet - build from source with [Cargo](https://www.rust-lang.org/tools/install):

```sh
git clone https://github.com/merikan/gig.git
cd gig
cargo install --path .
```

This installs the `gig` binary to `~/.cargo/bin` (make sure that's on your `PATH`).

If you use [mise](https://mise.jdx.dev/), the repo's `mise.toml` also exposes `mise run build:release`, which produces `target/release/gig` without installing it.

## Getting started

Set a `root-dir` - the directory under which every clone will be organized. This is stored in your global git config (`gig.root-dir`), not a separate config file:

```sh
gig config root-dir ~/code
```

Now clone something:

```sh
gig get https://github.com/rust-lang/rust
# clones into ~/code/github.com/rust-lang/rust
```

Run it again any time - it's safe:

```sh
gig get https://github.com/rust-lang/rust
# already cloned, no-op

gig get https://github.com/rust-lang/rust --pull
# already cloned, pulls latest instead
```

## Commands

### `gig get <url>`

Clone a repo by URL (SSH or HTTPS), or update it if it's already cloned. This is also the implicit default subcommand - `gig <url>` alone is equivalent to `gig get <url>`.

```sh
gig get git@github.com:owner/repo.git
gig https://gitlab.com/group/subgroup/repo   # same thing, shorthand
```

Behavior depends on what's already at the destination:

| Destination state | Default | With `--pull` |
|---|---|---|
| Nothing there | clones | clones |
| Already cloned (has `.git`) | no-op | pulls |
| Occupied (exists, not a clone) | errors, refuses to touch it | errors, refuses to touch it |

Flags:

- `--pull` - if already cloned, pull instead of no-op.
- `--category <name>` - force this clone into a specific declared category's destination, overriding automatic pattern routing (see [Category routing](#category-routing) below).
- `--cd` - auto-cd into the destination after this invocation, overriding `gig.autocd-into` (see [Auto-cd](#auto-cd) below). Mutually exclusive with `--no-cd`.
- `--no-cd` - skip auto-cd after this invocation, overriding `gig.autocd-into`. Mutually exclusive with `--cd`.

### `gig config root-dir [path]`

View or set the root directory repos are cloned under.

```sh
gig config root-dir          # print the current root-dir (errors if unset)
gig config root-dir ~/code   # set it (accepts ~, expanded to your home dir)
```

### `gig config autocd-into [true|false]`

View or set whether `get` auto-cds into the destination (see [Auto-cd](#auto-cd)). Defaults to `true` when unset.

```sh
gig config autocd-into         # print the current value ("true" if unset)
gig config autocd-into false   # disable it
```

### `gig config category [name] [patterns...]`

View, declare, or update category routing rules. See [Category routing](#category-routing).

```sh
gig config category                    # list every declared category
gig config category work               # print one category's patterns
gig config category work '^gitlab\.company\.com/'   # declare/replace
gig config category work '^github\.com/my-employer/' --add  # append a pattern
gig config category adhoc --flag-only  # declare with no patterns - usable only via --category
gig config category personal --default # route anything unmatched here (see Category routing)
gig config category personal --no-default  # stop being the fallback, patterns untouched
```

### `gig list` (alias: `gig ls`)

List every repo already cloned under `root-dir`, as plain relative paths:

```sh
gig list
# github.com/rust-lang/rust
# gitlab.company.com/team/service
```

### `gig cd`

Opens a fuzzy-searchable picker (type to filter, arrows to move, Enter to pick) of every repo already cloned under `root-dir` - the same set `gig list` prints. On its own, `gig cd` can only print the chosen repo's absolute path to stdout; it's a separate process and can't change your shell's working directory. Pair it with `gig shellenv` (below) to actually `cd`:

```sh
gig cd
# type to filter, Enter to select - your shell cd's into the chosen repo
```

Without `gig shellenv` set up, you can still use it manually:

```sh
cd "$(gig cd)"
```

Errors (nothing found under `root-dir`, the picker cancelled, stderr isn't a terminal) print to stderr and print nothing to stdout, so a `cd "$(gig cd)"` never `cd`s anywhere on failure.

### `gig shellenv <shell>`

Prints a shell function for `bash`, `zsh`, or `fish` that wires `gig cd`'s picker into an actual `cd`, and `gig get`'s auto-cd (see [Auto-cd](#auto-cd)) into an actual `cd` too - every other subcommand passes through to `gig` unchanged. `eval`/`source` its output:

```sh
eval "$(gig shellenv bash)"
eval "$(gig shellenv zsh)"
gig shellenv fish | source
```

To make it permanent, append the relevant line to your shell's startup file:

```sh
# ~/.bashrc
echo 'eval "$(gig shellenv bash)"' >> ~/.bashrc

# ~/.zshrc
echo 'eval "$(gig shellenv zsh)"' >> ~/.zshrc

# ~/.config/fish/config.fish
echo 'gig shellenv fish | source' >> ~/.config/fish/config.fish
```

### `gig completion <shell>`

Print a shell completion script to stdout, for `bash`, `zsh`, or `fish`. Covers subcommand and flag names (structural completion); it doesn't complete dynamic values like category names or already-cloned repo paths.

Try it for a session without installing anything:

```sh
source <(gig completion bash)
source <(gig completion zsh)
gig completion fish | source
```

To make it permanent, append the relevant line to your shell's startup file:

```sh
# ~/.bashrc
echo 'source <(gig completion bash)' >> ~/.bashrc

# ~/.zshrc
echo 'source <(gig completion zsh)' >> ~/.zshrc

# ~/.config/fish/config.fish
echo 'gig completion fish | source' >> ~/.config/fish/config.fish
```

### `gig version`

Print the current version, including the git commit it was built from, e.g. `gig 0.1.0 (a1b2c3d)`. Same output as `gig --version`/`gig -V`.

### Global flags

- `--debug` - print diagnostic detail to stderr: which git commands ran, how category routing decided a destination, which candidate destinations were checked. Works on any subcommand, e.g. `gig get <url> --debug`.
- `--version`/`-V` - print the current version and exit; same as `gig version`.

## Category routing

By default, every clone lands under `root-dir/host/owner/repo`. Categories let you declare regex rules that route matching repos under a different subtree instead - handy for separating work repos from personal ones, or grouping everything from one self-hosted GitLab under its own folder.

A category is a name plus one or more RE2-syntax patterns, matched against the `host/owner/repo` path a repo *would* be cloned to (not its URL). When you `gig get` a URL, `gig` checks it against every declared category's patterns; a match routes the clone under `root-dir/<category-name>/...` instead of the default path.

Example - route everything under a work GitLab into its own subtree:

```sh
gig config category work '^gitlab\.company\.com/'

gig get https://gitlab.company.com/team/service
# routed via category "work" -> root-dir/work/gitlab.company.com/team/service
```

You can also force a clone into a category regardless of pattern matching:

```sh
gig get https://github.com/some/repo --category work
```

Or declare a category with no patterns at all (`--flag-only`), usable only through the explicit `--category` override, never automatically.

### Default category

To route everything that no other category's pattern matches, mark one category `--default` instead of the default `root-dir` root:

```sh
gig config category personal --default

gig get https://github.com/some/unmatched-repo
# no category pattern matched -> root-dir/personal/github.com/some/unmatched-repo
```

`--default` is independent of patterns - a category can carry real patterns *and* be the default, matching normally by pattern first and only catching leftovers when nothing (including itself) matched. Only one category can be default at a time; marking a new one auto-demotes the previous, with a warning. `--no-default` clears the flag again (patterns untouched, safe to run even if the category wasn't already default).

This replaces the older trick of declaring a catch-all pattern (e.g. `.*`) as the *last* category, which only worked because matching is first-match-in-declaration-order - reordering categories later would silently break it. `--default` isn't affected by declaration order.

`gig` never moves a repo that's already cloned somewhere else just because you add or change a category rule afterward - it searches existing clone locations non-destructively before deciding where a `get` lands.

## Auto-cd

With `gig shellenv` set up (see [`gig shellenv`](#gig-shellenv-shell) above), `gig get` `cd`s your shell into the destination on success - whether it just cloned, was already cloned, or was pulled - the same way `gig cd` does for its picker. Controlled by `gig.autocd-into`, defaulting to enabled:

```sh
gig config autocd-into false   # disable auto-cd everywhere
gig get https://github.com/rust-lang/rust
# clones (or no-ops/pulls), but doesn't cd this time

gig get https://github.com/rust-lang/rust --cd
# cds anyway, just for this invocation
```

Or leave it enabled (the default) and opt out per invocation instead:

```sh
gig get https://github.com/rust-lang/rust --no-cd
# clones (or no-ops/pulls), but doesn't cd this time
```

`--cd` and `--no-cd` are mutually exclusive and always override the config for that one invocation. Without `gig shellenv` set up, auto-cd has nothing to hook into and is a no-op either way - `get`'s own "Cloned into `<path>`"/"Already cloned at `<path>`" messages still show you the destination.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, testing, and commit conventions. Participation is governed by the [Code of Conduct](CODE_OF_CONDUCT.md).

Design rationale for some of the less obvious decisions lives in `docs/adr/`.

## License

MIT - see [LICENSE](LICENSE).
