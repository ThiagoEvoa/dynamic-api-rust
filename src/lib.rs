//! `dynamic_api_rust`: a minimal dynamic JSON store HTTP API.
//!
//! This is a Rust port of [`dynamic_api_dart`]. It exposes a single route, `/`,
//! that acts as a process-wide, in-memory JSON document:
//!
//! * `GET /`  returns the stored JSON (an empty object `{}` by default).
//! * `HEAD /` like `GET /` but no response body — axum's `get(...)` maps both `GET` and `HEAD`.
//! * `POST /` replaces the stored JSON with the request body and returns it.
//! * any other method on `/` (`PUT`, `DELETE`, `PATCH`, `OPTIONS`) is `405 Method Not Allowed` (axum).
//!
//! [`dynamic_api_dart`]: https://github.com/ThiagoEvoa/dynamic_api_dart

mod handlers;
mod router;
mod state;

pub use router::build_router;
pub use state::AppState;
