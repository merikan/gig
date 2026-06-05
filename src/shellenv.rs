//! Shell integration for `gig cd`: `gig` itself can't change its parent
//! shell's working directory, so `shellenv` prints a shell function - meant
//! to be `eval`'d into the user's shell - that runs `gig cd`, captures the
//! path it printed, and `cd`s into it. Every other subcommand passes through
//! the function unchanged.
//!
//! `gig cd` draws its interactive picker on stderr (see `run_cd` in
//! `main.rs`) and prints only the chosen path to stdout on success, nothing
//! on failure/cancellation - so `dir=$(command gig cd)` here is exactly the
//! contract these functions rely on. See
//! `docs/adr/0005-shell-integration-via-stderr-picker-not-a-pty-wrapper.md`.
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
        command gig "$@"
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
        command gig $argv
    end
end
"#;
