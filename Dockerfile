# syntax=docker/dockerfile:1
#
# chirp — single self-contained production image.
#
# One multi-stage build that (1) compiles the release `chirp-server` binary, (2) builds the
# `chirp-frontend` WebAssembly SPA and runs `wasm-bindgen` to emit its JS/wasm glue, and (3)
# assembles the server binary + the built SPA bundle into a minimal, non-root, health-checked
# distroless runtime image that serves the whole app (REST API + SPA) out of the box with the
# in-memory repository default — no external database.
#
# Build:   docker build -t chirp:local .
# Run:     docker run --rm -p 8080:8080 chirp:local
# Verify:  curl -fsS http://127.0.0.1:8080/healthz   # -> {"status":"ok",...}
#          open   http://127.0.0.1:8080/             # -> the SPA
#
# See deploy/README.md for the full build/run/config/healthcheck reference.

############################################################################
# Stage 1 — builder: compile the server binary + the wasm SPA bundle.
############################################################################
# Pinned by tag AND digest for reproducibility. `-slim-bookworm` is Debian 12 (glibc 2.36), which
# is ABI-identical to the distroless/cc-debian12 runtime base below, so the dynamically-linked
# server binary runs there unchanged. Rust 1.88 satisfies the workspace MSRV (1.82) and is required
# to build wasm-bindgen-cli 0.2.129, whose transitive deps (icu_*, time) need rustc 1.88.
FROM rust:1.88-slim-bookworm@sha256:38bc5a86d998772d4aec2348656ed21438d20fcdce2795b56ca434cf21430d89 AS builder

# The wasm-bindgen CLI MUST match the `wasm-bindgen` crate version locked in Cargo.lock, or the
# generated JS glue and the wasm module disagree at init. Keep this in lockstep with Cargo.lock.
ARG WASM_BINDGEN_VERSION=0.2.129

WORKDIR /app

# ca-certificates for the crates.io TLS fetch during `cargo install`. The chirp crate graph is pure
# Rust (no openssl/pkg-config/system -sys deps), so nothing else is needed. Cleaned in-layer.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# The wasm compile target and the matching wasm-bindgen CLI. `--locked` builds the CLI against its
# own pinned Cargo.lock for a reproducible tool. This layer is cached until the version arg changes.
RUN rustup target add wasm32-unknown-unknown \
    && cargo install wasm-bindgen-cli --version "${WASM_BINDGEN_VERSION}" --locked

# The whole workspace (the build context is trimmed by .dockerignore — notably `target/` and VCS).
# A manifest-only dependency pre-build layer is intentionally omitted: the workspace is a web of
# sibling path-crates, so a dummy-source skeleton would misrepresent the real graph. The BuildKit
# cache mounts below give the rebuild-speed benefit without that fragility.
COPY . .

# Build everything in one layer, with cargo's registry and target dirs mounted as BuildKit caches
# (fast rebuilds, and the caches never bloat an image layer). Artifacts are copied OUT of the
# cache-mounted target dir to stable /out paths the runtime stage can COPY from.
#   1. release server binary
#   2. frontend wasm (the `chirp` example is the SPA's wasm entry — calls chirp_frontend::start())
#   3. wasm-bindgen JS + wasm glue into the asset tree's pkg/ (index.html imports ./pkg/chirp.js)
#   4. the static index.html harness
#   5. the standalone healthcheck probe (distroless has no shell/curl/wget)
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    set -eux; \
    cargo build --release --locked -p chirp-server; \
    cargo build --release --locked -p chirp-frontend --example chirp --target wasm32-unknown-unknown; \
    mkdir -p /out/bin /out/static/pkg; \
    cp target/release/chirp-server /out/bin/chirp-server; \
    wasm-bindgen --target web --no-typescript --out-name chirp \
        --out-dir /out/static/pkg \
        target/wasm32-unknown-unknown/release/examples/chirp.wasm; \
    cp crates/chirp-frontend/index.html /out/static/index.html; \
    rustc -C opt-level=z -C strip=symbols deploy/docker/healthcheck.rs -o /out/bin/healthcheck

############################################################################
# Stage 2 — runtime: minimal, non-root distroless image.
############################################################################
# distroless/cc = glibc + libgcc only: NO shell, NO package manager, NO build tools — a small
# attack surface a scanner cannot flag packages it does not contain. The `:nonroot` tag ships a
# built-in non-root user (uid/gid 65532) and defaults USER to it. Pinned by tag AND digest.
FROM gcr.io/distroless/cc-debian12:nonroot@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f

LABEL org.opencontainers.image.title="chirp" \
      org.opencontainers.image.description="chirp — axum REST API + ferric WASM SPA in one minimal, non-root image" \
      org.opencontainers.image.source="https://github.com/Nousix-LLC/nswe-demo"

# Server binary + the standalone healthcheck probe (root-owned, world-executable — the default for
# COPY'd 0755 files; the non-root runtime user executes them, it does not need to own them).
COPY --from=builder /out/bin/chirp-server /usr/local/bin/chirp-server
COPY --from=builder /out/bin/healthcheck  /usr/local/bin/healthcheck

# The built SPA bundle (index.html + pkg/chirp.js + pkg/chirp_bg.wasm), owned by the non-root user
# and only ever read at runtime.
COPY --from=builder --chown=65532:65532 /out/static /srv/chirp

# Runtime configuration — all overridable at `docker run -e …`:
#   CHIRP_BIND_ADDR  bind 0.0.0.0 (not the 127.0.0.1 dev default) so the port is reachable from
#                    outside the container; 8080 is unprivileged, so the non-root user can bind it.
#   CHIRP_STATIC_DIR where the server reads the SPA bundle from inside the image.
#   RUST_LOG         tracing goes to stdout/stderr for `docker logs` capture.
ENV CHIRP_BIND_ADDR=0.0.0.0:8080 \
    CHIRP_STATIC_DIR=/srv/chirp \
    RUST_LOG=info,chirp_server=info,tower_http=info

EXPOSE 8080

# Already non-root via the `:nonroot` tag; stated explicitly for auditability.
USER 65532:65532

# Liveness against /healthz via the standalone probe (no shell in distroless to run curl/wget).
# start-period is short because the in-memory server is ready in well under a second.
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/healthcheck"]

# exec form → chirp-server is PID 1 and receives SIGTERM directly; it already drains in-flight
# requests on signal via tokio graceful shutdown.
ENTRYPOINT ["/usr/local/bin/chirp-server"]
