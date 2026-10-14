//! The default in-memory [`ChirpRepository`] backing.
//!
//! [`InMemoryRepository`] keeps the entire application state in process behind a single
//! [`std::sync::Mutex`], so the server boots and the whole test suite runs with **no database and
//! no external infrastructure**. It is the demo's default backing; a persistent store (SQLite via
//! `sqlx`, etc.) would be a second implementor of [`ChirpRepository`], never a replacement that the
//! tests require.
//!
//! # Concurrency model
//!
//! All mutable state lives in one `Store` behind `Mutex<Store>`. Each trait method is a single
//! synchronous critical section — acquire the lock, read/mutate, drop the lock — with **no `.await`
//! held across the guard. Because the guard never crosses a suspension point, the futures
//! `async_trait` produces remain `Send`, and a plain `std::sync::Mutex` is correct and cheap. A
//! single global lock is deliberately simple for a demo-scale store; a production backing would
//! shard or use finer-grained locking, but the [`ChirpRepository`] seam would be unchanged.
//!
//! # Identifiers, ordering, and time
//!
//! User and chirp ids are assigned from monotonically increasing counters, so a larger id always
//! denotes a later creation. Timelines therefore order by **id descending**, which is a stable
//! proxy for reverse-chronological order that is immune to two chirps sharing a millisecond
//! timestamp. Timestamps themselves are wall-clock epoch-milliseconds and are recorded on each
//! entity for display; ordering never depends on them.

use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use chirp_types::api::{
    AuthResponse, CreateChirpRequest, CreateUserRequest, Cursor, FollowResponse, LikeResponse,
    LoginRequest, Page, SessionToken, TimelineQuery,
};
use chirp_types::domain::{Chirp, User, Username};
use chirp_types::ids::{ChirpId, Timestamp, UserId};

use super::{ChirpRepository, DEFAULT_TIMELINE_LIMIT, MAX_TIMELINE_LIMIT};
use crate::error::ServerError;

/// All mutable server state, guarded as one unit by the repository's [`Mutex`].
///
/// The follow and like relationships are stored as directed edge sets; the denormalized counts on
/// [`User`] and [`Chirp`] are kept in lockstep with those sets by every mutating method, so a read
/// of a count never disagrees with the edges.
#[derive(Debug, Default)]
struct Store {
    /// Users by id.
    users: HashMap<UserId, User>,
    /// Handle → id index, enforcing username uniqueness and backing handle-based login.
    username_to_id: HashMap<String, UserId>,
    /// Chirps by id.
    chirps: HashMap<ChirpId, Chirp>,
    /// Directed follow edges `(follower, followee)`.
    follows: HashSet<(UserId, UserId)>,
    /// Like edges `(user, chirp)`; the set membership enforces at-most-one-like-per-pair.
    likes: HashSet<(UserId, ChirpId)>,
    /// Issued session tokens → the user they authenticate.
    tokens: HashMap<String, UserId>,
    /// Durable credential per user: user id → the at-rest secret representation. One per user,
    /// mirroring the `credentials` table's `user_id` primary key. The secret's exact format and the
    /// verification protocol are owned by identity & trust (`REQ-ID-*`); this backing only holds the
    /// slot and compares what it was given, so the in-memory and durable backings agree.
    credentials: HashMap<UserId, String>,
    /// Outstanding account-recovery challenges: the opaque recovery token → the account it recovers
    /// and when the challenge expires. Mirrors the durable `recovery_tokens` table (token primary
    /// key, `user_id`, `expires_at`); a validated challenge is consumed (removed) so it is single-use.
    recovery_tokens: HashMap<String, RecoveryChallenge>,
    /// Next user id to assign.
    next_user_id: u64,
    /// Next chirp id to assign.
    next_chirp_id: u64,
    /// Monotonic counter making issued token strings unique.
    next_token: u64,
}

/// An in-memory [`ChirpRepository`]: the default, infrastructure-free backing for the chirp server.
///
/// Cheap to [`Clone`]? No — it is meant to be wrapped once in an `Arc` and shared (the api-layer
/// holds `Arc<dyn ChirpRepository>`). Construct one with [`InMemoryRepository::new`].
#[derive(Debug, Default)]
pub struct InMemoryRepository {
    store: Mutex<Store>,
}

impl InMemoryRepository {
    /// Creates an empty repository with no users, chirps, or sessions.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Locks the store, translating lock poisoning into an [`Internal`](ServerError::Internal)
    /// error rather than panicking — a poisoned lock is a server fault, not the caller's.
    fn lock(&self) -> Result<std::sync::MutexGuard<'_, Store>, ServerError> {
        self.store
            .lock()
            .map_err(|_| ServerError::Internal("repository lock poisoned".to_owned()))
    }
}

/// An outstanding in-memory account-recovery challenge: the account it recovers and its expiry,
/// mirroring a `recovery_tokens` row's `user_id` / `expires_at`.
#[derive(Debug, Clone, Copy)]
struct RecoveryChallenge {
    /// The account this challenge recovers.
    user: UserId,
    /// When the challenge stops being valid; `complete_account_recovery` rejects it from this instant.
    expires_at: Timestamp,
}

/// The default validity window, in milliseconds, stamped onto a freshly-issued recovery challenge.
///
/// Mirrors the durable backing's `RECOVERY_TOKEN_VALIDITY_MILLIS` so the two implementors agree. The
/// committed schema requires a non-null `recovery_tokens.expires_at`; the authoritative validity
/// *value* is owned downstream (EPIC-IDENTITY `REQ-ID-*`, with the retention stage sweeping expired
/// rows). This backing persists and enforces an expiry at `complete_account_recovery`; it does not
/// own the policy value.
const RECOVERY_TOKEN_VALIDITY_MILLIS: i64 = 24 * 60 * 60 * 1000;

/// Current wall-clock time as epoch-milliseconds. Pre-epoch system clocks (which do not occur in
/// practice) degrade to `0` rather than panicking.
fn now_millis() -> Timestamp {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    Timestamp::from_millis(millis)
}

/// The single, indistinguishable [`Unauthorized`](ServerError::Unauthorized) returned by
/// `complete_account_recovery` for every validation failure (unknown handle, unknown or mismatched
/// token, or expired token), matching the durable backing's fail-secure response.
fn unauthorized_recovery() -> ServerError {
    ServerError::Unauthorized("invalid or expired recovery token".to_owned())
}

#[async_trait]
impl ChirpRepository for InMemoryRepository {
    async fn create_user(&self, request: CreateUserRequest) -> Result<User, ServerError> {
        let mut store = self.lock()?;

        if store.username_to_id.contains_key(request.username.as_str()) {
            return Err(ServerError::Conflict(format!(
                "username '{}' is already taken",
                request.username
            )));
        }

        let id = UserId::new(store.next_user_id);
        store.next_user_id += 1;

        let user = User {
            id,
            username: request.username,
            display_name: request.display_name,
            bio: request.bio,
            created_at: now_millis(),
            follower_count: 0,
            following_count: 0,
        };

        store
            .username_to_id
            .insert(user.username.as_str().to_owned(), id);
        store.users.insert(id, user.clone());
        Ok(user)
    }

    async fn get_user(&self, id: UserId) -> Result<User, ServerError> {
        let store = self.lock()?;
        store
            .users
            .get(&id)
            .cloned()
            .ok_or_else(|| ServerError::NotFound(format!("user {id}")))
    }

    async fn login(&self, request: LoginRequest) -> Result<AuthResponse, ServerError> {
        let mut store = self.lock()?;

        let id = *store
            .username_to_id
            .get(request.username.as_str())
            .ok_or_else(|| ServerError::NotFound(format!("user '{}'", request.username)))?;

        // Known handle ⇒ fetch the profile and mint an opaque session token.
        let user = store
            .users
            .get(&id)
            .cloned()
            .ok_or_else(|| ServerError::Internal("user index/record out of sync".to_owned()))?;

        let token_value = format!("tok-{}-{}", store.next_token, id.get());
        store.next_token += 1;
        store.tokens.insert(token_value.clone(), id);

        Ok(AuthResponse {
            user,
            token: SessionToken(token_value),
        })
    }

    async fn resolve_token(&self, token: &SessionToken) -> Result<UserId, ServerError> {
        let store = self.lock()?;
        store
            .tokens
            .get(&token.0)
            .copied()
            .ok_or_else(|| ServerError::Unauthorized("invalid or expired session token".to_owned()))
    }

    async fn create_chirp(
        &self,
        author: UserId,
        request: CreateChirpRequest,
    ) -> Result<Chirp, ServerError> {
        let mut store = self.lock()?;

        if !store.users.contains_key(&author) {
            return Err(ServerError::NotFound(format!("author {author}")));
        }
        if let Some(parent) = request.reply_to {
            if !store.chirps.contains_key(&parent) {
                return Err(ServerError::NotFound(format!("reply target {parent}")));
            }
        }

        let id = ChirpId::new(store.next_chirp_id);
        store.next_chirp_id += 1;

        let chirp = Chirp {
            id,
            author_id: author,
            text: request.text,
            created_at: now_millis(),
            like_count: 0,
            reply_to: request.reply_to,
        };

        store.chirps.insert(id, chirp.clone());
        Ok(chirp)
    }

    async fn timeline(
        &self,
        viewer: UserId,
        query: TimelineQuery,
    ) -> Result<Page<Chirp>, ServerError> {
        let store = self.lock()?;

        if !store.users.contains_key(&viewer) {
            return Err(ServerError::NotFound(format!("viewer {viewer}")));
        }

        let limit = query
            .limit
            .unwrap_or(DEFAULT_TIMELINE_LIMIT)
            .clamp(1, MAX_TIMELINE_LIMIT) as usize;

        // A malformed cursor is treated as "from the newest" — cursors are server-minted and
        // opaque, so a value we did not mint simply fails safe to the first page.
        let cursor_id: Option<u64> = query.cursor.as_ref().and_then(|c| c.0.parse::<u64>().ok());

        // Home timeline = the viewer's own chirps plus those of everyone the viewer follows.
        let followees: HashSet<UserId> = store
            .follows
            .iter()
            .filter(|(follower, _)| *follower == viewer)
            .map(|(_, followee)| *followee)
            .collect();
        let is_visible = |author: UserId| author == viewer || followees.contains(&author);

        let mut visible: Vec<&Chirp> = store
            .chirps
            .values()
            .filter(|chirp| is_visible(chirp.author_id))
            .filter(|chirp| cursor_id.is_none_or(|cid| chirp.id.get() < cid))
            .collect();
        // Reverse-chronological: newest (largest id) first.
        visible.sort_by_key(|chirp| std::cmp::Reverse(chirp.id.get()));

        // Take one extra to learn whether a further page exists.
        let mut items: Vec<Chirp> = visible.into_iter().take(limit + 1).cloned().collect();
        let has_more = items.len() > limit;
        if has_more {
            items.truncate(limit);
        }
        let next_cursor = if has_more {
            items.last().map(|chirp| Cursor(chirp.id.get().to_string()))
        } else {
            None
        };

        Ok(Page::new(items, next_cursor))
    }

    async fn follow(
        &self,
        follower: UserId,
        followee: UserId,
    ) -> Result<FollowResponse, ServerError> {
        if follower == followee {
            return Err(ServerError::Forbidden(
                "a user cannot follow themselves".to_owned(),
            ));
        }

        let mut store = self.lock()?;
        if !store.users.contains_key(&follower) {
            return Err(ServerError::NotFound(format!("follower {follower}")));
        }
        if !store.users.contains_key(&followee) {
            return Err(ServerError::NotFound(format!("followee {followee}")));
        }

        // Idempotent: counts move only on a genuine state change.
        if store.follows.insert((follower, followee)) {
            if let Some(f) = store.users.get_mut(&follower) {
                f.following_count += 1;
            }
            if let Some(f) = store.users.get_mut(&followee) {
                f.follower_count += 1;
            }
        }

        let follower_count = store.users[&followee].follower_count;
        Ok(FollowResponse {
            followee_id: followee,
            following: true,
            follower_count,
        })
    }

    async fn unfollow(
        &self,
        follower: UserId,
        followee: UserId,
    ) -> Result<FollowResponse, ServerError> {
        let mut store = self.lock()?;
        if !store.users.contains_key(&follower) {
            return Err(ServerError::NotFound(format!("follower {follower}")));
        }
        if !store.users.contains_key(&followee) {
            return Err(ServerError::NotFound(format!("followee {followee}")));
        }

        // Idempotent: counts move only if an edge actually existed.
        if store.follows.remove(&(follower, followee)) {
            if let Some(f) = store.users.get_mut(&follower) {
                f.following_count = f.following_count.saturating_sub(1);
            }
            if let Some(f) = store.users.get_mut(&followee) {
                f.follower_count = f.follower_count.saturating_sub(1);
            }
        }

        let follower_count = store.users[&followee].follower_count;
        Ok(FollowResponse {
            followee_id: followee,
            following: false,
            follower_count,
        })
    }

    async fn like(&self, user: UserId, chirp: ChirpId) -> Result<LikeResponse, ServerError> {
        let mut store = self.lock()?;
        if !store.users.contains_key(&user) {
            return Err(ServerError::NotFound(format!("user {user}")));
        }
        if !store.chirps.contains_key(&chirp) {
            return Err(ServerError::NotFound(format!("chirp {chirp}")));
        }

        if store.likes.insert((user, chirp)) {
            if let Some(c) = store.chirps.get_mut(&chirp) {
                c.like_count += 1;
            }
        }

        let like_count = store.chirps[&chirp].like_count;
        Ok(LikeResponse {
            chirp_id: chirp,
            liked: true,
            like_count,
        })
    }

    async fn unlike(&self, user: UserId, chirp: ChirpId) -> Result<LikeResponse, ServerError> {
        let mut store = self.lock()?;
        if !store.users.contains_key(&user) {
            return Err(ServerError::NotFound(format!("user {user}")));
        }
        if !store.chirps.contains_key(&chirp) {
            return Err(ServerError::NotFound(format!("chirp {chirp}")));
        }

        if store.likes.remove(&(user, chirp)) {
            if let Some(c) = store.chirps.get_mut(&chirp) {
                c.like_count = c.like_count.saturating_sub(1);
            }
        }

        let like_count = store.chirps[&chirp].like_count;
        Ok(LikeResponse {
            chirp_id: chirp,
            liked: false,
            like_count,
        })
    }

    // ---- Release-bar additions (issue #66) ------------------------------------------------------
    // The in-memory backing is retained only to keep the store-free test suite compiling and green;
    // the durable behaviour of these capabilities is implemented downstream (EPIC-PERSIST durable
    // implementation). Each placeholder is error-contract-consistent: an operation whose durable
    // behaviour is not yet built reports a `ServerError` and NEVER falsely confirms success. The one
    // exception is `store_health`, which is genuinely, truthfully healthy for an in-process store.

    /// Sets (or replaces) `user`'s credential, mirroring the durable backing's set/replace semantics.
    ///
    /// The user must exist ([`NotFound`](ServerError::NotFound) otherwise, matching the durable
    /// backing's `credentials.user_id` foreign key); setting an existing credential replaces it. The
    /// secret is stored verbatim as the at-rest representation — the credential protocol is owned
    /// downstream (`REQ-ID-*`); this backing only holds the slot.
    async fn set_credential(&self, user: UserId, secret: &str) -> Result<(), ServerError> {
        let mut store = self.lock()?;
        if !store.users.contains_key(&user) {
            return Err(ServerError::NotFound(format!("user {user}")));
        }
        store.credentials.insert(user, secret.to_owned());
        Ok(())
    }

    /// Verifies `secret` against the stored credential for the account named by `username`,
    /// returning the authenticated [`UserId`] on a match.
    ///
    /// Fails [`Unauthorized`](ServerError::Unauthorized) — indistinguishably — when the handle is
    /// unknown, when no credential has been set for it, or when the secret does not match, so the
    /// backing never discloses which of those held (no account enumeration).
    async fn verify_credential(
        &self,
        username: &Username,
        secret: &str,
    ) -> Result<UserId, ServerError> {
        let store = self.lock()?;
        let id = *store
            .username_to_id
            .get(username.as_str())
            .ok_or_else(|| ServerError::Unauthorized("invalid username or secret".to_owned()))?;
        let stored = store
            .credentials
            .get(&id)
            .ok_or_else(|| ServerError::Unauthorized("invalid username or secret".to_owned()))?;
        if stored == secret {
            Ok(id)
        } else {
            Err(ServerError::Unauthorized(
                "invalid username or secret".to_owned(),
            ))
        }
    }

    /// Invalidates a session by removing its issued token, so the token no longer
    /// [`resolve_token`](ChirpRepository::resolve_token)s — mirroring the durable backing's session
    /// delete.
    ///
    /// Idempotent: invalidating a token that was never issued (or was already invalidated) succeeds,
    /// matching the durable backing's zero-row delete.
    async fn invalidate_session(&self, token: &SessionToken) -> Result<(), ServerError> {
        let mut store = self.lock()?;
        store.tokens.remove(&token.0);
        Ok(())
    }

    /// Issues an account-recovery challenge for the account named by `username`, mirroring the durable
    /// backing: it records an opaque recovery token for the user, stamped with an expiry. The token is
    /// delivered to the holder out of band (owned downstream); this backing only records the
    /// challenge, so it returns `()`.
    ///
    /// [`NotFound`](ServerError::NotFound) if no account has that handle, matching the durable
    /// backing's in-transaction existence decision.
    async fn begin_account_recovery(&self, username: &Username) -> Result<(), ServerError> {
        let mut store = self.lock()?;
        let user = *store
            .username_to_id
            .get(username.as_str())
            .ok_or_else(|| ServerError::NotFound(format!("user '{username}'")))?;

        let token = format!("rec-{}", store.next_token);
        store.next_token += 1;
        let expires_at = Timestamp::from_millis(
            now_millis()
                .as_millis()
                .saturating_add(RECOVERY_TOKEN_VALIDITY_MILLIS),
        );
        store
            .recovery_tokens
            .insert(token, RecoveryChallenge { user, expires_at });
        Ok(())
    }

    /// Completes account recovery, mirroring the durable backing: validates `recovery_token` for
    /// `username` and, on success, sets `new_secret` as the account's credential (the same effect as
    /// [`set_credential`](ChirpRepository::set_credential)) and consumes the single-use challenge.
    ///
    /// Fail-secure: reports [`Unauthorized`](ServerError::Unauthorized) indistinguishably when the
    /// handle is unknown, the token is unknown, it belongs to another account, or it has expired, so
    /// the backing never discloses which held (no account or token enumeration).
    async fn complete_account_recovery(
        &self,
        username: &Username,
        recovery_token: &str,
        new_secret: &str,
    ) -> Result<(), ServerError> {
        let mut store = self.lock()?;

        // Resolve the handle; an unknown handle is the same Unauthorized as an unknown token.
        let Some(&user) = store.username_to_id.get(username.as_str()) else {
            return Err(unauthorized_recovery());
        };

        // Validate the presented challenge: it must exist, belong to this account, and be unexpired.
        let valid = matches!(
            store.recovery_tokens.get(recovery_token),
            Some(challenge)
                if challenge.user == user
                    && challenge.expires_at.as_millis() > now_millis().as_millis()
        );
        if !valid {
            return Err(unauthorized_recovery());
        }

        // Consume the single-use challenge and reset the credential (same effect as set_credential).
        store.recovery_tokens.remove(recovery_token);
        store.credentials.insert(user, new_secret.to_owned());
        Ok(())
    }

    /// Updates the mutable profile fields (`display_name`, `bio`) of `user` and returns the updated
    /// profile, mirroring the durable backing.
    ///
    /// [`NotFound`](ServerError::NotFound) if no user has that id, matching the durable backing's
    /// zero-row update. Only the mutable fields change: the stored [`UserId`] and [`Username`] are the
    /// account's immutable identity and are never rewritten here (`REQ-DOM-008`), and `created_at` and
    /// the denormalized counts are left untouched, so the returned profile differs from the stored one
    /// only in `display_name`/`bio`.
    async fn update_profile(
        &self,
        user: UserId,
        display_name: String,
        bio: Option<String>,
    ) -> Result<User, ServerError> {
        let mut store = self.lock()?;
        let record = store
            .users
            .get_mut(&user)
            .ok_or_else(|| ServerError::NotFound(format!("user {user}")))?;
        // Identity (id, username) and created_at/counts are immutable here; only the profile changes.
        record.display_name = display_name;
        record.bio = bio;
        Ok(record.clone())
    }

    async fn delete_account(&self, _user: UserId) -> Result<(), ServerError> {
        Err(ServerError::Internal(
            "delete_account is not yet implemented in the in-memory backing".to_owned(),
        ))
    }

    async fn export_user_data(&self, _user: UserId) -> Result<String, ServerError> {
        Err(ServerError::Internal(
            "export_user_data is not yet implemented in the in-memory backing".to_owned(),
        ))
    }

    async fn list_followers(
        &self,
        _user: UserId,
        _query: TimelineQuery,
    ) -> Result<Page<User>, ServerError> {
        Err(ServerError::Internal(
            "list_followers is not yet implemented in the in-memory backing".to_owned(),
        ))
    }

    async fn list_following(
        &self,
        _user: UserId,
        _query: TimelineQuery,
    ) -> Result<Page<User>, ServerError> {
        Err(ServerError::Internal(
            "list_following is not yet implemented in the in-memory backing".to_owned(),
        ))
    }

    async fn remove_chirp(&self, _actor: UserId, _chirp: ChirpId) -> Result<(), ServerError> {
        Err(ServerError::Internal(
            "remove_chirp is not yet implemented in the in-memory backing".to_owned(),
        ))
    }

    async fn set_account_suspension(
        &self,
        _actor: UserId,
        _target: UserId,
        _suspended: bool,
    ) -> Result<(), ServerError> {
        Err(ServerError::Internal(
            "set_account_suspension is not yet implemented in the in-memory backing".to_owned(),
        ))
    }

    /// The in-memory store is in-process and always reachable, so it is genuinely healthy — this is
    /// a truthful trivially-correct result, not a placeholder that masks unimplemented behaviour.
    async fn store_health(&self) -> Result<(), ServerError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chirp_types::domain::ChirpText;

    /// Registers a user with the given handle and returns the created profile.
    async fn seed_user(repo: &InMemoryRepository, handle: &str) -> User {
        repo.create_user(CreateUserRequest {
            username: Username::parse(handle).expect("valid handle"),
            display_name: format!("{handle} display"),
            bio: None,
        })
        .await
        .expect("user creation succeeds")
    }

    /// Posts a top-level chirp authored by `author` and returns it.
    async fn seed_chirp(repo: &InMemoryRepository, author: UserId, body: &str) -> Chirp {
        repo.create_chirp(
            author,
            CreateChirpRequest {
                text: ChirpText::parse(body).expect("valid body"),
                reply_to: None,
            },
        )
        .await
        .expect("chirp creation succeeds")
    }

    #[tokio::test]
    async fn create_user_assigns_id_timestamp_and_zeroed_counts() {
        let repo = InMemoryRepository::new();
        let user = seed_user(&repo, "alice").await;

        assert_eq!(user.username.as_str(), "alice");
        assert_eq!(user.follower_count, 0);
        assert_eq!(user.following_count, 0);
        // The user is retrievable by its assigned id.
        let fetched = repo.get_user(user.id).await.expect("user exists");
        assert_eq!(fetched, user);
    }

    #[tokio::test]
    async fn duplicate_username_is_a_conflict() {
        let repo = InMemoryRepository::new();
        seed_user(&repo, "alice").await;

        let err = repo
            .create_user(CreateUserRequest {
                username: Username::parse("alice").unwrap(),
                display_name: "Another Alice".to_owned(),
                bio: None,
            })
            .await
            .expect_err("duplicate handle must conflict");
        assert!(matches!(err, ServerError::Conflict(_)));
    }

    #[tokio::test]
    async fn get_user_is_not_found_for_absent_id() {
        let repo = InMemoryRepository::new();
        let err = repo
            .get_user(UserId::new(999))
            .await
            .expect_err("absent user");
        assert!(matches!(err, ServerError::NotFound(_)));
    }

    #[tokio::test]
    async fn login_issues_token_resolvable_back_to_the_user() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;

        let auth = repo
            .login(LoginRequest {
                username: Username::parse("alice").unwrap(),
            })
            .await
            .expect("known handle logs in");
        assert_eq!(auth.user.id, alice.id);

        let resolved = repo
            .resolve_token(&auth.token)
            .await
            .expect("issued token resolves");
        assert_eq!(resolved, alice.id);
    }

    #[tokio::test]
    async fn login_unknown_handle_is_not_found() {
        let repo = InMemoryRepository::new();
        let err = repo
            .login(LoginRequest {
                username: Username::parse("ghost").unwrap(),
            })
            .await
            .expect_err("unknown handle");
        assert!(matches!(err, ServerError::NotFound(_)));
    }

    #[tokio::test]
    async fn resolve_token_rejects_an_unknown_token() {
        let repo = InMemoryRepository::new();
        let err = repo
            .resolve_token(&SessionToken("not-a-real-token".to_owned()))
            .await
            .expect_err("bogus token");
        assert!(matches!(err, ServerError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn invalidate_session_stops_the_token_resolving_and_is_idempotent() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;

        let auth = repo
            .login(LoginRequest {
                username: Username::parse("alice").unwrap(),
            })
            .await
            .expect("known handle logs in");
        // The freshly issued token resolves...
        assert_eq!(repo.resolve_token(&auth.token).await.unwrap(), alice.id);

        // ...invalidation removes it, so it stops resolving...
        repo.invalidate_session(&auth.token)
            .await
            .expect("invalidate the live session");
        assert!(matches!(
            repo.resolve_token(&auth.token).await.unwrap_err(),
            ServerError::Unauthorized(_)
        ));

        // ...and invalidating again is an idempotent no-op, never an error.
        repo.invalidate_session(&auth.token)
            .await
            .expect("invalidating an already-invalid token succeeds");
    }

    #[tokio::test]
    async fn set_then_verify_credential_round_trips_to_the_user() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;

        repo.set_credential(alice.id, "s3cr3t")
            .await
            .expect("set credential for an existing user");

        let verified = repo
            .verify_credential(&Username::parse("alice").unwrap(), "s3cr3t")
            .await
            .expect("the right secret verifies");
        assert_eq!(verified, alice.id);
    }

    #[tokio::test]
    async fn set_credential_replaces_the_previous_secret() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;

        repo.set_credential(alice.id, "old").await.unwrap();
        repo.set_credential(alice.id, "new").await.unwrap();

        // The replaced secret no longer verifies; only the current one does.
        let handle = Username::parse("alice").unwrap();
        assert!(matches!(
            repo.verify_credential(&handle, "old").await.unwrap_err(),
            ServerError::Unauthorized(_)
        ));
        assert_eq!(
            repo.verify_credential(&handle, "new").await.unwrap(),
            alice.id
        );
    }

    #[tokio::test]
    async fn set_credential_is_not_found_for_an_absent_user() {
        let repo = InMemoryRepository::new();
        let err = repo
            .set_credential(UserId::new(9999), "secret")
            .await
            .expect_err("no such user");
        assert!(matches!(err, ServerError::NotFound(_)));
    }

    #[tokio::test]
    async fn verify_credential_is_unauthorized_for_unknown_handle_missing_credential_or_mismatch() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;
        let alice_handle = Username::parse("alice").unwrap();

        // Unknown handle.
        assert!(matches!(
            repo.verify_credential(&Username::parse("ghost").unwrap(), "x")
                .await
                .unwrap_err(),
            ServerError::Unauthorized(_)
        ));
        // Known handle, but no credential has been set yet.
        assert!(matches!(
            repo.verify_credential(&alice_handle, "x")
                .await
                .unwrap_err(),
            ServerError::Unauthorized(_)
        ));
        // Known handle with a credential set, but the wrong secret.
        repo.set_credential(alice.id, "right").await.unwrap();
        assert!(matches!(
            repo.verify_credential(&alice_handle, "wrong")
                .await
                .unwrap_err(),
            ServerError::Unauthorized(_)
        ));
    }

    #[tokio::test]
    async fn create_chirp_assigns_fields_and_rejects_absent_author_or_reply_target() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;

        let chirp = seed_chirp(&repo, alice.id, "hello chirp").await;
        assert_eq!(chirp.author_id, alice.id);
        assert_eq!(chirp.like_count, 0);
        assert_eq!(chirp.reply_to, None);

        // Absent author.
        let err = repo
            .create_chirp(
                UserId::new(12345),
                CreateChirpRequest {
                    text: ChirpText::parse("orphan").unwrap(),
                    reply_to: None,
                },
            )
            .await
            .expect_err("absent author");
        assert!(matches!(err, ServerError::NotFound(_)));

        // Absent reply target.
        let err = repo
            .create_chirp(
                alice.id,
                CreateChirpRequest {
                    text: ChirpText::parse("reply to nothing").unwrap(),
                    reply_to: Some(ChirpId::new(9999)),
                },
            )
            .await
            .expect_err("absent reply target");
        assert!(matches!(err, ServerError::NotFound(_)));

        // A valid reply to an existing chirp succeeds.
        let reply = repo
            .create_chirp(
                alice.id,
                CreateChirpRequest {
                    text: ChirpText::parse("a real reply").unwrap(),
                    reply_to: Some(chirp.id),
                },
            )
            .await
            .expect("valid reply");
        assert_eq!(reply.reply_to, Some(chirp.id));
    }

    #[tokio::test]
    async fn timeline_is_reverse_chronological_and_scoped_to_self_plus_followees() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;
        let bob = seed_user(&repo, "bob").await;
        let carol = seed_user(&repo, "carol").await;

        let a1 = seed_chirp(&repo, alice.id, "alice one").await;
        let b1 = seed_chirp(&repo, bob.id, "bob one").await;
        let _c1 = seed_chirp(&repo, carol.id, "carol one (unfollowed)").await;
        let a2 = seed_chirp(&repo, alice.id, "alice two").await;

        // Alice follows bob but not carol.
        repo.follow(alice.id, bob.id).await.expect("follow bob");

        let page = repo
            .timeline(alice.id, TimelineQuery::default())
            .await
            .expect("timeline");

        let ids: Vec<ChirpId> = page.items.iter().map(|c| c.id).collect();
        // Newest first: a2, b1, a1 — carol's chirp excluded (not followed).
        assert_eq!(ids, vec![a2.id, b1.id, a1.id]);
        assert!(page.next_cursor.is_none());
    }

    #[tokio::test]
    async fn timeline_paginates_with_cursor_and_clamps_the_limit() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;

        // Five chirps, ids ascending in post order.
        let mut posted = Vec::new();
        for i in 0..5 {
            posted.push(seed_chirp(&repo, alice.id, &format!("chirp {i}")).await);
        }

        // First page of 2 (limit 0 is clamped up to 1? here request 2).
        let page1 = repo
            .timeline(
                alice.id,
                TimelineQuery {
                    limit: Some(2),
                    cursor: None,
                },
            )
            .await
            .expect("page 1");
        let ids1: Vec<ChirpId> = page1.items.iter().map(|c| c.id).collect();
        assert_eq!(ids1, vec![posted[4].id, posted[3].id]);
        let cursor = page1.next_cursor.clone().expect("more pages remain");

        // Second page continues strictly older than the cursor.
        let page2 = repo
            .timeline(
                alice.id,
                TimelineQuery {
                    limit: Some(2),
                    cursor: Some(cursor),
                },
            )
            .await
            .expect("page 2");
        let ids2: Vec<ChirpId> = page2.items.iter().map(|c| c.id).collect();
        assert_eq!(ids2, vec![posted[2].id, posted[1].id]);

        // A limit above the ceiling is clamped (does not error, returns all remaining).
        let big = repo
            .timeline(
                alice.id,
                TimelineQuery {
                    limit: Some(MAX_TIMELINE_LIMIT + 50),
                    cursor: None,
                },
            )
            .await
            .expect("clamped limit");
        assert_eq!(big.items.len(), 5);
        assert!(big.next_cursor.is_none());
    }

    #[tokio::test]
    async fn timeline_is_not_found_for_absent_viewer() {
        let repo = InMemoryRepository::new();
        let err = repo
            .timeline(UserId::new(42), TimelineQuery::default())
            .await
            .expect_err("absent viewer");
        assert!(matches!(err, ServerError::NotFound(_)));
    }

    #[tokio::test]
    async fn follow_updates_counts_is_idempotent_and_forbids_self_follow() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;
        let bob = seed_user(&repo, "bob").await;

        let resp = repo.follow(alice.id, bob.id).await.expect("follow");
        assert!(resp.following);
        assert_eq!(resp.follower_count, 1);

        // Idempotent: a second follow does not double-count.
        let resp2 = repo
            .follow(alice.id, bob.id)
            .await
            .expect("idempotent follow");
        assert_eq!(resp2.follower_count, 1);

        // Denormalized counts on both profiles are consistent.
        assert_eq!(repo.get_user(bob.id).await.unwrap().follower_count, 1);
        assert_eq!(repo.get_user(alice.id).await.unwrap().following_count, 1);

        // Self-follow is forbidden.
        let err = repo
            .follow(alice.id, alice.id)
            .await
            .expect_err("self-follow");
        assert!(matches!(err, ServerError::Forbidden(_)));

        // Absent target is not found.
        let err = repo
            .follow(alice.id, UserId::new(777))
            .await
            .expect_err("absent followee");
        assert!(matches!(err, ServerError::NotFound(_)));
    }

    #[tokio::test]
    async fn unfollow_updates_counts_and_is_idempotent() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;
        let bob = seed_user(&repo, "bob").await;
        repo.follow(alice.id, bob.id).await.unwrap();

        let resp = repo.unfollow(alice.id, bob.id).await.expect("unfollow");
        assert!(!resp.following);
        assert_eq!(resp.follower_count, 0);
        assert_eq!(repo.get_user(alice.id).await.unwrap().following_count, 0);

        // Idempotent: unfollowing again stays at zero (no underflow).
        let resp2 = repo.unfollow(alice.id, bob.id).await.expect("idempotent");
        assert_eq!(resp2.follower_count, 0);
    }

    #[tokio::test]
    async fn like_updates_count_is_idempotent_and_requires_an_existing_chirp() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;
        let chirp = seed_chirp(&repo, alice.id, "likeable").await;

        let resp = repo.like(alice.id, chirp.id).await.expect("like");
        assert!(resp.liked);
        assert_eq!(resp.like_count, 1);

        // Idempotent.
        let resp2 = repo
            .like(alice.id, chirp.id)
            .await
            .expect("idempotent like");
        assert_eq!(resp2.like_count, 1);

        // Absent chirp is not found.
        let err = repo
            .like(alice.id, ChirpId::new(4242))
            .await
            .expect_err("absent chirp");
        assert!(matches!(err, ServerError::NotFound(_)));
    }

    #[tokio::test]
    async fn unlike_updates_count_and_is_idempotent() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;
        let chirp = seed_chirp(&repo, alice.id, "likeable").await;
        repo.like(alice.id, chirp.id).await.unwrap();

        let resp = repo.unlike(alice.id, chirp.id).await.expect("unlike");
        assert!(!resp.liked);
        assert_eq!(resp.like_count, 0);

        // Idempotent: no underflow below zero.
        let resp2 = repo.unlike(alice.id, chirp.id).await.expect("idempotent");
        assert_eq!(resp2.like_count, 0);
    }

    // ---- Account recovery (store-free) ----------------------------------------------------------

    /// Reads the recovery token currently issued for `user` straight from the in-memory store — the
    /// store-free analogue of the durable test reading the `recovery_tokens` row (a holder receiving
    /// the out-of-band challenge).
    fn issued_recovery_token(repo: &InMemoryRepository, user: UserId) -> String {
        let store = repo.lock().expect("lock the store");
        store
            .recovery_tokens
            .iter()
            .find(|(_, challenge)| challenge.user == user)
            .map(|(token, _)| token.clone())
            .expect("a recovery challenge was issued for the user")
    }

    /// Forces the stored challenge named by `token` to be already expired, so
    /// `complete_account_recovery` rejects it — the store-free analogue of injecting a past-dated
    /// `recovery_tokens` row.
    fn expire_recovery_token(repo: &InMemoryRepository, token: &str) {
        let mut store = repo.lock().expect("lock the store");
        let challenge = store
            .recovery_tokens
            .get_mut(token)
            .expect("the challenge exists");
        challenge.expires_at = Timestamp::from_millis(0);
    }

    #[tokio::test]
    async fn account_recovery_begin_complete_resets_credential_and_is_single_use() {
        let repo = InMemoryRepository::new();
        let user = seed_user(&repo, "recovery_user").await;
        repo.set_credential(user.id, "old-secret")
            .await
            .expect("seed an initial credential");

        // begin issues a challenge for the handle; complete validates it and resets the credential.
        repo.begin_account_recovery(&user.username)
            .await
            .expect("begin issues a challenge for a known handle");
        let token = issued_recovery_token(&repo, user.id);
        repo.complete_account_recovery(&user.username, &token, "new-secret")
            .await
            .expect("a valid challenge completes");

        // The credential set via recovery verifies; the old one no longer does.
        assert_eq!(
            repo.verify_credential(&user.username, "new-secret")
                .await
                .expect("the recovery-set secret verifies"),
            user.id
        );
        assert!(matches!(
            repo.verify_credential(&user.username, "old-secret")
                .await
                .unwrap_err(),
            ServerError::Unauthorized(_)
        ));

        // Single-use: the consumed challenge cannot be replayed.
        assert!(matches!(
            repo.complete_account_recovery(&user.username, &token, "another")
                .await
                .unwrap_err(),
            ServerError::Unauthorized(_)
        ));
    }

    #[tokio::test]
    async fn begin_account_recovery_is_not_found_for_an_unknown_handle() {
        let repo = InMemoryRepository::new();
        let unknown = Username::parse("ghost").unwrap();
        assert!(matches!(
            repo.begin_account_recovery(&unknown).await.unwrap_err(),
            ServerError::NotFound(_)
        ));
    }

    #[tokio::test]
    async fn complete_account_recovery_is_unauthorized_for_unknown_mismatched_or_expired_token() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;
        let bob = seed_user(&repo, "bob").await;

        // Unknown handle → Unauthorized (indistinguishable from an unknown token; no enumeration).
        assert!(matches!(
            repo.complete_account_recovery(&Username::parse("ghost").unwrap(), "rec-x", "s")
                .await
                .unwrap_err(),
            ServerError::Unauthorized(_)
        ));

        // Known handle, unknown token.
        assert!(matches!(
            repo.complete_account_recovery(&alice.username, "rec-never-issued", "s")
                .await
                .unwrap_err(),
            ServerError::Unauthorized(_)
        ));

        // A challenge issued for bob must not recover alice.
        repo.begin_account_recovery(&bob.username).await.unwrap();
        let bob_token = issued_recovery_token(&repo, bob.id);
        assert!(matches!(
            repo.complete_account_recovery(&alice.username, &bob_token, "s")
                .await
                .unwrap_err(),
            ServerError::Unauthorized(_)
        ));

        // Expired token: force alice's challenge into the past, then complete → Unauthorized.
        repo.begin_account_recovery(&alice.username).await.unwrap();
        let alice_token = issued_recovery_token(&repo, alice.id);
        expire_recovery_token(&repo, &alice_token);
        assert!(matches!(
            repo.complete_account_recovery(&alice.username, &alice_token, "s")
                .await
                .unwrap_err(),
            ServerError::Unauthorized(_)
        ));
    }

    // ---- Profile update (store-free) ------------------------------------------------------------

    #[tokio::test]
    async fn update_profile_changes_mutable_fields_and_preserves_immutable_identity() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;

        let updated = repo
            .update_profile(
                alice.id,
                "Alice In Wonderland".to_owned(),
                Some("hi".to_owned()),
            )
            .await
            .expect("update succeeds for an existing user");

        // The mutable fields changed and the returned profile reflects the update.
        assert_eq!(updated.display_name, "Alice In Wonderland");
        assert_eq!(updated.bio.as_deref(), Some("hi"));
        // Immutable identity and non-profile fields are untouched (REQ-DOM-008).
        assert_eq!(updated.id, alice.id);
        assert_eq!(updated.username, alice.username);
        assert_eq!(updated.created_at, alice.created_at);
        assert_eq!(updated.follower_count, alice.follower_count);
        assert_eq!(updated.following_count, alice.following_count);

        // The change is durable in the backing: a fresh read returns the updated profile.
        let fetched = repo.get_user(alice.id).await.expect("user exists");
        assert_eq!(fetched, updated);
    }

    #[tokio::test]
    async fn update_profile_can_clear_the_optional_bio() {
        let repo = InMemoryRepository::new();
        let alice = repo
            .create_user(CreateUserRequest {
                username: Username::parse("alice").unwrap(),
                display_name: "Alice".to_owned(),
                bio: Some("original".to_owned()),
            })
            .await
            .expect("create");

        let updated = repo
            .update_profile(alice.id, "Alice".to_owned(), None)
            .await
            .expect("update succeeds");
        assert_eq!(updated.bio, None);
        assert_eq!(
            repo.get_user(alice.id).await.expect("exists").bio,
            None,
            "the cleared bio is durable"
        );
    }

    #[tokio::test]
    async fn update_profile_is_not_found_for_an_absent_user() {
        let repo = InMemoryRepository::new();
        let err = repo
            .update_profile(UserId::new(999), "ghost".to_owned(), None)
            .await
            .expect_err("no such user");
        assert!(matches!(err, ServerError::NotFound(_)));
    }
}
