# Ferric — Documentation & Developer Experience

This document designs the **documentation and developer-experience bar** for Ferric 1.0: the guide, the
tutorial, the API documentation, the examples set, and a `cargo-generate` starter template, together with the
one acceptance path that ties them together — a stranger scaffolds from the starter and follows the guide to a
working app. It owns the `REQ-DX-*` register.

**What this document settles, and what it does not.** It states *what documentation and tooling must exist and
what "done" looks like from a new user's side* — the requirement and its testable bar — not the prose of the
guide, the tutorial, or the API-doc comments. Writing that prose is downstream build work for the epics the
release plan files; this document is the design those epics build against. Where a DX requirement is the
user-facing half of a capability another area owns (macro diagnostics, the SSR build, performance numbers),
this document states the DX requirement and cross-references that area's register read-only.

**What, not how.** A `REQ-DX-*` requirement reads as something a reviewer or a new user could check against the
repository and reach the same yes/no: a document that exists and is accurate, a starter that scaffolds and
builds, a tutorial whose end state compiles. It does not choose the doc toolchain internals, the exact chapter
list, or the starter's file layout — those are the build's to decide within the bar.

**Grounding.** Observations of "Ferric as it exists today" are grounded in `Nousix-LLC/nswe-demo` at the
baseline commit `276138f` on `main`: a crate `README.md` for each of `ferric`, `ferric-router`, and
`ferric-http`, and one runnable `examples/counter.rs` (plus the router's `examples/app.rs`). There is no guide,
no tutorial, no rendered API-doc site, and no zero-to-app starter. Peer-framework DX is compared by name
(Leptos, Dioxus, Yew) against each framework's documented, public tooling.

The sections below are: the DX surface a framework 1.0 owes its users; the three designed areas (the learning
path, API docs, the starter and the new-user acceptance path); the `REQ-DX-*` register; the named peer
comparison; and the out-of-1.0 DX note.

---

## 1. The DX surface a framework 1.0 owes its users

A Rust/WASM UI framework is judged in its first minute by whether a stranger can start. The scope document's
personas fix who those strangers are, and each one leans on a different part of the DX surface:

- **A1, the evaluating author**, decides in an afternoon from the README/landing, the guide, one runnable
  example, and visible performance numbers (`REQ-FW-002`, `REQ-FW-011`, `REQ-FW-012`). A framework with thin
  docs is ruled out before its internals are read.
- **A2, the application author**, lives in the guide, the tutorial, the examples, and the API docs while
  building a real app, and needs them to show how the pieces compose (`REQ-FW-008`, `REQ-FW-011`).
- **A4, the operator of the build**, needs the build-and-ship path documented and reproducible, including the
  SSR server integration (`REQ-FW-003`, `REQ-FW-004` server side, `REQ-FW-013`).

From those needs, the DX surface a credible 1.0 must have is: a **conceptual guide**, a **start-to-finish
tutorial**, **rendered API documentation** for the public surface, a **curated examples set** across the
capability span, a **`cargo-generate` starter**, **build/toolchain documentation** (client and SSR), a
**troubleshooting/diagnostics** entry, and an **accurate README/landing** for the evaluator. The integrating
test over all of them is a single **"new user → working app" acceptance path**. Each of these is registered
below; none is assumed present because the code works.

Today's baseline covers exactly one corner of this surface — three crate READMEs and the counter — so every
`REQ-DX-*` below closes a stated gap rather than restating what exists.

---

## 2. The learning path (guide, tutorial, examples)

The learning path is the ordered route a new author walks: land on the README, read the guide to understand the
model, follow the tutorial to build something real, and reach for the examples when building their own. These
are facets of one onboarding arc, so they are designed together and cross-reference one register.

### 2.1 The conceptual guide ("the book")

A conceptual guide — the shape of the **Leptos Book** or the **Dioxus** / **Yew** guides — explains the model an
author must hold: fine-grained reactivity (signals/memos/effects, batching, scoped cleanup), the component/view
model, the declarative authoring macro (`REQ-API-*`), SSR + hydration (`REQ-SSR-*`), routing (`REQ-RT-*`), and
data/async/forms (`REQ-ASYNC-*`, `REQ-HTTP-*`). The guide's job is comprehension, not copy-paste: an author who
reads it can predict how Ferric will behave. Registered as `REQ-DX-001`.

### 2.2 The start-to-finish tutorial

A tutorial takes an author from an empty project to a working, non-trivial app in ordered steps — the step the
counter example cannot carry alone. Its binding property is that **its end state compiles and runs**: the code a
reader types by following it is real code, not prose that drifts from the API. Registered as `REQ-DX-002`, with
the end-state build as its acceptance check.

### 2.3 The examples set

The baseline has `counter.rs` (reactivity) and the router's `app.rs` (routing). A 1.0 needs a curated set that
covers the capability span an author must see work: reactivity + keyed lists, routing, async data with the four
suspense states (loading/loaded/empty/error), forms with two-way binding and the full event range, and an **SSR
+ hydrate** example (the must-have, `REQ-FW-004`). Each example builds to `wasm32-unknown-unknown`. Registered
as `REQ-DX-004`; the client-side examples are `usable_early`, the SSR example is `one_oh`.

### 2.4 The README/landing for the evaluator

The crate README and the repository landing are where A1's afternoon starts. A 1.0 README states what Ferric is
in a sentence, compares it to named peers, links to the guide / API docs / benchmark numbers, and shows a
runnable hello-app. The baseline READMEs are strong on architecture but predate the guide, the benchmark, and
the starter they must link to. Registered as `REQ-DX-009`.

---

## 3. API documentation

### 3.1 Rendered API docs for the public surface

Rust's ecosystem convention is **rustdoc rendered on docs.rs**, built automatically for every version published
to crates.io. The 1.0 bar is that every public item across the preludes (`ferric`, `ferric-router`,
`ferric-http`) carries doc comments, each crate has a crate-level overview, intra-doc links resolve, and the
doc build is clean. Because `REQ-FW-001` puts the crates on crates.io, docs.rs is the delivery surface and needs
no bespoke hosting. Registered as `REQ-DX-003`, verified by doc inspection plus a clean `cargo doc` and passing
doctests.

### 3.2 Diagnostics documentation

Much of the day-to-day experience of a framework is what happens when an author gets it wrong. The macro and
runtime diagnostics themselves are owned by the authoring model (`REQ-FW-014` → `REQ-API-*`); the DX half is a
documented **troubleshooting / common-mistakes** entry that names the frequent errors (a malformed `view!`, a
misused prop, a missing mount point) and what to do about them. Registered as `REQ-DX-008`, cross-referencing
`REQ-API-*` read-only.

### 3.3 Docs that do not rot

Docs and examples that drift from the shipped API are worse than none, because they mislead. The 1.0 bar keeps
them in sync mechanically: the examples and the tutorial's end state compile in CI, doctests run, and docs.rs
builds per published version. Registered as `REQ-DX-010`.

---

## 4. The cargo-generate starter and the new-user acceptance path

### 4.1 The starter template

The standard Rust scaffolding mechanism is **`cargo-generate`**: an author runs `cargo generate --git <repo>`
and gets a ready project. Leptos ships `start-trunk` (a CSR starter consumed exactly this way) and `cargo leptos
new` for SSR starters; Dioxus ships `dx new` with selectable templates (bare-bones, jumpstart, workspace); Yew
documents a Trunk-based scaffold. Ferric's starter scaffolds a buildable app with the toolchain already wired:
the `ferric` dependencies, an `index.html` with a mount point, the Trunk/`wasm-bindgen` configuration, and a
minimal app that reactivity and the `view!` macro already drive. The author adds nothing to get a first build.
Registered as `REQ-DX-005`.

A client-side-rendered starter is the floor; because SSR + hydration is the 1.0 must-have, the starter (or a
second starter variant) scaffolds the SSR + Axum server integration so a new author reaches the gating
capability from the documented path, not by hand-assembly. The SSR variant is `one_oh`.

### 4.2 The "new user → working app" acceptance path

This is the integrating requirement the whole area answers to, and the one the brief's success criterion names.
It is a single scripted path a stranger can follow using only documented commands and the standard toolchain:

1. Install the prerequisites the docs name (stable Rust, the `wasm32-unknown-unknown` target, Trunk / the
   `wasm-bindgen` tooling).
2. Scaffold a project with `cargo generate` from the Ferric starter.
3. Build and serve it (`trunk serve`) and see the app run in a browser.
4. Follow the tutorial from that project to a working multi-view, data-backed app whose end state compiles.

The acceptance check is that someone who has never used Ferric completes steps 1–4 from the documentation alone.
Registered as `REQ-DX-006`; the build/toolchain documentation it depends on is `REQ-DX-007`.

---

## 5. Requirements & acceptance criteria (`REQ-DX-*`)

Each requirement states what must be true from a new user's or an author's side; its acceptance criteria are
checkable by a reviewer who was not in the room. `stage` is `usable_early | one_oh` (per the scope document's
MVP-vs-1.0 line); `release_gating` is `true` for every requirement here (the DX deferral is recorded in §7 as
`REQ-FW-017`, not minted here). `verification_method` is drawn from the contract vocabulary
(`../contracts/id_and_traceability_schema.yaml`): `doc_inspection`, `example_build`, `automated_test`.

### REQ-DX-001 — A conceptual guide
- **Statement:** The repository provides a conceptual guide that explains Ferric's model — reactivity, the
  component/view model, the authoring macro, SSR + hydration, routing, and data/async/forms — so an author can
  learn how Ferric behaves without reading its source.
- **Rationale:** Every credible peer ships a conceptual guide (the Leptos Book; the Dioxus and Yew guides); an
  evaluating author (A1) and an application author (A2) learn the model from it, and a framework whose only true
  documentation is its source cannot be evaluated or adopted at scale.
- **Acceptance criteria:** A guide exists in the repository covering each of the named topics; a reviewer
  confirms each topic is present and that its description matches the shipped behavior for a stated example.
- **Stage:** one_oh · **Release gating:** true · **Verification:** doc_inspection
- **Traceability:** `REQ-FW-011`; JA1, JA2; A1, A2.

### REQ-DX-002 — A start-to-finish tutorial whose end state compiles
- **Statement:** The repository provides an ordered tutorial that takes an author from an empty project to a
  working, non-trivial app, and the tutorial's end-state code compiles and runs.
- **Rationale:** Adoption is decided in the first afternoon; a tutorial is how an author reaches a first real
  success, and a tutorial whose code has drifted from the API misleads rather than teaches.
- **Acceptance criteria:** A tutorial exists with ordered steps to a stated end app; the end-state source builds
  to `wasm32-unknown-unknown` and runs the described behavior using only documented commands. (`example_build`
  on the end state; `doc_inspection` on the steps.)
- **Stage:** one_oh · **Release gating:** true · **Verification:** example_build, doc_inspection
- **Traceability:** `REQ-FW-002`, `REQ-FW-011`; JA1, JA2; A2.

### REQ-DX-003 — Rendered API documentation for the public surface
- **Statement:** Every public item across the `ferric`, `ferric-router`, and `ferric-http` preludes is
  documented, each crate carries a crate-level overview, and the API docs render cleanly (the docs.rs
  convention for crates.io crates).
- **Rationale:** rustdoc on docs.rs is the Rust ecosystem's standard API-doc surface and is produced
  automatically once the crates publish (`REQ-FW-001`); an author (A2/A3) builds against the API docs, and
  undocumented public items force them back into the source.
- **Acceptance criteria:** `cargo doc` builds with no warnings for broken intra-doc links; a reviewer confirms
  every public prelude item and each crate root has documentation; doctests compile and pass
  (`cargo test --doc`).
- **Stage:** one_oh · **Release gating:** true · **Verification:** doc_inspection, automated_test
- **Traceability:** `REQ-FW-011`; JA2; A2, A3.

### REQ-DX-004 — A curated examples set across the capability span
- **Statement:** The repository provides runnable examples covering reactivity + keyed lists, client routing,
  async data with loading/loaded/empty/error states, forms with two-way binding and the full event range, and
  SSR + hydration; each example builds to WebAssembly.
- **Rationale:** Examples are how authors confirm a capability works and how they start their own; the baseline
  has only the counter and the router app, which do not show async, forms, or SSR — the surfaces an author most
  needs to see compose.
- **Acceptance criteria:** Each listed example exists and builds for `wasm32-unknown-unknown`; the SSR example
  renders on the server and hydrates without a client re-render (exercising `REQ-FW-004` from the examples set).
- **Stage:** usable_early for the client-side examples; one_oh for the SSR example · **Release gating:** true ·
  **Verification:** example_build
- **Traceability:** `REQ-FW-008`, `REQ-FW-011`, `REQ-FW-004`; JA2, JA3; A2.

### REQ-DX-005 — A cargo-generate starter template
- **Statement:** The repository provides a `cargo-generate` starter that scaffolds a buildable Ferric app with
  the toolchain wired (dependencies, an `index.html` mount point, the Trunk/`wasm-bindgen` configuration), and
  an SSR + Axum starter variant for the server-rendered path.
- **Rationale:** `cargo generate --git <repo>` is the ecosystem-standard scaffold (Leptos `start-trunk` and
  `cargo leptos new`; Dioxus `dx new`); without a starter a new author assembles the WASM toolchain by trial
  and error, which the scope document names as the current gap (`REQ-FW-002`).
- **Acceptance criteria:** Running the documented `cargo generate` command against the starter produces a
  project that builds to `wasm32-unknown-unknown` and serves in a browser with no edits; the SSR variant builds
  the server-integrated app from documented steps.
- **Stage:** one_oh · **Release gating:** true · **Verification:** example_build
- **Traceability:** `REQ-FW-002`; JA1; A1, A2, A4.

### REQ-DX-006 — The "new user → working app" acceptance path
- **Statement:** A stranger can go from nothing to a running Ferric app and then to a working multi-view,
  data-backed app, following only the documented path (prerequisites → `cargo generate` scaffold → serve → the
  tutorial).
- **Rationale:** This is the integrating test of the whole DX surface — the one the framework's first minute is
  judged by — and the brief's primary success criterion. Each piece can exist and still fail to connect; this
  requirement checks the connection end to end.
- **Acceptance criteria:** A reviewer who has not used Ferric completes the four documented steps (install
  prerequisites, scaffold from the starter, build and serve, follow the tutorial to its compiling end state)
  using only the repository's documentation and the standard toolchain, and reaches a running app at each stage.
- **Stage:** one_oh · **Release gating:** true · **Verification:** example_build, doc_inspection
- **Traceability:** `REQ-FW-002`, `REQ-FW-011`; JA1, JA2; A1, A2. Depends on `REQ-DX-002`, `REQ-DX-005`,
  `REQ-DX-007`.

### REQ-DX-007 — Build and toolchain documentation (client and SSR)
- **Statement:** The build-and-ship path is documented and reproducible on stable Rust and the standard WASM
  toolchain — the client WASM build and the SSR server-integration build — with the compatible toolchain
  versions stated.
- **Rationale:** The operator of the build (A4) needs the path to be documented rather than folklore, including
  how the SSR render wires into a real server (the repository's Axum); this is the DX half of `REQ-FW-003` and
  the build side of `REQ-FW-004`.
- **Acceptance criteria:** Following the documented build steps on current stable Rust targeting
  `wasm32-unknown-unknown` produces an optimized bundle and (for SSR) a server that serves server-rendered HTML
  and hydrates; nothing in the required path needs a nightly feature; version ranges are stated.
- **Stage:** usable_early for the client build; one_oh for the SSR build · **Release gating:** true ·
  **Verification:** doc_inspection, example_build
- **Traceability:** `REQ-FW-003`, `REQ-FW-004` (server side); JA4; A4. Build-size numbers → `REQ-PERF-*`.

### REQ-DX-008 — Troubleshooting and diagnostics documentation
- **Statement:** The documentation includes a troubleshooting / common-mistakes entry that names the frequent
  authoring errors and their resolutions, surfacing the macro and runtime diagnostics authored under the
  authoring model.
- **Rationale:** What happens when an author gets it wrong is a large part of whether a Rust frontend framework
  is kept or abandoned; the diagnostics themselves are `REQ-API-*` / `REQ-FW-014`, but an author also needs them
  explained where they look for help.
- **Acceptance criteria:** A troubleshooting section exists listing a stated set of common mistakes (for example
  a malformed `view!`, a misused prop, a missing mount element) with resolutions; a reviewer reproduces the
  stated cases and confirms the documented guidance matches the observed diagnostic. Representative macro
  diagnostics are covered by a `trybuild` test owned under `REQ-API-*`.
- **Stage:** usable_early · **Release gating:** true · **Verification:** doc_inspection, automated_test
- **Traceability:** `REQ-FW-014`; JA2; A2, A3. Diagnostics mechanism → `REQ-API-*` (read-only).

### REQ-DX-009 — An accurate README / landing for the evaluating author
- **Statement:** The crate README and repository landing state what Ferric is in a sentence, compare it to named
  peers, link to the guide, the API docs, and the benchmark numbers, and show a runnable hello-app.
- **Rationale:** The evaluating author (A1) starts here and decides fast; a landing that omits the comparison,
  the performance numbers, or the getting-started link loses the evaluator before the framework's strengths are
  reached.
- **Acceptance criteria:** A reviewer confirms the README/landing contains the one-sentence definition, a
  named-peer comparison, working links to the guide / API docs / benchmark results, and a runnable hello-app
  snippet; the snippet builds.
- **Stage:** one_oh · **Release gating:** true · **Verification:** doc_inspection, example_build
- **Traceability:** `REQ-FW-002`, `REQ-FW-011`, `REQ-FW-012`; JA1; A1.

### REQ-DX-010 — Documentation and examples kept in sync with the release
- **Statement:** The examples, the tutorial's end state, and the doctests are built in CI against the shipped
  API, and API docs build per published version, so the documentation does not drift from the code.
- **Rationale:** Drifted docs mislead; a 1.0 that promises its docs teach the real API must keep them mechanically
  in sync, not by manual vigilance.
- **Acceptance criteria:** CI builds every example and the tutorial end state and runs the doctests on each
  change; a failed example build or doctest fails the gate; docs.rs (or the equivalent doc build) produces
  rendered docs for the published version.
- **Stage:** one_oh · **Release gating:** true · **Verification:** automated_test, doc_inspection
- **Traceability:** `REQ-FW-011`; JA4; A2. Publishing/versioning → `REQ-REL-*` (read-only).

### Sub-area → requirement coverage

| DX sub-area | Covering `REQ-DX-*` |
|---|---|
| The learning path (guide, tutorial, examples, README) | 001, 002, 004, 009 |
| API docs | 003, 008, 010 |
| The cargo-generate starter & new-user path | 005, 006, 007 |

Every requirement cites at least one upstream `REQ-FW-*`, so the gating reqs the release bar consolidates
(`BAR-FERRIC-*`) trace back to a framework-level need (contract identities I2, I3).

---

## 6. Named peer comparison

Ferric's DX is measured against the frameworks an evaluating author (A1) weighs, by each one's documented public
tooling:

| Framework | Guide / tutorial | API docs | Starter / scaffold | Build tool |
|---|---|---|---|---|
| **Leptos** | The Leptos Book (guide + tutorial) | rustdoc on docs.rs | `start-trunk` via `cargo generate --git`; `cargo leptos new` (SSR) | `trunk`, `cargo-leptos` |
| **Dioxus** | Guide + tutorial on the docs site | rustdoc on docs.rs | `dx new` with selectable templates (bare-bones / jumpstart / workspace) | `dx` CLI |
| **Yew** | Guide + tutorial on the docs site | rustdoc on docs.rs | Documented Trunk-based scaffold | `trunk` |
| **Ferric (today, `276138f`)** | None (three crate READMEs) | None rendered (doc comments only) | None (counter + router app examples) | Trunk / `wasm-bindgen` (documented in READMEs) |
| **Ferric (1.0 bar, this design)** | Guide (`REQ-DX-001`) + tutorial (`REQ-DX-002`) | rustdoc on docs.rs (`REQ-DX-003`) | `cargo generate` starter, CSR + SSR (`REQ-DX-005`) | Trunk / `wasm-bindgen`, documented (`REQ-DX-007`) |

The assessment: today's Ferric sits below every named peer on DX — it has no guide, no tutorial, no rendered API
site, and no scaffold, where all three peers ship each. The 1.0 bar above brings Ferric to parity on the
surfaces an evaluator checks first. Ferric deliberately leans on the ecosystem-standard tools (`cargo-generate`,
Trunk, docs.rs) rather than a bespoke CLI like Dioxus's `dx`, which keeps the DX footprint lean and matches
Leptos's and Yew's approach. (Reproducible speed/memory/bundle-size numbers are owned by `REQ-PERF-*`; this
comparison is DX tooling only.)

---

## 7. Out-of-1.0 DX note

One DX-adjacent capability a reader might expect is **hot reload / HMR**, recorded in the scope document as
`REQ-FW-017` at `stage: post_one_oh`, `release_gating: false`. It is not minted as a `REQ-DX-*` and is referenced
by no `BAR-FERRIC-*` (contract identity I5): HMR for Rust/WASM reaches into the build pipeline and the bundler
and is far harder than in JS, and the standard edit-rebuild-refresh loop — documented under `REQ-DX-007` — is
workable for 1.0. It is named here as a deliberate deferral so the gap is recorded, not silently dropped.

---

## Metadata

- **Owning task:** `SUBTASK_docs_and_dx` (owns the `REQ-DX-*` register; identity I1).
- **Repository target:** `design/ferric/docs-and-dx.md` (placed by `SUBTASK_design_pr`).
- **Conforms to:** `../contracts/id_and_traceability_schema.yaml` (REQ fields, stage vocabulary,
  `verification_method`, `release_gating`) and `../contracts/_MANIFEST.yaml` (ownership, identities I1/I5).
- **Grounding commit:** `Nousix-LLC/nswe-demo` @ `276138f` (`main`).
- **Register summary:** `REQ-DX-001`–`REQ-DX-010`, all `release_gating: true`; staged `usable_early`
  (004 client examples, 007 client build, 008) or `one_oh` (001, 002, 003, 005, 006, 009, 010, and the SSR
  parts of 004/007). No `REQ-DX-*` is `post_one_oh`; the DX deferral is `REQ-FW-017` (§7), owned by scope.
- **Cross-references** to `REQ-FW-*`, `REQ-API-*`, `REQ-SSR-*`, `REQ-RT-*`, `REQ-HTTP-*`, `REQ-PERF-*`,
  `REQ-REL-*` are read-only; those registers are owned by their sibling spokes.
- **Peer sources (public tooling, cited by name):** the Leptos Book and `leptos-rs/start-trunk` /
  `cargo leptos new`; Dioxus `dx new` templates; Yew's Trunk-based getting-started; `cargo-generate`; docs.rs.
