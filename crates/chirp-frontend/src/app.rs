//! The composition root: shared reactive state, the router route table, the persistent nav, and
//! the wasm mount entry.
//!
//! This module (owned by `SUBTASK_scaffold`) realizes the frozen `contracts/app_api.rs` surface:
//! [`Session`], [`AppContext`] (with [`AppContext::new`] and [`AppContext::client`]), and the
//! [`start`] entry point that assembles the [`Router`] route table (`/` → timeline, `/compose` →
//! compose, `/user/:id` → profile, plus a fallback), renders a persistent nav above the reactive
//! outlet, and mounts the application into `#app` (falling back to `<body>`).
//!
//! The three route views ([`timeline_view`](crate::timeline::timeline_view),
//! [`compose_view`](crate::compose::compose_view),
//! [`profile_view`](crate::profile::profile_view)) are pre-mounted here behind the frozen seam;
//! each owning feature spoke fills in its module body without changing this wiring.

use chirp_types::prelude::{SessionToken, User};
use ferric::prelude::*;
use ferric::vdom::text;
use ferric_router::prelude::*;

use crate::api::ApiClient;
use crate::compose::compose_view;
use crate::profile::profile_view;
use crate::timeline::timeline_view;

/// The default REST base URL: a relative base, so the SPA calls the same origin that served it.
/// See `contracts/rest_endpoints.md`.
const DEFAULT_API_BASE_URL: &str = "/api";

/// The authenticated session: the current user and their bearer token. A `None` session means
/// "logged out". Set by the compose/login flow; read by any view that needs auth.
#[derive(Clone)]
pub struct Session {
    /// The authenticated user's profile.
    pub user: User,
    /// The bearer token presented on authenticated requests.
    pub token: SessionToken,
}

/// Shared, cheaply-cloneable application context handed to every route view. Holds the
/// configurable API base URL and the reactive session signal. [`Signal`] is `Copy`, so
/// `AppContext` is `Clone`.
#[derive(Clone)]
pub struct AppContext {
    /// Base URL the [`ApiClient`] targets (default `/api`; configurable — see [`api_base_url`]).
    pub base_url: String,
    /// Reactive current session; writing it (login/logout) re-renders dependent views.
    pub session: Signal<Option<Session>>,
}

impl AppContext {
    /// Build a context with a base URL and an empty (logged-out) session signal.
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        AppContext {
            base_url: base_url.into(),
            session: create_signal(None),
        }
    }

    /// An [`ApiClient`] configured for the current session (adds the bearer token, if any).
    #[must_use]
    pub fn client(&self) -> ApiClient {
        let client = ApiClient::new(self.base_url.clone());
        match self
            .session
            .with(|s| s.as_ref().map(|sess| sess.token.clone()))
        {
            Some(token) => client.with_token(token),
            None => client,
        }
    }
}

/// Resolve the REST base URL, honoring a build-time `CHIRP_API_BASE_URL` override and otherwise
/// using [`DEFAULT_API_BASE_URL`]. Configuration is compile-time so the wasm bundle carries no
/// runtime secret; see `README.md`.
fn api_base_url() -> String {
    match option_env!("CHIRP_API_BASE_URL") {
        Some(url) => url.to_string(),
        None => DEFAULT_API_BASE_URL.to_string(),
    }
}

/// The persistent navigation bar rendered above the reactive outlet.
fn nav_bar() -> VNode {
    VElement::new("nav")
        .attr("class", "chirp-nav")
        .child(link("/", "Timeline"))
        .child(text(" · "))
        .child(link("/compose", "Compose"))
        .into()
}

/// The fallback view rendered when no route matches.
fn not_found_view() -> VNode {
    VElement::new("section")
        .attr("class", "chirp-not-found")
        .child(VElement::new("h1").child(text("Not found")))
        .child(VElement::new("p").child(text("No route matched this path.")))
        .into()
}

/// The wasm entry point: build the [`AppContext`], assemble the [`Router`] route table, render a
/// persistent nav above the reactive outlet, and mount the app into `#app` (falling back to
/// `<body>`).
///
/// This is the **thin wasm mount entry**. Per the subtree constraint it MAY use `expect()` for
/// mount-point acquisition (and only for that) — mirroring the `ferric` / `ferric-router` examples;
/// everything below it (views, `AppContext` logic, the feature modules) surfaces failure as
/// [`AppError`](crate::error::AppError) UI state and never panics.
pub fn start() {
    let ctx = AppContext::new(api_base_url());

    let timeline_ctx = ctx.clone();
    let compose_ctx = ctx.clone();
    let profile_ctx = ctx.clone();

    let router = Router::new()
        .route("/", move |_params| timeline_view(&timeline_ctx))
        .route("/compose", move |_params| compose_view(&compose_ctx))
        .route("/user/:id", move |params| {
            profile_view(&profile_ctx, params)
        })
        .fallback(|_params| not_found_view())
        .mount_history();

    let outlet = router.outlet();

    // One root component: the persistent nav above the reactive outlet. Calling `outlet.render()`
    // here subscribes the mount effect to the current-path signal, so navigation re-renders the
    // routed view in place.
    let app = move || -> VNode {
        VElement::new("div")
            .attr("class", "chirp-app")
            .child(nav_bar())
            .child(outlet.render())
            .into()
    };

    // Mount into `#app` if the page provides it, otherwise into `<body>`.
    let document = web_sys::window()
        .expect("chirp-frontend: no browser `window` — the app requires a browser context")
        .document()
        .expect("chirp-frontend: no `document` on `window`");
    let root = document
        .get_element_by_id("app")
        .or_else(|| document.body().map(Into::into))
        .expect("chirp-frontend: the page has no `#app` element and no `<body>` to mount into");

    mount(root, app);
}
