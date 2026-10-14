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
