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
//! **object-safe** (via `async_trait`) so the transport layer can hold an
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
//! [`login`](ChirpRepository::login) issues an opaque [`SessionToken`]
//! for a known handle and [`resolve_token`](ChirpRepository::resolve_token) maps a presented token
//! back to its [`UserId`]. Token storage lives behind the trait so the
//! api-layer's auth extractor can resolve a bearer token without owning any session state.
//!
//! # Release-bar additions (issue #66, REQ-DOM-018)
//!
//! The release bar requires the seam to carry more than today's operations. Those capabilities are
//! added here **additively** — the methods above are unchanged — as new `async fn`s expressed in
//! the same [`chirp_types`] surface and returning the same [`ServerError`] contract, so the trait
//! stays object-safe and the transport layer is untouched. Each entry of the contract's
//! `capability_set.release_bar_additions` maps to one or more methods:
//!
//! | Capability (`capability_set`) | Method(s) |
//! |-------------------------------|-----------|
//! | `credential_set_verify` | [`set_credential`](ChirpRepository::set_credential), [`verify_credential`](ChirpRepository::verify_credential) |
//! | `session_invalidation` | [`invalidate_session`](ChirpRepository::invalidate_session) |
//! | `account_recovery` | [`begin_account_recovery`](ChirpRepository::begin_account_recovery), [`complete_account_recovery`](ChirpRepository::complete_account_recovery) |
//! | `profile_update` | [`update_profile`](ChirpRepository::update_profile) |
//! | `account_deletion_cascade` | [`delete_account`](ChirpRepository::delete_account) |
//! | `data_export` | [`export_user_data`](ChirpRepository::export_user_data) |
//! | `follower_following_listing` | [`list_followers`](ChirpRepository::list_followers), [`list_following`](ChirpRepository::list_following) |
//! | `chirp_removal` | [`remove_chirp`](ChirpRepository::remove_chirp) |
//! | `account_suspension` | [`set_account_suspension`](ChirpRepository::set_account_suspension) |
//! | `store_health` | [`store_health`](ChirpRepository::store_health) |
//!
//! These signatures fix the *surface*; the durable behaviour behind them is implemented downstream
//! (the EPIC-PERSIST durable implementation). The retained [`InMemoryRepository`] satisfies them
//! with error-contract-consistent placeholders (it never falsely confirms an unimplemented durable
//! operation), so the store-free test suite keeps compiling and passing.

pub mod in_memory;

/// The durable PostgreSQL backing (compiling scaffold), behind the optional `postgres` feature.
///
/// Gated so the default build and the store-free test suite pull in neither `sqlx` nor a database;
/// production builds enable `--features postgres` and select it at the composition root when a
/// connection string is configured. See the [`postgres`] module and ADR-0001.
#[cfg(feature = "postgres")]
pub mod postgres;

pub use in_memory::InMemoryRepository;

#[cfg(feature = "postgres")]
pub use postgres::PostgresRepository;

use async_trait::async_trait;
use chirp_types::api::{
    AuthResponse, CreateChirpRequest, CreateUserRequest, FollowResponse, LikeResponse,
    LoginRequest, Page, SessionToken, TimelineQuery,
};
use chirp_types::domain::{Chirp, User, Username};
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
    /// [`SessionToken`]; there is no password/credential step. The
    /// returned [`AuthResponse`] carries the user's profile and the
    /// token to present on subsequent authenticated requests.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if no user has the requested handle.
    async fn login(&self, request: LoginRequest) -> Result<AuthResponse, ServerError>;

    /// Resolves a presented bearer token to the authenticated [`UserId`].
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

    // ---- Release-bar additions (issue #66, REQ-DOM-018 / §4.2) -----------------------------------
    // Added additively; everything above is unchanged. Each method below realizes an entry of the
    // contract's `capability_set.release_bar_additions`. The durable behaviour is implemented
    // downstream; see the module docs for the capability→method map.

    /// Sets (or replaces) the durable credential (e.g. a password) for `user`.
    ///
    /// Part of the `credential_set_verify` capability: credentialed sign-in replaces the demo's
    /// handle-only login. The credential protocol and at-rest representation are owned downstream
    /// (identity & trust, `REQ-ID-*`); this method fixes only the seam.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if `user` does not exist; [`Internal`](ServerError::Internal)
    /// if the durable store cannot record the credential.
    async fn set_credential(&self, user: UserId, secret: &str) -> Result<(), ServerError>;

    /// Verifies a presented `secret` for the account identified by `username`, returning the
    /// authenticated [`UserId`] on success.
    ///
    /// Part of the `credential_set_verify` capability.
    ///
    /// # Errors
    ///
    /// [`Unauthorized`](ServerError::Unauthorized) if the handle is unknown or the secret does not
    /// match the stored credential.
    async fn verify_credential(
        &self,
        username: &Username,
        secret: &str,
    ) -> Result<UserId, ServerError>;

    /// Invalidates a durable session (deliberate sign-out), so the token no longer
    /// [`resolve_token`](ChirpRepository::resolve_token)s.
    ///
    /// Realizes the `session_invalidation` capability. Idempotent: invalidating an already-invalid
    /// token succeeds.
    ///
    /// # Errors
    ///
    /// [`Internal`](ServerError::Internal) if the durable store cannot record the invalidation.
    async fn invalidate_session(&self, token: &SessionToken) -> Result<(), ServerError>;

    /// Begins account recovery for the account identified by `username` (issues/delivers a recovery
    /// challenge).
    ///
    /// Part of the `account_recovery` capability. The challenge mechanism (token issuance, delivery,
    /// lifetime) is owned downstream (identity & trust, `REQ-ID-*`).
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if no account has that handle.
    async fn begin_account_recovery(&self, username: &Username) -> Result<(), ServerError>;

    /// Completes account recovery: validates `recovery_token` for `username` and sets `new_secret`
    /// as the account's credential.
    ///
    /// Part of the `account_recovery` capability.
    ///
    /// # Errors
    ///
    /// [`Unauthorized`](ServerError::Unauthorized) if the recovery token is unknown, expired, or does
    /// not match the account.
    async fn complete_account_recovery(
        &self,
        username: &Username,
        recovery_token: &str,
        new_secret: &str,
    ) -> Result<(), ServerError>;

    /// Updates the mutable profile fields (`display_name`, `bio`) of `user` and returns the updated
    /// profile. The handle and id are immutable identity and are never changed here (`REQ-DOM-008`).
    ///
    /// Realizes the `profile_update` capability.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if `user` does not exist.
    async fn update_profile(
        &self,
        user: UserId,
        display_name: String,
        bio: Option<String>,
    ) -> Result<User, ServerError>;

    /// Deletes `user` and cascades removal of their chirps, follow edges, likes, sessions, and the
    /// denormalized counts on the accounts they touched — atomically.
    ///
    /// Realizes the `account_deletion_cascade` capability. The multi-write atomicity guarantee is a
    /// durable-store concern (`REQ-DOM-015`); this method fixes the seam.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if `user` does not exist.
    async fn delete_account(&self, user: UserId) -> Result<(), ServerError>;

    /// Exports all of `user`'s data as a single serialized export document.
    ///
    /// Realizes the `data_export` capability. The concrete export-document shape is a wire-contract
    /// concern settled downstream (`REQ-DOM-010` / `REQ-DOM-017`); the seam returns the materialized
    /// artifact as an opaque serialized string so no new `chirp-types` DTO is introduced here.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if `user` does not exist; [`Internal`](ServerError::Internal)
    /// if the export cannot be materialized.
    async fn export_user_data(&self, user: UserId) -> Result<String, ServerError>;

    /// Returns one page of the accounts following `user`, newest-first, using the same opaque-cursor
    /// paging as [`timeline`](ChirpRepository::timeline).
    ///
    /// Part of the `follower_following_listing` capability.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if `user` does not exist.
    async fn list_followers(
        &self,
        user: UserId,
        query: TimelineQuery,
    ) -> Result<Page<User>, ServerError>;

    /// Returns one page of the accounts `user` follows, newest-first, using the same opaque-cursor
    /// paging as [`timeline`](ChirpRepository::timeline).
    ///
    /// Part of the `follower_following_listing` capability.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if `user` does not exist.
    async fn list_following(
        &self,
        user: UserId,
        query: TimelineQuery,
    ) -> Result<Page<User>, ServerError>;

    /// Removes `chirp` on behalf of `actor` — the chirp's author removing their own post, or a
    /// moderator acting on abuse.
    ///
    /// Realizes the `chirp_removal` capability. The authorization boundary (author vs. moderator) is
    /// fixed in the model (`REQ-DOM-020`) and enforced downstream (`REQ-ID-*`).
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if the chirp does not exist;
    /// [`Forbidden`](ServerError::Forbidden) if `actor` is not permitted to remove it.
    async fn remove_chirp(&self, actor: UserId, chirp: ChirpId) -> Result<(), ServerError>;

    /// Sets (`suspended = true`) or clears (`suspended = false`) the moderation suspension state of
    /// `target`, on behalf of moderator `actor`.
    ///
    /// Realizes the `account_suspension` capability — a moderation action reserved to an
    /// operator/moderator role (`REQ-DOM-011`, `REQ-DOM-020`).
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if `target` does not exist;
    /// [`Forbidden`](ServerError::Forbidden) if `actor` is not a moderator.
    async fn set_account_suspension(
        &self,
        actor: UserId,
        target: UserId,
        suspended: bool,
    ) -> Result<(), ServerError>;

    /// Reports durable-store health: `Ok(())` when the backing store is reachable and able to serve,
    /// an error otherwise.
    ///
    /// Realizes the `store_health` capability, backing the product's store-reachability health signal
    /// (`REQ-DOM-021`).
    ///
    /// # Errors
    ///
    /// [`Internal`](ServerError::Internal) if the backing store is unreachable or unhealthy.
    async fn store_health(&self) -> Result<(), ServerError>;
}
