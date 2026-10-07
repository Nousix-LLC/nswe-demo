//! The shared application state every axum handler and the router builder hold.
//!
//! [`AppState`] is the single value axum threads through the request lifecycle as
//! [`State`](axum::extract::State). It carries the persistence seam —
//! [`ChirpRepository`] — as a trait object behind an
//! [`Arc`], so the transport layer depends only on the repository *capability* and never on a
//! concrete backend (the default [`InMemoryRepository`](crate::repository::InMemoryRepository),
//! or a future database implementor) — swapping the backing store changes no handler.
//!
//! The type is cheap to [`Clone`] (an `Arc` bump), which is what axum requires of application
//! state: the router clones it per request, and the [`FromRef`](axum::extract::FromRef) blanket
//! impl makes it directly usable as the extractor state for
//! [`AuthUser`](crate::api::auth::AuthUser).

use std::sync::Arc;

use crate::repository::ChirpRepository;

/// The transport-layer application state shared across every request.
///
/// Holds the persistence backend behind the [`ChirpRepository`] trait object so handlers
/// `.await` the repository without owning or naming the concrete store. Constructed once at
/// startup (and once per test) via [`AppState::new`] and handed to the router as axum
/// [`State`](axum::extract::State).
#[derive(Clone)]
pub struct AppState {
    /// The persistence seam the server is backed by, shared across concurrent requests.
    ///
    /// All durable state flows through this trait object; the default implementor is the
    /// in-memory repository, so the server boots with no database.
    pub repo: Arc<dyn ChirpRepository>,
}

impl AppState {
    /// Builds the shared state from any [`ChirpRepository`] implementation.
    ///
    /// The caller supplies the backing store (typically
    /// `Arc::new(InMemoryRepository::new())`); the server and its tests construct state the same
    /// way, so both exercise the identical transport stack.
    #[must_use]
    pub fn new(repo: Arc<dyn ChirpRepository>) -> Self {
        Self { repo }
    }
}
