#!/usr/bin/env bash
# verify-cluster.sh — real readiness proof for the k3s deployment cluster.
#
# PURPOSE
#   Prove (not assume) that the k3s cluster stood up by install-k3s.sh is actually usable:
#     1. at least one node reports STATUS=Ready,
#     2. core kube-system workloads (CoreDNS at minimum) are Running/Completed,
#     3. the recorded k3s server version is reported.
#   Exits non-zero if the cluster is not ready, so CI / the synthesis gate can gate on it.
#
# PREREQUISITES
#   kubectl (client) on PATH and a kubeconfig context pointing at the cluster. By default this
#   script targets the context named by KUBE_CONTEXT (default: k3d-nswe, the k3d context created
#   by install-k3s.sh in K3D mode). For a NATIVE install, point kubectl at
#   /etc/rancher/k3s/k3s.yaml and set KUBE_CONTEXT=default (see README.md).
#
# RE-RUN BEHAVIOUR
#   Read-only; safe to run any number of times. Makes no changes to the cluster.
#
# NO SECRETS: reads cluster state only; prints no credentials.
set -euo pipefail

KUBE_CONTEXT="${KUBE_CONTEXT:-k3d-nswe}"
READY_TIMEOUT="${READY_TIMEOUT:-120s}"

kc() { kubectl --context "$KUBE_CONTEXT" "$@"; }
log() { printf '[verify-cluster] %s\n' "$*" >&2; }

log "context=$KUBE_CONTEXT"

# 1. API reachable?
if ! kc version -o json >/dev/null 2>&1; then
  log "FAIL: kube API not reachable via context '$KUBE_CONTEXT'."
  log "      Check the context exists (kubectl config get-contexts) and the cluster is running."
  exit 1
fi

# 2. Wait for every node to reach Ready, then show them.
log "waiting up to $READY_TIMEOUT for all nodes to be Ready ..."
if ! kc wait --for=condition=Ready nodes --all --timeout="$READY_TIMEOUT" >/dev/null 2>&1; then
  log "FAIL: not all nodes reached Ready within $READY_TIMEOUT."
  kc get nodes -o wide || true
  exit 1
fi
echo "---- nodes ----"
kc get nodes -o wide

# 3. k3s version (from the Ready node's kubelet / node info).
K3S_VER="$(kc get nodes -o jsonpath='{.items[0].status.nodeInfo.osImage}' 2>/dev/null || true)"
echo "k3s server version (node osImage): ${K3S_VER:-unknown}"

# 4. Core system workloads healthy (CoreDNS is the hard requirement; others informational).
echo "---- kube-system workloads ----"
kc get pods -n kube-system
if ! kc get pods -n kube-system -l k8s-app=kube-dns -o jsonpath='{.items[*].status.phase}' 2>/dev/null | grep -q Running; then
  # k3s labels CoreDNS k8s-app=kube-dns; fall back to a name match if the label differs.
  if ! kc get pods -n kube-system 2>/dev/null | grep -Eiq 'coredns.*Running'; then
    log "FAIL: CoreDNS is not Running in kube-system."
    exit 1
  fi
fi

log "PASS: cluster is Ready and core system workloads are healthy."
