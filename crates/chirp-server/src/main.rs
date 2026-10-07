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

use std::sync::Arc;

use chirp_server::{build_router, AppState, InMemoryRepository};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    // Default backing store: fully in-memory, so the server boots with no database.
    let state = AppState::new(Arc::new(InMemoryRepository::new()));
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

/// Resolve when the server receives Ctrl-C, driving `axum`'s graceful shutdown.
///
/// On signal the server stops accepting new connections and lets in-flight requests finish. If the
/// Ctrl-C handler cannot be installed the future returns immediately, so startup never hangs on a
/// platform that denies it.
async fn shutdown_signal() {
    match tokio::signal::ctrl_c().await {
        Ok(()) => tracing::info!("shutdown signal received; draining in-flight requests"),
        Err(error) => tracing::error!(%error, "failed to install Ctrl-C handler; shutting down"),
    }
}
