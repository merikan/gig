//! Pure parser turning a clone URL into its host/path/repo-name parts. No I/O,
//! no filesystem - the destination-path builder consumes its output separately.
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedUrl {
    pub host: String,
    pub path_segments: Vec<String>,
    pub repo_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    url: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unsupported URL form: `{}` (expected `git@host:path[.git]` or `https://host/path[.git]`)",
            self.url
        )
    }
}

impl std::error::Error for ParseError {}

/// Accepts only SCP-style SSH (`git@host:path[.git]`) and `https://host/path[.git]`.
/// Everything else - `ssh://`, `git://`, `http://`, bare `owner/repo`, aliased
/// shorthand like `alias:owner/repo` - is rejected: none of those match either
/// prefix check below, so they fall through to the same parse error.
pub fn parse(url: &str) -> Result<ParsedUrl, ParseError> {
    if let Some(rest) = url.strip_prefix("https://") {
        let (host, path) = rest.split_once('/').ok_or_else(|| new_error(url))?;
        return build(url, host, path);
    }
    if let Some(rest) = url.strip_prefix("git@") {
        let (host, path) = rest.split_once(':').ok_or_else(|| new_error(url))?;
        return build(url, host, path);
    }
    Err(new_error(url))
}

fn build(url: &str, host: &str, path: &str) -> Result<ParsedUrl, ParseError> {
    if host.is_empty() {
        return Err(new_error(url));
    }

    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let Some((last, path_segments)) = segments.split_last() else {
        return Err(new_error(url));
    };

    let repo_name = last.strip_suffix(".git").unwrap_or(last);
    if repo_name.is_empty() {
        return Err(new_error(url));
    }

    Ok(ParsedUrl {
        host: host.to_string(),
        path_segments: path_segments.iter().map(ToString::to_string).collect(),
        repo_name: repo_name.to_string(),
    })
}

fn new_error(url: &str) -> ParseError {
    ParseError {
        url: url.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(host: &str, path_segments: &[&str], repo_name: &str) -> ParsedUrl {
        ParsedUrl {
            host: host.to_string(),
            path_segments: path_segments.iter().map(ToString::to_string).collect(),
            repo_name: repo_name.to_string(),
        }
    }

    #[test]
    fn valid_urls_parse_into_host_path_and_repo_name() {
        let cases: &[(&str, ParsedUrl)] = &[
            (
                "git@github.com:owner/repo.git",
                parsed("github.com", &["owner"], "repo"),
            ),
            (
                "git@github.com:owner/repo",
                parsed("github.com", &["owner"], "repo"),
            ),
            (
                "https://github.com/owner/repo.git",
                parsed("github.com", &["owner"], "repo"),
            ),
            (
                "https://github.com/owner/repo",
                parsed("github.com", &["owner"], "repo"),
            ),
            (
                "https://gitlab.com/group/subgroup/repo",
                parsed("gitlab.com", &["group", "subgroup"], "repo"),
            ),
            (
                "https://gitlab.com/group/subgroup/deeper/repo.git",
                parsed("gitlab.com", &["group", "subgroup", "deeper"], "repo"),
            ),
            (
                "git@git.sr.ht:~user/repo",
                parsed("git.sr.ht", &["~user"], "repo"),
            ),
            (
                "git@example.com:repo.git",
                parsed("example.com", &[], "repo"),
            ),
        ];

        for (url, expected) in cases {
            assert_eq!(parse(url).as_ref(), Ok(expected), "parsing `{url}`");
        }
    }

    #[test]
    fn unsupported_forms_produce_a_parse_error() {
        let invalid_urls = [
            "ssh://git@github.com/owner/repo.git",
            "git://github.com/owner/repo.git",
            "http://github.com/owner/repo.git",
            "owner/repo",
            "alias:owner/repo",
            "git@github.com",
            "https://github.com",
            "https:///owner/repo",
            "git@:owner/repo",
        ];

        for url in invalid_urls {
            assert!(parse(url).is_err(), "expected `{url}` to be rejected");
        }
    }
}
