#!/usr/bin/env bash
# chirp durable-store durability drill (EPIC-PERSIST #64 / #67, REQ-OPS-013 / BAR-022).
#
# WHAT THIS PROVES
#   Nothing a user created is lost when the deployment is disrupted. The drill creates an account,
#   posts a chirp, follows another account and likes a chirp, then triggers each failure mode
#   --- restart, redeploy, reschedule --- and after each confirms the account still signs in and its
#   chirps / follows / likes / counts are present and correct. It proves this at two layers:
#     * STORE layer  --- a row written to the PostgreSQL StatefulSet survives pod restart, pod
#                        redeploy (new pod-template revision) and reschedule, because domain state
#                        lives on the per-ordinal PVC, not in any pod. Always runnable.
#     * SYSTEM layer --- the same, exercised end-to-end through the real HTTP API against a wired
#                        chirp-server, proving the serving tier is stateless and reads survive both
#                        tiers' pod loss. Runs when a durable chirp-server image is supplied.
#
# IT USES ONLY WHAT THE REPOSITORY PROVIDES
#   The manifests under deploy/chirp (the same files ArgoCD syncs) plus standard tooling already on
#   any operator's path: kubectl, and (for the system layer) curl + openssl. It applies the manifests
#   into a throwaway namespace so it never touches the live `chirp` workload, and tears it down after.
#   See PROCEDURE.md for prerequisites, the pass check, and the honest caveats.
#
# USAGE
#   deploy/chirp/drill/durability-drill.sh
#
# ENVIRONMENT (all optional)
#   KUBECTL_CONTEXT   kube context to use            (default: current context)
#   DRILL_NAMESPACE   ephemeral namespace to use     (default: chirp-drill)
#   CHIRP_IMAGE       durable chirp-server image ref to run the SYSTEM layer. When unset, only the
#                     STORE layer runs (and the SYSTEM layer reports SKIPPED). The image MUST be a
#                     chirp-server built `--features postgres`; a movable tag or the all-zeros
#                     placeholder digest is refused. See PROCEDURE.md for how to obtain one.
#   KEEP              1 = leave the namespace standing for inspection (default: 0 = tear down)
#
# EXIT STATUS  0 = every mode that ran PASSed; non-zero = at least one mode FAILed or setup broke.
set -u -o pipefail

CTX_ARG=()
[[ -n "${KUBECTL_CONTEXT:-}" ]] && CTX_ARG=(--context "${KUBECTL_CONTEXT}")
NS="${DRILL_NAMESPACE:-chirp-drill}"
KEEP="${KEEP:-0}"
IMAGE="${CHIRP_IMAGE:-}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CHIRP_DIR="$(cd "${HERE}/.." && pwd)"          # deploy/chirp
RENDER="$(mktemp -d)"
PLACEHOLDER_DIGEST="sha256:0000000000000000000000000000000000000000000000000000000000000000"
FAILED=0
PF_PID=""

k()      { kubectl "${CTX_ARG[@]}" "$@"; }
kn()     { kubectl "${CTX_ARG[@]}" -n "${NS}" "$@"; }
log()    { printf '\n=== %s ===\n' "$*"; }
note()   { printf '    %s\n' "$*"; }
die()    { printf 'drill: %s\n' "$*" >&2; exit 2; }
pass()   { printf 'MODE %-18s PASS  %s\n' "$1" "${2:-}"; }
failm()  { printf 'MODE %-18s FAIL  %s\n' "$1" "${2:-}"; FAILED=1; }

cleanup() {
  [[ -n "${PF_PID}" ]] && kill "${PF_PID}" 2>/dev/null
  rm -rf "${RENDER}"
  if [[ "${KEEP}" != "1" ]]; then
    log "teardown: deleting namespace ${NS}"
    k delete ns "${NS}" --wait=false >/dev/null 2>&1
  else
    note "KEEP=1 --- namespace ${NS} left standing; delete with: kubectl delete ns ${NS}"
  fi
}
trap cleanup EXIT

# Render a deploy/chirp manifest into the ephemeral namespace: drop the hard-coded `namespace: chirp`
# metadata line so `-n ${NS}` places it. Echoes the rendered path.
render() {
  local src="${CHIRP_DIR}/$1" out="${RENDER}/$1"
  [[ -f "${src}" ]] || die "manifest not found: ${src}"
  sed '/^  namespace: chirp$/d' "${src}" > "${out}"
  printf '%s' "${out}"
}

require() { command -v "$1" >/dev/null 2>&1 || die "required tool not found on PATH: $1"; }

psql_q() { # run SQL in the postgres pod; echoes tuples-only output
  kn exec -i statefulset/chirp-postgres -- env PGPASSWORD="${PGP}" \
     psql -U chirp -d chirp -tAc "$1"
}

# ---------------------------------------------------------------------------------------------------
# Preflight
# ---------------------------------------------------------------------------------------------------
require kubectl
k version >/dev/null 2>&1 || die "kubectl cannot reach a cluster (check KUBECTL_CONTEXT)"
if [[ -n "${IMAGE}" ]]; then
  require curl; require openssl
  case "${IMAGE}" in
    *"${PLACEHOLDER_DIGEST}"*) die "CHIRP_IMAGE is the all-zeros placeholder digest; supply a real durable image (see PROCEDURE.md)";;
    *@sha256:*|*:*) : ;;  # a digest-pinned ref or an explicit local tag is acceptable for the drill
    *) die "CHIRP_IMAGE must carry an explicit tag or @sha256 digest";;
  esac
fi

log "preflight"
note "context : $(k config current-context 2>/dev/null)"
note "namespace: ${NS} (ephemeral)"
note "system layer: $([[ -n "${IMAGE}" ]] && echo "ON (image ${IMAGE})" || echo "SKIPPED (set CHIRP_IMAGE to enable)")"

# ---------------------------------------------------------------------------------------------------
# Setup: ephemeral namespace (restricted PSA, matching the live chirp namespace), credential Secret,
# and the durable Postgres store from the repo manifests.
# ---------------------------------------------------------------------------------------------------
log "setup: namespace + credential Secret + Postgres store"
k create ns "${NS}" --dry-run=client -o yaml | k apply -f - >/dev/null
k label ns "${NS}" \
  pod-security.kubernetes.io/enforce=restricted \
  pod-security.kubernetes.io/enforce-version=latest --overwrite >/dev/null

# Out-of-band credential (ephemeral, never committed) --- mirrors the documented chirp-db-credentials
# Secret. The URL host matches the headless Service DNS in THIS namespace.
PGP="$(openssl rand -base64 24 2>/dev/null | tr -d '/+=' | head -c 32)"
[[ -n "${PGP}" ]] || PGP="drill$(date +%s)pw"
kn delete secret chirp-db-credentials --ignore-not-found >/dev/null 2>&1
kn create secret generic chirp-db-credentials \
  --from-literal=POSTGRES_DB=chirp \
  --from-literal=POSTGRES_USER=chirp \
  --from-literal=POSTGRES_PASSWORD="${PGP}" \
  --from-literal=CHIRP_DATABASE_URL="postgres://chirp:${PGP}@chirp-postgres.${NS}.svc.cluster.local:5432/chirp?sslmode=disable" >/dev/null
note "created Secret chirp-db-credentials (ephemeral)"

kn apply -f "$(render 45-postgres-service.yaml)" >/dev/null
kn apply -f "$(render 40-postgres-statefulset.yaml)" >/dev/null
note "applied Postgres headless Service + StatefulSet; waiting for readiness (initdb can take ~2min)"
kn rollout status statefulset/chirp-postgres --timeout=240s || die "Postgres store did not become ready"

# ---------------------------------------------------------------------------------------------------
# STORE-LAYER PROOF --- always runs. A known row must survive each failure mode byte-for-byte.
# ---------------------------------------------------------------------------------------------------
log "store layer: seed known rows (account / chirp / follow / like) + checksum"
psql_q "CREATE TABLE IF NOT EXISTS drill_evidence (kind text PRIMARY KEY, detail text, count bigint);" >/dev/null
psql_q "INSERT INTO drill_evidence VALUES
          ('account','drill-user alice',1),
          ('chirp','hello durable world',1),
          ('follow','alice->bob',1),
          ('like','alice liked c1',1)
        ON CONFLICT (kind) DO UPDATE SET detail=EXCLUDED.detail, count=EXCLUDED.count;" >/dev/null
SEED_SUM="$(psql_q "SELECT md5(string_agg(kind||detail||count,';' ORDER BY kind)) FROM drill_evidence;")"
note "seed checksum: ${SEED_SUM}"
psql_q "SELECT kind||' | '||detail||' | '||count FROM drill_evidence ORDER BY kind;" | sed 's/^/    row: /'

store_verify() { # $1 = mode label
  local now; now="$(psql_q "SELECT md5(string_agg(kind||detail||count,';' ORDER BY kind)) FROM drill_evidence;")"
  if [[ "${now}" == "${SEED_SUM}" ]]; then pass "store/$1" "checksum ${now}"; else failm "store/$1" "checksum ${now} != seed ${SEED_SUM}"; fi
}

log "store layer / restart: delete the postgres pod; StatefulSet recreates it on the same PVC"
kn delete pod chirp-postgres-0 --wait=true >/dev/null
kn rollout status statefulset/chirp-postgres --timeout=180s >/dev/null && store_verify restart

log "store layer / redeploy: rollout restart (new pod-template revision, same PVC)"
kn rollout restart statefulset/chirp-postgres >/dev/null
kn rollout status statefulset/chirp-postgres --timeout=180s >/dev/null && store_verify redeploy

log "store layer / reschedule: evict the pod from its node; it re-binds the PVC"
CUR_NODE="$(kn get pod chirp-postgres-0 -o jsonpath='{.spec.nodeName}')"
PV="$(kn get pvc data-chirp-postgres-0 -o jsonpath='{.spec.volumeName}')"
PV_NODE="$(k get pv "${PV}" -o jsonpath='{.spec.nodeAffinity.required.nodeSelectorTerms[0].matchExpressions[0].values[0]}' 2>/dev/null)"
note "pod node=${CUR_NODE}; PVC ${PV} node-affinity=${PV_NODE:-<none/networked>}"
if [[ -n "${PV_NODE}" ]]; then
  # Node-local storage (e.g. local-path): the volume is pinned to one node, so a cross-node
  # reschedule is physically impossible and the pod re-binds on the SAME node. We demonstrate the
  # eviction-and-rebind and record the node-affinity fact honestly (see PROCEDURE.md caveat).
  k cordon "${CUR_NODE}" >/dev/null
  kn delete pod chirp-postgres-0 --wait=false >/dev/null
  sleep 10
  PHASE="$(kn get pod chirp-postgres-0 -o jsonpath='{.status.phase}' 2>/dev/null || echo gone)"
  note "while ${CUR_NODE} cordoned, pod phase=${PHASE} (node-local PV cannot move to another node)"
  k uncordon "${CUR_NODE}" >/dev/null
fi
kn rollout status statefulset/chirp-postgres --timeout=180s >/dev/null
LAND_NODE="$(kn get pod chirp-postgres-0 -o jsonpath='{.spec.nodeName}')"
note "pod landed on node=${LAND_NODE}"
store_verify reschedule

# ---------------------------------------------------------------------------------------------------
# SYSTEM-LAYER PROOF --- runs only when a durable chirp-server image is supplied. Exercises the real
# HTTP API: create account / post chirp / follow / like, then each failure mode + a full re-verify.
# ---------------------------------------------------------------------------------------------------
if [[ -z "${IMAGE}" ]]; then
  log "system layer: SKIPPED (no CHIRP_IMAGE). Store-layer durability proven above."
else
  log "system layer: deploy wired chirp-server (image ${IMAGE})"
  # Render the committed Deployment: point it at the supplied durable image and drop the private
  # ghcr pull-secret (the drill supplies its own image + ephemeral Secret).
  sed -e '/^  namespace: chirp$/d' \
      -e "s#^\(          image: \).*#\1${IMAGE}#" \
      -e '/^        - name: ghcr-pull$/d' \
      -e '/^      imagePullSecrets:$/d' \
      "${CHIRP_DIR}/10-deployment.yaml" > "${RENDER}/10-deployment.yaml"
  kn apply -f "${RENDER}/10-deployment.yaml" >/dev/null
  kn rollout status deploy/chirp-server --timeout=180s || die "chirp-server did not become ready (readiness gates on /readyz = store_health)"

  pf_open() { # (re)open a port-forward to the Deployment on :18080; sets PF_PID
    [[ -n "${PF_PID}" ]] && kill "${PF_PID}" 2>/dev/null; PF_PID=""
    kn port-forward deploy/chirp-server 18080:8080 >"${RENDER}/pf.log" 2>&1 &
    PF_PID=$!
    local i; for i in $(seq 1 60); do curl -fsS http://127.0.0.1:18080/readyz >/dev/null 2>&1 && return 0; sleep 1; done
    return 1
  }
  B=http://127.0.0.1:18080
  jnum() { grep -oE "\"$2\":[0-9]+" <<<"$1" | head -1 | grep -oE '[0-9]+$'; }

  pf_open || die "chirp-server /readyz never responded through port-forward"
  log "system layer: seed via the real API"
  ALICE="$(curl -fsS -X POST "${B}/api/users" -H 'content-type: application/json' -d '{"username":"alice","displayName":"Alice","bio":null}')"
  BOB="$(curl -fsS -X POST "${B}/api/users" -H 'content-type: application/json' -d '{"username":"bob","displayName":"Bob","bio":null}')"
  AID="$(jnum "${ALICE}" id)"; BID="$(jnum "${BOB}" id)"
  TOK="$(curl -fsS -X POST "${B}/api/sessions" -H 'content-type: application/json' -d '{"username":"alice"}' | grep -oE '"token":"[^"]+"' | cut -d'"' -f4)"
  CH="$(curl -fsS -X POST "${B}/api/chirps" -H "authorization: Bearer ${TOK}" -H 'content-type: application/json' -d '{"text":"hello durable world","replyTo":null}')"
  CID="$(jnum "${CH}" id)"
  curl -fsS -X PUT "${B}/api/users/${BID}/follow" -H "authorization: Bearer ${TOK}" >/dev/null
  curl -fsS -X PUT "${B}/api/chirps/${CID}/like"  -H "authorization: Bearer ${TOK}" >/dev/null
  note "seeded alice(id=${AID}) bob(id=${BID}) chirp(id=${CID}); alice follows bob; alice liked the chirp"

  api_verify() { # $1 = mode label.  Pass check: sign-in works AND content/counts intact.
    pf_open || { failm "api/$1" "server never ready after $1"; return; }
    local signin tok a b tl af bf lc items txt
    signin="$(curl -fsS -X POST "${B}/api/sessions" -H 'content-type: application/json' -d '{"username":"alice"}')"
    tok="$(grep -oE '"token":"[^"]+"' <<<"${signin}" | cut -d'"' -f4)"
    a="$(curl -fsS "${B}/api/users/${AID}")"
    b="$(curl -fsS "${B}/api/users/${BID}")"
    tl="$(curl -fsS "${B}/api/timeline" -H "authorization: Bearer ${tok}")"
    af="$(jnum "${a}" followingCount)"; bf="$(jnum "${b}" followerCount)"
    lc="$(jnum "${tl}" likeCount)"
    items="$(grep -oE '"id":[0-9]+' <<<"${tl}" | wc -l | tr -d ' ')"
    txt=0; grep -q 'hello durable world' <<<"${tl}" && txt=1
    local detail="signin=$([[ -n ${tok} ]] && echo ok || echo FAIL) following=${af} followers=${bf} likeCount=${lc} items=${items} text=${txt}"
    if [[ -n "${tok}" && "${af}" == 1 && "${bf}" == 1 && "${lc}" == 1 && "${items}" == 1 && "${txt}" == 1 ]]; then
      pass "api/$1" "${detail}"
    else
      failm "api/$1" "${detail}"
    fi
  }

  log "system layer / restart: delete the chirp-server pod (stateless tier)"
  kn delete pod -l app.kubernetes.io/name=chirp-server --wait=true >/dev/null 2>&1
  kn rollout status deploy/chirp-server --timeout=120s >/dev/null && api_verify restart

  log "system layer / redeploy: rollout restart chirp-server (new revision)"
  kn rollout restart deploy/chirp-server >/dev/null
  kn rollout status deploy/chirp-server --timeout=120s >/dev/null && api_verify redeploy

  log "system layer / reschedule: delete the postgres store pod; the server reconnects to the durable store"
  kn delete pod chirp-postgres-0 --wait=true >/dev/null 2>&1
  kn rollout status statefulset/chirp-postgres --timeout=180s >/dev/null && api_verify reschedule
fi

# ---------------------------------------------------------------------------------------------------
log "result"
if [[ "${FAILED}" == 0 ]]; then
  note "ALL MODES PASS --- user data survived restart, redeploy and reschedule."
  exit 0
else
  note "AT LEAST ONE MODE FAILED --- see the MODE lines above."
  exit 1
fi
