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
    AuthResponse, CreateChirpRequest, CreateUserRequest, FollowResponse, LikeResponse,
    LoginRequest, Page, SessionToken, TimelineQuery,
};
use chirp_types::domain::{Chirp, User, Username};
use chirp_types::ids::{ChirpId, Timestamp, UserId};

use crate::error::ServerError;
use crate::repository::ChirpRepository;

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

/// The shared message appended to every not-yet-implemented durable operation, naming the
/// downstream owner so a caller (or log) sees exactly why the operation failed.
const NOT_IMPLEMENTED: &str =
    "durable PostgreSQL backend not yet implemented (EPIC-PERSIST SUBTASK_durable_impl)";

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

    async fn create_chirp(
        &self,
        _author: UserId,
        _request: CreateChirpRequest,
    ) -> Result<Chirp, ServerError> {
        Err(ServerError::Internal(format!(
            "create_chirp: {NOT_IMPLEMENTED}"
        )))
    }

    async fn timeline(
        &self,
        _viewer: UserId,
        _query: TimelineQuery,
    ) -> Result<Page<Chirp>, ServerError> {
        Err(ServerError::Internal(format!(
            "timeline: {NOT_IMPLEMENTED}"
        )))
    }

    async fn follow(
        &self,
        _follower: UserId,
        _followee: UserId,
    ) -> Result<FollowResponse, ServerError> {
        Err(ServerError::Internal(format!("follow: {NOT_IMPLEMENTED}")))
    }

    async fn unfollow(
        &self,
        _follower: UserId,
        _followee: UserId,
    ) -> Result<FollowResponse, ServerError> {
        Err(ServerError::Internal(format!(
            "unfollow: {NOT_IMPLEMENTED}"
        )))
    }

    async fn like(&self, _user: UserId, _chirp: ChirpId) -> Result<LikeResponse, ServerError> {
        Err(ServerError::Internal(format!("like: {NOT_IMPLEMENTED}")))
    }

    async fn unlike(&self, _user: UserId, _chirp: ChirpId) -> Result<LikeResponse, ServerError> {
        Err(ServerError::Internal(format!("unlike: {NOT_IMPLEMENTED}")))
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

    async fn delete_account(&self, _user: UserId) -> Result<(), ServerError> {
        Err(ServerError::Internal(format!(
            "delete_account: {NOT_IMPLEMENTED}"
        )))
    }

    async fn export_user_data(&self, _user: UserId) -> Result<String, ServerError> {
        Err(ServerError::Internal(format!(
            "export_user_data: {NOT_IMPLEMENTED}"
        )))
    }

    async fn list_followers(
        &self,
        _user: UserId,
        _query: TimelineQuery,
    ) -> Result<Page<User>, ServerError> {
        Err(ServerError::Internal(format!(
            "list_followers: {NOT_IMPLEMENTED}"
        )))
    }

    async fn list_following(
        &self,
        _user: UserId,
        _query: TimelineQuery,
    ) -> Result<Page<User>, ServerError> {
        Err(ServerError::Internal(format!(
            "list_following: {NOT_IMPLEMENTED}"
        )))
    }

    async fn remove_chirp(&self, _actor: UserId, _chirp: ChirpId) -> Result<(), ServerError> {
        Err(ServerError::Internal(format!(
            "remove_chirp: {NOT_IMPLEMENTED}"
        )))
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
    ///   succeeds** is `Ok(value)` returned to the caller. A failed commit is reported as
    ///   [`ServerError::Internal`] and the write is NOT confirmed;
    /// * the closure returns `Err(e)` → the transaction is rolled back and `Err(e)` is propagated
    ///   unchanged, so a partial write never becomes durable.
    ///
    /// A success return therefore always means "durably committed", and no failure path ever leaves
    /// a write falsely confirmed.
    ///
    /// # Errors
    ///
    /// [`ServerError::Internal`] if the transaction cannot be started or the commit fails; otherwise
    /// whatever error the closure returns.
    pub async fn with_transaction<F, T>(&self, operations: F) -> Result<T, ServerError>
    where
        F: for<'c> FnOnce(&'c mut sqlx::PgConnection) -> TransactionFuture<'c, T>,
        T: Send,
    {
        let mut transaction = self.pool.begin().await.map_err(|error| {
            ServerError::Internal(format!("failed to begin a durable transaction: {error}"))
        })?;

        match operations(&mut transaction).await {
            Ok(value) => {
                transaction.commit().await.map_err(|error| {
                    ServerError::Internal(format!(
                        "durable commit failed; the write was NOT confirmed: {error}"
                    ))
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
