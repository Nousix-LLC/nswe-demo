# Durability drill

A repeatable check that nothing a user created is lost when the deployment is disrupted. It creates
an account, posts a chirp, follows another account and likes a chirp, then triggers each failure mode
in #64 — **restart**, **redeploy**, **reschedule** — and after each confirms the account still signs
in and its chirps, follows, likes and counts are present and correct.

The drill uses only what this repository provides: the manifests under `deploy/chirp` (the same files
ArgoCD syncs) plus `kubectl`, `curl` and `openssl`. It applies those manifests into a throwaway
namespace, so it never touches the live `chirp` workload, and deletes the namespace when it finishes.

## What it proves, at two layers

| Layer | Mechanism exercised | Runs when |
|-------|---------------------|-----------|
| **Store** | A row written to the PostgreSQL StatefulSet survives pod restart, redeploy (new pod-template revision) and reschedule, because domain state lives on the per-ordinal PVC, not in any pod. | Always — `postgres` is a public image. |
| **System** | The same, end-to-end through the real HTTP API against a wired `chirp-server`: the serving tier is stateless and every read survives both tiers' pod loss. | When a durable `chirp-server` image is supplied (`CHIRP_IMAGE`). |

## Prerequisites

- `kubectl` pointing at the target cluster. Set `KUBECTL_CONTEXT` if it is not your current context.
- `curl` and `openssl` on your PATH (only needed for the system layer).
- For the system layer, a `chirp-server` image built `--features postgres` — see
  [Supplying a durable image](#supplying-a-durable-image). Without it, only the store layer runs and
  the system layer reports `SKIPPED`.

## Run it

```sh
# store layer only (no server image needed)
deploy/chirp/drill/durability-drill.sh

# full store + system layer
CHIRP_IMAGE=<durable chirp-server image ref> deploy/chirp/drill/durability-drill.sh
```

Environment variables (all optional):

| Variable | Default | Meaning |
|----------|---------|---------|
| `KUBECTL_CONTEXT` | current context | kube context to use |
| `DRILL_NAMESPACE` | `chirp-drill` | ephemeral namespace to create and delete |
| `CHIRP_IMAGE` | _unset_ | durable `chirp-server` image; enables the system layer |
| `KEEP` | `0` | `1` leaves the namespace standing for inspection |

The script exits `0` only if every mode that ran passed, and non-zero if any mode failed or setup
broke, so it is usable as a gate in CI or a release checklist.

## The pass check

For each failure mode the drill asserts, after the system has recovered:

- **Sign-in works** — `POST /api/sessions` for the account returns a session token.
- **Content is present** — the account's chirp is in `GET /api/timeline` with its original text.
- **Counts are correct** — `followingCount`, `followerCount` and `likeCount` match what was created.
- **Store layer** additionally checksums the seeded rows and requires a byte-for-byte match.

A mode passes only if all of these hold; otherwise the drill reports `FAIL` for that mode and exits
non-zero.

## What each mode does

- **restart** — delete the pod; the controller recreates it (new pod, same PVC for Postgres). Proves
  no state lived in the pod process.
- **redeploy** — `rollout restart`; a new pod-template revision replaces the old pod. Proves a manifest
  or image roll preserves data.
- **reschedule** — evict the pod from its node. For `chirp-server` (stateless) it moves freely; for
  Postgres it re-binds its PVC. See the storage caveat below.

## Supplying a durable image

The published `chirp-server` image is built and pushed by `main`'s CI **by digest, only on push to
`main`**. Because this change is not merged yet, a fully-pinned durable digest does not exist until
merge. Two ways to run the system layer:

- **Pre-merge (local):** build the image from this repo and load it into the cluster, then pass the
  local tag:

  ```sh
  docker build -t chirp-server:drill .        # Dockerfile builds chirp-server --features postgres
  k3d image import chirp-server:drill -c <cluster>   # or: kind load / docker save | ctr import
  CHIRP_IMAGE=chirp-server:drill deploy/chirp/drill/durability-drill.sh
  ```

- **Post-merge:** use the digest `main`'s CI published (the same digest that pins
  `deploy/chirp/10-deployment.yaml`):

  ```sh
  CHIRP_IMAGE=ghcr.io/nousix-llc/chirp-server@sha256:<digest> deploy/chirp/drill/durability-drill.sh
  ```

## Caveats (read before trusting a pass)

- **Node-local storage pins the volume to one node.** On a cluster whose default StorageClass is
  node-local (for example `local-path`), the Postgres PVC is bound to the node that first provisioned
  it. A pod can only reschedule back to that node; a genuine cross-node reschedule is physically
  impossible and the drill records the PVC's node affinity and the `Pending` state it observes when
  the node is cordoned. Cross-node reschedule durability and database failover need networked or
  replicated storage and are deferred by ADR-0001 (documented limitation), not covered here.
- **The drill runs in a throwaway namespace, not the live one.** It stands the store up from the
  manifests so it is self-contained and safe to run anywhere; it is not a check of the currently
  reconciled `chirp` namespace. Delivery is GitOps: durability is live once this change is on `main`
  and reconciled.
- **The credential is created ad hoc and never committed**, mirroring the out-of-band
  `chirp-db-credentials` Secret the deployment expects.
