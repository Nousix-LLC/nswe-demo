# Changelog — ferric-http

All notable changes to `ferric-http` are documented here.
Versioning follows the workspace `version.workspace = true` setting.

---

## Unreleased — usable_early surface (#148)

### Verification: all usable_early acceptance criteria met

The `usable_early` scope (issue **#148**, traces to REQ-HTTP-001–008, 013) is fully satisfied
by the codebase on `main`. No new functional code was required; this entry records the formal
closure of #148 and declares the frozen public surface.

#### Public API surface (frozen at usable_early)

The following types and functions are exported from `ferric_http::prelude`:

**`Client`** — HTTP client holding a base URL and default headers.

```rust
pub struct Client { ... }

impl Client {
    pub fn new() -> Self;
    pub fn builder() -> ClientBuilder;
    pub fn request(&self, method: Method, path: &str) -> RequestBuilder;
    pub fn get(&self, path: &str) -> RequestBuilder;
    pub fn post(&self, path: &str) -> RequestBuilder;
    pub fn put(&self, path: &str) -> RequestBuilder;
    pub fn delete(&self, path: &str) -> RequestBuilder;
    pub fn base_url(&self) -> Option<&str>;
}
```

**`ClientBuilder`** — Fluent builder for `Client`.

```rust
pub struct ClientBuilder { ... }

impl ClientBuilder {
    pub fn base_url(self, url: impl Into<String>) -> Self;
    pub fn default_header(self, name: &str, value: &str) -> Self;
    pub fn build(self) -> Client;
}
```

**`RequestBuilder`** — Fluent request builder; created by `Client::get()`/`post()`/etc.

```rust
pub struct RequestBuilder { ... }

impl RequestBuilder {
    /// Append query-string parameters (RFC-3986 percent-encoded); accumulates across calls.
    pub fn query(self, params: &[(&str, &str)]) -> Self;
    /// Append a per-request header (same-named headers accumulate; Fetch API combines them).
    pub fn header(self, name: &str, value: &str) -> Self;
    /// Serialize `body` as JSON and set Content-Type: application/json.
    pub fn json<B: Serialize + ?Sized>(self, body: &B) -> Result<Self>;
    /// Set a raw string request body; caller is responsible for any `Content-Type` header.
    pub fn body_string(self, body: impl Into<String>) -> Self;
    /// Send the request; non-2xx responses are NOT an error (caller inspects).
    pub async fn send(self) -> Result<Response>;
    /// Send and deserialize the response body as `T`; non-2xx → `Err(Error::Status)`.
    pub async fn send_json<T: DeserializeOwned>(self) -> Result<T>;
}
```

**`Response`** — Completed HTTP exchange as plain owned data.

```rust
pub struct Response { ... }

impl Response {
    pub fn status(&self) -> u16;
    pub fn status_text(&self) -> &str;
    pub fn ok(&self) -> bool;
    pub fn url(&self) -> &str;
    pub fn headers(&self) -> &[(String, String)];
    pub fn header(&self, name: &str) -> Option<&str>;
    pub fn body(&self) -> &str;
    pub fn json<T: DeserializeOwned>(&self) -> Result<T>;
    pub fn text(self) -> String;
    pub fn error_for_status(self) -> Result<Self>;
}
```

**`Error`** — Matchable error enum; `#[non_exhaustive]`.

```rust
#[non_exhaustive]
pub enum Error {
    Builder(String),
    Network(String),
    Status { status: u16, status_text: String, url: String, body: Option<String> },
    Serialize(serde_json::Error),
    Deserialize(serde_json::Error),
    Body(String),
}

impl Error {
    pub fn is_network(&self) -> bool;
    pub fn is_status(&self) -> bool;
    pub fn status(&self) -> Option<u16>;
}
```

**`Method`** — HTTP verb.

```rust
pub enum Method { Get, Post, Put, Delete }

impl Method {
    /// The uppercase HTTP token (e.g. `"GET"`).
    pub fn as_str(&self) -> &'static str;
}

impl std::fmt::Display for Method { /* delegates to as_str() */ }
```

**`type Result<T> = std::result::Result<T, Error>`**

#### Acceptance criteria (issue #148)

| Criterion | Status |
|-----------|--------|
| `Client` accepts a base URL at configuration time | ✅ `ClientBuilder::base_url()` |
| `RequestBuilder` chains URL resolution per RFC 3986: GET, POST, PUT, DELETE | ✅ `Client::{get,post,put,delete}` + `resolve_url()` |
| Typed `body::<T>()` serializes `T` to JSON (Content-Type: application/json) | ✅ `RequestBuilder::json()` |
| Typed `send::<T>()` deserializes response body as `T` | ✅ `RequestBuilder::send_json::<T>()` |
| Matchable `Error` enum: network failure, non-2xx status, deserialization failure | ✅ `Error::{Network,Status,Deserialize,...}` |
| Non-2xx responses surface as `Err` by default | ✅ `send_json()` → `Err(Error::Status)` |
| Request headers can be set per-request | ✅ `RequestBuilder::header()` |
| All #6 baseline prelude surfaces preserved (no removal or redefinition) | ✅ frozen surface intact |
| Stated HTTP client examples compile | ✅ doctests in `src/lib.rs` pass |
| REQ-HTTP-011: no `ferric` dependency; `send_json()` driven from async context | ✅ `cargo tree -p ferric-http` clean; `chirp-frontend/src/api.rs` drives it |

#### Not in scope (deferred to issue #149)

- `PATCH` method (REQ-HTTP-009); when added in #149, `Method` will gain `#[non_exhaustive]`
- Per-request timeout (REQ-HTTP-010)
- Lean-footprint audit (BAR-FERRIC-021)

---

## Initial implementation (#6)

The initial crate was introduced as a single atomic work-item in issue **#6**:

- `src/error.rs` — `Error` enum + `Result` type alias
- `src/request.rs` — `Method`, fluent `RequestBuilder`, RFC-3986 URL resolution +
  percent-encoding, `web-sys` Fetch integration
- `src/response.rs` — `Response` as plain owned data
- `src/client.rs` — `Client` + `ClientBuilder`
- `src/lib.rs` — crate docs, re-exports, `ferric_http::prelude`, wasm32 smoke test
- `Cargo.toml`, `README.md`

Dependencies: `web-sys`, `js-sys`, `wasm-bindgen`, `wasm-bindgen-futures`, `serde`,
`serde_json`. Standalone — no dependency on the `ferric` view framework.
