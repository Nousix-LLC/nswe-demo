#!/usr/bin/env bash
# install-k3s.sh — reproducible, idempotent k3s cluster stand-up for the nswe-demo deployment environment.
#
# PURPOSE
#   Stand up THE deployment cluster (a single logical k3s cluster) that the engagement's
#   continuous delivery (ArgoCD, app workloads — separate work items) will run on.
#
# PREREQUISITES
#   Common:   bash, curl, kubectl (client). Pinned k3s version: v1.35.5+k3s1.
#   NATIVE mode (production hosts):   root (or passwordless sudo) + systemd + cgroup v2.
#                                     Installs the k3s binary + a systemd unit via https://get.k3s.io.
#   K3D mode (rootless / dev / CI / sandbox):   a usable Docker daemon (member of the `docker`
#                                     group is sufficient) + the `k3d` CLI (>= v5.9.0). Runs real
#                                     k3s inside Docker containers; needs NO host root.
#
#   The script auto-selects a mode (override with K3S_INSTALL_MODE=native|k3d). It prefers NATIVE
#   when root+systemd are available, else falls back to K3D when Docker+k3d are available. Both
#   modes produce a genuine k3s cluster; `verify-cluster.sh` proves readiness identically for both.
#
# RE-RUN BEHAVIOUR (idempotent where feasible)
#   NATIVE: the upstream get.k3s.io installer is safe to re-run; re-running reconciles the unit and
#           re-applies the pinned version without destroying an existing node.
#   K3D:    the cluster is created ONLY if a cluster of the same name does not already exist
#           (create-if-missing). Re-running against an existing cluster is a no-op that still
#           (re)writes/merges the kubeconfig context. Use K3S_FORCE_RECREATE=1 to delete+recreate.
#
# NO SECRETS: this script neither prints nor persists any kubeconfig/token/password. Credential
#   RETRIEVAL is documented in README.md (the runbook fragment) — never committed.
set -euo pipefail

# ---- pinned configuration (reproducibility) --------------------------------------------------
K3S_VERSION="${K3S_VERSION:-v1.35.5+k3s1}"              # native installer tag (INSTALL_K3S_VERSION)
K3D_IMAGE="${K3D_IMAGE:-docker.io/rancher/k3s:v1.35.5-k3s1}"  # k3d node image (same k3s version)
CLUSTER_NAME="${K3S_CLUSTER_NAME:-nswe}"               # k3d cluster name -> kube context k3d-<name>
K3S_INSTALL_MODE="${K3S_INSTALL_MODE:-auto}"           # auto | native | k3d
K3S_FORCE_RECREATE="${K3S_FORCE_RECREATE:-0}"          # k3d only: 1 = delete+recreate

log() { printf '[install-k3s] %s\n' "$*" >&2; }
have() { command -v "$1" >/dev/null 2>&1; }

can_native() {
  local pid1; pid1="$(cat /proc/1/comm 2>/dev/null || true)"
  [ "$pid1" = "systemd" ] || return 1
  if [ "$(id -u)" = "0" ]; then return 0; fi
  sudo -n true >/dev/null 2>&1 && return 0
  return 1
}
can_k3d() { have k3d && have docker && docker info >/dev/null 2>&1; }

select_mode() {
  case "$K3S_INSTALL_MODE" in
    native) echo native ;;
    k3d)    echo k3d ;;
    auto)   if can_native; then echo native; elif can_k3d; then echo k3d; else echo none; fi ;;
    *)      log "unknown K3S_INSTALL_MODE='$K3S_INSTALL_MODE'"; echo none ;;
  esac
}

install_native() {
  log "NATIVE mode: installing k3s $K3S_VERSION via get.k3s.io (root/systemd)"
  local sh_pfx=""; [ "$(id -u)" = "0" ] || sh_pfx="sudo"
  # --write-kubeconfig-mode 0644 lets a non-root client read /etc/rancher/k3s/k3s.yaml; adjust to taste.
  curl -sfL https://get.k3s.io | \
    INSTALL_K3S_VERSION="$K3S_VERSION" \
    $sh_pfx sh -s - server --write-kubeconfig-mode 0644
  log "NATIVE kubeconfig written to /etc/rancher/k3s/k3s.yaml (see README.md for client retrieval)"
}

install_k3d() {
  have k3d   || { log "FATAL: k3d not found"; return 1; }
  docker info >/dev/null 2>&1 || { log "FATAL: docker daemon not usable"; return 1; }
  if k3d cluster list -o json 2>/dev/null | grep -q "\"name\":\"${CLUSTER_NAME}\""; then
    if [ "$K3S_FORCE_RECREATE" = "1" ]; then
      log "K3D mode: cluster '${CLUSTER_NAME}' exists; K3S_FORCE_RECREATE=1 -> deleting"
      k3d cluster delete "$CLUSTER_NAME"
    else
      log "K3D mode: cluster '${CLUSTER_NAME}' already exists -> idempotent no-op (merging kubeconfig)"
      k3d kubeconfig merge "$CLUSTER_NAME" --kubeconfig-merge-default --kubeconfig-switch-context >/dev/null
      return 0
    fi
  fi
  log "K3D mode: creating k3s cluster '${CLUSTER_NAME}' from image ${K3D_IMAGE}"
  k3d cluster create "$CLUSTER_NAME" \
    --image "$K3D_IMAGE" \
    --servers 1 --agents 1 \
    --wait --timeout 180s
  log "K3D kubeconfig merged into default kubeconfig; context = k3d-${CLUSTER_NAME}"
}

main() {
  local mode; mode="$(select_mode)"
  case "$mode" in
    native) install_native ;;
    k3d)    install_k3d ;;
    none)
      log "FATAL: no viable install mode."
      log "  NATIVE needs root/sudo + systemd; K3D needs a usable Docker daemon + the k3d CLI."
      log "  See README.md 'Blocker / remediation' for how to enable one of them."
      exit 3 ;;
  esac
  log "install complete (mode=$mode). Run ./verify-cluster.sh to prove readiness."
}
main "$@"
