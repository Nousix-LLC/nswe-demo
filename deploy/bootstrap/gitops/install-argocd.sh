#!/usr/bin/env bash
# install-argocd.sh — reproducible, idempotent ArgoCD control-plane install (issue #10, infra-bootstrap)
#
# WHAT: Installs the ArgoCD GitOps control plane into a dedicated namespace from the UPSTREAM
#       declarative manifest, PINNED to a specific ArgoCD version, then waits for core workloads Ready.
#       This stands up the control plane ONLY. It does NOT create any ArgoCD `Application`, app
#       Deployment/Service manifests, or container images (those are issues #13 / #12 — out of scope).
#
# PREREQUISITES:
#   - A reachable Kubernetes cluster and a kubeconfig/context with cluster-admin (ArgoCD installs
#     CRDs + ClusterRoles). On this engagement the cluster is the k3s cluster from
#     deploy/bootstrap/cluster/ (see that runbook for kubeconfig retrieval); any conformant cluster works.
#   - kubectl >= 1.27 in PATH (server-side apply with --force-conflicts is relied upon; see WHY below).
#   - Network egress to raw.githubusercontent.com and quay.io (to fetch the manifest and pull images).
#
# RE-RUN BEHAVIOR (idempotent):
#   - Namespace creation uses `apply` of a dry-run-rendered manifest: safe to re-run (no error if it
#     already exists).
#   - The install uses `kubectl apply --server-side --force-conflicts`: re-running converges to the
#     pinned desired state and reports `serverside-applied`/`unchanged` with exit 0. No duplication.
#   - To upgrade/downgrade, change ARGOCD_VERSION and re-run.
#
# WHY SERVER-SIDE APPLY (not plain `kubectl apply`):
#   ArgoCD's `applicationsets.argoproj.io` CRD exceeds the 262144-byte client-side
#   last-applied-configuration annotation limit, so plain `kubectl apply -f install.yaml` FAILS with
#   `metadata.annotations: Too long`. `--server-side` avoids the annotation entirely. This is the
#   upstream-recommended install path for ArgoCD's large CRDs.
#
# USAGE:
#   ./install-argocd.sh                 # install pinned ARGOCD_VERSION into ARGOCD_NAMESPACE
#   ARGOCD_VERSION=v3.5.4 ./install-argocd.sh
#   ARGOCD_NAMESPACE=argocd ./install-argocd.sh
#   SKIP_WAIT=1 ./install-argocd.sh     # apply only; skip the readiness wait (verify separately)
#
# SECURITY: installs no secret. ArgoCD generates its own `argocd-initial-admin-secret` inside the
#   cluster at first boot; retrieve it at runtime (see README.md). This script neither reads nor prints it.
set -euo pipefail

ARGOCD_VERSION="${ARGOCD_VERSION:-v3.5.4}"
ARGOCD_NAMESPACE="${ARGOCD_NAMESPACE:-argocd}"
MANIFEST_URL="https://raw.githubusercontent.com/argoproj/argo-cd/${ARGOCD_VERSION}/manifests/install.yaml"

log() { echo "[install-argocd] $*"; }

log "ArgoCD version (pinned): ${ARGOCD_VERSION}"
log "Target namespace:        ${ARGOCD_NAMESPACE}"
log "Manifest:                ${MANIFEST_URL}"

command -v kubectl >/dev/null 2>&1 || { echo "[install-argocd] ERROR: kubectl not found in PATH" >&2; exit 2; }

log "verifying cluster reachability (kubectl get --raw=/healthz) ..."
if ! kubectl get --raw='/healthz' >/dev/null 2>&1; then
  echo "[install-argocd] ERROR: cluster not reachable with the current kube context." >&2
  echo "[install-argocd]        Ensure kubeconfig/context is set (see deploy/bootstrap/cluster/README.md)." >&2
  exit 3
fi

log "ensuring namespace '${ARGOCD_NAMESPACE}' exists (idempotent) ..."
kubectl create namespace "${ARGOCD_NAMESPACE}" --dry-run=client -o yaml | kubectl apply -f -

log "applying pinned ArgoCD manifest via server-side apply (idempotent; converges to desired state) ..."
kubectl apply -n "${ARGOCD_NAMESPACE}" --server-side --force-conflicts -f "${MANIFEST_URL}"

if [[ "${SKIP_WAIT:-0}" == "1" ]]; then
  log "SKIP_WAIT=1 set — applied without waiting. Run ./verify-argocd.sh to confirm readiness."
  exit 0
fi

log "waiting for core ArgoCD workloads to become Ready (deployments + application-controller StatefulSet) ..."
for d in argocd-redis argocd-repo-server argocd-dex-server \
         argocd-applicationset-controller argocd-notifications-controller argocd-server; do
  kubectl -n "${ARGOCD_NAMESPACE}" rollout status "deployment/${d}" --timeout=300s
done
kubectl -n "${ARGOCD_NAMESPACE}" rollout status statefulset/argocd-application-controller --timeout=300s

log "install complete. ArgoCD ${ARGOCD_VERSION} is installed and its core workloads are Ready in '${ARGOCD_NAMESPACE}'."
log "Next: ./verify-argocd.sh (readiness proof) and README.md (access + credential retrieval + RBAC posture)."
