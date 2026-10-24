# ---------- Build stage ----------
# Official Rust image with a matching glibc-based Debian base.
# `rust:1-bookworm-slim` pins the latest stable 1.x toolchain on bookworm,
# matching the `debian:bookworm-slim` runtime below so the release binary links
# cleanly.
FROM rust:1-bookworm-slim AS build

WORKDIR /app

# Cache dependencies in their own layer: only rebuilt when manifests change.
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# AOT-compile a single release binary (analogous to `dart compile exe`).
COPY src ./src
RUN --network=none cargo build --release --bin dynamic-api \
    && cp target/release/dynamic-api /usr/local/bin/dynamic-api

# ---------- Runtime stage ----------
# Minimal Debian base with only the shared libraries a Rust release binary needs
# (glibc, SSL for ca-certificates). No Rust toolchain or source shipped.
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/local/bin/dynamic-api /usr/local/bin/dynamic-api

# Expose the port the API binds by default (matches the Dart reference: 8080).
EXPOSE 8080

CMD ["dynamic-api"]
