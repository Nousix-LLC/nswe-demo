# deploy/argocd — GitOps wiring for the `chirp` app (Argo CD)

This directory holds the Argo CD GitOps configuration that makes Argo CD manage the `chirp`
application. It is authored by `SUBTASK_argocd_gitops` and composed against the frozen identities in
`taskflow/.../deploy-argocd/contracts/deploy_contract.yaml` and the resolved facts in
`SUBTASK_discovery/outputs/ground_truth.md`.

| File | Kind | Role |
|------|------|------|
| `appproject.yaml` | `argoproj.io/v1alpha1` `AppProject` (`chirp`) | Least-privilege authorization boundary |
| `application.yaml` | `argoproj.io/v1alpha1` `Application` (`chirp`) | Tracks `deploy/chirp` → namespace `chirp` |

Both resources live in the Argo CD control-plane namespace **`argocd`** and sit **outside** the
synced path (`deploy/chirp`), so Argo CD does not manage its own definition.

## What this satisfies (contract identities)

- **`argocd-application-wired`** — `application.yaml`: `source.path: deploy/chirp`,
  `repoURL: https://github.com/Nousix-LLC/nswe-demo.git`, `destination.namespace: chirp`,
  `spec.project: chirp` (not `default`).
- **`appproject-least-privilege`** — `appproject.yaml`: `destinations` restricted to
  `{server: https://kubernetes.default.svc, namespace: chirp}`; `sourceRepos` restricted to the
  nswe-demo repo; explicit `clusterResourceWhitelist` (Namespace only) and
  `namespaceResourceWhitelist` (Service, ServiceAccount, ConfigMap, Deployment, Ingress). **No
  `*/*/*` cluster-admin-equivalent grant.**

## Key engineering decisions (and their rationale)

1. **Least-privilege AppProject, not the stock cluster-admin posture (#24 SEC-2/SEC-4).** The
   `chirp` AppProject is the Application's authorization boundary. Even though the Argo CD
   application-controller's own ServiceAccount is broadly privileged, the project confines what the
   `chirp` Application may do: one source repo, one destination namespace, an explicit allow-list of
   resource kinds. This is the standard Argo CD multi-tenancy/least-privilege mechanism.

2. **`Secret` is deliberately excluded from the whitelist.** No secrets are committed (#24). The
   GHCR image is private (ground_truth §6), so an `imagePullSecret` is required — but it is applied
   **out-of-band** (not via Git), so Argo CD never manages a `Secret`. Excluding `Secret` from the
   project whitelist is defense-in-depth: a Secret could not be synced even if accidentally
   committed. **Coordination:** the `deploy/chirp` spoke / synthesis gate must provision the GHCR
   pull secret out-of-band (e.g. a `kubernetes.io/dockerconfigjson` Secret created with `kubectl`
   or via External Secrets/SOPS) OR make the GHCR package public. If the pull path is instead
   handled by a committed mechanism, request an **additive** amendment to the whitelist.

3. **Sync policy = automated prune + selfHeal + `allowEmpty: false`.** The GitOps idiom: Git is the
   source of truth, out-of-band edits are reverted, and a bad commit cannot empty the app. Backed by
   a bounded `retry` so transient apply-ordering converges without intervention.

4. **`CreateNamespace` intentionally NOT set; rely on the committed `Namespace` manifest.** The
   `chirp` Namespace is a committed manifest in `deploy/chirp` (contract identity
   `namespace-isolation-no-secrets`), so it is declaratively owned and labeled; `CreateNamespace=true`
   would imperatively create an unlabeled namespace that then diffs against the committed one.
   **Coordination recommendation (not owned here):** the `deploy/chirp` `Namespace` manifest should
   carry `argocd.argoproj.io/sync-wave: "-1"` so it applies before the namespaced resources
   deterministically. With the configured `retry`, the deploy also converges without it.

5. **`targetRevision: main`.** The committed Application tracks the default branch as the GitOps
   steady state. Tracking the ephemeral feature branch `work/deploy-argocd` would be an anti-pattern
   (the branch is deleted after the PR merges). See the pre-merge testing note below.

## SHA-pinning (#24 SEC-3)

The Application references **only in-repo plain manifests** (`path: deploy/chirp`) — it uses **no
external Helm charts or remote manifests**, so there is nothing external to pin by SHA at the GitOps
layer; SEC-3's external-reference-pinning requirement is satisfied vacuously here. (The container
**image** digest-pin lives in the `deploy/chirp` Deployment, owned by `SUBTASK_app_manifests`;
ground_truth §2 provides the manifest-list digest to pin.)

## Noted follow-ups (not implemented here — flagged for tracking)

- **cosign / admission-time image verification (noted per brief).** A supply-chain hardening
  follow-up: sign the `chirp-server` image with cosign/Sigstore and enforce signature verification
  at admission (e.g. a `ValidatingAdmissionPolicy` or Kyverno/Gatekeeper policy, or
  sigstore-policy-controller) so only signed images run. This is cluster-admission policy, out of
  scope for `deploy/argocd` and left as a tracked follow-up.
- **Apply-layer least privilege via sync impersonation.** To stop relying on the
  application-controller's broad ServiceAccount at the *apply* layer (not just the
  Application-authorization layer), enable Argo CD sync impersonation
  (`application.sync.impersonation.enabled: true` in `argocd-cm`) and add
  `spec.destinationServiceAccounts` to this AppProject mapping the chirp destination to a dedicated,
  minimally-scoped ServiceAccount (with a Role/RoleBinding in `chirp`, and a ClusterRole for the
  cluster-scoped Namespace create). This requires an `argocd-cm` change tied to the Argo CD install
  path (#10) — it is inert without the feature flag — so it is flagged here rather than
  half-implemented. The whitelisted AppProject above is the mandatory least-privilege control; this
  is the deeper enhancement.

## Operational notes

- **Base-checkout caveat (ground_truth §1).** The working checkout at `/home/mattm/nswe-demo` was
  reported 5 commits behind `origin/main` (`ada3aa9`), which is the real base that first introduced
  the `deploy/` tree. The synthesis gate (which owns git/branch/PR) must branch
  `work/deploy-argocd` from `ada3aa9` (== `origin/main`) before committing — not from the stale
  local `main`. These `deploy/argocd/` files are additive and survive that branch creation as
  untracked files.
- **Reconciling this directory itself / surviving a `--force-conflicts` install re-run (#24 SEC-4).**
  `deploy/argocd` is applied by the synthesis gate (`kubectl apply`), not synced by the `chirp`
  Application. The narrowing is GitOps-managed in the sense of being committed, declarative, and
  re-appliable from Git, and the project is named `chirp` (distinct from `default`), so an install
  path that resets the stock `default` project does not touch it. For *continuous* reconciliation of
  the project itself, a separate platform/bootstrap Application (owned by the Argo CD install, not by
  `chirp`) could reconcile `deploy/argocd` — a recommended follow-up, deliberately not an
  app-of-apps rooted here to avoid self-management recursion.
- **k3s vs native cluster (#23 HSO-1).** Targets the k3d-hosted k3s cluster `nswe` only. The
  in-cluster destination server (`https://kubernetes.default.svc`) and these CRs are identical on a
  native cluster; nothing in this directory is k3d-specific. (Ingress class/host specifics live in
  `deploy/chirp`, owned by the sibling.)

## Scope boundary

This directory owns **only** the Argo CD `Application` + `AppProject`. The workload manifests
(Namespace, Deployment, Service, Ingress, probes, resources, securityContext, imagePullSecret
wiring) are owned by `SUBTASK_app_manifests` under `deploy/chirp`. Applying to the cluster, proving
reachability, and opening the PR are owned by `SUBTASK_synthesis`.
