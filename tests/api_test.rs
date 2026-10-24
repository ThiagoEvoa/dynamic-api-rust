//! Integration tests for the `/` route, mirroring `test/routes/index_test.dart`
//! from `dynamic_api_dart` and adding coverage for the 400 path.
//!
//! `axum::test::TestClient` is crate-internal, so we drive the router directly
//! with `tower::ServiceExt::oneshot`, which exercises the full router pipeline
//! (routing, `405`, `400`, and the handler logic) without binding a socket.

use axum::body::{to_bytes, Body};
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
    for method in ["PUT", "DELETE", "PATCH"] {
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
