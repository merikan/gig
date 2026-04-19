//! Pure regex-matching/precedence-selection logic for automatic category
//! routing. Consumes already-enumerated categories (in git-config
//! declaration order, as [`crate::category_config::list`] returns them) and
//! the normalized `host/owner/repo` string a clone targets - no I/O of its
//! own, and no notion of the `--category` flag override (that rung of the
//! precedence chain lives entirely in `main`, layered on top of this).
use crate::category_config::Category;
use anyhow::{Result, anyhow};
use regex::Regex;

/// Compiles every declared category's pattern - including empty ones, which
/// compile trivially - before testing any of them, so an invalid regex in
/// any declared category aborts immediately, naming that category, even if
/// `match_target` never reaches it during matching. Then returns the name of
/// the first (in declaration order) category whose *non-empty* pattern
/// matches `match_target` - an empty pattern (a flag-only category) is never
/// auto-matched. `Ok(None)` means no declared category matched.
pub fn resolve(categories: &[Category], match_target: &str) -> Result<Option<String>> {
    let mut compiled = Vec::with_capacity(categories.len());
    for category in categories {
        let regex = Regex::new(&category.pattern).map_err(|err| {
            anyhow!(
                "category '{}' has an invalid pattern `{}`: {err}",
                category.name,
                category.pattern
            )
        })?;
        compiled.push((category, regex));
    }

    for (category, regex) in &compiled {
        if !category.pattern.is_empty() && regex.is_match(match_target) {
            return Ok(Some(category.name.clone()));
        }
    }
    Ok(None)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn category(name: &str, pattern: &str) -> Category {
        Category {
            name: name.to_string(),
            pattern: pattern.to_string(),
        }
    }

    #[test]
    fn a_url_matching_exactly_one_category_resolves_to_it() {
        let categories = [category("personal", r"^github\.com/merikan/")];

        let result = resolve(&categories, "github.com/merikan/gig").expect("compiles");

        assert_eq!(result.as_deref(), Some("personal"));
    }

    #[test]
    fn no_declared_category_matches_resolves_to_none() {
        let categories = [category("personal", r"^github\.com/merikan/")];

        let result = resolve(&categories, "github.com/someone-else/repo").expect("compiles");

        assert_eq!(result, None);
    }

    #[test]
    fn the_first_declared_of_two_overlapping_patterns_wins() {
        let categories = [
            category("specific", r"^github\.com/merikan/gig$"),
            category("broad", r"^github\.com/merikan/"),
        ];

        let result = resolve(&categories, "github.com/merikan/gig").expect("compiles");

        assert_eq!(result.as_deref(), Some("specific"));
    }

    #[test]
    fn declaration_order_is_respected_even_when_the_broad_pattern_is_declared_first() {
        let categories = [
            category("broad", r"^github\.com/merikan/"),
            category("specific", r"^github\.com/merikan/gig$"),
        ];

        let result = resolve(&categories, "github.com/merikan/gig").expect("compiles");

        assert_eq!(result.as_deref(), Some("broad"));
    }

    #[test]
    fn a_category_with_an_empty_pattern_is_never_auto_matched() {
        let categories = [category("oneoff", "")];

        let result = resolve(&categories, "github.com/merikan/gig").expect("compiles");

        assert_eq!(result, None);
    }

    #[test]
    fn an_invalid_regex_errors_naming_the_category_even_if_it_would_not_have_matched() {
        let categories = [
            category("broken", "("),
            category("personal", r"^github\.com/merikan/"),
        ];

        let err = resolve(&categories, "gitlab.com/unrelated/repo").expect_err("invalid regex");

        assert!(err.to_string().contains("broken"));
    }
}
