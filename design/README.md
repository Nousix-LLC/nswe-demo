# chirp — Design

## Overview

This directory is the **product design for `chirp`**: what the product must be to be a *releasable*
Twitter-style product, and the concrete, testable bar that defines "releasable". It is the engagement's
first product design, grounded in the product as it stands at commit `c805525`.

It describes **what the product must do, for whom, and why, and what "done" looks like from the user's
and operator's side — not how it is built.** Every topic with an engineering "how" (durable persistence,
authentication, sessions, rate limiting, backups) is stated here as a **requirement with a testable bar**;
the implementation is deferred to later code work. No product code is changed by this design.

Each requirement carries a stable id in an area register — `REQ-PROD-*` (product), `REQ-DOM-*` (domain &
application), `REQ-UX-*` (experience), `REQ-ID-*` (identity & trust), `REQ-OPS-*` (operational readiness) —
and the release bar consolidates them into `BAR-*` criteria, each of which is checkable against the live
product by someone who was not in the room. That shared id scheme is what makes requirements, bar criteria,
and the delivery backlog traceable to one another.

## Document index

Read in this order for the first pass. Product requirements set the foundation every other area traces
back to; the release bar sits on top of all five concern designs.

| Document | What it covers |
|----------|----------------|
| [`product-requirements.md`](product-requirements.md) | The foundation: what chirp is and which product kind(s) apply, the users and personas, their end‑to‑end journeys (first visit, daily use, leaving), and the testable product requirements and acceptance criteria. Sole owner of the `REQ-PROD-*` register; every other area traces back here. |
| [`domain-application-design.md`](domain-application-design.md) | The application and its domain as **requirements and structure**: the domain model and invariants, how the product's parts divide and their responsibilities (held to the `ferric` / `chirp-types` / `chirp-server` crates), the **durable‑persistence requirement** (the in‑memory store is a release blocker), interfaces and entry points, and where state lives. Owner of `REQ-DOM-*`. |
| [`experience-design.md`](experience-design.md) | How chirp looks and behaves: information architecture and interaction, every screen and flow in the inventory with its empty / error / loading / not‑found / session‑expired / validation / busy / success states, the visual language and design system, responsive layout, and accessibility (WCAG 2.1 AA). Owner of `REQ-UX-*`. |
| [`identity-security.md`](identity-security.md) | Identity, security and trust as requirements: accounts and their full lifecycle (sign‑up / sign‑in / recovery, deletion, data export), real authentication and safe sessions, authorization (what each actor may see and change), abuse‑resistance and privacy basics, and the threat model this surface implies. Owner of `REQ-ID-*`. |
| [`operational-readiness.md`](operational-readiness.md) | What must be true in production beyond "it runs": durable data, backups and a proven recovery path, limits and quotas, observability, and status/incident response — building on the persistence model fixed by the domain design. Owner of `REQ-OPS-*`. |
| [`release-bar.md`](release-bar.md) | The consolidated, testable definition of "releasable": the honest **MVP‑vs‑releasable scoping** (what makes chirp usable early vs. what release requires on top, with no hidden cuts), the **release bar** (`BAR-*`, each with a verification method and the `REQ-*` it enforces), and a **completeness & gap review** that records an in/out‑of‑scope judgment for every surface a product of this kind is expected to have. Owner of `BAR-*`. |

**Where the release bar lives:** [`release-bar.md`](release-bar.md). That is the single document to open
to answer "is chirp releasable yet?" — each `BAR-*` names a verification method (`manual_user_check`,
`automated_test`, `inspection`, or `operational_check`) a stranger could run against the live product.

## How the bar and the backlog relate

The design and the bar describe **what must be true**; the **delivery backlog** on the project board
describes **the work to make it true**. They are connected by the shared id scheme:

- Each requirement (`REQ-<AREA>-NNN`) states a condition from the user's or operator's side with testable
  acceptance criteria and a `release_gating` flag.
- Each release‑bar criterion (`BAR-NNN`) is a concrete, checkable release condition that references the
  `REQ-*` it enforces and names how to verify it.
- Each **backlog issue** carries acceptance criteria that cite the `REQ-*`/`BAR-*` it satisfies and the
  design section it traces to, and sits under exactly one **epic** — where one epic is a single coherent,
  landable code goal. The epics are ordered to reach a usable product early (the `usable_early` stage of
  the bar) and the full bar (`releasable`) after.

So any release‑gating requirement is covered by at least one bar criterion, and every bar criterion is
covered by at least one backlog issue under an epic. The backlog itself lives on the repository's project
board (epics and issues), not in this directory; this design is what those issues trace back to.
