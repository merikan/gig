//! Pure builder turning a parsed URL into the on-disk path it clones to,
//! mirroring the URL's own host/path structure under `root-dir` verbatim.
use std::path::{Path, PathBuf};

use crate::url_parser::ParsedUrl;

pub fn destination_path(root_dir: &Path, parsed: &ParsedUrl) -> PathBuf {
    let mut path = root_dir.join(&parsed.host);
    path.extend(&parsed.path_segments);
    path.push(&parsed.repo_name);
    path
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::url_parser;

    #[test]
    fn destination_mirrors_the_parsed_url_structure() {
        let root_dir = Path::new("/root");
        let cases: &[(&str, &str)] = &[
            (
                "git@github.com:owner/repo.git",
                "/root/github.com/owner/repo",
            ),
            (
                "https://github.com/owner/repo",
                "/root/github.com/owner/repo",
            ),
            (
                "https://gitlab.com/group/subgroup/repo",
                "/root/gitlab.com/group/subgroup/repo",
            ),
            (
                "https://gitlab.com/group/subgroup/deeper/repo.git",
                "/root/gitlab.com/group/subgroup/deeper/repo",
            ),
            ("git@git.sr.ht:~user/repo", "/root/git.sr.ht/~user/repo"),
            ("git@example.com:repo.git", "/root/example.com/repo"),
        ];

        for (url, expected) in cases {
            let parsed = url_parser::parse(url).expect("valid url");
            assert_eq!(
                destination_path(root_dir, &parsed),
                PathBuf::from(expected),
                "destination for `{url}`"
            );
        }
    }
}
