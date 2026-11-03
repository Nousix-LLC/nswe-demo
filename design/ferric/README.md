# Ferric 1.0 — Framework Design

## Overview

This directory is the **framework design for `ferric`**: what the `crates/ferric` Rust/WebAssembly UI
framework must become for a credible 1.0, and the concrete, testable bar that defines "1.0". It is
grounded in the framework as it stands at commit `276138f`.

It describes **what the framework must do, for whom, and why, and what "done" looks like from a framework
user's side — not how it is built.** Every topic with an engineering "how" (the hydration algorithm, the
macro expansion, the router matcher) is stated here as a **requirement with a testable bar**; the
implementation is deferred to later code work. No framework code is changed by this design.

Ferric's users are Rust/WASM application authors — `chirp` is the first real consumer — who build
browser UIs against the framework's public API surface. The design covers what they must be able to
express, what they can rely on in production, and what the 1.0 bar looks like from their side.

Each requirement carries a stable id in an area register — `REQ-FW-*` (framework scope), `REQ-SSR-*`
(server-side rendering + hydration), `REQ-API-*` (authoring model), `REQ-ASYNC-*` (async, forms, events),
`REQ-RT-*` (router), `REQ-HTTP-*` (HTTP/JSON client), `REQ-DX-*` (docs & developer experience),
`REQ-REL-*` (release & distribution), `REQ-PERF-*` (performance & benchmarks) — and the release bar
consolidates them into `BAR-FERRIC-*` criteria, each of which is checkable against the framework by
someone who was not in the room. That shared id scheme is what makes requirements, bar criteria, and the
delivery backlog traceable to one another.

## Document index

Read in this order for the first pass. Scope and requirements set the foundation every other area traces
back to; the release bar sits on top of all eight capability designs.

| Document | What it covers |
|----------|----------------|
| [`scope-and-requirements.md`](scope-and-requirements.md) | The foundation: what Ferric is and which framework kind applies, the users and what they build, the 1.0 requirements and acceptance criteria, the MVP-vs-1.0 staging line, and the explicit out-of-1.0 list (scoped CSS, devtools, hot reload, desktop/mobile). Sole owner of the `REQ-FW-*` register; every other area traces back here. |
| [`ssr-hydration.md`](ssr-hydration.md) | Server-side rendering via Axum integration and the hydrate-without-re-render architecture: how server-rendered HTML becomes interactive on the client by hydration alone, how application state travels from server to client, and how event handlers wire up at hydration time. The one must-have for 1.0. Owner of `REQ-SSR-*`. |
| [`authoring-model.md`](authoring-model.md) | The `view!` macro (JSX-style template DSL), component definition as functions, props, children, slots, composition patterns, and the context system for cross-component state. Owner of `REQ-API-*`. |
| [`async-forms-events.md`](async-forms-events.md) | Async resource loading with suspense (loading, error, and success states), form handling with two-way binding and validation, and the full DOM event coverage a Ferric application can depend on. Owner of `REQ-ASYNC-*`. |
| [`router-and-http.md`](router-and-http.md) | The client-side router (URL params, nested routes, `<Link>`, navigation guards) and the HTTP/JSON client, both reconciled with the frozen `ferric-router` and `ferric-http` public surfaces that already exist in the workspace. Extends and supersedes issues #5 and #6 rather than duplicating them. Owner of `REQ-RT-*` and `REQ-HTTP-*`. |
| [`docs-and-dx.md`](docs-and-dx.md) | What 1.0 documentation must cover: guide, tutorial, API docs, runnable examples, and a `cargo-generate` starter template — and the developer-experience bar a first-time user can hold the release to. Owner of `REQ-DX-*`. |
| [`release-and-distribution.md`](release-and-distribution.md) | crates.io publishing across the multi-crate workspace (`ferric`, `ferric-router`, `ferric-http`), semver and breaking-change policy, changelog discipline, and the versioning and compatibility commitments a 1.0 implies. Owner of `REQ-REL-*`. |
| [`performance-and-benchmark.md`](performance-and-benchmark.md) | The js-framework-benchmark methodology applied to Ferric: what to measure (update speed, memory, bundle size), how to run it reproducibly, and the honest peer comparison against Leptos, Dioxus, Sycamore, and Yew that any 1.0 candidate must be able to stand behind. Owner of `REQ-PERF-*`. |
| [`release-bar.md`](release-bar.md) | The consolidated, testable definition of "1.0": the **MVP-vs-1.0 staging** (what makes Ferric usable early vs. what 1.0 requires on top, with no hidden cuts), the **release bar** (`BAR-FERRIC-*`, each with a verification method and the `REQ-*` it enforces), and a **completeness & gap review** that records an in/out-of-scope judgment for every surface a Rust/WASM framework of this kind is expected to have. Owner of `BAR-FERRIC-*`. |
| [`release-plan.md`](release-plan.md) | The ordered build plan: epics and work items under epic [#1](https://github.com/Nousix-LLC/nswe-demo/issues/1), each a coherent landable capability with acceptance criteria that trace to `REQ-*`/`BAR-*` ids, in dependency order. Records the explicit fold of router issue [#5](https://github.com/Nousix-LLC/nswe-demo/issues/5) and HTTP issue [#6](https://github.com/Nousix-LLC/nswe-demo/issues/6). |

**Where the release bar lives:** [`release-bar.md`](release-bar.md). That is the single document to open
to answer "is Ferric 1.0 ready to ship?" — each `BAR-FERRIC-*` names a verification method a reviewer or
code DAG could run against the framework.

## How the bar and the backlog relate

The design and the bar describe **what must be true**; the **delivery backlog** on the project board
describes **the work to make it true**. They are connected by the shared id scheme:

- Each requirement (`REQ-<AREA>-NNN`) states a condition from the framework user's side with testable
  acceptance criteria and a `stage` field (`mvp` or `one_oh`).
- Each release-bar criterion (`BAR-FERRIC-NNN`) is a concrete, checkable release condition that
  references the `REQ-*` it enforces and names how to verify it, and carries the stage at which it
  must hold.
- Each **backlog issue** carries acceptance criteria that cite the `REQ-*`/`BAR-*` it satisfies and the
  design section it traces to, and sits under exactly one **epic** under issue [#1](https://github.com/Nousix-LLC/nswe-demo/issues/1) — where one epic is a
  single coherent, landable code goal. The epics are ordered so the most foundational capabilities land
  first (SSR + hydration is the gating must-have) and the full bar follows.

So every `one_oh`-gating requirement is covered by at least one bar criterion, and every bar criterion is
covered by at least one backlog issue under an epic. The backlog itself lives on the repository's project
board (epics and issues), not in this directory; this design is what those issues trace back to.

## The `contracts/` directory

[`contracts/`](contracts/) ships with this design and documents the traceability scheme in machine-readable
form. [`id_and_traceability_schema.yaml`](contracts/id_and_traceability_schema.yaml) defines the id
namespaces, stage vocabulary, and verification-method vocabulary the `REQ-*` and `BAR-FERRIC-*` ids
follow. [`design_artifact_map.yaml`](contracts/design_artifact_map.yaml) records which spoke owns each
document and where it lands in the repository.
