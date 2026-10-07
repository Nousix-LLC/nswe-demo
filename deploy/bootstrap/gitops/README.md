# GitOps control plane — ArgoCD install (bootstrap runbook fragment)

Part of the infra-bootstrap (issue #10) `deploy/bootstrap/` runbook. This fragment covers the **ArgoCD
GitOps control plane** only: a pinned, reproducible install into a dedicated namespace, a real
readiness check, how to reach the API/UI, how to **retrieve** the initial admin credential at runtime
(no secret is ever committed), and the least-privilege/RBAC posture.

> Scope guard: this installs and proves the **control plane**. It does **not** create any ArgoCD
> `Application`, app Deployment/Service manifests, or container images — those are issues #13 / #12.
> ArgoCD's *own* install workloads are in scope; an app `Application` is not.

## Pinned versions

| Component | Pinned version |
|-----------|----------------|
| ArgoCD | **v3.5.4** (`quay.io/argoproj/argocd:v3.5.4`; upstream `manifests/install.yaml` @ tag `v3.5.4`) |
| Namespace | `argocd` (dedicated) |

To move versions, set `ARGOCD_VERSION=vX.Y.Z` and re-run `install-argocd.sh`.

## Prerequisites

- A reachable Kubernetes cluster + a kube context with cluster-admin (ArgoCD installs CRDs and
  ClusterRoles). On this engagement that is the k3s cluster from `deploy/bootstrap/cluster/` — see
  that fragment's README for kubeconfig **retrieval** (no kubeconfig is committed here).
- `kubectl` >= 1.27 in `PATH`. Server-side apply with `--force-conflicts` is required (see below).
- Network egress to `raw.githubusercontent.com` (manifest) and `quay.io` (images).

## Run order

```bash
# 1. Install the pinned control plane into the argocd namespace (idempotent; waits for Ready).
./install-argocd.sh
#    (ARGOCD_VERSION / ARGOCD_NAMESPACE / SKIP_WAIT are overridable env vars.)

# 2. Prove readiness with a real, read-only check (exits non-zero if anything is not Ready).
./verify-argocd.sh
```

`install-argocd.sh` is safe to re-run: namespace creation is apply-of-dry-run, and the manifest is
applied with `kubectl apply --server-side --force-conflicts`, which converges to the pinned desired
state (`serverside-applied`/`unchanged`, exit 0) rather than erroring or duplicating.

**Why server-side apply:** ArgoCD's `applicationsets.argoproj.io` CRD is larger than the 262144-byte
client-side `last-applied-configuration` annotation limit, so a plain `kubectl apply -f install.yaml`
fails with `metadata.annotations: Too long`. `--server-side` stores no such annotation and is the
upstream-recommended path for ArgoCD's large CRDs.

### What "Ready" means here

`verify-argocd.sh` runs `kubectl rollout status` on all six core Deployments
(`argocd-redis`, `argocd-repo-server`, `argocd-dex-server`, `argocd-applicationset-controller`,
`argocd-notifications-controller`, `argocd-server`) and the `argocd-application-controller`
StatefulSet, asserts all pods are `Running`, and prints the installed server image. It changes
nothing. A PASS means the control plane is genuinely up — not assumed.

## Reaching the API / UI

The stock install exposes `argocd-server` as a **ClusterIP** service (no external LB, no NodePort) —
the safe default. Reach it with a port-forward (works on any cluster, including a sandboxed k3s/k3d):

```bash
# HTTPS UI + API on https://localhost:8080 (self-signed cert on first boot — expect a TLS warning).
kubectl -n argocd port-forward svc/argocd-server 8080:443
# then browse https://localhost:8080  (user: admin)
```

Optional CLI login (ArgoCD CLI installed separately):

```bash
argocd login localhost:8080 --username admin --password "<retrieved at runtime — see below>" --insecure
```

For a durable external endpoint later, front `argocd-server` with an Ingress/Gateway (TLS-terminated)
or a LoadBalancer — intentionally **not** configured here, to keep the control plane closed by default
until a real exposure decision is made.

## Retrieving the initial admin credential (NEVER committed)

ArgoCD generates its own bootstrap admin password inside the cluster at first boot and stores it in the
`argocd-initial-admin-secret` Secret. It is **not** in this repo and must be read from the cluster at
runtime:

```bash
# Prints the one-time initial admin password to YOUR terminal only. Do not paste it into any file/commit.
kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath='{.data.password}' | base64 -d; echo
```

Hardening (do this once you have logged in):

```bash
# 1. Change the admin password (ArgoCD CLI), then
argocd account update-password
# 2. Delete the bootstrap secret so the initial password no longer works.
kubectl -n argocd delete secret argocd-initial-admin-secret
```

The `admin` account is intended for bootstrap only; for ongoing use configure SSO (the bundled
`argocd-dex-server` brokers OIDC/SAML) and scope humans via `argocd-rbac-cm`, rather than sharing
`admin`.

## RBAC / least-privilege posture

The stock upstream install creates **one ServiceAccount per component** and grants cluster-scope only
where the component's job requires it:

| ServiceAccount | Cluster-scoped grant? | Why |
|----------------|-----------------------|-----|
| `argocd-application-controller` | **Yes — ClusterRole** (currently `*/*/*`, cluster-admin-equivalent) | It reconciles arbitrary target resources cluster-wide. This is the widest grant and the primary hardening lever (below). |
| `argocd-server` | Yes — ClusterRole | Serves the API/UI; needs to read/manage across target namespaces. |
| `argocd-applicationset-controller` | Yes — ClusterRole | Generates `Application` resources across namespaces/clusters. |
| `argocd-repo-server` | **No** — namespace Role only | Clones repos and renders manifests; runs arbitrary templating, so it is deliberately given **zero** cluster permissions (highest-risk component, least privilege). |
| `argocd-redis` | No — namespace Role only | Internal cache. |
| `argocd-dex-server` | No — namespace Role only | OIDC/SAML broker. |
| `argocd-notifications-controller` | No — namespace Role only | Sends notifications. |

The install also ships **seven default NetworkPolicies** (one per component) that constrain pod-to-pod
traffic within the namespace — a sane default-isolation posture out of the box.

**Hardening levers (deferred — not applied here, to avoid over-provisioning *or* under-provisioning
blindly):**
- Scope the `argocd-application-controller` ClusterRole down from `*/*/*` to the specific
  apiGroups/resources the workloads this instance manages actually need, and/or constrain targets with
  **AppProjects** (per-team allow-lists of source repos, destination clusters, and namespaces).
- For a single-tenant/namespaced footprint, consider the upstream **namespace-scoped install**
  (`namespace-install.yaml`) so the control plane cannot touch resources outside its own namespace.
- These are a follow-on hardening decision for when the first `Application` is introduced (#13), not a
  control-plane-bring-up concern.

## Security notes

- **No secret is committed.** No kubeconfig with live credentials, no admin password, no token or key
  appears in any staged file. The admin credential is *retrieved* at runtime via the command above and
  should be rotated + the bootstrap secret deleted immediately after first login.
- The port-forward uses the self-signed cert ArgoCD generates on first boot; replace it with a real
  certificate when a durable external endpoint is configured.

## Files in this fragment

| File | What it does | Re-run behavior |
|------|--------------|-----------------|
| `install-argocd.sh` | Pinned, idempotent ArgoCD install (server-side apply) into `argocd` ns + readiness wait | Safe to re-run; converges to desired state |
| `verify-argocd.sh` | Read-only readiness proof (rollout status of all core workloads + pod check) | Side-effect-free; re-runnable |
| `README.md` | This runbook fragment | — |
