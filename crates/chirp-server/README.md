# chirp-server

The **axum REST API backend** for **chirp**, the Twitter-style application in the `nswe-demo`
engagement (work-item #8). It serves the core social surface — users, sessions, chirps, a home
timeline, follows, and likes — over the shared [`chirp-types`](../chirp-types) wire contract, with
persistence hidden behind a repository trait (in-memory by default), static/SPA asset serving, a
health probe, and structured tracing.

The crate is **library + binary**:

- the **library** (`lib.rs`) exposes the composition root [`build_router`] plus `AppState` and the
  repository, so the binary and the end-to-end tests construct the *same* app through one path;
- the **binary** (`main.rs`) wires tracing, builds the state and router, and serves over TCP.

## Quick start

```sh
# From the workspace root. Boots on 127.0.0.1:3000 with the in-memory store — no database required.
cargo run -p chirp-server

# In another shell: create a user, open a session, post a chirp, read the timeline.
curl -s -XPOST localhost:3000/api/users \
  -H 'content-type: application/json' \
  -d '{"username":"alice","displayName":"Alice","bio":null}'

TOKEN=$(curl -s -XPOST localhost:3000/api/sessions \
  -H 'content-type: application/json' \
  -d '{"username":"alice"}' | jq -r .token)

curl -s -XPOST localhost:3000/api/chirps \
  -H "authorization: Bearer $TOKEN" -H 'content-type: application/json' \
  -d '{"text":"hello, chirp!","replyTo":null}'

curl -s localhost:3000/api/timeline -H "authorization: Bearer $TOKEN"

curl -s localhost:3000/healthz
```

### Configuration

The server reads three optional environment variables:

| Variable | Default | Purpose |
|----------|---------|---------|
| `CHIRP_BIND_ADDR` | `127.0.0.1:3000` | `host:port` the server binds. |
| `CHIRP_STATIC_DIR` | crate-local `static/` | Static-asset root served under the SPA fallback. |
| `RUST_LOG` | `info,chirp_server=debug,tower_http=info` | `tracing` env-filter directive. |

Graceful shutdown drains in-flight requests on `Ctrl-C`.

## API

All request and response bodies are JSON built from the `chirp-types` DTOs (field names are
**camelCase**; ids and tokens serialize as their bare inner value). Endpoints marked **Auth** require
an `Authorization: Bearer <token>` header (obtained from `POST /api/sessions`); a missing or invalid
token is rejected with **401** before the handler runs. The nine API routes are namespaced under
`/api/…` so they never collide with the static/SPA path space.

| Method & path | Auth | Request | Response | Success |
|---------------|------|---------|----------|---------|
| `POST /api/users` | — | `CreateUserRequest` | `User` | **201** |
| `GET /api/users/{id}` | — | — | `User` | **200** |
| `POST /api/sessions` | — | `LoginRequest` | `AuthResponse` | **200** |
| `POST /api/chirps` | ✅ | `CreateChirpRequest` | `Chirp` | **201** |
| `GET /api/timeline` | ✅ | `TimelineQuery` (query string) | `Page<Chirp>` | **200** |
| `PUT /api/users/{id}/follow` | ✅ | — | `FollowResponse` | **200** |
| `DELETE /api/users/{id}/follow` | ✅ | — | `FollowResponse` | **200** |
| `PUT /api/chirps/{id}/like` | ✅ | — | `LikeResponse` | **200** |
| `DELETE /api/chirps/{id}/like` | ✅ | — | `LikeResponse` | **200** |
| `GET /healthz` | — | — | `{status, service, version}` | **200** |

Notes:

- **Sessions.** Authentication is handle-based and deliberately light for the demo: `POST
  /api/sessions` with a known `username` returns an `AuthResponse` carrying the profile and an opaque
  bearer token. There is no password exchange.
- **Timeline** is the caller's **home** timeline — their own chirps plus those of everyone they
  follow — newest-first. `limit` (default 20, clamped `1..=100`) and an opaque `cursor` page it;
  echo `nextCursor` back on the next request until it is `null`.
- **Follow/like** are **idempotent**: `PUT` applies the state, `DELETE` removes it, and repeating a
  call does not double-count. The actor is always the authenticated caller, never taken from the
  body, so a caller can only act as itself (self-follow is rejected with **403**).

### Errors

Every failure surfaces as the `chirp-types` `ApiError { code, message, details }` wire type with the
HTTP status fixed by the contract:

| Condition | `ErrorCode` | HTTP |
|-----------|-------------|------|
| Missing/invalid session token | `UNAUTHORIZED` | 401 |
| Not permitted (e.g. self-follow) | `FORBIDDEN` | 403 |
| Addressed user/chirp absent | `NOT_FOUND` | 404 |
| State conflict (duplicate username) | `CONFLICT` | 409 |
| Unexpected server fault | `INTERNAL` | 500 |

> **Known gap (DTO validation).** A body that is well-formed JSON but carries an invalid field value
> (e.g. a username with a space, or over-length chirp text) is currently rejected by axum's default
> JSON extractor as **422 Unprocessable Entity** with a plain-text body, rather than the contract's
> intended **400 `VALIDATION_ERROR`** `ApiError`. The input is still safely rejected; only the error
> *shape* differs. See `TEST_NOTES.md` (finding F1) for the recommended fix (a custom JSON extractor
> that maps a deserialization failure to `ServerError::Validation`).

## Persistence: the repository trait

The server depends only on a persistence **capability**, the `ChirpRepository` trait, held by the
transport layer as `Arc<dyn ChirpRepository>` inside `AppState`. Every handler `.await`s the trait;
no handler names a concrete store.

```text
transport (axum handlers, router, middleware)
    │  depends on
    ▼
ChirpRepository  (trait — the persistence seam)
    ▲  implemented by
    │
InMemoryRepository   ← default backing (no database)
```

**Why in-memory is the default.** The demo must boot and `cargo test` must run with **no database and
no external infrastructure**. `InMemoryRepository` keeps all state in process behind a single
`Mutex`, so the server is runnable and the whole suite is hermetic. It also keeps the seam honest: a
realistic replacement (an async database) is exactly what the `async` trait was shaped for.

**How to swap the backing store.** Add a second implementor of `ChirpRepository` (e.g. a
SQLite/`sqlx`-backed `SqlRepository`) and construct `AppState::new(Arc::new(SqlRepository::connect(...)?))`
in `main.rs`. No handler, route, or DTO changes — the transport layer is already generic over the
trait object. A persistent backing is an **optional, documented alternative**; it must never become a
`cargo test` prerequisite.

## Testing

```sh
cargo test -p chirp-server              # unit (repository + handlers) + end-to-end API integration
cargo fmt -p chirp-server --check
cargo clippy -p chirp-server --all-targets -- -D warnings
```

The end-to-end suite (`tests/api.rs`) mounts the real router via `build_router` against a fresh
`InMemoryRepository` and drives it in-process with `tower`'s `oneshot` — no network bind — asserting
both the HTTP status and the decoded `chirp-types` DTO for the full acceptance surface and the
error-mapping table. See `TEST_NOTES.md` for the inventory and acceptance-criteria coverage map.

## Status & scope

Implemented here (work-item #8): the REST API, the repository seam + in-memory backing, static/SPA
serving with a placeholder `static/index.html`, `GET /healthz`, tracing, and the end-to-end tests.
Out of scope: the real frontend SPA (#9 — only the serving path is wired) and deploy/CI (#10–#13).
