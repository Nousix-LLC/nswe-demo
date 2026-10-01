## Execution Task — Permission Posture: execute

READ-WRITE. You are authorized to carry out the request below, including the external-state changes it entails, within
its scope and nothing beyond it:

- the GitHub repository `Nousix-LLC/nswe-demo` (code, branches, pull requests, reviews, issues, labels, milestones, the
  repository's GitHub Project, and GitHub Actions workflows);
- the k3s cluster on the office server, plus the ArgoCD instance you install or configure on it, for this project's
  namespaces only.

Do not touch other repositories, other clusters, or other namespaces. Never force-push to `main`, and never rewrite merged
history.

## Taskflow & Output Location

Author your taskflow (thread iterations, every forked DAG's `_FORK.md`, and all lifecycle and synthesis records) under
`~/nousix-runtime/nswe-demo/taskflow/`. The product's source of truth is the GitHub repository; the taskflow is the record
of how the work was run.

## The Request (owner's words)

"I want the NSWE to build a toy demo project, say a small web application framework and twitter clone, production
quality in rust/wasm and build and run the deployment on k3s. I don't want it to try to stuff the whole build in 1 DAG,
it should use the projects tab on github to define what each dag does in terms of new code. It should make PR's and then
schedule dags to review the PRs with the code review methodologies, and it should deploy the software on k3s and write
the deployment code using github actions and ArgoCD on k3s, then it should report bugs to issues and spawn dags to fix
the issues. (the devops work should also be dags of course) I want it to treat projects and issues like a real jira
environment to manage and schedule work and code reviews."

## What done looks like

- A small Rust/WebAssembly web-application framework, and a Twitter-style application built on it, at production quality.
- Both are built, tested and continuously delivered: GitHub Actions for CI, and ArgoCD on k3s for deployment. The
  application runs on the cluster and is reachable.
- The repository's GitHub Project, issues and pull requests are the working record of the engagement: planned work,
  in-flight work, reviewed and merged changes, deployments, and bugs found and fixed.

## Context

- You are the overseer of this engagement. Each unit of work you schedule is its own forked DAG, including code,
  reviews, devops and bug fixes. The GitHub Project and Issues are where that work is defined, tracked and closed.
- Methodologies for software engineering and code review are available through the methodology loader.
- The repository currently holds only a README.
