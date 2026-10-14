//! The durable PostgreSQL backing for [`ChirpRepository`] — compiling scaffold (EPIC-PERSIST, #66).
//!
//! This module is the second implementor of the persistence seam, alongside the default
//! [`InMemoryRepository`](crate::repository::InMemoryRepository). It is the config-selected
//! **production** backing per [ADR-0001](../../../../design/adr/0001-durable-store-and-persistence-posture.md):
//! a PostgreSQL database reached over the network with the async [`sqlx`] driver, so the store is
//! pod-independent (a `chirp-server` restart, redeploy, or reschedule never touches the data).
//!
//! # Scaffold boundary
//!
//! This is the **foundation scaffold**, not the durable implementation. What it stands up:
//!
//! * a connection pool constructed from a connection string ([`PostgresRepository::connect`]);
//! * a **versioned, inspectable schema** driven by [`sqlx`]'s embedded, forward-only migration
//!   runner (the `migrations/` directory, applied at startup) with a readable
//!   [schema-version surface](PostgresRepository::schema_version);
//! * a truthful [`store_health`](PostgresRepository::store_health) that round-trips the live pool.
//!
//! What it deliberately does **not** do: the durable method **bodies**. Every
//! [`ChirpRepository`] write/read below is a skeleton stub that returns
//! [`ServerError::Internal`] — error-contract-consistent, so an operation whose durable behaviour is
//! not yet built reports a **failed** action and NEVER falsely confirms success (the
//! `confirmed_write_durable` identity). The real DDL, migration content, and method bodies are the
//! downstream EPIC-PERSIST `SUBTASK_durable_impl` work; this scaffold only makes them compile
//! against a live, migratable store.
//!
//! The whole module is gated behind the optional `postgres` cargo feature, so the default
//! workspace build and `cargo test` pull in neither [`sqlx`] nor a database.

use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use chirp_types::api::{
    AuthResponse, CreateChirpRequest, CreateUserRequest, Cursor, FollowResponse, LikeResponse,
    LoginRequest, Page, SessionToken, TimelineQuery,
};
use chirp_types::domain::{Chirp, User, Username};
use chirp_types::ids::{ChirpId, Timestamp, UserId};

use crate::error::ServerError;
use crate::repository::{export, ChirpRepository, DEFAULT_TIMELINE_LIMIT, MAX_TIMELINE_LIMIT};

/// The embedded, forward-only migration set for the durable store.
///
/// [`sqlx::migrate!`] reads the crate-local `migrations/` directory **at compile time** and embeds
/// every `NNNN_description.sql` file into the binary, so a deployment carries its own schema and
/// needs no migration files on disk. The set is applied by [`PostgresRepository::connect`] at
/// startup; see the module docs and ADR-0001 §3 for the forward-only, version-tracked approach.
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

/// Durable [`ChirpRepository`] backed by a PostgreSQL database over [`sqlx`].
///
/// Holds a single shared [`PgPool`]; the pool is `Send + Sync` and used through `&self`, so the
/// repository drops into the transport layer's `Arc<dyn ChirpRepository>` shape with no change to
/// the trait's object-safety or signatures. Construct it with [`PostgresRepository::connect`],
/// which also brings the schema up to date by running the embedded migration set.
pub struct PostgresRepository {
    /// The shared connection pool to the PostgreSQL primary. All durable operations borrow it
    /// through `&self`.
    pool: PgPool,
}

impl PostgresRepository {
    /// Connects to the PostgreSQL database at `database_url`, runs the embedded forward-only
    /// migrations to bring the schema up to date, and returns the ready repository.
    ///
    /// Running the migrator is **idempotent**: on a fresh database every migration is applied in
    /// order; on an up-to-date database none is applied and the call is a no-op. Migrations run
    /// *before* the repository is handed to the transport layer, so the server never serves against
    /// an un-migrated schema.
    ///
    /// # Errors
    ///
    /// [`ServerError::Internal`] if the pool cannot be established (bad connection string,
    /// unreachable primary, exhausted capacity) or if applying the migrations fails. A connection
    /// or migration failure is surfaced as a failed startup, never swallowed.
    pub async fn connect(database_url: &str) -> Result<Self, ServerError> {
        let pool = PgPoolOptions::new()
            .max_connections(DEFAULT_MAX_CONNECTIONS)
            .connect(database_url)
            .await
            .map_err(|error| {
                ServerError::Internal(format!("failed to connect to the durable store: {error}"))
            })?;

        MIGRATOR.run(&pool).await.map_err(|error| {
            ServerError::Internal(format!("failed to apply durable-store migrations: {error}"))
        })?;

        Ok(Self { pool })
    }

    /// Builds a repository over an already-constructed [`PgPool`] **without** running migrations.
    ///
    /// For callers that own pool construction and migration themselves (for example a test harness
    /// or an operator tool that migrates out of band). Prefer [`connect`](Self::connect) for the
    /// normal startup path, which also migrates.
    #[must_use]
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Reads the current durable **schema version** — the greatest applied migration version, as
    /// tracked in [`sqlx`]'s `_sqlx_migrations` table — for inspection and health reporting.
    ///
    /// Returns `Ok(None)` when no migration has been applied yet (an empty database before the
    /// first [`connect`](Self::connect)), and `Ok(Some(version))` otherwise. This is the inspectable
    /// schema-version surface the release bar requires (BAR-021 / REQ-DOM-022).
    ///
    /// # Errors
    ///
    /// [`ServerError::Internal`] if the version query cannot be executed (for example the backing
    /// store is unreachable, or the migration-tracking table does not yet exist).
    pub async fn schema_version(&self) -> Result<Option<i64>, ServerError> {
        sqlx::query_scalar::<_, Option<i64>>("SELECT MAX(version) FROM _sqlx_migrations")
            .fetch_one(&self.pool)
            .await
            .map_err(|error| {
                ServerError::Internal(format!("failed to read durable schema version: {error}"))
            })
    }
}

/// The default upper bound on pooled connections to the durable primary.
///
/// A conservative default for the scaffold; connection-pool sizing is an operational concern the
/// downstream operational-readiness work tunes (it may be made configurable alongside
/// `DATABASE_URL`).
const DEFAULT_MAX_CONNECTIONS: u32 = 10;

/// Current wall-clock time as epoch-milliseconds, for stamping a newly created durable entity.
///
/// Mirrors the in-memory backing's `now_millis`: a pre-epoch system clock (which does not occur in
/// practice) degrades to `0` rather than panicking, keeping the durable path panic-free.
fn now_millis() -> Timestamp {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as i64)
        .unwrap_or(0);
    Timestamp::from_millis(millis)
}

/// The default validity window, in milliseconds, stamped onto a freshly-issued recovery challenge.
///
/// The committed schema makes `recovery_tokens.expires_at` `NOT NULL` (REQ-OPS-010: a recovery token
/// must not be usable past a documented validity), so this stage MUST stamp every issued challenge
/// with an expiry and MUST reject an expired one at
/// [`complete_account_recovery`](ChirpRepository::complete_account_recovery). This constant is the
/// durable default applied at issuance; the **authoritative** validity *value* — like the credential
/// crypto scheme and the session lifetime — is owned by EPIC-IDENTITY (`REQ-ID-*`), and sweeping
/// expired rows is owned by the retention stage. This stage persists and enforces an expiry; it does
/// not own the policy value. The in-memory backing mirrors this window so the two implementors agree.
const RECOVERY_TOKEN_VALIDITY_MILLIS: i64 = 24 * 60 * 60 * 1000;

/// The single, indistinguishable [`Unauthorized`](ServerError::Unauthorized) returned by
/// [`complete_account_recovery`](ChirpRepository::complete_account_recovery) for every validation
/// failure — unknown handle, unknown or mismatched token, or expired token.
///
/// The durable store never discloses which of those held, so an attacker can neither enumerate
/// accounts nor probe recovery-token state (fail-secure, mirroring `verify_credential`).
fn unauthorized_recovery() -> ServerError {
    ServerError::Unauthorized("invalid or expired recovery token".to_owned())
}

#[async_trait]
impl ChirpRepository for PostgresRepository {
    // ---- Existing capability_set (scaffold stubs) -----------------------------------------------
    // The durable bodies are downstream; each stub reports a failed action and never falsely
    // confirms success, preserving the error contract and the `confirmed_write_durable` identity.

    /// Durably registers a new user: allocates an opaque server id, stamps the creation time, and
    /// inserts the row inside a confirmed-write transaction, returning the created [`User`] only
    /// after the write has durably committed.
    ///
    /// The id is allocated from `user_id_seq` on the transaction's own connection, so the
    /// allocation and the insert are one atomic unit. The unique-handle index (`users_username_key`)
    /// makes a duplicate handle a unique violation, which is mapped to
    /// [`Conflict`](ServerError::Conflict); the row is lowered through the committed mapping pattern
    /// ([`mapping::UserRow::from_user`]) so `chirp-types` is the single source of the column shape.
    /// The `CreateUserRequest.username` is a [`Username`] — unrepresentable-if-invalid — so an
    /// invalid handle cannot reach the store through this path (`domain_boundary_validated`).
    ///
    /// # Errors
    ///
    /// [`Conflict`](ServerError::Conflict) if the handle is already taken;
    /// [`Internal`](ServerError::Internal) if the id cannot be allocated or the write cannot be
    /// durably committed — an undurable write is reported as failed, never falsely confirmed
    /// (`confirmed_write_durable`).
    async fn create_user(&self, request: CreateUserRequest) -> Result<User, ServerError> {
        self.with_transaction(move |conn| {
            Box::pin(async move {
                let id = PostgresRepository::allocate_user_id(&mut *conn).await?;
                let user = User {
                    id,
                    username: request.username,
                    display_name: request.display_name,
                    bio: request.bio,
                    created_at: now_millis(),
                    follower_count: 0,
                    following_count: 0,
                };
                // Lower through the committed mapping pattern; counts/moderation columns take their
                // schema defaults (0 / not-suspended) for a freshly created account.
                let row = mapping::UserRow::from_user(&user)?;
                sqlx::query(
                    "INSERT INTO users (id, username, display_name, bio, created_at) \
                     VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(row.id)
                .bind(row.username)
                .bind(row.display_name)
                .bind(row.bio)
                .bind(row.created_at)
                .execute(&mut *conn)
                .await
                .map_err(|error| {
                    if mapping::is_unique_violation(&error) {
                        ServerError::Conflict(format!(
                            "username '{}' is already taken",
                            user.username
                        ))
                    } else {
                        ServerError::Internal(format!("failed to insert user: {error}"))
                    }
                })?;
                Ok(user)
            })
        })
        .await
    }

    /// Fetches a user's public profile by id, lifting the stored row back into a validated
    /// [`User`] through the committed mapping pattern.
    ///
    /// A plain pooled read (no transaction is needed for a single-statement lookup). The row is
    /// lifted through [`mapping::UserRow::into_user`], which re-parses the stored handle at the
    /// durable boundary, so a corrupt row surfaces as an error rather than a malformed value or a
    /// panic (`domain_boundary_validated`).
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if no user has that id;
    /// [`Validation`](ServerError::Validation) if a stored row violates a `chirp-types` invariant;
    /// [`Internal`](ServerError::Internal) if the read fails or a stored id/count column is corrupt.
    async fn get_user(&self, id: UserId) -> Result<User, ServerError> {
        let row: Option<mapping::UserRow> = sqlx::query_as(
            "SELECT id, username, display_name, bio, created_at, follower_count, \
             following_count, suspended, suspended_at FROM users WHERE id = $1",
        )
        .bind(mapping::user_id_to_column(id)?)
        .fetch_optional(self.pool())
        .await
        .map_err(|error| ServerError::Internal(format!("failed to read user {id}: {error}")))?;

        match row {
            Some(row) => row.into_user(),
            None => Err(ServerError::NotFound(format!("user {id}"))),
        }
    }

    /// Authenticates by handle and issues a **durable** session: resolves the handle to its user,
    /// mints an opaque token from the durable counter, and persists the session row inside a
    /// confirmed-write transaction, returning the [`AuthResponse`] only after the session has durably
    /// committed (`confirmed_write_durable`).
    ///
    /// The handle lookup and the session insert run in the **same** transaction, so the
    /// [`NotFound`](ServerError::NotFound) decision and the write are one atomic unit, and the stored
    /// user row is lifted back through the committed mapping ([`mapping::UserRow::into_user`]) so a
    /// corrupt stored row surfaces as an error, never a panic (`domain_boundary_validated`). The token
    /// is composed from the durable `token_seq` value allocated on the transaction's own connection —
    /// opaque, server-assigned, monotonic, stable across restart, never reused
    /// (`ids_opaque_monotonic_stable`). The session's `expires_at` is deliberately left unset: this
    /// stage **persists and resolves** sessions durably; the session *lifetime value* and its
    /// enforcement are owned by EPIC-IDENTITY and the retention stage (`REQ-ID-*`), not decided here.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if no user has the requested handle;
    /// [`Internal`](ServerError::Internal) if the token cannot be allocated or the session cannot be
    /// durably committed — an undurable write is reported as failed, never falsely confirmed.
    async fn login(&self, request: LoginRequest) -> Result<AuthResponse, ServerError> {
        let created_at = mapping::timestamp_to_column(now_millis());
        self.with_transaction(move |conn| {
            Box::pin(async move {
                // Resolve the handle to its stored row INSIDE the transaction, so NotFound and the
                // session insert are one atomic decision. The row is lifted back through the
                // committed mapping (a corrupt stored handle surfaces as an error, never a panic).
                let row: Option<mapping::UserRow> = sqlx::query_as(
                    "SELECT id, username, display_name, bio, created_at, follower_count, \
                     following_count, suspended, suspended_at FROM users WHERE username = $1",
                )
                .bind(request.username.as_str())
                .fetch_optional(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!(
                        "failed to read user '{}' for login: {error}",
                        request.username
                    ))
                })?;
                let user = match row {
                    Some(row) => row.into_user()?,
                    None => {
                        return Err(ServerError::NotFound(format!(
                            "user '{}'",
                            request.username
                        )))
                    }
                };

                // Mint an opaque token from the durable `token_seq` on the SAME connection, so the
                // allocation and the session insert are one atomic unit. The value is monotonic and
                // never reused, so the composed token string is globally unique.
                let token_value = PostgresRepository::allocate_token_value(&mut *conn).await?;
                let token = format!("tok-{token_value}");

                // `expires_at` is NULL: lifetime policy/enforcement is EPIC-IDENTITY / retention, not
                // this stage. The session simply exists durably until explicitly invalidated.
                sqlx::query(
                    "INSERT INTO sessions (token, user_id, created_at, expires_at) \
                     VALUES ($1, $2, $3, $4)",
                )
                .bind(&token)
                .bind(mapping::user_id_to_column(user.id)?)
                .bind(created_at)
                .bind(Option::<i64>::None)
                .execute(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!(
                        "failed to durably record a session for user {}: {error}",
                        user.id
                    ))
                })?;

                Ok(AuthResponse {
                    user,
                    token: SessionToken(token),
                })
            })
        })
        .await
    }

    /// Resolves a presented bearer token to its authenticated [`UserId`] from the durable `sessions`
    /// store.
    ///
    /// A single pooled read (no transaction is needed for a one-statement lookup); the token is bound
    /// as a parameter (never concatenated), so the lookup carries no injection surface. A token not
    /// present in the store — never issued, or already invalidated — is reported
    /// [`Unauthorized`](ServerError::Unauthorized). Session *expiry* is not enforced here: the
    /// lifetime policy and its enforcement belong to EPIC-IDENTITY / the retention stage; this stage
    /// resolves a persisted, non-invalidated session. The stored id is lifted back through the
    /// committed mapping, so a corrupt id column surfaces as an error rather than a bogus id.
    ///
    /// # Errors
    ///
    /// [`Unauthorized`](ServerError::Unauthorized) if the token is unknown or no longer valid;
    /// [`Internal`](ServerError::Internal) if the read fails or a stored id column is corrupt.
    async fn resolve_token(&self, token: &SessionToken) -> Result<UserId, ServerError> {
        let row: Option<(i64,)> = sqlx::query_as("SELECT user_id FROM sessions WHERE token = $1")
            .bind(token.0.as_str())
            .fetch_optional(self.pool())
            .await
            .map_err(|error| {
                ServerError::Internal(format!("failed to resolve a session token: {error}"))
            })?;

        match row {
            Some((user_id,)) => mapping::column_to_user_id(user_id),
            None => Err(ServerError::Unauthorized(
                "invalid or expired session token".to_owned(),
            )),
        }
    }

    /// Durably posts a chirp: verifies the author exists and, for a reply, that the reply target
    /// exists; allocates an opaque chirp id; and inserts the row with a **zeroed `like_count`** —
    /// all inside one confirmed-write transaction, returning the created [`Chirp`] only after the
    /// write has durably committed (`confirmed_write_durable`).
    ///
    /// The author and reply-target existence checks run in the **same** transaction as the id
    /// allocation and insert, so the [`NotFound`](ServerError::NotFound) decisions and the write are
    /// one atomic unit (the `chirps.author_id` / `chirps.reply_to` foreign keys are the in-store
    /// backstop). `request.text` is a [`ChirpText`](chirp_types::domain::ChirpText) —
    /// unrepresentable-if-invalid — so an invalid body cannot reach the store (`domain_boundary_validated`).
    /// The id is drawn from `chirp_id_seq` on the transaction's own connection (opaque, monotonic,
    /// stable across restart, never reused — `ids_opaque_monotonic_stable`), and the row is lowered
    /// through the committed mapping ([`mapping::ChirpRow::from_chirp`]) so `chirp-types` is the
    /// single source of the column shape. The denormalized `like_count` is **initialised to 0** here;
    /// its atomic maintenance under like/unlike is the graph stage's.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if the author does not exist, or the reply target does not
    /// exist; [`Internal`](ServerError::Internal) if the id cannot be allocated or the write cannot be
    /// durably committed — an undurable write is reported as failed, never falsely confirmed.
    async fn create_chirp(
        &self,
        author: UserId,
        request: CreateChirpRequest,
    ) -> Result<Chirp, ServerError> {
        let created_at = now_millis();
        self.with_transaction(move |conn| {
            Box::pin(async move {
                let author_column = mapping::user_id_to_column(author)?;
                // Author existence, decided in the same transaction as the write.
                let author_exists: bool =
                    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
                        .bind(author_column)
                        .fetch_one(&mut *conn)
                        .await
                        .map_err(|error| {
                            ServerError::Internal(format!(
                                "failed to check author {author} before posting a chirp: {error}"
                            ))
                        })?;
                if !author_exists {
                    return Err(ServerError::NotFound(format!("author {author}")));
                }

                // Reply-target existence (referential integrity, REQ-DOM-007), when this is a reply.
                if let Some(parent) = request.reply_to {
                    let parent_exists: bool =
                        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM chirps WHERE id = $1)")
                            .bind(mapping::chirp_id_to_column(parent)?)
                            .fetch_one(&mut *conn)
                            .await
                            .map_err(|error| {
                                ServerError::Internal(format!(
                                    "failed to check reply target {parent}: {error}"
                                ))
                            })?;
                    if !parent_exists {
                        return Err(ServerError::NotFound(format!("reply target {parent}")));
                    }
                }

                // Allocate the opaque chirp id on the SAME connection, so allocation and insert are
                // one atomic unit.
                let id = PostgresRepository::allocate_chirp_id(&mut *conn).await?;
                let chirp = Chirp {
                    id,
                    author_id: author,
                    text: request.text,
                    created_at,
                    like_count: 0,
                    reply_to: request.reply_to,
                };
                // Lower through the committed mapping; `like_count` is initialised to 0 and the
                // moderation columns take their schema defaults (not removed).
                let row = mapping::ChirpRow::from_chirp(&chirp)?;
                sqlx::query(
                    "INSERT INTO chirps (id, author_id, text, created_at, like_count, reply_to) \
                     VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(row.id)
                .bind(row.author_id)
                .bind(row.text)
                .bind(row.created_at)
                .bind(row.like_count)
                .bind(row.reply_to)
                .execute(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!("failed to durably insert a chirp: {error}"))
                })?;

                Ok(chirp)
            })
        })
        .await
    }

    /// Returns one reverse-chronological page of `viewer`'s home timeline (the viewer's own chirps
    /// plus those of everyone the viewer follows), excluding soft-removed chirps, read as a
    /// consistent durable view.
    ///
    /// The viewer-existence check and the page read run in one read transaction, so an absent viewer
    /// is [`NotFound`](ServerError::NotFound) (not an empty page) against a consistent snapshot. The
    /// scan orders by `id DESC` — the stable reverse-chronological proxy the schema's
    /// `chirps_timeline_idx` supports and the in-memory backing uses — and takes `limit + 1` rows to
    /// learn whether a further page exists. `query.limit` is clamped to
    /// `1..=`[`MAX_TIMELINE_LIMIT`] (default [`DEFAULT_TIMELINE_LIMIT`]); `query.cursor`, when present
    /// and parseable, restricts the scan to `id <` the cursor, and a malformed cursor fails safe to
    /// the first page (cursors are server-minted). The returned
    /// [`next_cursor`](chirp_types::api::Page::next_cursor) is the last returned id iff more remain —
    /// so paging is **exactly-once** (no skips, no duplicates) and **stable across restart** (the ids
    /// are durable sequence values).
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if the viewer does not exist;
    /// [`Validation`](ServerError::Validation) if a stored row violates a `chirp-types` invariant;
    /// [`Internal`](ServerError::Internal) if the read fails or a stored column is corrupt.
    async fn timeline(
        &self,
        viewer: UserId,
        query: TimelineQuery,
    ) -> Result<Page<Chirp>, ServerError> {
        let limit = clamp_limit(query.limit);
        let cursor = decode_cursor(query.cursor.as_ref());
        let viewer_column = mapping::user_id_to_column(viewer)?;
        self.with_transaction(move |conn| {
            Box::pin(async move {
                let viewer_exists: bool =
                    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
                        .bind(viewer_column)
                        .fetch_one(&mut *conn)
                        .await
                        .map_err(|error| {
                            ServerError::Internal(format!(
                                "failed to check viewer {viewer} for the timeline: {error}"
                            ))
                        })?;
                if !viewer_exists {
                    return Err(ServerError::NotFound(format!("viewer {viewer}")));
                }

                let rows: Vec<mapping::ChirpRow> = sqlx::query_as(
                    "SELECT id, author_id, text, created_at, like_count, reply_to, removed, \
                     removed_at FROM chirps \
                     WHERE removed = FALSE \
                       AND (author_id = $1 \
                            OR author_id IN (SELECT followee_id FROM follows WHERE follower_id = $1)) \
                       AND ($2::BIGINT IS NULL OR id < $2) \
                     ORDER BY id DESC LIMIT $3",
                )
                .bind(viewer_column)
                .bind(cursor)
                .bind(fetch_limit(limit))
                .fetch_all(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!("failed to read the timeline: {error}"))
                })?;

                let chirps = rows
                    .into_iter()
                    .map(mapping::ChirpRow::into_chirp)
                    .collect::<Result<Vec<Chirp>, ServerError>>()?;
                Ok(paginate(chirps, limit, |chirp| chirp.id.get()))
            })
        })
        .await
    }

    /// Durably makes `follower` follow `followee`, maintaining the denormalized follow counts in the
    /// SAME confirmed-write transaction as the edge insert (`counts_match_edges`, `multiwrite_atomic`,
    /// `confirmed_write_durable`).
    ///
    /// The edge insert and both count updates commit as one atomic unit, so a reader never observes a
    /// `follower_count` / `following_count` that disagrees with the `follows` edge set, even under
    /// concurrency or a mid-operation failure. The write is **idempotent**: the `(follower, followee)`
    /// primary key makes a repeated follow an `ON CONFLICT DO NOTHING` no-op, and the counts move only
    /// on a genuine edge change (`rows_affected == 1`) — exactly mirroring the in-memory reference. The
    /// no-self-follow invariant is enforced at the durable boundary via
    /// [`mapping::ensure_distinct_users`] (the schema's `follows_no_self` CHECK is the in-store
    /// backstop), and both users' existence is decided inside the transaction so a missing account is a
    /// clean [`NotFound`](ServerError::NotFound) rather than a raw foreign-key error.
    ///
    /// # Errors
    ///
    /// [`Forbidden`](ServerError::Forbidden) if `follower == followee`;
    /// [`NotFound`](ServerError::NotFound) if either user does not exist;
    /// [`Internal`](ServerError::Internal) if the transaction cannot be durably committed — an
    /// undurable write is reported failed, never falsely confirmed.
    async fn follow(
        &self,
        follower: UserId,
        followee: UserId,
    ) -> Result<FollowResponse, ServerError> {
        // Domain-boundary invariant first, exactly as the in-memory implementor orders it: a
        // self-follow is Forbidden and never reaches the store.
        mapping::ensure_distinct_users(follower, followee)?;

        let created_at = mapping::timestamp_to_column(now_millis());
        self.with_transaction(move |conn| {
            Box::pin(async move {
                let follower_column = mapping::user_id_to_column(follower)?;
                let followee_column = mapping::user_id_to_column(followee)?;

                // Both endpoints' existence is decided in the SAME transaction as the write, so
                // NotFound and the edge insert are one atomic decision (matching the in-memory order:
                // follower, then followee).
                ensure_user_exists(&mut *conn, follower_column, follower, "follower").await?;
                ensure_user_exists(&mut *conn, followee_column, followee, "followee").await?;

                // Idempotent insert: the (follower_id, followee_id) PK makes a repeat follow a no-op.
                let inserted = sqlx::query(
                    "INSERT INTO follows (follower_id, followee_id, created_at) \
                     VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
                )
                .bind(follower_column)
                .bind(followee_column)
                .bind(created_at)
                .execute(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!(
                        "failed to durably insert a follow edge {follower} -> {followee}: {error}"
                    ))
                })?
                .rows_affected();

                // Counts move only on a genuine state change, in the same transaction as the edge.
                if inserted == 1 {
                    bump_follow_counts(&mut *conn, follower_column, followee_column, 1).await?;
                }

                let follower_count =
                    read_follower_count(&mut *conn, followee_column, followee).await?;
                Ok(FollowResponse {
                    followee_id: followee,
                    following: true,
                    follower_count,
                })
            })
        })
        .await
    }

    /// Durably makes `follower` stop following `followee`, maintaining the denormalized follow counts
    /// in the SAME confirmed-write transaction as the edge delete (`counts_match_edges`,
    /// `multiwrite_atomic`, `confirmed_write_durable`).
    ///
    /// Idempotent: unfollowing a user who is not followed deletes no row (`rows_affected == 0`), leaves
    /// the counts unchanged, and succeeds — mirroring the in-memory reference. The counts decrement
    /// only when an edge actually existed, so they can never fall below the edge set (the schema's
    /// `CHECK (... >= 0)` is the backstop). Both users' existence is decided inside the transaction.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if either user does not exist;
    /// [`Internal`](ServerError::Internal) if the transaction cannot be durably committed.
    async fn unfollow(
        &self,
        follower: UserId,
        followee: UserId,
    ) -> Result<FollowResponse, ServerError> {
        self.with_transaction(move |conn| {
            Box::pin(async move {
                let follower_column = mapping::user_id_to_column(follower)?;
                let followee_column = mapping::user_id_to_column(followee)?;

                ensure_user_exists(&mut *conn, follower_column, follower, "follower").await?;
                ensure_user_exists(&mut *conn, followee_column, followee, "followee").await?;

                let removed =
                    sqlx::query("DELETE FROM follows WHERE follower_id = $1 AND followee_id = $2")
                        .bind(follower_column)
                        .bind(followee_column)
                        .execute(&mut *conn)
                        .await
                        .map_err(|error| {
                            ServerError::Internal(format!(
                        "failed to durably delete a follow edge {follower} -> {followee}: {error}"
                    ))
                        })?
                        .rows_affected();

                if removed == 1 {
                    bump_follow_counts(&mut *conn, follower_column, followee_column, -1).await?;
                }

                let follower_count =
                    read_follower_count(&mut *conn, followee_column, followee).await?;
                Ok(FollowResponse {
                    followee_id: followee,
                    following: false,
                    follower_count,
                })
            })
        })
        .await
    }

    /// Durably records that `user` likes `chirp`, maintaining the chirp's denormalized `like_count` in
    /// the SAME confirmed-write transaction as the edge insert (`counts_match_edges`,
    /// `multiwrite_atomic`, `confirmed_write_durable`).
    ///
    /// Idempotent via the `(user_id, chirp_id)` primary key (≤ 1 like per pair): a repeated like is an
    /// `ON CONFLICT DO NOTHING` no-op and does not double-count. `like` requires an existing chirp (and
    /// user), decided inside the transaction so a missing entity is a clean
    /// [`NotFound`](ServerError::NotFound) rather than a foreign-key error — mirroring the in-memory
    /// reference (user checked first, then chirp).
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if the user or the chirp does not exist;
    /// [`Internal`](ServerError::Internal) if the transaction cannot be durably committed.
    async fn like(&self, user: UserId, chirp: ChirpId) -> Result<LikeResponse, ServerError> {
        let created_at = mapping::timestamp_to_column(now_millis());
        self.with_transaction(move |conn| {
            Box::pin(async move {
                let user_column = mapping::user_id_to_column(user)?;
                let chirp_column = mapping::chirp_id_to_column(chirp)?;

                ensure_user_exists(&mut *conn, user_column, user, "user").await?;
                ensure_chirp_exists(&mut *conn, chirp_column, chirp).await?;

                let inserted = sqlx::query(
                    "INSERT INTO likes (user_id, chirp_id, created_at) \
                     VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
                )
                .bind(user_column)
                .bind(chirp_column)
                .bind(created_at)
                .execute(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!(
                        "failed to durably insert a like edge {user} -> {chirp}: {error}"
                    ))
                })?
                .rows_affected();

                if inserted == 1 {
                    bump_like_count(&mut *conn, chirp_column, 1).await?;
                }

                let like_count = read_like_count(&mut *conn, chirp_column, chirp).await?;
                Ok(LikeResponse {
                    chirp_id: chirp,
                    liked: true,
                    like_count,
                })
            })
        })
        .await
    }

    /// Durably removes `user`'s like of `chirp`, maintaining the chirp's denormalized `like_count` in
    /// the SAME confirmed-write transaction as the edge delete (`counts_match_edges`,
    /// `multiwrite_atomic`, `confirmed_write_durable`).
    ///
    /// Idempotent: unliking a chirp the user has not liked deletes no row, leaves the count unchanged,
    /// and succeeds. The count decrements only when an edge actually existed. `unlike` requires an
    /// existing chirp (and user), decided inside the transaction — mirroring the in-memory reference.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if the user or the chirp does not exist;
    /// [`Internal`](ServerError::Internal) if the transaction cannot be durably committed.
    async fn unlike(&self, user: UserId, chirp: ChirpId) -> Result<LikeResponse, ServerError> {
        self.with_transaction(move |conn| {
            Box::pin(async move {
                let user_column = mapping::user_id_to_column(user)?;
                let chirp_column = mapping::chirp_id_to_column(chirp)?;

                ensure_user_exists(&mut *conn, user_column, user, "user").await?;
                ensure_chirp_exists(&mut *conn, chirp_column, chirp).await?;

                let removed = sqlx::query("DELETE FROM likes WHERE user_id = $1 AND chirp_id = $2")
                    .bind(user_column)
                    .bind(chirp_column)
                    .execute(&mut *conn)
                    .await
                    .map_err(|error| {
                        ServerError::Internal(format!(
                            "failed to durably delete a like edge {user} -> {chirp}: {error}"
                        ))
                    })?
                    .rows_affected();

                if removed == 1 {
                    bump_like_count(&mut *conn, chirp_column, -1).await?;
                }

                let like_count = read_like_count(&mut *conn, chirp_column, chirp).await?;
                Ok(LikeResponse {
                    chirp_id: chirp,
                    liked: false,
                    like_count,
                })
            })
        })
        .await
    }

    // ---- Release-bar additions (scaffold stubs, #66 / REQ-DOM-018) ------------------------------

    /// Durably sets (or replaces) `user`'s credential inside a confirmed-write transaction,
    /// returning only after the write has committed (`confirmed_write_durable`).
    ///
    /// The user's existence is checked inside the same transaction as the write, so the
    /// [`NotFound`](ServerError::NotFound) decision and the credential write are one atomic unit (the
    /// `credentials.user_id` foreign key is the in-store backstop). A present credential is replaced
    /// via `ON CONFLICT (user_id)` — the primary key makes "one credential per user" a store
    /// invariant. The `secret` is persisted verbatim as the at-rest representation: its format and
    /// the verification protocol are owned downstream (identity & trust, `REQ-ID-*`); this method
    /// fixes only the durable slot and never logs the secret.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if `user` does not exist;
    /// [`Internal`](ServerError::Internal) if the existence check or the write cannot be durably
    /// committed — an undurable write is reported as failed, never falsely confirmed.
    async fn set_credential(&self, user: UserId, secret: &str) -> Result<(), ServerError> {
        let secret = secret.to_owned();
        let updated_at = mapping::timestamp_to_column(now_millis());
        self.with_transaction(move |conn| {
            Box::pin(async move {
                let user_column = mapping::user_id_to_column(user)?;

                // Verify the account exists in the same transaction as the write, so NotFound and
                // the credential upsert are one atomic decision. `EXISTS` yields a single BOOLEAN
                // row, so the check neither depends on the `users` row shape nor decodes a scalar
                // whose SQL type must be pinned.
                let exists: bool =
                    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
                        .bind(user_column)
                        .fetch_one(&mut *conn)
                        .await
                        .map_err(|error| {
                            ServerError::Internal(format!(
                                "failed to check user {user} before setting a credential: {error}"
                            ))
                        })?;
                if !exists {
                    return Err(ServerError::NotFound(format!("user {user}")));
                }

                // Set-or-replace: the PRIMARY KEY on user_id makes this at most one row per user.
                sqlx::query(
                    "INSERT INTO credentials (user_id, secret, updated_at) VALUES ($1, $2, $3) \
                     ON CONFLICT (user_id) DO UPDATE \
                     SET secret = EXCLUDED.secret, updated_at = EXCLUDED.updated_at",
                )
                .bind(user_column)
                .bind(&secret)
                .bind(updated_at)
                .execute(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!(
                        "failed to durably record the credential for user {user}: {error}"
                    ))
                })?;

                Ok(())
            })
        })
        .await
    }

    /// Verifies a presented `secret` for the account named by `username` against the durable store,
    /// returning the authenticated [`UserId`] on a match.
    ///
    /// A single pooled read joins the handle to its credential; no transaction is needed for a
    /// read-only lookup. The handle is bound as a parameter (never concatenated), so the lookup
    /// carries no injection surface. Verification fails [`Unauthorized`](ServerError::Unauthorized)
    /// **indistinguishably** whether the handle is unknown, has no stored credential, or the secret
    /// does not match — the store never discloses which held, so an attacker cannot enumerate
    /// accounts. The secret comparison is a direct equality against the stored at-rest
    /// representation; the credential/verification protocol (hashing, timing properties) is owned
    /// downstream (`REQ-ID-*`), not decided here.
    ///
    /// # Errors
    ///
    /// [`Unauthorized`](ServerError::Unauthorized) if the handle is unknown, has no credential, or
    /// the secret does not match; [`Internal`](ServerError::Internal) if the read fails or a stored
    /// id column is corrupt.
    async fn verify_credential(
        &self,
        username: &Username,
        secret: &str,
    ) -> Result<UserId, ServerError> {
        let row: Option<(i64, String)> = sqlx::query_as(
            "SELECT c.user_id, c.secret FROM credentials c \
             JOIN users u ON u.id = c.user_id WHERE u.username = $1",
        )
        .bind(username.as_str())
        .fetch_optional(self.pool())
        .await
        .map_err(|error| {
            ServerError::Internal(format!(
                "failed to read a credential for verification: {error}"
            ))
        })?;

        match row {
            // A stored credential exists and the presented secret matches it: lift the id back
            // through the boundary mapping (a corrupt id column surfaces as Internal).
            Some((user_id, stored)) if stored == secret => mapping::column_to_user_id(user_id),
            // Unknown handle, no credential, or a mismatch — all reported identically.
            _ => Err(ServerError::Unauthorized(
                "invalid username or secret".to_owned(),
            )),
        }
    }

    /// Durably invalidates a session (deliberate sign-out) by deleting its row inside a confirmed-write
    /// transaction, so the token no longer [`resolve_token`](ChirpRepository::resolve_token)s.
    ///
    /// Idempotent: invalidating a token that is absent (never issued, or already invalidated) succeeds
    /// — a `DELETE` of zero rows is a no-op that still commits, so repeated sign-out is safe. The
    /// delete is confirmed durable before success is returned (`confirmed_write_durable`); an undurable
    /// write is reported as failed, never falsely confirmed.
    ///
    /// # Errors
    ///
    /// [`Internal`](ServerError::Internal) if the durable store cannot record the invalidation.
    async fn invalidate_session(&self, token: &SessionToken) -> Result<(), ServerError> {
        let token = token.0.clone();
        self.with_transaction(move |conn| {
            Box::pin(async move {
                sqlx::query("DELETE FROM sessions WHERE token = $1")
                    .bind(&token)
                    .execute(&mut *conn)
                    .await
                    .map_err(|error| {
                        ServerError::Internal(format!(
                            "failed to durably invalidate a session: {error}"
                        ))
                    })?;
                Ok(())
            })
        })
        .await
    }

    /// Durably issues an account-recovery challenge for the account named by `username`: resolves the
    /// handle to its user and inserts an opaque recovery token into `recovery_tokens` inside a
    /// confirmed-write transaction, returning only after the write has durably committed
    /// (`confirmed_write_durable`).
    ///
    /// The handle lookup and the token insert run in the **same** transaction, so the
    /// [`NotFound`](ServerError::NotFound) decision and the write are one atomic unit, and the stored
    /// id is lifted back through the committed mapping so a corrupt id column surfaces as an error
    /// rather than a bogus id (`domain_boundary_validated`). The token value is drawn from the durable
    /// `token_seq` via [`allocate_token_value`](Self::allocate_token_value) on the transaction's own
    /// connection — opaque, monotonic, stable across restart, never reused
    /// (`ids_opaque_monotonic_stable`) — and the challenge is stamped with a `created_at`/`expires_at`
    /// window (the committed schema requires a non-null expiry; the authoritative validity value is
    /// owned downstream — see [`RECOVERY_TOKEN_VALIDITY_MILLIS`]). The issued token is delivered to the
    /// account holder out of band (owned downstream, `REQ-ID-*`); this method only persists the
    /// challenge durably, so it returns `()` rather than the token.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if no account has that handle;
    /// [`Internal`](ServerError::Internal) if the token cannot be allocated or the challenge cannot be
    /// durably committed — an undurable write is reported as failed, never falsely confirmed.
    async fn begin_account_recovery(&self, username: &Username) -> Result<(), ServerError> {
        let username = username.clone();
        let created_at = mapping::timestamp_to_column(now_millis());
        let expires_at = created_at.saturating_add(RECOVERY_TOKEN_VALIDITY_MILLIS);
        self.with_transaction(move |conn| {
            Box::pin(async move {
                // Resolve the handle to its user id INSIDE the transaction, so NotFound and the token
                // insert are one atomic decision. The stored id is lifted through the committed
                // mapping (a corrupt id column surfaces as Internal, never a bogus id).
                let row: Option<(i64,)> =
                    sqlx::query_as("SELECT id FROM users WHERE username = $1")
                        .bind(username.as_str())
                        .fetch_optional(&mut *conn)
                        .await
                        .map_err(|error| {
                            ServerError::Internal(format!(
                                "failed to read user '{username}' for recovery: {error}"
                            ))
                        })?;
                let user_id = match row {
                    Some((user_id,)) => mapping::column_to_user_id(user_id)?,
                    None => return Err(ServerError::NotFound(format!("user '{username}'"))),
                };

                // Mint an opaque challenge token from the durable `token_seq` on the SAME connection,
                // so the allocation and the insert are one atomic unit and the value is never reused.
                let token_value = PostgresRepository::allocate_token_value(&mut *conn).await?;
                let token = format!("rec-{token_value}");

                sqlx::query(
                    "INSERT INTO recovery_tokens (token, user_id, created_at, expires_at) \
                     VALUES ($1, $2, $3, $4)",
                )
                .bind(&token)
                .bind(mapping::user_id_to_column(user_id)?)
                .bind(created_at)
                .bind(expires_at)
                .execute(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!(
                        "failed to durably record a recovery challenge for user {user_id}: {error}"
                    ))
                })?;

                Ok(())
            })
        })
        .await
    }

    /// Durably completes account recovery: validates `recovery_token` for `username` against the
    /// committed `recovery_tokens` table and, on success, sets `new_secret` as the account's
    /// credential — all inside one confirmed-write transaction, returning only after the credential
    /// write has durably committed (`confirmed_write_durable`).
    ///
    /// Validation is fail-secure and reports [`Unauthorized`](ServerError::Unauthorized)
    /// **indistinguishably** whether the handle is unknown, the token is unknown, the token belongs to
    /// a different account, or the token has expired (see [`unauthorized_recovery`]) — the store never
    /// discloses which held, so an attacker can neither enumerate accounts nor probe token state. A
    /// validated challenge is **consumed** (deleted) in the same transaction, so a recovery token is
    /// single-use and cannot be replayed. The new credential is written through the **same durable
    /// upsert** as [`set_credential`](ChirpRepository::set_credential) (`INSERT … ON CONFLICT
    /// (user_id) DO UPDATE`), so a credential set via recovery is indistinguishable from one set
    /// directly and [`verify_credential`](ChirpRepository::verify_credential) authenticates it. The
    /// secret is persisted verbatim as the at-rest representation; its format and the credential
    /// protocol are owned downstream (`REQ-ID-*`) and the secret is never logged.
    ///
    /// # Errors
    ///
    /// [`Unauthorized`](ServerError::Unauthorized) if the recovery token is unknown, expired, or does
    /// not match the account; [`Internal`](ServerError::Internal) if the credential reset cannot be
    /// durably committed — an undurable write is reported as failed, never falsely confirmed.
    async fn complete_account_recovery(
        &self,
        username: &Username,
        recovery_token: &str,
        new_secret: &str,
    ) -> Result<(), ServerError> {
        let username = username.clone();
        let recovery_token = recovery_token.to_owned();
        let new_secret = new_secret.to_owned();
        let now = mapping::timestamp_to_column(now_millis());
        self.with_transaction(move |conn| {
            Box::pin(async move {
                // Resolve the handle inside the transaction. An unknown handle is reported as the same
                // Unauthorized as an unknown/mismatched token (fail-secure, no account enumeration).
                let user_row: Option<(i64,)> =
                    sqlx::query_as("SELECT id FROM users WHERE username = $1")
                        .bind(username.as_str())
                        .fetch_optional(&mut *conn)
                        .await
                        .map_err(|error| {
                            ServerError::Internal(format!(
                                "failed to read user '{username}' for recovery: {error}"
                            ))
                        })?;
                let Some((user_id_col,)) = user_row else {
                    return Err(unauthorized_recovery());
                };
                // Lift the stored id through the committed mapping (a corrupt id surfaces as Internal).
                let user_id = mapping::column_to_user_id(user_id_col)?;

                // Validate AND consume the challenge atomically: delete the row only if it matches the
                // presented token, belongs to this account, and has not expired. A zero-row delete
                // (unknown / cross-account / expired) is Unauthorized; a single deleted row proves a
                // valid, unexpired, single-use challenge that is now spent.
                let consumed = sqlx::query(
                    "DELETE FROM recovery_tokens \
                     WHERE token = $1 AND user_id = $2 AND expires_at > $3",
                )
                .bind(&recovery_token)
                .bind(user_id_col)
                .bind(now)
                .execute(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!(
                        "failed to validate a recovery challenge: {error}"
                    ))
                })?;
                if consumed.rows_affected() == 0 {
                    return Err(unauthorized_recovery());
                }

                // Reset the credential through the SAME durable upsert as `set_credential`, so a
                // credential set via recovery is identical to one set directly (one row per user).
                sqlx::query(
                    "INSERT INTO credentials (user_id, secret, updated_at) VALUES ($1, $2, $3) \
                     ON CONFLICT (user_id) DO UPDATE \
                     SET secret = EXCLUDED.secret, updated_at = EXCLUDED.updated_at",
                )
                .bind(user_id_col)
                .bind(&new_secret)
                .bind(now)
                .execute(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!(
                        "failed to durably reset the credential for user {user_id} via recovery: \
                         {error}"
                    ))
                })?;

                Ok(())
            })
        })
        .await
    }

    /// Durably updates the mutable profile fields (`display_name`, `bio`) of `user` and returns the
    /// updated [`User`], confirmed durable before success is returned (`confirmed_write_durable`).
    ///
    /// The write is a single `UPDATE ... RETURNING` run inside
    /// [`with_transaction`](Self::with_transaction), so the existence decision and the mutation are
    /// one atomic, confirmed-durable unit: a matching row is updated and lifted back through the
    /// committed mapping ([`mapping::UserRow::into_user`], which re-parses the stored handle at the
    /// durable boundary — a corrupt row surfaces as an error, never a panic —
    /// `domain_boundary_validated`), while a zero-row update (no such id) is reported
    /// [`NotFound`](ServerError::NotFound). The `SET` list names only `display_name` and `bio`; the
    /// `WHERE` keys on the immutable `id` and `username` is never written, so the handle and id are
    /// immutable identity by construction (`REQ-DOM-008`). The new values are bound as parameters
    /// (never concatenated), so the update carries no injection surface, and — `display_name` /`bio`
    /// being unconstrained `chirp-types` strings — any value is admissible; the returned [`User`]
    /// reflects the durable post-update state (its immutable `created_at` and denormalized counts
    /// unchanged).
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if `user` does not exist;
    /// [`Validation`](ServerError::Validation) if the returned row violates a `chirp-types` invariant
    /// (a corrupt stored handle); [`Internal`](ServerError::Internal) if the write cannot be durably
    /// committed or a stored id/count column is corrupt — an undurable write is reported as failed,
    /// never falsely confirmed (`confirmed_write_durable`).
    async fn update_profile(
        &self,
        user: UserId,
        display_name: String,
        bio: Option<String>,
    ) -> Result<User, ServerError> {
        self.with_transaction(move |conn| {
            Box::pin(async move {
                let user_column = mapping::user_id_to_column(user)?;

                // One statement decides NotFound AND performs the mutation: a matching row is updated
                // and RETURNED (then lifted through the boundary mapping); a zero-row update means no
                // such id. `id`/`username` are never in the SET list, so identity is immutable here.
                let row: Option<mapping::UserRow> = sqlx::query_as(
                    "UPDATE users SET display_name = $1, bio = $2 WHERE id = $3 \
                     RETURNING id, username, display_name, bio, created_at, follower_count, \
                     following_count, suspended, suspended_at",
                )
                .bind(&display_name)
                .bind(&bio)
                .bind(user_column)
                .fetch_optional(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!(
                        "failed to durably update the profile for user {user}: {error}"
                    ))
                })?;

                match row {
                    Some(row) => row.into_user(),
                    None => Err(ServerError::NotFound(format!("user {user}"))),
                }
            })
        })
        .await
    }

    /// Durably deletes `user` and cascades the removal of ALL their dependent state — authored
    /// chirps, follow edges (in both directions), like edges, sessions, the durable credential,
    /// outstanding recovery tokens, and materialised export artefacts — as ONE confirmed-write
    /// transaction, reconciling the denormalized counts of every SURVIVING neighbour in that same
    /// transaction (`multiwrite_atomic`, `counts_match_edges_under_cascade`,
    /// `confirmed_write_durable`).
    ///
    /// # Cascade versus explicit reconciliation
    ///
    /// The schema's `ON DELETE CASCADE` foreign keys remove the dependent *rows* for free once the
    /// `users` row is deleted (chirps by author — and their like edges; follows on either side;
    /// likes by the user; sessions; credential; recovery tokens; export artefacts; a surviving reply
    /// to a removed chirp is `SET NULL`). What the database does NOT do is fix the denormalized
    /// *count columns* on the neighbours that SURVIVE the deletion — that reconciliation is this
    /// method's load-bearing work, and it MUST run while the edges still exist, i.e. BEFORE the
    /// `users` row (and its cascade) is deleted:
    ///
    /// * every account the deleted user follows loses one follower → decrement its `follower_count`;
    /// * every account that follows the deleted user loses one followee → decrement its
    ///   `following_count`;
    /// * every SURVIVING chirp the deleted user liked loses one like → decrement its `like_count`.
    ///   A chirp authored by the deleted user is itself cascaded away, so it is excluded
    ///   (`author_id <> user`): its count is irrelevant and touching a row about to vanish is
    ///   pointless.
    ///
    /// The `follows_no_self` invariant guarantees both endpoints of every follow edge touching the
    /// user are distinct, so each such edge has exactly one surviving neighbour to reconcile; and the
    /// edge-set primary keys guarantee each neighbour appears at most once per family, so a single
    /// set-based `UPDATE` per family moves each surviving count by exactly one and never underflows
    /// the `CHECK (... >= 0)` backstop (the prior stage's `counts_match_edges` means the neighbour's
    /// count is at least the edge being removed).
    ///
    /// The user's existence is decided inside the same transaction — a clean
    /// [`NotFound`](ServerError::NotFound) for an absent or already-deleted account, never a silent
    /// no-op — so the existence decision, the count reconciliation, and the cascade are one atomic
    /// unit: a reader never observes a surviving count left stale behind a deleted edge, and a
    /// mid-operation failure rolls the whole cascade back rather than leaving the account
    /// half-deleted.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if `user` does not exist;
    /// [`Internal`](ServerError::Internal) if the transaction cannot be durably committed — an
    /// undurable write is reported failed, never falsely confirmed.
    async fn delete_account(&self, user: UserId) -> Result<(), ServerError> {
        self.with_transaction(move |conn| {
            Box::pin(async move {
                let user_column = mapping::user_id_to_column(user)?;

                // Existence is decided inside the transaction: an absent (or already-deleted)
                // account is a clean NotFound, never a silent no-op — matching the in-memory
                // reference and the trait's documented error contract.
                ensure_user_exists(&mut *conn, user_column, user, "user").await?;

                // Reconcile the SURVIVING neighbours' denormalized counts BEFORE the cascade removes
                // the edges (afterwards the edge rows are gone and the information is lost). Each
                // UPDATE is set-based and keyed on the still-present edge rows, so every surviving
                // neighbour moves by exactly one.

                // Accounts the user follows each lose one follower.
                sqlx::query(
                    "UPDATE users SET follower_count = follower_count - 1 \
                     WHERE id IN (SELECT followee_id FROM follows WHERE follower_id = $1)",
                )
                .bind(user_column)
                .execute(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!(
                        "failed to reconcile follower_count on the accounts {user} follows: {error}"
                    ))
                })?;

                // Accounts that follow the user each lose one followee.
                sqlx::query(
                    "UPDATE users SET following_count = following_count - 1 \
                     WHERE id IN (SELECT follower_id FROM follows WHERE followee_id = $1)",
                )
                .bind(user_column)
                .execute(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!(
                        "failed to reconcile following_count on the followers of {user}: {error}"
                    ))
                })?;

                // Surviving chirps the user liked each lose one like. A chirp authored by the user is
                // cascaded away below, so it is excluded (`author_id <> user`).
                sqlx::query(
                    "UPDATE chirps SET like_count = like_count - 1 \
                     WHERE id IN (SELECT chirp_id FROM likes WHERE user_id = $1) \
                       AND author_id <> $1",
                )
                .bind(user_column)
                .execute(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!(
                        "failed to reconcile like_count on the surviving chirps {user} liked: {error}"
                    ))
                })?;

                // Remove the account. The schema's ON DELETE CASCADE foreign keys remove every
                // dependent family atomically with this delete: the user's chirps (and, cascading
                // off them, their like edges), follow edges on both sides, like edges by the user,
                // sessions, the credential, recovery tokens, and export artefacts; a surviving reply
                // to one of the removed chirps has its reply_to SET NULL.
                sqlx::query("DELETE FROM users WHERE id = $1")
                    .bind(user_column)
                    .execute(&mut *conn)
                    .await
                    .map_err(|error| {
                        ServerError::Internal(format!(
                            "failed to durably delete account {user} and cascade its state: {error}"
                        ))
                    })?;

                Ok(())
            })
        })
        .await
    }

    /// Materializes a user's full data export as a consistent durable view: the user's profile, their
    /// authored **non-removed** chirps (newest-first), their follow graph (the ids they follow and
    /// the ids that follow them), and the ids of the chirps they like — serialized through the shared
    /// [`ExportDocument`](export::ExportDocument) so the durable and in-memory backings produce
    /// byte-identical exports.
    ///
    /// Every read runs inside one transaction, so the export is a single consistent snapshot rather
    /// than a set of statements that could straddle a concurrent write. The profile read is also the
    /// [`NotFound`](ServerError::NotFound) decision; each stored row is lifted back through the
    /// committed mapping, so a corrupt row surfaces as an error, never a bad value
    /// (`domain_boundary_validated`). The export is **not** persisted to `export_artifacts` here —
    /// materialized-artifact retention / lifetime is the retention stage's concern; this method
    /// produces and returns the document.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if `user` does not exist;
    /// [`Validation`](ServerError::Validation) if a stored row violates a `chirp-types` invariant;
    /// [`Internal`](ServerError::Internal) if a read fails, a stored column is corrupt, or the
    /// document cannot be serialized.
    async fn export_user_data(&self, user: UserId) -> Result<String, ServerError> {
        let user_column = mapping::user_id_to_column(user)?;
        self.with_transaction(move |conn| {
            Box::pin(async move {
                // The profile read is also the existence decision.
                let profile_row: Option<mapping::UserRow> = sqlx::query_as(
                    "SELECT id, username, display_name, bio, created_at, follower_count, \
                     following_count, suspended, suspended_at FROM users WHERE id = $1",
                )
                .bind(user_column)
                .fetch_optional(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!("failed to read user {user} for export: {error}"))
                })?;
                let profile = match profile_row {
                    Some(row) => row.into_user()?,
                    None => return Err(ServerError::NotFound(format!("user {user}"))),
                };

                let chirp_rows: Vec<mapping::ChirpRow> = sqlx::query_as(
                    "SELECT id, author_id, text, created_at, like_count, reply_to, removed, \
                     removed_at FROM chirps WHERE author_id = $1 AND removed = FALSE ORDER BY id DESC",
                )
                .bind(user_column)
                .fetch_all(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!("failed to read chirps for export: {error}"))
                })?;
                let chirps = chirp_rows
                    .into_iter()
                    .map(mapping::ChirpRow::into_chirp)
                    .collect::<Result<Vec<Chirp>, ServerError>>()?;

                let following = fetch_user_ids(
                    &mut *conn,
                    "SELECT followee_id FROM follows WHERE follower_id = $1",
                    user_column,
                )
                .await?;
                let followers = fetch_user_ids(
                    &mut *conn,
                    "SELECT follower_id FROM follows WHERE followee_id = $1",
                    user_column,
                )
                .await?;

                let liked_columns: Vec<i64> =
                    sqlx::query_scalar("SELECT chirp_id FROM likes WHERE user_id = $1")
                        .bind(user_column)
                        .fetch_all(&mut *conn)
                        .await
                        .map_err(|error| {
                            ServerError::Internal(format!("failed to read likes for export: {error}"))
                        })?;
                let likes = liked_columns
                    .into_iter()
                    .map(mapping::column_to_chirp_id)
                    .collect::<Result<Vec<ChirpId>, ServerError>>()?;

                export::ExportDocument::new(profile, chirps, following, followers, likes).to_json()
            })
        })
        .await
    }

    /// Returns one page of the accounts following `user`, newest-first, with the same opaque-cursor
    /// paging as [`timeline`](ChirpRepository::timeline).
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if `user` does not exist;
    /// [`Validation`](ServerError::Validation) if a stored row violates a `chirp-types` invariant;
    /// [`Internal`](ServerError::Internal) if the read fails or a stored column is corrupt.
    async fn list_followers(
        &self,
        user: UserId,
        query: TimelineQuery,
    ) -> Result<Page<User>, ServerError> {
        self.list_follow_edge(user, query, FollowListing::Followers)
            .await
    }

    /// Returns one page of the accounts `user` follows, newest-first, with the same opaque-cursor
    /// paging as [`timeline`](ChirpRepository::timeline).
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if `user` does not exist;
    /// [`Validation`](ServerError::Validation) if a stored row violates a `chirp-types` invariant;
    /// [`Internal`](ServerError::Internal) if the read fails or a stored column is corrupt.
    async fn list_following(
        &self,
        user: UserId,
        query: TimelineQuery,
    ) -> Result<Page<User>, ServerError> {
        self.list_follow_edge(user, query, FollowListing::Following)
            .await
    }

    /// Durably soft-removes `chirp` so it is no longer served, inside a confirmed-write transaction,
    /// returning only after the write has committed (`confirmed_write_durable`).
    ///
    /// A single `UPDATE ... RETURNING` both decides [`NotFound`](ServerError::NotFound) (a zero-row
    /// update means no such chirp) and performs the soft-removal. The row is retained (so a reply
    /// target or a like edge still resolves — distinct from the account-deletion cascade, which
    /// hard-removes rows); its `removed = TRUE` flag excludes it from timelines and exports. The
    /// first removal stamps `removed_at`; `COALESCE` preserves it, so repeated removal is an
    /// idempotent no-op that keeps the original audit time.
    ///
    /// The author-vs-moderator **authorization** boundary (`REQ-DOM-020`) is fixed in the model and
    /// enforced downstream of the repository (`REQ-ID-*`); this persistence layer accepts `actor` and
    /// performs the durable removal, exactly as the sibling
    /// [`set_account_suspension`](ChirpRepository::set_account_suspension) defers the moderator-authority
    /// determination for a distinct actor upstream.
    ///
    /// # Errors
    ///
    /// [`NotFound`](ServerError::NotFound) if the chirp does not exist;
    /// [`Internal`](ServerError::Internal) if the write cannot be durably committed — an undurable
    /// write is reported as failed, never falsely confirmed.
    async fn remove_chirp(&self, _actor: UserId, chirp: ChirpId) -> Result<(), ServerError> {
        let chirp_column = mapping::chirp_id_to_column(chirp)?;
        let removed_at = mapping::timestamp_to_column(now_millis());
        self.with_transaction(move |conn| {
            Box::pin(async move {
                let updated: Option<i64> = sqlx::query_scalar(
                    "UPDATE chirps SET removed = TRUE, removed_at = COALESCE(removed_at, $1) \
                     WHERE id = $2 RETURNING id",
                )
                .bind(removed_at)
                .bind(chirp_column)
                .fetch_optional(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!(
                        "failed to durably remove chirp {chirp}: {error}"
                    ))
                })?;
                match updated {
                    Some(_) => Ok(()),
                    None => Err(ServerError::NotFound(format!("chirp {chirp}"))),
                }
            })
        })
        .await
    }

    /// Durably sets (`suspended = true`) or clears (`suspended = false`) the moderation suspension
    /// state of `target`, on behalf of moderator `actor`.
    ///
    /// Realizes the durable `account_suspension` capability (`REQ-DOM-011`). The `suspended` flag and
    /// its `suspended_at` audit timestamp are the committed `users` columns; a single
    /// `UPDATE ... RETURNING` both decides [`NotFound`](ServerError::NotFound) (a zero-row update
    /// means no such target) and performs the mutation inside
    /// [`with_transaction`](PostgresRepository::with_transaction), so success is returned only after
    /// the write is durably committed (`confirmed_write_durable`). Setting stamps `suspended_at` with
    /// the current time; clearing resets it to `NULL`.
    ///
    /// # Authorization boundary
    ///
    /// Moderation is reserved to an operator/moderator acting on **another** account (`REQ-DOM-020`);
    /// the ordinary self-service surface, where a user acts only on their own account, cannot suspend
    /// (`REQ-ID-041`). This persistence stage enforces the one model-level result it owns from the
    /// committed data — a self-targeted moderation (`actor == target`) is
    /// [`Forbidden`](ServerError::Forbidden) — mirroring the committed no-self-follow guard. The
    /// *authority* determination (whether a distinct `actor` actually holds moderator authority) is
    /// owned by identity & trust (`REQ-ID-047`) and enforced upstream of the repository, exactly as
    /// the sibling [`remove_chirp`](ChirpRepository::remove_chirp) defers its author-vs-moderator
    /// check; this stage persists the state durably and does not model the (deliberately-absent)
    /// moderator role.
    ///
    /// # Errors
    ///
    /// [`Forbidden`](ServerError::Forbidden) if `actor == target` (a moderation action cannot target
    /// the acting account); [`NotFound`](ServerError::NotFound) if `target` does not exist;
    /// [`Internal`](ServerError::Internal) if the write cannot be durably committed — an undurable
    /// write is reported as failed, never falsely confirmed (`confirmed_write_durable`).
    async fn set_account_suspension(
        &self,
        actor: UserId,
        target: UserId,
        suspended: bool,
    ) -> Result<(), ServerError> {
        // The model-level authorization result this persistence layer owns: moderation acts on
        // ANOTHER account (REQ-DOM-020), so a self-targeted moderation is refused — mirroring the
        // committed no-self-follow guard. The moderator-authority gate for a distinct actor is
        // REQ-ID-* and enforced upstream of the repository.
        if actor == target {
            return Err(ServerError::Forbidden(
                "a moderation action cannot target the acting account".to_owned(),
            ));
        }

        // Stamp the audit timestamp when setting; clearing resets it to NULL.
        let suspended_at = suspended.then(|| mapping::timestamp_to_column(now_millis()));

        self.with_transaction(move |conn| {
            Box::pin(async move {
                let target_column = mapping::user_id_to_column(target)?;

                // One statement decides NotFound AND performs the mutation: a matching row is updated
                // and its id RETURNED; a zero-row update means no such target.
                let updated: Option<i64> = sqlx::query_scalar(
                    "UPDATE users SET suspended = $1, suspended_at = $2 WHERE id = $3 RETURNING id",
                )
                .bind(suspended)
                .bind(suspended_at)
                .bind(target_column)
                .fetch_optional(&mut *conn)
                .await
                .map_err(|error| {
                    ServerError::Internal(format!(
                        "failed to durably set the suspension state of user {target}: {error}"
                    ))
                })?;

                match updated {
                    Some(_) => Ok(()),
                    None => Err(ServerError::NotFound(format!("user {target}"))),
                }
            })
        })
        .await
    }

    /// Truthful durable-store health: round-trips the live connection pool with a trivial query.
    ///
    /// Unlike the other scaffold methods this is **not** a stub — store reachability is exactly what
    /// the `store_health` capability (REQ-DOM-021) reports, and it is meaningful for the durable
    /// backing from day one. `Ok(())` means the primary answered; an error means it did not.
    ///
    /// # Errors
    ///
    /// [`ServerError::Internal`] if the backing store is unreachable or does not answer.
    async fn store_health(&self) -> Result<(), ServerError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|error| {
                ServerError::Internal(format!("durable store health check failed: {error}"))
            })
    }
}

// =================================================================================================
// Content / timeline helpers (EPIC-PERSIST, SUBTASK_content_timeline_impl, #64/#65).
//
// The shared paging + listing machinery the content/timeline capability methods above compose
// against, so the opaque-cursor scheme lives once rather than being re-derived per method. The
// cursor decode matches the in-memory backing exactly (parse a u64; a malformed or out-of-range
// cursor fails safe to the first page), so the two backings page identically.
// =================================================================================================

/// Which side of the follow graph a listing walks.
#[derive(Debug, Clone, Copy)]
enum FollowListing {
    /// The accounts that follow the user (filter `follows.followee_id`, join `follows.follower_id`).
    Followers,
    /// The accounts the user follows (filter `follows.follower_id`, join `follows.followee_id`).
    Following,
}

/// Clamps a requested page limit to `1..=`[`MAX_TIMELINE_LIMIT`], defaulting to
/// [`DEFAULT_TIMELINE_LIMIT`] when absent — identical to the in-memory backing.
fn clamp_limit(requested: Option<u32>) -> u32 {
    requested
        .unwrap_or(DEFAULT_TIMELINE_LIMIT)
        .clamp(1, MAX_TIMELINE_LIMIT)
}

/// The `LIMIT` to fetch for a page of size `limit`: one extra row, so the caller learns whether a
/// further page exists without a second query.
fn fetch_limit(limit: u32) -> i64 {
    i64::from(limit) + 1
}

/// Decodes an opaque cursor to the exclusive upper-bound id for the next page.
///
/// Cursors are server-minted stringified ids; a value we did not mint — unparseable, or larger than
/// the signed id space — decodes to `None`, which fails safe to the first page (no filter), exactly
/// as the in-memory backing treats an unrecognised cursor.
fn decode_cursor(cursor: Option<&Cursor>) -> Option<i64> {
    cursor
        .and_then(|c| c.0.parse::<u64>().ok())
        .and_then(|value| i64::try_from(value).ok())
}

/// Turns the `limit + 1` rows a page query fetched (already ordered newest-first) into a [`Page`]:
/// trims to `limit`, and sets [`next_cursor`](Page::next_cursor) to the last returned id iff a
/// further row was fetched — the exactly-once opaque-cursor contract shared by the timeline and the
/// follow listings.
fn paginate<T>(mut items: Vec<T>, limit: u32, id_of: impl Fn(&T) -> u64) -> Page<T> {
    let limit = limit as usize;
    let has_more = items.len() > limit;
    items.truncate(limit);
    let next_cursor = if has_more {
        items.last().map(|item| Cursor(id_of(item).to_string()))
    } else {
        None
    };
    Page::new(items, next_cursor)
}

/// Runs a single-column `BIGINT` id query bound to `user_column` and lifts each value back to a
/// [`UserId`] through the committed mapping (a corrupt id column surfaces as an error). Shared by the
/// export's `following` / `followers` reads.
async fn fetch_user_ids(
    conn: &mut sqlx::PgConnection,
    query: &'static str,
    user_column: i64,
) -> Result<Vec<UserId>, ServerError> {
    let columns: Vec<i64> = sqlx::query_scalar(query)
        .bind(user_column)
        .fetch_all(conn)
        .await
        .map_err(|error| {
            ServerError::Internal(format!("failed to read follow edges for export: {error}"))
        })?;
    columns
        .into_iter()
        .map(mapping::column_to_user_id)
        .collect()
}

// ---- Social-graph edge-toggle helpers (follow/unfollow/like/unlike) -----------------------------
// Each runs on the caller's transaction connection, so the existence check, the edge mutation, and
// the denormalized-count maintenance commit as one atomic unit (`multiwrite_atomic`).

/// Decides a user's existence INSIDE the caller's transaction, returning a clean
/// [`NotFound`](ServerError::NotFound) labelled by `role` (e.g. `"follower"`, `"followee"`, `"user"`)
/// rather than letting a later foreign-key violation surface as an opaque error — mirroring the
/// in-memory implementor's existence checks.
async fn ensure_user_exists(
    conn: &mut sqlx::PgConnection,
    user_column: i64,
    id: UserId,
    role: &str,
) -> Result<(), ServerError> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
        .bind(user_column)
        .fetch_one(conn)
        .await
        .map_err(|error| ServerError::Internal(format!("failed to check {role} {id}: {error}")))?;
    if exists {
        Ok(())
    } else {
        Err(ServerError::NotFound(format!("{role} {id}")))
    }
}

/// Decides a chirp's existence INSIDE the caller's transaction (a clean
/// [`NotFound`](ServerError::NotFound), not an opaque foreign-key error) — so `like`/`unlike` require
/// an existing chirp exactly as the in-memory implementor does.
async fn ensure_chirp_exists(
    conn: &mut sqlx::PgConnection,
    chirp_column: i64,
    id: ChirpId,
) -> Result<(), ServerError> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM chirps WHERE id = $1)")
        .bind(chirp_column)
        .fetch_one(conn)
        .await
        .map_err(|error| ServerError::Internal(format!("failed to check chirp {id}: {error}")))?;
    if exists {
        Ok(())
    } else {
        Err(ServerError::NotFound(format!("chirp {id}")))
    }
}

/// Adjusts BOTH denormalized follow counts by `delta` (`+1` on follow, `-1` on unfollow) in the
/// caller's transaction: the follower's `following_count` and the followee's `follower_count`. Called
/// ONLY when the edge genuinely changed, so the counts track set membership and never fall below it
/// (the schema's `CHECK (... >= 0)` is the in-store backstop).
async fn bump_follow_counts(
    conn: &mut sqlx::PgConnection,
    follower_column: i64,
    followee_column: i64,
    delta: i64,
) -> Result<(), ServerError> {
    sqlx::query("UPDATE users SET following_count = following_count + $1 WHERE id = $2")
        .bind(delta)
        .bind(follower_column)
        .execute(&mut *conn)
        .await
        .map_err(|error| {
            ServerError::Internal(format!("failed to adjust following_count: {error}"))
        })?;
    sqlx::query("UPDATE users SET follower_count = follower_count + $1 WHERE id = $2")
        .bind(delta)
        .bind(followee_column)
        .execute(conn)
        .await
        .map_err(|error| {
            ServerError::Internal(format!("failed to adjust follower_count: {error}"))
        })?;
    Ok(())
}

/// Adjusts a chirp's denormalized `like_count` by `delta` in the caller's transaction (only on a
/// genuine edge change, so it tracks the `likes` set and never underflows the `CHECK (... >= 0)`).
async fn bump_like_count(
    conn: &mut sqlx::PgConnection,
    chirp_column: i64,
    delta: i64,
) -> Result<(), ServerError> {
    sqlx::query("UPDATE chirps SET like_count = like_count + $1 WHERE id = $2")
        .bind(delta)
        .bind(chirp_column)
        .execute(conn)
        .await
        .map_err(|error| ServerError::Internal(format!("failed to adjust like_count: {error}")))?;
    Ok(())
}

/// Reads a user's current `follower_count` inside the caller's transaction and lifts it to the
/// `chirp-types` `u64` through the committed mapping (a negative stored value is corrupt and surfaces
/// as an error, never a panic — `domain_boundary_validated`).
async fn read_follower_count(
    conn: &mut sqlx::PgConnection,
    user_column: i64,
    id: UserId,
) -> Result<u64, ServerError> {
    let count: i64 = sqlx::query_scalar("SELECT follower_count FROM users WHERE id = $1")
        .bind(user_column)
        .fetch_one(conn)
        .await
        .map_err(|error| {
            ServerError::Internal(format!("failed to read follower_count for {id}: {error}"))
        })?;
    mapping::column_to_u64(count, "follower_count")
}

/// Reads a chirp's current `like_count` inside the caller's transaction and lifts it to `u64` through
/// the committed mapping (a negative stored value surfaces as an error, never a panic).
async fn read_like_count(
    conn: &mut sqlx::PgConnection,
    chirp_column: i64,
    id: ChirpId,
) -> Result<u64, ServerError> {
    let count: i64 = sqlx::query_scalar("SELECT like_count FROM chirps WHERE id = $1")
        .bind(chirp_column)
        .fetch_one(conn)
        .await
        .map_err(|error| {
            ServerError::Internal(format!("failed to read like_count for {id}: {error}"))
        })?;
    mapping::column_to_u64(count, "like_count")
}

impl PostgresRepository {
    /// The shared body of [`list_followers`](ChirpRepository::list_followers) and
    /// [`list_following`](ChirpRepository::list_following): verifies the user exists and returns one
    /// page of the accounts on the requested side of its follow graph, newest-account-first
    /// (`id DESC` — the same stable id proxy the timeline and the in-memory backing use, since the
    /// retained follow edges carry no per-edge timestamp), with the shared opaque-cursor paging.
    ///
    /// The existence check and the page read run in one read transaction (a consistent snapshot), so
    /// an absent user is [`NotFound`](ServerError::NotFound) rather than an empty page. Each stored
    /// row is lifted back through the committed mapping (`domain_boundary_validated`).
    async fn list_follow_edge(
        &self,
        user: UserId,
        query: TimelineQuery,
        listing: FollowListing,
    ) -> Result<Page<User>, ServerError> {
        let limit = clamp_limit(query.limit);
        let cursor = decode_cursor(query.cursor.as_ref());
        let user_column = mapping::user_id_to_column(user)?;
        // Fixed, crate-controlled SQL per listing side; `$1` is the only user input and is bound.
        let sql: &'static str = match listing {
            FollowListing::Followers => {
                "SELECT u.id, u.username, u.display_name, u.bio, u.created_at, u.follower_count, \
                 u.following_count, u.suspended, u.suspended_at \
                 FROM follows f JOIN users u ON u.id = f.follower_id \
                 WHERE f.followee_id = $1 AND ($2::BIGINT IS NULL OR u.id < $2) \
                 ORDER BY u.id DESC LIMIT $3"
            }
            FollowListing::Following => {
                "SELECT u.id, u.username, u.display_name, u.bio, u.created_at, u.follower_count, \
                 u.following_count, u.suspended, u.suspended_at \
                 FROM follows f JOIN users u ON u.id = f.followee_id \
                 WHERE f.follower_id = $1 AND ($2::BIGINT IS NULL OR u.id < $2) \
                 ORDER BY u.id DESC LIMIT $3"
            }
        };
        self.with_transaction(move |conn| {
            Box::pin(async move {
                let user_exists: bool =
                    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)")
                        .bind(user_column)
                        .fetch_one(&mut *conn)
                        .await
                        .map_err(|error| {
                            ServerError::Internal(format!(
                                "failed to check user {user} for a follow listing: {error}"
                            ))
                        })?;
                if !user_exists {
                    return Err(ServerError::NotFound(format!("user {user}")));
                }

                let rows: Vec<mapping::UserRow> = sqlx::query_as(sql)
                    .bind(user_column)
                    .bind(cursor)
                    .bind(fetch_limit(limit))
                    .fetch_all(&mut *conn)
                    .await
                    .map_err(|error| {
                        ServerError::Internal(format!("failed to read a follow listing: {error}"))
                    })?;
                let users = rows
                    .into_iter()
                    .map(mapping::UserRow::into_user)
                    .collect::<Result<Vec<User>, ServerError>>()?;
                Ok(paginate(users, limit, |listed| listed.id.get()))
            })
        })
        .await
    }
}

// =================================================================================================
// Store-access substrate (EPIC-PERSIST, SUBTASK_store_access_substrate, #64/#65).
//
// The load-bearing primitives every later durable capability-method author composes against behind
// `&self`, authored once on top of the foundation scaffold (the pool + migration runner above) and
// the committed domain schema (`migrations/0002_domain_schema.sql`). The surface is deliberately
// additive and stable — a downstream method opens a transaction, allocates an id, and commits
// durably through these helpers without re-opening or re-authoring them:
//
//   * `pool` — shared access to the live pool for single-statement reads that need no transaction.
//   * `with_transaction` — the atomicity helper enforcing *confirmed-write-durable-before-success*:
//     a value is returned ONLY after the enclosing transaction has durably committed; an undurable
//     write (closure error, or a failing commit) is reported as a failure and NEVER falsely
//     confirmed (the `confirmed_write_durable` identity).
//   * `allocate_user_id` / `allocate_chirp_id` / `allocate_token_value` — the id & sequence
//     allocator reading the server-side `*_id_seq` / `token_seq` counters the schema owns. The ids
//     it issues are opaque, server-assigned, monotonic, stable across restart, and never reused
//     (the `ids_opaque_monotonic_stable` identity), because a PostgreSQL `SEQUENCE` is durable and
//     only ever advances.
//
// These helpers consume the frozen foundation (store choice, pool construction, migration runner)
// and the committed schema as-is; they re-author neither.
// =================================================================================================

/// A future returned by a [`PostgresRepository::with_transaction`] closure: a boxed, `Send` future
/// resolving to the operation's result, borrowing the transaction's connection for `'c`.
///
/// Aliased so the [`with_transaction`](PostgresRepository::with_transaction) bound reads cleanly and
/// callers have a name for the type their closure returns (they construct it with `Box::pin`).
type TransactionFuture<'c, T> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, ServerError>> + Send + 'c>>;

/// The durable id/sequence counters the schema (`migrations/0002_domain_schema.sql`) owns. Each maps
/// to one PostgreSQL `SEQUENCE`; the substrate allocates an id by reading the next value from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sequence {
    /// `user_id_seq` — backs opaque [`UserId`]s.
    User,
    /// `chirp_id_seq` — backs opaque [`ChirpId`]s.
    Chirp,
    /// `token_seq` — backs the monotonic numeric component of durable session tokens.
    Token,
}

impl Sequence {
    /// The static `SELECT nextval(...)` query for this counter.
    ///
    /// The sequence name is a fixed, crate-controlled identifier (never caller input), so the query
    /// is a `&'static str` literal and carries no injection surface.
    const fn nextval_query(self) -> &'static str {
        match self {
            Sequence::User => "SELECT nextval('user_id_seq')",
            Sequence::Chirp => "SELECT nextval('chirp_id_seq')",
            Sequence::Token => "SELECT nextval('token_seq')",
        }
    }

    /// A short human-readable label for error messages.
    const fn label(self) -> &'static str {
        match self {
            Sequence::User => "user",
            Sequence::Chirp => "chirp",
            Sequence::Token => "token",
        }
    }
}

/// Maps a raw `BIGINT` sequence value onto the `u64` the opaque `chirp-types` ids wrap.
///
/// The `*_id_seq` / `token_seq` sequences are `MINVALUE 1` and only advance, so every value is a
/// positive `i64` that fits in `u64`. A non-positive value would mean the durable store's id
/// counters are corrupt; rather than silently wrapping it into a bogus opaque id we surface it as an
/// [`ServerError::Internal`], so a corrupt counter is reported as a failure and never confirmed.
fn sequence_value_to_u64(raw: i64) -> Result<u64, ServerError> {
    u64::try_from(raw).map_err(|_| {
        ServerError::Internal(format!(
            "id/sequence allocator read a non-positive counter value ({raw}); the durable store's \
             id counters are corrupt"
        ))
    })
}

/// Reads the next value of `sequence` through `executor` and maps it onto `u64`.
///
/// `executor` is any sqlx executor over Postgres — the shared pool (for a standalone allocation) or
/// a `&mut` connection inside a [`with_transaction`](PostgresRepository::with_transaction) closure
/// (so an id is allocated atomically with the write that uses it).
async fn next_sequence_value<'e, E>(executor: E, sequence: Sequence) -> Result<u64, ServerError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let raw: i64 = sqlx::query_scalar(sequence.nextval_query())
        .fetch_one(executor)
        .await
        .map_err(|error| {
            ServerError::Internal(format!(
                "failed to allocate a {} id from the durable store: {error}",
                sequence.label()
            ))
        })?;
    sequence_value_to_u64(raw)
}

/// Does this PostgreSQL `SQLSTATE` denote capacity / resource exhaustion?
///
/// SQLSTATE **class `53`** is *insufficient_resources* — the backing store could not service the
/// statement because a resource ran out: `53100` disk_full, `53200` out_of_memory, `53300`
/// too_many_connections, `53400` configuration_limit_exceeded, and the class-generic `53000`. A
/// SQLSTATE is five characters whose first two are the class, so the whole `53` class (including any
/// future `53xxx` member, which is by definition still *insufficient_resources*) is matched by its
/// class prefix. Every other class (`23` integrity, `08` connection, `42` syntax, `40`
/// transaction-rollback, …) is NOT capacity exhaustion and is left to map to a generic internal
/// failure.
///
/// This is the pure, store-free core of capacity detection; it is unit-tested directly against the
/// SQLSTATE strings with no database (`capacity_failsafe_tests`).
fn is_capacity_exhaustion_sqlstate(sqlstate: &str) -> bool {
    sqlstate.starts_with("53")
}

/// Classifies a raw [`sqlx::Error`] as capacity / resource exhaustion of the durable store.
///
/// Two shapes count as the store being out of capacity:
/// * a database error whose [`SQLSTATE`](sqlx::error::DatabaseError::code) is in the
///   *insufficient_resources* class (see [`is_capacity_exhaustion_sqlstate`]); and
/// * [`sqlx::Error::PoolTimedOut`] — no connection could be acquired within the pool's timeout, i.e.
///   the connection-pool capacity is exhausted.
///
/// Anything else (a constraint violation, a dropped connection, a decode error) is not capacity
/// exhaustion and is classified as a generic internal failure by [`classify_write_error`].
fn is_capacity_exhaustion(error: &sqlx::Error) -> bool {
    match error {
        // The acquire timeout fired: every pooled connection is in use — connection capacity is
        // exhausted.
        sqlx::Error::PoolTimedOut => true,
        // A resource ran out inside the server (disk, memory, connection slots, configured limits).
        sqlx::Error::Database(db) => db
            .code()
            .is_some_and(|sqlstate| is_capacity_exhaustion_sqlstate(sqlstate.as_ref())),
        _ => false,
    }
}

/// Maps a durable write-path [`sqlx::Error`] onto the preserved [`ServerError`] contract, failing
/// **safe**: a capacity / resource-exhaustion failure becomes a typed [`ServerError::Unavailable`],
/// every other failure becomes a [`ServerError::Internal`]. Both are *failed actions* — this
/// function never maps an error onto a success, so a write that could not be durably committed is
/// never falsely confirmed (`capacity_fail_safe_never_false_confirms`). The original `sqlx` error is
/// preserved in the (log-only) message so the underlying cause is never lost.
///
/// This is the single detection point the durable write path routes through, so every write that
/// commits via [`PostgresRepository::with_transaction`] — including the cleanup writes a later
/// retention / transient-lifetime stage adds — inherits the same fail-safe mapping by construction.
fn classify_write_error(error: &sqlx::Error, context: &str) -> ServerError {
    let detail = format!("{context}: {error}");
    if is_capacity_exhaustion(error) {
        ServerError::Unavailable(detail)
    } else {
        ServerError::Internal(detail)
    }
}

impl PostgresRepository {
    /// Borrows the shared connection pool for a single-statement read that needs no transaction.
    ///
    /// Later capability-method authors use this for plain reads (for example `get_user`'s row
    /// lookup); multi-statement or any-write operations go through
    /// [`with_transaction`](Self::with_transaction) instead, so their durability is confirmed.
    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Runs `operations` inside a single durable transaction, returning its value **only after the
    /// transaction has committed** — the `confirmed_write_durable` mechanism.
    ///
    /// The closure receives the transaction's `&mut` connection and performs its reads/writes
    /// against it (allocating ids with [`allocate_user_id`](Self::allocate_user_id) and friends on
    /// the same connection keeps the allocation inside the atomic unit). Control flow:
    ///
    /// * the closure returns `Ok(value)` → the transaction is committed; **only if the commit itself
    ///   succeeds** is `Ok(value)` returned to the caller. A failed commit is reported as a *failed
    ///   action* ([`ServerError::Unavailable`] when the store is out of capacity, otherwise
    ///   [`ServerError::Internal`]) and the write is NOT confirmed;
    /// * the closure returns `Err(e)` → the transaction is rolled back and `Err(e)` is propagated
    ///   unchanged, so a partial write never becomes durable.
    ///
    /// A success return therefore always means "durably committed", and no failure path ever leaves
    /// a write falsely confirmed.
    ///
    /// # Errors
    ///
    /// A failed start or commit is mapped through [`classify_write_error`]: a capacity /
    /// resource-exhaustion failure (PostgreSQL SQLSTATE class `53`, or a connection-pool acquire
    /// timeout) becomes [`ServerError::Unavailable`] and any other failure becomes
    /// [`ServerError::Internal`] — both *failed actions*, so a write that could not be durably
    /// committed is never falsely confirmed. Otherwise whatever error the closure returns.
    pub async fn with_transaction<F, T>(&self, operations: F) -> Result<T, ServerError>
    where
        F: for<'c> FnOnce(&'c mut sqlx::PgConnection) -> TransactionFuture<'c, T>,
        T: Send,
    {
        let mut transaction = self.pool.begin().await.map_err(|error| {
            classify_write_error(&error, "failed to begin a durable transaction")
        })?;

        match operations(&mut transaction).await {
            Ok(value) => {
                transaction.commit().await.map_err(|error| {
                    classify_write_error(
                        &error,
                        "durable commit failed; the write was NOT confirmed",
                    )
                })?;
                Ok(value)
            }
            Err(error) => {
                if let Err(rollback_error) = transaction.rollback().await {
                    tracing::warn!(
                        error = %rollback_error,
                        "failed to roll back a durable transaction after an operation error"
                    );
                }
                Err(error)
            }
        }
    }

    /// Allocates the next opaque, server-assigned [`UserId`] from the durable `user_id_seq` counter.
    ///
    /// `executor` is the shared pool ([`pool`](Self::pool)) or a `&mut` connection from a
    /// [`with_transaction`](Self::with_transaction) closure. The returned id is monotonic, stable
    /// across restart, and never reused (`ids_opaque_monotonic_stable`).
    ///
    /// # Errors
    ///
    /// [`ServerError::Internal`] if the counter cannot be read or yields a corrupt (non-positive)
    /// value.
    pub async fn allocate_user_id<'e, E>(executor: E) -> Result<UserId, ServerError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        Ok(UserId::new(
            next_sequence_value(executor, Sequence::User).await?,
        ))
    }

    /// Allocates the next opaque, server-assigned [`ChirpId`] from the durable `chirp_id_seq`
    /// counter. See [`allocate_user_id`](Self::allocate_user_id) for the executor and guarantees.
    ///
    /// # Errors
    ///
    /// [`ServerError::Internal`] if the counter cannot be read or yields a corrupt (non-positive)
    /// value.
    pub async fn allocate_chirp_id<'e, E>(executor: E) -> Result<ChirpId, ServerError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        Ok(ChirpId::new(
            next_sequence_value(executor, Sequence::Chirp).await?,
        ))
    }

    /// Allocates the next value of the durable `token_seq` counter — the monotonic numeric component
    /// the session-creating capability method composes into an opaque [`SessionToken`] string.
    ///
    /// Returned as a raw `u64` rather than a typed id because a session token is a `TEXT` value, not
    /// one of the `u64` entity-id newtypes. The guarantees match the id allocators: monotonic,
    /// stable across restart, never reused.
    ///
    /// # Errors
    ///
    /// [`ServerError::Internal`] if the counter cannot be read or yields a corrupt (non-positive)
    /// value.
    pub async fn allocate_token_value<'e, E>(executor: E) -> Result<u64, ServerError>
    where
        E: sqlx::Executor<'e, Database = sqlx::Postgres>,
    {
        next_sequence_value(executor, Sequence::Token).await
    }
}

// =================================================================================================
// chirp-types <-> storage mapping + domain-boundary validation
// (EPIC-PERSIST, SUBTASK_mapping_validation, #64/#66).
//
// The reusable pattern every later durable capability-method author composes against to move values
// across the durable boundary WITHOUT re-authoring the conversion or re-implementing the validation.
// It has three faces, and together they establish the `domain_boundary_validated` identity for the
// durable store:
//
//   * to-column (`*_to_column`, `UserRow::from_user`, ...): a validated `chirp-types` value is
//     lowered to the scalar column types the schema (`migrations/0002_domain_schema.sql`) declares
//     (`BIGINT` / `TEXT` / `BOOLEAN`). Because the value objects are unrepresentable-if-invalid, an
//     invalid `Username` / `ChirpText` cannot even be constructed to reach this path — so an invalid
//     value cannot enter the store through the pattern.
//   * from-column (`column_to_*`, `UserRow::into_user`, ...): a stored row is lifted back into a
//     `chirp-types` value THROUGH the validating constructors (`Username::parse` / `ChirpText::parse`),
//     so a row that somehow violates a domain invariant surfaces as a `ServerError` — never a panic
//     and never a silently-wrong value. This is the boundary guard, matching the schema's in-store
//     CHECKs as defence in depth.
//   * invariant guards (`ensure_distinct_users`, `is_unique_violation`, and the structural notes
//     below): entity invariants are expressible at the durable boundary exactly as they are in-memory.
//
// `chirp-types` is canonical and unchanged; this module validates AGAINST it. The pattern is additive
// and authored once — the row types are the single place that names PostgreSQL's scalar
// representation, and every method author reuses them rather than re-deriving the mapping
// (`substrate_authored_once` / `substrate_surface_stable`). It consumes the committed schema and the
// store-access substrate as real code; it re-authors neither.
//
// Expressing the remaining entity invariants at the durable boundary, exactly as in-memory:
//   * no self-follow — `ensure_distinct_users` rejects `follower == followee` with `Forbidden`, as
//     the in-memory implementor does; the schema's `follows_no_self` CHECK is the in-store backstop.
//   * <=1 like per (user, chirp) and <=1 follow per pair — the composite PRIMARY KEYs on `likes` /
//     `follows` (and the `users_username_key` unique handle index) make a duplicate a unique
//     violation; a method author detects it with `is_unique_violation` and maps it to an idempotent
//     no-op (repeated follow/like) or a `Conflict` (duplicate handle).
//   * reverse-chronological timelines — the schema scans by `id DESC` (a larger id is always a later
//     row), the same stable proxy the in-memory implementor uses; the mapping preserves id order
//     because `column_to_*_id` is monotonic in the raw value.
// =================================================================================================

/// The `chirp-types` <-> durable-row mapping and the domain-boundary validation entry points.
///
/// Authored once by `SUBTASK_mapping_validation`; every later `SUBTASK_durable_impl` capability
/// method reuses these entry points rather than re-deriving the conversion or re-implementing the
/// boundary validation. See the module-level comment above for the three faces of the pattern and
/// how it expresses each entity invariant at the durable boundary.
pub mod mapping {
    use chirp_types::domain::{Chirp, ChirpText, Follow, Like, User, Username};
    use chirp_types::ids::{ChirpId, Timestamp, UserId};

    use crate::error::ServerError;

    // ---- Scalar column conversions --------------------------------------------------------------

    /// Lowers an unsigned `chirp-types` scalar (an id or a denormalized count) to the signed
    /// `BIGINT` the schema stores.
    ///
    /// The durable counters and counts are `BIGINT ... MINVALUE 1` / `CHECK (... >= 0)`, so every
    /// legitimate value fits an `i64`. A value that does not fit is impossible for a store-sourced
    /// id/count; rather than wrap it silently the mapping surfaces it as a failure.
    ///
    /// # Errors
    ///
    /// [`ServerError::Internal`] if `value` exceeds [`i64::MAX`].
    pub fn u64_to_column(value: u64, field: &str) -> Result<i64, ServerError> {
        i64::try_from(value).map_err(|_| {
            ServerError::Internal(format!(
                "{field} value {value} does not fit the durable BIGINT column"
            ))
        })
    }

    /// Lifts a signed `BIGINT` column back to the unsigned `chirp-types` scalar it represents.
    ///
    /// # Errors
    ///
    /// [`ServerError::Internal`] if `value` is negative — the durable id/count columns are
    /// non-negative by construction (`MINVALUE 1` / `CHECK (... >= 0)`), so a negative value means
    /// the stored representation is corrupt; it is reported as a failure, never wrapped into a bogus
    /// id/count.
    pub fn column_to_u64(value: i64, field: &str) -> Result<u64, ServerError> {
        u64::try_from(value).map_err(|_| {
            ServerError::Internal(format!(
                "durable {field} column holds a negative value ({value}); the stored \
                 representation is corrupt"
            ))
        })
    }

    /// Lowers a [`UserId`] to its `BIGINT` column value.
    ///
    /// # Errors
    ///
    /// [`ServerError::Internal`] on the impossible overflow described in [`u64_to_column`].
    pub fn user_id_to_column(id: UserId) -> Result<i64, ServerError> {
        u64_to_column(id.get(), "user id")
    }

    /// Lowers a [`ChirpId`] to its `BIGINT` column value.
    ///
    /// # Errors
    ///
    /// [`ServerError::Internal`] on the impossible overflow described in [`u64_to_column`].
    pub fn chirp_id_to_column(id: ChirpId) -> Result<i64, ServerError> {
        u64_to_column(id.get(), "chirp id")
    }

    /// Lifts a `BIGINT` column to a [`UserId`].
    ///
    /// # Errors
    ///
    /// [`ServerError::Internal`] if the column is negative (corrupt store).
    pub fn column_to_user_id(value: i64) -> Result<UserId, ServerError> {
        Ok(UserId::new(column_to_u64(value, "user id")?))
    }

    /// Lifts a `BIGINT` column to a [`ChirpId`].
    ///
    /// # Errors
    ///
    /// [`ServerError::Internal`] if the column is negative (corrupt store).
    pub fn column_to_chirp_id(value: i64) -> Result<ChirpId, ServerError> {
        Ok(ChirpId::new(column_to_u64(value, "chirp id")?))
    }

    /// Lowers a [`Timestamp`] to its `BIGINT` epoch-milliseconds column value.
    ///
    /// Infallible: the stored representation is itself an `i64` ([`Timestamp`] is epoch millis).
    #[must_use]
    pub fn timestamp_to_column(ts: Timestamp) -> i64 {
        ts.as_millis()
    }

    /// Lifts a `BIGINT` epoch-milliseconds column to a [`Timestamp`] (infallible).
    #[must_use]
    pub fn column_to_timestamp(millis: i64) -> Timestamp {
        Timestamp::from_millis(millis)
    }

    // ---- Boundary validation entry points -------------------------------------------------------

    /// Validates a raw handle string at the durable boundary, yielding a well-formed [`Username`].
    ///
    /// The reusable guard for any raw string crossing the boundary as a handle: it routes through
    /// the canonical [`Username::parse`], so an invalid handle cannot pass.
    ///
    /// # Errors
    ///
    /// [`ServerError::Validation`] (never a panic) if `raw` is not a valid [`Username`].
    pub fn validate_username(raw: &str) -> Result<Username, ServerError> {
        Ok(Username::parse(raw)?)
    }

    /// Validates a raw chirp body at the durable boundary, yielding a well-formed [`ChirpText`].
    ///
    /// # Errors
    ///
    /// [`ServerError::Validation`] (never a panic) if `raw` is not a valid [`ChirpText`].
    pub fn validate_chirp_text(raw: &str) -> Result<ChirpText, ServerError> {
        Ok(ChirpText::parse(raw)?)
    }

    /// Upholds the no-self-follow invariant at the durable boundary, exactly as the in-memory
    /// implementor does: a user may not follow themselves.
    ///
    /// Later `follow`/edge authors call this before writing a follow edge; the schema's
    /// `follows_no_self` CHECK is the in-store backstop.
    ///
    /// # Errors
    ///
    /// [`ServerError::Forbidden`] if `follower == followee`.
    pub fn ensure_distinct_users(follower: UserId, followee: UserId) -> Result<(), ServerError> {
        if follower == followee {
            return Err(ServerError::Forbidden(
                "a user cannot follow themselves".to_owned(),
            ));
        }
        Ok(())
    }

    /// Returns `true` when `error` is a durable unique-constraint violation.
    ///
    /// The reusable discriminator for the uniqueness invariants the schema enforces with keys and
    /// indexes (the `users_username_key` unique handle index; the composite primary keys on
    /// `follows` and `likes`). A write author uses it to map a duplicate to the right domain
    /// outcome — a [`Conflict`](ServerError::Conflict) for a duplicate handle, or an idempotent
    /// no-op for a repeated follow/like — rather than surfacing a raw database error.
    #[must_use]
    pub fn is_unique_violation(error: &sqlx::Error) -> bool {
        matches!(error, sqlx::Error::Database(db) if db.is_unique_violation())
    }

    // ---- Row types (entity <-> row, both directions) --------------------------------------------

    /// The durable row shape of the `users` table, mirroring every column 1:1.
    ///
    /// Read a selected row with [`sqlx::query_as`] into this type, then lift it with
    /// [`into_user`](UserRow::into_user); build one for an insert with
    /// [`from_user`](UserRow::from_user) and bind its fields. The moderation columns (`suspended`,
    /// `suspended_at`) have no counterpart on the on-the-wire [`User`]; they are carried here so a
    /// moderation-aware method can read them without a second query.
    #[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
    pub struct UserRow {
        /// `users.id` — the opaque [`UserId`] as a signed `BIGINT`.
        pub id: i64,
        /// `users.username` — the unique handle, stored as `TEXT`.
        pub username: String,
        /// `users.display_name` — free-form display name.
        pub display_name: String,
        /// `users.bio` — optional profile biography (`NULL` when unset).
        pub bio: Option<String>,
        /// `users.created_at` — account creation time, epoch milliseconds.
        pub created_at: i64,
        /// `users.follower_count` — denormalized follower count.
        pub follower_count: i64,
        /// `users.following_count` — denormalized following count.
        pub following_count: i64,
        /// `users.suspended` — moderation suspension flag (not part of the wire [`User`]).
        pub suspended: bool,
        /// `users.suspended_at` — when suspension was applied, epoch millis (`NULL` if not suspended).
        pub suspended_at: Option<i64>,
    }

    impl UserRow {
        /// Lowers a domain [`User`] to its durable row for an insert, as a freshly-created,
        /// non-suspended account.
        ///
        /// # Errors
        ///
        /// [`ServerError::Internal`] on the impossible id/count overflow (see [`u64_to_column`]).
        pub fn from_user(user: &User) -> Result<Self, ServerError> {
            Ok(Self {
                id: user_id_to_column(user.id)?,
                username: user.username.as_str().to_owned(),
                display_name: user.display_name.clone(),
                bio: user.bio.clone(),
                created_at: timestamp_to_column(user.created_at),
                follower_count: u64_to_column(user.follower_count, "follower_count")?,
                following_count: u64_to_column(user.following_count, "following_count")?,
                suspended: false,
                suspended_at: None,
            })
        }

        /// Lifts the row to a validated [`User`], enforcing `chirp-types` validation at the durable
        /// boundary: the stored handle is re-parsed through [`Username::parse`], so a corrupt row
        /// surfaces as an error rather than a malformed value (`domain_boundary_validated`).
        ///
        /// # Errors
        ///
        /// [`ServerError::Validation`] if the stored handle is not a valid [`Username`];
        /// [`ServerError::Internal`] if an id/count column is corrupt (negative).
        pub fn into_user(self) -> Result<User, ServerError> {
            Ok(User {
                id: column_to_user_id(self.id)?,
                username: validate_username(&self.username)?,
                display_name: self.display_name,
                bio: self.bio,
                created_at: column_to_timestamp(self.created_at),
                follower_count: column_to_u64(self.follower_count, "follower_count")?,
                following_count: column_to_u64(self.following_count, "following_count")?,
            })
        }
    }

    /// The durable row shape of the `chirps` table, mirroring every column 1:1.
    ///
    /// The moderation columns (`removed`, `removed_at`) have no counterpart on the wire [`Chirp`];
    /// they are carried for a removal-aware method to read.
    #[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
    pub struct ChirpRow {
        /// `chirps.id` — the opaque [`ChirpId`] as a signed `BIGINT`.
        pub id: i64,
        /// `chirps.author_id` — the authoring [`UserId`].
        pub author_id: i64,
        /// `chirps.text` — the chirp body, stored as `TEXT`.
        pub text: String,
        /// `chirps.created_at` — post time, epoch milliseconds.
        pub created_at: i64,
        /// `chirps.like_count` — denormalized like count.
        pub like_count: i64,
        /// `chirps.reply_to` — the replied-to [`ChirpId`], if any (`NULL` for a top-level chirp).
        pub reply_to: Option<i64>,
        /// `chirps.removed` — moderation soft-removal flag (not part of the wire [`Chirp`]).
        pub removed: bool,
        /// `chirps.removed_at` — when the chirp was removed, epoch millis (`NULL` if not removed).
        pub removed_at: Option<i64>,
    }

    impl ChirpRow {
        /// Lowers a domain [`Chirp`] to its durable row for an insert, as a freshly-posted,
        /// not-removed chirp.
        ///
        /// # Errors
        ///
        /// [`ServerError::Internal`] on the impossible id/count overflow (see [`u64_to_column`]).
        pub fn from_chirp(chirp: &Chirp) -> Result<Self, ServerError> {
            Ok(Self {
                id: chirp_id_to_column(chirp.id)?,
                author_id: user_id_to_column(chirp.author_id)?,
                text: chirp.text.as_str().to_owned(),
                created_at: timestamp_to_column(chirp.created_at),
                like_count: u64_to_column(chirp.like_count, "like_count")?,
                reply_to: chirp.reply_to.map(chirp_id_to_column).transpose()?,
                removed: false,
                removed_at: None,
            })
        }

        /// Lifts the row to a validated [`Chirp`], re-parsing the stored body through
        /// [`ChirpText::parse`] at the durable boundary (`domain_boundary_validated`).
        ///
        /// # Errors
        ///
        /// [`ServerError::Validation`] if the stored body is not a valid [`ChirpText`];
        /// [`ServerError::Internal`] if an id/count column is corrupt (negative).
        pub fn into_chirp(self) -> Result<Chirp, ServerError> {
            Ok(Chirp {
                id: column_to_chirp_id(self.id)?,
                author_id: column_to_user_id(self.author_id)?,
                text: validate_chirp_text(&self.text)?,
                created_at: column_to_timestamp(self.created_at),
                like_count: column_to_u64(self.like_count, "like_count")?,
                reply_to: self.reply_to.map(column_to_chirp_id).transpose()?,
            })
        }
    }

    /// The durable row shape of a `follows` edge, mirroring every column 1:1.
    #[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
    pub struct FollowRow {
        /// `follows.follower_id` — the following [`UserId`].
        pub follower_id: i64,
        /// `follows.followee_id` — the followed [`UserId`].
        pub followee_id: i64,
        /// `follows.created_at` — when the edge was created, epoch milliseconds.
        pub created_at: i64,
    }

    impl FollowRow {
        /// Lowers a domain [`Follow`] edge to its durable row.
        ///
        /// # Errors
        ///
        /// [`ServerError::Internal`] on the impossible id overflow (see [`u64_to_column`]).
        pub fn from_follow(follow: &Follow) -> Result<Self, ServerError> {
            Ok(Self {
                follower_id: user_id_to_column(follow.follower_id)?,
                followee_id: user_id_to_column(follow.followee_id)?,
                created_at: timestamp_to_column(follow.created_at),
            })
        }

        /// Lifts the row to a domain [`Follow`] edge.
        ///
        /// # Errors
        ///
        /// [`ServerError::Internal`] if an id column is corrupt (negative).
        pub fn into_follow(self) -> Result<Follow, ServerError> {
            Ok(Follow {
                follower_id: column_to_user_id(self.follower_id)?,
                followee_id: column_to_user_id(self.followee_id)?,
                created_at: column_to_timestamp(self.created_at),
            })
        }
    }

    /// The durable row shape of a `likes` edge, mirroring every column 1:1.
    #[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
    pub struct LikeRow {
        /// `likes.user_id` — the liking [`UserId`].
        pub user_id: i64,
        /// `likes.chirp_id` — the liked [`ChirpId`].
        pub chirp_id: i64,
        /// `likes.created_at` — when the like was recorded, epoch milliseconds.
        pub created_at: i64,
    }

    impl LikeRow {
        /// Lowers a domain [`Like`] edge to its durable row.
        ///
        /// # Errors
        ///
        /// [`ServerError::Internal`] on the impossible id overflow (see [`u64_to_column`]).
        pub fn from_like(like: &Like) -> Result<Self, ServerError> {
            Ok(Self {
                user_id: user_id_to_column(like.user_id)?,
                chirp_id: chirp_id_to_column(like.chirp_id)?,
                created_at: timestamp_to_column(like.created_at),
            })
        }

        /// Lifts the row to a domain [`Like`] edge.
        ///
        /// # Errors
        ///
        /// [`ServerError::Internal`] if an id column is corrupt (negative).
        pub fn into_like(self) -> Result<Like, ServerError> {
            Ok(Like {
                user_id: column_to_user_id(self.user_id)?,
                chirp_id: column_to_chirp_id(self.chirp_id)?,
                created_at: column_to_timestamp(self.created_at),
            })
        }
    }
}

#[cfg(test)]
mod substrate_tests {
    //! Store-free unit tests for the substrate's pure logic — the sequence-value mapping and the
    //! per-counter query selection. These need no database and exercise the parts of the mechanism
    //! that are checkable without a live store; the end-to-end durability/monotonicity behaviour is
    //! covered by `substrate_live_tests` (gated on a real store) and, end-to-end against the durable
    //! backing, by the proving slice.

    use super::{sequence_value_to_u64, Sequence};
    use chirp_types::api::ErrorCode;

    #[test]
    fn sequence_value_maps_a_positive_counter_to_the_matching_u64() {
        assert_eq!(sequence_value_to_u64(1).unwrap(), 1);
        assert_eq!(sequence_value_to_u64(42).unwrap(), 42);
        assert_eq!(sequence_value_to_u64(i64::MAX).unwrap(), i64::MAX as u64);
    }

    #[test]
    fn sequence_value_rejects_a_negative_counter_as_an_internal_error() {
        // A corrupt (non-positive) counter must surface as a failure, never a silently-wrapped id.
        let error = sequence_value_to_u64(-1).unwrap_err();
        assert_eq!(error.code(), ErrorCode::Internal);
    }

    #[test]
    fn each_sequence_selects_the_query_naming_its_own_counter() {
        assert!(Sequence::User.nextval_query().contains("user_id_seq"));
        assert!(Sequence::Chirp.nextval_query().contains("chirp_id_seq"));
        assert!(Sequence::Token.nextval_query().contains("token_seq"));
        // The three counters are distinct so ids never collide across entity kinds.
        assert_ne!(
            Sequence::User.nextval_query(),
            Sequence::Chirp.nextval_query()
        );
        assert_ne!(
            Sequence::Chirp.nextval_query(),
            Sequence::Token.nextval_query()
        );
    }
}

#[cfg(test)]
mod capacity_failsafe_tests {
    //! Store-free unit tests for the capacity-fail-safe classification logic — the pure SQLSTATE
    //! predicate that decides whether a durable write-path failure is capacity / resource
    //! exhaustion. These need no database: they fault-inject the SQLSTATE strings directly and
    //! assert the classification, so they run under `--features postgres` without a live store and
    //! the default `cargo test` stays store-free. The end-to-end behaviour against a real,
    //! capacity-constrained backing — a `sqlx::Error` carrying one of these codes, and an exhausted
    //! connection pool — is covered by the bounded-capacity live test in `substrate_live_tests`
    //! (gated on `CHIRP_TEST_DATABASE_URL`), because `sqlx::Error` is `#[non_exhaustive]` and cannot
    //! be constructed outside `sqlx`.

    use super::is_capacity_exhaustion_sqlstate;

    #[test]
    fn insufficient_resources_class_53_is_capacity_exhaustion() {
        // SQLSTATE class 53 = insufficient_resources: every member is capacity/resource exhaustion.
        assert!(is_capacity_exhaustion_sqlstate("53000")); // insufficient_resources (class-generic)
        assert!(is_capacity_exhaustion_sqlstate("53100")); // disk_full
        assert!(is_capacity_exhaustion_sqlstate("53200")); // out_of_memory
        assert!(is_capacity_exhaustion_sqlstate("53300")); // too_many_connections
        assert!(is_capacity_exhaustion_sqlstate("53400")); // configuration_limit_exceeded
    }

    #[test]
    fn other_sqlstate_classes_are_not_capacity_exhaustion() {
        // Real failures, but NOT capacity exhaustion — they must map to a generic internal failure,
        // never to Unavailable, so the capacity category stays truthful.
        assert!(!is_capacity_exhaustion_sqlstate("23505")); // unique_violation
        assert!(!is_capacity_exhaustion_sqlstate("23503")); // foreign_key_violation
        assert!(!is_capacity_exhaustion_sqlstate("08006")); // connection_failure
        assert!(!is_capacity_exhaustion_sqlstate("42601")); // syntax_error
        assert!(!is_capacity_exhaustion_sqlstate("40001")); // serialization_failure
        assert!(!is_capacity_exhaustion_sqlstate("")); // absent/empty code
                                                       // The match is on the leading class, so a non-53 class whose code merely ends in "53" or
                                                       // contains "53" is not mistaken for exhaustion.
        assert!(!is_capacity_exhaustion_sqlstate("40053"));
        assert!(!is_capacity_exhaustion_sqlstate("25301"));
    }
}

#[cfg(test)]
mod substrate_live_tests {
    //! Mechanism tests that require a real PostgreSQL store. They are **skipped** unless
    //! `CHIRP_TEST_DATABASE_URL` names a reachable test database, so the default `cargo test` (which
    //! does not even compile this `postgres`-gated module) and `cargo test --features postgres`
    //! without a database both stay green and store-free. When a store IS provided they demonstrate
    //! the two substrate identities directly against the durable backing.
    //!
    //! The committed baseline table `chirp_store_meta (key TEXT PRIMARY KEY, value TEXT)` is used as
    //! a neutral write target so these tests exercise commit/rollback without depending on the
    //! domain tables' foreign keys; each test derives a unique key from the token counter so reruns
    //! never collide.

    use super::PostgresRepository;
    use crate::error::ServerError;

    /// The test store URL, or `None` when unset/blank (→ the test skips).
    fn test_database_url() -> Option<String> {
        std::env::var("CHIRP_TEST_DATABASE_URL")
            .ok()
            .filter(|url| !url.trim().is_empty())
    }

    #[tokio::test]
    async fn allocated_ids_are_monotonic_and_never_reused() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url)
            .await
            .expect("connect + migrate the test store");

        let first = PostgresRepository::allocate_user_id(repo.pool())
            .await
            .unwrap();
        let second = PostgresRepository::allocate_user_id(repo.pool())
            .await
            .unwrap();
        let third = PostgresRepository::allocate_user_id(repo.pool())
            .await
            .unwrap();

        // Strictly increasing ⇒ monotonic and never reused (ids_opaque_monotonic_stable).
        assert!(first.get() < second.get());
        assert!(second.get() < third.get());
    }

    #[tokio::test]
    async fn a_committed_write_is_durable_and_visible_after_commit() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();
        let key = format!(
            "substrate_commit_{}",
            PostgresRepository::allocate_token_value(repo.pool())
                .await
                .unwrap()
        );

        let insert_key = key.clone();
        repo.with_transaction(move |conn| {
            Box::pin(async move {
                sqlx::query("INSERT INTO chirp_store_meta (key, value) VALUES ($1, $2)")
                    .bind(&insert_key)
                    .bind("committed")
                    .execute(&mut *conn)
                    .await
                    .map_err(|error| ServerError::Internal(error.to_string()))?;
                Ok(())
            })
        })
        .await
        .expect("the transaction commits durably");

        let stored: Option<String> =
            sqlx::query_scalar("SELECT value FROM chirp_store_meta WHERE key = $1")
                .bind(&key)
                .fetch_optional(repo.pool())
                .await
                .unwrap();
        assert_eq!(stored.as_deref(), Some("committed"));
    }

    #[tokio::test]
    async fn a_failed_transaction_rolls_back_and_is_never_confirmed() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();
        let key = format!(
            "substrate_rollback_{}",
            PostgresRepository::allocate_token_value(repo.pool())
                .await
                .unwrap()
        );

        let insert_key = key.clone();
        let result: Result<(), ServerError> = repo
            .with_transaction(move |conn| {
                Box::pin(async move {
                    sqlx::query("INSERT INTO chirp_store_meta (key, value) VALUES ($1, $2)")
                        .bind(&insert_key)
                        .bind("should-not-persist")
                        .execute(&mut *conn)
                        .await
                        .map_err(|error| ServerError::Internal(error.to_string()))?;
                    // Fail AFTER the write: a confirmed-write-durable helper must not let this
                    // partially-written row become durable.
                    Err(ServerError::Internal(
                        "deliberate failure after the write".into(),
                    ))
                })
            })
            .await;

        assert!(result.is_err(), "the transaction must report failure");
        let stored: Option<String> =
            sqlx::query_scalar("SELECT value FROM chirp_store_meta WHERE key = $1")
                .bind(&key)
                .fetch_optional(repo.pool())
                .await
                .unwrap();
        assert_eq!(
            stored, None,
            "a write inside a failed transaction must never be durable"
        );
    }

    #[tokio::test]
    async fn an_exhausted_connection_pool_fails_safe_as_unavailable() {
        // Bounded-capacity fault injection: a pool with a single connection, held open, leaves no
        // capacity for a second concurrent transaction. The acquire times out, and the durable
        // write path must report that as a *failed action* through ServerError::Unavailable — never
        // a false confirmation (capacity_fail_safe_never_false_confirms), and the atomicity of the
        // transaction mechanism is untouched (nothing was committed).
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };

        use sqlx::postgres::PgPoolOptions;
        use std::time::Duration;

        // A deliberately capacity-of-one pool with a short acquire timeout.
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .acquire_timeout(Duration::from_millis(250))
            .connect(&url)
            .await
            .expect("connect a single-connection test pool");
        let repo = PostgresRepository { pool };

        // Hold the one and only connection open, exhausting the pool for the rest of the test.
        let _held = repo
            .pool()
            .acquire()
            .await
            .expect("acquire the single pooled connection");

        // A transaction now cannot obtain a connection within the acquire timeout; begin() times
        // out and the write path classifies that as capacity exhaustion.
        let result: Result<(), ServerError> = repo
            .with_transaction(move |conn| {
                Box::pin(async move {
                    sqlx::query("INSERT INTO chirp_store_meta (key, value) VALUES ($1, $2)")
                        .bind("capacity-failsafe-should-not-persist")
                        .bind("x")
                        .execute(&mut *conn)
                        .await
                        .map(|_| ())
                        .map_err(|error| ServerError::Internal(error.to_string()))
                })
            })
            .await;

        match result {
            Err(ServerError::Unavailable(_)) => { /* fail-safe: capacity exhaustion reported */ }
            other => panic!(
                "an exhausted pool must fail safe as ServerError::Unavailable, got {other:?}"
            ),
        }

        // And nothing was committed: the failed, uncommitted write left no durable row.
        let stored: Option<String> =
            sqlx::query_scalar("SELECT value FROM chirp_store_meta WHERE key = $1")
                .bind("capacity-failsafe-should-not-persist")
                .fetch_optional(repo.pool())
                .await
                .expect("read back the neutral key");
        assert_eq!(
            stored, None,
            "a write that failed safe on capacity exhaustion must never be durable"
        );
    }
}

#[cfg(test)]
mod mapping_tests {
    //! Store-free unit tests for the mapping/validation pattern: they need no database and cover
    //! the boundary validation, the entity-invariant guards, and that every entity round-trips
    //! through its row mapping unchanged. End-to-end exercise against the durable backing is the
    //! proving slice's job.

    use super::mapping;
    use chirp_types::api::ErrorCode;
    use chirp_types::domain::{Chirp, ChirpText, Follow, Like, User, Username};
    use chirp_types::ids::{ChirpId, Timestamp, UserId};

    fn sample_user() -> User {
        User {
            id: UserId::new(7),
            username: Username::parse("alice_01").unwrap(),
            display_name: "Alice".to_owned(),
            bio: Some("hi there".to_owned()),
            created_at: Timestamp::from_millis(1_700_000_000_000),
            follower_count: 3,
            following_count: 5,
        }
    }

    fn sample_chirp(reply_to: Option<ChirpId>) -> Chirp {
        Chirp {
            id: ChirpId::new(42),
            author_id: UserId::new(7),
            text: ChirpText::parse("hello durable world").unwrap(),
            created_at: Timestamp::from_millis(1_700_000_001_000),
            like_count: 2,
            reply_to,
        }
    }

    #[test]
    fn validate_username_accepts_valid_and_rejects_invalid() {
        assert_eq!(mapping::validate_username("bob").unwrap().as_str(), "bob");
        assert_eq!(
            mapping::validate_username("has space").unwrap_err().code(),
            ErrorCode::ValidationError
        );
        assert_eq!(
            mapping::validate_username("").unwrap_err().code(),
            ErrorCode::ValidationError
        );
    }

    #[test]
    fn validate_chirp_text_accepts_valid_and_rejects_invalid() {
        assert_eq!(mapping::validate_chirp_text("hi").unwrap().as_str(), "hi");
        assert_eq!(
            mapping::validate_chirp_text("").unwrap_err().code(),
            ErrorCode::ValidationError
        );
        let too_long = "x".repeat(281);
        assert_eq!(
            mapping::validate_chirp_text(&too_long).unwrap_err().code(),
            ErrorCode::ValidationError
        );
    }

    #[test]
    fn user_round_trips_through_the_row_mapping() {
        let user = sample_user();
        let row = mapping::UserRow::from_user(&user).unwrap();
        // The wire-irrelevant moderation columns default to "not suspended" for a fresh account.
        assert!(!row.suspended);
        assert_eq!(row.suspended_at, None);
        // An entity written through the mapping reads back equal to its chirp-types value.
        let back = row.into_user().unwrap();
        assert_eq!(back, user);
    }

    #[test]
    fn chirp_round_trips_including_optional_reply() {
        for reply in [None, Some(ChirpId::new(40))] {
            let chirp = sample_chirp(reply);
            let back = mapping::ChirpRow::from_chirp(&chirp)
                .unwrap()
                .into_chirp()
                .unwrap();
            assert_eq!(back, chirp);
            assert_eq!(back.reply_to, reply);
        }
    }

    #[test]
    fn follow_and_like_rows_round_trip() {
        let follow = Follow {
            follower_id: UserId::new(1),
            followee_id: UserId::new(2),
            created_at: Timestamp::from_millis(10),
        };
        assert_eq!(
            mapping::FollowRow::from_follow(&follow)
                .unwrap()
                .into_follow()
                .unwrap(),
            follow
        );

        let like = Like {
            user_id: UserId::new(1),
            chirp_id: ChirpId::new(9),
            created_at: Timestamp::from_millis(20),
        };
        assert_eq!(
            mapping::LikeRow::from_like(&like)
                .unwrap()
                .into_like()
                .unwrap(),
            like
        );
    }

    #[test]
    fn into_user_rejects_a_corrupt_stored_handle_at_the_boundary() {
        // A row whose stored handle violates the chirp-types invariant cannot become a domain
        // User: the boundary re-validates and surfaces a ServerError rather than a bad value or a
        // panic (domain_boundary_validated).
        let row = mapping::UserRow {
            id: 7,
            username: "has a space".to_owned(),
            display_name: "Corrupt".to_owned(),
            bio: None,
            created_at: 1,
            follower_count: 0,
            following_count: 0,
            suspended: false,
            suspended_at: None,
        };
        assert_eq!(
            row.into_user().unwrap_err().code(),
            ErrorCode::ValidationError
        );
    }

    #[test]
    fn into_chirp_rejects_corrupt_stored_text_at_the_boundary() {
        let row = mapping::ChirpRow {
            id: 1,
            author_id: 1,
            text: String::new(), // an empty body violates ChirpText
            created_at: 1,
            like_count: 0,
            reply_to: None,
            removed: false,
            removed_at: None,
        };
        assert_eq!(
            row.into_chirp().unwrap_err().code(),
            ErrorCode::ValidationError
        );
    }

    #[test]
    fn column_conversions_round_trip_and_reject_corrupt_values() {
        assert_eq!(mapping::u64_to_column(0, "x").unwrap(), 0);
        assert_eq!(mapping::u64_to_column(1234, "x").unwrap(), 1234);
        assert_eq!(mapping::column_to_u64(1234, "x").unwrap(), 1234);
        // A negative stored id/count is corrupt -> Internal, never a wrapped value.
        assert_eq!(
            mapping::column_to_u64(-1, "user id").unwrap_err().code(),
            ErrorCode::Internal
        );
        // Id and timestamp column mappings round-trip.
        let id = UserId::new(99);
        assert_eq!(
            mapping::column_to_user_id(mapping::user_id_to_column(id).unwrap()).unwrap(),
            id
        );
        let chirp_id = ChirpId::new(123);
        assert_eq!(
            mapping::column_to_chirp_id(mapping::chirp_id_to_column(chirp_id).unwrap()).unwrap(),
            chirp_id
        );
        let ts = Timestamp::from_millis(1_700_000_000_123);
        assert_eq!(
            mapping::column_to_timestamp(mapping::timestamp_to_column(ts)),
            ts
        );
    }

    #[test]
    fn ensure_distinct_users_upholds_no_self_follow() {
        assert!(mapping::ensure_distinct_users(UserId::new(1), UserId::new(2)).is_ok());
        assert_eq!(
            mapping::ensure_distinct_users(UserId::new(1), UserId::new(1))
                .unwrap_err()
                .code(),
            ErrorCode::Forbidden
        );
    }
}

#[cfg(test)]
mod proving_slice_tests {
    //! End-to-end proving-slice tests for durable `create_user` (incl. the unique-handle index)
    //! and `get_user`, exercised against a REAL PostgreSQL backing — the walking skeleton that
    //! proves the store-access substrate before any later `durable_impl` stage builds on it.
    //!
    //! They are **skipped** unless `CHIRP_TEST_DATABASE_URL` names a reachable test database, so
    //! the default `cargo test` (which does not compile this `postgres`-gated module) and
    //! `cargo test --features postgres` without a database both stay green and store-free. When a
    //! store IS provided, the slice demonstrates the three substrate identities end-to-end against
    //! the durable backing:
    //!
    //!   * `confirmed_write_durable` — a committed `create_user` is durably visible afterwards (and
    //!     across a reconnect); a rejected duplicate write is never falsely confirmed.
    //!   * `ids_opaque_monotonic_stable` — assigned [`UserId`]s are strictly increasing and stay
    //!     larger across a reconnect (monotonic, stable across restart, never reused).
    //!   * `domain_boundary_validated` — a handle that violates the `chirp-types` invariant cannot
    //!     be read back out of the store; the boundary surfaces a validation error, not a bad value.
    //!
    //! Each test derives its handles from the durable token counter so repeated runs against the
    //! same store never collide on the unique-handle index.

    use super::{mapping, PostgresRepository};
    use crate::repository::ChirpRepository;
    use chirp_types::api::{CreateUserRequest, ErrorCode};
    use chirp_types::domain::Username;

    /// The test store URL, or `None` when unset/blank (→ the test skips).
    fn test_database_url() -> Option<String> {
        std::env::var("CHIRP_TEST_DATABASE_URL")
            .ok()
            .filter(|url| !url.trim().is_empty())
    }

    /// A rerun-safe, schema-valid handle derived from the durable `token_seq` counter.
    ///
    /// `"u"` + up to 14 digits stays within the 1..=15-char username limit and the ASCII
    /// alphanumeric/underscore charset, so repeated runs against the same store never collide on
    /// `users_username_key`.
    async fn unique_handle(repo: &PostgresRepository) -> Username {
        let counter = PostgresRepository::allocate_token_value(repo.pool())
            .await
            .expect("allocate a token value for a unique handle");
        Username::parse(format!("u{}", counter % 100_000_000_000_000))
            .expect("the derived handle is a valid Username")
    }

    /// A `CreateUserRequest` for `handle` with a derived display name and no bio.
    fn request(handle: &Username) -> CreateUserRequest {
        CreateUserRequest {
            username: handle.clone(),
            display_name: format!("{handle} display"),
            bio: None,
        }
    }

    #[tokio::test]
    async fn create_user_is_durable_and_get_user_round_trips() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url)
            .await
            .expect("connect + migrate the test store");

        let handle = unique_handle(&repo).await;
        let created = repo
            .create_user(request(&handle))
            .await
            .expect("create_user succeeds");

        // The returned profile carries a server-assigned id and zeroed counts, and matches the
        // request's handle.
        assert_eq!(created.username, handle);
        assert_eq!(created.follower_count, 0);
        assert_eq!(created.following_count, 0);

        // confirmed_write_durable + round trip: the write is visible via get_user AFTER returning.
        let fetched = repo
            .get_user(created.id)
            .await
            .expect("get_user after create");
        assert_eq!(fetched, created);

        // Stable across restart: a FRESH connection to the same durable store still sees the row.
        let reconnected = PostgresRepository::connect(&url)
            .await
            .expect("reconnect to the same store");
        let after_restart = reconnected
            .get_user(created.id)
            .await
            .expect("get_user after reconnect");
        assert_eq!(after_restart, created);
    }

    #[tokio::test]
    async fn get_user_is_not_found_for_an_absent_id() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();

        // An id freshly drawn from the counter but never used to create a row.
        let absent = PostgresRepository::allocate_user_id(repo.pool())
            .await
            .unwrap();
        let error = repo.get_user(absent).await.expect_err("absent user");
        assert_eq!(error.code(), ErrorCode::NotFound);
    }

    #[tokio::test]
    async fn duplicate_handle_is_a_conflict_and_is_never_falsely_confirmed() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();
        let handle = unique_handle(&repo).await;

        let first = repo
            .create_user(request(&handle))
            .await
            .expect("first create succeeds");

        // The unique-handle index rejects the duplicate as a Conflict...
        let error = repo
            .create_user(request(&handle))
            .await
            .expect_err("a duplicate handle must conflict");
        assert_eq!(error.code(), ErrorCode::Conflict);

        // ...and the rejected write was never confirmed: the handle still resolves to exactly the
        // first account, unchanged (confirmed_write_durable, negative case).
        let fetched = repo
            .get_user(first.id)
            .await
            .expect("the first user is still present");
        assert_eq!(fetched, first);
    }

    #[tokio::test]
    async fn assigned_ids_are_opaque_monotonic_and_stable_across_restart() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();

        let handle_one = unique_handle(&repo).await;
        let handle_two = unique_handle(&repo).await;
        let user_one = repo.create_user(request(&handle_one)).await.unwrap();
        let user_two = repo.create_user(request(&handle_two)).await.unwrap();

        // Strictly increasing ⇒ monotonic and never reused within a run.
        assert!(user_one.id.get() < user_two.id.get());

        // Stable across restart + never reused: a fresh connection assigns a STILL-larger id, and
        // the earlier users remain retrievable unchanged.
        let reconnected = PostgresRepository::connect(&url).await.unwrap();
        let handle_three = unique_handle(&reconnected).await;
        let user_three = reconnected
            .create_user(request(&handle_three))
            .await
            .unwrap();
        assert!(user_three.id.get() > user_two.id.get());
        assert_eq!(reconnected.get_user(user_one.id).await.unwrap(), user_one);
    }

    #[tokio::test]
    async fn an_invalid_handle_cannot_be_read_back_out_of_the_store() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();

        // domain_boundary_validated: the well-typed create path cannot introduce an invalid handle
        // (`Username` is unrepresentable-if-invalid). To prove the READ boundary guards the store,
        // inject a corrupt handle directly via SQL (bypassing the domain type) — "has a space" is
        // 11 chars, so it clears the in-store length CHECK but violates the Username charset — then
        // assert get_user surfaces a validation error rather than a malformed value or a panic.
        let id = PostgresRepository::allocate_user_id(repo.pool())
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO users (id, username, display_name, bio, created_at) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(mapping::user_id_to_column(id).unwrap())
        .bind("has a space")
        .bind("Corrupt")
        .bind(Option::<String>::None)
        .bind(0_i64)
        .execute(repo.pool())
        .await
        .expect("inject a corrupt row directly");

        let error = repo
            .get_user(id)
            .await
            .expect_err("a corrupt stored handle is rejected at the boundary");
        assert_eq!(error.code(), ErrorCode::ValidationError);
    }
}

#[cfg(test)]
mod credential_durability_tests {
    //! End-to-end tests for durable `set_credential` / `verify_credential` against a REAL
    //! PostgreSQL backing, composing on the committed store-access substrate and the `credentials`
    //! table from `SUBTASK_store_substrate`.
    //!
    //! Like the proving-slice tests they are **skipped** unless `CHIRP_TEST_DATABASE_URL` names a
    //! reachable test database, so the default `cargo test` (which does not compile this
    //! `postgres`-gated module) and `cargo test --features postgres` without a database both stay
    //! green and store-free. When a store IS provided they demonstrate, against the durable backing:
    //!
    //!   * `confirmed_write_durable` — a credential set is visible to a later verify, and still
    //!     verifies across a fresh reconnect (durable, pod-independent).
    //!   * set/replace semantics — re-setting a credential replaces it (one row per user); the old
    //!     secret stops verifying and the new one starts.
    //!   * `domain_boundary_validated` error contract — `set_credential` is [`NotFound`] for an
    //!     absent user, and `verify_credential` is [`Unauthorized`] for an unknown handle, a
    //!     handle with no credential, or a secret mismatch — never a panic.
    //!
    //! Handles are derived from the durable token counter so repeated runs never collide on the
    //! unique-handle index.
    //!
    //! [`NotFound`]: crate::error::ServerError::NotFound
    //! [`Unauthorized`]: crate::error::ServerError::Unauthorized

    use super::PostgresRepository;
    use crate::repository::ChirpRepository;
    use chirp_types::api::{CreateUserRequest, ErrorCode};
    use chirp_types::domain::{User, Username};

    /// The test store URL, or `None` when unset/blank (→ the test skips).
    fn test_database_url() -> Option<String> {
        std::env::var("CHIRP_TEST_DATABASE_URL")
            .ok()
            .filter(|url| !url.trim().is_empty())
    }

    /// A rerun-safe, schema-valid handle derived from the durable `token_seq` counter, so repeated
    /// runs against the same store never collide on `users_username_key`.
    async fn unique_handle(repo: &PostgresRepository) -> Username {
        let counter = PostgresRepository::allocate_token_value(repo.pool())
            .await
            .expect("allocate a token value for a unique handle");
        Username::parse(format!("u{}", counter % 100_000_000_000_000))
            .expect("the derived handle is a valid Username")
    }

    /// Creates a durable user with a unique handle and returns the created profile.
    async fn seed_user(repo: &PostgresRepository) -> User {
        let handle = unique_handle(repo).await;
        repo.create_user(CreateUserRequest {
            username: handle.clone(),
            display_name: format!("{handle} display"),
            bio: None,
        })
        .await
        .expect("create_user succeeds")
    }

    #[tokio::test]
    async fn set_credential_then_verify_round_trips_and_is_durable_across_reconnect() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url)
            .await
            .expect("connect + migrate the test store");

        let user = seed_user(&repo).await;
        repo.set_credential(user.id, "s3cr3t")
            .await
            .expect("set_credential succeeds for an existing user");

        // confirmed_write_durable: the set is visible to a verify after returning.
        let verified = repo
            .verify_credential(&user.username, "s3cr3t")
            .await
            .expect("the right secret verifies");
        assert_eq!(verified, user.id);

        // Durable across restart: a FRESH connection to the same store still verifies.
        let reconnected = PostgresRepository::connect(&url)
            .await
            .expect("reconnect to the same store");
        let after_restart = reconnected
            .verify_credential(&user.username, "s3cr3t")
            .await
            .expect("the credential verifies after reconnect");
        assert_eq!(after_restart, user.id);
    }

    #[tokio::test]
    async fn set_credential_replaces_the_previous_secret() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();

        let user = seed_user(&repo).await;
        repo.set_credential(user.id, "old").await.unwrap();
        repo.set_credential(user.id, "new").await.unwrap();

        // The replaced secret no longer verifies; only the current one does (one row per user).
        let old = repo
            .verify_credential(&user.username, "old")
            .await
            .expect_err("the replaced secret must not verify");
        assert_eq!(old.code(), ErrorCode::Unauthorized);
        assert_eq!(
            repo.verify_credential(&user.username, "new").await.unwrap(),
            user.id
        );
    }

    #[tokio::test]
    async fn set_credential_is_not_found_for_an_absent_user() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();

        // An id freshly drawn from the counter but never used to create a user row.
        let absent = PostgresRepository::allocate_user_id(repo.pool())
            .await
            .unwrap();
        let error = repo
            .set_credential(absent, "secret")
            .await
            .expect_err("no such user");
        assert_eq!(error.code(), ErrorCode::NotFound);
    }

    #[tokio::test]
    async fn verify_credential_is_unauthorized_for_unknown_handle_missing_credential_or_mismatch() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();

        // Unknown handle (never created).
        let unknown = unique_handle(&repo).await;
        assert_eq!(
            repo.verify_credential(&unknown, "x")
                .await
                .expect_err("unknown handle")
                .code(),
            ErrorCode::Unauthorized
        );

        // Known handle, but no credential has been set yet.
        let user = seed_user(&repo).await;
        assert_eq!(
            repo.verify_credential(&user.username, "x")
                .await
                .expect_err("no credential set")
                .code(),
            ErrorCode::Unauthorized
        );

        // Known handle with a credential, but the wrong secret.
        repo.set_credential(user.id, "right").await.unwrap();
        assert_eq!(
            repo.verify_credential(&user.username, "wrong")
                .await
                .expect_err("wrong secret")
                .code(),
            ErrorCode::Unauthorized
        );
    }
}

#[cfg(test)]
mod session_lifecycle_tests {
    //! End-to-end tests for the durable session lifecycle — `login` / `resolve_token` /
    //! `invalidate_session` — against a REAL PostgreSQL backing, composing on the committed
    //! store-access substrate (`with_transaction`, `allocate_token_value`), the `mapping` pattern, and
    //! the `sessions` table from `SUBTASK_store_substrate`.
    //!
    //! Like the proving-slice and credential tests they are **skipped** unless
    //! `CHIRP_TEST_DATABASE_URL` names a reachable test database, so the default `cargo test` (which
    //! does not compile this `postgres`-gated module) and `cargo test --features postgres` without a
    //! database both stay green and store-free. When a store IS provided they demonstrate, against the
    //! durable backing:
    //!
    //!   * `confirmed_write_durable` + bearer-token resolution — `login` issues an opaque token that
    //!     `resolve_token` maps back to the issuing `UserId`, and still resolves across a fresh
    //!     reconnect (durable, pod-independent).
    //!   * the error contract — `login` is [`NotFound`] for an unknown handle, `resolve_token` is
    //!     [`Unauthorized`] for an unknown token — never a panic.
    //!   * `invalidate_session` is idempotent and, once applied, the token stops resolving.
    //!
    //! Handles are derived from the durable token counter so repeated runs never collide on the
    //! unique-handle index.
    //!
    //! [`NotFound`]: crate::error::ServerError::NotFound
    //! [`Unauthorized`]: crate::error::ServerError::Unauthorized

    use super::PostgresRepository;
    use crate::repository::ChirpRepository;
    use chirp_types::api::{CreateUserRequest, ErrorCode, LoginRequest, SessionToken};
    use chirp_types::domain::{User, Username};

    /// The test store URL, or `None` when unset/blank (→ the test skips).
    fn test_database_url() -> Option<String> {
        std::env::var("CHIRP_TEST_DATABASE_URL")
            .ok()
            .filter(|url| !url.trim().is_empty())
    }

    /// A rerun-safe, schema-valid handle derived from the durable `token_seq` counter, so repeated
    /// runs against the same store never collide on `users_username_key`.
    async fn unique_handle(repo: &PostgresRepository) -> Username {
        let counter = PostgresRepository::allocate_token_value(repo.pool())
            .await
            .expect("allocate a token value for a unique handle");
        Username::parse(format!("u{}", counter % 100_000_000_000_000))
            .expect("the derived handle is a valid Username")
    }

    /// Creates a durable user with a unique handle and returns the created profile.
    async fn seed_user(repo: &PostgresRepository) -> User {
        let handle = unique_handle(repo).await;
        repo.create_user(CreateUserRequest {
            username: handle.clone(),
            display_name: format!("{handle} display"),
            bio: None,
        })
        .await
        .expect("create_user succeeds")
    }

    #[tokio::test]
    async fn login_issues_a_durable_token_that_resolves_back_and_survives_reconnect() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url)
            .await
            .expect("connect + migrate the test store");

        let user = seed_user(&repo).await;
        let auth = repo
            .login(LoginRequest {
                username: user.username.clone(),
            })
            .await
            .expect("a known handle logs in");
        // The returned profile is the account that logged in.
        assert_eq!(auth.user.id, user.id);

        // confirmed_write_durable + resolution: the issued token maps back to the issuing UserId.
        let resolved = repo
            .resolve_token(&auth.token)
            .await
            .expect("the issued token resolves");
        assert_eq!(resolved, user.id);

        // Durable across restart: a FRESH connection to the same store still resolves the token.
        let reconnected = PostgresRepository::connect(&url)
            .await
            .expect("reconnect to the same store");
        let after_restart = reconnected
            .resolve_token(&auth.token)
            .await
            .expect("the session resolves after reconnect");
        assert_eq!(after_restart, user.id);
    }

    #[tokio::test]
    async fn login_is_not_found_for_an_unknown_handle() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();

        // A valid, schema-conformant handle that was never used to create a user.
        let unknown = unique_handle(&repo).await;
        let error = repo
            .login(LoginRequest { username: unknown })
            .await
            .expect_err("no such user");
        assert_eq!(error.code(), ErrorCode::NotFound);
    }

    #[tokio::test]
    async fn resolve_token_is_unauthorized_for_an_unknown_token() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();

        let error = repo
            .resolve_token(&SessionToken("tok-never-issued".to_owned()))
            .await
            .expect_err("an unknown token does not resolve");
        assert_eq!(error.code(), ErrorCode::Unauthorized);
    }

    #[tokio::test]
    async fn invalidate_session_is_idempotent_and_the_token_stops_resolving() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();

        let user = seed_user(&repo).await;
        let auth = repo
            .login(LoginRequest {
                username: user.username.clone(),
            })
            .await
            .expect("login issues a session");
        assert_eq!(repo.resolve_token(&auth.token).await.unwrap(), user.id);

        // Once invalidated, the token no longer resolves.
        repo.invalidate_session(&auth.token)
            .await
            .expect("invalidate the live session durably");
        assert_eq!(
            repo.resolve_token(&auth.token)
                .await
                .expect_err("an invalidated token stops resolving")
                .code(),
            ErrorCode::Unauthorized
        );

        // Invalidating again is an idempotent no-op (a zero-row delete still commits), never an error.
        repo.invalidate_session(&auth.token)
            .await
            .expect("invalidating an already-invalid token succeeds");
    }
}

#[cfg(test)]
mod account_recovery_tests {
    //! End-to-end tests for durable account recovery — `begin_account_recovery` /
    //! `complete_account_recovery` — against a REAL PostgreSQL backing, composing on the committed
    //! store-access substrate (`with_transaction`, `allocate_token_value`), the `mapping` pattern, the
    //! committed `recovery_tokens` table, and the credential store the prior stage established.
    //!
    //! Like the proving-slice, credential, and session tests they are **skipped** unless
    //! `CHIRP_TEST_DATABASE_URL` names a reachable test database, so the default `cargo test` (which
    //! does not compile this `postgres`-gated module) and `cargo test --features postgres` without a
    //! database both stay green and store-free. When a store IS provided they demonstrate, against the
    //! durable backing:
    //!
    //!   * `confirmed_write_durable` + credential reset — a challenge issued by `begin` is validated by
    //!     `complete`, which durably resets the credential; the new secret then `verify_credential`s
    //!     (including across a fresh reconnect), proving the reset is durable and pod-independent.
    //!   * single-use — a validated challenge is consumed, so replaying it is [`Unauthorized`].
    //!   * the fail-secure error contract — `begin` is [`NotFound`] for an unknown handle, and
    //!     `complete` is [`Unauthorized`] — indistinguishably — for an unknown handle, an unknown token,
    //!     a token issued for another account, or an expired token — never a panic.
    //!
    //! Handles are derived from the durable token counter so repeated runs never collide on the
    //! unique-handle index. The "expired" case injects a past-dated `recovery_tokens` row directly via
    //! SQL (the durable analogue of forcing a stored challenge into the past), exactly as the
    //! proving-slice test injects a corrupt row to exercise the read boundary.
    //!
    //! [`NotFound`]: crate::error::ServerError::NotFound
    //! [`Unauthorized`]: crate::error::ServerError::Unauthorized

    use super::{mapping, now_millis, PostgresRepository};
    use crate::repository::ChirpRepository;
    use chirp_types::api::{CreateUserRequest, ErrorCode};
    use chirp_types::domain::{User, Username};

    /// The test store URL, or `None` when unset/blank (→ the test skips).
    fn test_database_url() -> Option<String> {
        std::env::var("CHIRP_TEST_DATABASE_URL")
            .ok()
            .filter(|url| !url.trim().is_empty())
    }

    /// A rerun-safe, schema-valid handle derived from the durable `token_seq` counter, so repeated
    /// runs against the same store never collide on `users_username_key`.
    async fn unique_handle(repo: &PostgresRepository) -> Username {
        let counter = PostgresRepository::allocate_token_value(repo.pool())
            .await
            .expect("allocate a token value for a unique handle");
        Username::parse(format!("u{}", counter % 100_000_000_000_000))
            .expect("the derived handle is a valid Username")
    }

    /// Creates a durable user with a unique handle and returns the created profile.
    async fn seed_user(repo: &PostgresRepository) -> User {
        let handle = unique_handle(repo).await;
        repo.create_user(CreateUserRequest {
            username: handle.clone(),
            display_name: format!("{handle} display"),
            bio: None,
        })
        .await
        .expect("create_user succeeds")
    }

    /// Reads the recovery token `begin_account_recovery` durably issued for `user` straight from the
    /// `recovery_tokens` table — the durable analogue of a holder receiving the out-of-band challenge.
    async fn issued_recovery_token(repo: &PostgresRepository, user: &User) -> String {
        sqlx::query_scalar("SELECT token FROM recovery_tokens WHERE user_id = $1")
            .bind(mapping::user_id_to_column(user.id).unwrap())
            .fetch_one(repo.pool())
            .await
            .expect("a recovery challenge was durably issued for the user")
    }

    #[tokio::test]
    async fn begin_then_complete_durably_resets_the_credential_across_reconnect_and_is_single_use()
    {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url)
            .await
            .expect("connect + migrate the test store");

        let user = seed_user(&repo).await;
        repo.set_credential(user.id, "old-secret")
            .await
            .expect("seed an initial credential");

        // begin durably issues a challenge; complete validates it and resets the credential.
        repo.begin_account_recovery(&user.username)
            .await
            .expect("begin issues a durable challenge for a known handle");
        let token = issued_recovery_token(&repo, &user).await;
        repo.complete_account_recovery(&user.username, &token, "new-secret")
            .await
            .expect("a valid, unexpired challenge completes");

        // confirmed_write_durable: the recovery-set credential verifies, and the old one no longer
        // does. The reset survives a FRESH reconnect to the same store (durable, pod-independent).
        assert_eq!(
            repo.verify_credential(&user.username, "new-secret")
                .await
                .expect("the recovery-set secret verifies"),
            user.id
        );
        assert_eq!(
            repo.verify_credential(&user.username, "old-secret")
                .await
                .expect_err("the replaced secret is rejected")
                .code(),
            ErrorCode::Unauthorized
        );
        let reconnected = PostgresRepository::connect(&url)
            .await
            .expect("reconnect to the same store");
        assert_eq!(
            reconnected
                .verify_credential(&user.username, "new-secret")
                .await
                .expect("the recovery-set secret verifies after reconnect"),
            user.id
        );

        // Single-use: the validated challenge was consumed, so replaying it is Unauthorized.
        assert_eq!(
            repo.complete_account_recovery(&user.username, &token, "another")
                .await
                .expect_err("a consumed challenge cannot be replayed")
                .code(),
            ErrorCode::Unauthorized
        );
    }

    #[tokio::test]
    async fn begin_account_recovery_is_not_found_for_an_unknown_handle() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();

        // A valid, schema-conformant handle that was never used to create a user.
        let unknown = unique_handle(&repo).await;
        let error = repo
            .begin_account_recovery(&unknown)
            .await
            .expect_err("no such account");
        assert_eq!(error.code(), ErrorCode::NotFound);
    }

    #[tokio::test]
    async fn complete_account_recovery_is_unauthorized_for_unknown_mismatched_or_expired_token() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();

        let alice = seed_user(&repo).await;
        let bob = seed_user(&repo).await;

        // Unknown handle → Unauthorized (indistinguishable from an unknown token; no enumeration).
        let unknown = unique_handle(&repo).await;
        assert_eq!(
            repo.complete_account_recovery(&unknown, "rec-whatever", "s")
                .await
                .expect_err("unknown handle")
                .code(),
            ErrorCode::Unauthorized
        );

        // Known handle, unknown token.
        assert_eq!(
            repo.complete_account_recovery(&alice.username, "rec-never-issued", "s")
                .await
                .expect_err("unknown token")
                .code(),
            ErrorCode::Unauthorized
        );

        // A challenge issued for bob must not recover alice.
        repo.begin_account_recovery(&bob.username).await.unwrap();
        let bob_token = issued_recovery_token(&repo, &bob).await;
        assert_eq!(
            repo.complete_account_recovery(&alice.username, &bob_token, "s")
                .await
                .expect_err("a cross-account token is rejected")
                .code(),
            ErrorCode::Unauthorized
        );

        // Expired token: inject a past-dated row for alice directly, then complete → Unauthorized.
        let expired_token = format!("rec-expired-{}", alice.id.get());
        let past = mapping::timestamp_to_column(now_millis()) - 1;
        sqlx::query(
            "INSERT INTO recovery_tokens (token, user_id, created_at, expires_at) \
             VALUES ($1, $2, $3, $4)",
        )
        .bind(&expired_token)
        .bind(mapping::user_id_to_column(alice.id).unwrap())
        .bind(past)
        .bind(past)
        .execute(repo.pool())
        .await
        .expect("inject an expired challenge directly");
        assert_eq!(
            repo.complete_account_recovery(&alice.username, &expired_token, "s")
                .await
                .expect_err("an expired token is rejected")
                .code(),
            ErrorCode::Unauthorized
        );
    }
}

#[cfg(test)]
mod profile_update_tests {
    //! End-to-end tests for durable `update_profile` against a REAL PostgreSQL backing, composing on
    //! the committed store-access substrate (`with_transaction`) and the `mapping` boundary pattern.
    //!
    //! Like the proving-slice, credential, session, and recovery tests they are **skipped** unless
    //! `CHIRP_TEST_DATABASE_URL` names a reachable test database, so the default `cargo test` (which
    //! does not compile this `postgres`-gated module) and `cargo test --features postgres` without a
    //! database both stay green and store-free. When a store IS provided they demonstrate, against the
    //! durable backing:
    //!
    //!   * `confirmed_write_durable` + immutable identity — the mutable `display_name`/`bio` are
    //!     durably updated and the updated [`User`] returned, while the handle and id are never
    //!     rewritten (`REQ-DOM-008`); the change survives a FRESH reconnect to the same store (durable,
    //!     pod-independent), and the optional `bio` can be durably cleared to `NULL`.
    //!   * the error contract — updating an absent id is [`NotFound`], never a panic.
    //!
    //! Handles are derived from the durable token counter so repeated runs never collide on the
    //! unique-handle index.
    //!
    //! [`NotFound`]: crate::error::ServerError::NotFound

    use super::PostgresRepository;
    use crate::repository::ChirpRepository;
    use chirp_types::api::{CreateUserRequest, ErrorCode};
    use chirp_types::domain::{User, Username};
    use chirp_types::ids::UserId;

    /// The test store URL, or `None` when unset/blank (→ the test skips).
    fn test_database_url() -> Option<String> {
        std::env::var("CHIRP_TEST_DATABASE_URL")
            .ok()
            .filter(|url| !url.trim().is_empty())
    }

    /// A rerun-safe, schema-valid handle derived from the durable `token_seq` counter, so repeated
    /// runs against the same store never collide on `users_username_key`.
    async fn unique_handle(repo: &PostgresRepository) -> Username {
        let counter = PostgresRepository::allocate_token_value(repo.pool())
            .await
            .expect("allocate a token value for a unique handle");
        Username::parse(format!("u{}", counter % 100_000_000_000_000))
            .expect("the derived handle is a valid Username")
    }

    /// Creates a durable user with a unique handle and an initial `bio`, returning the created profile.
    async fn seed_user(repo: &PostgresRepository) -> User {
        let handle = unique_handle(repo).await;
        repo.create_user(CreateUserRequest {
            username: handle.clone(),
            display_name: format!("{handle} display"),
            bio: Some("initial bio".to_owned()),
        })
        .await
        .expect("create_user succeeds")
    }

    #[tokio::test]
    async fn update_profile_durably_changes_fields_preserves_identity_and_is_pod_independent() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url)
            .await
            .expect("connect + migrate the test store");

        let user = seed_user(&repo).await;

        let updated = repo
            .update_profile(
                user.id,
                "New Display".to_owned(),
                Some("new bio".to_owned()),
            )
            .await
            .expect("update succeeds for an existing user");

        // The mutable fields changed; the returned profile reflects the durable post-update state.
        assert_eq!(updated.display_name, "New Display");
        assert_eq!(updated.bio.as_deref(), Some("new bio"));
        // Immutable identity and non-profile fields are untouched (REQ-DOM-008).
        assert_eq!(updated.id, user.id);
        assert_eq!(updated.username, user.username);
        assert_eq!(updated.created_at, user.created_at);
        assert_eq!(updated.follower_count, user.follower_count);
        assert_eq!(updated.following_count, user.following_count);

        // confirmed_write_durable: a FRESH reconnect to the same store reads the updated profile
        // (durable, pod-independent) — and the handle and id are unchanged in the store.
        let reconnected = PostgresRepository::connect(&url)
            .await
            .expect("reconnect to the same store");
        let fetched = reconnected
            .get_user(user.id)
            .await
            .expect("the user is still present after update");
        assert_eq!(fetched, updated);
        assert_eq!(fetched.username, user.username);
    }

    #[tokio::test]
    async fn update_profile_can_durably_clear_the_optional_bio() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();

        let user = seed_user(&repo).await; // seeded with Some("initial bio")
        let updated = repo
            .update_profile(user.id, user.display_name.clone(), None)
            .await
            .expect("update succeeds");
        assert_eq!(updated.bio, None);

        // The cleared bio is durable (NULL in the store), read back across a fresh reconnect.
        let reconnected = PostgresRepository::connect(&url).await.unwrap();
        assert_eq!(
            reconnected.get_user(user.id).await.expect("exists").bio,
            None
        );
    }

    #[tokio::test]
    async fn update_profile_is_not_found_for_an_absent_user() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();

        // An id far above the monotonic `user_id_seq` (MINVALUE 1) is guaranteed absent.
        let error = repo
            .update_profile(UserId::new(9_000_000_000_000), "ghost".to_owned(), None)
            .await
            .expect_err("no such user");
        assert_eq!(error.code(), ErrorCode::NotFound);
    }
}

#[cfg(test)]
mod set_account_suspension_tests {
    //! End-to-end tests for durable `set_account_suspension` against a REAL PostgreSQL backing,
    //! composing on the committed store-access substrate
    //! ([`with_transaction`](super::PostgresRepository::with_transaction)) and the `mapping` boundary
    //! pattern, persisting into the committed `users.suspended` / `users.suspended_at` moderation
    //! columns.
    //!
    //! Like the proving-slice, credential, session, recovery, and profile tests they are **skipped**
    //! unless `CHIRP_TEST_DATABASE_URL` names a reachable test database, so the default `cargo test`
    //! (which does not compile this `postgres`-gated module) and `cargo test --features postgres`
    //! without a database both stay green and store-free. When a store IS provided they demonstrate,
    //! against the durable backing:
    //!
    //!   * `confirmed_write_durable` — a moderator durably sets the target's suspension (stamping
    //!     `suspended_at`) and durably clears it (resetting `suspended_at` to `NULL`), and each state
    //!     survives a FRESH reconnect to the same store (durable, pod-independent).
    //!   * the error contract — suspending an absent target is [`NotFound`], and a self-targeted
    //!     moderation (`actor == target`) is [`Forbidden`], never a panic.
    //!
    //! Handles are derived from the durable token counter so repeated runs never collide on the
    //! unique-handle index.
    //!
    //! [`NotFound`]: crate::error::ServerError::NotFound
    //! [`Forbidden`]: crate::error::ServerError::Forbidden

    use super::{mapping, PostgresRepository};
    use crate::repository::ChirpRepository;
    use chirp_types::api::{CreateUserRequest, ErrorCode};
    use chirp_types::domain::{User, Username};
    use chirp_types::ids::UserId;

    /// The test store URL, or `None` when unset/blank (→ the test skips).
    fn test_database_url() -> Option<String> {
        std::env::var("CHIRP_TEST_DATABASE_URL")
            .ok()
            .filter(|url| !url.trim().is_empty())
    }

    /// A rerun-safe, schema-valid handle derived from the durable `token_seq` counter, so repeated
    /// runs against the same store never collide on `users_username_key`.
    async fn unique_handle(repo: &PostgresRepository) -> Username {
        let counter = PostgresRepository::allocate_token_value(repo.pool())
            .await
            .expect("allocate a token value for a unique handle");
        Username::parse(format!("u{}", counter % 100_000_000_000_000))
            .expect("the derived handle is a valid Username")
    }

    /// Creates a durable user with a unique handle and returns the created profile.
    async fn seed_user(repo: &PostgresRepository) -> User {
        let handle = unique_handle(repo).await;
        repo.create_user(CreateUserRequest {
            username: handle.clone(),
            display_name: format!("{handle} display"),
            bio: None,
        })
        .await
        .expect("create_user succeeds")
    }

    /// Reads the committed `(suspended, suspended_at)` moderation columns straight from the `users`
    /// row — the durable analogue of observing the persisted suspension state.
    async fn stored_suspension(repo: &PostgresRepository, user: &User) -> (bool, Option<i64>) {
        sqlx::query_as("SELECT suspended, suspended_at FROM users WHERE id = $1")
            .bind(mapping::user_id_to_column(user.id).expect("user id lowers to a column"))
            .fetch_one(repo.pool())
            .await
            .expect("the user row exists")
    }

    #[tokio::test]
    async fn set_account_suspension_durably_sets_and_clears_across_reconnect() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url)
            .await
            .expect("connect + migrate the test store");

        let moderator = seed_user(&repo).await;
        let target = seed_user(&repo).await;

        // Accounts start unsuspended with no audit timestamp.
        let (suspended, suspended_at) = stored_suspension(&repo, &target).await;
        assert!(!suspended, "a fresh account is not suspended");
        assert_eq!(suspended_at, None);

        // Setting durably records the suspension and stamps suspended_at.
        repo.set_account_suspension(moderator.id, target.id, true)
            .await
            .expect("a moderator durably suspends another account");
        let (suspended, suspended_at) = stored_suspension(&repo, &target).await;
        assert!(suspended, "the target is suspended");
        assert!(suspended_at.is_some(), "setting stamps suspended_at");

        // confirmed_write_durable: a FRESH reconnect reads the suspension (durable, pod-independent).
        let reconnected = PostgresRepository::connect(&url)
            .await
            .expect("reconnect to the same store");
        assert!(
            stored_suspension(&reconnected, &target).await.0,
            "the suspension is durable across a reconnect"
        );

        // Clearing durably resets the flag and the audit timestamp to NULL.
        repo.set_account_suspension(moderator.id, target.id, false)
            .await
            .expect("a moderator durably clears the suspension");
        let (suspended, suspended_at) = stored_suspension(&repo, &target).await;
        assert!(!suspended, "the suspension is cleared");
        assert_eq!(suspended_at, None, "clearing resets suspended_at to NULL");

        let reconnected = PostgresRepository::connect(&url)
            .await
            .expect("reconnect to the same store");
        assert!(
            !stored_suspension(&reconnected, &target).await.0,
            "the cleared state is durable across a reconnect"
        );

        // The moderator's own account is never touched by moderating another.
        assert!(!stored_suspension(&repo, &moderator).await.0);
    }

    #[tokio::test]
    async fn set_account_suspension_is_not_found_for_an_absent_target() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();
        let moderator = seed_user(&repo).await;

        // An id far above the monotonic `user_id_seq` (MINVALUE 1) is guaranteed absent.
        let error = repo
            .set_account_suspension(moderator.id, UserId::new(9_000_000_000_000), true)
            .await
            .expect_err("no such target");
        assert_eq!(error.code(), ErrorCode::NotFound);
    }

    #[tokio::test]
    async fn set_account_suspension_is_forbidden_when_actor_targets_themselves() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();
        let alice = seed_user(&repo).await;

        let error = repo
            .set_account_suspension(alice.id, alice.id, true)
            .await
            .expect_err("a self-targeted moderation is refused");
        assert_eq!(error.code(), ErrorCode::Forbidden);

        // The refused action left no suspension behind.
        assert!(!stored_suspension(&repo, &alice).await.0);
    }
}

#[cfg(test)]
mod store_health_tests {
    //! End-to-end tests for durable `store_health` against a REAL PostgreSQL backing. Unlike the
    //! other durable methods `store_health` takes no domain input and owns no table — it reports
    //! store *reachability* (`REQ-DOM-021`) by round-tripping the live connection pool, so these
    //! tests exercise the backing store directly rather than through the `mapping`/`with_transaction`
    //! substrate.
    //!
    //! Like the proving-slice, credential, session, recovery, profile, and suspension tests they are
    //! **skipped** unless `CHIRP_TEST_DATABASE_URL` names a reachable test database, so the default
    //! `cargo test` (which does not compile this `postgres`-gated module) and `cargo test --features
    //! postgres` without a database both stay green and store-free. When a store IS provided they
    //! demonstrate, against the durable backing:
    //!
    //!   * reachable → `Ok(())` — a connected repository reports healthy, and still reports healthy
    //!     across a FRESH reconnect to the same store (the signal tracks live reachability, not a
    //!     one-shot constant).
    //!   * unreachable → [`Internal`] — once the live pool is closed the SAME method reports a failed
    //!     health check, never a false `Ok`. This is what makes the check a real round-trip of the
    //!     backing store rather than a hardcoded value.
    //!
    //! [`Internal`]: crate::error::ServerError::Internal

    use super::PostgresRepository;
    use crate::repository::ChirpRepository;
    use chirp_types::api::ErrorCode;

    /// The test store URL, or `None` when unset/blank (→ the test skips).
    fn test_database_url() -> Option<String> {
        std::env::var("CHIRP_TEST_DATABASE_URL")
            .ok()
            .filter(|url| !url.trim().is_empty())
    }

    #[tokio::test]
    async fn store_health_reports_ok_when_the_store_is_reachable() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url)
            .await
            .expect("connect + migrate the test store");

        // A reachable store reports healthy, and repeating the probe stays healthy (the check is a
        // repeatable round-trip of the live pool, not a one-shot).
        repo.store_health()
            .await
            .expect("a reachable store is healthy");
        repo.store_health()
            .await
            .expect("the health probe is repeatable while the store stays reachable");

        // Reachability is pod-independent: a FRESH connection to the same store is healthy too.
        let reconnected = PostgresRepository::connect(&url)
            .await
            .expect("reconnect to the same store");
        reconnected
            .store_health()
            .await
            .expect("the same store is healthy across a fresh connection");
    }

    #[tokio::test]
    async fn store_health_reports_internal_when_the_store_is_unreachable() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url)
            .await
            .expect("connect + migrate the test store");

        // Sanity: healthy while reachable.
        repo.store_health()
            .await
            .expect("a reachable store is healthy before we close the pool");

        // Make the backing store unreachable by closing the live pool, then probe the SAME method.
        // Because the check actually round-trips the pool (it is not a constant), it now fails.
        repo.pool().close().await;

        let error = repo
            .store_health()
            .await
            .expect_err("an unreachable store is reported unhealthy, never falsely Ok");
        assert_eq!(error.code(), ErrorCode::Internal);
    }
}

#[cfg(test)]
mod content_timeline_durability_tests {
    //! End-to-end tests for the durable content/timeline capability methods — `create_chirp`,
    //! `remove_chirp`, `timeline`, `export_user_data`, and `list_followers` / `list_following` —
    //! against a REAL PostgreSQL backing, composing on the committed store-access substrate and the
    //! domain schema from `SUBTASK_store_substrate` and the identity/auth stage.
    //!
    //! Like the other durable suites they are **skipped** unless `CHIRP_TEST_DATABASE_URL` names a
    //! reachable test database, so the default `cargo test` (which does not compile this
    //! `postgres`-gated module) and `cargo test --features postgres` without a database both stay
    //! green and store-free. When a store IS provided they demonstrate, against the durable backing:
    //!
    //!   * `confirmed_write_durable` — a posted chirp and a soft-removal are visible after returning
    //!     and across a fresh reconnect; the chirp's `like_count` is initialised to 0.
    //!   * `ids_opaque_monotonic_stable` — the timeline's opaque cursor paginates **exactly once**
    //!     (no skips, no duplicates) and the pages are stable across a reconnect.
    //!   * `domain_boundary_validated` error contract — `create_chirp` is [`NotFound`] for an absent
    //!     author or reply target; `timeline` / `export_user_data` / the follow listings are
    //!     [`NotFound`] for an absent user — never a panic.
    //!
    //! Handles are derived from the durable token counter so repeated runs never collide on the
    //! unique-handle index, and each test uses fresh users so their content is isolated on a shared
    //! store.
    //!
    //! [`NotFound`]: crate::error::ServerError::NotFound

    use super::PostgresRepository;
    use crate::repository::ChirpRepository;
    use chirp_types::api::{
        CreateChirpRequest, CreateUserRequest, Cursor, ErrorCode, TimelineQuery,
    };
    use chirp_types::domain::{ChirpText, User, Username};
    use chirp_types::ids::{ChirpId, UserId};

    /// The test store URL, or `None` when unset/blank (→ the test skips).
    fn test_database_url() -> Option<String> {
        std::env::var("CHIRP_TEST_DATABASE_URL")
            .ok()
            .filter(|url| !url.trim().is_empty())
    }

    /// A rerun-safe, schema-valid handle derived from the durable `token_seq` counter.
    async fn unique_handle(repo: &PostgresRepository) -> Username {
        let counter = PostgresRepository::allocate_token_value(repo.pool())
            .await
            .expect("allocate a token value for a unique handle");
        Username::parse(format!("u{}", counter % 100_000_000_000_000))
            .expect("the derived handle is a valid Username")
    }

    /// Creates a durable user with a unique handle and returns the created profile.
    async fn seed_user(repo: &PostgresRepository) -> User {
        let handle = unique_handle(repo).await;
        repo.create_user(CreateUserRequest {
            username: handle.clone(),
            display_name: format!("{handle} display"),
            bio: None,
        })
        .await
        .expect("create_user succeeds")
    }

    /// Posts a top-level chirp authored by `author` and returns it.
    async fn seed_chirp(
        repo: &PostgresRepository,
        author: UserId,
        body: &str,
    ) -> chirp_types::domain::Chirp {
        repo.create_chirp(
            author,
            CreateChirpRequest {
                text: ChirpText::parse(body).expect("valid body"),
                reply_to: None,
            },
        )
        .await
        .expect("create_chirp succeeds")
    }

    /// Inserts a `follows` edge directly, as test scaffolding.
    ///
    /// The durable `follow` write (with its atomic denormalized-count maintenance) is the NEXT
    /// serial stage's (`SUBTASK_graph_counts_atomic`); it is still a scaffold stub here. These
    /// content/timeline read methods only need the edge rows to exist, so the tests seed them
    /// directly against the committed `follows` schema (mirroring the proving slice's direct-SQL row
    /// injection). The denormalized counts are deliberately left unmaintained — the read methods
    /// under test do not depend on them, and `counts_match_edges` is the next stage's identity.
    async fn insert_follow(repo: &PostgresRepository, follower: UserId, followee: UserId) {
        sqlx::query(
            "INSERT INTO follows (follower_id, followee_id, created_at) VALUES ($1, $2, $3)",
        )
        .bind(i64::try_from(follower.get()).unwrap())
        .bind(i64::try_from(followee.get()).unwrap())
        .bind(0_i64)
        .execute(repo.pool())
        .await
        .expect("seed a follow edge directly");
    }

    /// Inserts a `likes` edge directly, as test scaffolding (the durable `like` write is the next
    /// stage's; see [`insert_follow`]).
    async fn insert_like(repo: &PostgresRepository, user: UserId, chirp: ChirpId) {
        sqlx::query("INSERT INTO likes (user_id, chirp_id, created_at) VALUES ($1, $2, $3)")
            .bind(i64::try_from(user.get()).unwrap())
            .bind(i64::try_from(chirp.get()).unwrap())
            .bind(0_i64)
            .execute(repo.pool())
            .await
            .expect("seed a like edge directly");
    }

    #[tokio::test]
    async fn create_chirp_is_durable_with_zeroed_like_count_and_round_trips_on_the_timeline() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url)
            .await
            .expect("connect + migrate the test store");

        let author = seed_user(&repo).await;
        let chirp = seed_chirp(&repo, author.id, "hello durable world").await;
        assert_eq!(chirp.author_id, author.id);
        assert_eq!(
            chirp.like_count, 0,
            "a fresh chirp starts with a zeroed like_count"
        );
        assert_eq!(chirp.reply_to, None);

        // confirmed_write_durable + round trip: the chirp shows on the author's own timeline, and
        // still does from a FRESH connection to the same store (stable across restart).
        let reconnected = PostgresRepository::connect(&url).await.unwrap();
        let page = reconnected
            .timeline(author.id, TimelineQuery::default())
            .await
            .expect("timeline after reconnect");
        assert_eq!(
            page.items.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec![chirp.id]
        );
    }

    #[tokio::test]
    async fn create_chirp_rejects_an_absent_author_or_reply_target_and_accepts_a_valid_reply() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();
        let author = seed_user(&repo).await;

        // Absent author: an id drawn from the counter but never used to create a user.
        let absent_author = PostgresRepository::allocate_user_id(repo.pool())
            .await
            .unwrap();
        let err = repo
            .create_chirp(
                absent_author,
                CreateChirpRequest {
                    text: ChirpText::parse("orphan").unwrap(),
                    reply_to: None,
                },
            )
            .await
            .expect_err("absent author");
        assert_eq!(err.code(), ErrorCode::NotFound);

        // Absent reply target.
        let err = repo
            .create_chirp(
                author.id,
                CreateChirpRequest {
                    text: ChirpText::parse("reply to nothing").unwrap(),
                    reply_to: Some(ChirpId::new(9_999_999)),
                },
            )
            .await
            .expect_err("absent reply target");
        assert_eq!(err.code(), ErrorCode::NotFound);

        // A valid reply to an existing chirp succeeds and records the parent.
        let parent = seed_chirp(&repo, author.id, "parent").await;
        let reply = repo
            .create_chirp(
                author.id,
                CreateChirpRequest {
                    text: ChirpText::parse("a real reply").unwrap(),
                    reply_to: Some(parent.id),
                },
            )
            .await
            .expect("valid reply");
        assert_eq!(reply.reply_to, Some(parent.id));
    }

    #[tokio::test]
    async fn timeline_is_reverse_chronological_scoped_to_self_plus_followees_and_excludes_removed()
    {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();
        let alice = seed_user(&repo).await;
        let bob = seed_user(&repo).await;
        let carol = seed_user(&repo).await;

        let a1 = seed_chirp(&repo, alice.id, "alice one").await;
        let _b1 = seed_chirp(&repo, bob.id, "bob one (unfollowed)").await;
        let a2 = seed_chirp(&repo, alice.id, "alice two").await;
        insert_follow(&repo, alice.id, carol.id).await;
        let c1 = seed_chirp(&repo, carol.id, "carol one (followed)").await;
        let doomed = seed_chirp(&repo, alice.id, "to be removed").await;

        // Remove one of alice's own chirps; it must drop out of the served timeline.
        repo.remove_chirp(alice.id, doomed.id).await.unwrap();

        let page = repo
            .timeline(alice.id, TimelineQuery::default())
            .await
            .expect("timeline");
        // Newest-first: carol's followed chirp, alice two, alice one. Bob (unfollowed) and the
        // removed chirp are excluded.
        assert_eq!(
            page.items.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec![c1.id, a2.id, a1.id]
        );
        assert!(page.next_cursor.is_none());
    }

    #[tokio::test]
    async fn timeline_paginates_exactly_once_and_is_stable_across_restart() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();
        let alice = seed_user(&repo).await;

        let mut posted = Vec::new();
        for i in 0..5 {
            posted.push(seed_chirp(&repo, alice.id, &format!("chirp {i}")).await);
        }
        let mut expected: Vec<ChirpId> = posted.iter().map(|c| c.id).collect();
        expected.sort_by_key(|id| std::cmp::Reverse(id.get())); // newest-first

        // Page through two at a time; the union must be every chirp EXACTLY ONCE, in order.
        let mut seen = Vec::new();
        let mut cursor: Option<Cursor> = None;
        loop {
            let page = repo
                .timeline(
                    alice.id,
                    TimelineQuery {
                        limit: Some(2),
                        cursor: cursor.clone(),
                    },
                )
                .await
                .unwrap();
            seen.extend(page.items.iter().map(|c| c.id));
            match page.next_cursor {
                Some(next) => cursor = Some(next),
                None => break,
            }
        }
        assert_eq!(
            seen, expected,
            "every chirp appears exactly once, newest-first"
        );

        // Stable across restart: a fresh connection returns the same first page.
        let reconnected = PostgresRepository::connect(&url).await.unwrap();
        let first_again = reconnected
            .timeline(
                alice.id,
                TimelineQuery {
                    limit: Some(2),
                    cursor: None,
                },
            )
            .await
            .unwrap();
        assert_eq!(
            first_again.items.iter().map(|c| c.id).collect::<Vec<_>>(),
            expected[..2].to_vec()
        );
    }

    #[tokio::test]
    async fn remove_chirp_is_durable_idempotent_and_not_found_for_an_absent_chirp() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();
        let alice = seed_user(&repo).await;
        let chirp = seed_chirp(&repo, alice.id, "removable").await;

        repo.remove_chirp(alice.id, chirp.id).await.expect("remove");
        // Durable across restart: a fresh connection no longer serves the removed chirp.
        let reconnected = PostgresRepository::connect(&url).await.unwrap();
        let page = reconnected
            .timeline(alice.id, TimelineQuery::default())
            .await
            .unwrap();
        assert!(
            !page.items.iter().any(|c| c.id == chirp.id),
            "a soft-removed chirp is not served after reconnect"
        );

        // Idempotent re-removal.
        repo.remove_chirp(alice.id, chirp.id)
            .await
            .expect("re-removal is an idempotent no-op");

        // Absent chirp is NotFound.
        let err = repo
            .remove_chirp(alice.id, ChirpId::new(9_999_999))
            .await
            .expect_err("no such chirp");
        assert_eq!(err.code(), ErrorCode::NotFound);
    }

    #[tokio::test]
    async fn export_user_data_captures_a_consistent_view_and_is_not_found_for_an_absent_user() {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();
        let alice = seed_user(&repo).await;
        let bob = seed_user(&repo).await;

        let c1 = seed_chirp(&repo, alice.id, "alice one").await;
        let c2 = seed_chirp(&repo, alice.id, "alice two").await;
        let removed = seed_chirp(&repo, alice.id, "removed").await;
        repo.remove_chirp(alice.id, removed.id).await.unwrap();
        insert_follow(&repo, alice.id, bob.id).await;
        let bob_chirp = seed_chirp(&repo, bob.id, "bob chirp").await;
        insert_like(&repo, alice.id, bob_chirp.id).await;

        let json = repo.export_user_data(alice.id).await.expect("export");
        let doc: crate::repository::export::ExportDocument =
            serde_json::from_str(&json).expect("the export is valid JSON of the export shape");

        assert_eq!(doc.profile.id, alice.id);
        // Authored, non-removed chirps, newest-first; the removed chirp is excluded.
        assert_eq!(
            doc.chirps.iter().map(|c| c.id).collect::<Vec<_>>(),
            vec![c2.id, c1.id]
        );
        assert_eq!(doc.following, vec![bob.id]);
        assert_eq!(doc.likes, vec![bob_chirp.id]);

        // Absent user → NotFound.
        let absent = PostgresRepository::allocate_user_id(repo.pool())
            .await
            .unwrap();
        let err = repo
            .export_user_data(absent)
            .await
            .expect_err("no such user");
        assert_eq!(err.code(), ErrorCode::NotFound);
    }

    #[tokio::test]
    async fn follow_listings_return_the_right_accounts_page_exactly_once_and_are_not_found_for_absent(
    ) {
        let Some(url) = test_database_url() else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let repo = PostgresRepository::connect(&url).await.unwrap();
        let alice = seed_user(&repo).await;

        // Five accounts follow alice; alice follows exactly one of them.
        let mut followers = Vec::new();
        for _ in 0..5 {
            let u = seed_user(&repo).await;
            insert_follow(&repo, u.id, alice.id).await;
            followers.push(u);
        }
        insert_follow(&repo, alice.id, followers[0].id).await;

        // list_followers pages through exactly once, newest-account-first (id DESC).
        let mut expected: Vec<UserId> = followers.iter().map(|u| u.id).collect();
        expected.sort_by_key(|id| std::cmp::Reverse(id.get()));
        let mut seen = Vec::new();
        let mut cursor: Option<Cursor> = None;
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
        assert_eq!(
            seen, expected,
            "every follower appears exactly once, newest-first"
        );

        // list_following returns exactly the one account alice follows.
        let following = repo
            .list_following(alice.id, TimelineQuery::default())
            .await
            .unwrap();
        assert_eq!(
            following.items.iter().map(|u| u.id).collect::<Vec<_>>(),
            vec![followers[0].id]
        );

        // Absent user → NotFound on both listings.
        let absent = PostgresRepository::allocate_user_id(repo.pool())
            .await
            .unwrap();
        assert_eq!(
            repo.list_followers(absent, TimelineQuery::default())
                .await
                .expect_err("no such user")
                .code(),
            ErrorCode::NotFound
        );
        assert_eq!(
            repo.list_following(absent, TimelineQuery::default())
                .await
                .expect_err("no such user")
                .code(),
            ErrorCode::NotFound
        );
    }
}

#[cfg(test)]
mod edge_toggle_tests {
    //! Durable-behaviour tests for the social-graph edge toggles `follow`/`unfollow`/`like`/`unlike`
    //! against a REAL PostgreSQL backing, proving this stage's carried identities on the toggle paths:
    //!
    //!   * `counts_match_edges` — after every toggle the denormalized `follower_count` /
    //!     `following_count` / `like_count` equals the COUNT of the current `follows` / `likes` edge
    //!     set, read directly from the store.
    //!   * `multiwrite_atomic` / `confirmed_write_durable` — the edge mutation and its count move as one
    //!     committed unit, visible (and still consistent) via a fresh read after the call returns.
    //!   * idempotency — a repeated follow/like does not double-count; a repeat unfollow/unlike on an
    //!     absent edge is a well-defined no-op; the count reflects set membership, not call count.
    //!   * entity invariants at the durable boundary — no self-follow (Forbidden); like/unlike require
    //!     an existing chirp; an absent user/chirp is NotFound — all via the `ServerError` contract,
    //!     never a panic.
    //!
    //! They are **skipped** unless `CHIRP_TEST_DATABASE_URL` names a reachable test database, so the
    //! default `cargo test` (which does not compile this `postgres`-gated module) and
    //! `cargo test --features postgres` without a database both stay green and store-free. Handles are
    //! derived from the durable `token_seq`, so repeated runs against the same store never collide.

    use super::{mapping, PostgresRepository};
    use crate::repository::ChirpRepository;
    use chirp_types::api::{CreateChirpRequest, CreateUserRequest, ErrorCode};
    use chirp_types::domain::{ChirpText, Username};
    use chirp_types::ids::{ChirpId, UserId};

    fn test_database_url() -> Option<String> {
        std::env::var("CHIRP_TEST_DATABASE_URL")
            .ok()
            .filter(|url| !url.trim().is_empty())
    }

    /// Connect to the test store and migrate it, or `None` to signal a skip.
    async fn connect() -> Option<PostgresRepository> {
        let url = test_database_url()?;
        Some(
            PostgresRepository::connect(&url)
                .await
                .expect("connect + migrate the test store"),
        )
    }

    /// A rerun-safe, schema-valid handle derived from the durable `token_seq`.
    async fn unique_handle(repo: &PostgresRepository) -> Username {
        let counter = PostgresRepository::allocate_token_value(repo.pool())
            .await
            .expect("allocate a token value for a unique handle");
        Username::parse(format!("u{}", counter % 100_000_000_000_000))
            .expect("the derived handle is a valid Username")
    }

    /// Create a fresh account and return its id.
    async fn new_user(repo: &PostgresRepository) -> UserId {
        let handle = unique_handle(repo).await;
        repo.create_user(CreateUserRequest {
            username: handle.clone(),
            display_name: format!("{handle} display"),
            bio: None,
        })
        .await
        .expect("create_user succeeds")
        .id
    }

    /// Create a fresh chirp authored by `author` and return its id.
    async fn new_chirp(repo: &PostgresRepository, author: UserId) -> ChirpId {
        repo.create_chirp(
            author,
            CreateChirpRequest {
                text: ChirpText::parse("a chirp to be liked").expect("valid chirp text"),
                reply_to: None,
            },
        )
        .await
        .expect("create_chirp succeeds")
        .id
    }

    /// The COUNT of the current `follows` edges whose followee is `user` — the authoritative edge set
    /// the denormalized `follower_count` must equal.
    async fn edge_follower_count(repo: &PostgresRepository, user: UserId) -> u64 {
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM follows WHERE followee_id = $1")
            .bind(mapping::user_id_to_column(user).unwrap())
            .fetch_one(repo.pool())
            .await
            .expect("count follower edges");
        u64::try_from(n).unwrap()
    }

    /// The COUNT of the current `follows` edges whose follower is `user`.
    async fn edge_following_count(repo: &PostgresRepository, user: UserId) -> u64 {
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM follows WHERE follower_id = $1")
            .bind(mapping::user_id_to_column(user).unwrap())
            .fetch_one(repo.pool())
            .await
            .expect("count following edges");
        u64::try_from(n).unwrap()
    }

    /// The COUNT of the current `likes` edges on `chirp`.
    async fn edge_like_count(repo: &PostgresRepository, chirp: ChirpId) -> u64 {
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM likes WHERE chirp_id = $1")
            .bind(mapping::chirp_id_to_column(chirp).unwrap())
            .fetch_one(repo.pool())
            .await
            .expect("count like edges");
        u64::try_from(n).unwrap()
    }

    /// Assert `counts_match_edges` for a user: both denormalized follow counts equal their edge sets,
    /// read with a FRESH connection so the assertion also proves `confirmed_write_durable`.
    async fn assert_user_counts_match_edges(url: &str, repo: &PostgresRepository, user: UserId) {
        let fresh = PostgresRepository::connect(url).await.expect("reconnect");
        let profile = fresh.get_user(user).await.expect("get_user");
        assert_eq!(
            profile.follower_count,
            edge_follower_count(repo, user).await,
            "follower_count must equal the follows edge set"
        );
        assert_eq!(
            profile.following_count,
            edge_following_count(repo, user).await,
            "following_count must equal the follows edge set"
        );
    }

    #[tokio::test]
    async fn follow_sets_the_edge_updates_counts_atomically_and_is_idempotent() {
        let Some(repo) = connect().await else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let url = test_database_url().unwrap();
        let (a, b) = (new_user(&repo).await, new_user(&repo).await);

        // First follow sets the edge and moves BOTH counts in one committed unit.
        let response = repo.follow(a, b).await.expect("follow succeeds");
        assert_eq!(response.followee_id, b);
        assert!(response.following);
        assert_eq!(response.follower_count, 1);
        assert_eq!(repo.get_user(a).await.unwrap().following_count, 1);
        assert_user_counts_match_edges(&url, &repo, a).await;
        assert_user_counts_match_edges(&url, &repo, b).await;

        // Idempotent: a repeat follow does not double-count and does not error.
        let again = repo.follow(a, b).await.expect("repeat follow is a no-op");
        assert!(again.following);
        assert_eq!(again.follower_count, 1);
        assert_eq!(repo.get_user(a).await.unwrap().following_count, 1);
        assert_user_counts_match_edges(&url, &repo, b).await;
    }

    #[tokio::test]
    async fn unfollow_removes_the_edge_updates_counts_and_is_idempotent() {
        let Some(repo) = connect().await else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let url = test_database_url().unwrap();
        let (a, b) = (new_user(&repo).await, new_user(&repo).await);
        repo.follow(a, b).await.expect("follow succeeds");

        // Unfollow removes the edge and moves both counts back to zero, atomically.
        let response = repo.unfollow(a, b).await.expect("unfollow succeeds");
        assert!(!response.following);
        assert_eq!(response.follower_count, 0);
        assert_eq!(repo.get_user(a).await.unwrap().following_count, 0);
        assert_user_counts_match_edges(&url, &repo, a).await;
        assert_user_counts_match_edges(&url, &repo, b).await;

        // Idempotent: unfollowing an absent edge is a well-defined no-op, counts unchanged.
        let again = repo
            .unfollow(a, b)
            .await
            .expect("repeat unfollow is a no-op");
        assert!(!again.following);
        assert_eq!(again.follower_count, 0);
        assert_user_counts_match_edges(&url, &repo, b).await;
    }

    #[tokio::test]
    async fn follow_forbids_self_follow_and_is_not_found_for_absent_users() {
        let Some(repo) = connect().await else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let a = new_user(&repo).await;

        // No self-follow, reported via the ServerError contract (never a panic, never an Internal).
        assert_eq!(
            repo.follow(a, a).await.expect_err("self-follow").code(),
            ErrorCode::Forbidden
        );

        // An absent endpoint is NotFound in either position, decided inside the transaction.
        let absent = PostgresRepository::allocate_user_id(repo.pool())
            .await
            .unwrap();
        assert_eq!(
            repo.follow(a, absent)
                .await
                .expect_err("absent followee")
                .code(),
            ErrorCode::NotFound
        );
        assert_eq!(
            repo.follow(absent, a)
                .await
                .expect_err("absent follower")
                .code(),
            ErrorCode::NotFound
        );
        assert_eq!(
            repo.unfollow(a, absent)
                .await
                .expect_err("absent followee")
                .code(),
            ErrorCode::NotFound
        );
    }

    #[tokio::test]
    async fn like_sets_the_edge_updates_count_atomically_and_is_idempotent() {
        let Some(repo) = connect().await else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let author = new_user(&repo).await;
        let liker = new_user(&repo).await;
        let chirp = new_chirp(&repo, author).await;

        let response = repo.like(liker, chirp).await.expect("like succeeds");
        assert_eq!(response.chirp_id, chirp);
        assert!(response.liked);
        assert_eq!(response.like_count, 1);
        assert_eq!(edge_like_count(&repo, chirp).await, 1);

        // Idempotent: a repeat like does not double-count.
        let again = repo
            .like(liker, chirp)
            .await
            .expect("repeat like is a no-op");
        assert!(again.liked);
        assert_eq!(again.like_count, 1);
        assert_eq!(edge_like_count(&repo, chirp).await, 1);
    }

    #[tokio::test]
    async fn unlike_removes_the_edge_updates_count_and_is_idempotent() {
        let Some(repo) = connect().await else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let author = new_user(&repo).await;
        let liker = new_user(&repo).await;
        let chirp = new_chirp(&repo, author).await;
        repo.like(liker, chirp).await.expect("like succeeds");

        let response = repo.unlike(liker, chirp).await.expect("unlike succeeds");
        assert!(!response.liked);
        assert_eq!(response.like_count, 0);
        assert_eq!(edge_like_count(&repo, chirp).await, 0);

        // Idempotent: unliking a chirp the user has not liked is a no-op.
        let again = repo
            .unlike(liker, chirp)
            .await
            .expect("repeat unlike is a no-op");
        assert!(!again.liked);
        assert_eq!(again.like_count, 0);
    }

    #[tokio::test]
    async fn like_and_unlike_require_an_existing_chirp_and_user() {
        let Some(repo) = connect().await else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let user = new_user(&repo).await;
        let author = new_user(&repo).await;
        let chirp = new_chirp(&repo, author).await;

        let absent_chirp = PostgresRepository::allocate_chirp_id(repo.pool())
            .await
            .unwrap();
        let absent_user = PostgresRepository::allocate_user_id(repo.pool())
            .await
            .unwrap();

        assert_eq!(
            repo.like(user, absent_chirp)
                .await
                .expect_err("absent chirp")
                .code(),
            ErrorCode::NotFound
        );
        assert_eq!(
            repo.like(absent_user, chirp)
                .await
                .expect_err("absent user")
                .code(),
            ErrorCode::NotFound
        );
        assert_eq!(
            repo.unlike(user, absent_chirp)
                .await
                .expect_err("absent chirp")
                .code(),
            ErrorCode::NotFound
        );
    }
}

#[cfg(test)]
mod deletion_cascade_tests {
    //! Durable-behaviour tests for `delete_account` (the `account_deletion_cascade` capability)
    //! against a REAL PostgreSQL backing, proving this stage's carried identities on the cascade
    //! path:
    //!
    //!   * `multiwrite_atomic` / `confirmed_write_durable` — deleting the account removes ALL its
    //!     dependent families (authored chirps and their likes, follow edges in both directions,
    //!     like edges by the user, sessions, credential, recovery tokens, export artefacts) as ONE
    //!     committed unit, visible via fresh reads after the call returns.
    //!   * `counts_match_edges_under_cascade` — after the cascade every SURVIVING neighbour's
    //!     denormalized `follower_count` / `following_count` / `like_count` still equals its current
    //!     edge set; no stale count is left behind a deleted edge, and an edge not touching the
    //!     deleted account is untouched.
    //!   * the error contract — deleting an absent or already-deleted account is a clean
    //!     [`NotFound`](crate::error::ServerError::NotFound), never a panic and never a silent no-op.
    //!
    //! They are **skipped** unless `CHIRP_TEST_DATABASE_URL` names a reachable test database (the same
    //! harness as the edge-toggle tests), so the default store-free `cargo test` (which does not even
    //! compile this `postgres`-gated module) stays green. Handles are derived from the durable
    //! `token_seq`, so repeated runs against the same store never collide.

    use super::{mapping, PostgresRepository};
    use crate::repository::ChirpRepository;
    use chirp_types::api::{CreateChirpRequest, CreateUserRequest, ErrorCode, LoginRequest};
    use chirp_types::domain::{ChirpText, Username};
    use chirp_types::ids::{ChirpId, UserId};

    fn test_database_url() -> Option<String> {
        std::env::var("CHIRP_TEST_DATABASE_URL")
            .ok()
            .filter(|url| !url.trim().is_empty())
    }

    /// Connect to the test store and migrate it, or `None` to signal a skip.
    async fn connect() -> Option<PostgresRepository> {
        let url = test_database_url()?;
        Some(
            PostgresRepository::connect(&url)
                .await
                .expect("connect + migrate the test store"),
        )
    }

    /// A rerun-safe, schema-valid handle derived from the durable `token_seq`.
    async fn unique_handle(repo: &PostgresRepository) -> Username {
        let counter = PostgresRepository::allocate_token_value(repo.pool())
            .await
            .expect("allocate a token value for a unique handle");
        Username::parse(format!("u{}", counter % 100_000_000_000_000))
            .expect("the derived handle is a valid Username")
    }

    /// Create a fresh account and return its id together with its handle (the handle is needed to
    /// drive the real `login` / `begin_account_recovery` paths that seed the transient families).
    async fn new_user(repo: &PostgresRepository) -> (UserId, Username) {
        let handle = unique_handle(repo).await;
        let id = repo
            .create_user(CreateUserRequest {
                username: handle.clone(),
                display_name: format!("{handle} display"),
                bio: None,
            })
            .await
            .expect("create_user succeeds")
            .id;
        (id, handle)
    }

    /// Create a fresh top-level chirp authored by `author` and return its id.
    async fn new_chirp(repo: &PostgresRepository, author: UserId) -> ChirpId {
        repo.create_chirp(
            author,
            CreateChirpRequest {
                text: ChirpText::parse("a chirp").expect("valid chirp text"),
                reply_to: None,
            },
        )
        .await
        .expect("create_chirp succeeds")
        .id
    }

    /// `COUNT(*)` of a one-bind (user-keyed) query — the "no rows remain" cascade probe, read with a
    /// fresh pool connection so a surviving row would be observed (`confirmed_write_durable`).
    async fn count_by_user(repo: &PostgresRepository, sql: &str, user: UserId) -> i64 {
        sqlx::query_scalar(sql)
            .bind(mapping::user_id_to_column(user).unwrap())
            .fetch_one(repo.pool())
            .await
            .expect("count query")
    }

    #[tokio::test]
    async fn delete_account_cascades_all_families_and_reconciles_survivor_counts_atomically() {
        let Some(repo) = connect().await else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };

        let (alice, alice_handle) = new_user(&repo).await;
        let (bob, _bob_handle) = new_user(&repo).await;
        let (carol, _carol_handle) = new_user(&repo).await;

        // Follow graph: alice→bob and carol→alice both touch alice; bob→carol is a control edge that
        // must survive untouched.
        repo.follow(alice, bob).await.unwrap();
        repo.follow(carol, alice).await.unwrap();
        repo.follow(bob, carol).await.unwrap();

        // Content: bob's chirp survives; alice's is cascaded away.
        let b1 = new_chirp(&repo, bob).await;
        let a1 = new_chirp(&repo, alice).await;

        // Likes: alice likes the surviving b1 (its count must reconcile to 0) and her own a1
        // (cascaded away with it); bob likes a1 (that like edge is cascaded with a1).
        repo.like(alice, b1).await.unwrap();
        repo.like(alice, a1).await.unwrap();
        repo.like(bob, a1).await.unwrap();

        // A surviving reply to alice's chirp — its reply_to must become NULL when a1 is removed.
        let reply = repo
            .create_chirp(
                carol,
                CreateChirpRequest {
                    text: ChirpText::parse("reply to alice").unwrap(),
                    reply_to: Some(a1),
                },
            )
            .await
            .unwrap()
            .id;

        // Seed the remaining per-account dependent families through the real durable paths: a
        // credential, a session, and a recovery challenge.
        repo.set_credential(alice, "secret").await.unwrap();
        let session = repo
            .login(LoginRequest {
                username: alice_handle.clone(),
            })
            .await
            .unwrap()
            .token;
        repo.begin_account_recovery(&alice_handle).await.unwrap();

        // A materialised export artefact for alice (no trait method persists one, so insert the row
        // directly — the point is to prove the `export_artifacts` ON DELETE CASCADE fires).
        let export_id = i64::try_from(
            PostgresRepository::allocate_token_value(repo.pool())
                .await
                .unwrap(),
        )
        .unwrap();
        sqlx::query(
            "INSERT INTO export_artifacts (id, user_id, content, created_at, expires_at) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(export_id)
        .bind(mapping::user_id_to_column(alice).unwrap())
        .bind("alice export document")
        .bind(0_i64)
        .bind(i64::MAX)
        .execute(repo.pool())
        .await
        .expect("seed an export artefact row");

        // --- Delete alice: the whole cascade commits (or rolls back) as one unit. ---
        repo.delete_account(alice)
            .await
            .expect("delete_account succeeds");

        // The account itself is gone.
        assert_eq!(
            repo.get_user(alice)
                .await
                .expect_err("alice deleted")
                .code(),
            ErrorCode::NotFound
        );

        // Surviving neighbours' denormalized counts are reconciled to match their current edge sets.
        let bob_after = repo.get_user(bob).await.unwrap();
        let carol_after = repo.get_user(carol).await.unwrap();
        assert_eq!(
            bob_after.follower_count, 0,
            "alice's follow of bob is gone, so bob's follower_count reconciles to 0"
        );
        assert_eq!(
            carol_after.following_count, 0,
            "carol's follow of alice is gone, so carol's following_count reconciles to 0"
        );
        // The control edge bob→carol is untouched.
        assert_eq!(bob_after.following_count, 1);
        assert_eq!(carol_after.follower_count, 1);
        let control_edge: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM follows WHERE follower_id = $1 AND followee_id = $2",
        )
        .bind(mapping::user_id_to_column(bob).unwrap())
        .bind(mapping::user_id_to_column(carol).unwrap())
        .fetch_one(repo.pool())
        .await
        .unwrap();
        assert_eq!(
            control_edge, 1,
            "the bob→carol edge survives alice's deletion"
        );

        // b1 survives; its like_count reconciles to 0 (alice's like removed, in the same txn).
        let b1_like_count: i64 = sqlx::query_scalar("SELECT like_count FROM chirps WHERE id = $1")
            .bind(mapping::chirp_id_to_column(b1).unwrap())
            .fetch_one(repo.pool())
            .await
            .unwrap();
        assert_eq!(
            b1_like_count, 0,
            "alice's like of the surviving chirp is reconciled away"
        );

        // Every dependent family alice owned or touched is gone.
        assert_eq!(
            count_by_user(
                &repo,
                "SELECT COUNT(*) FROM chirps WHERE author_id = $1",
                alice
            )
            .await,
            0,
            "authored chirps are cascaded"
        );
        assert_eq!(
            count_by_user(
                &repo,
                "SELECT COUNT(*) FROM follows WHERE follower_id = $1 OR followee_id = $1",
                alice,
            )
            .await,
            0,
            "follow edges on both sides are cascaded"
        );
        assert_eq!(
            count_by_user(
                &repo,
                "SELECT COUNT(*) FROM likes WHERE user_id = $1",
                alice
            )
            .await,
            0,
            "the deleted user's like edges are cascaded"
        );
        let likes_on_a1: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM likes WHERE chirp_id = $1")
            .bind(mapping::chirp_id_to_column(a1).unwrap())
            .fetch_one(repo.pool())
            .await
            .unwrap();
        assert_eq!(likes_on_a1, 0, "likes on the removed chirp are cascaded");
        assert_eq!(
            count_by_user(
                &repo,
                "SELECT COUNT(*) FROM sessions WHERE user_id = $1",
                alice
            )
            .await,
            0,
            "sessions are cascaded"
        );
        assert_eq!(
            count_by_user(
                &repo,
                "SELECT COUNT(*) FROM credentials WHERE user_id = $1",
                alice
            )
            .await,
            0,
            "the credential is cascaded"
        );
        assert_eq!(
            count_by_user(
                &repo,
                "SELECT COUNT(*) FROM recovery_tokens WHERE user_id = $1",
                alice,
            )
            .await,
            0,
            "recovery tokens are cascaded"
        );
        assert_eq!(
            count_by_user(
                &repo,
                "SELECT COUNT(*) FROM export_artifacts WHERE user_id = $1",
                alice,
            )
            .await,
            0,
            "export artefacts are cascaded"
        );

        // The surviving reply lost its target (ON DELETE SET NULL).
        let reply_to: Option<i64> = sqlx::query_scalar("SELECT reply_to FROM chirps WHERE id = $1")
            .bind(mapping::chirp_id_to_column(reply).unwrap())
            .fetch_one(repo.pool())
            .await
            .unwrap();
        assert_eq!(reply_to, None, "a reply to the removed chirp is SET NULL");

        // The deleted account's session no longer resolves.
        assert_eq!(
            repo.resolve_token(&session)
                .await
                .expect_err("session invalidated by deletion")
                .code(),
            ErrorCode::Unauthorized
        );
    }

    #[tokio::test]
    async fn delete_account_is_not_found_for_an_absent_or_already_deleted_account() {
        let Some(repo) = connect().await else {
            eprintln!("skipping: CHIRP_TEST_DATABASE_URL not set");
            return;
        };
        let (alice, _handle) = new_user(&repo).await;

        // An id that was never used is NotFound (never a panic).
        let absent = PostgresRepository::allocate_user_id(repo.pool())
            .await
            .unwrap();
        assert_eq!(
            repo.delete_account(absent)
                .await
                .expect_err("absent account")
                .code(),
            ErrorCode::NotFound
        );

        // The first deletion succeeds; a second on the now-deleted account is NotFound — never a
        // silent no-op.
        repo.delete_account(alice).await.expect("first delete");
        assert_eq!(
            repo.delete_account(alice)
                .await
                .expect_err("already deleted")
                .code(),
            ErrorCode::NotFound
        );
    }
}
