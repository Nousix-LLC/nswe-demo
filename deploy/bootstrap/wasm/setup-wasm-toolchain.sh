#!/usr/bin/env bash
#
# setup-wasm-toolchain.sh — enable the chirp-frontend WebAssembly build toolchain.
#
# WHAT IT DOES
#   1. Adds the `wasm32-unknown-unknown` rustc target.
#   2. Installs a `wasm-bindgen` CLI whose version EXACTLY matches the `wasm-bindgen`
#      crate the workspace resolves to (wasm-bindgen requires an exact CLI<->crate match;
#      a mismatch produces a confusing "schema version" error at bind time). The required
#      version is derived from Cargo.lock so this script stays correct if the crate is bumped.
#
# PREREQUISITES
#   - rustup + a Rust toolchain on PATH (rustc/cargo). MSRV for this workspace is 1.82.
#       Verified-against: rustc 1.97.1, cargo 1.97.1, rustup 1.29.0.
#   - Network access to static.rust-lang.org (target download) and crates.io
#       (wasm-bindgen-cli download/build). If offline, see the runbook's blocker notes.
#   - Run from anywhere inside the `nswe-demo` repo working tree (the script locates the
#       repo root itself).
#
# RE-RUN BEHAVIOUR (idempotent)
#   - `rustup target add` is a no-op if the target is already installed.
#   - The wasm-bindgen install is SKIPPED when the installed CLI already matches the
#       required version; otherwise it installs the pinned version with `--locked`.
#   Safe to run repeatedly.
#
# EXIT CODES: 0 success; non-zero with a diagnostic on the first failing step.

set -euo pipefail

log() { printf '[setup-wasm] %s\n' "$*"; }
die() { printf '[setup-wasm] ERROR: %s\n' "$*" >&2; exit 1; }

# --- locate the repo root (git first, then walk up for Cargo.lock) ---------------------
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
[ -n "$REPO_ROOT" ] && [ -f "$REPO_ROOT/Cargo.lock" ] || \
  die "could not locate repo root (no Cargo.lock found); run from inside the nswe-demo tree."
log "repo root: $REPO_ROOT"

# --- tool checks -----------------------------------------------------------------------
command -v rustup >/dev/null 2>&1 || die "rustup not found on PATH."
command -v cargo  >/dev/null 2>&1 || die "cargo not found on PATH."
log "rustc: $(rustc --version 2>/dev/null || echo '?')"
log "cargo: $(cargo --version 2>/dev/null || echo '?')"

# --- 1. wasm32 target ------------------------------------------------------------------
TARGET="wasm32-unknown-unknown"
if rustup target list --installed 2>/dev/null | grep -qx "$TARGET"; then
  log "target $TARGET already installed."
else
  log "adding target $TARGET ..."
  rustup target add "$TARGET"
fi

# --- 2. derive the required wasm-bindgen version from Cargo.lock -----------------------
# Pull the `version` line from the [[package]] block named exactly "wasm-bindgen".
REQ_VER="$(
  awk '
    $0=="name = \"wasm-bindgen\"" {inblk=1; next}
    inblk && /^version = "/ {
      gsub(/^version = "|"$/,""); print; exit
    }
    inblk && /^\[\[package\]\]/ {inblk=0}
  ' "$REPO_ROOT/Cargo.lock"
)"
[ -n "$REQ_VER" ] || die "could not read the resolved wasm-bindgen version from Cargo.lock."
log "workspace resolves wasm-bindgen crate = $REQ_VER (CLI must match exactly)."

# --- 3. install the version-matched CLI if needed --------------------------------------
HAVE_VER=""
if command -v wasm-bindgen >/dev/null 2>&1; then
  HAVE_VER="$(wasm-bindgen --version 2>/dev/null | awk '{print $2}')"
fi
if [ "$HAVE_VER" = "$REQ_VER" ]; then
  log "wasm-bindgen $HAVE_VER already installed and matches — nothing to do."
else
  [ -n "$HAVE_VER" ] && log "installed wasm-bindgen ($HAVE_VER) != required ($REQ_VER); reinstalling."
  log "installing wasm-bindgen-cli $REQ_VER (compiles from source; may take a few minutes) ..."
  cargo install wasm-bindgen-cli --version "$REQ_VER" --locked
fi

# --- verification ----------------------------------------------------------------------
INSTALLED="$(wasm-bindgen --version 2>/dev/null | awk '{print $2}')"
[ "$INSTALLED" = "$REQ_VER" ] || die "post-install check failed: wasm-bindgen is '$INSTALLED', expected '$REQ_VER'."
log "OK: target=$TARGET present; wasm-bindgen=$INSTALLED (matches crate)."
log "next: ./build-chirp-wasm.sh"
