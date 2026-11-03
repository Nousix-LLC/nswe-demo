//! `chirp-server` binary entrypoint.
//!
//! Installs tracing, then dispatches on the optional first argument:
//!
//! * **No argument (default)** — serve mode: constructs [`AppState`], builds the axum router via
//!   [`build_router`], binds a TCP listener, and serves with graceful shutdown on SIGINT/SIGTERM.
//!   Selects the durable PostgreSQL backing when the binary is built with `--features postgres`
//!   *and* `CHIRP_DATABASE_URL` / `DATABASE_URL` is set (the production path); otherwise the
//!   in-memory [`InMemoryRepository`].
//!
//! * **`sweep`** — sweep mode (requires `--features postgres` and a configured database URL):
//!   connects to PostgreSQL (applying migrations on first connect, exactly as the serve path does),
//!   calls [`ChirpRepository::sweep_expired_transient_state`], logs the per-class reclaimed counts,
//!   and exits 0. This is the scheduled reclamation path for expired transient durable state —
//!   sessions, recovery tokens, export artefacts (REQ-OPS-010 / BAR-022). It calls the single Rust
//!   seam rather than duplicating the DELETE predicates (REQ-DOM-014 compliance).
//!
//! # Running
//!
//! ```text
//! # serve (default)
//! cargo run -p chirp-server
//!
//! # sweep (requires the postgres feature and a database URL)
//! CHIRP_DATABASE_URL=postgres://... cargo run -p chirp-server --features postgres -- sweep
//! ```
//!
//! # Configuration (environment)
//!
//! * `CHIRP_BIND_ADDR` — host:port to bind (default `127.0.0.1:3000`); serve mode only.
//! * `CHIRP_STATIC_DIR` — static-asset root for the SPA fallback; serve mode only.
//! * `RUST_LOG` — tracing env-filter (default `info,chirp_server=debug,tower_http=info`).
//! * `CHIRP_DATABASE_URL` / `DATABASE_URL` — PostgreSQL connection string (ADR-0001). Required for
//!   `sweep`; optional for serve (falls back to in-memory when absent). `CHIRP_DATABASE_URL` takes
//!   precedence over `DATABASE_URL` when both are set.

use std::sync::Arc;

use chirp_server::{build_router, AppState, InMemoryRepository};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let subcommand = std::env::args().nth(1);
    match subcommand.as_deref() {
        Some("sweep") => run_sweep().await,
        None => run_serve().await,
        Some(cmd) => {
            Err(format!("unknown subcommand '{cmd}'; use 'sweep' or omit to serve").into())
        }
    }
}

/// Connects to PostgreSQL and runs the transient-state sweep (the `sweep` subcommand).
///
/// Calls [`chirp_server::ChirpRepository::sweep_expired_transient_state`] — the single Rust seam —
/// rather than duplicating the DELETE predicates (REQ-DOM-014). Applies migrations on connect so
/// the sweep path, like the serve path, never requires a pre-migrated schema.
///
/// # Errors
///
/// Returns an error if `CHIRP_DATABASE_URL` / `DATABASE_URL` is absent, if the connection pool
/// cannot be established, if migrations fail, or if the sweep itself fails.
#[cfg(feature = "postgres")]
async fn run_sweep() -> Result<(), Box<dyn std::error::Error>> {
    use chirp_server::ChirpRepository;

    let database_url = durable_database_url()
        .ok_or("chirp-server sweep: CHIRP_DATABASE_URL or DATABASE_URL must be set")?;
    tracing::info!("sweep: connecting to PostgreSQL and applying migrations");
    let repo = chirp_server::PostgresRepository::connect(&database_url).await?;
    tracing::info!("sweep: calling sweep_expired_transient_state");
    let result = repo.sweep_expired_transient_state().await?;
    tracing::info!(
        sessions = result.sessions,
        recovery_tokens = result.recovery_tokens,
        export_artifacts = result.export_artifacts,
        total = result.total(),
        "sweep: complete"
    );
    Ok(())
}

/// Stub used when the binary is built without `--features postgres`.
///
/// The `sweep` subcommand requires a durable store; the in-memory backing cannot satisfy the
/// transient-sweep contract. Returning an error here makes a misconfigured binary fail loudly
/// rather than silently running a no-op.
#[cfg(not(feature = "postgres"))]
async fn run_sweep() -> Result<(), Box<dyn std::error::Error>> {
    Err(
        "chirp-server sweep: this binary was built without the 'postgres' feature; \
         rebuild with '--features postgres' to enable the sweep subcommand"
            .into(),
    )
}

/// Starts the axum HTTP server (the default mode when no subcommand is given).
async fn run_serve() -> Result<(), Box<dyn std::error::Error>> {
    // Config-driven backend selection (ADR-0001): durable PostgreSQL when configured and compiled
    // in, in-memory otherwise. No code edit is required to switch between them.
    let state = build_app_state().await?;
    let app = build_router(state);

    let addr = std::env::var("CHIRP_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_owned());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "chirp-server listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("chirp-server stopped");
    Ok(())
}

/// Builds the shared [`AppState`] by selecting the persistence backing from configuration.
///
/// Returns the durable `PostgresRepository` when the binary is built with the `postgres` feature
/// **and** a non-empty connection string is configured (`CHIRP_DATABASE_URL` or `DATABASE_URL`);
/// otherwise the in-memory [`InMemoryRepository`]. Constructing the durable backing also applies the
/// embedded migrations, so a returned durable state is already schema-current. Only the backend
/// *construction* varies here — the router, handlers, and `AppState`'s `Arc<dyn ChirpRepository>`
/// shape are identical for both backings.
///
/// # Errors
///
/// Propagates an error if the durable backing is selected but its connection pool cannot be
/// established or its migrations fail.
async fn build_app_state() -> Result<AppState, Box<dyn std::error::Error>> {
    #[cfg(feature = "postgres")]
    {
        if let Some(database_url) = durable_database_url() {
            tracing::info!(
                "durable backing selected: connecting to PostgreSQL and applying migrations"
            );
            let repo = chirp_server::PostgresRepository::connect(&database_url).await?;
            return Ok(AppState::new(Arc::new(repo)));
        }
        tracing::info!("no database URL configured; using the in-memory backing");
    }

    #[cfg(not(feature = "postgres"))]
    {
        // The durable backing is not compiled in. If a URL is nonetheless set, warn loudly so a
        // production misconfiguration (DB URL present, but the binary built without `postgres`) is
        // visible rather than silently serving non-durable data.
        if durable_database_url().is_some() {
            tracing::warn!(
                "a database URL is configured but this binary was built without the `postgres` \
                 feature; falling back to the in-memory backing (data is NOT durable)"
            );
        }
    }

    Ok(AppState::new(Arc::new(InMemoryRepository::new())))
}

/// Reads the durable-store connection string from the environment, preferring `CHIRP_DATABASE_URL`
/// over `DATABASE_URL`. An unset, empty, or whitespace-only value is treated as "not configured".
fn durable_database_url() -> Option<String> {
    ["CHIRP_DATABASE_URL", "DATABASE_URL"]
        .into_iter()
        .find_map(|key| std::env::var(key).ok())
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

/// Install the global `tracing` subscriber.
///
/// Uses the `RUST_LOG` env-filter when present; otherwise a sensible default that keeps the crate
/// at `debug` and third-party noise at `info`. Installed once, in the binary only — library code
/// and tests never install a global subscriber.
fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,chirp_server=debug,tower_http=info"));
    fmt().with_env_filter(filter).init();
}

/// Resolve when the server receives a shutdown signal, driving `axum`'s graceful shutdown.
///
/// Waits for **either** `SIGINT` (Ctrl-C, interactive runs) **or** `SIGTERM` (the signal
/// `docker stop`, Kubernetes, and other orchestrators send first). Handling SIGTERM is what lets a
/// containerized instance drain in-flight requests and exit 0 rather than being SIGKILL'd (exit
/// 137) when its grace period expires. On signal the server stops accepting new connections and
/// lets in-flight requests finish. If a handler cannot be installed the corresponding future
/// resolves (SIGINT: immediately; SIGTERM: never, deferring to SIGINT), so startup never hangs on a
/// platform that denies it.
async fn shutdown_signal() {
    let interrupt = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "failed to install Ctrl-C (SIGINT) handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        match signal(SignalKind::terminate()) {
            Ok(mut term) => {
                term.recv().await;
            }
            Err(error) => {
                tracing::error!(%error, "failed to install SIGTERM handler");
                // Defer to the SIGINT branch rather than triggering a spurious shutdown.
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = interrupt => tracing::info!("SIGINT received; draining in-flight requests"),
        _ = terminate => tracing::info!("SIGTERM received; draining in-flight requests"),
    }
}
