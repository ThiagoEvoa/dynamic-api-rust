# Pinned to image *digests* for reproducible builds. When you bump the Rust
# toolchain or Debian base, update the two ARG defaults to the freshly-pulled
# `...@sha256:...` digest for the target tags (see .github/workflows/ci.yml and
# the routine below). Note: `rust:1-bookworm-slim` does not exist on Docker Hub;
# Docker's scheme is `rust:1-slim-bookworm` (Debian bookworm-slim + Rust 1.x).
ARG RUST_IMAGE=rust:1-slim-bookworm@sha256:86234e857a413004348b471d11a4ebe1aa92d2eae2b6b143cac68fd1ac3c48be
ARG DEBIAN_IMAGE=debian:bookworm-slim@sha256:7ff6ba31527b9e336162fa061746c366c4b2032f4d2cff2078bea43f78efe3f8

# ---------- Build stage ----------
# Official Rust image on a matching glibc-based Debian base, so the release
# binary links cleanly against the runtime below.
FROM ${RUST_IMAGE} AS build

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
FROM ${DEBIAN_IMAGE}

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /usr/local/bin/dynamic-api /usr/local/bin/dynamic-api

# Expose the port the API binds by default (matches the Dart reference: 8080).
EXPOSE 8080

CMD ["dynamic-api"]
