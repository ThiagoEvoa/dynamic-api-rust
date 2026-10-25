//! Integration tests for the `/` route, mirroring `test/routes/index_test.dart`
//! from `dynamic_api_dart` and adding coverage for the 400 path.
//!
//! `axum::test::TestClient` is crate-internal, so we drive the router directly
//! with `tower::ServiceExt::oneshot`, which exercises the full router pipeline
//! (routing, `405`, `400`, and the handler logic) without binding a socket.

use axum::body::{to_bytes, Body, Bytes};
use axum::http::Request;
use axum::response::Response;
use dynamic_api::{build_router, AppState};
use tower::ServiceExt; // for `oneshot`

/// Build a request for `/` with the given method and a JSON string body.
/// An empty string yields an empty body.
fn request(method: &str, body: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri("/")
        .body(Body::from(body.as_bytes().to_vec()))
        .expect("request builds")
}

async fn read_body(resp: Response) -> Vec<u8> {
    to_bytes(resp.into_body(), usize::MAX)
        .await
        .expect("read body")
        .to_vec()
}

#[tokio::test]
async fn get_returns_empty_json_by_default() {
    for state in [AppState::new(), AppState::default()] {
        let app = build_router().with_state(state);
        let resp = app.oneshot(request("GET", "")).await.expect("oneshot");

        assert_eq!(resp.status(), 200, "GET / should be 200");
        assert_eq!(
            read_body(resp).await,
            b"{}",
            "both constructors should initialize an empty JSON object"
        );
    }
}

#[tokio::test]
async fn post_replaces_document_and_returns_it() {
    let state = AppState::new();
    let payload = r#"{"foo":"bar"}"#;

    let post = build_router()
        .with_state(state.clone())
        .oneshot(request("POST", payload))
        .await
        .expect("post oneshot");
    assert_eq!(post.status(), 200, "POST / should be 200");
    assert_eq!(
        read_body(post).await,
        payload.as_bytes(),
        "POST returns the new document"
    );

    // The document persists for a subsequent request on the same store.
    let get = build_router()
        .with_state(state.clone())
        .oneshot(request("GET", ""))
        .await
        .expect("get oneshot");
    assert_eq!(
        read_body(get).await,
        payload.as_bytes(),
        "GET reflects last POST"
    );
}

#[tokio::test]
async fn post_accepts_non_object_json() {
    // The reference uses a `dynamic` document, so any JSON value is valid.
    let app = build_router().with_state(AppState::new());
    let payload = r#"[1,2,3]"#;

    let resp = app
        .oneshot(request("POST", payload))
        .await
        .expect("oneshot");
    assert_eq!(resp.status(), 200, "POST / should accept any JSON value");
    assert_eq!(read_body(resp).await, payload.as_bytes());
}

#[tokio::test]
async fn invalid_json_body_is_bad_request_and_leaves_store_unchanged() {
    let state = AppState::new();

    let resp = build_router()
        .with_state(state.clone())
        .oneshot(request("POST", "not json"))
        .await
        .expect("oneshot");
    assert_eq!(
        resp.status(),
        400,
        "malformed body should be 400 Bad Request"
    );

    // The store must remain the empty default.
    let get = build_router()
        .with_state(state.clone())
        .oneshot(request("GET", ""))
        .await
        .expect("oneshot");
    assert_eq!(read_body(get).await, b"{}");
}

#[tokio::test]
async fn disallowed_methods_return_405() {
    let app = build_router().with_state(AppState::new());
    // NOTE: HEAD is intentionally excluded — axum's `get(...)` maps both GET and
    // HEAD, so `HEAD /` is a legal 200 with an empty body, not a 405.
    for method in ["PUT", "DELETE", "PATCH", "OPTIONS"] {
        // `oneshot` consumes the service, so clone the (cheap, `Arc`-backed)
        // router for each independent request.
        let resp = app
            .clone()
            .oneshot(request(method, ""))
            .await
            .expect("oneshot");
        assert_eq!(
            resp.status(),
            405,
            "{method} / should be 405 Method Not Allowed"
        );
    }
}

/// `HEAD /` is served like `GET /` (axum's `get(...)` maps both) but with no
/// response body — it is NOT a 405.
#[tokio::test]
async fn head_succeeds_with_empty_body() {
    let app = build_router().with_state(AppState::new());

    let get = build_router()
        .with_state(AppState::new())
        .oneshot(request("GET", ""))
        .await
        .expect("get oneshot");
    assert_eq!(get.status(), 200, "GET / should be 200");
    assert_eq!(read_body(get).await, b"{}", "GET returns the body");

    let resp = app
        .oneshot(
            Request::builder()
                .method("HEAD")
                .uri("/")
                .body(Body::empty())
                .expect("head request builds"),
        )
        .await
        .expect("head oneshot");
    assert_eq!(resp.status(), 200, "HEAD / should be 200, not 405");
    assert_eq!(read_body(resp).await, b"", "HEAD must return an empty body");
}

/// Any path other than `/` yields `404 Not Found`.
#[tokio::test]
async fn unknown_path_returns_404() {
    let app = build_router().with_state(AppState::new());

    let resp = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/not-root")
                .body(Body::empty())
                .expect("request builds"),
        )
        .await
        .expect("oneshot");
    assert_eq!(resp.status(), 404, "a non-`/` path should be 404 Not Found");
}

/// A `400` response carries a JSON body shaped as `{ "error": ... }`.
#[tokio::test]
async fn bad_request_responds_with_error_json() {
    let resp = build_router()
        .with_state(AppState::new())
        .oneshot(request("POST", "not json"))
        .await
        .expect("oneshot");
    assert_eq!(resp.status(), 400);

    let bytes = read_body(resp).await;
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).expect("400 body is JSON");
    assert!(
        parsed.get("error").is_some(),
        "400 body should contain an `error` field"
    );
}

/// `POST` accepts any JSON value, so a non-object (array) persists and is
/// returned by a subsequent `GET` on the same store.
#[tokio::test]
async fn post_then_get_reflects_non_object_json() {
    let state = AppState::new();
    let payload = r#"[1,2,3]"#;

    let post = build_router()
        .with_state(state.clone())
        .oneshot(request("POST", payload))
        .await
        .expect("post oneshot");
    assert_eq!(post.status(), 200, "POST / should be 200");

    let get = build_router()
        .with_state(state)
        .oneshot(request("GET", ""))
        .await
        .expect("get oneshot");
    assert_eq!(
        read_body(get).await,
        payload.as_bytes(),
        "GET / reflects the persisted non-object document"
    );
}

/// axum enforces a default 2 MiB request-body limit, so a larger body is
/// rejected (`413 Payload Too Large`) before the store is touched.
#[tokio::test]
async fn oversized_body_returns_413() {
    let state = AppState::new();
    let big = vec![b'1'; 2 * 1024 * 1024 + 1]; // one byte over the 2 MiB default
    let req = Request::builder()
        .method("POST")
        .uri("/")
        .body(Body::from(Bytes::from(big)))
        .expect("request builds");

    let resp = build_router()
        .with_state(state.clone())
        .oneshot(req)
        .await
        .expect("oneshot");
    assert_eq!(
        resp.status(),
        413,
        "a body over 2 MiB should be 413 Payload Too Large"
    );

    // The oversized body is rejected before replacing the store.
    let get = build_router()
        .with_state(state)
        .oneshot(request("GET", ""))
        .await
        .expect("get oneshot");
    assert_eq!(read_body(get).await, b"{}", "store unchanged after 413");
}
