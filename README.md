# nswe-demo

A live demonstration of **NSWE** — the Nousix software-engineering application — building and operating a real
software project autonomously.

NSWE started this project from [`TASK.md`](TASK.md), the prompt it loaded at launch. Everything else in the repository
is authored by NSWE: the project plan on the **Projects** tab, the issues, the pull requests and their reviews, the CI/CD
in GitHub Actions, the ArgoCD deployment to a k3s cluster, and the bug reports and fixes that follow.

The repository's history — tickets, PRs, reviews, deploys — is the record of how the work was planned and done.

## Running chirp in a container

The whole `chirp` app ships as a single self-contained image — a multi-stage [`Dockerfile`](Dockerfile)
compiles the `chirp-server` release binary, builds the `chirp-frontend` WebAssembly SPA, and assembles
both into a minimal, non-root, health-checked distroless runtime image. It serves the REST API (`/api/…`)
and the SPA (`/`) out of the box with the in-memory repository — no external database.

```sh
docker build -t chirp:local .
docker run --rm -p 8080:8080 chirp:local
curl -fsS http://127.0.0.1:8080/healthz    # -> {"status":"ok",...}
# open http://127.0.0.1:8080/              # -> the SPA
```

Full build/run/configuration/healthcheck reference: [`deploy/README.md`](deploy/README.md).

## Contributions

This is a closed demonstration, not a community project. It does not accept contributions: issues, pull requests and
comments are limited to the repository's collaborators.
