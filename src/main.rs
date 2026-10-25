//! Production entrypoint for `dynamic-api`.
//!
//! Boots a Tokio multi-threaded runtime, binds a TCP listener, and serves the
//! router built by [`dynamic_api::build_router`]. Host and port are configurable
//! via the `HOST` and `PORT` environment variables (defaults: `0.0.0.0`,
//! `8080` — matching the Dart reference's default port).
//!
//! The server shuts down gracefully on `SIGINT` (interactive Ctrl-C), and, on
//! Unix, on `SIGTERM` (the default container stop signal sent by
//! `docker`/`compose`): new connections are refused while in-flight requests are
//! allowed to finish, then the process exits. (`SIGTERM` isn't available on non-
//! Unix platforms, so only `SIGINT` is watched there.)

use std::net::SocketAddr;

use dynamic_api::{build_router, AppState};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let addr = resolve_addr();
    let state = AppState::new();
    let app = build_router().with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("dynamic-api-rust listening on http://{addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    eprintln!("dynamic-api-rust stopped gracefully");
    Ok(())
}

/// Resolves to `()` when a shutdown signal is received, which drives
/// `axum::serve`'s graceful shutdown.
///
/// On Unix (our deployment target) this additionally watches `SIGTERM` — the
/// stop signal `docker`/`compose` send by default — alongside `SIGINT`. On
/// other platforms only `SIGINT`/Ctrl+C is available, so we watch that.
#[cfg(unix)]
async fn shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigterm =
        signal(SignalKind::terminate()).expect("failed to register a SIGTERM handler");

    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .map_err(|err| eprintln!("WARN: failed to catch Ctrl+C: {err}"))
            .ok();
    };

    tokio::select! {
        _ = ctrl_c => eprintln!("received SIGINT (Ctrl+C); shutting down gracefully"),
        _ = sigterm.recv() => eprintln!("received SIGTERM; shutting down gracefully"),
    }

    // `Signal` stops observing for signals when it is dropped, which happens at
    // the end of this scope; that is enough before the process exits.
    drop(sigterm);
}

#[cfg(not(unix))]
async fn shutdown_signal() {
    // `SIGTERM` is not a portable concept; only the Ctrl-C interrupt is honored
    // on these platforms.
    if let Err(err) = tokio::signal::ctrl_c().await {
        eprintln!("WARN: failed to catch Ctrl+C: {err}");
    }
    eprintln!("received SIGINT (Ctrl+C); shutting down gracefully");
}

fn resolve_addr() -> SocketAddr {
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    match format!("{host}:{port}").parse::<SocketAddr>() {
        Ok(addr) => addr,
        Err(err) => {
            eprintln!("WARN: could not parse HOST={host}/PORT={port} ({err}); falling back to 0.0.0.0:8080");
            "0.0.0.0:8080"
                .parse()
                .expect("default address literal is valid")
        }
    }
}
