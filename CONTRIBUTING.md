# Contributing

# Code of Conduct

Before contributing please read our [Code of Conduct](CODE_OF_CONDUCT.md) which
all contributors are expected to adhere to.

By contributing, you agree your contributions are licensed under this project's [MIT license](LICENSE).

## Copyright assignment

Any contribution to this project - whether via pull request, direct commit, or otherwise - assigns to Peter Merikan the economic/exploitation copyright in that contribution, so the project can change license terms in the future without needing to track down every past contributor. In return, you keep a broad, perpetual license to use your own contribution however you like elsewhere.

You represent that the contribution is your own original work and that you have the right to assign it (e.g. it isn't your employer's, or encumbered by a third party's rights).


## Development

Notes on how this repo is built, tested, and organized.

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

There's no CI pipeline yet, so `mise run ci` locally is the check before opening a PR.

## Tests

Tests are CLI-level (black-box, via `assert_cmd`) against a stub `git` binary, not the real one - see `tests/`. Add or update tests when behavior changes.

## Lint and format

`Cargo.toml` denies clippy's `pedantic` and `nursery` lint groups, plus explicit denies on `unwrap`/`expect`/`panic`/`todo`/`unimplemented`/indexing/etc. - these aren't optional style, `mise run lint` must pass clean. Run `mise run format` before committing.

## Commit messages

We follow the conventions on [Conventional Commits](https://www.conventionalcommits.org/) and
[How to Write a Git Commit Message](http://chris.beams.io/posts/git-commit/).

## ADRs

When a change involves a non-obvious design decision (a tradeoff that isn't self-evident from the code), add an entry to `docs/adr/`. See the existing ADRs there for the expected format: a short title, one paragraph of context + decision, and a **Consequence:** paragraph.

## Pull requests

Branch off `main`. Run `mise run ci` locally before opening a PR - there's no automated CI to catch issues yet.
