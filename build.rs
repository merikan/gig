fn main() {
    // Emits build-time metadata (package version, git commit, dirty state)
    // consumed by `src/version.rs`. `built` itself already degrades
    // gracefully when there's no `.git` directory (source tarball,
    // `cargo install`) or a shallow clone (CI) - GIT_* constants just come
    // back `None`, which version.rs falls back on. Only report, don't fail
    // the build, if writing the generated file itself goes wrong.
    if let Err(err) = built::write_built_file() {
        println!("cargo:warning=failed to gather build-time info: {err}");
    }
}
