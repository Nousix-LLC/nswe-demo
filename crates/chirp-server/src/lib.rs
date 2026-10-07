//! `chirp-server` — the axum REST API backend for **chirp**, the Twitter-style app in this
//! engagement.
//!
//! The crate is a thin, layered server over the shared [`chirp_types`] contract: it imports every
//! wire type (domain entities + API DTOs) from `chirp-types` and never re-declares them, so the
//! server and the frontend cannot drift.
//!
//! # Layering
//!
//! ```text
//! transport (axum handlers, router, middleware)   ← api-layer spoke
//!     │
//!     ▼
//! persistence (ChirpRepository trait)             ← this module tree
//!     │
//!     ▼
//! InMemoryRepository (default backing, no DB)
//! ```
//!
//! Persistence is abstracted behind the [`ChirpRepository`](repository::ChirpRepository) trait so
//! the server runs entirely in memory for the demo (the default [`InMemoryRepository`]) and could
//! adopt a real database later by adding another implementor — without changing a single handler.
//! The transport layer holds the repository as `Arc<dyn ChirpRepository>` shared state.
//!
//! # What this crate exposes
//!
//! * [`error::ServerError`] — the one typed error returned across every layer, with its mapping to
//!   the `chirp-types` wire error contract.
//! * [`repository`] — the [`ChirpRepository`](repository::ChirpRepository) seam and its default
//!   [`InMemoryRepository`] backing.
//!
//! The axum application/router builder and the server binary are the api-layer spoke's: the router
//! builder [`build_router`] is re-exported here so the end-to-end integration tests and the
//! [`main`](../main/index.html) binary construct the server against an [`InMemoryRepository`]
//! through one entry point.

pub mod api;
pub mod error;
pub mod repository;

pub use api::{build_router, AppState, AuthUser, ValidatedJson};
pub use error::ServerError;
pub use repository::{ChirpRepository, InMemoryRepository};

// NOTE: `pub mod api;` and the `AppState` / `AuthUser` re-exports are the transport foundations
// handlers compile against; `build_router` is the router-and-bootstrap composition root. Together
// they give `tests/` and the `main` binary one shared way to construct the app over an
// `InMemoryRepository`.
