//! `ferric-http` — a small async HTTP/JSON client for the browser, built on the Fetch API.
//!
//! It gives a WebAssembly app a typed, ergonomic way to talk to a JSON backend: configure a
//! [`Client`], build a request with a fluent [`RequestBuilder`], and `await` a typed result.
//! Bodies are (de)serialized with `serde`; failures are a single, matchable [`Error`].
//!
//! It is deliberately lean (`web-sys`, `js-sys`, `wasm-bindgen`, `wasm-bindgen-futures`,
//! `serde`, `serde_json` — nothing else) and is usable on its own; it does **not** depend on
//! the `ferric` view framework, though it is designed to sit beside it so a `ferric` SPA can
//! call a REST backend.
//!
//! # Architecture
//!
//! ```text
//!   Client ──get/post/put/delete──► RequestBuilder ──.query/.header/.json──► RequestBuilder
//!   (base URL + default headers)                                                   │
//!                                                                      .send() / .send_json()
//!                                                                                   │
//!                                     web_sys::fetch(Request)  ◄───────────────────┘
//!                                                 │
//!                                                 ▼
//!                               Response (status + headers + body String)
//!                                    .json::<T>() / .error_for_status()
//! ```
//!
//! * [`Client`] / [`ClientBuilder`] — base URL and default headers; one client is shareable
//!   and cheap to clone.
//! * [`RequestBuilder`] — layer on query parameters, headers, and a JSON body; URL
//!   resolution and percent-encoding are pure and unit-tested natively.
//! * [`Response`] — a completed response as owned data, so [`Response::json`] and
//!   [`Response::error_for_status`] are synchronous.
//! * [`Error`] / [`Result`] — one error type distinguishing builder, network, non-`2xx`
//!   status, and (de)serialization failures. No fallible path panics.
//!
//! Only [`RequestBuilder::send`] (and [`send_json`](RequestBuilder::send_json)) touch the
//! browser; everything else is plain Rust.
//!
//! # Example
//!
//! ```no_run
//! use ferric_http::Client;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Serialize)]
//! struct NewUser<'a> {
//!     name: &'a str,
//! }
//!
//! #[derive(Deserialize)]
//! struct User {
//!     id: u64,
//!     name: String,
//! }
//!
//! async fn create(client: &Client) -> ferric_http::Result<User> {
//!     // POST a JSON body, get a typed JSON response; a non-2xx status becomes an error.
//!     client
//!         .post("/users")
//!         .json(&NewUser { name: "ada" })?
//!         .send_json()
//!         .await
//! }
//! # let _ = create;
//! ```

mod client;
mod error;
mod request;
mod response;

pub use client::{Client, ClientBuilder};
pub use error::{Error, Result};
pub use request::{Method, RequestBuilder};
pub use response::Response;

/// The crate's common imports in one `use`.
///
/// ```
/// use ferric_http::prelude::*;
/// let _client = Client::new();
/// ```
///
/// Note that this re-exports [`Result`](crate::Result), which shadows [`std::result::Result`]
/// in the importing scope — the usual trade-off for a client-library prelude.
pub mod prelude {
    pub use crate::{Client, ClientBuilder, Error, Method, RequestBuilder, Response, Result};
}

// A browser-only smoke test proving the crate's logic also runs under `wasm32`. It is
// compiled only for `cargo test --target wasm32-unknown-unknown` and needs a headless
// browser runner (e.g. `wasm-pack test --headless`) to execute; where none is available the
// native `cargo test` suite is the gate. It performs no network I/O, so it is deterministic
// wherever it runs.
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use crate::Client;
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn url_building_runs_in_browser() {
        let client = Client::builder()
            .base_url("https://api.example.com")
            .build();
        let url = client.get("/users").query(&[("q", "a b")]).url();
        assert_eq!(url, "https://api.example.com/users?q=a%20b");
    }
}
