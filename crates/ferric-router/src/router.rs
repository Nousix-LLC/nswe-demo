//! The stateful routing layer: a declarative route table, the reactive "current path" state, the
//! History API integration (`pushState`/`popstate`), programmatic navigation, and the reactive
//! outlet.
//!
//! This layer composes `ferric` signals for reactivity — it does **not** re-implement reactivity or
//! the DOM diff. A route's view builder is `Fn(&Params) -> VNode`: it receives the matched
//! [`Params`] and returns a `ferric` view tree, and route resolution uses
//! the real [`RoutePattern`] matcher.
//!
//! # The reactive current path
//!
//! "What path is routed now" is a single crate-internal [`Signal<String>`] held in a thread-local.
//! [`current_path`] reads it (subscribing the calling reactive context, exactly like a `ferric`
//! signal read), and [`navigate`] / the `popstate` listener write it. Because the outlet renders
//! inside `ferric`'s render effect, writing the signal re-runs every mounted outlet and patches the
//! DOM — the router stores no DOM state of its own.
//!
//! The signal is created lazily, but always at **builder time** (in [`Router::outlet`],
//! [`Router::mount_history`], or the first [`navigate`]/[`current_path`] call), i.e. *outside* any
//! reactive effect. That matters: a `ferric` node created while an effect is the active owner would
//! be disposed when that effect re-runs. Creating it outside any effect makes it a runtime-root node
//! that lives for the whole application, so repeated re-renders never dispose the routing state.
//!
//! # Native vs. browser
//!
//! Every browser-only call (`window.location`, `history.pushState`, the `popstate` listener) is
//! behind `#[cfg(target_arch = "wasm32")]`, mirroring how `ferric::dom` isolates its browser calls.
//! Off-wasm, `current_location_path` returns `"/"`, `navigate` updates the reactive path without
//! touching the History API, and `mount_history` installs no listener — so the whole crate compiles
//! and unit-tests natively (no browser, no `web-sys` panics), while the full History integration is
//! exercised by the `wasm-bindgen` test and a real browser.

use std::cell::RefCell;

use ferric::component::Component;
use ferric::signal::{create_signal, Signal};
use ferric::vdom::{text, VNode};

use crate::matcher::{Params, RoutePattern};

/// A boxed route view builder: given the matched params, produce a `ferric` view tree. Boxed so the
/// router can hold a heterogeneous table of route handlers; every handler is `'static`.
type ViewFn = Box<dyn Fn(&Params) -> VNode>;

thread_local! {
    /// The single source of truth for the currently-routed path, as a `ferric` signal. `None` until
    /// first use; created lazily by [`path_signal`] outside any reactive effect so it is owned by
    /// the runtime root and never disposed by an outlet re-render.
    static CURRENT_PATH: RefCell<Option<Signal<String>>> = const { RefCell::new(None) };
}

/// Return the current-path signal, creating it (seeded from the current location) on first use.
///
/// The read of the existing handle and the create-and-store path are kept in separate thread-local
/// borrows so no `RefCell` borrow is ever held across `create_signal`.
fn path_signal() -> Signal<String> {
    if let Some(signal) = CURRENT_PATH.with(|cell| *cell.borrow()) {
        return signal;
    }
    let signal = create_signal(current_location_path());
    CURRENT_PATH.with(|cell| *cell.borrow_mut() = Some(signal));
    signal
}

/// Write the reactive current path, notifying every subscribed outlet so it re-renders.
fn set_current_path(path: String) {
    path_signal().set(path);
}

/// The path currently shown by the browser (`window.location.pathname`), or `"/"` when unavailable.
#[cfg(target_arch = "wasm32")]
fn current_location_path() -> String {
    web_sys::window()
        .and_then(|window| window.location().pathname().ok())
        .filter(|path| !path.is_empty())
        .unwrap_or_else(|| "/".to_string())
}

/// Off-wasm there is no browser location; the reactive path starts at the root.
#[cfg(not(target_arch = "wasm32"))]
fn current_location_path() -> String {
    "/".to_string()
}

/// Install a `popstate` listener that syncs the reactive current path with the browser's history on
/// back/forward. The listener closure is `forget`-ed: a router is mounted once for the application's
/// lifetime, so the listener is intentionally kept alive for that lifetime (the same trade-off
/// `ferric::dom` documents for its event closures).
#[cfg(target_arch = "wasm32")]
fn install_popstate_listener() {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    let Some(window) = web_sys::window() else {
        return;
    };
    let closure = Closure::<dyn FnMut(web_sys::Event)>::new(move |_event: web_sys::Event| {
        set_current_path(current_location_path());
    });
    let _ = window.add_event_listener_with_callback("popstate", closure.as_ref().unchecked_ref());
    closure.forget();
}

/// The client-side router: a declarative route table plus History-API navigation, rendered through
/// a reactive outlet that re-renders when the current path changes.
///
/// Build it fluently, then consume it into a `ferric` component:
///
/// ```no_run
/// use ferric_router::prelude::*;
/// use ferric::prelude::*;
/// use ferric::vdom::text;
///
/// # fn body() -> web_sys::Element { unimplemented!() }
/// let app = Router::new()
///     .route("/", |_params| text("home"))
///     .route("/user/:id", |params| text(format!("user {}", params.get("id").unwrap_or(""))))
///     .fallback(|_params| text("not found"))
///     .mount_history();
/// mount(body(), app.outlet());
/// ```
pub struct Router {
    /// Registered routes, in declaration order; resolution is first-match-wins.
    routes: Vec<(RoutePattern, ViewFn)>,
    /// The no-match view, invoked with empty params when no route matches.
    fallback: Option<ViewFn>,
}

impl Router {
    /// Create an empty router.
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
            fallback: None,
        }
    }

    /// Register a route: `pattern` (see [`RoutePattern::new`](crate::matcher::RoutePattern::new))
    /// mapped to a view builder that receives the matched params and returns a `ferric` [`VNode`].
    /// Builder-style (returns `self`). Routes are resolved in the order they are registered, so
    /// register more specific patterns before more general ones.
    pub fn route(mut self, pattern: &str, view: impl Fn(&Params) -> VNode + 'static) -> Self {
        self.routes
            .push((RoutePattern::new(pattern), Box::new(view)));
        self
    }

    /// Register the fallback (no-match / "404") view, invoked with empty params when no route
    /// matches. Builder-style (returns `self`); a later call replaces an earlier fallback.
    pub fn fallback(mut self, view: impl Fn(&Params) -> VNode + 'static) -> Self {
        self.fallback = Some(Box::new(view));
        self
    }

    /// Seed the current path from `window.location` and install the `popstate` listener so browser
    /// back/forward updates the reactive current path. Call once at startup. Builder-style.
    ///
    /// Off-wasm this only ensures the reactive current-path signal exists (seeded to `"/"`) and
    /// installs no listener; there is no browser history to integrate with.
    pub fn mount_history(self) -> Self {
        // Create the current-path signal now, at builder time (outside any reactive effect).
        let _ = path_signal();
        #[cfg(target_arch = "wasm32")]
        {
            set_current_path(current_location_path());
            install_popstate_listener();
        }
        self
    }

    /// Consume the router into a [`Component`] (the router-outlet) that renders the view of the
    /// currently-matched route and re-renders reactively whenever the current path changes. Mount
    /// it with `ferric::mount(root, router.outlet())`.
    ///
    /// Resolution is first-match-wins over the registered routes; on no match the `fallback` (if
    /// any) is invoked with empty params, otherwise an empty node is rendered. The returned
    /// component reads [`current_path`] inside its render, so `ferric`'s render effect re-runs it on
    /// every navigation.
    pub fn outlet(self) -> impl Component + 'static {
        // Force the current-path signal into existence here, at builder time (outside any reactive
        // effect), so it is a runtime-root node and the outlet's own render effect never owns — and
        // therefore never disposes — it across re-renders.
        let _ = path_signal();
        let routes = self.routes;
        let fallback = self.fallback;
        move || -> VNode {
            // Reactive read: subscribes this render effect to the current-path signal.
            let path = current_path();
            for (pattern, view) in &routes {
                if let Some(params) = pattern.match_path(&path) {
                    return view(&params);
                }
            }
            match &fallback {
                Some(view) => view(&Params::default()),
                None => text(""),
            }
        }
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

/// Programmatically navigate to `path`: push a new History entry (`pushState`) and update the
/// reactive current path so every mounted outlet re-renders. Safe to call from a `ferric` event
/// handler (which takes no event argument).
///
/// Off-wasm the History API is skipped; only the reactive current path is updated, which keeps the
/// routing logic (navigation → resolution → view) exercisable in native unit tests.
pub fn navigate(path: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsValue;
        if let Some(history) = web_sys::window().and_then(|window| window.history().ok()) {
            let _ = history.push_state_with_url(&JsValue::NULL, "", Some(path));
        }
    }
    set_current_path(path.to_string());
}

/// Read the current path reactively (subscribes the calling reactive context, like a `ferric`
/// signal read). Returns the path portion currently routed (e.g. `"/user/42"`).
pub fn current_path() -> String {
    path_signal().get()
}

// =====================================================================================
// Native tests — no browser required. `navigate` updates the reactive path off-wasm without the
// History API, so navigation → first-match resolution → fallback are all natively exercisable.
// Each test navigates before asserting, so they do not depend on the (thread-local) path left by a
// previously-run test on the same harness thread.
// =====================================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigate_updates_current_path() {
        navigate("/about");
        assert_eq!(current_path(), "/about");
        navigate("/user/42");
        assert_eq!(current_path(), "/user/42");
    }

    #[test]
    fn outlet_renders_first_matching_route_with_params() {
        navigate("/user/7");
        let router = Router::new()
            .route("/", |_| text("home"))
            .route("/user/:id", |params| {
                text(format!("user {}", params.get("id").unwrap_or("?")))
            })
            .fallback(|_| text("not found"));
        assert!(matches!(router.outlet().render(), VNode::Text(s) if s == "user 7"));
    }

    #[test]
    fn outlet_resolution_is_first_match_wins() {
        navigate("/x");
        let router = Router::new()
            .route("/:first", |_| text("first"))
            .route("/:second", |_| text("second"));
        // Both patterns match a single segment; the earlier registration wins.
        assert!(matches!(router.outlet().render(), VNode::Text(s) if s == "first"));
    }

    #[test]
    fn outlet_uses_fallback_with_empty_params_on_no_match() {
        navigate("/no/such/route");
        let router = Router::new()
            .route("/home", |_| text("home"))
            .fallback(|params| {
                assert!(params.is_empty(), "fallback receives empty params");
                text("not found")
            });
        assert!(matches!(router.outlet().render(), VNode::Text(s) if s == "not found"));
    }

    #[test]
    fn outlet_renders_empty_node_when_no_match_and_no_fallback() {
        navigate("/unmatched");
        let router = Router::new().route("/home", |_| text("home"));
        assert!(matches!(router.outlet().render(), VNode::Text(s) if s.is_empty()));
    }

    #[test]
    fn default_router_matches_new() {
        navigate("/only");
        let view = Router::default()
            .route("/only", |_| text("ok"))
            .outlet()
            .render();
        assert!(matches!(view, VNode::Text(s) if s == "ok"));
    }
}

// =====================================================================================
// Browser test — compiles only for wasm32 and runs only where a wasm-bindgen test runner and a
// headless browser are available. It exercises the navigate → pushState → current_path path end to
// end in a real History context. It is NOT the crate's gate (the native tests above are).
// =====================================================================================
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::{current_path, navigate};
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn navigate_pushes_history_and_updates_current_path() {
        navigate("/user/99");
        assert_eq!(current_path(), "/user/99");
    }
}
