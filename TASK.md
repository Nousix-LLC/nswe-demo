## Execution Task — Permission Posture: execute

READ-WRITE. You are authorized to carry out the request below, including the external-state changes it entails, within
its scope and nothing beyond it:

- the GitHub repository `Nousix-LLC/nswe-demo` (code, branches, pull requests, reviews, issues, labels, milestones, the
  repository's GitHub Project, and GitHub Actions workflows);
- the project's k3s cluster, plus the ArgoCD instance you install or configure on it, for this project's namespaces only.

Do not touch other repositories, other clusters, or other namespaces. Never force-push to `main`, and never rewrite merged
history.

## Taskflow & Output Location

Author your taskflow (thread iterations, every forked DAG's `_FORK.md`, and all lifecycle and synthesis records) under
`~/nousix-runtime/nswe-demo/taskflow/`. The product's source of truth is the GitHub repository; the taskflow is the record
of how the work was run.

## The Request

Build a demo project: a small web-application framework, and a Twitter-style clone built on it, at production quality,
in Rust/WebAssembly. Build, run and deploy it on k3s.

Don't try to do the whole build in one DAG. Use the repository's GitHub Project to define what each DAG does in terms of
new code. Changes land as pull requests, and you schedule DAGs to review those pull requests with the code-review
methodologies. Deploy the software on k3s, and write the deployment code with GitHub Actions and ArgoCD on k3s; the
devops work is DAGs too. Report bugs as issues and spawn DAGs to fix them.

Treat the GitHub Project and Issues like a real Jira environment: they are where you manage and schedule the work and the
code reviews.

## What done looks like

- A small Rust/WebAssembly web-application framework, and a Twitter-style application built on it, at production quality.
- Both are built, tested and continuously delivered: GitHub Actions for CI, and ArgoCD on k3s for deployment. The
  application runs on the cluster and is reachable.
- The repository's GitHub Project, issues and pull requests are the working record of the engagement: planned work,
  in-flight work, reviewed and merged changes, deployments, and bugs found and fixed.

## Context

- You are the overseer of this engagement. Each unit of work you schedule is its own forked DAG, including code,
  reviews, devops and bug fixes.
- Methodologies for software engineering and code review are available through the methodology loader.
- The repository currently holds only this file and a README.
