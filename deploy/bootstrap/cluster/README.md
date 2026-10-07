# Cluster bootstrap (k3s) — runbook fragment

This fragment covers standing up and proving the **k3s** deployment cluster for `nswe-demo`.
The synthesis gate assembles it into the unified `deploy/bootstrap/README.md`. ArgoCD and app
workloads are **separate** work items that install *into* this cluster.

## Prerequisites

- `bash`, `curl`, `kubectl` client on PATH.
- **One** install mode must be satisfiable:
  - **NATIVE** (production host): root or passwordless `sudo`, `systemd` as init, cgroup v2.
  - **K3D** (rootless / dev / CI / sandbox): a usable Docker daemon (membership in the `docker`
    group suffices) and the `k3d` CLI (>= v5.9.0).
- Pinned version: **k3s v1.35.5+k3s1** (native tag `v1.35.5+k3s1`; k3d image
  `docker.io/rancher/k3s:v1.35.5-k3s1`).

## Run order

```bash
cd deploy/bootstrap/cluster
./install-k3s.sh        # stands up the cluster (idempotent; auto-selects native vs k3d)
./verify-cluster.sh     # real readiness proof: nodes Ready + CoreDNS Running (exits non-zero if not)
```

Mode selection is automatic; override with `K3S_INSTALL_MODE=native|k3d`. Cluster name defaults to
`nswe` (override `K3S_CLUSTER_NAME`). `install-k3s.sh` is safe to re-run: NATIVE re-runs the upstream
installer; K3D creates the cluster only if absent (`K3S_FORCE_RECREATE=1` forces delete+recreate).

## Obtaining kubeconfig access (NO secret is committed)

The kubeconfig contains client credentials and is **never** staged or committed. Retrieve it at
runtime on the host that owns the cluster:

- **K3D mode** — the context `k3d-<cluster>` (default `k3d-nswe`) is merged into your default
  kubeconfig (`${KUBECONFIG:-~/.kube/config}`) by `install-k3s.sh`. To (re)fetch or export it:
  ```bash
  k3d kubeconfig merge nswe --kubeconfig-merge-default --kubeconfig-switch-context   # merge + select
  k3d kubeconfig get nswe > "$SOME_PRIVATE_PATH"                                      # export (treat as a secret)
  kubectl config use-context k3d-nswe
  ```
- **NATIVE mode** — k3s writes the admin kubeconfig to `/etc/rancher/k3s/k3s.yaml`. A client
  consumes it via:
  ```bash
  export KUBECONFIG=/etc/rancher/k3s/k3s.yaml     # or copy to ~/.kube/config (chmod 600)
  kubectl config use-context default
  ```
  (The installer here uses `--write-kubeconfig-mode 0644` so a non-root client can read it; tighten
  for shared hosts. For remote clients, replace the `server:` host `127.0.0.1` with the node's
  reachable address and keep the CA/credential data private.)

## Verification (what "Ready" means here)

`verify-cluster.sh` executes a **real** check and gates on it:
`kubectl wait --for=condition=Ready nodes --all`, then confirms CoreDNS is `Running` in
`kube-system`, and prints the k3s version. Observed on this host: both `k3d-nswe-server-0`
(control-plane) and `k3d-nswe-agent-0` **Ready** at `v1.35.5+k3s1`; CoreDNS, local-path-provisioner,
metrics-server, and traefik all healthy.

## Native production install — blocker on this build host + remediation

This build host is a **rootless sandbox**: `uid=1001`, non-interactive `sudo` requires a password,
and `/usr/local/bin` + `/etc/systemd/system` are not writable — so the canonical
`curl -sfL https://get.k3s.io | sh -` (systemd) install cannot run here. The cluster above was
therefore stood up in **K3D mode** (real k3s in Docker; proven Ready). To use the NATIVE systemd
install on a production host instead:

1. Run on a host where you have root or passwordless `sudo`, `systemd` as PID 1, and cgroup v2.
2. `K3S_INSTALL_MODE=native ./install-k3s.sh` (installs k3s `v1.35.5+k3s1` + the `k3s.service` unit).
3. `KUBE_CONTEXT=default KUBECONFIG=/etc/rancher/k3s/k3s.yaml ./verify-cluster.sh`.
