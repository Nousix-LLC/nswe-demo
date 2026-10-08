# `deploy/chirp/` — chirp-server Kubernetes workload manifests

Declarative manifests that run the **chirp-server** app on the k3d-hosted k3s cluster `nswe`
and make it reachable from outside. Authored by `SUBTASK_app_manifests` (NSWE #13, deploy-argocd).
These manifests are tracked by the ArgoCD `Application` authored by the sibling task
(`deploy/argocd/`, which syncs **only** this `deploy/chirp` path).

| File | Resource | Purpose |
|------|----------|---------|
| `00-namespace.yaml` | `Namespace chirp` | Isolation; Pod Security Admission = `restricted`. |
| `10-deployment.yaml` | `Deployment chirp-server` | The workload: digest-pinned image, probes, resources, graceful shutdown, hardened securityContext. |
| `20-service.yaml` | `Service chirp-server` | `ClusterIP`, port `80` → targetPort `8080`. |
| `30-ingress.yaml` | `Ingress chirp-server` | Traefik ingress, host `chirp.localhost`, path `/`. |

All resources live in namespace `chirp` and carry the contract label set
(`app.kubernetes.io/{name,part-of,managed-by}`). Filenames are numerically prefixed so a manual
`kubectl apply -f deploy/chirp/` applies the Namespace first (ArgoCD orders resources itself).

## Resolved facts these manifests are pinned to

From `../../<taskflow>/SUBTASK_discovery/outputs/ground_truth.md`:

- **Image** (immutable manifest-list digest, #24 SEC-3):
  `ghcr.io/nousix-llc/chirp-server@sha256:054af27f588c13aeebd8698dbddf690e15bf75cd36c9a087b38763fd0902abe3`
- **Container listen port**: `8080` (`CHIRP_BIND_ADDR=0.0.0.0:8080`, image default)
- **Health surface**: `GET /healthz` on `:8080` → `{"status":"ok",...}` (the SPA is at `/`; probes
  target `/healthz`, **not** `/`)
- **Ingress class**: `traefik` (k3s built-in)
- **Runtime user**: non-root uid/gid `65532` (distroless) → read-only root fs is safe

## Reachability (how the synthesis gate tests it)

The k3d serverlb maps host `localhost:8081 -> 80` through Traefik. Routing is **host-based** on
`chirp.localhost`, so the Host header is required:

```sh
curl -H 'Host: chirp.localhost' http://localhost:8081/healthz   # -> HTTP 200 {"status":"ok",...}
curl -H 'Host: chirp.localhost' http://localhost:8081/          # -> HTTP 200 (the WASM SPA)
```

## imagePullSecret (private GHCR package) — handled, no secret committed

The `chirp-server` GHCR package is **private** (ground_truth.md §6). The Deployment references an
`imagePullSecrets` entry named **`ghcr-pull`**. That Secret is **NOT committed to git** (constraint:
no secrets in the tree). Exactly one of the following must hold before/at deploy so the pod does not
wedge in `ImagePullBackOff` (gate identity `image-pull-works`):

1. **Create the pull Secret out-of-band** in namespace `chirp` (the synthesis gate's deploy step):
   ```sh
   kubectl -n chirp create secret docker-registry ghcr-pull \
     --docker-server=ghcr.io \
     --docker-username='<gh-user-or-bot>' \
     --docker-password='<GHCR token with read:packages>'
   ```
   (A sealed/external-secret mechanism is the GitOps-managed equivalent; the raw Secret never lands
   in git.) **or**
2. **Make the GHCR package public**, after which the `imagePullSecrets` field is unnecessary and may
   be dropped.

The decision taken (and the credential source) should be recorded in the root `SYNTHESIS.md`.

## Design notes

- **`replicas: 1` is deliberate.** chirp-server uses an in-memory repository (no DB/PVC). Multiple
  replicas would each hold independent state and serve inconsistent data. Horizontal scale-out
  requires introducing a shared backing store first.
- **Pod Security Admission = `restricted`.** The pod is fully `restricted`-compliant
  (`runAsNonRoot`, `runAsUser: 65532`, `allowPrivilegeEscalation: false`, `capabilities.drop: [ALL]`,
  `seccompProfile: RuntimeDefault`, `readOnlyRootFilesystem: true`, only an `emptyDir`), so enforcing
  the standard at the namespace is safe and adds admission-time defense in depth.
- **Read-only root filesystem** with a small `emptyDir` mounted at `/tmp` for scratch.
- **Graceful shutdown** relies on chirp-server being PID 1 (exec-form entrypoint) and draining on
  `SIGTERM`; `terminationGracePeriodSeconds: 30`, no preStop hook needed.
- **RollingUpdate `maxSurge: 1`/`maxUnavailable: 0`** keeps one pod serving across a roll.
- **Ingress vs Gateway API**: Ingress is the idiomatic choice for k3s/Traefik and is what the
  contract + ground_truth specify; see the header comment in `30-ingress.yaml`.

## Divergences on a native/systemd k3s cluster (#23 HSO-1)

These manifests target **only** the proven k3d-hosted k3s cluster. On a native/systemd install:

- The `localhost:8081 -> 80` serverlb mapping is k3d-specific. On a native cluster the Ingress is
  reached via a real node/LB address on `:80`/`:443` (and a real DNS name rather than
  `chirp.localhost`); the Host-based rule still applies but the test URL differs.
- TLS would be terminated at Traefik with a real certificate (cert-manager or provided) rather than
  omitted; revisit `30-ingress.yaml` to add a `tls:` block and listener on `:443`.
- The `restricted` PSA posture, digest pinning, resources/probes, and the pull-secret requirement are
  cluster-independent and carry over unchanged.

## Handoff to the synthesis gate (git state)

The checkout at `/home/mattm/nswe-demo` was on local `main` (`25e90c0`), **5 commits behind**
`origin/main @ ada3aa9` — the `deploy/` tree and the `Dockerfile` only exist at `ada3aa9`
(ground_truth.md §1). Per this subtree's decomposition, **git branch/commit/PR is the synthesis
gate's job, not an authoring spoke's** (two concurrent spokes must not race on shared git state).

Therefore these files were written as **untracked** files into the working tree. The synthesis gate
should:

1. `git -C /home/mattm/nswe-demo fetch origin`
2. `git -C /home/mattm/nswe-demo checkout -B work/deploy-argocd ada3aa9`  (== `origin/main`)
   — `deploy/chirp/` does not exist at `ada3aa9`, so these untracked files survive the checkout.
3. Stage and commit `deploy/chirp/` (and the sibling's `deploy/argocd/`) onto `work/deploy-argocd`.
   **Do not** `git clean`/`reset --hard` before staging — that would discard these untracked files.

A verbatim mirror of these manifests is also kept under the task workspace
(`SUBTASK_app_manifests/outputs/`) as a stable source independent of any repo git operation.
