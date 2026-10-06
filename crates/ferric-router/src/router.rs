//! The stateful routing layer: a declarative route table, the reactive "current path" state, the
//! History API integration (`pushState`/`popstate`), programmatic navigation, and the reactive
//! outlet.
//!
//! This layer composes `ferric` signals for reactivity — it MUST NOT re-implement reactivity or
//! the DOM diff. A route's view builder is `Fn(&Params) -> VNode`: it receives the matched params
//! and returns a `ferric` view tree. **Walking-skeleton stub:** the public surface below is frozen
//! by `contracts/ferric_router_api.rs`; the bodies are `todo!()` placeholders filled in by the
//! `navigation` work-item.

use ferric::component::Component;
use ferric::vdom::VNode;

use crate::matcher::Params;

/// The client-side router: a declarative route table plus History-API navigation, rendered through
/// a reactive outlet that re-renders when the current path changes.
pub struct Router {
    // Owner-designed representation (filled in by the `navigation` spoke): route table + fallback
    // + reactive current-path handle.
    _private: (),
}

impl Router {
    /// Create an empty router.
    pub fn new() -> Self {
        todo!()
    }

    /// Register a route: `pattern` (see [`RoutePattern::new`](crate::matcher::RoutePattern::new))
    /// mapped to a view builder that receives the matched params and returns a `ferric` [`VNode`].
    /// Builder-style (returns `self`).
    pub fn route(self, pattern: &str, view: impl Fn(&Params) -> VNode + 'static) -> Self {
        let _ = (pattern, view);
        todo!()
    }

    /// Register the fallback (no-match / "404") view, invoked with empty params when no route
    /// matches.
    pub fn fallback(self, view: impl Fn(&Params) -> VNode + 'static) -> Self {
        let _ = view;
        todo!()
    }

    /// Seed the current path from `window.location` and install the `popstate` listener so browser
    /// back/forward updates the reactive current path. Call once at startup. Builder-style.
    pub fn mount_history(self) -> Self {
        todo!()
    }

    /// Consume the router into a [`Component`] (the router-outlet) that renders the view of the
    /// currently-matched route and re-renders reactively whenever the current path changes. Mount
    /// it with `ferric::mount(root, router.outlet())`.
    pub fn outlet(self) -> impl Component + 'static {
        // A function component — any `Fn() -> VNode` is a `ferric::Component`. The `navigation`
        // spoke replaces the body with the reactive matched-route render.
        move || -> VNode { todo!() }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

/// Programmatically navigate to `path`: push a new History entry (`pushState`) and update the
/// reactive current path so every mounted outlet re-renders. Safe to call from an event handler.
pub fn navigate(path: &str) {
    let _ = path;
    todo!()
}

/// Read the current path reactively (subscribes the calling reactive context, like a `ferric`
/// signal read). Returns the path portion currently routed (e.g. `"/user/42"`).
pub fn current_path() -> String {
    todo!()
}
