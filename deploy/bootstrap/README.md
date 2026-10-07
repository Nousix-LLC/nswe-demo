# nswe-demo — deployment-environment bootstrap (`deploy/bootstrap/`)

Reproducible scripts + runbook that **stand up and prove** the deployment environment the
engagement's continuous delivery runs on (issue #10):

1. a **k3s** cluster,
2. the **ArgoCD** GitOps control plane installed into its own namespace, and
3. the **WebAssembly build toolchain** proven against `crates/chirp-frontend`.

**Scope — infra only.** This tree stands up and *proves* the environment. It does **not** containerize
the app (#12) and does **not** author app Deployment/Service manifests or the ArgoCD `Application`
(#13). ArgoCD's *own* install workloads are in scope; an app `Application` is not.

Every script is idempotent / safe to re-run and declares its own prerequisites + re-run behaviour in a
header. Each area has a detailed fragment README; this top-level runbook is the entry point and the
run-order map.

---

## Layout

```
deploy/bootstrap/
├── README.md                 # this unified runbook
├── cluster/                  # k3s cluster  (owner: infra-bootstrap cluster spoke)
│   ├── install-k3s.sh        # stand up k3s (auto-selects NATIVE systemd / K3D-in-Docker); pinned v1.35.5+k3s1
│   ├── verify-cluster.sh     # real readiness proof: nodes Ready + CoreDNS Running (exits non-zero if not)
│   └── README.md             # cluster fragment: prereqs, kubeconfig retrieval, native-install blocker + remediation
├── gitops/                   # ArgoCD control plane  (owner: infra-bootstrap gitops spoke)
│   ├── install-argocd.sh     # pinned (v3.5.4) idempotent install into the argocd namespace (server-side apply)
│   ├── verify-argocd.sh      # read-only readiness proof: all core workloads Ready
│   └── README.md             # gitops fragment: API/UI access, admin-credential RETRIEVAL, RBAC posture
└── wasm/                     # build toolchain  (owner: infra-bootstrap wasm spoke)
    ├── setup-wasm-toolchain.sh  # add wasm32 target + version-matched wasm-bindgen CLI (idempotent)
    ├── build-chirp-wasm.sh      # build crates/chirp-frontend to pkg/ and verify the artifact
    ├── versions.txt             # exact pinned/recorded toolchain versions
    └── README.md                # wasm fragment: prereqs, run order, verification, blocker remediation
```

---

## Prerequisites (summary — see each fragment for detail)

| Area | Prerequisites |
|------|---------------|
| cluster | `bash`, `curl`, `kubectl`; **one** of — NATIVE: root/passwordless-sudo + systemd + cgroup v2; or K3D: a usable Docker daemon + `k3d` CLI (≥ v5.9.0). |
| gitops | a reachable cluster + kube context with cluster-admin; `kubectl` ≥ 1.27 (server-side apply); egress to `raw.githubusercontent.com` + `quay.io`. |
| wasm | `rustup` + a Rust toolchain (workspace MSRV 1.82; proven on rustc/cargo 1.97.1); egress to `static.rust-lang.org` + `crates.io`. Run from inside the repo tree. |

---

## Run order

Two independent tracks. The **cluster → gitops** track is strictly ordered (ArgoCD installs *into* the
running cluster). The **wasm** track is independent and may run in parallel / any time.

### Track 1 — cluster, then GitOps control plane

```bash
# 1a. Stand up the k3s cluster (idempotent; auto-selects native vs k3d).
deploy/bootstrap/cluster/install-k3s.sh
# 1b. Prove it: nodes Ready + CoreDNS Running (exits non-zero if not). REQUIRED before step 2.
deploy/bootstrap/cluster/verify-cluster.sh

# 2a. Install the ArgoCD control plane into the `argocd` namespace (pinned v3.5.4; idempotent).
#     Requires a kube context pointing at the cluster from step 1 (see cluster/README.md for retrieval).
deploy/bootstrap/gitops/install-argocd.sh
# 2b. Prove it: all core ArgoCD workloads Ready (read-only).
deploy/bootstrap/gitops/verify-argocd.sh
```

### Track 2 — build toolchain (independent)

```bash
# 3a. Add the wasm32 target + a wasm-bindgen CLI version-matched to the crate (idempotent).
deploy/bootstrap/wasm/setup-wasm-toolchain.sh
# 3b. Build crates/chirp-frontend to pkg/ and verify chirp.js + chirp_bg.wasm (asserts wasm magic).
deploy/bootstrap/wasm/build-chirp-wasm.sh
```

---

## Verification (what "done" means)

Each track has a real, gating verification step — no step is assumed:

| Step | Verification script | Proves |
|------|---------------------|--------|
| cluster | `cluster/verify-cluster.sh` | `kubectl wait --for=condition=Ready nodes --all` succeeds **and** CoreDNS is Running; prints the k3s version. Exits non-zero otherwise. |
| gitops | `gitops/verify-argocd.sh` | `kubectl rollout status` is green for all 6 core Deployments + the `argocd-application-controller` StatefulSet, every pod Running; prints the installed server image. Exits non-zero otherwise. |
| wasm | `wasm/build-chirp-wasm.sh` | A real `cargo build --target wasm32-unknown-unknown` + `wasm-bindgen` run produces `pkg/chirp.js` + `pkg/chirp_bg.wasm` and the `.wasm` carries the WebAssembly magic header. Exits non-zero otherwise. |

Proven on the bootstrap host (see the root `SYNTHESIS.md` and each fragment README for verbatim
evidence): k3s **v1.35.5+k3s1** (2 Ready nodes), ArgoCD **v3.5.4** (7/7 core workloads Ready),
chirp-frontend wasm build green on rustc/cargo **1.97.1** + wasm-bindgen **0.2.129**.

---

## Access

- **Cluster (kubeconfig).** Never committed. Retrieve at runtime per `cluster/README.md`:
  K3D — `k3d kubeconfig merge nswe --kubeconfig-merge-default --kubeconfig-switch-context`;
  NATIVE — `export KUBECONFIG=/etc/rancher/k3s/k3s.yaml`.
- **ArgoCD API/UI.** Stock ClusterIP (closed by default). Port-forward and browse
  `https://localhost:8080` (user `admin`):
  `kubectl -n argocd port-forward svc/argocd-server 8080:443`.
- **ArgoCD admin credential.** Never committed; generated in-cluster. Retrieve at runtime, then rotate
  and delete the bootstrap secret (see `gitops/README.md`):
  `kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath='{.data.password}' | base64 -d`.

**No secret is committed anywhere in this tree** — only runtime *retrieval* commands are documented.

---

## Environment caveat (honest reporting)

The canonical **native** k3s install (`curl -sfL https://get.k3s.io | sh -`, systemd) cannot run on a
rootless sandbox build host (no root / no non-interactive sudo / `/usr/local/bin` + `/etc/systemd/system`
not writable). `install-k3s.sh` therefore auto-selects **K3D mode** there — a *genuine* k3s cluster
running inside Docker, proven Ready — and performs the native systemd install unchanged on a privileged
host (`K3S_INSTALL_MODE=native`). The exact blocker + remediation is recorded in `cluster/README.md`
and the root `SYNTHESIS.md`.
