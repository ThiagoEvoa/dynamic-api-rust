//! Router assembly.
//!
//! The single `/` route accepts `GET` and `POST` only, so axum responds with
//! `405 Method Not Allowed` (and an `Allow` header) for any other method, and
//! `404 Not Found` for any other path.
//!
//! The function returns a [`Router<AppState>`] so that callers (production and
//! tests) bind the concrete [`AppState`] via `.with_state(...)`.

use crate::AppState;
use axum::{routing::get, Router};

/// Builds the application router, parameterized over the shared [`AppState`].
pub fn build_router() -> Router<AppState> {
    Router::new().route("/", get(crate::handlers::get).post(crate::handlers::post))
}
