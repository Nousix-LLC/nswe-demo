# Deploying `chirp` as a container

`chirp` ships as a **single self-contained image**: one multi-stage [`Dockerfile`](../Dockerfile) at the
repo root compiles the release `chirp-server` binary, builds the `chirp-frontend` WebAssembly SPA, and
assembles both into a minimal, non-root, health-checked runtime image that serves the whole app — the
REST API under `/api/…` and the SPA at `/` — out of the box with the in-memory repository default. **No
external database is required.**

> Scope: this document and the `Dockerfile` cover building and running the image locally. CI image
> publishing (GHCR) is owned by the `ci-actions` work-item (`.github/workflows`), and Kubernetes
> manifests / ArgoCD by `deploy-argocd`; neither is in this image's scope.

## Build

```sh
# From the repository root:
docker build -t chirp:local .
```

The build is a two-stage pipeline:

| Stage | Base image | What it does |
|-------|-----------|--------------|
| `builder` | `rust:1.88-slim-bookworm` (pinned by digest) | `cargo install wasm-bindgen-cli@0.2.129`; builds the release server binary; builds the frontend `chirp` wasm example and runs `wasm-bindgen --target web`; compiles the standalone healthcheck probe. |
| runtime | `gcr.io/distroless/cc-debian12:nonroot` (pinned by digest) | Contains only the server binary, the healthcheck probe, and the built SPA bundle. No shell, package manager, or build toolchain. |

Both base images are pinned by **tag + `@sha256` digest** for reproducible builds. `wasm-bindgen-cli` is
pinned to **0.2.129** to match the `wasm-bindgen` crate version in `Cargo.lock`; keep the two in lockstep
(the `WASM_BINDGEN_VERSION` build-arg overrides it if needed, but it must equal the locked crate version).

The builder is Debian 12 **bookworm** (glibc 2.36) — the same libc as the distroless `cc-debian12`
runtime — so the dynamically-linked server binary runs unchanged in the final image.

## Run

```sh
docker run --rm -p 8080:8080 chirp:local
```

Then:

```sh
curl -fsS http://127.0.0.1:8080/healthz      # -> {"status":"ok","service":"chirp-server","version":"0.1.0"}
open        http://127.0.0.1:8080/            # -> the chirp SPA (served from the same origin as the API)
```

The SPA calls the REST API at the **relative** base `/api`, i.e. the same origin that served it, so no
host/port is baked into the frontend — the image works behind any hostname or reverse proxy without a
rebuild.

### Hardened run (optional)

The image is friendly to a read-only root filesystem (the in-memory server writes nothing to disk):

```sh
docker run --rm -p 8080:8080 \
  --read-only --tmpfs /tmp \
  --cap-drop=ALL \
  --security-opt=no-new-privileges \
  --memory 256m --cpus 1 --pids-limit 128 \
  chirp:local
```

## Configuration

All configuration is via environment variables, overridable at `docker run -e …`. The image ships
production-appropriate defaults:

| Variable | Image default | Purpose |
|----------|---------------|---------|
| `CHIRP_BIND_ADDR` | `0.0.0.0:8080` | `host:port` the server binds. `0.0.0.0` (not the `127.0.0.1` dev default) makes the published port reachable; `8080` is unprivileged so the non-root user can bind it. |
| `CHIRP_STATIC_DIR` | `/srv/chirp` | Directory the server serves the SPA bundle from (`index.html` + `pkg/`). |
| `RUST_LOG` | `info,chirp_server=info,tower_http=info` | `tracing` env-filter; logs go to stdout/stderr for `docker logs`. |

To run on a different port, set **one** knob — the healthcheck derives its port from the same variable:

```sh
docker run --rm -p 9000:9000 -e CHIRP_BIND_ADDR=0.0.0.0:9000 chirp:local
```

## Healthcheck

The image declares a Docker `HEALTHCHECK` that probes `GET /healthz` every 30s (5s timeout, 5s
start-period, 3 retries). Because the distroless runtime has no shell or `curl`/`wget`, the probe is a
tiny std-only Rust binary ([`deploy/docker/healthcheck.rs`](docker/healthcheck.rs), compiled with
`rustc -O` in the build stage) that performs one loopback `GET /healthz` and exits `0` on HTTP 200,
non-zero otherwise. It reads the port from the same `CHIRP_BIND_ADDR` the server uses, so there is no
second port knob to keep in sync.

Inspect health status:

```sh
docker inspect --format '{{.State.Health.Status}}' <container>   # -> healthy
```

## Image characteristics

- **Non-root:** runs as uid/gid `65532:65532` (the distroless `nonroot` user); explicit `USER` in the Dockerfile.
- **Minimal:** distroless `cc` base — no shell, package manager, or build tools in the final image.
- **No build leakage:** the Rust/wasm toolchain, source, and `target/` exist only in the discarded builder stage.
- **Signals:** `chirp-server` is PID 1 (exec-form `ENTRYPOINT`) and drains in-flight requests on `SIGTERM`.
- **Reproducible:** base images digest-pinned; `--locked` Cargo builds; `wasm-bindgen-cli` version-pinned.

The measured image size and build/run evidence are recorded in the work-item's `SYNTHESIS.md`.
