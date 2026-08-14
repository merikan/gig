//! Shell integration for `gig cd` and `gig get`: `gig` itself can't change
//! its parent shell's working directory, so `shellenv` prints a shell
//! function - meant to be `eval`'d into the user's shell - that runs the
//! real `gig` and `cd`s based on what it reports back. Two different
//! mechanisms are wired up here, one per subcommand, because they have
//! different stdio constraints:
//!
//! - `gig cd` draws its interactive picker on stderr (see `run_cd` in
//!   `main.rs`) and prints only the chosen path to stdout on success,
//!   nothing on failure/cancellation - so `dir=$(command gig cd)` alone is
//!   the whole contract. See
//!   `docs/adr/0005-shell-integration-via-stderr-picker-not-a-pty-wrapper.md`.
//! - `gig get` can't reserve stdout the same way: `git clone`/`git pull`
//!   inherit stdio directly (see
//!   `docs/adr/0001-inherit-stdio-for-clone-and-pull.md`), so stdout has to
//!   stay free for their own live output. Every invocation instead sets
//!   `GIG_CD_FILE` to a fresh tmp file before running `command gig`; `get`
//!   writes its destination there itself (see `maybe_write_cd_file` in
//!   `main.rs`) when auto-cd applies, and the wrapper reads it back
//!   afterwards. See
//!   `docs/adr/0009-sidecar-file-for-gets-auto-cd.md`.
use crate::cli::Shell;

/// The shell function text for `shell`, ready to print for the user to
/// `eval`/`source`. Bash and zsh share one syntax; fish needs its own.
pub const fn script(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash | Shell::Zsh => BASH_ZSH,
        Shell::Fish => FISH,
    }
}

const BASH_ZSH: &str = r#"gig() {
    if [ "$1" = "cd" ]; then
        local dir
        if dir=$(command gig "$@"); then
            cd "$dir" || return
        else
            return 1
        fi
    else
        local cd_file status
        cd_file=$(mktemp)
        GIG_CD_FILE="$cd_file" command gig "$@"
        status=$?
        if [ $status -eq 0 ] && [ -s "$cd_file" ]; then
            cd "$(cat "$cd_file")"
        fi
        rm -f "$cd_file"
        return $status
    fi
}
"#;

const FISH: &str = r#"function gig
    if test "$argv[1]" = cd
        set -l dir (command gig $argv)
        if test $status -eq 0
            cd $dir
        else
            return 1
        end
    else
        set -l cd_file (mktemp)
        set -lx GIG_CD_FILE $cd_file
        command gig $argv
        set -l cmd_status $status
        if test $cmd_status -eq 0; and test -s $cd_file
            cd (cat $cd_file)
        end
        rm -f $cd_file
        return $cmd_status
    end
end
"#;
