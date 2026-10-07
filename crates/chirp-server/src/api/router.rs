//! The axum application router — the composition root that mounts the whole HTTP surface.
//!
//! [`build_router`] assembles one [`Router`] from three concerns and is the single entry point the
//! binary ([`crate::main`]) and the end-to-end integration tests both construct the app through:
//!
//! 1. **The API** — every handler in [`crate::api::handlers`], mounted under `/api/…` over the
//!    shared [`AppState`] (see the authoritative method/path table in `HANDLERS_NOTES.md`).
//! 2. **Liveness** — `GET /healthz`, an unauthenticated health probe that touches no state.
//! 3. **The SPA** — a static-file service rooted at the static directory, with an `index.html`
//!    fallback so client-side routes (anything not matched above) resolve to the single-page app.
//!
//! A `tower-http` [`TraceLayer`] wraps the whole stack so every request is traced (the spans are
//! emitted only when a `tracing` subscriber is installed — the binary installs one in
//! [`crate::main`]; tests may omit it harmlessly).
//!
//! # Route / fallback precedence
//!
//! axum matches the explicit routes first; only unmatched paths reach the static fallback. The API
//! is deliberately namespaced under `/api/…` so it never collides with the static/SPA path space —
//! an unknown `/api/…` path still falls through to the SPA fallback (returning `index.html`), which
//! is the conventional SPA behavior; genuine API 404s come from the handlers themselves (e.g. an
//! unknown user id), not from routing.
//!
//! # Static assets (placeholder)
//!
//! The real SPA bundle is built by a later work-item (#9). Until then the static root holds a
//! committed placeholder `index.html`, so the server boots and serves a page with no frontend
//! build step. The static root is resolved by [`static_dir`]: the `CHIRP_STATIC_DIR` environment
//! variable when set, else the crate-local `static/` directory (resolved at compile time from
//! `CARGO_MANIFEST_DIR`, so it is found regardless of the process working directory).

use std::path::PathBuf;

use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde_json::{json, Value};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::api::handlers;
use crate::AppState;

/// Build the fully-wired application [`Router`] over the supplied [`AppState`].
///
/// Mounts every API handler (`/api/…`), the `GET /healthz` probe, and the static/SPA fallback
/// service, then wraps the result in a request [`TraceLayer`] and binds the state. The returned
/// `Router` is state-erased (`Router<()>`), ready to hand to [`axum::serve`] or to an in-process
/// test harness.
///
/// This is re-exported at the crate root as [`chirp_server::build_router`](crate::build_router),
/// which is the single way both the binary and the integration tests construct the server, so they
/// exercise the identical transport stack.
pub fn build_router(state: AppState) -> Router {
    let static_dir = static_dir();
    // SPA fallback: serve files from the static root; for any path with no matching file, fall
    // back to index.html so the single-page app owns client-side routing.
    let spa = ServeDir::new(&static_dir).fallback(ServeFile::new(static_dir.join("index.html")));

    Router::new()
        // Liveness probe — unauthenticated, state-free.
        .route("/healthz", get(healthz))
        // --- API surface (see HANDLERS_NOTES.md for the authoritative table) ---
        .route("/api/users", post(handlers::create_user))
        .route("/api/users/{id}", get(handlers::get_user))
        .route("/api/sessions", post(handlers::login))
        .route("/api/chirps", post(handlers::create_chirp))
        .route("/api/timeline", get(handlers::timeline))
        .route(
            "/api/users/{id}/follow",
            put(handlers::follow).delete(handlers::unfollow),
        )
        .route(
            "/api/chirps/{id}/like",
            put(handlers::like).delete(handlers::unlike),
        )
        // --- Static assets + SPA fallback for every unmatched path ---
        .fallback_service(spa)
        // --- Cross-cutting: trace every request/response ---
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// `GET /healthz` — an unauthenticated liveness probe.
///
/// Returns **200 OK** with a small JSON body (`status`, `service`, `version`). It touches no
/// application state, so it reports process liveness only (there is no external dependency to check
/// in the in-memory configuration).
async fn healthz() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "chirp-server",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Resolve the static-asset root directory.
///
/// Prefers the `CHIRP_STATIC_DIR` environment variable (for deployments that ship the SPA bundle
/// elsewhere); otherwise falls back to the crate-local `static/` directory, resolved from
/// `CARGO_MANIFEST_DIR` at compile time so it is found regardless of the working directory the
/// server is launched from.
fn static_dir() -> PathBuf {
    std::env::var_os("CHIRP_STATIC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/static")))
}
