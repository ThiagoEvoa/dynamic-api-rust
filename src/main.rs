//! Production entrypoint for `dynamic-api`.
//!
//! Boots a Tokio multi-threaded runtime, binds a TCP listener, and serves the
//! router built by [`dynamic_api::build_router`]. Host and port are configurable
//! via the `HOST` and `PORT` environment variables (defaults: `0.0.0.0`,
//! `8080` — matching the Dart reference's default port).

use std::net::SocketAddr;

use dynamic_api::{build_router, AppState};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let addr = resolve_addr();
    let state = AppState::new();
    let app = build_router().with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("dynamic-api-rust listening on http://{addr}");

    axum::serve(listener, app).await?;
    Ok(())
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
