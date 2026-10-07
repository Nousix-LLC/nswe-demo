//! The persistence seam: the [`ChirpRepository`] trait and its default in-memory backing.
//!
//! Everything the server does to durable state goes through [`ChirpRepository`]. The trait is the
//! single boundary between the transport/service layers and the store, which is what lets the demo
//! run entirely in memory today and swap in a real database later without touching a handler. The
//! default implementation is [`InMemoryRepository`] (see the [`in_memory`] module); a SQLite/`sqlx`
//! backing could be added as a second implementor of this same trait, but is explicitly optional
//! and is never required for `cargo test`.
//!
//! # Why a trait, and why `async`
//!
//! The trait exists to make persistence *replaceable*. Because the realistic replacement is an
//! async database, the seam is async: every operation is an `async fn`. The trait is kept
//! **object-safe** (via [`async_trait`]) so the transport layer can hold an
//! `Arc<dyn ChirpRepository>` as shared axum state rather than threading a generic backend type
//! through every handler. All methods take `&self` (shared, not `&mut`): an implementation manages
//! its own interior mutability, so the repository can live behind a plain `Arc` shared across
//! concurrent requests.
//!
//! # Capability set (the frozen seam)
//!
//! The trait covers exactly the operations the contract's `repository_seam` requires, expressed in
//! terms of the [`chirp_types`] surface:
//!
//! | Operation | Purpose | Notable failure |
//! |-----------|---------|-----------------|
//! | [`create_user`](ChirpRepository::create_user) | register a user | [`Conflict`](crate::error::ServerError::Conflict) on duplicate handle |
//! | [`get_user`](ChirpRepository::get_user) | fetch a profile | [`NotFound`](crate::error::ServerError::NotFound) |
//! | [`login`](ChirpRepository::login) | handle-based session | [`NotFound`](crate::error::ServerError::NotFound) for an unknown handle |
//! | [`resolve_token`](ChirpRepository::resolve_token) | authenticate a bearer token | [`Unauthorized`](crate::error::ServerError::Unauthorized) |
//! | [`create_chirp`](ChirpRepository::create_chirp) | post a chirp | [`NotFound`](crate::error::ServerError::NotFound) for an absent author/reply target |
//! | [`timeline`](ChirpRepository::timeline) | home timeline page | [`NotFound`](crate::error::ServerError::NotFound) for an absent viewer |
//! | [`follow`](ChirpRepository::follow) / [`unfollow`](ChirpRepository::unfollow) | follow graph | [`Forbidden`](crate::error::ServerError::Forbidden) on self-follow |
//! | [`like`](ChirpRepository::like) / [`unlike`](ChirpRepository::unlike) | likes | [`NotFound`](crate::error::ServerError::NotFound) for an absent chirp |
//!
//! Authentication is deliberately light for the demo (see the contract's `auth_note`):
//! [`login`](ChirpRepository::login) issues an opaque [`SessionToken`](chirp_types::api::SessionToken)
//! for a known handle and [`resolve_token`](ChirpRepository::resolve_token) maps a presented token
//! back to its [`UserId`](chirp_types::ids::UserId). Token storage lives behind the trait so the
//! api-layer's auth extractor can resolve a bearer token without owning any session state.

pub mod in_memory;

pub use in_memory::InMemoryRepository;

use async_trait::async_trait;
use chirp_types::api::{
    AuthResponse, CreateChirpRequest, CreateUserRequest, FollowResponse, LikeResponse,
    LoginRequest, Page, SessionToken, TimelineQuery,
};
use chirp_types::domain::{Chirp, User};
use chirp_types::ids::{ChirpId, UserId};

use crate::error::ServerError;

/// The default maximum number of chirps a timeline page returns when the request omits a limit.
pub const DEFAULT_TIMELINE_LIMIT: u32 = 20;

/// The hard ceiling on a timeline page size. A requested limit above this is clamped down to it, so
/// a client cannot force an unbounded page.
pub const MAX_TIMELINE_LIMIT: u32 = 100;

/// Abstracts all durable state for the chirp server behind one replaceable boundary.
///
/// Every method returns `Result<_, ServerError>`; the [`ServerError`] variant encodes the failure
/// category (which the transport layer maps to an HTTP status). Implementations MUST uphold the
/// domain invariants the contract fixes:
///
/// * **Denormalized counts stay consistent** — `User::follower_count` / `User::following_count` and
///   `Chirp::like_count` always reflect the current edge sets.
/// * **No self-follow** — a user cannot follow themselves ([`Forbidden`](ServerError::Forbidden)).
/// * **At most one like per `(user, chirp)`** — [`like`](Self::like) / [`unlike`](Self::unlike) are
///   idempotent.
/// * **Timelines are reverse-chronological** — newest chirp first.
///
/// The trait is object-safe (`Arc<dyn ChirpRepository>`) and `Send + Sync` so it can be shared as
/// axum application state across concurrent requests.
#[async_trait]
pub trait ChirpRepository: Send + Sync {
    /// Registers a new user, assigning the id, creation timestamp, and zeroed follower/following
    /// counts.
    ///
    /// # Errors
    ///
    /// [`Conflict`](ServerError::Conflict) if the requested username is already taken.
    async fn create_user(&self, request: CreateUserRequest) -> Result<User, ServerError>;

    /// Fetches a user's public profile by id.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if no user has that id.
    async fn get_user(&self, id: UserId) -> Result<User, ServerError>;

    /// Authenticates by handle and issues a session.
    ///
    /// Authentication is handle-based for this demo: a known username is exchanged for an opaque
    /// [`SessionToken`](chirp_types::api::SessionToken); there is no password/credential step. The
    /// returned [`AuthResponse`](chirp_types::api::AuthResponse) carries the user's profile and the
    /// token to present on subsequent authenticated requests.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if no user has the requested handle.
    async fn login(&self, request: LoginRequest) -> Result<AuthResponse, ServerError>;

    /// Resolves a presented bearer token to the authenticated [`UserId`](chirp_types::ids::UserId).
    ///
    /// Supports the api-layer's authentication extractor: the session store lives behind the trait,
    /// so the transport layer resolves a token without owning session state.
    ///
    /// # Errors
    ///
    /// [`Unauthorized`](ServerError::Unauthorized) if the token is unknown or no longer valid.
    async fn resolve_token(&self, token: &SessionToken) -> Result<UserId, ServerError>;

    /// Posts a new chirp authored by `author`, assigning the id, timestamp, and a zeroed like count.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if the author does not exist, or if the request replies
    /// to a chirp id that does not exist.
    async fn create_chirp(
        &self,
        author: UserId,
        request: CreateChirpRequest,
    ) -> Result<Chirp, ServerError>;

    /// Returns one reverse-chronological page of `viewer`'s home timeline (chirps authored by the
    /// viewer or by anyone the viewer follows).
    ///
    /// The page size is `query.limit` clamped to `1..=`[`MAX_TIMELINE_LIMIT`], defaulting to
    /// [`DEFAULT_TIMELINE_LIMIT`] when absent. `query.cursor`, when present, is the continuation
    /// token from a previous page's [`next_cursor`](chirp_types::api::Page::next_cursor); only
    /// chirps older than the cursor are returned. The returned
    /// [`next_cursor`](chirp_types::api::Page::next_cursor) is `Some` iff more chirps remain.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if the viewer does not exist.
    async fn timeline(
        &self,
        viewer: UserId,
        query: TimelineQuery,
    ) -> Result<Page<Chirp>, ServerError>;

    /// Makes `follower` follow `followee`. Idempotent: following an already-followed user succeeds
    /// and leaves the counts unchanged.
    ///
    /// # Errors
    ///
    /// [`Forbidden`](ServerError::Forbidden) if `follower == followee` (no self-follow);
    /// [`NotFound`](ServerError::NotFound) if either user does not exist.
    async fn follow(
        &self,
        follower: UserId,
        followee: UserId,
    ) -> Result<FollowResponse, ServerError>;

    /// Makes `follower` stop following `followee`. Idempotent: unfollowing a user who is not
    /// followed succeeds and leaves the counts unchanged.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if either user does not exist.
    async fn unfollow(
        &self,
        follower: UserId,
        followee: UserId,
    ) -> Result<FollowResponse, ServerError>;

    /// Records that `user` likes `chirp`. Idempotent: liking an already-liked chirp succeeds and
    /// leaves the like count unchanged.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if the chirp (or the user) does not exist.
    async fn like(&self, user: UserId, chirp: ChirpId) -> Result<LikeResponse, ServerError>;

    /// Removes `user`'s like of `chirp`. Idempotent: unliking a chirp the user has not liked
    /// succeeds and leaves the like count unchanged.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if the chirp (or the user) does not exist.
    async fn unlike(&self, user: UserId, chirp: ChirpId) -> Result<LikeResponse, ServerError>;
}
