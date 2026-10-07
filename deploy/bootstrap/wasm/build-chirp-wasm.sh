#!/usr/bin/env bash
#
# build-chirp-wasm.sh — prove the WebAssembly build of crates/chirp-frontend.
#
# WHAT IT DOES (the exact path the crate's README already documents)
#   1. cargo build -p chirp-frontend --example chirp --target wasm32-unknown-unknown
#   2. wasm-bindgen --target web --no-typescript --out-dir crates/chirp-frontend/pkg \
#          target/wasm32-unknown-unknown/debug/examples/chirp.wasm
#   3. Verifies pkg/chirp.js + pkg/chirp_bg.wasm were produced and that the .wasm carries
#      the WebAssembly magic header (\0asm). Prints the exact toolchain versions used.
#
#   This does NOT modify chirp-frontend source. It produces build artifacts only:
#   target/ (cargo cache) and the generated pkg/ binding bundle index.html imports.
#
# PREREQUISITES
#   - ./setup-wasm-toolchain.sh has run (wasm32 target + version-matched wasm-bindgen).
#       This script re-asserts both and fails fast with a pointer if either is missing.
#   - Run from anywhere inside the nswe-demo repo working tree (it locates the root).
#
# RE-RUN BEHAVIOUR (idempotent)
#   - cargo build is incremental; wasm-bindgen overwrites pkg/ each run. Safe to re-run.
#   - OUT_DIR defaults to crates/chirp-frontend/pkg (what index.html imports). Override by
#       exporting OUT_DIR=/some/dir to emit the bundle elsewhere (e.g. a scratch proof dir).
#
# EXIT CODES: 0 success; non-zero with a diagnostic on the first failing step.

set -euo pipefail

log() { printf '[build-wasm] %s\n' "$*"; }
die() { printf '[build-wasm] ERROR: %s\n' "$*" >&2; exit 1; }

# --- locate the repo root --------------------------------------------------------------
REPO_ROOT=""
if command -v git >/dev/null 2>&1 && git rev-parse --show-toplevel >/dev/null 2>&1; then
  REPO_ROOT="$(git rev-parse --show-toplevel)"
else
  d="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  while [ "$d" != "/" ]; do
    [ -f "$d/Cargo.lock" ] && { REPO_ROOT="$d"; break; }
    d="$(dirname "$d")"
  done
fi
[ -n "$REPO_ROOT" ] && [ -d "$REPO_ROOT/crates/chirp-frontend" ] || \
  die "could not locate the nswe-demo repo root containing crates/chirp-frontend."
cd "$REPO_ROOT"
log "repo root: $REPO_ROOT"

TARGET="wasm32-unknown-unknown"
OUT_DIR="${OUT_DIR:-crates/chirp-frontend/pkg}"
RAW_WASM="target/$TARGET/debug/examples/chirp.wasm"

# --- prereq assertions -----------------------------------------------------------------
command -v cargo >/dev/null 2>&1 || die "cargo not found; see ./setup-wasm-toolchain.sh"
rustup target list --installed 2>/dev/null | grep -qx "$TARGET" || \
  die "target $TARGET missing; run ./setup-wasm-toolchain.sh first."
command -v wasm-bindgen >/dev/null 2>&1 || \
  die "wasm-bindgen CLI missing; run ./setup-wasm-toolchain.sh first."

# version-match re-assert (CLI must equal the resolved crate version)
REQ_VER="$(awk '
  $0=="name = \"wasm-bindgen\"" {inblk=1; next}
  inblk && /^version = "/ {gsub(/^version = "|"$/,""); print; exit}
  inblk && /^\[\[package\]\]/ {inblk=0}' "$REPO_ROOT/Cargo.lock")"
HAVE_VER="$(wasm-bindgen --version 2>/dev/null | awk '{print $2}')"
[ -n "$REQ_VER" ] && [ "$HAVE_VER" = "$REQ_VER" ] || \
  die "wasm-bindgen version mismatch (have '$HAVE_VER', crate needs '$REQ_VER'); run ./setup-wasm-toolchain.sh."

log "toolchain: $(rustc --version) | $(cargo --version) | wasm-bindgen $HAVE_VER (matches crate)"

# --- 1. build the example for wasm32 ---------------------------------------------------
log "building chirp example for $TARGET ..."
cargo build -p chirp-frontend --example chirp --target "$TARGET"
[ -f "$RAW_WASM" ] || die "expected raw wasm not produced at $RAW_WASM"
log "raw wasm: $RAW_WASM ($(wc -c < "$RAW_WASM") bytes)"

# --- 2. generate the JS/wasm bindings --------------------------------------------------
log "running wasm-bindgen --target web --no-typescript --out-dir $OUT_DIR ..."
mkdir -p "$OUT_DIR"
wasm-bindgen --target web --no-typescript --out-dir "$OUT_DIR" "$RAW_WASM"

# --- 3. verify the produced bundle -----------------------------------------------------
JS="$OUT_DIR/chirp.js"
BG="$OUT_DIR/chirp_bg.wasm"
[ -f "$JS" ] || die "wasm-bindgen did not produce $JS"
[ -f "$BG" ] || die "wasm-bindgen did not produce $BG"
# WebAssembly magic header: 0x00 'a' 's' 'm'
magic="$(head -c4 "$BG" | od -An -tx1 | tr -d ' ')"
[ "$magic" = "0061736d" ] || die "$BG is not a valid wasm module (magic=$magic, expected 0061736d)."
grep -q "as default" "$JS" || log "note: $JS has no 'as default' export line (index.html imports default init)."

log "PASS: produced $(basename "$JS") ($(wc -c < "$JS") bytes) + $(basename "$BG") ($(wc -c < "$BG") bytes); wasm magic OK."
log "serve for a manual smoke test:  python3 -m http.server --directory crates/chirp-frontend 8080  # then open /index.html"
