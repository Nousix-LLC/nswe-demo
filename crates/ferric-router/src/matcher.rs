//! Pure, browser-free route matching: compile a pattern like `/user/:id`, match a concrete path,
//! and extract named parameters.
//!
//! This module is natively testable with no wasm/DOM and MUST NOT depend on `web-sys` or any
//! browser API. **Walking-skeleton stub:** the public surface below is frozen by
//! `contracts/ferric_router_api.rs`; the bodies are `todo!()` placeholders filled in by the
//! `matcher` work-item.

/// Named path parameters extracted from a matched route (e.g. pattern `/user/:id` against
/// `/user/42` yields `id = "42"`).
pub struct Params {
    // Owner-designed representation (filled in by the `matcher` spoke): ordered name→value pairs.
    _private: (),
}

impl Params {
    /// The value bound to parameter `name`, if the matched pattern declared it.
    pub fn get(&self, name: &str) -> Option<&str> {
        let _ = name;
        todo!()
    }

    /// True when the matched pattern declared no parameters.
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// The number of bound parameters.
    pub fn len(&self) -> usize {
        todo!()
    }
}

/// A compiled route pattern. Supports static segments (`/about`), named params (`/user/:id`), and
/// the root (`/`). Trailing-slash normalization and any wildcard support are owner latitude.
pub struct RoutePattern {
    // Owner-designed representation (filled in by the `matcher` spoke): parsed segments.
    _private: (),
}

impl RoutePattern {
    /// Compile a path pattern. `pattern` is a `/`-separated path; a segment beginning `:` is a
    /// named parameter whose name is the remainder.
    pub fn new(pattern: &str) -> Self {
        let _ = pattern;
        todo!()
    }

    /// Match a concrete `path` against this pattern. Returns `Some(params)` with any extracted
    /// named parameters on a match, `None` otherwise. Matching is exact over segments.
    pub fn match_path(&self, path: &str) -> Option<Params> {
        let _ = path;
        todo!()
    }
}
