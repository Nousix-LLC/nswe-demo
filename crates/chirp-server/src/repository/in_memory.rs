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

use super::{ChirpRepository, TransientSweep, DEFAULT_TIMELINE_LIMIT, MAX_TIMELINE_LIMIT};
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
    /// Chirps under moderation soft-removal — the in-memory mirror of the durable `chirps.removed`
    /// column (`chirp_removal`, REQ-DOM-011). Membership means removed: the chirp row is retained
    /// (so a reply target or a like edge still resolves) but the chirp is **not served** — excluded
    /// from timelines and from data exports, exactly as the durable backing excludes `removed` rows.
    /// The durable backing also records a `removed_at` audit timestamp; the in-memory mirror tracks
    /// only the behavioural flag the two backings must agree on.
    removed: HashSet<ChirpId>,
    /// Issued session tokens → the [`Session`] they authenticate (the user plus the enforced
    /// expiry). Mirrors the durable `sessions` table (`token` primary key, `user_id`, `expires_at`):
    /// an expired session is not resolved and is reclaimed by the sweep.
    tokens: HashMap<String, Session>,
    /// Durable credential per user: user id → the at-rest secret representation. One per user,
    /// mirroring the `credentials` table's `user_id` primary key. The secret's exact format and the
    /// verification protocol are owned by identity & trust (`REQ-ID-*`); this backing only holds the
    /// slot and compares what it was given, so the in-memory and durable backings agree.
    credentials: HashMap<UserId, String>,
    /// Outstanding account-recovery challenges: the opaque recovery token → the account it recovers
    /// and when the challenge expires. Mirrors the durable `recovery_tokens` table (token primary
    /// key, `user_id`, `expires_at`); a validated challenge is consumed (removed) so it is single-use.
    recovery_tokens: HashMap<String, RecoveryChallenge>,
    /// Accounts currently under moderation suspension — the in-memory mirror of the durable
    /// `users.suspended` column (`account_suspension`, REQ-DOM-011). Membership means suspended;
    /// `set_account_suspension` toggles it. The durable backing also records a `suspended_at` audit
    /// timestamp; the in-memory mirror tracks only the behavioral flag the two backings must agree on.
    suspended: HashSet<UserId>,
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

    /// Pages a set of candidate account ids newest-first with the same opaque-cursor scheme as the
    /// timeline, resolving each surviving id to its [`User`]. Shared by
    /// [`list_followers`](ChirpRepository::list_followers) and
    /// [`list_following`](ChirpRepository::list_following) so the paging logic lives once.
    ///
    /// "Newest-first" is id descending — the same stable id-as-reverse-chronological proxy the
    /// timeline uses — because the retained follow edges carry no per-edge timestamp (edge creation
    /// time is the graph stage's concern), so the account id is the stable, unique key both backings
    /// agree on. `query.cursor` (when present and parseable) excludes ids `>=` it; a malformed
    /// cursor fails safe to the first page. The returned
    /// [`next_cursor`](chirp_types::api::Page::next_cursor) is `Some` iff a further page remains, and
    /// is the last returned id — so paging is **exactly-once** (no skips, no duplicates).
    fn page_users(store: &Store, mut candidates: Vec<UserId>, query: &TimelineQuery) -> Page<User> {
        let limit = query
            .limit
            .unwrap_or(DEFAULT_TIMELINE_LIMIT)
            .clamp(1, MAX_TIMELINE_LIMIT) as usize;
        let cursor_id: Option<u64> = query.cursor.as_ref().and_then(|c| c.0.parse::<u64>().ok());

        candidates.retain(|id| cursor_id.is_none_or(|cid| id.get() < cid));
        candidates.sort_by_key(|id| std::cmp::Reverse(id.get()));

        let has_more = candidates.len() > limit;
        candidates.truncate(limit);
        let next_cursor = if has_more {
            candidates.last().map(|id| Cursor(id.get().to_string()))
        } else {
            None
        };

        // Resolve each surviving id to its profile (an id in an edge set always has a user record).
        let items: Vec<User> = candidates
            .into_iter()
            .filter_map(|id| store.users.get(&id).cloned())
            .collect();
        Page::new(items, next_cursor)
    }

    /// Whether `user` is currently under moderation suspension — the in-memory mirror of reading the
    /// durable `users.suspended` column. Test-only observation of `set_account_suspension`'s effect,
    /// which the [`ChirpRepository`] seam does not otherwise surface (suspension is honoured on reads
    /// by a separate capability sibling).
    #[cfg(test)]
    fn is_suspended(&self, user: UserId) -> bool {
        self.lock()
            .expect("store lock is not poisoned in tests")
            .suspended
            .contains(&user)
    }
}

/// An in-memory session: the account it authenticates and its enforced expiry, mirroring a durable
/// `sessions` row's `user_id` / `expires_at`.
#[derive(Debug, Clone, Copy)]
struct Session {
    /// The account this session authenticates.
    user: UserId,
    /// When the session stops resolving. `None` is an unbounded (no-expiry) session — not issued by
    /// this stage, but tolerated so the in-memory and durable NULL-`expires_at` semantics agree; it
    /// is never swept. A `Some(t)` session is expired once `t <= now`.
    expires_at: Option<Timestamp>,
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

/// The durable default session lifetime, in milliseconds, stamped onto a freshly-issued session at
/// [`login`](ChirpRepository::login).
///
/// Mirrors the durable backing's `SESSION_LIFETIME_MILLIS` so the two implementors agree. The
/// substrate left `sessions.expires_at` unset and deferred the lifetime value + sweep to this
/// retention stage (REQ-OPS-010); this backing stamps every new session with a bounded expiry,
/// rejects an expired session at `resolve_token`, and reclaims it in the sweep. The authoritative
/// validity *value* and the session security policy are owned downstream (EPIC-IDENTITY,
/// `REQ-ID-*`); this constant is only the durable default this stage applies so a bounded lifetime
/// is enforced, not a policy decision.
const SESSION_LIFETIME_MILLIS: i64 = 30 * 24 * 60 * 60 * 1000;

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
        // Stamp the enforced session lifetime (durable default; authoritative value owned by
        // EPIC-IDENTITY), mirroring the durable backing so the two agree.
        let expires_at = Some(Timestamp::from_millis(
            now_millis()
                .as_millis()
                .saturating_add(SESSION_LIFETIME_MILLIS),
        ));
        store.tokens.insert(
            token_value.clone(),
            Session {
                user: id,
                expires_at,
            },
        );

        Ok(AuthResponse {
            user,
            token: SessionToken(token_value),
        })
    }

    async fn resolve_token(&self, token: &SessionToken) -> Result<UserId, ServerError> {
        let store = self.lock()?;
        // An expired session is treated as absent (same Unauthorized as an unknown token), the
        // complement of the sweep's `expires_at <= now`. A `None` expiry is unbounded and resolves.
        let now = now_millis().as_millis();
        match store.tokens.get(&token.0) {
            Some(session) if session.expires_at.is_none_or(|t| t.as_millis() > now) => {
                Ok(session.user)
            }
            _ => Err(ServerError::Unauthorized(
                "invalid or expired session token".to_owned(),
            )),
        }
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
            // Soft-removed chirps are not served (chirp_removal), mirroring the durable
            // `WHERE removed = FALSE`.
            .filter(|chirp| !store.removed.contains(&chirp.id))
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

    /// Deletes `user` and cascades the removal of ALL their dependent state, reconciling the
    /// denormalized counts of every surviving neighbour — the store-free analogue of the durable
    /// backing's single-transaction cascade (`multiwrite_atomic`,
    /// `counts_match_edges_under_cascade`).
    ///
    /// [`NotFound`](ServerError::NotFound) if no user has that id (an absent or already-deleted
    /// account), matching the durable backing — never a silent no-op. The whole cascade runs under
    /// the single store lock as one atomic critical section: the surviving neighbours' counts are
    /// reconciled from the user's current edges FIRST (while the edges still exist), then every
    /// family the user owns or touches is removed, so a read after the call never sees a surviving
    /// count disagreeing with its edges. The removals mirror the durable schema's foreign keys: a
    /// chirp authored by the user is removed together with its like edges (`ON DELETE CASCADE`), and
    /// a surviving chirp that replied to a removed chirp loses its reply target (`ON DELETE SET
    /// NULL`).
    async fn delete_account(&self, user: UserId) -> Result<(), ServerError> {
        let mut store = self.lock()?;
        if !store.users.contains_key(&user) {
            return Err(ServerError::NotFound(format!("user {user}")));
        }

        // Reconcile SURVIVING neighbours' denormalized counts from the user's current edges, BEFORE
        // removing anything (afterwards the edges are gone and the information is lost). Mirrors the
        // durable backing's pre-cascade UPDATEs.

        // Accounts the user follows each lose one follower; accounts that follow the user each lose
        // one followee. (No self-follow, so the user is never its own neighbour here.)
        let following: Vec<UserId> = store
            .follows
            .iter()
            .filter(|(follower, _)| *follower == user)
            .map(|(_, followee)| *followee)
            .collect();
        let followers: Vec<UserId> = store
            .follows
            .iter()
            .filter(|(_, followee)| *followee == user)
            .map(|(follower, _)| *follower)
            .collect();
        for followee in following {
            if let Some(f) = store.users.get_mut(&followee) {
                f.follower_count = f.follower_count.saturating_sub(1);
            }
        }
        for follower in followers {
            if let Some(f) = store.users.get_mut(&follower) {
                f.following_count = f.following_count.saturating_sub(1);
            }
        }

        // Surviving chirps the user liked each lose one like. Chirps authored by the user are
        // cascaded away below, so they are excluded — decrementing a count on a row about to vanish
        // is pointless (and mirrors the durable `author_id <> user` filter).
        let liked_surviving: Vec<ChirpId> = store
            .likes
            .iter()
            .filter(|(liker, _)| *liker == user)
            .map(|(_, chirp)| *chirp)
            .filter(|chirp| store.chirps.get(chirp).is_some_and(|c| c.author_id != user))
            .collect();
        for chirp in liked_surviving {
            if let Some(c) = store.chirps.get_mut(&chirp) {
                c.like_count = c.like_count.saturating_sub(1);
            }
        }

        // The set of chirps authored by the user — removed below, with their dependents cascaded.
        let authored: HashSet<ChirpId> = store
            .chirps
            .iter()
            .filter(|(_, chirp)| chirp.author_id == user)
            .map(|(id, _)| *id)
            .collect();

        // Remove the account's identity and its handle index entry.
        if let Some(removed_user) = store.users.remove(&user) {
            store.username_to_id.remove(removed_user.username.as_str());
        }

        // Remove the authored chirps and everything that cascades off them.
        store.chirps.retain(|id, _| !authored.contains(id));
        store.removed.retain(|id| !authored.contains(id));
        // Like edges BY the user and like edges ON any removed chirp (by anyone) cascade away.
        store
            .likes
            .retain(|(liker, chirp)| *liker != user && !authored.contains(chirp));
        // A surviving chirp that replied to one of the removed chirps loses its target (SET NULL).
        for chirp in store.chirps.values_mut() {
            if chirp
                .reply_to
                .is_some_and(|parent| authored.contains(&parent))
            {
                chirp.reply_to = None;
            }
        }

        // Follow edges on either side cascade away.
        store
            .follows
            .retain(|(follower, followee)| *follower != user && *followee != user);

        // Transient per-user state cascades away: sessions, credential, recovery tokens, suspension.
        store.tokens.retain(|_, session| session.user != user);
        store.credentials.remove(&user);
        store
            .recovery_tokens
            .retain(|_, challenge| challenge.user != user);
        store.suspended.remove(&user);

        Ok(())
    }

    /// Materializes a user's full data export, mirroring the durable backing: the user's profile,
    /// their authored **non-removed** chirps, their follow graph, and the chirps they like —
    /// serialized through the shared [`ExportDocument`](super::export::ExportDocument) so the two
    /// backings produce byte-identical exports.
    ///
    /// Read under a single lock, so the export is a consistent snapshot (the in-memory analogue of
    /// the durable backing's single read transaction). [`NotFound`](ServerError::NotFound) if no
    /// user has that id, matching the durable backing. Soft-removed chirps are excluded, matching
    /// both the durable `WHERE removed = FALSE` and this backing's timeline.
    async fn export_user_data(&self, user: UserId) -> Result<String, ServerError> {
        let store = self.lock()?;
        let profile = store
            .users
            .get(&user)
            .cloned()
            .ok_or_else(|| ServerError::NotFound(format!("user {user}")))?;

        let chirps: Vec<Chirp> = store
            .chirps
            .values()
            .filter(|chirp| chirp.author_id == user && !store.removed.contains(&chirp.id))
            .cloned()
            .collect();
        let following: Vec<UserId> = store
            .follows
            .iter()
            .filter(|(follower, _)| *follower == user)
            .map(|(_, followee)| *followee)
            .collect();
        let followers: Vec<UserId> = store
            .follows
            .iter()
            .filter(|(_, followee)| *followee == user)
            .map(|(follower, _)| *follower)
            .collect();
        let likes: Vec<ChirpId> = store
            .likes
            .iter()
            .filter(|(liker, _)| *liker == user)
            .map(|(_, chirp)| *chirp)
            .collect();

        super::export::ExportDocument::new(profile, chirps, following, followers, likes).to_json()
    }

    /// Returns one page of the accounts following `user`, newest-first, mirroring the durable backing.
    ///
    /// [`NotFound`](ServerError::NotFound) if `user` does not exist.
    async fn list_followers(
        &self,
        user: UserId,
        query: TimelineQuery,
    ) -> Result<Page<User>, ServerError> {
        let store = self.lock()?;
        if !store.users.contains_key(&user) {
            return Err(ServerError::NotFound(format!("user {user}")));
        }
        let candidates: Vec<UserId> = store
            .follows
            .iter()
            .filter(|(_, followee)| *followee == user)
            .map(|(follower, _)| *follower)
            .collect();
        Ok(Self::page_users(&store, candidates, &query))
    }

    /// Returns one page of the accounts `user` follows, newest-first, mirroring the durable backing.
    ///
    /// [`NotFound`](ServerError::NotFound) if `user` does not exist.
    async fn list_following(
        &self,
        user: UserId,
        query: TimelineQuery,
    ) -> Result<Page<User>, ServerError> {
        let store = self.lock()?;
        if !store.users.contains_key(&user) {
            return Err(ServerError::NotFound(format!("user {user}")));
        }
        let candidates: Vec<UserId> = store
            .follows
            .iter()
            .filter(|(follower, _)| *follower == user)
            .map(|(_, followee)| *followee)
            .collect();
        Ok(Self::page_users(&store, candidates, &query))
    }

    /// Soft-removes `chirp` so it is no longer served, mirroring the durable backing's
    /// `UPDATE chirps SET removed = TRUE`.
    ///
    /// [`NotFound`](ServerError::NotFound) if the chirp does not exist. Idempotent: removing an
    /// already-removed chirp succeeds (a repeated set-membership insert is a no-op). The chirp row
    /// is retained (so a reply target or a like edge still resolves); it is excluded from timelines
    /// and exports by its membership in the removed set.
    ///
    /// The author-vs-moderator **authorization** boundary (`REQ-DOM-020`) is fixed in the model and
    /// enforced downstream of the repository (`REQ-ID-*`); this persistence layer accepts `actor`
    /// and performs the durable removal, exactly as the sibling `set_account_suspension` defers the
    /// moderator-authority determination upstream.
    async fn remove_chirp(&self, _actor: UserId, chirp: ChirpId) -> Result<(), ServerError> {
        let mut store = self.lock()?;
        if !store.chirps.contains_key(&chirp) {
            return Err(ServerError::NotFound(format!("chirp {chirp}")));
        }
        store.removed.insert(chirp);
        Ok(())
    }

    /// Sets or clears the moderation suspension state of `target` on behalf of moderator `actor`,
    /// mirroring the durable backing.
    ///
    /// [`Forbidden`](ServerError::Forbidden) if `actor == target` — moderation acts on *another*
    /// account (REQ-DOM-020), so a self-targeted moderation is refused exactly as the durable backing
    /// refuses it (and as no-self-follow is refused); the moderator-authority gate for a distinct
    /// actor is `REQ-ID-*` and enforced upstream. [`NotFound`](ServerError::NotFound) if `target`
    /// does not exist. On success the target's membership in the suspended set reflects `suspended`.
    async fn set_account_suspension(
        &self,
        actor: UserId,
        target: UserId,
        suspended: bool,
    ) -> Result<(), ServerError> {
        // Mirror the durable backing's model-level authorization result: a moderation action targets
        // ANOTHER account, so a self-targeted moderation is refused regardless of store state.
        if actor == target {
            return Err(ServerError::Forbidden(
                "a moderation action cannot target the acting account".to_owned(),
            ));
        }
        let mut store = self.lock()?;
        if !store.users.contains_key(&target) {
            return Err(ServerError::NotFound(format!("user {target}")));
        }
        if suspended {
            store.suspended.insert(target);
        } else {
            store.suspended.remove(&target);
        }
        Ok(())
    }

    /// The in-memory store is in-process and always reachable, so it is genuinely healthy — this is
    /// a truthful trivially-correct result, not a placeholder that masks unimplemented behaviour.
    async fn store_health(&self) -> Result<(), ServerError> {
        Ok(())
    }

    /// Reclaims expired transient state held in memory, mirroring the durable sweep: expired
    /// sessions and expired recovery-token challenges, under a single lock (the in-memory analogue
    /// of the durable backing's single transaction, so the removals are atomic as one unit).
    ///
    /// "Expired" is `expires_at <= now` — the exact complement of the serve conditions in
    /// [`resolve_token`](ChirpRepository::resolve_token) and
    /// [`complete_account_recovery`](ChirpRepository::complete_account_recovery), so a row is either
    /// servable or sweepable, never both. A session with a `None` expiry (unbounded) is never swept.
    /// User content (users, chirps, follows, likes) is never touched, so the life-of-account
    /// retention guarantee holds. This backing does not materialise export artefacts
    /// (`export_user_data` computes and returns the document, persisting nothing), so it holds no
    /// `export_artifacts` to expire and reports `export_artifacts: 0`; the durable backing owns and
    /// sweeps that table.
    async fn sweep_expired_transient_state(&self) -> Result<TransientSweep, ServerError> {
        let mut store = self.lock()?;
        let now = now_millis().as_millis();

        let sessions_before = store.tokens.len();
        store
            .tokens
            .retain(|_, session| session.expires_at.is_none_or(|t| t.as_millis() > now));
        let sessions = (sessions_before - store.tokens.len()) as u64;

        let recovery_before = store.recovery_tokens.len();
        store
            .recovery_tokens
            .retain(|_, challenge| challenge.expires_at.as_millis() > now);
        let recovery_tokens = (recovery_before - store.recovery_tokens.len()) as u64;

        Ok(TransientSweep {
            sessions,
            recovery_tokens,
            export_artifacts: 0,
        })
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

    #[tokio::test]
    async fn delete_account_cascades_all_state_and_reconciles_survivor_counts() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;
        let bob = seed_user(&repo, "bob").await;
        let carol = seed_user(&repo, "carol").await;

        // Follow graph: alice→bob and carol→alice (both touch alice), plus a control edge bob→carol
        // that does NOT touch alice and must survive untouched.
        repo.follow(alice.id, bob.id).await.unwrap();
        repo.follow(carol.id, alice.id).await.unwrap();
        repo.follow(bob.id, carol.id).await.unwrap();

        // Content: bob authors a chirp that survives; alice authors one that is cascaded away.
        let b1 = seed_chirp(&repo, bob.id, "bob chirp").await;
        let a1 = seed_chirp(&repo, alice.id, "alice chirp").await;

        // Likes: alice likes bob's surviving chirp (its count must reconcile to 0) and her own chirp
        // (cascaded away with it); bob likes alice's chirp (that like edge is cascaded with a1).
        repo.like(alice.id, b1.id).await.unwrap();
        repo.like(alice.id, a1.id).await.unwrap();
        repo.like(bob.id, a1.id).await.unwrap();

        // A surviving reply to alice's chirp — its reply_to must become None when a1 is removed.
        let reply = repo
            .create_chirp(
                carol.id,
                CreateChirpRequest {
                    text: ChirpText::parse("reply to alice").unwrap(),
                    reply_to: Some(a1.id),
                },
            )
            .await
            .unwrap();

        // Transient per-account state: a session, a durable credential, and a recovery challenge.
        let session = repo
            .login(LoginRequest {
                username: Username::parse("alice").unwrap(),
            })
            .await
            .unwrap()
            .token;
        repo.set_credential(alice.id, "secret").await.unwrap();
        repo.begin_account_recovery(&Username::parse("alice").unwrap())
            .await
            .unwrap();

        // Sanity BEFORE deletion: the denormalized counts reflect the seeded edges.
        assert_eq!(repo.get_user(bob.id).await.unwrap().follower_count, 1);
        assert_eq!(repo.get_user(carol.id).await.unwrap().following_count, 1);

        // --- Delete alice: the whole cascade is one atomic critical section. ---
        repo.delete_account(alice.id)
            .await
            .expect("delete_account succeeds");

        // The account itself is gone.
        assert!(matches!(
            repo.get_user(alice.id).await.unwrap_err(),
            ServerError::NotFound(_)
        ));

        // Surviving neighbours' counts are reconciled to match their current edge sets.
        assert_eq!(
            repo.get_user(bob.id).await.unwrap().follower_count,
            0,
            "alice's follow of bob is gone, so bob's follower_count reconciles to 0"
        );
        assert_eq!(
            repo.get_user(carol.id).await.unwrap().following_count,
            0,
            "carol's follow of alice is gone, so carol's following_count reconciles to 0"
        );
        // The control edge bob→carol is untouched by alice's deletion.
        assert_eq!(repo.get_user(bob.id).await.unwrap().following_count, 1);
        assert_eq!(repo.get_user(carol.id).await.unwrap().follower_count, 1);

        {
            let store = repo.lock().unwrap();
            // b1 survives; its like_count reconciles to 0 (alice's like removed).
            let b1_stored = store.chirps.get(&b1.id).expect("bob's chirp survives");
            assert_eq!(
                b1_stored.like_count, 0,
                "alice's like of the surviving chirp is reconciled away"
            );
            // alice's chirp and everything cascading off it are gone.
            assert!(
                !store.chirps.contains_key(&a1.id),
                "alice's chirp is cascaded away"
            );
            assert!(
                !store.likes.iter().any(|(_, chirp)| *chirp == a1.id),
                "likes ON the removed chirp are cascaded away"
            );
            assert!(
                !store.likes.iter().any(|(liker, _)| *liker == alice.id),
                "likes BY the deleted user are cascaded away"
            );
            // The surviving reply lost its target (ON DELETE SET NULL analogue).
            let reply_stored = store.chirps.get(&reply.id).expect("carol's reply survives");
            assert_eq!(reply_stored.reply_to, None);
            // No follow edge references alice anymore; the control edge remains.
            assert!(
                !store
                    .follows
                    .iter()
                    .any(|(f, t)| *f == alice.id || *t == alice.id),
                "follow edges on either side of the deleted user are cascaded away"
            );
            assert!(store.follows.contains(&(bob.id, carol.id)));
            // Transient per-account state is gone.
            assert!(!store.credentials.contains_key(&alice.id));
            assert!(!store.recovery_tokens.values().any(|c| c.user == alice.id));
            assert!(!store.tokens.values().any(|s| s.user == alice.id));
            // The handle index entry is freed, so the handle could be re-registered.
            assert!(!store.username_to_id.contains_key("alice"));
        }

        // The deleted account's session no longer resolves.
        assert!(matches!(
            repo.resolve_token(&session).await.unwrap_err(),
            ServerError::Unauthorized(_)
        ));
    }

    #[tokio::test]
    async fn delete_account_is_not_found_for_an_absent_or_already_deleted_account() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;

        // An account that never existed is NotFound (no panic).
        assert!(matches!(
            repo.delete_account(UserId::new(9999)).await.unwrap_err(),
            ServerError::NotFound(_)
        ));

        // The first deletion succeeds; a second on the now-deleted account is NotFound — never a
        // silent no-op, matching the trait's documented error contract.
        repo.delete_account(alice.id)
            .await
            .expect("first delete succeeds");
        assert!(matches!(
            repo.delete_account(alice.id).await.unwrap_err(),
            ServerError::NotFound(_)
        ));
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

    #[tokio::test]
    async fn set_account_suspension_sets_and_clears_the_target_state() {
        let repo = InMemoryRepository::new();
        let moderator = seed_user(&repo, "moderator").await;
        let target = seed_user(&repo, "target").await;

        assert!(!repo.is_suspended(target.id), "accounts start unsuspended");

        repo.set_account_suspension(moderator.id, target.id, true)
            .await
            .expect("a moderator can suspend another account");
        assert!(repo.is_suspended(target.id), "the target is now suspended");

        repo.set_account_suspension(moderator.id, target.id, false)
            .await
            .expect("a moderator can clear the suspension");
        assert!(!repo.is_suspended(target.id), "the suspension is cleared");

        // Moderating another account never suspends the acting moderator.
        assert!(!repo.is_suspended(moderator.id));
    }

    #[tokio::test]
    async fn set_account_suspension_is_not_found_for_an_absent_target() {
        let repo = InMemoryRepository::new();
        let moderator = seed_user(&repo, "moderator").await;
        let err = repo
            .set_account_suspension(moderator.id, UserId::new(999), true)
            .await
            .expect_err("no such target");
        assert!(matches!(err, ServerError::NotFound(_)));
    }

    #[tokio::test]
    async fn set_account_suspension_is_forbidden_when_actor_targets_themselves() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;
        let err = repo
            .set_account_suspension(alice.id, alice.id, true)
            .await
            .expect_err("a self-targeted moderation is refused");
        assert!(matches!(err, ServerError::Forbidden(_)));
        // The refused action left no suspension behind.
        assert!(!repo.is_suspended(alice.id));
    }

    #[tokio::test]
    async fn store_health_is_ok_for_the_in_process_store() {
        // The in-memory store is in-process and always reachable, so its health is trivially `Ok` —
        // a truthful result (the store genuinely is serving), not a placeholder. This keeps the
        // store-free `cargo test` suite exercising the retained implementor's `store_health` parity.
        let repo = InMemoryRepository::new();
        repo.store_health()
            .await
            .expect("the in-process store is always healthy");
    }

    // ---- Chirp removal (store-free) -------------------------------------------------------------

    #[tokio::test]
    async fn remove_chirp_excludes_it_from_the_timeline_and_is_idempotent() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;
        let keep = seed_chirp(&repo, alice.id, "kept").await;
        let gone = seed_chirp(&repo, alice.id, "removed").await;

        // Both chirps are visible before removal.
        let before = repo
            .timeline(alice.id, TimelineQuery::default())
            .await
            .unwrap();
        assert_eq!(
            before.items.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec![gone.id, keep.id]
        );

        repo.remove_chirp(alice.id, gone.id)
            .await
            .expect("the author removes their own chirp");

        // The removed chirp is no longer served on the timeline; the other remains.
        let after = repo
            .timeline(alice.id, TimelineQuery::default())
            .await
            .unwrap();
        assert_eq!(
            after.items.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec![keep.id]
        );

        // Idempotent: removing an already-removed chirp succeeds.
        repo.remove_chirp(alice.id, gone.id)
            .await
            .expect("re-removal is an idempotent no-op");
    }

    #[tokio::test]
    async fn remove_chirp_is_not_found_for_an_absent_chirp() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;
        let err = repo
            .remove_chirp(alice.id, ChirpId::new(4242))
            .await
            .expect_err("no such chirp");
        assert!(matches!(err, ServerError::NotFound(_)));
    }

    // ---- Data export (store-free) ---------------------------------------------------------------

    #[tokio::test]
    async fn export_user_data_captures_profile_chirps_graph_and_likes_excluding_removed() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;
        let bob = seed_user(&repo, "bob").await;
        let carol = seed_user(&repo, "carol").await;

        let c1 = seed_chirp(&repo, alice.id, "alice one").await;
        let c2 = seed_chirp(&repo, alice.id, "alice two").await;
        let removed = seed_chirp(&repo, alice.id, "to be removed").await;
        repo.remove_chirp(alice.id, removed.id).await.unwrap();

        // alice follows bob; carol follows alice; alice likes bob's chirp.
        repo.follow(alice.id, bob.id).await.unwrap();
        repo.follow(carol.id, alice.id).await.unwrap();
        let bob_chirp = seed_chirp(&repo, bob.id, "bob chirp").await;
        repo.like(alice.id, bob_chirp.id).await.unwrap();

        let json = repo
            .export_user_data(alice.id)
            .await
            .expect("export succeeds for an existing user");
        let doc: crate::repository::export::ExportDocument =
            serde_json::from_str(&json).expect("the export is valid JSON of the export shape");

        // The export captures the CURRENT profile (its denormalized counts reflect the edges added
        // above), so compare against a fresh read rather than the pre-follow snapshot.
        assert_eq!(doc.profile, repo.get_user(alice.id).await.unwrap());
        // Authored, non-removed chirps, newest-first; the removed chirp is excluded.
        assert_eq!(
            doc.chirps.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec![c2.id, c1.id]
        );
        assert_eq!(doc.following, vec![bob.id]);
        assert_eq!(doc.followers, vec![carol.id]);
        assert_eq!(doc.likes, vec![bob_chirp.id]);
    }

    #[tokio::test]
    async fn export_user_data_is_not_found_for_an_absent_user() {
        let repo = InMemoryRepository::new();
        let err = repo
            .export_user_data(UserId::new(999))
            .await
            .expect_err("no such user");
        assert!(matches!(err, ServerError::NotFound(_)));
    }

    // ---- Follower / following listing (store-free) ----------------------------------------------

    #[tokio::test]
    async fn list_followers_and_following_return_the_right_accounts_newest_first() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;
        let bob = seed_user(&repo, "bob").await;
        let carol = seed_user(&repo, "carol").await;

        // bob and carol follow alice; alice follows carol.
        repo.follow(bob.id, alice.id).await.unwrap();
        repo.follow(carol.id, alice.id).await.unwrap();
        repo.follow(alice.id, carol.id).await.unwrap();

        // alice's followers are bob and carol, newest-account-first (higher id first): carol, bob.
        let followers = repo
            .list_followers(alice.id, TimelineQuery::default())
            .await
            .unwrap();
        assert_eq!(
            followers.items.iter().map(|u| u.id).collect::<Vec<_>>(),
            vec![carol.id, bob.id]
        );
        assert!(followers.next_cursor.is_none());

        // alice follows only carol.
        let following = repo
            .list_following(alice.id, TimelineQuery::default())
            .await
            .unwrap();
        assert_eq!(
            following.items.iter().map(|u| u.id).collect::<Vec<_>>(),
            vec![carol.id]
        );
    }

    #[tokio::test]
    async fn list_followers_paginates_exactly_once_with_the_cursor() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;

        // Five accounts all follow alice.
        let mut followers = Vec::new();
        for i in 0..5 {
            let u = seed_user(&repo, &format!("f{i}")).await;
            repo.follow(u.id, alice.id).await.unwrap();
            followers.push(u);
        }

        // Page through two at a time and collect; the union must be every follower exactly once.
        let mut seen = Vec::new();
        let mut cursor = None;
        loop {
            let page = repo
                .list_followers(
                    alice.id,
                    TimelineQuery {
                        limit: Some(2),
                        cursor: cursor.clone(),
                    },
                )
                .await
                .unwrap();
            seen.extend(page.items.iter().map(|u| u.id));
            match page.next_cursor {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }

        let mut expected: Vec<UserId> = followers.iter().map(|u| u.id).collect();
        expected.sort_by_key(|id| std::cmp::Reverse(id.get()));
        assert_eq!(
            seen, expected,
            "every follower appears exactly once, newest-first"
        );
    }

    #[tokio::test]
    async fn listing_is_not_found_for_an_absent_user() {
        let repo = InMemoryRepository::new();
        assert!(matches!(
            repo.list_followers(UserId::new(1), TimelineQuery::default())
                .await
                .unwrap_err(),
            ServerError::NotFound(_)
        ));
        assert!(matches!(
            repo.list_following(UserId::new(1), TimelineQuery::default())
                .await
                .unwrap_err(),
            ServerError::NotFound(_)
        ));
    }

    // ---- Transient-state lifetime + sweep (REQ-OPS-010, #67 / BAR-022) --------------------------
    //
    // These exercise the store-side half of the transient-lifetime guarantee against the in-memory
    // backing (store-free): a session carries a bounded, enforced lifetime; an expired session is
    // not served and is swept; expired recovery tokens are swept; and the sweep never touches live
    // user content. Expiry is injected directly (a past-dated row) rather than by waiting, mirroring
    // the durable backing's expired-row injection tests.

    /// Directly injects a session row with an explicit expiry, so expiry-honouring and the sweep can
    /// be tested without waiting out `SESSION_LIFETIME_MILLIS`.
    fn inject_session(
        repo: &InMemoryRepository,
        token: &str,
        user: UserId,
        expires_at_millis: i64,
    ) {
        repo.lock().expect("store lock").tokens.insert(
            token.to_owned(),
            Session {
                user,
                expires_at: Some(Timestamp::from_millis(expires_at_millis)),
            },
        );
    }

    /// Directly injects a recovery-token challenge with an explicit expiry.
    fn inject_recovery_token(
        repo: &InMemoryRepository,
        token: &str,
        user: UserId,
        expires_at_millis: i64,
    ) {
        repo.lock().expect("store lock").recovery_tokens.insert(
            token.to_owned(),
            RecoveryChallenge {
                user,
                expires_at: Timestamp::from_millis(expires_at_millis),
            },
        );
    }

    #[tokio::test]
    async fn login_stamps_a_bounded_session_lifetime_that_resolves_while_valid() {
        let repo = InMemoryRepository::new();
        let user = seed_user(&repo, "alice").await;

        let auth = repo
            .login(LoginRequest {
                username: user.username.clone(),
            })
            .await
            .expect("login succeeds");

        // The freshly-issued session carries a bounded, future expiry and resolves.
        assert_eq!(
            repo.resolve_token(&auth.token).await.expect("resolves"),
            user.id
        );
        let expiry = repo
            .lock()
            .expect("store lock")
            .tokens
            .get(&auth.token.0)
            .and_then(|s| s.expires_at)
            .expect("a bounded expiry is stamped at login");
        assert!(
            expiry.as_millis() > now_millis().as_millis(),
            "the enforced session lifetime is in the future"
        );
    }

    #[tokio::test]
    async fn an_expired_session_is_not_resolved_and_is_swept_while_a_live_one_survives() {
        let repo = InMemoryRepository::new();
        let user = seed_user(&repo, "alice").await;

        // A live session (issued by login) and a directly-injected already-expired one.
        let live = repo
            .login(LoginRequest {
                username: user.username.clone(),
            })
            .await
            .expect("login succeeds");
        let past = now_millis().as_millis() - 1;
        inject_session(&repo, "tok-expired", user.id, past);

        // Expiry is honoured on read before any sweep: the expired session is treated as absent.
        assert!(matches!(
            repo.resolve_token(&SessionToken("tok-expired".to_owned()))
                .await
                .unwrap_err(),
            ServerError::Unauthorized(_)
        ));
        // The live session still resolves.
        assert_eq!(
            repo.resolve_token(&live.token).await.expect("resolves"),
            user.id
        );

        // The sweep reclaims exactly the expired session and leaves the live one.
        let swept = repo
            .sweep_expired_transient_state()
            .await
            .expect("sweep succeeds");
        assert_eq!(swept.sessions, 1, "the expired session row is reclaimed");
        assert_eq!(swept.recovery_tokens, 0);
        assert_eq!(swept.export_artifacts, 0);
        assert_eq!(
            repo.resolve_token(&live.token)
                .await
                .expect("still resolves"),
            user.id,
            "the live session is untouched by the sweep"
        );

        // A second sweep reclaims nothing — the first already removed the expired row.
        assert_eq!(
            repo.sweep_expired_transient_state()
                .await
                .expect("sweep succeeds")
                .total(),
            0
        );
    }

    #[tokio::test]
    async fn sweep_reclaims_expired_recovery_tokens_and_keeps_valid_ones() {
        let repo = InMemoryRepository::new();
        let user = seed_user(&repo, "alice").await;
        let now = now_millis().as_millis();

        inject_recovery_token(&repo, "rec-expired", user.id, now - 1);
        inject_recovery_token(&repo, "rec-valid", user.id, now + SESSION_LIFETIME_MILLIS);

        let swept = repo
            .sweep_expired_transient_state()
            .await
            .expect("sweep succeeds");
        assert_eq!(
            swept.recovery_tokens, 1,
            "only the expired challenge is reclaimed"
        );
        assert_eq!(swept.sessions, 0);

        let remaining = repo.lock().expect("store lock");
        assert!(!remaining.recovery_tokens.contains_key("rec-expired"));
        assert!(
            remaining.recovery_tokens.contains_key("rec-valid"),
            "the unexpired challenge survives the sweep"
        );
    }

    #[tokio::test]
    async fn sweep_never_removes_live_user_content() {
        let repo = InMemoryRepository::new();
        let alice = seed_user(&repo, "alice").await;
        let bob = seed_user(&repo, "bob").await;
        let chirp = seed_chirp(&repo, alice.id, "hello world").await;
        repo.follow(alice.id, bob.id).await.expect("follow");
        repo.like(bob.id, chirp.id).await.expect("like");

        // Live transient state alongside the content, plus one expired session to force a non-empty
        // sweep so we prove the sweep reclaims transient rows WITHOUT disturbing content.
        let live = repo
            .login(LoginRequest {
                username: alice.username.clone(),
            })
            .await
            .expect("login");
        inject_session(&repo, "tok-expired", alice.id, now_millis().as_millis() - 1);

        let swept = repo
            .sweep_expired_transient_state()
            .await
            .expect("sweep succeeds");
        assert_eq!(swept.sessions, 1, "the one expired session is reclaimed");

        // All user content survives untouched.
        assert_eq!(repo.get_user(alice.id).await.expect("alice").id, alice.id);
        assert_eq!(repo.get_user(bob.id).await.expect("bob").id, bob.id);
        let timeline = repo
            .timeline(alice.id, TimelineQuery::default())
            .await
            .expect("timeline");
        assert_eq!(timeline.items.len(), 1, "alice's chirp is retained");
        assert_eq!(timeline.items[0].id, chirp.id);
        assert_eq!(timeline.items[0].like_count, 1, "the like edge is retained");
        assert_eq!(
            repo.get_user(bob.id).await.expect("bob").follower_count,
            1,
            "the follow edge is retained"
        );
        // The live session is still valid; only transient expired state was reclaimed.
        assert_eq!(
            repo.resolve_token(&live.token).await.expect("resolves"),
            alice.id
        );
    }
}
