# ferric-http

A small **async HTTP/JSON client** for the browser, built on the Fetch API — the networking
companion to the [`ferric`](../ferric) WebAssembly framework.

It gives a WASM app a typed, ergonomic way to talk to a JSON backend: configure a `Client`,
build a request fluently, and `await` a typed result. Request and response bodies are
(de)serialized with `serde`; every failure is one matchable `Error`.

It is deliberately lean (`web-sys`, `js-sys`, `wasm-bindgen`, `wasm-bindgen-futures`, `serde`,
`serde_json` — nothing else) and **usable on its own**: it does not depend on the `ferric`
view framework, though it is designed to sit beside it so a `ferric` SPA can call a REST
backend (for example a `chirp-server`).

## Usage

```rust
use ferric_http::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct NewUser<'a> {
    name: &'a str,
}

#[derive(Deserialize)]
struct User {
    id: u64,
    name: String,
}

async fn example() -> ferric_http::Result<()> {
    let client = Client::builder()
        .base_url("https://api.example.com")
        .default_header("Authorization", "Bearer token")
        .build();

    // GET with query params → typed JSON (non-2xx becomes an `Error::Status`).
    let users: Vec<User> = client
        .get("/users")
        .query(&[("page", "1"), ("limit", "20")])
        .send_json()
        .await?;

    // POST a JSON body → typed JSON response.
    let created: User = client
        .post("/users")
        .json(&NewUser { name: "ada" })?
        .send_json()
        .await?;

    // Inspect a raw response yourself when you don't want the status to short-circuit.
    let resp = client.delete("/users/1").send().await?;
    if resp.ok() {
        // deleted
    }

    let _ = (users, created);
    Ok(())
}
```

## Architecture

```
  Client ──get/post/put/delete──► RequestBuilder ──.query/.header/.json──► RequestBuilder
  (base URL + default headers)                                                   │
                                                                   .send() / .send_json()
                                                                                │
                                  web_sys::fetch(Request)  ◄────────────────────┘
                                              │
                                              ▼
                            Response (status + headers + body String)
                                 .json::<T>() / .error_for_status()
```

- **`Client` / `ClientBuilder`** — hold a base URL and default headers. A client is cheap to
  clone and holds no connection state (the browser owns the connection pool), so one client
  can be shared across an app.
- **`RequestBuilder`** — layer on query parameters, headers, and a JSON (or raw) body. URL
  resolution (base + path join, `?`/`&` handling) and RFC 3986 query percent-encoding are
  **pure functions**, unit-tested natively without a browser.
- **`Response`** — a completed response as owned data (status, headers, body `String`). Because
  it owns its body, `json::<T>()`, `text()`, and `error_for_status()` are synchronous.
- **`Error` / `Result`** — one error type distinguishing **builder** (bad URL/header),
  **network** (fetch failed, no response), non-`2xx` **status**, and **(de)serialization**
  failures. No fallible path panics; `send()` returns the `Response` even for non-`2xx` so the
  caller decides, while `send_json()` is safe-by-default (non-`2xx` → `Error::Status`).

Only `RequestBuilder::send` / `send_json` touch the browser (via `web-sys` + the Fetch API);
everything else is plain Rust.

## Testing

The pure logic — URL/query building, header assembly, error mapping and `Display`, and serde
round-trips — is covered by native `cargo test` (no browser needed). A `wasm-bindgen-test`
smoke test (`src/lib.rs`, gated on `cfg(target_arch = "wasm32")`) proves the logic also runs
under `wasm32`; run it with a headless browser:

```
wasm-pack test --headless --firefox crates/ferric-http
```

```
cargo test -p ferric-http                               # native unit + doc tests
cargo build -p ferric-http --target wasm32-unknown-unknown   # wasm build
```
