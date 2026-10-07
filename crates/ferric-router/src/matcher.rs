//! Pure, browser-free route matching: compile a pattern like `/user/:id`, match a concrete path,
//! and extract named parameters.
//!
//! This module is natively testable with no wasm/DOM and MUST NOT depend on `web-sys` or any
//! browser API. The public surface ([`Params`], [`RoutePattern`]) is frozen by
//! `contracts/ferric_router_api.rs`; the internal representation below is owner latitude.
//!
//! # Matching model
//!
//! A pattern and a path are each split on `/` into segments, with empty segments discarded. This
//! gives three useful properties for free:
//!
//! - the root `/` is the empty segment list, so `RoutePattern::new("/").match_path("/")` matches;
//! - a trailing slash is insignificant (`/about/` matches the pattern `/about`);
//! - matching is **exact over segments** — a pattern never matches a path with extra or missing
//!   segments. A `:name` segment is a wildcard that binds the corresponding path segment to `name`.
//!
//! ```
//! use ferric_router::matcher::RoutePattern;
//!
//! let pattern = RoutePattern::new("/user/:id");
//! let params = pattern.match_path("/user/42").expect("should match");
//! assert_eq!(params.get("id"), Some("42"));
//! assert!(pattern.match_path("/user").is_none()); // missing segment
//! assert!(pattern.match_path("/user/42/extra").is_none()); // extra segment
//! ```

/// Named path parameters extracted from a matched route (e.g. pattern `/user/:id` against
/// `/user/42` yields `id = "42"`).
///
/// Parameters are stored in the order their `:name` segments appear in the pattern. Lookups are a
/// linear scan — route patterns have a handful of params in practice, so this is both the simplest
/// and the fastest representation (no hashing, no allocation per lookup).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Params {
    /// Ordered `(name, value)` pairs, one per named segment in the matched pattern.
    pairs: Vec<(String, String)>,
}

impl Params {
    /// The value bound to parameter `name`, if the matched pattern declared it.
    ///
    /// Returns `None` when no parameter with that name was declared by the pattern.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    /// True when the matched pattern declared no parameters.
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// The number of bound parameters.
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// Iterate over the bound `(name, value)` pairs in pattern order.
    ///
    /// Additive convenience beyond the frozen surface: lets consumers enumerate every captured
    /// parameter without knowing their names in advance.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.pairs
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }
}

/// One compiled segment of a route pattern.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    /// A literal segment that must equal the corresponding path segment exactly.
    Static(String),
    /// A `:name` wildcard that matches any single path segment and binds it to `name`.
    Param(String),
}

/// A compiled route pattern. Supports static segments (`/about`), named params (`/user/:id`), and
/// the root (`/`). Matching is exact over segments; a trailing slash is not significant (see the
/// [module docs](self) for the full matching model).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoutePattern {
    /// The pattern's segments, in order, with empty (`//` or leading/trailing `/`) segments removed.
    segments: Vec<Segment>,
}

impl RoutePattern {
    /// Compile a path pattern. `pattern` is a `/`-separated path; a segment beginning `:` is a
    /// named parameter whose name is the remainder (e.g. `:id` binds parameter `id`).
    ///
    /// Empty segments — from a leading, trailing, or doubled `/` — are discarded, so `/`, `""`,
    /// and `//` all compile to the zero-segment (root) pattern.
    pub fn new(pattern: &str) -> Self {
        let segments = split_segments(pattern)
            .map(|segment| match segment.strip_prefix(':') {
                Some(name) => Segment::Param(name.to_owned()),
                None => Segment::Static(segment.to_owned()),
            })
            .collect();
        RoutePattern { segments }
    }

    /// Match a concrete `path` against this pattern. Returns `Some(params)` with any extracted
    /// named parameters on a match, `None` otherwise.
    ///
    /// Matching is exact over segments: the path must have the same number of (non-empty) segments
    /// as the pattern, every static segment must be equal, and each `:name` segment binds the
    /// corresponding path segment to `name`.
    pub fn match_path(&self, path: &str) -> Option<Params> {
        let path_segments: Vec<&str> = split_segments(path).collect();
        if path_segments.len() != self.segments.len() {
            return None;
        }

        let mut pairs = Vec::new();
        for (pattern_segment, path_segment) in self.segments.iter().zip(path_segments) {
            match pattern_segment {
                Segment::Static(expected) => {
                    if expected != path_segment {
                        return None;
                    }
                }
                Segment::Param(name) => {
                    pairs.push((name.clone(), path_segment.to_owned()));
                }
            }
        }

        Some(Params { pairs })
    }
}

/// Split a `/`-separated path (pattern or concrete) into its non-empty segments.
///
/// Filtering empty segments is what makes the root, leading/trailing slashes, and doubled slashes
/// normalize consistently between patterns and paths — the single source of the matching model.
fn split_segments(path: &str) -> impl Iterator<Item = &str> {
    path.split('/').filter(|segment| !segment.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_static_match() {
        let pattern = RoutePattern::new("/about");
        let params = pattern
            .match_path("/about")
            .expect("static path should match");
        assert!(params.is_empty());
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn static_mismatch_returns_none() {
        let pattern = RoutePattern::new("/about");
        assert!(pattern.match_path("/contact").is_none());
    }

    #[test]
    fn param_extraction() {
        // The contract identity `matcher_param_extraction`.
        let pattern = RoutePattern::new("/user/:id");
        let params = pattern
            .match_path("/user/42")
            .expect("param path should match");
        assert_eq!(params.get("id"), Some("42"));
        assert_eq!(params.len(), 1);
        assert!(!params.is_empty());
    }

    #[test]
    fn extra_segment_does_not_match() {
        // `matcher_param_extraction`: `/user/42/extra` is None.
        let pattern = RoutePattern::new("/user/:id");
        assert!(pattern.match_path("/user/42/extra").is_none());
    }

    #[test]
    fn missing_segment_does_not_match() {
        // `matcher_param_extraction`: `/user` is None.
        let pattern = RoutePattern::new("/user/:id");
        assert!(pattern.match_path("/user").is_none());
    }

    #[test]
    fn root_matches_root() {
        // `matcher_param_extraction`: `/` matches `/`.
        let pattern = RoutePattern::new("/");
        let params = pattern.match_path("/").expect("root should match root");
        assert!(params.is_empty());
    }

    #[test]
    fn root_does_not_match_nonroot() {
        let pattern = RoutePattern::new("/");
        assert!(pattern.match_path("/home").is_none());
    }

    #[test]
    fn empty_string_is_the_root_pattern() {
        // "" and "/" both normalize to zero segments, so they are interchangeable.
        assert_eq!(RoutePattern::new(""), RoutePattern::new("/"));
        assert!(RoutePattern::new("").match_path("").is_some());
        assert!(RoutePattern::new("/").match_path("").is_some());
    }

    #[test]
    fn multi_param_pattern() {
        let pattern = RoutePattern::new("/user/:uid/post/:pid");
        let params = pattern
            .match_path("/user/7/post/99")
            .expect("multi-param path should match");
        assert_eq!(params.get("uid"), Some("7"));
        assert_eq!(params.get("pid"), Some("99"));
        assert_eq!(params.len(), 2);
    }

    #[test]
    fn mixed_static_and_param() {
        let pattern = RoutePattern::new("/api/v1/user/:id");
        let params = pattern
            .match_path("/api/v1/user/alice")
            .expect("mixed path should match");
        assert_eq!(params.get("id"), Some("alice"));
        // A path with a wrong static prefix must not match.
        assert!(pattern.match_path("/api/v2/user/alice").is_none());
    }

    #[test]
    fn trailing_slash_is_insignificant() {
        let pattern = RoutePattern::new("/user/:id");
        let with_slash = pattern
            .match_path("/user/42/")
            .expect("trailing slash should match");
        assert_eq!(with_slash.get("id"), Some("42"));

        // Pattern authored with a trailing slash behaves identically.
        let pattern_slash = RoutePattern::new("/about/");
        assert!(pattern_slash.match_path("/about").is_some());
    }

    #[test]
    fn doubled_slashes_are_collapsed() {
        let pattern = RoutePattern::new("/user/:id");
        let params = pattern
            .match_path("//user//42//")
            .expect("doubled slashes should collapse");
        assert_eq!(params.get("id"), Some("42"));
    }

    #[test]
    fn unknown_param_name_returns_none() {
        let pattern = RoutePattern::new("/user/:id");
        let params = pattern.match_path("/user/42").unwrap();
        assert_eq!(params.get("missing"), None);
    }

    #[test]
    fn empty_param_value_cannot_be_produced() {
        // An "empty param" segment (e.g. `/user/` against `/user/:id`) normalizes away, so the
        // pattern simply fails to match rather than binding an empty value.
        let pattern = RoutePattern::new("/user/:id");
        assert!(pattern.match_path("/user/").is_none());
    }

    #[test]
    fn param_values_are_not_decoded_or_altered() {
        // The matcher extracts path segments verbatim; it does not percent-decode.
        let pattern = RoutePattern::new("/file/:name");
        let params = pattern.match_path("/file/a%20b").unwrap();
        assert_eq!(params.get("name"), Some("a%20b"));
    }

    #[test]
    fn params_iter_yields_pairs_in_order() {
        let pattern = RoutePattern::new("/:a/:b/:c");
        let params = pattern.match_path("/1/2/3").unwrap();
        let collected: Vec<(&str, &str)> = params.iter().collect();
        assert_eq!(collected, vec![("a", "1"), ("b", "2"), ("c", "3")]);
    }

    #[test]
    fn same_param_name_segment_count_is_exact() {
        // A single-segment pattern must not match a two-segment path even when the first matches.
        let pattern = RoutePattern::new("/:slug");
        assert!(pattern.match_path("/a").is_some());
        assert!(pattern.match_path("/a/b").is_none());
    }
}
