//! Router assembly.
//!
//! The `/` route is registered with `get(...).post(...)`. Because axum's `get`
//! maps both `GET` and `HEAD`, `HEAD /` succeeds with an empty body; any other
//! method (`PUT`, `DELETE`, `PATCH`, `OPTIONS`, …) yields `405 Method Not Allowed`
//! (with an `Allow` header), and any non-`/` path yields `404 Not Found`.
//!
//! The function returns a [`Router<AppState>`] so that callers (production and
//! tests) bind the concrete [`AppState`] via `.with_state(...)`.

use crate::AppState;
use axum::{routing::get, Router};

/// Builds the application router, parameterized over the shared [`AppState`].
pub fn build_router() -> Router<AppState> {
    Router::new().route("/", get(crate::handlers::get).post(crate::handlers::post))
}
