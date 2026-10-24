# dynamic-api-rust

A minimal **dynamic JSON store** HTTP API — a Rust port of
[dynamic_api_dart](https://github.com/ThiagoEvoa/dynamic_api_dart), built with
**[axum](https://github.com/tokio-rs/axum)**.

## What it does

A single in-memory, process-wide JSON document is exposed at `/`:

| Method | Behaviour |
| --- | --- |
| `GET /` | Returns the current JSON document. Starts as an empty object `{}`. → `200` |
| `POST /` | Parses the request body as JSON and **replaces** the stored document with it, then returns the new document. → `200` (or `400` on invalid JSON) |
| any other method on `/` | `405 Method Not Allowed` (handled automatically by axum) |

The stored document accepts any JSON value (object, array, string, …), matching
the dynamic semantics of the original Dart `dynamic json = {}`.

## Development

```bash
# Run locally (binds 0.0.0.0:8080 by default; override with HOST/PORT)
cargo run

# Try it
curl -s localhost:8080/                      # {}
curl -s -X POST -d '{"foo":"bar"}' localhost:8080/   # {"foo":"bar"}
curl -s localhost:8080/                      # {"foo":"bar"}
curl -s -X PUT localhost:8080/ -o /dev/null -w '%{http_code}\n'   # 405
```

Set `PORT` and/or `HOST` to change the bind address.

## Quality gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Docker

```bash
docker build -t dynamic-api-rust .
docker run --rm -p 8080:8080 dynamic-api-rust
```

## Deploy (Cloudflare tunnel)

Mirrors the reference's `docker-compose.yaml`: the API service and a
`cloudflared` tunnel service on a shared network.

```bash
TUNNEL_TOKEN=... docker compose up --build
```

## Project layout

| Path | Purpose |
| --- | --- |
| `src/lib.rs` | Crate root; public API (`build_router`, `AppState`). |
| `src/state.rs` | Shared `Arc<Mutex<serde_json::Value>>` store (`AppState`). |
| `src/handlers.rs` | `GET`/`POST` handlers. |
| `src/router.rs` | Route assembly. |
| `src/main.rs` | Server bootstrap (runtime + listener). |
| `tests/api_test.rs` | Integration tests mirroring the Dart suite. |

## License

MIT.
