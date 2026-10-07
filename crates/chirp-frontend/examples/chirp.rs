//! The chirp SPA wasm entry point.
//!
//! This example is the browser binary: it calls [`chirp_frontend::start`], which builds the app
//! and mounts it into the page. Build it for wasm and generate the JS bindings as documented in
//! `crates/chirp-frontend/README.md`:
//!
//! ```text
//! cargo build -p chirp-frontend --example chirp --target wasm32-unknown-unknown
//! wasm-bindgen --target web --no-typescript \
//!     --out-dir crates/chirp-frontend/pkg \
//!     target/wasm32-unknown-unknown/debug/examples/chirp.wasm
//! ```
//!
//! Then serve `crates/chirp-frontend/index.html` (which imports `./pkg/chirp.js`) over HTTP.

fn main() {
    chirp_frontend::start();
}
