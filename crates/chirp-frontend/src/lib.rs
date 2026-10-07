//! `chirp-frontend` — the **chirp** Twitter-style client: a client-side WebAssembly single-page
//! app built on the [`ferric`] framework and the [`ferric_router`] router, talking to the chirp
//! REST backend through [`ferric_http`] and the shared [`chirp_types`] wire contract.
//!
//! This file is the crate's module root — the walking-skeleton composition layout. The
//! composition root itself (shared reactive state, the route table, the nav, and the wasm mount
//! entry) lives in [`app`]; the single UI error type lives in [`error`]; the typed REST client
//! lives in [`api`]; and the three route views live in [`timeline`], [`compose`], and
//! [`profile`].
//!
//! The in-crate public surface the feature spokes compose against is frozen in
//! `../contracts/app_api.rs`; this crate realizes those signatures verbatim. During the
//! walking-skeleton phase [`api`], [`timeline`], [`compose`], and [`profile`] are compiling
//! stubs (shape-only, non-panicking) that each owning feature spoke replaces.
//!
//! # Building for the browser
//!
//! ```text
//! cargo build -p chirp-frontend --target wasm32-unknown-unknown
//! cargo build -p chirp-frontend --example chirp --target wasm32-unknown-unknown
//! ```
//!
//! See `README.md` for generating the JS bindings and serving `index.html`.
//!
//! [`ferric`]: ferric
//! [`ferric_router`]: ferric_router
//! [`ferric_http`]: ferric_http
//! [`chirp_types`]: chirp_types

/// Typed async REST client over `ferric-http` + the `chirp-types` DTOs (owner: `SUBTASK_api_client`).
pub mod api;
/// Composition root: shared reactive state, the router route table, the nav, and the wasm mount entry.
pub mod app;
/// `/compose` — handle-based login and the validated compose-chirp form (owner: `SUBTASK_compose`).
pub mod compose;
/// The single UI error type ([`error::AppError`]) every failure funnels through.
pub mod error;
/// `/user/:id` — the user profile view with follow/like actions (owner: `SUBTASK_profile`).
pub mod profile;
/// `/` — the timeline view (owner: `SUBTASK_timeline`).
pub mod timeline;

/// The wasm entry point. Re-exported from [`app::start`] for a stable `chirp_frontend::start()`
/// call site (see `examples/chirp.rs`); builds the context, router, and nav and mounts the app.
pub use app::start;
