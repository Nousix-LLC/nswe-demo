//! Typed async REST client over `ferric-http` + the `chirp-types` DTOs.
//!
//! **Scaffold stub.** This file is the walking-skeleton stub owned (initially) by
//! `SUBTASK_scaffold` and **replaced** by `SUBTASK_api_client` with the real implementation. It
//! realizes the frozen `contracts/app_api.rs` signatures exactly so the feature spokes compile
//! against it, but every request returns an [`AppError`] (never panics, never performs I/O). The
//! method ⇄ endpoint map and real behaviour live in `contracts/rest_endpoints.md` and the owning
//! spoke.

use chirp_types::prelude::*;

use crate::error::AppError;

/// Typed async REST client. Wraps a `ferric_http` client, (de)serializes `chirp_types` DTOs,
/// attaches the bearer token when present, and maps every failure to [`AppError`]. Cheap to clone.
///
/// Each method's REST method+path and DTOs are frozen in `contracts/rest_endpoints.md`.
pub struct ApiClient;

impl ApiClient {
    /// New client for `base_url` with no token (logged out).
    #[must_use]
    pub fn new(_base_url: impl Into<String>) -> Self {
        ApiClient
    }

    /// Same client carrying a bearer token added to each request's `Authorization` header.
    #[must_use]
    pub fn with_token(self, _token: SessionToken) -> Self {
        self
    }

    /// `POST /sessions` — [`LoginRequest`] → [`AuthResponse`].
    pub async fn login(&self, _req: &LoginRequest) -> Result<AuthResponse, AppError> {
        Err(not_implemented())
    }

    /// `POST /users` — [`CreateUserRequest`] → [`User`].
    pub async fn create_user(&self, _req: &CreateUserRequest) -> Result<User, AppError> {
        Err(not_implemented())
    }

    /// `POST /chirps` — [`CreateChirpRequest`] → [`Chirp`].
    pub async fn create_chirp(&self, _req: &CreateChirpRequest) -> Result<Chirp, AppError> {
        Err(not_implemented())
    }

    /// `GET /timeline` — [`TimelineQuery`] (query string) → [`Page<Chirp>`].
    pub async fn timeline(&self, _query: &TimelineQuery) -> Result<Page<Chirp>, AppError> {
        Err(not_implemented())
    }

    /// `GET /users/{id}` → [`User`].
    pub async fn get_user(&self, _id: UserId) -> Result<User, AppError> {
        Err(not_implemented())
    }

    /// `PUT`/`DELETE /users/{id}/follow` (by `follow`) → [`FollowResponse`].
    pub async fn set_follow(&self, _id: UserId, _follow: bool) -> Result<FollowResponse, AppError> {
        Err(not_implemented())
    }

    /// `PUT`/`DELETE /chirps/{id}/like` (by `like`) → [`LikeResponse`].
    pub async fn set_like(&self, _id: ChirpId, _like: bool) -> Result<LikeResponse, AppError> {
        Err(not_implemented())
    }
}

// The uniform stub result: the real client (SUBTASK_api_client) replaces this whole module.
fn not_implemented() -> AppError {
    AppError::Transport(
        "chirp-frontend: ApiClient is a scaffold stub; SUBTASK_api_client provides the real \
         implementation"
            .to_string(),
    )
}
