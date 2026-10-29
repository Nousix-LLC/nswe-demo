# Ferric — Framework Scope & 1.0 Requirements

This document is the foundation of the Ferric 1.0 design: what the framework must be, for whom, and why, and
what "a real 1.0" looks like from the side of the person building an app with it. Every other design area —
SSR + hydration, the authoring model, async/forms/events, router + HTTP, docs & DX, release & distribution,
performance — traces back to the requirements registered here.

**What this document settles, and what it does not.** It states *framework-level* requirements — what Ferric
must let an app author do and be able to rely on — not *how* those capabilities are built. The hydration
algorithm, the macro expansion, the route matcher, the fetch wrapper, the benchmark harness: each is owned by
its sibling design area and carries its own register. Where a requirement's mechanism belongs to another area,
this document states the cross-cutting requirement from the app author's side and names that area; the area
carries its own `REQ-SSR-*` (SSR + hydration), `REQ-API-*` (authoring model), `REQ-ASYNC-*` (async/forms/
events), `REQ-RT-*` / `REQ-HTTP-*` (router + HTTP client), `REQ-DX-*` (docs & DX), `REQ-REL-*` (release &
distribution), or `REQ-PERF-*` (performance). This document is the sole owner of the `REQ-FW-*` register; all
cross-references to other areas' ids are read-only.

**What, not how.** A requirement here reads as something a stranger could check against the framework: an app
an author can build, a surface they can call, a guarantee they can rely on — never a chosen data structure or
code path. The implementation of each capability is deferred to later code DAGs, one epic at a time.

**Grounding.** Observations of "Ferric as it exists today" are grounded in the repository `Nousix-LLC/nswe-demo`
at the baseline commit `276138f` on `main`. They are stated so the 1.0 line is drawn against what is real, not
hypothetical. Peer-framework comparisons name the frameworks (Leptos, Sycamore, Dioxus, Yew among Rust peers;
Solid, Svelte, Vue 3 as design analogs) and rest on their documented, public behavior.

The six sections below are the required sections of this artifact: framework definition & kind; users & what
they build; author journeys; requirements & acceptance criteria (`REQ-FW-*`); the MVP-vs-1.0 line; and the
out-of-1.0 list.

---

## 1. Framework definition & kind

*(section: `framework_definition_and_kind`)*

### What Ferric is

Ferric is a **client-side-first UI framework for building web applications in Rust, compiled to WebAssembly**.
An app author writes components as plain Rust that read reactive **signals** and return a view tree; Ferric
renders that tree to real DOM and, when a signal a component read changes, re-renders by applying a minimal
**keyed diff** to the live DOM rather than rebuilding the subtree. It is the foundation the rest of a Rust/WASM
single-page application is assembled on: reactivity, a component/view model, a router, and an HTTP/JSON client
for talking to a backend.

Ferric is delivered as a **small set of library crates published to crates.io** — the `ferric` foundation plus
the `ferric-router` and `ferric-http` companions — that an author adds as ordinary `cargo` dependencies. An
author needs nothing but a Rust toolchain, the `wasm32-unknown-unknown` target, and the standard
`wasm-bindgen` tooling; there is no bespoke runtime to install and no server framework Ferric forces on them.

### What exists today (grounded at `276138f`)

The definition above is grounded in Ferric as it stands at `276138f`. Described from an app author's side,
today's Ferric lets someone:

- **Hold reactive state** with `create_signal` / `create_memo` / `create_effect`, coalesce updates with
  `batch`, and rely on scoped cleanup so a dropped reactive scope unsubscribes everything it owned (no
  stale-subscriber leaks). Signals and memos are cheap `Copy` handles into a thread-local runtime.
- **Write components** as any `Fn() -> VNode`, building a view tree out of `VElement` (which carries a `key`
  so sibling lists reconcile by identity) and text nodes, and attach `click` and `input` handlers.
- **Mount an app** with `dom::mount`, which wraps the render in an effect so the first render builds the DOM
  and every later render applies a keyed patch to the live DOM.
- **Route** and **fetch**: `crates/ferric-router` and `crates/ferric-http` already exist as walking-skeleton
  composition roots with **frozen public API surfaces** (their `prelude` modules) — route matching with
  path parameters, History-API navigation and a reactive current path, an in-app `link`; a `Client` /
  `RequestBuilder` / `Response` / `Error` Fetch-based JSON client. Many leaf bodies are still `todo!()`
  placeholders to be implemented by their owning work-items; the public shape is fixed.

Three facts about today's Ferric matter to the definition because an app author feels them directly:

1. **There is no server-side rendering and no hydration.** Ferric renders entirely in the browser after the
   WASM bundle loads. A first paint waits on download-and-execute, and there is no way to serve a
   server-rendered page that then becomes interactive. For a framework that means to be a credible 1.0 for
   real apps, this is the single defining gap — not an implementation detail. *(The mechanism is owned by
   SSR + hydration, `REQ-SSR-*`; stated here as the cross-cutting must-have.)*

2. **The authoring surface is hand-built view trees, not a template.** An author composes `VElement::new("div")
   .child(...)` by hand. There is no `view!`-style macro, no props/children/slots convention, and no context
   mechanism for passing shared values down a tree without threading them through every call. *(Mechanism owned
   by the authoring model, `REQ-API-*`.)*

3. **The composite "build a whole app" surface is partial.** Async data loading with suspense states, forms
   with two-way binding, the full span of DOM events, and the still-`todo!()` router/HTTP bodies are not yet a
   capability an author can lean on end to end. *(Owned by `REQ-ASYNC-*`, `REQ-RT-*`, `REQ-HTTP-*`.)*

### Which framework kind(s) apply, and what each makes app authors assume

Ferric is a **developer-facing library / UI framework**, distributed as open crates. That kind sets a baseline
of what an app author assumes without being told, and it is different from the expectations of an end-user
product (chirp, Ferric's first real consumer, is such a product and lives in `design/`; its end-user concerns —
sign-up screens, accounts, a product threat model — are **not** Ferric's).

**Primary kind — a UI framework an author builds real apps on.** What its users assume:

- *I can add it like any crate.* `cargo add ferric` resolves published, semver-versioned crates from crates.io;
  I am not vendoring a git revision to get a working build.
- *I can build a real application, not a toy.* Components, reactive state, keyed lists, events, client routing,
  data fetching, and forms are all present and compose — enough to build a multi-view app that talks to a
  backend.
- *It renders on the server and comes alive in the browser.* A framework presented as production-credible in
  2026 is expected to server-render a page and hydrate it to interactivity without a client-side re-render.
- *It is documented well enough to learn without reading the source,* and *its public API is stable* across a
  release line, with breaking changes signalled by semver.
- *Its cost and trade-offs are stated* — bundle size, speed, and where it sits relative to named peers — so I
  can check them rather than take performance on faith.

**A self-hostable, source-available project surface.** The crates are public source with a reproducible build.
Whoever consumes Ferric expects to build it from the documented steps without hidden prerequisites and to read
how it works. This is a real surface and the 1.0 keeps it working; it does not make Ferric a governed
open-source *community* project (see the kinds that do not apply).

**Kinds that do NOT apply to Ferric (recorded with reasons):**

- **An end-user product — not applicable.** Ferric has no end users of its own, no accounts, no deployment, no
  product threat model. Its "users" are app authors. The releasable-product concerns that apply to chirp do not
  apply to a library, and none should be assumed here. *Watch-point:* those concerns apply to the apps built
  *with* Ferric, which is why SSR, routing, forms, and async must be first-class — so an author can meet them.
- **A multi-tenant service / SaaS — not applicable.** Ferric is not run as a service; it ships as crates. There
  is no tenancy, no runtime an operator stands up.
- **A governed open-source community project — not applicable as such.** The repository is a closed demonstration
  within one engagement; it has the outward shape of public source and a reproducible build but no contributor
  workflow, governance, or community-support commitment. The 1.0 bar designs for the *library* and *source-
  available* expectations above, not for a contributor community that does not exist.

---

## 2. Users & what they build

*(section: `users_and_what_they_build`)*

Ferric's users are **Rust developers building for the browser**. They divide by what they are trying to do and
how much of the framework they lean on — not by seniority. Each persona is given by its goal and context.

### A1 — The evaluating author ("is this worth adopting?")

**Who:** A Rust developer choosing a frontend framework for a new project, comparing Ferric against Leptos,
Sycamore, Dioxus, and Yew, and against reaching for a JavaScript framework instead.

**Goal:** Decide, in an afternoon, whether Ferric can build the app they have in mind — and whether it will
still be a reasonable choice in a year.

**Context:** They judge the framework by its README, its guide, one runnable example, and a glance at bundle
size and benchmark numbers. A framework that cannot server-render, has thin docs, or hides its performance is
ruled out at this stage regardless of how elegant its internals are.

**What Ferric owes them:** a truthful one-page account of what it is and what it is for, a working "hello app"
they can run in minutes, a named comparison to its peers, and visible performance characteristics.

### A2 — The application author ("I'm building a real app on it")

**Who:** The core user. A developer building a multi-view, data-backed single-page application — the
`chirp-frontend` crate in this repository is the first real one.

**Goal:** Build the whole application — reactive UI, lists, forms, navigation between views, data fetched from a
REST backend, loading and error states — in idiomatic Rust, and ship it to users with a fast first paint.

**Context:** This author lives in the authoring model and the composite surface. They notice every missing
piece instantly: no template macro means verbose hand-built trees; no suspense means ad-hoc loading flags; no
SSR means a blank screen until the bundle loads. They expect the pieces to compose — router, HTTP client, and
view framework working together, not three libraries with mismatched assumptions.

**What Ferric owes them:** a complete, composable app-building surface (authoring, async, forms, events,
routing, HTTP), SSR + hydration for a fast first paint, and docs that show how the pieces fit.

### A3 — The library/component author ("I'm building on top of Ferric")

**Who:** A developer writing reusable components or a small library that others use with Ferric.

**Goal:** Expose components with a clean props/children/slots surface and shared state via context, against a
public API that will not shift under them between patch releases.

**Context:** They depend on the stability and expressiveness of the authoring model and on a semver contract
they can trust. A breaking change in a minor release, or an under-specified composition model, costs them more
than it costs an app author.

**What Ferric owes them:** an expressive, documented component/composition model (`REQ-API-*`) and a stable,
semver-governed public API across the workspace (`REQ-REL-*`).

### A4 — The operator of the build ("I package and ship the WASM")

**Who:** Whoever owns the build-and-deploy path for an app built on Ferric — often the same person as A2, acting
through a different surface.

**Goal:** Produce an optimized WASM bundle and the server integration for SSR, reproducibly, from documented
steps, and know the bundle's size.

**Context:** They work with the standard toolchain (`wasm32-unknown-unknown`, `wasm-bindgen`, a bundler such as
Trunk). They need Ferric's SSR story to fit a real server (the repository uses Axum) and the build to be
documented, not folklore.

**What Ferric owes them:** a documented, reproducible build on stable Rust and the standard WASM toolchain, a
server-integration path for SSR, and a visible bundle-size characterization (`REQ-DX-*`, `REQ-SSR-*`,
`REQ-PERF-*`).

### The kinds of app a 1.0 must let authors build

A credible 1.0 is defined by the apps it can carry, not by features in isolation. At 1.0 an author must be able
to build:

- a **reactive single-page application** with multiple views and client-side navigation;
- a **data-backed app** that fetches from and posts to a JSON/REST backend, with loading and error states;
- a **form-driven app** with two-way-bound inputs and the full range of DOM events;
- a **server-rendered app** whose first paint is server HTML and which becomes interactive by hydration with no
  client-side re-render;
- all of the above as a **real, shippable app** (`chirp-frontend` is the proof), built and bundled from
  documented steps.

### Where Ferric sits among its peers (named comparison)

Ferric's distinctive position is **fine-grained signal reactivity paired with a keyed virtual DOM** and a lean
dependency set (`wasm-bindgen`, `web-sys`, `js-sys`). Among the frameworks an evaluating author (A1) weighs:

| Framework | Reactivity model | SSR + hydration | Closest to Ferric on |
|---|---|---|---|
| **Leptos** (Rust) | Fine-grained signals, no VDOM | Yes — mature (SSR, hydration, islands, server fns) | Signal reactivity; Leptos is the SSR bar to meet |
| **Sycamore** (Rust) | Fine-grained signals, no VDOM | Yes — SSR + hydration | Signal reactivity; smaller surface |
| **Dioxus** (Rust) | React-like VDOM, RSX macro | Yes — fullstack | The VDOM + diff; broader (desktop/mobile) than Ferric targets |
| **Yew** (Rust) | React-like VDOM, `html!` macro | Partial — SSR exists, hydration less emphasized | The VDOM + macro authoring |
| **Solid** (JS, analog) | Fine-grained signals, compiled JSX | Yes | The signal model Ferric's `signal` echoes |
| **Svelte** (JS, analog) | Compiler reactivity | Yes (SvelteKit) | Design analog for ergonomics, not architecture |
| **Vue 3** (JS, analog) | Fine-grained reactivity + VDOM | Yes | The **closest architectural analog**: signals *and* a VDOM |

The assessment: among Rust peers, Ferric sits between the signal-only frameworks (Leptos, Sycamore) and the
React-style VDOM frameworks (Dioxus, Yew) by combining both, and its closest design analog is Vue 3. The two
capabilities its Rust peers have that today's Ferric lacks — **SSR + hydration** and a **declarative authoring
macro** — are exactly what the 1.0 bar must close, because an evaluating author will reject a framework that
lacks them. (Detailed, reproducible comparison numbers are owned by `REQ-PERF-*`; this is the qualitative
positioning the requirements answer to.)

---

## 3. Author journeys

*(section: `author_journeys`)*

Each journey is the path a Ferric user walks end to end — including first contact, building a real app, the
server-render path, and shipping and upgrading. Observations of today's framework are grounded at `276138f`.
Where the detailed requirement belongs to a sibling area, the author-side expectation is recorded here and the
mechanism is left to that area.

### JA1 — First contact and getting started (A1 → A2)

1. **Evaluating.** The author reads the README and a guide, sees what Ferric is and how it compares to named
   peers, and finds one runnable example.
2. **Scaffolding.** They start a new project from a documented path (ideally a `cargo generate` starter) and get
   a running app without assembling the toolchain by trial and error.
   - *Current gap:* there is a `counter` example but no documented zero-to-app starter. *(Owned by `REQ-DX-*`.)*
3. **First success.** The app builds to WASM and runs in the browser from the documented commands.

### JA2 — Building a real application (A2)

The loop the core author lives in, each step with the states it must handle:

- **Author components and state.** Write components with a declarative view surface, props/children/slots, and
  context for shared values; hold state in signals/memos; render keyed lists that reconcile by identity.
  - *Current gap:* no template macro, no props/slots/context convention — view trees are hand-built. *(Owned by
    `REQ-API-*`.)*
- **Fetch and show data.** Load data asynchronously with explicit loading, loaded, empty, and error states
  (suspense), and post changes back. *(Owned by `REQ-ASYNC-*`, with the client in `REQ-HTTP-*`.)*
- **Handle input.** Bind form inputs two-way and respond to the full range of DOM events, not only `click` and
  `input`. *(Owned by `REQ-ASYNC-*`.)*
- **Navigate.** Move between views with client-side routing: route parameters, nested routes, and in-app links,
  with the view re-rendering on path change. *(Owned by `REQ-RT-*`.)*
- **Compose the pieces.** Router, HTTP client, and view framework work together in one app without conflicting
  assumptions, over the frozen `ferric-router` / `ferric-http` surfaces extended, never silently redefined.

### JA3 — Server-side rendering and hydration (A2, A4)

1. **Render on the server.** The app renders to HTML on the server (the repository's server is Axum) so the
   first paint is real content, not a blank page waiting on the WASM bundle.
2. **Hydrate.** The shipped WASM attaches to the server-rendered DOM and makes it interactive **without
   re-rendering it** — event handlers wire up and reactive state picks up from the server-provided state.
3. **Carry state across the seam.** State the server used to render is carried to the client so hydration does
   not refetch or diverge.
   - *Current gap:* none of this exists today; it is the defining 1.0 must-have. *(Owned by `REQ-SSR-*`; stated
     cross-cuttingly as `REQ-FW-004`.)*

### JA4 — Shipping to production (A4)

1. **Build and optimize.** Produce an optimized WASM bundle from documented steps on stable Rust and the
   standard toolchain, and know its size. *(Build/DX owned by `REQ-DX-*`; size by `REQ-PERF-*`.)*
2. **Integrate the server.** Wire the SSR render into a real server so the deployed app serves server HTML and
   hydrates. *(Owned by `REQ-SSR-*`.)*
3. **Verify performance.** Consult reproducible speed/memory/bundle-size numbers against named peers rather than
   trusting a claim. *(Owned by `REQ-PERF-*`.)*

### JA5 — Depending on, and upgrading, Ferric (A2, A3)

1. **Add as a dependency.** Add the published crates from crates.io at a semver version. *(Owned by `REQ-REL-*`;
   stated cross-cuttingly as `REQ-FW-001`.)*
2. **Rely on stability.** Build against a public API that does not shift under a patch or minor release, with a
   documented compatibility policy across the multi-crate workspace. *(Owned by `REQ-REL-*`.)*
3. **Upgrade safely.** Read a changelog, understand what a version bump changes, and upgrade without silent
   breakage. *(Owned by `REQ-REL-*`.)*

### Journey coverage summary

| Journey | What it surfaces |
|---|---|
| JA1 First contact & getting started | truthful positioning; named comparison; a zero-to-app starter; a runnable first success |
| JA2 Building a real app | declarative authoring; props/slots/context; async + suspense; forms + full events; routing; composition over frozen surfaces |
| JA3 SSR & hydration | server render; hydrate without re-render; state across the seam — the one must-have |
| JA4 Shipping to production | reproducible optimized build; server integration; visible performance |
| JA5 Depend & upgrade | crates.io install; stable semver API; changelog & safe upgrade |

Each journey traces to at least one `REQ-FW-*` (see the coverage table in §4).

---

## 4. Requirements & acceptance criteria

*(section: `requirements_and_acceptance_criteria`)*

The register below turns every need surfaced by the definition (§1), the users (§2), and the journeys (§3) into
a testable, cross-cutting requirement. Each states what must be true from an app author's side; its acceptance
criteria are written so a person who never met the author can exercise the framework and reach the same yes/no.
Each carries a `stage` (`usable_early` | `one_oh`; see §5) and a `release_gating` flag: **true** where the 1.0
release depends on it, **false** only for the recorded post-1.0 deferrals in §6.

**Boundary.** These are *cross-cutting, framework-level* requirements. Where the detailed requirement and its
mechanism are owned by a sibling area, the requirement names that area; the area carries its own `REQ-SSR-*`,
`REQ-API-*`, `REQ-ASYNC-*`, `REQ-RT-*`, `REQ-HTTP-*`, `REQ-DX-*`, `REQ-REL-*`, or `REQ-PERF-*`. No "how" is
settled here, and no other area's id is minted here. Conformance to `../contracts/id_and_traceability_schema.yaml`.

### A. Adoption and toolchain

#### REQ-FW-001 — Installable as ordinary crates.io dependencies
- **Statement:** An author can add Ferric to a Rust project as normal, semver-versioned `cargo` dependencies from crates.io, without vendoring a git revision.
- **Rationale:** Every Rust peer an evaluating author weighs (Leptos, Sycamore, Dioxus, Yew) is on crates.io; a framework only obtainable as a git revision is not treated as a dependable choice.
- **Acceptance criteria:** From a fresh project, `cargo add ferric` (and `ferric-router` / `ferric-http` where used) resolves published crates at a semver version and the project builds; no git or path dependency on Ferric is required to get a working build. (Today the crates are workspace-internal; this requirement closes the publishing gap.)
- **Stage:** one_oh · **Release gating:** true
- **Traceability:** JA5; A1, A2, A3. Publishing, semver, workspace versioning → release & distribution (`REQ-REL-*`).

#### REQ-FW-002 — A documented path from zero to a running app
- **Statement:** A new author can go from nothing to a running Ferric app using only the repository's documentation, without reverse-engineering the toolchain.
- **Rationale:** Adoption is decided in the first afternoon (persona A1); a framework whose getting-started path is folklore loses evaluators before they reach its strengths. Peers ship starters (`cargo generate` templates, scaffolds) for this reason.
- **Acceptance criteria:** Following the documented getting-started path (ideally a `cargo generate` / scaffold starter), an author who has never used Ferric produces a project that builds to WASM and runs in a browser, using only documented commands and the standard toolchain. (Today a `counter` example exists but no zero-to-app starter.)
- **Stage:** one_oh · **Release gating:** true
- **Traceability:** JA1. Starter, tutorial, build docs → docs & DX (`REQ-DX-*`).

#### REQ-FW-003 — Builds on stable Rust and the standard WASM toolchain
- **Statement:** Ferric builds and runs on stable Rust with the standard `wasm32-unknown-unknown` + `wasm-bindgen` toolchain, with no nightly-only requirement.
- **Rationale:** A nightly-only framework narrows its audience and breaks unpredictably on compiler updates; a credible 1.0 targets stable and the toolchain authors already have (`wasm-bindgen` is the baseline's only binding dependency).
- **Acceptance criteria:** A documented build of a Ferric app succeeds on current stable Rust targeting `wasm32-unknown-unknown` through the standard tooling; nothing in the required path depends on a nightly feature. Compatible-version ranges are stated.
- **Stage:** usable_early · **Release gating:** true
- **Traceability:** JA1, JA4; A4. Toolchain/version specifics → docs & DX (`REQ-DX-*`), release (`REQ-REL-*`).

### B. The composite app-building surface

#### REQ-FW-004 — Server-side rendering with hydration (the 1.0 must-have)
- **Statement:** Ferric can render an app to HTML on the server and hydrate that markup into a fully interactive app in the browser without a client-side re-render.
- **Rationale:** Every Rust peer that reads as production-credible (Leptos, Sycamore, Dioxus) ships SSR + hydration, and client-only rendering means a blank first paint until the WASM downloads and runs. It is the one capability today's Ferric lacks that an evaluating author will reject it for — the defining 1.0 must-have.
- **Acceptance criteria:** A stated example app renders its initial HTML on the server; the browser receives real content as the first paint; the shipped WASM attaches to the server-rendered DOM, wiring event handlers and reactive state, such that the DOM is **not** re-rendered on load (observably: no full re-paint/replacement of the hydrated subtree) and interaction works immediately; state used for the server render is carried to the client without a refetch. This is the single defining 1.0 capability.
- **Stage:** one_oh · **Release gating:** true
- **Traceability:** JA3; A2, A4; the SSR gap at §1. Mechanism (isomorphic render, hydration, state transfer, server integration) → SSR + hydration (`REQ-SSR-*`).

#### REQ-FW-005 — A declarative authoring model
- **Statement:** An author can describe UI declaratively — a `view!`-style macro or equivalent, components with props, children, and slots, and a context mechanism for shared values — rather than hand-building view trees.
- **Rationale:** A declarative template is the baseline authoring ergonomic of every peer (Leptos/Sycamore `view!`, Yew `html!`, Dioxus RSX, Solid/Vue JSX/templates); hand-built `VElement` trees are too verbose to build a real app comfortably, and props/slots/context are what make components composable and reusable.
- **Acceptance criteria:** An author can write a component using the declarative authoring surface, pass it props and children (including slot-style composition), and read a value provided by an ancestor via context without threading it through every call; the stated examples compile and render the described UI. (Today authoring is hand-built `VElement` trees with no macro, props convention, or context.)
- **Stage:** usable_early · **Release gating:** true
- **Traceability:** JA2; A2, A3. Macro, props/children/slots, context, component model → authoring model (`REQ-API-*`).

#### REQ-FW-006 — Async data, forms, and full event coverage
- **Statement:** An author can load data asynchronously with explicit loading/error (suspense) states, bind form inputs two-way, and handle the full range of DOM events — not only `click` and `input`.
- **Rationale:** Real apps are data-backed and form-driven; without framework support for async states, two-way binding, and the full event set, authors hand-roll loading flags and event plumbing that the framework should own. Today's runtime binds only `click` and `input`, which cannot carry a real form.
- **Acceptance criteria:** A stated example fetches data and shows distinct loading, loaded, empty, and error states; a form binds an input two-way so the model and the field stay in sync; event handlers beyond click/input (for example submit, keyboard, focus/blur, change) can be attached and fire. (Today only `click` and `input` are bound and there is no suspense or two-way binding.)
- **Stage:** usable_early · **Release gating:** true
- **Traceability:** JA2; A2. Resources/suspense, forms/two-way binding, event coverage → async/forms/events (`REQ-ASYNC-*`).

#### REQ-FW-007 — Client-side routing and an HTTP/JSON client
- **Statement:** An author can navigate between views with client-side routing (route parameters, nested routes, in-app links) and talk to a JSON/REST backend through an HTTP client, both composing with the view framework.
- **Rationale:** A single-page app is defined by in-app navigation and backend data; routing and an HTTP client are table stakes for any real app. The `ferric-router` and `ferric-http` crates already fix the public surfaces (board issues #5/#6), so the 1.0 completes and composes them rather than inventing a parallel story.
- **Acceptance criteria:** A stated multi-route example navigates between views via in-app links and route parameters with the view re-rendering on path change; a stated example performs a typed JSON request/response against a backend and handles a non-2xx status as an error; the router and client operate over the existing `ferric-router` / `ferric-http` public surfaces. (Today both crates exist as frozen skeletons with `todo!()` bodies.)
- **Stage:** usable_early · **Release gating:** true
- **Traceability:** JA2; A2. Router → `REQ-RT-*`; HTTP/JSON client → `REQ-HTTP-*` (both own the detail; both absorb board issues #5/#6).

#### REQ-FW-008 — An author can build a complete, real single-page application
- **Statement:** The capabilities above compose into a complete real app — multiple views, reactive state, keyed lists, events, forms, routing, and backend data — demonstrated by a real reference application, not a toy.
- **Rationale:** A pile of features that each demo in isolation can still fail to add up to a buildable app; the only proof a framework carries real apps is a real app built on it. `chirp-frontend` is Ferric's first real consumer and the dogfooding check that the pieces actually compose.
- **Acceptance criteria:** A non-toy reference application (the repository's `chirp-frontend`) is built on Ferric and exercises reactivity, keyed lists, events, forms, client routing, and backend data end to end, and builds and runs from documented steps. "The examples compile" is not sufficient; the reference app is the proof.
- **Stage:** one_oh · **Release gating:** true
- **Traceability:** JA2, JA4; A2. Composite of `REQ-API-*`, `REQ-ASYNC-*`, `REQ-RT-*`, `REQ-HTTP-*`, `REQ-SSR-*`.

### C. Framework qualities an author relies on

#### REQ-FW-009 — Frozen public surfaces carried forward additively
- **Statement:** The existing frozen public API surfaces of the foundation and the `ferric-router` / `ferric-http` companions are carried to 1.0 additively — extended, never silently redefined or broken.
- **Rationale:** The baseline already froze the `prelude` surfaces (`ferric`, `ferric-router`, `ferric-http`) and apps are written against them; silently redefining them during the 1.0 work would break consumers and the reference app. Additive-only evolution is what keeps the walking skeleton a dependable base.
- **Acceptance criteria:** The `prelude` surfaces present at the baseline remain usable at 1.0; any change to them is additive (new items), and no previously exported item is removed or redefined without a semver-signalled break documented in the changelog. An app written against the baseline preludes still compiles at 1.0.
- **Stage:** usable_early · **Release gating:** true
- **Traceability:** §1 (frozen preludes); JA2, JA5. Additive-over-frozen-surface detail → router + HTTP (`REQ-RT-*`, `REQ-HTTP-*`); compatibility policy → release (`REQ-REL-*`).

#### REQ-FW-010 — A stable, semver-governed public API with a compatibility policy
- **Statement:** Ferric exposes a defined public API across its crates, governed by semver, with a documented compatibility policy so authors and library authors can depend on it across releases.
- **Rationale:** A "1.0" is a promise of stability; library authors (A3) building on Ferric need to know what a version bump can change across a multi-crate workspace where the crates must version coherently. Without a stated policy, "1.0" is a number with no commitment behind it.
- **Acceptance criteria:** The public API of each crate is identified; a written compatibility policy states what semver level signals a breaking change across the multi-crate workspace; a documented version-coordination rule covers how the crates version together. An author can read, from the repository, what a given version bump is allowed to change.
- **Stage:** one_oh · **Release gating:** true
- **Traceability:** JA5; A3. Semver, changelog, versioning/compat policy → release & distribution (`REQ-REL-*`).

#### REQ-FW-011 — Documentation sufficient to learn and build without the source
- **Statement:** An author can learn Ferric and build a real app from its documentation — a guide, a tutorial, API docs, and runnable examples — without reading the framework's source.
- **Rationale:** Docs are how a framework is adopted and how an author succeeds without the maintainer's help; a 1.0 whose only true documentation is its source cannot be evaluated (A1) or used at scale (A2). Every credible peer ships a guide, API docs, and examples.
- **Acceptance criteria:** The repository provides a conceptual guide, a start-to-finish tutorial, rendered API documentation for the public surface, and runnable examples; a reviewer can confirm each exists and that a stated task (build the tutorial app) can be completed from the docs alone.
- **Stage:** one_oh · **Release gating:** true
- **Traceability:** JA1, JA2, JA4; A1, A2. Guide/tutorial/API-docs/examples/starter → docs & DX (`REQ-DX-*`).

#### REQ-FW-012 — Reproducible, named-peer performance characterization
- **Statement:** Ferric's speed, memory, and bundle size are characterized reproducibly and compared to named peer frameworks, so an author can judge its cost rather than take it on faith.
- **Rationale:** Performance and bundle size are first-order adoption criteria for WASM frameworks (a WASM bundle is downloaded before the app runs), and the standard way the ecosystem states them is js-framework-benchmark. An unmeasured framework asks authors to trust a claim; a 1.0 should let them check it.
- **Acceptance criteria:** A reproducible benchmark entry (a js-framework-benchmark implementation) runs and produces stated speed, memory, and bundle-size numbers; the results are presented against named peers; a reader can re-run the benchmark from documented steps and reach comparable numbers.
- **Stage:** one_oh · **Release gating:** true
- **Traceability:** JA1, JA4; A1, A4. Benchmark harness, methodology, comparison → performance (`REQ-PERF-*`).

#### REQ-FW-013 — A lean dependency footprint
- **Statement:** The 1.0 preserves Ferric's lean dependency footprint — the core builds on `wasm-bindgen`, `web-sys`, and `js-sys` without pulling in a heavy runtime — so the bundle-size cost stays defensible.
- **Rationale:** In WASM, every dependency is downloaded bytes; leanness is the baseline's stated design choice and a real differentiator against heavier stacks. Letting the dependency set grow unchecked during the 1.0 work would quietly erode the performance story the framework competes on.
- **Acceptance criteria:** The core framework's required dependency set remains limited to the WASM-binding essentials plus narrowly-scoped additions justified per crate (the HTTP client's `serde`/`wasm-bindgen-futures` being the expected example); no heavy general-purpose runtime is introduced; the dependency set is documented and its bundle-size impact is reflected in the performance numbers.
- **Stage:** one_oh · **Release gating:** true
- **Traceability:** §1; A1, A4. Bundle-size measurement → performance (`REQ-PERF-*`); per-crate dependency choices → each owning area.

#### REQ-FW-014 — Actionable failure diagnostics
- **Statement:** When an author makes a mistake, Ferric's failures are actionable — macro/compile-time diagnostics point at the author's code, and runtime panics carry enough context to locate the cause.
- **Rationale:** Much of the day-to-day experience of a framework is what happens when you get it wrong; opaque macro errors and context-free panics are a common reason authors abandon a Rust frontend framework. Diagnostics that name the author's mistake are part of what makes the authoring model usable, not a nicety.
- **Acceptance criteria:** A stated set of common authoring mistakes (for example a malformed view, a misused prop) produces a compile-time or macro diagnostic that names the problem in terms of the author's code rather than framework internals; a representative runtime failure carries a message that locates the cause. A reviewer can reproduce the stated cases and confirm the diagnostic quality.
- **Stage:** usable_early · **Release gating:** true
- **Traceability:** JA2; A2, A3. Macro diagnostics → authoring model (`REQ-API-*`); DX of errors → docs & DX (`REQ-DX-*`).

### Journey → requirement coverage

Every journey in §3 traces to at least one `REQ-FW-*`.

| Journey | Covering REQ-FW-* |
|---|---|
| JA1 First contact & getting started | 002, 003, 011, 012 |
| JA2 Building a real app | 005, 006, 007, 008, 009, 014 |
| JA3 SSR & hydration | 004, 008 |
| JA4 Shipping to production | 003, 008, 011, 012, 013 |
| JA5 Depend & upgrade | 001, 009, 010 |

### Framework-kind implication coverage

Each assumption §1 named for the framework kind is reflected in a requirement or recorded out-of-scope in §6.

| Kind | Assumption | Where it lands |
|---|---|---|
| UI framework | I can add it like any crate | REQ-FW-001 |
| UI framework | I can build a real app, not a toy | REQ-FW-008 (over 005/006/007) |
| UI framework | It server-renders and hydrates | REQ-FW-004 |
| UI framework | Documented enough to learn without the source | REQ-FW-011 |
| UI framework | Stable public API across a release line | REQ-FW-010 |
| UI framework | Cost and trade-offs stated, not taken on faith | REQ-FW-012, 013 |
| Source-available | Reproducible build from documented steps | REQ-FW-002, 003 |
| End-user product | (sign-up, accounts, product threat model) | Out of scope (§6), with reason — Ferric is a library |

---

## 5. The MVP-vs-1.0 line

*(section: `mvp_vs_one_oh_line`)*

This is the explicit, principled line between what makes Ferric **usable early** — the floor at which an author
can first build a real (non-toy) app with it — and what the **full 1.0** requires on top. No hidden cuts: every
requirement in §4 carries its stage, and the two stages below are the two the contract's schema defines
(`usable_early`, `one_oh`).

### 5.1 The two stages and the principle that separates them

- **`usable_early`** — the floor at which Ferric first becomes a framework a developer can actually build a real
  app with. **Not a toy and not the counter example.** An author at this floor can author components
  declaratively, hold reactive state, render keyed lists, handle forms and the full event range, load backend
  data with explicit loading/error states, and navigate between views with client routing — all on stable Rust
  with the standard toolchain, over the frozen foundation surfaces carried forward. The app renders in the
  browser (client-side). A framework that cannot do these is **below** the floor, not within it.
- **`one_oh`** — the full bar. Everything in `usable_early` **plus** what makes Ferric a framework a stranger
  would adopt for production in 2026: **SSR + hydration** (the defining must-have), a demonstrated real
  reference app, crates.io publishing with a stable semver API and compatibility policy, complete docs
  (guide/tutorial/API/examples/starter), and reproducible named-peer performance numbers with a lean,
  documented dependency footprint.

**The principle.** A requirement is `usable_early` when *its absence makes building a real app impossible or
means hand-rolling what a framework should provide* (no declarative authoring, no async/suspense, no forms, no
routing, no stable foundation surface). A requirement is `one_oh` when *an author can build a client-side app
without it, but no credible 1.0 of a framework of this kind ships without it* (no SSR, so first paint waits on
the bundle; not on crates.io, so it cannot be depended on normally; undocumented or unbenchmarked, so it cannot
be evaluated or trusted). SSR + hydration sits deliberately in `one_oh` as the gating must-have: an app is
usable early without it, but Ferric is not a 1.0 without it.

### 5.2 What is in `usable_early` (the early-usable floor)

- **Declarative authoring** — a view macro/equivalent, props/children/slots, context (`REQ-FW-005`).
- **Async, forms, and events** — resources with loading/error states, two-way form binding, full DOM event
  coverage (`REQ-FW-006`).
- **Routing and HTTP** — client routing with params/nested routes/links and a JSON/HTTP client (`REQ-FW-007`).
- **Stable-Rust/standard-toolchain build** (`REQ-FW-003`) over the **frozen foundation surfaces carried forward
  additively** (`REQ-FW-009`), with **actionable failure diagnostics** (`REQ-FW-014`).

### 5.3 What `one_oh` adds on top

- **SSR + hydration** — the gating must-have (`REQ-FW-004`).
- **A demonstrated real reference app** across the whole surface (`REQ-FW-008`).
- **crates.io publishing, a stable semver API, and a compatibility policy** (`REQ-FW-001`, `REQ-FW-010`).
- **Complete documentation and a zero-to-app starter** (`REQ-FW-002`, `REQ-FW-011`).
- **Reproducible named-peer performance numbers and a lean, documented footprint** (`REQ-FW-012`, `REQ-FW-013`).

The release bar (`BAR-FERRIC-*`, owned by `SUBTASK_release_bar`) consolidates these into checkable criteria and
marks SSR + hydration as the gating must-have; this section fixes the staging those criteria enforce.

---

## 6. Out-of-1.0 list

*(section: `out_of_one_oh`)*

These are capabilities a reader might expect of a UI framework that Ferric 1.0 **deliberately defers**. Each is
recorded as a post-1.0 item with a reason — not silently omitted — at `stage: post_one_oh` and
`release_gating: false`. Per contract identity I5, **no `BAR-FERRIC-*` criterion references any of these**: they
do not gate the release, and the release bar must leave them out.

#### REQ-FW-015 — Scoped CSS / styling story (deferred)
- **Statement (deferred):** A built-in scoped-CSS / component-styling system is **not** part of 1.0.
- **Reason:** Styling in a Rust/WASM app is served today by plain CSS, utility frameworks, or inline styles on
  the view tree; a framework-owned scoped-CSS system is a large design space (compile-time extraction, scoping
  semantics, theming) that is not required to build a real app and is not what makes Ferric credible against its
  peers. Deferring it keeps the 1.0 focused on rendering and reactivity. Several named peers also leave styling
  to the ecosystem.
- **Stage:** post_one_oh · **Release gating:** false · referenced by no `BAR-FERRIC-*`.

#### REQ-FW-016 — Devtools / component inspector (deferred)
- **Statement (deferred):** A browser-extension devtools / reactive-graph inspector is **not** part of 1.0.
- **Reason:** Devtools are a quality-of-life aid, not a prerequisite for building or shipping an app. They
  depend on a stable internal reactive model and are a substantial separate product; building one before the
  core surface and SSR are settled would be premature. Recorded for post-1.0.
- **Stage:** post_one_oh · **Release gating:** false · referenced by no `BAR-FERRIC-*`.

#### REQ-FW-017 — Hot reload / HMR (deferred)
- **Statement (deferred):** Hot-module-reload / hot reload of components in development is **not** part of 1.0.
- **Reason:** HMR for Rust/WASM is a hard, tooling-heavy problem (it reaches into the build pipeline and the
  bundler, and the Rust/WASM compile model makes it far harder than in JS); the standard edit-rebuild-refresh
  loop is workable for 1.0. It is a DX enhancement to pursue after the framework and its docs are solid, not a
  release blocker.
- **Stage:** post_one_oh · **Release gating:** false · referenced by no `BAR-FERRIC-*`.

#### REQ-FW-018 — Desktop / mobile / non-web render targets (deferred)
- **Statement (deferred):** Rendering to targets other than the browser DOM (desktop, mobile, native, TUI) is
  **not** part of 1.0.
- **Reason:** Ferric 1.0 is a **web** framework; its renderer binds the browser DOM through `web-sys`. Multi-
  target rendering (the space Dioxus occupies) is a different and much larger architecture — a renderer
  abstraction and platform backends — that would reshape the whole framework. A credible web-framework 1.0 does
  not require it; it is recorded as a possible future direction, out of this release.
- **Stage:** post_one_oh · **Release gating:** false · referenced by no `BAR-FERRIC-*`.

---

## Metadata

- **Owning task:** `SUBTASK_scope_and_requirements` (owns the `REQ-FW-*` register; identity I1).
- **Repository target:** `design/ferric/scope-and-requirements.md` (placed by `SUBTASK_design_pr`).
- **Conforms to:** `../contracts/id_and_traceability_schema.yaml` (REQ fields, stage vocabulary, `release_gating`),
  `../contracts/_MANIFEST.yaml` (ownership, identities I1/I5).
- **Grounding commit:** `Nousix-LLC/nswe-demo` @ `276138f` (`main`).
- **Register summary:** `REQ-FW-001`–`REQ-FW-014` are 1.0 requirements (`release_gating: true`), staged
  `usable_early` (003, 005, 006, 007, 009, 014) or `one_oh` (001, 002, 004, 008, 010, 011, 012, 013);
  `REQ-FW-015`–`REQ-FW-018` are recorded post-1.0 deferrals (`post_one_oh`, `release_gating: false`,
  referenced by no `BAR-FERRIC-*`).
- **Cross-references** to `REQ-SSR-*`, `REQ-API-*`, `REQ-ASYNC-*`, `REQ-RT-*`, `REQ-HTTP-*`, `REQ-DX-*`,
  `REQ-REL-*`, `REQ-PERF-*` are read-only; those registers are owned by their sibling spokes.
```
