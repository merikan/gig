# gig

A Rust CLI that clones repos into a predictable `root-dir/host/owner/repo` workspace, and recognizes repos already cloned there instead of re-cloning or clobbering them.

If you've ever ended up with three different local copies of the same repo because you `git clone`d it from three different directories, `gig` is for that. Give it a `root-dir` once, and every repo you fetch through it lands at a deterministic path derived from its URL - `github.com/owner/repo`, `gitlab.company.com/team/service`, and so on - so "do I already have this cloned?" always has one obvious answer.

Authentication is fully delegated to your system's existing `ssh-agent`/git credential setup - `gig` shells out to your own `git` binary for the actual clone/pull, it doesn't reimplement git.

## Features

- **Predictable clone paths** - `gig get <url>` always resolves to `root-dir/host/owner/repo`, regardless of your current directory.
- **Clone-or-update, never clobber** - re-running `gig get` on an already-cloned repo no-ops (or pulls, with `--pull`); it refuses to touch a destination that exists but isn't a clone.
- **Category routing** - declare regex rules that route clones under a different subtree (e.g. work repos under `~/root-dir/work`,  oss ones under `~/root-dir/oss`) instead of the default `root-dir` root.
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

### `gig config root-dir [path]`

View or set the root directory repos are cloned under.

```sh
gig config root-dir          # print the current root-dir (errors if unset)
gig config root-dir ~/code   # set it (accepts ~, expanded to your home dir)
```

### `gig config category [name] [patterns...]`

View, declare, or update category routing rules. See [Category routing](#category-routing).

```sh
gig config category                    # list every declared category
gig config category work               # print one category's patterns
gig config category work '^gitlab\.company\.com/'   # declare/replace
gig config category work '^github\.com/my-employer/' --add  # append a pattern
gig config category adhoc --flag-only  # declare with no patterns - usable only via --category
```

### `gig list` (alias: `gig ls`)

List every repo already cloned under `root-dir`, as plain relative paths:

```sh
gig list
# github.com/rust-lang/rust
# gitlab.company.com/team/service
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

### Global flags

- `--debug` - print diagnostic detail to stderr: which git commands ran, how category routing decided a destination, which candidate destinations were checked. Works on any subcommand, e.g. `gig get <url> --debug`.

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

`gig` never moves a repo that's already cloned somewhere else just because you add or change a category rule afterward - it searches existing clone locations non-destructively before deciding where a `get` lands.

## Development

Build/test tasks are defined in `mise.toml` (run via [mise](https://mise.jdx.dev/), or the underlying `cargo`/`cargo tarpaulin` commands directly):

```sh
mise run build          # cargo build
mise run test           # cargo test
mise run lint           # cargo clippy
mise run format         # cargo fmt
mise run check          # cargo check
mise run ci             # check + lint + test
mise run test:coverage  # cargo tarpaulin, HTML report in target/coverage
mise run doc            # cargo doc --open
```

Tests are CLI-level (black-box, via `assert_cmd`) against a stub `git` binary, not the real one - see `tests/`.

Design rationale for some of the less obvious decisions lives in `docs/adr/`.

## License

MIT - see [LICENSE](LICENSE).
