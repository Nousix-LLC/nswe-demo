//! The axum handler functions for the full `repository_seam` endpoint surface.
//!
//! Each handler is a thin transport adapter: it translates the HTTP request into the
//! [`chirp_types`] DTOs the repository speaks, `.await`s exactly one
//! [`ChirpRepository`](crate::repository::ChirpRepository) operation through the shared
//! [`AppState`], and renders the result as JSON. No handler re-implements persistence, re-declares a
//! wire type, or bypasses the repository — the layering is transport ⇄ DTO ⇄ repository, nothing
//! more.
//!
//! # Error path
//!
//! Every handler returns `Result<_, ServerError>`. The `?` operator propagates any
//! [`ServerError`] the repository (or an extractor) produces, and the committed
//! `impl IntoResponse for ServerError` (in [`crate::error`]) renders it as the contract's
//! `ApiError` JSON body with the matching HTTP status. A `chirp_types` `ValidationError` surfaced
//! while decoding a DTO becomes `ServerError::Validation` (HTTP 400) via the same path.
//!
//! # Authentication
//!
//! Handlers that act on behalf of a caller take the authenticated [`UserId`] from the
//! [`AuthUser`] extractor (a missing/invalid `Authorization: Bearer` token short-circuits with
//! HTTP 401 before the handler body runs). The unauthenticated handlers are
//! [`create_user`], [`get_user`], and [`login`].
//!
//! # Route metadata
//!
//! These functions are mounted by the router-and-bootstrap spoke; this module does not build the
//! router. The authoritative method/path/status table lives in `HANDLERS_NOTES.md`. In summary:
//!
//! | Handler | Method & path | Auth | Success |
//! |---------|---------------|------|---------|
//! | [`create_user`] | `POST /api/users` | no | 201 |
//! | [`get_user`] | `GET /api/users/{id}` | no | 200 |
//! | [`login`] | `POST /api/sessions` | no | 200 |
//! | [`create_chirp`] | `POST /api/chirps` | yes | 201 |
//! | [`timeline`] | `GET /api/timeline` | yes | 200 |
//! | [`follow`] | `PUT /api/users/{id}/follow` | yes | 200 |
//! | [`unfollow`] | `DELETE /api/users/{id}/follow` | yes | 200 |
//! | [`like`] | `PUT /api/chirps/{id}/like` | yes | 200 |
//! | [`unlike`] | `DELETE /api/chirps/{id}/like` | yes | 200 |

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chirp_types::api::{
    AuthResponse, CreateChirpRequest, CreateUserRequest, FollowResponse, LikeResponse,
    LoginRequest, Page, TimelineQuery,
};
use chirp_types::domain::{Chirp, User};
use chirp_types::ids::{ChirpId, UserId};

use crate::{AppState, AuthUser, ServerError};

/// `POST /api/users` — register a new user. Unauthenticated.
///
/// Decodes a [`CreateUserRequest`] body and delegates to
/// [`ChirpRepository::create_user`](crate::repository::ChirpRepository::create_user). Returns
/// **201 Created** with the server-assigned [`User`]. A duplicate username surfaces as
/// `ServerError::Conflict` (HTTP 409); an invalid handle in the body fails DTO validation as
/// `ServerError::Validation` (HTTP 400).
pub async fn create_user(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), ServerError> {
    let user = state.repo.create_user(request).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

/// `GET /api/users/{id}` — fetch a user's public profile by id. Unauthenticated.
///
/// Delegates to [`ChirpRepository::get_user`](crate::repository::ChirpRepository::get_user) and
/// returns **200 OK** with the [`User`]. An unknown id surfaces as `ServerError::NotFound`
/// (HTTP 404).
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<UserId>,
) -> Result<Json<User>, ServerError> {
    let user = state.repo.get_user(id).await?;
    Ok(Json(user))
}

/// `POST /api/sessions` — authenticate by handle and establish a session. Unauthenticated.
///
/// Decodes a [`LoginRequest`] and delegates to
/// [`ChirpRepository::login`](crate::repository::ChirpRepository::login). Returns **200 OK** with
/// an [`AuthResponse`] carrying the user's profile and the bearer [`SessionToken`] to present on
/// subsequent authenticated requests. An unknown handle surfaces as `ServerError::NotFound`
/// (HTTP 404).
///
/// Session establishment returns 200 (not 201): the session is not an addressable resource with a
/// URL of its own — the client receives its token in the body and presents it via the
/// `Authorization: Bearer` header.
///
/// [`SessionToken`]: chirp_types::api::SessionToken
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ServerError> {
    let auth = state.repo.login(request).await?;
    Ok(Json(auth))
}

/// `POST /api/chirps` — post a new chirp as the authenticated caller. Requires authentication.
///
/// Takes the author's [`UserId`] from the [`AuthUser`] extractor, decodes a
/// [`CreateChirpRequest`], and delegates to
/// [`ChirpRepository::create_chirp`](crate::repository::ChirpRepository::create_chirp). Returns
/// **201 Created** with the server-assigned [`Chirp`]. An absent `reply_to` target surfaces as
/// `ServerError::NotFound` (HTTP 404); an over-long body fails DTO validation as
/// `ServerError::Validation` (HTTP 400).
pub async fn create_chirp(
    AuthUser(author): AuthUser,
    State(state): State<AppState>,
    Json(request): Json<CreateChirpRequest>,
) -> Result<(StatusCode, Json<Chirp>), ServerError> {
    let chirp = state.repo.create_chirp(author, request).await?;
    Ok((StatusCode::CREATED, Json(chirp)))
}

/// `GET /api/timeline` — the authenticated caller's home timeline. Requires authentication.
///
/// Takes the viewer's [`UserId`] from the [`AuthUser`] extractor and the [`TimelineQuery`]
/// (`limit`, `cursor`) from the query string, then delegates to
/// [`ChirpRepository::timeline`](crate::repository::ChirpRepository::timeline). Returns **200 OK**
/// with a [`Page`]`<`[`Chirp`]`>`. This is the *home* timeline (the viewer's own chirps plus those
/// of everyone they follow), newest-first; the repository clamps `limit` and interprets the opaque
/// `cursor`, so the handler passes the query through unchanged and echoes
/// [`Page::next_cursor`](chirp_types::api::Page::next_cursor) back to the client.
pub async fn timeline(
    AuthUser(viewer): AuthUser,
    State(state): State<AppState>,
    Query(query): Query<TimelineQuery>,
) -> Result<Json<Page<Chirp>>, ServerError> {
    let page = state.repo.timeline(viewer, query).await?;
    Ok(Json(page))
}

/// `PUT /api/users/{id}/follow` — follow the target user. Requires authentication; idempotent.
///
/// The follower is the authenticated [`AuthUser`]; the followee is the path [`UserId`]. Delegates
/// to [`ChirpRepository::follow`](crate::repository::ChirpRepository::follow) and returns
/// **200 OK** with the resulting [`FollowResponse`]. Following an already-followed user succeeds
/// unchanged (idempotent, hence `PUT`). A self-follow surfaces as `ServerError::Forbidden`
/// (HTTP 403); an absent target as `ServerError::NotFound` (HTTP 404).
pub async fn follow(
    AuthUser(follower): AuthUser,
    State(state): State<AppState>,
    Path(followee): Path<UserId>,
) -> Result<Json<FollowResponse>, ServerError> {
    let response = state.repo.follow(follower, followee).await?;
    Ok(Json(response))
}

/// `DELETE /api/users/{id}/follow` — stop following the target user. Requires authentication;
/// idempotent.
///
/// The follower is the authenticated [`AuthUser`]; the followee is the path [`UserId`]. Delegates
/// to [`ChirpRepository::unfollow`](crate::repository::ChirpRepository::unfollow) and returns
/// **200 OK** with the resulting [`FollowResponse`]. Unfollowing a user who is not followed
/// succeeds unchanged (idempotent); an absent target surfaces as `ServerError::NotFound`
/// (HTTP 404).
pub async fn unfollow(
    AuthUser(follower): AuthUser,
    State(state): State<AppState>,
    Path(followee): Path<UserId>,
) -> Result<Json<FollowResponse>, ServerError> {
    let response = state.repo.unfollow(follower, followee).await?;
    Ok(Json(response))
}

/// `PUT /api/chirps/{id}/like` — like the target chirp. Requires authentication; idempotent.
///
/// The liker is the authenticated [`AuthUser`]; the chirp is the path [`ChirpId`]. Delegates to
/// [`ChirpRepository::like`](crate::repository::ChirpRepository::like) and returns **200 OK** with
/// the resulting [`LikeResponse`]. Liking an already-liked chirp succeeds unchanged (idempotent,
/// hence `PUT`); an absent chirp surfaces as `ServerError::NotFound` (HTTP 404).
pub async fn like(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(chirp): Path<ChirpId>,
) -> Result<Json<LikeResponse>, ServerError> {
    let response = state.repo.like(user, chirp).await?;
    Ok(Json(response))
}

/// `DELETE /api/chirps/{id}/like` — remove a like from the target chirp. Requires authentication;
/// idempotent.
///
/// The liker is the authenticated [`AuthUser`]; the chirp is the path [`ChirpId`]. Delegates to
/// [`ChirpRepository::unlike`](crate::repository::ChirpRepository::unlike) and returns **200 OK**
/// with the resulting [`LikeResponse`]. Unliking a chirp the caller has not liked succeeds
/// unchanged (idempotent); an absent chirp surfaces as `ServerError::NotFound` (HTTP 404).
pub async fn unlike(
    AuthUser(user): AuthUser,
    State(state): State<AppState>,
    Path(chirp): Path<ChirpId>,
) -> Result<Json<LikeResponse>, ServerError> {
    let response = state.repo.unlike(user, chirp).await?;
    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    //! Thin handler-level unit tests: they call each handler directly against a real
    //! [`InMemoryRepository`] (no HTTP stack) to prove the delegation, the success status codes, and
    //! the error mapping. End-to-end routing is exercised by the integration-tests spoke.

    use std::sync::Arc;

    // `super::*` re-provides everything the handler module imports — the axum extractors
    // (`State`/`Path`/`Query`/`Json`), `StatusCode`, the `chirp-types` DTOs + ids, and
    // `AppState`/`AuthUser`/`ServerError` — plus the handler fns under test. Only the test-only
    // extras are named explicitly here.
    use super::*;
    use crate::repository::InMemoryRepository;
    use chirp_types::domain::ChirpText;

    fn state() -> AppState {
        AppState::new(Arc::new(InMemoryRepository::new()))
    }

    fn new_user_request(handle: &str) -> CreateUserRequest {
        CreateUserRequest {
            username: handle.parse().expect("valid handle"),
            display_name: handle.to_owned(),
            bio: None,
        }
    }

    #[tokio::test]
    async fn create_user_returns_201_and_delegates() {
        let state = state();
        let (status, Json(user)) =
            create_user(State(state.clone()), Json(new_user_request("alice")))
                .await
                .expect("create_user succeeds");
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(user.username.as_str(), "alice");

        // The created user is then retrievable through get_user (200).
        let Json(fetched) = get_user(State(state), Path(user.id))
            .await
            .expect("get_user succeeds");
        assert_eq!(fetched, user);
    }

    #[tokio::test]
    async fn create_user_duplicate_is_conflict() {
        let state = state();
        let _ = create_user(State(state.clone()), Json(new_user_request("bob")))
            .await
            .expect("first create succeeds");
        let err = create_user(State(state), Json(new_user_request("bob")))
            .await
            .expect_err("duplicate handle conflicts");
        assert!(matches!(err, ServerError::Conflict(_)));
    }

    #[tokio::test]
    async fn get_user_unknown_is_not_found() {
        let err = get_user(State(state()), Path(UserId::new(9999)))
            .await
            .expect_err("unknown id is not found");
        assert!(matches!(err, ServerError::NotFound(_)));
    }

    #[tokio::test]
    async fn login_then_create_chirp_delegates_with_authenticated_author() {
        let state = state();
        let (_, Json(user)) = create_user(State(state.clone()), Json(new_user_request("carol")))
            .await
            .expect("create_user succeeds");

        // login returns 200 with a token + the same user profile.
        let Json(auth) = login(
            State(state.clone()),
            Json(LoginRequest {
                username: "carol".parse().expect("valid handle"),
            }),
        )
        .await
        .expect("login succeeds");
        assert_eq!(auth.user, user);

        // create_chirp as the authenticated author → 201, author id threaded through.
        let (status, Json(chirp)) = create_chirp(
            AuthUser(user.id),
            State(state.clone()),
            Json(CreateChirpRequest {
                text: ChirpText::parse("hello, chirp!").expect("valid body"),
                reply_to: None,
            }),
        )
        .await
        .expect("create_chirp succeeds");
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(chirp.author_id, user.id);

        // The authored chirp appears on the author's own home timeline.
        let Json(page) = timeline(
            AuthUser(user.id),
            State(state),
            Query(TimelineQuery::default()),
        )
        .await
        .expect("timeline succeeds");
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].id, chirp.id);
    }

    #[tokio::test]
    async fn self_follow_is_forbidden_and_like_absent_is_not_found() {
        let state = state();
        let (_, Json(user)) = create_user(State(state.clone()), Json(new_user_request("dave")))
            .await
            .expect("create_user succeeds");

        let err = follow(AuthUser(user.id), State(state.clone()), Path(user.id))
            .await
            .expect_err("self-follow is forbidden");
        assert!(matches!(err, ServerError::Forbidden(_)));

        let err = like(AuthUser(user.id), State(state), Path(ChirpId::new(4242)))
            .await
            .expect_err("liking an absent chirp is not found");
        assert!(matches!(err, ServerError::NotFound(_)));
    }
}
