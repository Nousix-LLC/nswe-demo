# Ferric 1.0 — Release Bar

This document states what "a real 1.0 framework" means for Ferric, in terms a stranger can check against the
framework without reading any author's notes. It owns the `BAR-FERRIC-*` register.

---

## 1. Introduction

A release bar is a list of conditions, each checkable against the shipped product. This one answers: what does
Ferric have to be before it can be announced as 1.0?

The bar draws on eight design areas — framework scope, SSR + hydration, authoring model, async/forms/events,
router + HTTP, docs & DX, release & distribution, and performance — each of which produced a `REQ-*` register.
Each `BAR-FERRIC-*` criterion here enforces one or more of those requirements.

**How to read this document.** Each criterion has:
- **id** — `BAR-FERRIC-NNN`, sequential and unique in this register
- **statement** — a condition a stranger can check against the framework; not an aspiration
- **references** — the `REQ-*` ids this criterion enforces (at least one per criterion)
- **verification_method** — from the closed vocabulary below
- **stage** — `usable_early` or `one_oh` (see §2)
- **must_have** — present only on the SSR + hydration gating criterion (§4, BAR-FERRIC-004)

**Verification method vocabulary:**
- `automated_test` — a test suite (`cargo test`, `wasm-bindgen-test`, `trybuild`) passes in CI
- `example_build` — a named example or the reference app (`chirp-frontend`) builds, runs, and demonstrates the
  stated capability
- `doc_inspection` — a reviewer reads the repository and confirms the stated artifact is present and accurate
- `benchmark_run` — a benchmark executes from documented steps and produces the stated results
- `publishability` — the stated crates publish cleanly to crates.io from documented steps

---

## 2. Staging principle

### Two stages, one principle

Every requirement in the design carries one of two stages:

**`usable_early`** — the floor at which Ferric first becomes a framework a developer can actually build a real
app with. At this floor: declarative authoring (view! macro, components, context), reactive state (signals,
memos, effects, batch), async data loading with suspense, forms and the full DOM event range, client routing,
and an HTTP/JSON client — all on stable Rust over the frozen foundation surfaces, with actionable diagnostics.
A framework that cannot do these is below the floor, not within it.

**`one_oh`** — everything in `usable_early`, plus what a framework announced as 1.0 in 2026 must have: SSR +
hydration (the gating must-have — an app is buildable without it, but not releasable as a credible 1.0 without
it), a demonstrated real reference application, crates.io publishing with a semver API and compatibility policy,
complete documentation (guide/tutorial/API docs/examples/starter), and reproducible named-peer performance
numbers with a lean, documented dependency footprint.

**The principle.** A requirement is `usable_early` when its absence makes building a real app impossible or
requires an author to hand-roll what the framework should own. A requirement is `one_oh` when an author can
ship a client-side app without it but no credible 1.0 of a framework of this kind goes without it.

### SSR + hydration as the gating must-have

SSR + hydration sits at `one_oh` deliberately. Every Rust/WASM UI framework peer that reads as production-
credible ships SSR + hydration; a client-only framework means a blank first paint until the WASM bundle
downloads and executes. It is the one capability today's Ferric lacks that an evaluating author will reject it
for. BAR-FERRIC-004 is the single `must_have: true` criterion in this register: Ferric does not ship as 1.0
without it.

---

## 3. BAR-FERRIC-* criteria

Criteria are organized by capability area and numbered sequentially. Stage appears at the end of each entry.

### Reactive core

---

**BAR-FERRIC-001 — Reactive state model**

**Statement:** The reactive runtime provides `create_signal`, `create_memo`, `create_effect`, and `batch`,
with scoped cleanup that unsubscribes every reactive subscription owned by a dropped scope without leaks;
`create_memo` does not recompute when its output value has not changed; `batch` defers effect execution until
the batch closes. An app written against the baseline `ferric` prelude compiles at 1.0 without modification to
any previously exported item.

**References:** REQ-FW-005, REQ-FW-009

**Verification method:** automated_test

**Stage:** usable_early

---

**BAR-FERRIC-002 — Virtual DOM and keyed DOM patching**

**Statement:** The framework renders component trees to real DOM and, when a signal changes, updates only the
DOM nodes affected — not the whole subtree. Keyed sibling lists reconcile by their `key` value, so a reorder
moves existing DOM nodes rather than destroying and recreating them. A test or example verifies that no
unaffected DOM nodes are replaced on a partial-state update.

**References:** REQ-FW-005

**Verification method:** automated_test

**Stage:** usable_early

---

### SSR + hydration

---

**BAR-FERRIC-003 — SSR — HTML rendering**

**Statement:** Ferric renders an app to an HTML string on the server (off WASM, on the server's native target)
using a renderable core that compiles with `--features ssr`; each concurrent request gets its own isolated
reactive scope; the produced HTML passes a standard HTML sanitizer without injected markup. A test produces
the HTML string for a representative component and checks its content without running a browser.

**References:** REQ-SSR-001, REQ-SSR-002, REQ-SSR-003, REQ-SSR-010, REQ-SSR-012

**Verification method:** automated_test

**Stage:** one_oh

---

**BAR-FERRIC-004 — Hydration — client picks up server HTML, no re-render**

**Statement:** The client WASM bundle attaches to the server-rendered DOM without replacing any server-rendered
element. After hydration: event handlers fire, reactive state responds to changes, and no DOM node that was
present in the server HTML is torn down and rebuilt on load. An SSR + hydrate example demonstrates the complete
path: server renders HTML, browser receives it as the first paint, WASM loads and wires up, interaction works
without a visible repaint.

**References:** REQ-SSR-004, REQ-SSR-005, REQ-SSR-008, REQ-FW-004, REQ-FW-008

**Verification method:** example_build

**Stage:** one_oh

**must_have:** true

---

**BAR-FERRIC-005 — State carry and async resource resume across the SSR seam**

**Statement:** State computed during the server render (including resolved async resources) is serialized into
the page and deserialized on the client without a refetch; the client resource starts in the loaded state with
the server's data, not the loading state. An SSR + hydrate example with at least one async resource verifies
that no network request fires for that resource during hydration.

**References:** REQ-SSR-006, REQ-SSR-007, REQ-SSR-012, REQ-ASYNC-012, REQ-ASYNC-013

**Verification method:** example_build

**Stage:** one_oh

---

**BAR-FERRIC-006 — Hydration mismatch detection and Axum integration**

**Statement:** When the client-rendered structure diverges from the server HTML, Ferric detects the mismatch
and recovers without a silent fault. The SSR integration works with Axum: the reference Axum server handler
renders Ferric to HTML and returns it in an HTTP response that the browser then hydrates. An Axum integration
example is documented and builds from the stated steps.

**References:** REQ-SSR-009, REQ-SSR-011

**Verification method:** example_build

**Stage:** one_oh

---

### Authoring model

---

**BAR-FERRIC-007 — view! macro — elements, dynamics, and component invocation**

**Statement:** The `view!` macro accepts HTML element names with their standard attributes and properties,
dynamic expressions (`{signal.get()}`), conditional rendering, keyed list rendering via `<For key=... each=...
view=...>`, and component invocation by name with props and children. The output compiles and renders the
described UI. A stated set of valid view! examples all compile without modification.

**References:** REQ-API-001, REQ-API-002, REQ-API-003, REQ-API-004, REQ-API-006, REQ-API-012, REQ-API-015,
REQ-API-023

**Verification method:** automated_test

**Stage:** usable_early

---

**BAR-FERRIC-008 — view! macro — compile-time diagnostics**

**Statement:** A stated set of malformed view! inputs — for example an unrecognized HTML element name, a
missing required attribute, a structurally invalid expression — each produces a compile-time diagnostic that
names the problem in terms of the author's code rather than macro internals. The diagnostic set is captured
by trybuild tests that pass in CI.

**References:** REQ-API-024, REQ-API-025, REQ-API-026, REQ-API-027, REQ-API-028

**Verification method:** automated_test

**Stage:** usable_early

---

**BAR-FERRIC-009 — Component model — props, children, named slots**

**Statement:** A component function accepts required and optional props (with defaults), a `children: VNode`
parameter, and named slots. A prop type can be converted from an accepted type to the stored type via a
declared `From` implementation (converting props). A slot can carry a typed closure for parameterized
rendering (`SlotFn<T>`). A component can be invoked dynamically through a `dyn Component` trait object.
Stated examples for each of these forms compile and render correctly.

**References:** REQ-API-030, REQ-API-031, REQ-API-032, REQ-API-033, REQ-API-034, REQ-API-035, REQ-API-037,
REQ-API-038, REQ-API-041, REQ-API-045, REQ-API-047, REQ-API-049

**Verification method:** automated_test

**Stage:** usable_early

---

**BAR-FERRIC-010 — Component model — type-level diagnostics**

**Statement:** A stated set of component authoring mistakes — for example a missing required prop, a type
mismatch in a prop value, a misused slot — each produces a compile-time error that names the problem at the
call site rather than inside framework internals. The diagnostic set is captured by trybuild tests that pass
in CI.

**References:** REQ-API-050, REQ-API-051, REQ-API-052

**Verification method:** automated_test

**Stage:** usable_early

---

**BAR-FERRIC-011 — Context / dependency injection**

**Statement:** A component can provide a value into the component tree with `provide_context`, and any
descendant can read it with `use_context` (returns `Option`) or `expect_context` (panics with a clear message
if absent). The value does not need to be threaded through every intermediate call. Stated examples for
provide/use/expect compile and pass.

**References:** REQ-API-060, REQ-API-061, REQ-API-063, REQ-API-066, REQ-API-068

**Verification method:** automated_test

**Stage:** usable_early

---

### Async, events, forms

---

**BAR-FERRIC-012 — Async resources and suspense**

**Statement:** `create_resource` tracks four states — loading, loaded (with data), empty (no data), and error
— and components reading it re-render when the state changes. A `<Suspense>` boundary shows the `fallback`
slot while any enclosed resource is loading and switches to the resolved content when all are loaded. The
boundary can be nested. A resource can be refetched on demand; a stale-while-revalidating pattern keeps
previous data visible during a refetch. An `<ErrorBoundary>` catches a resource error and renders the error
slot with the error value; the author can reset it to retry. A deferred resource starts loading only when
first read. Reactive scope cleanup cancels any in-flight resource load. Stated tests or examples cover each
of these states and transitions.

**References:** REQ-ASYNC-001, REQ-ASYNC-002, REQ-ASYNC-004, REQ-ASYNC-005, REQ-ASYNC-006, REQ-ASYNC-007,
REQ-ASYNC-008, REQ-ASYNC-009, REQ-ASYNC-010, REQ-ASYNC-011, REQ-ASYNC-015, REQ-ASYNC-016

**Verification method:** automated_test

**Stage:** usable_early

---

**BAR-FERRIC-013 — DOM event handling**

**Statement:** An author attaches handlers to DOM events using `on:<event_name>={handler}` syntax in view!,
where `handler` receives a typed payload (the concrete event type for named events). The full DOM event range
is covered: click, input, change, submit, keydown/keyup/keypress, focus, blur. An author can call
`event.prevent_default()` and `event.stop_propagation()`. For an event not in the named set, an author can
attach a handler by string name as an escape hatch. The async/forms/events examples compile and demonstrate
each category.

**References:** REQ-ASYNC-020, REQ-ASYNC-021, REQ-ASYNC-022, REQ-ASYNC-023, REQ-ASYNC-024, REQ-FW-006

**Verification method:** automated_test

**Stage:** usable_early

---

**BAR-FERRIC-014 — Forms and two-way binding**

**Statement:** An author binds a signal to a form input two-way: the input reflects the signal's current
value and a change to the input updates the signal. The binding covers text inputs, checkboxes, and `<select>`
elements. A submit handler on a `<form>` fires with the form data. Validation feedback can be wired to a
signal updated on submit or on change. Stated examples for each input kind compile and demonstrate the
two-way relationship.

**References:** REQ-ASYNC-040, REQ-ASYNC-041, REQ-ASYNC-042, REQ-ASYNC-043

**Verification method:** automated_test

**Stage:** usable_early

---

### Router + HTTP

---

**BAR-FERRIC-015 — Client-side routing**

**Statement:** The router matches path patterns with named parameters and renders the matching route's
component in a reactive `<Outlet>`. Nested routes nest their `<Outlet>` inside a parent route's view.
Each route level can declare a fallback for no-match. `navigate(path)` and the History API `push` / `replace`
functions change the current path reactively. An `<a>`-backed `<Link>` component is keyboard-focusable and
announced by assistive technology as a link. The `ferric-router` prelude surfaces present at the baseline
are present at 1.0; no previously exported item is removed or redefined. Stated router examples compile.

**References:** REQ-RT-001, REQ-RT-002, REQ-RT-003, REQ-RT-004, REQ-RT-005, REQ-RT-006, REQ-RT-008

**Verification method:** automated_test

**Stage:** usable_early

---

**BAR-FERRIC-016 — Active-route awareness**

**Statement:** The `<Link>` component applies an `aria-current="page"` attribute when its href matches the
current path. A stated test or example verifies that navigating to and away from a route toggles the attribute
correctly.

**References:** REQ-RT-007

**Verification method:** automated_test

**Stage:** one_oh

---

**BAR-FERRIC-017 — HTTP client — methods, JSON bodies, error handling**

**Statement:** The `Client` accepts a base URL at configuration time. A `RequestBuilder` chains URL resolution
per RFC 3986, GET/POST/PUT/DELETE methods, a typed `body::<T>()` that serializes `T` to JSON, a
typed `send::<T>()` that deserializes the response body as `T`, and a matchable `Error` enum covering network
failure, non-2xx status, and deserialization failure. Non-2xx responses surface as `Err` by default. Request
headers can be set per-request. The `ferric-http` prelude surfaces present at the baseline are present at 1.0
without removal or redefinition. Stated HTTP client examples compile.

**References:** REQ-HTTP-001, REQ-HTTP-002, REQ-HTTP-003, REQ-HTTP-004, REQ-HTTP-005, REQ-HTTP-006,
REQ-HTTP-007, REQ-HTTP-008, REQ-HTTP-011, REQ-HTTP-013

**Verification method:** automated_test

**Stage:** usable_early

---

**BAR-FERRIC-018 — HTTP client — PATCH, timeout, lean footprint**

**Statement:** The `RequestBuilder` supports the PATCH method in addition to GET/POST/PUT/DELETE. A
per-request timeout cancels the in-flight request and returns an `Error::Timeout` after the specified
duration. The HTTP client's dependency footprint (specifically `serde`, `wasm-bindgen-futures`, and any
added crates) is documented per crate with its bundle-size rationale, and that footprint is reflected in
the performance benchmark numbers.

**References:** REQ-HTTP-009, REQ-HTTP-010, REQ-HTTP-012

**Verification method:** automated_test

**Stage:** one_oh

---

### Documentation and DX

---

**BAR-FERRIC-019 — Conceptual guide and start-to-finish tutorial**

**Statement:** The repository provides a conceptual guide that explains Ferric's reactive model, component
system, and data-loading pattern in terms an author unfamiliar with the framework can read. The repository
also provides a tutorial that starts from a blank project and ends at a compiling, running app that exercises
view!, components, a signal, and at least one `create_resource`; a reviewer can follow the tutorial steps
exactly and reach the stated end state. Documentation is sufficient to learn and build without reading the
framework's source.

**References:** REQ-DX-001, REQ-DX-002, REQ-FW-011

**Verification method:** doc_inspection

**Stage:** one_oh

---

**BAR-FERRIC-020 — API docs and curated examples**

**Statement:** The public API of all three crates (`ferric`, `ferric-router`, `ferric-http`) is rendered on
docs.rs at a published version; each public item carries a doc comment with at least a one-sentence
description. The examples directory contains a curated set that spans the capability range — at minimum a
client-side SPA example and an SSR + Axum + hydrate example. Each example builds from the documented steps.

**References:** REQ-DX-003, REQ-DX-004

**Verification method:** doc_inspection

**Stage:** one_oh

---

**BAR-FERRIC-021 — Starter template, reference app, and new-user acceptance path**

**Statement:** A `cargo generate` (or equivalent scaffold) CSR starter and an SSR + Axum variant are
available; both build and run from the documented commands on stable Rust and the standard toolchain. The
non-toy reference application (`chirp-frontend`) is built on Ferric and exercises reactivity, keyed lists,
events, forms, client routing, and backend data end to end; it builds and runs from documented steps.
Following the documented new-user path from zero — scaffold, serve, tutorial to a compiling end state — can
be completed without reverse-engineering toolchain details.

**References:** REQ-DX-005, REQ-DX-006, REQ-DX-007, REQ-FW-002, REQ-FW-008

**Verification method:** example_build

**Stage:** one_oh

---

**BAR-FERRIC-022 — README, troubleshooting entry, and CI doc sync**

**Statement:** The landing README states what Ferric is, names its Rust/WASM peers with a brief positioning,
and points to the guide, API docs, and getting-started path — all accurately reflecting the 1.0 state. At
least one troubleshooting entry covers a common first-use failure (for example missing `wasm32-unknown-unknown`
target). The CI pipeline verifies that stated code examples in the documentation compile against the current
source, so the docs cannot drift from the API without a CI failure.

**References:** REQ-DX-008, REQ-DX-009, REQ-DX-010

**Verification method:** doc_inspection

**Stage:** one_oh

---

### Release and distribution

---

**BAR-FERRIC-023 — Publishing readiness — crates.io, licensing, workspace versioning**

**Statement:** All three crates (`ferric`, `ferric-router`, `ferric-http`) publish cleanly to crates.io from
`cargo publish` with no git or path dependencies on Ferric itself required to build a consuming project. Each
crate's `Cargo.toml` carries a `license` field with an SPDX identifier, a `description`, a `repository` URL,
and a `rust-version` field matching the workspace's declared MSRV. Intra-workspace dependencies are declared
at a semver version, not a path. The crates are publishable in the order their dependency graph requires.
`cargo add ferric` (and the companion crates) resolves published crates at a semver version from a fresh
project.

**References:** REQ-REL-001, REQ-REL-002, REQ-REL-003, REQ-REL-004, REQ-REL-005, REQ-REL-006, REQ-REL-007,
REQ-REL-008, REQ-REL-009, REQ-REL-010, REQ-REL-011, REQ-FW-001

**Verification method:** publishability

**Stage:** one_oh

---

**BAR-FERRIC-024 — Semver contract and mechanical enforcement**

**Statement:** The public API of each crate is identified; a change that removes or alters an existing public
item in a way that breaks a caller's code requires a major version bump. Additive changes (new items) require
at least a minor bump across the multi-crate workspace. `cargo semver-checks` runs in CI and fails on
violations of this contract before release. An author can read, from the repository, what version-level change
a given kind of modification requires.

**References:** REQ-REL-020, REQ-REL-021, REQ-REL-022, REQ-REL-023, REQ-REL-024, REQ-REL-025, REQ-REL-026,
REQ-REL-027, REQ-REL-028, REQ-REL-029, REQ-FW-010

**Verification method:** automated_test

**Stage:** one_oh

---

**BAR-FERRIC-025 — Changelog — Keep a Changelog 1.1.0**

**Statement:** `CHANGELOG.md` exists at the repository root, follows the Keep a Changelog 1.1.0 format
(sections: Added, Changed, Deprecated, Removed, Fixed, Security; unreleased section at top), and contains an
entry for the 1.0 release naming each user-visible change. Every breaking change in the 1.0 release is listed
under Changed or Removed.

**References:** REQ-REL-030, REQ-REL-031, REQ-REL-032, REQ-REL-033

**Verification method:** doc_inspection

**Stage:** one_oh

---

### Performance

---

**BAR-FERRIC-026 — js-framework-benchmark entry and per-operation results**

**Statement:** A keyed js-framework-benchmark implementation for Ferric is present in the benchmark harness,
uses only the public `ferric` API (no framework internals), and covers all standard keyed-category operations:
create rows, replace all rows, partial update, select row, swap rows, remove row, and create many rows. The
benchmark runs from documented steps on a pinned browser version and pinned dependency versions, and produces
duration and memory numbers for each operation. The setup steps and pinned conditions are documented so a
reviewer can reproduce the run.

**References:** REQ-PERF-001, REQ-PERF-002, REQ-PERF-003, REQ-PERF-004, REQ-PERF-005, REQ-FW-012

**Verification method:** benchmark_run

**Stage:** one_oh

---

**BAR-FERRIC-027 — Bundle size and dependency footprint**

**Statement:** The optimized WASM bundle size for a representative Ferric app (built with `wasm-opt` and
`wasm-bindgen` from documented steps) is measured and reported alongside the benchmark results. The
framework's dependency set — `wasm-bindgen`, `web-sys`, `js-sys` for the core; `serde` and
`wasm-bindgen-futures` for the HTTP client; any other additions — is documented per crate with a one-sentence
justification for each addition. No heavy general-purpose runtime is introduced; the bundle-size impact of the
dependency set is reflected in the measured numbers.

**References:** REQ-PERF-010, REQ-FW-013

**Verification method:** benchmark_run

**Stage:** one_oh

---

**BAR-FERRIC-028 — Peer comparison methodology**

**Statement:** The benchmark results are presented in a table that includes the named Rust/WASM peers (Leptos,
Sycamore, Dioxus, Yew) and places JS frameworks (Solid, Vue 3, vanilla JS) as context. All peers are run
under the same conditions (same benchmark harness, same browser version, same hardware class). The table is a
full per-operation comparison — no operation is omitted from the peer table. Measured numbers are kept
distinct from pre-measurement targets; the document states which numbers are measured results. The WASM-vs-JS
differences in memory and bundle-size accounting are stated explicitly so a reader can interpret the comparison.

**References:** REQ-PERF-006, REQ-PERF-007, REQ-PERF-008, REQ-PERF-009

**Verification method:** doc_inspection

**Stage:** one_oh

---

### Versioning and compatibility

---

**BAR-FERRIC-029 — Versioning and compatibility policy**

**Statement:** The three crates (`ferric`, `ferric-router`, `ferric-http`) version together as a lockstep
unit: they are published at the same version number and a consuming project depends on them at matched
versions. The MSRV is declared in the workspace `Cargo.toml` via `rust-version`; raising the MSRV within the
1.x line requires at minimum a minor version bump and a changelog entry. The `ferric` prelude public surface
is frozen for the 1.x line: new items may be added but no existing exported item is removed or altered
without a major version bump. A written compatibility policy, discoverable from the repository root, states
what the 1.x stability promise covers (the three preludes), what it does not cover (private modules,
`#[doc(hidden)]` items, compiler-internal representations), and the versioning rule for the multi-crate
workspace.

**References:** REQ-REL-040, REQ-REL-041, REQ-REL-042, REQ-REL-043, REQ-REL-044, REQ-REL-045, REQ-REL-046,
REQ-REL-047

**Verification method:** doc_inspection

**Stage:** one_oh

---

## 4. Coverage map

Every `one_oh`-gating `REQ-*` across all eight design areas maps to the `BAR-FERRIC-*` that enforces it.
This table is the primary check for identity I3: every row must have a BAR entry.

| REQ-* id | Stage | BAR-FERRIC-* |
|---|---|---|
| REQ-FW-001 | one_oh | BAR-FERRIC-023 |
| REQ-FW-002 | one_oh | BAR-FERRIC-021 |
| REQ-FW-004 | one_oh | BAR-FERRIC-004 |
| REQ-FW-008 | one_oh | BAR-FERRIC-021 |
| REQ-FW-010 | one_oh | BAR-FERRIC-024 |
| REQ-FW-011 | one_oh | BAR-FERRIC-019 |
| REQ-FW-012 | one_oh | BAR-FERRIC-026 |
| REQ-FW-013 | one_oh | BAR-FERRIC-027 |
| REQ-SSR-001 | one_oh | BAR-FERRIC-003 |
| REQ-SSR-002 | one_oh | BAR-FERRIC-003 |
| REQ-SSR-003 | one_oh | BAR-FERRIC-003 |
| REQ-SSR-004 | one_oh | BAR-FERRIC-004 |
| REQ-SSR-005 | one_oh | BAR-FERRIC-004 |
| REQ-SSR-006 | one_oh | BAR-FERRIC-005 |
| REQ-SSR-007 | one_oh | BAR-FERRIC-005 |
| REQ-SSR-008 | one_oh | BAR-FERRIC-004 |
| REQ-SSR-009 | one_oh | BAR-FERRIC-006 |
| REQ-SSR-010 | one_oh | BAR-FERRIC-003 |
| REQ-SSR-011 | one_oh | BAR-FERRIC-006 |
| REQ-SSR-012 | one_oh | BAR-FERRIC-003, BAR-FERRIC-005 |
| REQ-API-034 | one_oh | BAR-FERRIC-009 |
| REQ-API-045 | one_oh | BAR-FERRIC-009 |
| REQ-API-049 | one_oh | BAR-FERRIC-009 |
| REQ-ASYNC-012 | one_oh | BAR-FERRIC-005 |
| REQ-ASYNC-013 | one_oh | BAR-FERRIC-005 |
| REQ-RT-007 | one_oh | BAR-FERRIC-016 |
| REQ-HTTP-009 | one_oh | BAR-FERRIC-018 |
| REQ-HTTP-010 | one_oh | BAR-FERRIC-018 |
| REQ-HTTP-012 | one_oh | BAR-FERRIC-018 |
| REQ-DX-001 | one_oh | BAR-FERRIC-019 |
| REQ-DX-002 | one_oh | BAR-FERRIC-019 |
| REQ-DX-003 | one_oh | BAR-FERRIC-020 |
| REQ-DX-004 (SSR example) | one_oh | BAR-FERRIC-020 |
| REQ-DX-005 | one_oh | BAR-FERRIC-021 |
| REQ-DX-006 | one_oh | BAR-FERRIC-021 |
| REQ-DX-007 (SSR build docs) | one_oh | BAR-FERRIC-021 |
| REQ-DX-009 | one_oh | BAR-FERRIC-022 |
| REQ-DX-010 | one_oh | BAR-FERRIC-022 |
| REQ-REL-001 | one_oh | BAR-FERRIC-023 |
| REQ-REL-002 | one_oh | BAR-FERRIC-023 |
| REQ-REL-003 | one_oh | BAR-FERRIC-023 |
| REQ-REL-004 | one_oh | BAR-FERRIC-023 |
| REQ-REL-005 | one_oh | BAR-FERRIC-023 |
| REQ-REL-006 | one_oh | BAR-FERRIC-023 |
| REQ-REL-007 | one_oh | BAR-FERRIC-023 |
| REQ-REL-008 | one_oh | BAR-FERRIC-023 |
| REQ-REL-009 | one_oh | BAR-FERRIC-023 |
| REQ-REL-010 | one_oh | BAR-FERRIC-023 |
| REQ-REL-011 | one_oh | BAR-FERRIC-023 |
| REQ-REL-020 | one_oh | BAR-FERRIC-024 |
| REQ-REL-021 | one_oh | BAR-FERRIC-024 |
| REQ-REL-022 | one_oh | BAR-FERRIC-024 |
| REQ-REL-023 | one_oh | BAR-FERRIC-024 |
| REQ-REL-025 | one_oh | BAR-FERRIC-024 |
| REQ-REL-026 | one_oh | BAR-FERRIC-024 |
| REQ-REL-028 | one_oh | BAR-FERRIC-024 |
| REQ-REL-029 | one_oh | BAR-FERRIC-024 |
| REQ-REL-030 | one_oh | BAR-FERRIC-025 |
| REQ-REL-031 | one_oh | BAR-FERRIC-025 |
| REQ-REL-032 | one_oh | BAR-FERRIC-025 |
| REQ-REL-033 | one_oh | BAR-FERRIC-025 |
| REQ-REL-040 | one_oh | BAR-FERRIC-029 |
| REQ-REL-041 | one_oh | BAR-FERRIC-029 |
| REQ-REL-042 | one_oh | BAR-FERRIC-029 |
| REQ-REL-043 | one_oh | BAR-FERRIC-029 |
| REQ-REL-044 | one_oh | BAR-FERRIC-029 |
| REQ-REL-045 | one_oh | BAR-FERRIC-029 |
| REQ-REL-046 | one_oh | BAR-FERRIC-029 |
| REQ-REL-047 | one_oh | BAR-FERRIC-029 |
| REQ-PERF-001 | one_oh | BAR-FERRIC-026 |
| REQ-PERF-002 | one_oh | BAR-FERRIC-026 |
| REQ-PERF-003 | one_oh | BAR-FERRIC-026 |
| REQ-PERF-004 | one_oh | BAR-FERRIC-026 |
| REQ-PERF-005 | one_oh | BAR-FERRIC-026 |
| REQ-PERF-006 | one_oh | BAR-FERRIC-028 |
| REQ-PERF-007 | one_oh | BAR-FERRIC-028 |
| REQ-PERF-008 | one_oh | BAR-FERRIC-028 |
| REQ-PERF-009 | one_oh | BAR-FERRIC-028 |
| REQ-PERF-010 | one_oh | BAR-FERRIC-027 |

**Totals:** 29 BAR-FERRIC-* criteria — 12 `usable_early`, 17 `one_oh`. One criterion marked `must_have: true`
(BAR-FERRIC-004, SSR + hydration).

---

## 5. Completeness & gap review

This section sweeps every surface a Rust/WASM UI framework at Ferric's level is expected to have and records,
for each, whether the 1.0 design covers it, explicitly defers it, or leaves a gap. Surfaces covered by the
design carry the `REQ-*` ids of the requirements that address them. Out-of-1.0 surfaces carry their deferral
reason and the `REQ-FW-*` id that records the deferral.

### Required surfaces

| Surface | Status | req_refs | Notes |
|---|---|---|---|
| Reactive state model (signals, memos, effects, batch) | covered | REQ-FW-005, REQ-FW-009 | REQ-FW-005 requires fine-grained signal reactivity with scoped cleanup; REQ-FW-009 requires the baseline prelude to carry forward additively. |
| Virtual DOM and DOM patching | covered | REQ-FW-005 | REQ-FW-005 covers the reactive rendering model including the keyed VNode diff applied to the live DOM. |
| Server-side rendering (SSR) | covered | REQ-SSR-001, REQ-SSR-002, REQ-SSR-003, REQ-SSR-010, REQ-SSR-012 | REQ-SSR-001 through REQ-SSR-003 require server HTML rendering, a renderable core that builds off WASM, and per-request isolation; REQ-SSR-010 requires injection-safe output; REQ-SSR-012 covers the resource serialization seam. |
| Hydration (client picks up server HTML, no re-render) | covered | REQ-SSR-004, REQ-SSR-005, REQ-SSR-008, REQ-FW-004, REQ-FW-008 | REQ-SSR-004 (the gating must-have) requires hydration with no client re-render; REQ-SSR-005 covers event wiring; REQ-SSR-008 requires first-render determinism; REQ-FW-004 is the cross-cutting must-have; REQ-FW-008 requires a reference app to demonstrate the full composition. |
| Authoring ergonomics / view macro / JSX-style DSL | covered | REQ-API-001, REQ-API-002, REQ-API-003, REQ-API-004, REQ-API-006, REQ-API-012, REQ-API-015, REQ-API-023 | REQ-API Band A (001–029) covers the view! macro surface: HTML elements, dynamic expressions, conditionals, keyed lists via `<For>`, component invocation. |
| Component model (props, children, slots) | covered | REQ-API-030, REQ-API-034, REQ-API-045, REQ-API-049 | REQ-API Band B (030–052) covers required/optional props with defaults, children as VNode, named slots, converting props (034), parameterized slots (045), and dynamic dispatch (049). |
| Context / dependency injection | covered | REQ-API-060, REQ-API-061, REQ-API-063, REQ-API-066, REQ-API-068 | REQ-API Band C (060–070) requires provide_context, use_context, and expect_context over the component tree. |
| Async resources and suspense | covered | REQ-ASYNC-001, REQ-ASYNC-012, REQ-ASYNC-013 | REQ-ASYNC-001–016 cover create_resource with four states, suspense boundaries, and the SSR resource seam; REQ-ASYNC-012 and REQ-ASYNC-013 (SSR seam) are one_oh, the rest usable_early. |
| Forms and two-way data binding | covered | REQ-ASYNC-040, REQ-ASYNC-041, REQ-ASYNC-042, REQ-ASYNC-043 | REQ-ASYNC-040–043 cover two-way binding, the full input-kind range, validation ergonomics, and form submission. |
| DOM event handling | covered | REQ-ASYNC-020, REQ-ASYNC-021, REQ-ASYNC-022, REQ-ASYNC-023, REQ-ASYNC-024, REQ-FW-006 | REQ-ASYNC-020–024 and REQ-FW-006 cover the payload-handler form, the full DOM event range, default prevention, propagation control, and the string-name escape hatch. |
| Client-side routing (URL params, nested routes, navigation) | covered | REQ-RT-001, REQ-RT-002, REQ-RT-003, REQ-RT-004, REQ-RT-005, REQ-RT-006, REQ-RT-007, REQ-RT-008 | REQ-RT-001–008 cover the route table with path parameters, reactive outlet, History-API navigation, accessible links, nested routes with nested outlet, per-level fallback, active-route awareness, and the additive-over-frozen-surface identity. URL query/search parameters are a recorded post-1.0 deferral (REQ-RT-009). |
| HTTP / JSON client | covered | REQ-HTTP-001, REQ-HTTP-004, REQ-HTTP-005, REQ-HTTP-007, REQ-HTTP-008, REQ-HTTP-009, REQ-HTTP-010, REQ-HTTP-013 | REQ-HTTP-001–013 cover client configuration, fluent request building with typed JSON bodies, a matchable error model, safe-by-default non-2xx handling, PATCH method (one_oh), and per-request timeout/cancellation (one_oh). |
| Developer documentation (guide, tutorial, API reference) | covered | REQ-DX-001, REQ-DX-002, REQ-DX-003, REQ-DX-004, REQ-DX-008, REQ-DX-009, REQ-DX-010 | REQ-DX-001–010 require a guide, a tutorial with a compiling end state, rendered API docs, curated examples including an SSR + hydrate example, a troubleshooting entry, an accurate README, and CI-enforced doc sync. |
| DX (starter template, cargo-generate, error messages) | covered | REQ-DX-005, REQ-DX-006, REQ-DX-007, REQ-FW-014 | REQ-DX-005–007 require a cargo-generate CSR starter and an SSR + Axum variant, build and toolchain documentation for both paths, and the new-user acceptance path; REQ-FW-014 requires trybuild-checkable macro diagnostics. |
| Release and publishing (crates.io, semver, changelog) | covered | REQ-REL-001, REQ-REL-004, REQ-REL-005, REQ-REL-009, REQ-REL-020, REQ-REL-021, REQ-REL-026, REQ-REL-030, REQ-REL-031 | REQ-REL-001–011 cover licensing, per-crate crates.io metadata, path-to-version conversion, cargo package readiness, and coordinated lockstep release; REQ-REL-020–033 cover the semver level contract, breaking-vs-additive classification with mechanical enforcement, and changelog practice following Keep a Changelog 1.1.0. |
| Versioning / compatibility policy | covered | REQ-REL-040, REQ-REL-041, REQ-REL-042, REQ-REL-043, REQ-REL-044, REQ-REL-046, REQ-REL-047 | REQ-REL-040–047 declare the 1.0 MSRV via the rust-version field, the MSRV raise rule within 1.x, frozen-prelude stability for the 1.x line, additive-only public-surface evolution, lockstep versioning, and a written discoverable compatibility policy. |
| Performance and bundle size | covered | REQ-PERF-001, REQ-PERF-002, REQ-PERF-003, REQ-PERF-004, REQ-PERF-010 | REQ-PERF-001–004 and REQ-PERF-010 require a keyed js-framework-benchmark entry using Ferric's public API, per-operation speed and memory metrics, and bundle and startup size from the optimized build tied to the documented dependency set. |
| Peer comparison methodology | covered | REQ-PERF-005, REQ-PERF-006, REQ-PERF-007, REQ-PERF-008, REQ-PERF-009 | REQ-PERF-006–009 require a stated comparison method (same conditions, keyed category, full per-operation table, no cherry-picking, measured results kept distinct from pre-measurement targets), comparison against the named Rust/WASM peers, and the JS field as context with WASM-vs-JS caveats stated. |
| Accessibility of framework output (ARIA, semantic HTML) | gap | | No REQ-* in any spoke requires the framework's generated markup to meet any accessibility standard beyond the link component. REQ-RT-004 requires the `<Link>` component to be keyboard-focusable and announced by assistive technology (covered by BAR-FERRIC-015); REQ-RT-007 requires `aria-current="page"` on the active link (covered by BAR-FERRIC-016). The general case — ARIA roles, semantic HTML, and keyboard navigation for all framework-generated markup — is not addressed in the 1.0 design. |
| Testing story (how app authors test components, routes, async) | gap | | No REQ-* in any spoke addresses how an app author tests their Ferric components, routes, or async flows. REQ-FW-014 and REQ-API-* cover trybuild-checkable macro diagnostics; REQ-DX-010 covers CI for the framework's own docs and examples; but neither provides a path for app authors to write component or integration tests. cargo test and wasm-bindgen-test work, but the framework provides no testing utilities, no component-mount helper, and no guidance for what to test and how. |
| MSRV policy | covered | REQ-FW-003, REQ-REL-040, REQ-REL-041 | REQ-FW-003 requires stable Rust and the standard WASM toolchain; REQ-REL-040 requires the MSRV to be declared at 1.0 via the rust-version field; REQ-REL-041 states the rule for raising it within 1.x. |
| Scoped CSS / styling story | out-of-1.0 | | REQ-FW-015 defers scoped CSS and a first-class styling story to post-1.0; app authors use global stylesheets and external CSS tooling at 1.0. |
| Browser devtools / inspector integration | out-of-1.0 | | REQ-FW-016 defers browser devtools integration (component inspector, signal viewer) to post-1.0. |
| Hot reload / live reload | out-of-1.0 | | REQ-FW-017 defers hot/live reload to post-1.0; the standard edit-rebuild-refresh loop is the documented 1.0 development path. |
| Desktop / mobile targets (non-WASM) | out-of-1.0 | | REQ-FW-018 defers non-WASM targets (desktop, mobile, native) to post-1.0; Ferric 1.0 targets wasm32-unknown-unknown only. |

### Additional surfaces found in spoke designs

The following surfaces are explicitly deferred in spoke outputs and do not appear in the required surface list.

| Surface | Status | req_refs | Notes |
|---|---|---|---|
| Streaming SSR | out-of-1.0 | | The SSR design explicitly records streaming SSR as outside the 1.0 bar; the 1.0 model is full-page, eager server rendering with no REQ-SSR-* assigned to streaming. |
| Server functions / server actions | out-of-1.0 | | The SSR design explicitly records server functions and server actions as outside the 1.0 bar; the async resource pattern (REQ-ASYNC-001–016) is the 1.0 data-loading model. |
| Islands / partial hydration | out-of-1.0 | | The SSR design explicitly records islands architecture and partial hydration as outside the 1.0 bar; REQ-SSR-004 covers full-page hydration only. |
| URL query / search-string parameters | out-of-1.0 | | REQ-RT-009 explicitly defers router-owned reactive query/search-string parameters to post-1.0; app authors can read `window.location.search` directly for 1.0. |

### Gap findings — release plan implications

Two surfaces are neither covered by the design nor explicitly deferred:

**Accessibility of framework output.** The design does not require the framework's generated markup beyond the
link component to meet any accessibility standard. For a framework whose reference consumer (chirp) is a
public-facing web app, this gap means accessibility compliance of chirp's UI depends entirely on app-level
markup choices, with no framework assistance or guarantee. The release plan should decide whether to record
this as a known 1.0 limitation or to add a minimal framework-level accessibility requirement (for example
semantic HTML defaults for common elements) before release.

**Testing story.** The design provides no component testing utilities, no test harness integration beyond what
cargo test and wasm-bindgen-test already offer, and no guidance for app authors on how to test Ferric
components or async flows. An app author testing a Ferric component today must manually mount it into a DOM
environment and assert against it without framework-provided helpers. The release plan should decide whether
to address this with at minimum a testing guide section in the docs.

---

*BAR-FERRIC-* register version: 1.0 · Design area: Ferric 1.0 framework ·
Produced by: SUBTASK_rbar_synthesis · Repo target: design/ferric/release-bar.md*
