# chirp-frontend

The **chirp** Twitter-style client: a client-side WebAssembly single-page app built on the
[`ferric`](../ferric) framework and [`ferric-router`](../ferric-router), talking to the chirp REST
backend through [`ferric-http`](../ferric-http) and the shared [`chirp-types`](../chirp-types) wire
contract.

This crate composes the landed framework crates; it does not re-implement any of them, and it uses
the `chirp-types` DTOs directly as the wire contract (never redefining them).

## Layout

| Module | Responsibility |
|--------|----------------|
| `src/app.rs` | Composition root: `Session`, `AppContext`, the router route table, the nav, and the `start()` mount entry. |
| `src/error.rs` | `AppError` — the single UI error type; decodes a server `ApiError` out of a non-2xx response. |
| `src/api.rs` | `ApiClient` — the typed REST client over `ferric-http` (the feature spoke fills in the bodies). |
| `src/timeline.rs` · `src/compose.rs` · `src/profile.rs` | The three route views (filled in by their owning feature spokes). |
| `examples/chirp.rs` | The wasm entry binary — calls `chirp_frontend::start()`. |
| `index.html` | A static harness with the `#app` mount point and the wasm/JS glue. |

The in-crate public surface the feature spokes compose against is frozen in
`../../contracts/app_api.rs`; this crate realizes those signatures verbatim.

## Building for the browser

The library target builds for `wasm32-unknown-unknown`:

```sh
rustup target add wasm32-unknown-unknown            # once
cargo build -p chirp-frontend --target wasm32-unknown-unknown
```

To produce a runnable page, build the example binary and generate the JS bindings with
[`wasm-bindgen`](https://rustwasm.github.io/wasm-bindgen/):

```sh
cargo build -p chirp-frontend --example chirp --target wasm32-unknown-unknown
wasm-bindgen --target web --no-typescript \
    --out-dir crates/chirp-frontend/pkg \
    target/wasm32-unknown-unknown/debug/examples/chirp.wasm
```

Then serve the crate directory over HTTP (wasm modules must be served, not opened as `file://`)
and open `index.html`, which imports `./pkg/chirp.js`:

```sh
python3 -m http.server --directory crates/chirp-frontend 8080
# open http://localhost:8080/index.html
```

(Alternatively, [`trunk serve`](https://trunkrs.dev/) can build and serve `index.html` directly.)

## Configuring the API base URL

The client defaults to the relative base **`/api`**, so the SPA calls the same origin that served
it — no host/port is baked in. Override it at build time with the `CHIRP_API_BASE_URL` environment
variable, which `start()` reads via `option_env!`:

```sh
CHIRP_API_BASE_URL=https://api.example.com \
    cargo build -p chirp-frontend --example chirp --target wasm32-unknown-unknown
```

Programmatically, any `AppContext` can be built for a specific base with
`AppContext::new("https://api.example.com")`.

## Quality gates

```sh
cargo fmt --all --check
cargo clippy -p chirp-frontend --all-targets -- -D warnings
cargo test -p chirp-frontend
```
