#!/usr/bin/env bash
# verify-argocd.sh — real readiness proof for the ArgoCD control plane (issue #10, infra-bootstrap)
#
# WHAT: Asserts the ArgoCD core workloads are actually Ready in the target namespace. READ-ONLY: it
#       reads cluster state and makes NO changes. Exits 0 only if every expected workload is Ready;
#       exits non-zero (and prints what is wrong) otherwise. This is the honest "prove, don't assume"
#       check — not a claim.
#
# PREREQUISITES:
#   - kubectl in PATH; a kube context pointing at the cluster where ArgoCD was installed.
#   - ArgoCD installed via ./install-argocd.sh (same ARGOCD_NAMESPACE).
#
# RE-RUN BEHAVIOR: idempotent and side-effect-free; safe to run any number of times.
#
# USAGE:
#   ./verify-argocd.sh
#   ARGOCD_NAMESPACE=argocd ./verify-argocd.sh
set -euo pipefail

ARGOCD_NAMESPACE="${ARGOCD_NAMESPACE:-argocd}"
TIMEOUT="${TIMEOUT:-180s}"

DEPLOYMENTS=(argocd-redis argocd-repo-server argocd-dex-server
            argocd-applicationset-controller argocd-notifications-controller argocd-server)
STATEFULSETS=(argocd-application-controller)

log() { echo "[verify-argocd] $*"; }
fail() { echo "[verify-argocd] FAIL: $*" >&2; exit 1; }

command -v kubectl >/dev/null 2>&1 || { echo "[verify-argocd] ERROR: kubectl not found in PATH" >&2; exit 2; }

kubectl get namespace "${ARGOCD_NAMESPACE}" >/dev/null 2>&1 \
  || fail "namespace '${ARGOCD_NAMESPACE}' does not exist — run ./install-argocd.sh first."

log "checking ArgoCD CRDs are registered ..."
for crd in applications.argoproj.io applicationsets.argoproj.io appprojects.argoproj.io; do
  kubectl get crd "${crd}" >/dev/null 2>&1 || fail "CRD ${crd} missing."
done

log "waiting (up to ${TIMEOUT}) for each core Deployment to be Ready ..."
for d in "${DEPLOYMENTS[@]}"; do
  kubectl -n "${ARGOCD_NAMESPACE}" rollout status "deployment/${d}" --timeout="${TIMEOUT}" \
    || fail "deployment/${d} not Ready."
done

log "waiting (up to ${TIMEOUT}) for the application-controller StatefulSet to be Ready ..."
for s in "${STATEFULSETS[@]}"; do
  kubectl -n "${ARGOCD_NAMESPACE}" rollout status "statefulset/${s}" --timeout="${TIMEOUT}" \
    || fail "statefulset/${s} not Ready."
done

log "pod summary:"
kubectl -n "${ARGOCD_NAMESPACE}" get pods -o wide

# Assert no pod is in a non-Running phase (defensive: rollout status can pass transiently).
NOT_RUNNING="$(kubectl -n "${ARGOCD_NAMESPACE}" get pods \
  --field-selector=status.phase!=Running -o name 2>/dev/null || true)"
[[ -z "${NOT_RUNNING}" ]] || fail "one or more pods not Running: ${NOT_RUNNING}"

INSTALLED_IMAGE="$(kubectl -n "${ARGOCD_NAMESPACE}" get deploy argocd-server \
  -o jsonpath='{.spec.template.spec.containers[0].image}')"
log "installed argocd-server image: ${INSTALLED_IMAGE}"

log "PASS: ArgoCD control plane is installed and all core workloads are Ready in '${ARGOCD_NAMESPACE}'."
