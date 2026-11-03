//! `chirp-server` binary entrypoint.
//!
//! Boots the axum server over the default in-memory repository — no database or any external
//! infrastructure required. The binary is a thin shell: it installs tracing, constructs the shared
//! [`AppState`] and the [`build_router`] composition root (the same entry point the integration
//! tests use), binds a TCP listener, and serves with graceful shutdown on Ctrl-C.
//!
//! # Running
//!
//! ```text
//! cargo run -p chirp-server
//! ```
//!
//! # Configuration (environment)
//!
//! * `CHIRP_BIND_ADDR` — the `host:port` to bind (default `127.0.0.1:3000`).
//! * `CHIRP_STATIC_DIR` — the static-asset root for the SPA fallback (default: the crate-local
//!   `static/` directory; it currently holds a placeholder page until the SPA bundle lands in #9).
//! * `RUST_LOG` — the `tracing-subscriber` env-filter (default
//!   `info,chirp_server=debug,tower_http=info`).
//! * `CHIRP_DATABASE_URL` / `DATABASE_URL` — the PostgreSQL connection string that selects the
//!   **durable** backing (ADR-0001). The selection is **config-driven, not a code edit**: when the
//!   binary is built with `--features postgres` *and* a non-empty connection string is set (the
//!   production path), the server constructs the durable `PostgresRepository`, applies migrations,
//!   and serves against it; when the string is absent/empty, or the binary was built without the
//!   `postgres` feature (the test/local default), it constructs the in-memory
//!   [`InMemoryRepository`] and needs no database. `CHIRP_DATABASE_URL` takes precedence over
//!   `DATABASE_URL` when both are set.

use std::sync::Arc;

use chirp_server::{build_router, AppState, InMemoryRepository};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

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
