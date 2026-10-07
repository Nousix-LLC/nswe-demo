//! The `Authorization: Bearer` authentication extractor.
//!
//! [`AuthUser`] is an axum [`FromRequestParts`] extractor that reads the `Authorization: Bearer
//! <token>` header, resolves the presented [`SessionToken`] to a [`UserId`] through the
//! repository's [`resolve_token`](crate::repository::ChirpRepository::resolve_token), and yields
//! the authenticated id. A handler declares authentication simply by naming `AuthUser` in its
//! argument list:
//!
//! ```ignore
//! async fn create_chirp(
//!     AuthUser(author): AuthUser,
//!     State(state): State<AppState>,
//!     Json(req): Json<CreateChirpRequest>,
//! ) -> Result<Json<Chirp>, ServerError> { /* ... */ }
//! ```
//!
//! # How it reaches [`AppState`]
//!
//! The extractor is generic over the router state `S` with the bound `AppState: FromRef<S>`, so
//! it works against any state from which the [`AppState`] can be projected — including `S =
//! AppState` itself via axum's blanket `FromRef` impl. It therefore owns no session state of its
//! own; the session store lives behind the repository trait.
//!
//! # Failure
//!
//! Any missing, malformed, or unresolvable credential surfaces as
//! [`ServerError::Unauthorized`], which the [`IntoResponse`](axum::response::IntoResponse) impl in
//! [`crate::error`] renders as HTTP `401`. A valid header whose token the repository rejects
//! propagates that repository's [`Unauthorized`](ServerError::Unauthorized) unchanged.

use axum::extract::{FromRef, FromRequestParts};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use chirp_types::api::SessionToken;
use chirp_types::ids::UserId;

use crate::api::state::AppState;
use crate::error::ServerError;

/// An authenticated caller, extracted from a request's `Authorization: Bearer` header.
///
/// Yields the [`UserId`] the presented session token resolved to. Present as a handler argument,
/// it both enforces authentication (a missing/invalid token short-circuits the handler with a
/// `401`) and provides the caller's id to the handler body.
pub struct AuthUser(pub UserId);

impl AuthUser {
    /// Returns the authenticated caller's [`UserId`].
    #[must_use]
    pub fn user_id(&self) -> UserId {
        self.0
    }
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ServerError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);

        let header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or_else(|| ServerError::Unauthorized("missing Authorization header".to_owned()))?;

        let raw = header.to_str().map_err(|_| {
            ServerError::Unauthorized("Authorization header is not valid text".to_owned())
        })?;

        let (scheme, token) = raw.split_once(' ').ok_or_else(|| {
            ServerError::Unauthorized("malformed Authorization header".to_owned())
        })?;

        if !scheme.eq_ignore_ascii_case("bearer") {
            return Err(ServerError::Unauthorized(
                "expected a Bearer authorization scheme".to_owned(),
            ));
        }

        let token = token.trim();
        if token.is_empty() {
            return Err(ServerError::Unauthorized("empty bearer token".to_owned()));
        }

        let user_id = state
            .repo
            .resolve_token(&SessionToken(token.to_owned()))
            .await?;

        Ok(AuthUser(user_id))
    }
}
