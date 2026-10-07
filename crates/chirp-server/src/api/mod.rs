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
//! * [`handlers`] — the axum handler functions for the full `repository_seam` endpoint surface,
//!   each translating HTTP ⇄ DTO and delegating to the repository through [`AppState`].
//!
//! * [`build_router`](router::build_router) — the composition root in [`router`] that mounts the
//!   handlers, `GET /healthz`, and the static/SPA fallback into one [`Router`](axum::Router), with
//!   request tracing. It is the single entry point the binary and the integration tests build the
//!   app through.

pub mod auth;
pub mod handlers;
pub mod router;
pub mod state;

pub use auth::AuthUser;
pub use router::build_router;
pub use state::AppState;
