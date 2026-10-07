# Bootstrap: WebAssembly build toolchain

Runbook fragment for enabling and proving the `chirp-frontend` WebAssembly build. This is the
**build-toolchain** concern of the infra-bootstrap (#10) environment setup; it is independent of the
k3s cluster and ArgoCD fragments (no ordering dependency).

> **Scope note:** this proves the toolchain can build the existing `crates/chirp-frontend` to a
> runnable `pkg/` bundle. It does **not** containerize the app (#12) or author deploy manifests (#13).

## What this produces

The crate is a client-side WASM SPA that is built in two steps (the path the crate's own
`README.md` documents — the repo uses **wasm-bindgen**, not trunk):

1. `cargo build … --example chirp --target wasm32-unknown-unknown` → a raw `chirp.wasm`.
2. `wasm-bindgen --target web` → `pkg/chirp.js` + `pkg/chirp_bg.wasm`, the ES-module bundle that
   `crates/chirp-frontend/index.html` imports via `import init from "./pkg/chirp.js"`.

## Prerequisites

- **rustup + Rust toolchain** on PATH. Workspace MSRV is **1.82**; verified against rustc/cargo
  **1.97.1**, rustup **1.29.0**.
- **Network** to `static.rust-lang.org` (rustup target) and `crates.io` (the `wasm-bindgen-cli`
  build). The `wasm-bindgen` CLI version **must exactly match** the `wasm-bindgen` *crate* the
  workspace resolves to (currently **0.2.129**) — a mismatch fails at bind time with a schema-version
  error. `setup-wasm-toolchain.sh` derives the required version from `Cargo.lock` automatically.
- Run from inside the `nswe-demo` working tree (both scripts locate the repo root themselves).

## Run order

```sh
# from the repo root (or anywhere inside the working tree)
deploy/bootstrap/wasm/setup-wasm-toolchain.sh     # 1. add wasm32 target + version-matched wasm-bindgen (idempotent)
deploy/bootstrap/wasm/build-chirp-wasm.sh         # 2. build chirp-frontend to pkg/ and verify artifacts
```

Both scripts are safe to re-run. `setup-…` is a no-op when the target and a matching CLI are already
present; `build-…` is incremental and overwrites `pkg/` each run. By default `build-…` emits to
`crates/chirp-frontend/pkg` (what `index.html` imports); override with `OUT_DIR=<dir>` to emit a
proof bundle elsewhere.

## Verification

`build-chirp-wasm.sh` asserts success itself and exits non-zero on any failure. A green run prints:

```
[build-wasm] PASS: produced chirp.js (…bytes) + chirp_bg.wasm (…bytes); wasm magic OK.
```

It checks that both `pkg/chirp.js` and `pkg/chirp_bg.wasm` exist and that the `.wasm` starts with the
WebAssembly magic header (`\0asm` = `0061736d`). Optional manual smoke test in a browser:

```sh
python3 -m http.server --directory crates/chirp-frontend 8080
# open http://localhost:8080/index.html  (the SPA mounts into #app)
```

## Recorded toolchain versions

See `versions.txt` in this directory for the exact versions this fragment was proven against.

## If blocked

If the host has no network to add the target or build the CLI, the scripts fail fast with the exact
failing command. Remediation: on a connected host run `rustup target add wasm32-unknown-unknown` and
`cargo install wasm-bindgen-cli --version <crate-version-from-Cargo.lock> --locked`, or vendor the
`wasm-bindgen-cli` binary at that version into a PATH dir. The required version is always the
`wasm-bindgen` package version in the repo's `Cargo.lock`.
