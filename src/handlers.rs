//! HTTP handlers for the `/` route.
//!
//! `GET` reads the document; `POST` replaces it with the parsed body. Any other
//! HTTP method on `/` is rejected with `405 Method Not Allowed` by axum itself
//! (see [`crate::build_router`]); no explicit handling is required.

use crate::AppState;
use axum::body::{Body, Bytes};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, Json};

/// `GET /` — return the current stored JSON document.
pub async fn get(State(state): State<AppState>) -> Response<Body> {
    Json(state.current()).into_response()
}

/// `POST /` — parse the request body as JSON and replace the stored document.
///
/// The body must be a single valid JSON value (object, array, number, string,
/// boolean, or null) of any shape, matching the reference's dynamic semantics.
/// An empty or malformed body yields `400 Bad Request` and leaves the store
/// unchanged.
pub async fn post(State(state): State<AppState>, body: Bytes) -> Response<Body> {
    match serde_json::from_slice::<serde_json::Value>(&body) {
        Ok(value) => {
            state.replace(value.clone());
            Json(value).into_response()
        }
        Err(err) => {
            // Preserve the store on parse failure.
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": format!("invalid json body: {err}")
                })),
            )
                .into_response()
        }
    }
}
