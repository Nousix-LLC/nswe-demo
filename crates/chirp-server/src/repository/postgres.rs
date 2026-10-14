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
use chirp_types::ids::{ChirpId, UserId};

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

#[async_trait]
impl ChirpRepository for PostgresRepository {
    // ---- Existing capability_set (scaffold stubs) -----------------------------------------------
    // The durable bodies are downstream; each stub reports a failed action and never falsely
    // confirms success, preserving the error contract and the `confirmed_write_durable` identity.

    async fn create_user(&self, _request: CreateUserRequest) -> Result<User, ServerError> {
        Err(ServerError::Internal(format!(
            "create_user: {NOT_IMPLEMENTED}"
        )))
    }

    async fn get_user(&self, _id: UserId) -> Result<User, ServerError> {
        Err(ServerError::Internal(format!(
            "get_user: {NOT_IMPLEMENTED}"
        )))
    }

    async fn login(&self, _request: LoginRequest) -> Result<AuthResponse, ServerError> {
        Err(ServerError::Internal(format!("login: {NOT_IMPLEMENTED}")))
    }

    async fn resolve_token(&self, _token: &SessionToken) -> Result<UserId, ServerError> {
        Err(ServerError::Internal(format!(
            "resolve_token: {NOT_IMPLEMENTED}"
        )))
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

    async fn set_credential(&self, _user: UserId, _secret: &str) -> Result<(), ServerError> {
        Err(ServerError::Internal(format!(
            "set_credential: {NOT_IMPLEMENTED}"
        )))
    }

    async fn verify_credential(
        &self,
        _username: &Username,
        _secret: &str,
    ) -> Result<UserId, ServerError> {
        Err(ServerError::Internal(format!(
            "verify_credential: {NOT_IMPLEMENTED}"
        )))
    }

    async fn invalidate_session(&self, _token: &SessionToken) -> Result<(), ServerError> {
        Err(ServerError::Internal(format!(
            "invalidate_session: {NOT_IMPLEMENTED}"
        )))
    }

    async fn begin_account_recovery(&self, _username: &Username) -> Result<(), ServerError> {
        Err(ServerError::Internal(format!(
            "begin_account_recovery: {NOT_IMPLEMENTED}"
        )))
    }

    async fn complete_account_recovery(
        &self,
        _username: &Username,
        _recovery_token: &str,
        _new_secret: &str,
    ) -> Result<(), ServerError> {
        Err(ServerError::Internal(format!(
            "complete_account_recovery: {NOT_IMPLEMENTED}"
        )))
    }

    async fn update_profile(
        &self,
        _user: UserId,
        _display_name: String,
        _bio: Option<String>,
    ) -> Result<User, ServerError> {
        Err(ServerError::Internal(format!(
            "update_profile: {NOT_IMPLEMENTED}"
        )))
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

    async fn set_account_suspension(
        &self,
        _actor: UserId,
        _target: UserId,
        _suspended: bool,
    ) -> Result<(), ServerError> {
        Err(ServerError::Internal(format!(
            "set_account_suspension: {NOT_IMPLEMENTED}"
        )))
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
