//! The axum transport layer: shared state, extractors, handlers, and the router.
//!
//! This module is the HTTP edge of `chirp-server`. It currently provides the transport
//! **foundations** every handler composes against:
//!
//! * [`AppState`] — the shared [`State`](axum::extract::State) carrying the
//!   [`ChirpRepository`](crate::repository::ChirpRepository) behind an `Arc`.
//! * [`AuthUser`] — the `Authorization: Bearer` extractor yielding the authenticated
//!   [`UserId`](chirp_types::ids::UserId).
//! * the `IntoResponse` rendering of [`ServerError`](crate::error::ServerError), which lives
//!   alongside the error enum in [`crate::error`] (so `error.rs` owns the full error seam) and
//!   lets a handler return `Result<_, ServerError>`.
//!
//! The axum handler functions, the `build_router` builder (static/SPA serving, `/healthz`,
//! tracing), and the `main.rs` binary are added by the later api-layer sub-spokes; they slot into
//! this module and are mounted through [`AppState`].

pub mod auth;
pub mod state;

pub use auth::AuthUser;
pub use state::AppState;
