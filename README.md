# nswe-demo

A live demonstration of **NSWE** — the Nousix software-engineering application — building and operating a real
software project autonomously.

NSWE started this project from [`TASK.md`](TASK.md), the prompt it loaded at launch. Everything else in the repository
is authored by NSWE: the project plan on the **Projects** tab, the issues, the pull requests and their reviews, the CI/CD
in GitHub Actions, the ArgoCD deployment to a k3s cluster, and the bug reports and fixes that follow — except where the
human operator files a directive issue to steer the work (see **Human direction**).

The repository's history — tickets, PRs, reviews, deploys — is the record of how the work was planned and done.

## Human direction

The build is autonomous, but the human operator can steer it through the same GitHub workflow anyone
would use — by filing an issue on the board. NSWE picks the direction up and acts on it.

One example is in the history. Issue
[#26](https://github.com/Nousix-LLC/nswe-demo/issues/26), filed by the human operator, flagged that a
pull-request review had finished REQUEST-CHANGES while none of it was visible on GitHub — the verdict
was not on the PR and the PR was not on the board. It asked NSWE to put every PR on the board next to
the issue it closes, and to post review results on the PR. NSWE adopted that as standing practice and
the issue closed the same day; PRs and their reviews have been visible on the board since.

A second directive, [#109](https://github.com/Nousix-LLC/nswe-demo/issues/109), is a language pass: it
asks NSWE to revise the wording across every issue and the repository's written content to the project's
voice and remove AI-writing tells — without changing the substance of any work — and to keep new writing
clean, with the reviewer checking for it in every review.

The record is not only what NSWE built — it is also where a human corrected course and the autonomous
build followed.

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
