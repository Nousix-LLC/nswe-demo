# Ferric 1.0 — Release Plan

This document is the authoritative release plan for Ferric 1.0: eight epics under board issue #1, each a coherent, landable capability with acceptance criteria tracing to the design. The epics fold in the existing router (#5) and HTTP/JSON-client (#6) issues and are ordered so the usable-early floor lands before the full 1.0 bar.

---

## Epic Landscape

Eight epics sit under issue #1. Four are `usable_early` (together they constitute a working client-side framework); four are `one_oh` (required for the 1.0 release bar, with SSR and Hydration as the must-have).

| Epic | Design area(s) | BAR-FERRIC | Stage | Depends on | Issue fold |
|---|---|---|---|---|---|
| Authoring Model | authoring-model; scope-and-requirements | BAR-001, BAR-002, BAR-007–011 | usable_early | — | — |
| Async, Forms, and Events | async-forms-events; scope-and-requirements | BAR-012, BAR-013, BAR-014 | usable_early | Authoring Model | — |
| Client-side Router | router-and-http Part 1; scope-and-requirements | BAR-015, BAR-016 | usable_early | Authoring Model | extends #5 |
| HTTP/JSON Client | router-and-http Part 2; scope-and-requirements | BAR-017, BAR-018 | usable_early | Authoring Model | extends #6 |
| SSR and Hydration | ssr-hydration; scope-and-requirements | BAR-003, BAR-004 (must-have), BAR-005, BAR-006 | one_oh | Authoring Model, Async/Forms/Events | — |
| Docs and DX | docs-and-dx; scope-and-requirements | BAR-019, BAR-020, BAR-021, BAR-022 | one_oh | all usable_early + SSR | — |
| Performance and Benchmark | performance-and-benchmark; scope-and-requirements | BAR-026, BAR-027, BAR-028 | one_oh | Authoring Model, Async/Forms/Events, SSR | — |
| Release and Distribution | release-and-distribution; scope-and-requirements | BAR-023, BAR-024, BAR-025, BAR-029 | one_oh | Docs and DX, Performance and Benchmark | — |

---

## Ordering

**Usable framework (usable_early epics):** Authoring Model lands first — it is the foundation every other epic builds on. Async/Forms/Events, Client-side Router, and HTTP/JSON Client each depend only on Authoring Model and have no dependency on each other; they may proceed in any order or in parallel. When these four epics are done, an author has a complete client-side framework: signals, components, context, async resources, forms, events, routing, and HTTP. Real applications can be built and exercised against `chirp-frontend`.

**Full 1.0 bar (one_oh epics):** SSR and Hydration is the must-have (`BAR-FERRIC-004` carries `must_have: true`). It depends on Authoring Model and Async/Forms/Events. After SSR and Hydration lands, Docs and DX and Performance and Benchmark can proceed in parallel (both depend on SSR plus the usable_early epics). Release and Distribution is the final epic; it waits on Docs and DX and Performance and Benchmark so the crates.io publish happens when documentation and benchmark data are both complete.

```
Authoring Model
    ├── Async, Forms, and Events ──────────────────────┐
    ├── Client-side Router (extends #5) ───────────────┤
    └── HTTP/JSON Client (extends #6) ─────────────────┤
                                                       │
         Async, Forms, and Events ─────────────────────┤
         Authoring Model ─────────────────────────────►  SSR and Hydration (must-have)
                                                                │
                 all usable_early + SSR ────────────────────────┤
                                                       ┌────────┘
                                                       ├── Docs and DX
                                                       └── Performance and Benchmark
                                                                │
                                                         Release and Distribution
```

---

## Issues #5 and #6 — Fold

Neither issue is re-opened or superseded; both are extended by a new epic that adds capabilities over the frozen prelude surface each issue shipped.

**Issue #5 (ferric-router scaffold)** shipped `crates/ferric-router` with a flat route table, path parameters, History-API push navigation, a reactive outlet, an in-app link, and a frozen `prelude` surface. That surface is carried forward unchanged. The Client-side Router epic adds, additively: nested routes with a nested outlet, replace navigation, active-route awareness for links (`REQ-RT-007`, `BAR-016`), and per-level fallback handling. Issue #5 is closed (done); the epic extends its scope without reopening or superseding it.

**Issue #6 (ferric-http scaffold)** shipped `crates/ferric-http` with typed GET/POST/PUT/DELETE, serde, error handling, and a frozen `prelude` surface. That surface is carried forward unchanged. The HTTP/JSON Client epic adds, additively: PATCH support, configurable timeout, and a lean-dependency-footprint requirement (`REQ-HTTP-*` additions, `BAR-018`). Issue #6 is closed (done); the epic extends its scope without reopening or superseding it.

### Fold table (for SUBTASK_file_backlog)

| Issue | Current status | Action |
|---|---|---|
| #5 — ferric-router scaffold | Closed | Add a comment on #5 pointing to the Client-side Router epic; note that the epic extends #5 |
| #6 — ferric-http scaffold | Closed | Add a comment on #6 pointing to the HTTP/JSON Client epic; note that the epic extends #6 |

---

## Work Items by Epic

Work items are the smallest units that can be assigned, built, reviewed, and merged independently. Acceptance criteria cite the `REQ-*` and `BAR-FERRIC-*` ids each item satisfies and the design-doc section it traces to. No `REQ-*` or `BAR-FERRIC-*` id is redefined here.

---

### Authoring Model (usable_early)

**Design areas:** authoring-model.md, scope-and-requirements.md  
**BAR-FERRIC:** BAR-001, BAR-002, BAR-007, BAR-008, BAR-009, BAR-010, BAR-011  
**Depends on:** none

#### Reactive state primitives

**Acceptance criteria:** `create_signal`, `create_memo`, `create_effect`, and `batch` are present in the `ferric` prelude and compile on stable Rust at the declared MSRV. A test confirms `create_memo` does not recompute when its output is unchanged; a test confirms `batch` defers effect execution until the batch closes; a test confirms scoped cleanup unsubscribes every reactive subscription owned by a dropped scope without observable leaks. The `ferric` prelude from the baseline compiles at 1.0 without modification to any previously exported item. Traces to: authoring-model.md §reactive-core (REQ-FW-003, REQ-FW-005, REQ-FW-009); BAR-FERRIC-001.  
**Depends on:** none  
**Verification:** automated_test

#### Virtual DOM and keyed DOM patching

**Acceptance criteria:** A test demonstrates that a partial signal change updates only the affected DOM nodes — not the whole subtree. A test demonstrates that a keyed list reorder moves existing DOM nodes rather than destroying and recreating them. Traces to: authoring-model.md §vdom (REQ-FW-005); BAR-FERRIC-002.  
**Depends on:** Reactive state primitives  
**Verification:** automated_test

#### view! macro surface and compile-time diagnostics

**Acceptance criteria:** The stated set of valid view! inputs — HTML elements with their standard attributes and properties, dynamic expressions, conditional rendering, keyed list rendering via `<For key=... each=... view=...>`, and component invocations — all compile without modification. The stated set of malformed inputs — unrecognized element names, missing required attributes, structurally invalid expressions — each produce a compile-time diagnostic that names the problem in terms of the author's code rather than macro internals; trybuild tests for that set pass in CI. Traces to: authoring-model.md §view-macro (REQ-API-001, REQ-API-002, REQ-API-003, REQ-API-004, REQ-API-006, REQ-API-012, REQ-API-015, REQ-API-023, REQ-API-024, REQ-API-025, REQ-API-026, REQ-API-027, REQ-API-028, REQ-FW-014); BAR-FERRIC-007, BAR-FERRIC-008.  
**Depends on:** Virtual DOM and keyed DOM patching  
**Verification:** automated_test

#### Component model: props, children, named slots, and dynamic dispatch

**Acceptance criteria:** A component function accepts required and optional props (with defaults), a `children: VNode` parameter, and named slots; converting props via a declared `From` implementation; parameterized slot closures (`SlotFn<T>`); and dynamic invocation through a `dyn Component` trait object. Stated examples for each of these forms compile and render correctly. Traces to: authoring-model.md §component-model (REQ-API-030, REQ-API-031, REQ-API-032, REQ-API-033, REQ-API-034, REQ-API-035, REQ-API-037, REQ-API-038, REQ-API-041, REQ-API-045, REQ-API-047, REQ-API-049); BAR-FERRIC-009.  
**Depends on:** view! macro surface and compile-time diagnostics  
**Verification:** automated_test

#### Component type-level diagnostics

**Acceptance criteria:** The stated set of component authoring mistakes — missing required props, type mismatches in prop values, misused slots — each produce a compile-time error that names the problem at the call site rather than inside framework internals; trybuild tests for that set pass in CI. Traces to: authoring-model.md §component-diagnostics (REQ-API-050, REQ-API-051, REQ-API-052); BAR-FERRIC-010.  
**Depends on:** Component model: props, children, named slots, and dynamic dispatch  
**Verification:** automated_test

#### Context and dependency injection

**Acceptance criteria:** `provide_context`, `use_context` (returning `Option`), and `expect_context` (panicking with a clear message when the value is absent) are present in the `ferric` prelude; the value provided does not need to be threaded through every intermediate component; stated examples for each form compile and the tests pass. Traces to: authoring-model.md §context (REQ-API-060, REQ-API-061, REQ-API-063, REQ-API-066, REQ-API-068); BAR-FERRIC-011.  
**Depends on:** Component model: props, children, named slots, and dynamic dispatch  
**Verification:** automated_test

---

### Async, Forms, and Events (usable_early)

**Design areas:** async-forms-events.md, scope-and-requirements.md  
**BAR-FERRIC:** BAR-012, BAR-013, BAR-014  
**Depends on:** Authoring Model

*REQ-ASYNC-012 and REQ-ASYNC-013 (the SSR resource serialization seam) are `one_oh` requirements covered under SSR and Hydration — State carry and async resource resume, which depends on both this epic and the SSR work.*

#### Async resources and suspense

**Acceptance criteria:** `create_resource` tracks four states — loading, loaded, empty, and error — and re-renders consumers when the state changes; `<Suspense>` shows the fallback while any enclosed resource is loading and switches to resolved content when all are loaded; the boundary can be nested; a resource can be refetched on demand, keeping previous data visible during the refetch; `<ErrorBoundary>` renders the error slot with the error value and allows retry; a deferred resource starts loading only when first read; reactive scope cleanup cancels any in-flight resource load. All stated test cases for these states and transitions pass. Traces to: async-forms-events.md §async-resource (REQ-ASYNC-001, REQ-ASYNC-002, REQ-ASYNC-004, REQ-ASYNC-005, REQ-ASYNC-006, REQ-ASYNC-007, REQ-ASYNC-008, REQ-ASYNC-009, REQ-ASYNC-010, REQ-ASYNC-011, REQ-ASYNC-015, REQ-ASYNC-016); BAR-FERRIC-012.  
**Depends on:** Component model: props, children, named slots, and dynamic dispatch  
**Verification:** automated_test

#### DOM event handling

**Acceptance criteria:** An author attaches handlers to DOM events using `on:<event_name>={handler}` syntax in view!; the handler receives a typed payload (the concrete event type) for all named events: click, input, change, submit, keydown, keyup, keypress, focus, blur; `event.prevent_default()` and `event.stop_propagation()` compile and work; for an event not in the named set, an author can attach a handler by string name as an escape hatch. Examples for each category compile. Traces to: async-forms-events.md §event-handling (REQ-ASYNC-020, REQ-ASYNC-021, REQ-ASYNC-022, REQ-ASYNC-023, REQ-ASYNC-024, REQ-FW-006); BAR-FERRIC-013.  
**Depends on:** view! macro surface and compile-time diagnostics  
**Verification:** automated_test

#### Forms and two-way binding

**Acceptance criteria:** An author binds a signal to a text input, checkbox, and `<select>` two-way: the input reflects the current signal value and a change to the input updates the signal; a submit handler on a `<form>` fires with the form data; validation feedback can be wired to a signal updated on submit or on change. Stated examples for each input kind compile and demonstrate the two-way relationship. Traces to: async-forms-events.md §forms (REQ-ASYNC-040, REQ-ASYNC-041, REQ-ASYNC-042, REQ-ASYNC-043); BAR-FERRIC-014.  
**Depends on:** DOM event handling  
**Verification:** automated_test

---

### Client-side Router (usable_early — extends #5)

**Design areas:** router-and-http.md Part 1, scope-and-requirements.md  
**BAR-FERRIC:** BAR-015, BAR-016  
**Depends on:** Authoring Model  
**Issue fold:** extends #5

#### Core routing: path parameters, nested routes, navigation, and accessible links

**Acceptance criteria:** The router matches path patterns with named parameters and renders the matching route's component in a reactive `<Outlet>`; nested routes nest their `<Outlet>` inside a parent route's view; each route level can declare a fallback for no-match; `navigate(path)` and the History-API push/replace functions change the current path reactively; the `<Link>` component is keyboard-focusable and announced by assistive technology as a link; all `ferric-router` prelude surfaces present at the #5 baseline are present at 1.0 without removal or redefinition; stated router examples compile. Traces to: router-and-http.md §client-router (REQ-RT-001, REQ-RT-002, REQ-RT-003, REQ-RT-004, REQ-RT-005, REQ-RT-006, REQ-RT-008); BAR-FERRIC-015.  
**Depends on:** view! macro surface and compile-time diagnostics  
**Verification:** automated_test

#### Active-route awareness (one_oh)

**Acceptance criteria:** The `<Link>` component applies `aria-current="page"` when its href matches the current path; a test or example demonstrates that navigating to and away from a route toggles the attribute correctly. Traces to: router-and-http.md §active-link (REQ-RT-007); BAR-FERRIC-016.  
**Depends on:** Core routing: path parameters, nested routes, navigation, and accessible links  
**Verification:** automated_test

---

### HTTP/JSON Client (usable_early — extends #6)

**Design areas:** router-and-http.md Part 2, scope-and-requirements.md  
**BAR-FERRIC:** BAR-017, BAR-018  
**Depends on:** Authoring Model  
**Issue fold:** extends #6

#### HTTP methods, JSON bodies, and error handling

**Acceptance criteria:** `Client` accepts a base URL at configuration time; `RequestBuilder` chains URL resolution per RFC 3986, GET/POST/PUT/DELETE methods, a typed `body::<T>()` that serializes `T` to JSON, a typed `send::<T>()` that deserializes the response body as `T`, and a matchable `Error` enum covering network failure, non-2xx status, and deserialization failure; non-2xx responses surface as `Err` by default; request headers can be set per-request; all `ferric-http` prelude surfaces present at the #6 baseline are present at 1.0 without removal or redefinition; stated HTTP client examples compile. Traces to: router-and-http.md §http-client (REQ-HTTP-001, REQ-HTTP-002, REQ-HTTP-003, REQ-HTTP-004, REQ-HTTP-005, REQ-HTTP-006, REQ-HTTP-007, REQ-HTTP-008, REQ-HTTP-011, REQ-HTTP-013); BAR-FERRIC-017.  
**Depends on:** Reactive state primitives  
**Verification:** automated_test

#### PATCH method, per-request timeout, and lean dependency footprint (one_oh)

**Acceptance criteria:** `RequestBuilder` supports the PATCH method; a per-request timeout cancels the in-flight request and returns `Error::Timeout` after the specified duration; the HTTP client's dependency footprint (at minimum serde and wasm-bindgen-futures) is documented per crate with a one-sentence bundle-size rationale, and that footprint is reflected in the performance benchmark numbers. Traces to: router-and-http.md §http-extensions (REQ-HTTP-009, REQ-HTTP-010, REQ-HTTP-012); BAR-FERRIC-018.  
**Depends on:** HTTP methods, JSON bodies, and error handling; Performance and Benchmark — Bundle size and dependency footprint  
**Verification:** automated_test

---

### SSR and Hydration (one_oh — BAR-FERRIC-004 carries must_have: true)

**Design areas:** ssr-hydration.md, scope-and-requirements.md  
**BAR-FERRIC:** BAR-003, BAR-004 (must_have), BAR-005, BAR-006  
**Depends on:** Authoring Model, Async/Forms/Events

#### Server-side HTML rendering and per-request scope isolation

**Acceptance criteria:** The framework renders an app to an HTML string on the server using a renderable core that compiles with `--features ssr` on the server's native target (off WASM); each concurrent request gets its own isolated reactive scope; the produced HTML passes a standard HTML sanitizer without injected markup. A test produces the HTML string for a representative component and checks its content without running a browser. Traces to: ssr-hydration.md §server-rendering (REQ-SSR-001, REQ-SSR-002, REQ-SSR-003, REQ-SSR-010, REQ-SSR-012); BAR-FERRIC-003.  
**Depends on:** view! macro surface and compile-time diagnostics  
**Verification:** automated_test

#### Client hydration without re-render (must-have)

**Acceptance criteria:** The client WASM bundle attaches to the server-rendered DOM without replacing any server-rendered element; after hydration, event handlers fire and reactive state responds to changes; no DOM node present in the server HTML is torn down and rebuilt on load. An SSR + hydrate example demonstrates the complete path: server renders HTML, browser receives it as the first paint, WASM loads and wires up, interaction works without a visible repaint. Traces to: ssr-hydration.md §hydration (REQ-SSR-004, REQ-SSR-005, REQ-SSR-008, REQ-FW-004, REQ-FW-008); BAR-FERRIC-004.  
**Depends on:** Server-side HTML rendering and per-request scope isolation; Async resources and suspense  
**Verification:** example_build

#### State carry and async resource resume across the SSR seam

**Acceptance criteria:** State computed during the server render (including resolved async resources) is serialized into the page and deserialized on the client without a refetch; the client resource starts in the loaded state with the server's data, not the loading state. An SSR + hydrate example with at least one async resource confirms that no network request fires for that resource during hydration. Traces to: ssr-hydration.md §state-carry (REQ-SSR-006, REQ-SSR-007, REQ-SSR-012, REQ-ASYNC-012, REQ-ASYNC-013); BAR-FERRIC-005.  
**Depends on:** Client hydration without re-render  
**Verification:** example_build

#### Hydration mismatch detection and Axum integration

**Acceptance criteria:** When the client-rendered structure diverges from the server HTML, Ferric detects the mismatch and recovers without a silent fault. An Axum integration example shows a reference Axum server handler rendering Ferric to HTML and returning it in an HTTP response that the browser then hydrates; the example builds from the documented steps. Traces to: ssr-hydration.md §mismatch-and-axum (REQ-SSR-009, REQ-SSR-011); BAR-FERRIC-006.  
**Depends on:** Client hydration without re-render  
**Verification:** example_build

---

### Docs and DX (one_oh)

**Design areas:** docs-and-dx.md, scope-and-requirements.md  
**BAR-FERRIC:** BAR-019, BAR-020, BAR-021, BAR-022  
**Depends on:** Authoring Model, Async/Forms/Events, Client-side Router, HTTP/JSON Client, SSR and Hydration

#### Conceptual guide and start-to-finish tutorial

**Acceptance criteria:** A conceptual guide explains Ferric's reactive model, component system, and data-loading pattern in terms an author unfamiliar with the framework can read. A tutorial starts from a blank project and ends at a compiling, running app exercising view!, components, a signal, and at least one `create_resource`; a reviewer can follow the tutorial steps exactly and reach the stated end state. The documentation is sufficient to learn and build without reading the framework's source. Traces to: docs-and-dx.md §guide-and-tutorial (REQ-DX-001, REQ-DX-002, REQ-FW-011); BAR-FERRIC-019.  
**Depends on:** Client hydration without re-render  
**Verification:** doc_inspection

#### API docs and curated examples

**Acceptance criteria:** The public API of all three crates (`ferric`, `ferric-router`, `ferric-http`) is rendered on docs.rs at a published version; each public item carries a doc comment with at least a one-sentence description; the examples directory contains at minimum a client-side SPA example and an SSR + Axum + hydrate example, each building from the documented steps. Traces to: docs-and-dx.md §api-docs (REQ-DX-003, REQ-DX-004); BAR-FERRIC-020.  
**Depends on:** Conceptual guide and start-to-finish tutorial  
**Verification:** doc_inspection

#### Starter template, reference app, and new-user acceptance path

**Acceptance criteria:** A `cargo generate` CSR starter and an SSR + Axum variant are available; both build and run from the documented commands on stable Rust and the standard toolchain. The `chirp-frontend` reference app exercises reactivity, keyed lists, events, forms, client routing, and backend data end to end and builds from documented steps. Following the documented new-user path from zero — scaffold, serve, tutorial to a compiling end state — is completable without reverse-engineering toolchain details. Traces to: docs-and-dx.md §starters-and-refapp (REQ-DX-005, REQ-DX-006, REQ-DX-007, REQ-FW-002, REQ-FW-008); BAR-FERRIC-021.  
**Depends on:** API docs and curated examples  
**Verification:** example_build

#### README, troubleshooting entry, and CI doc sync

**Acceptance criteria:** The landing README states what Ferric is, names its Rust/WASM peers with brief positioning, and points to the guide, API docs, and getting-started path — all accurately reflecting the 1.0 state; at least one troubleshooting entry covers a common first-use failure (e.g., missing `wasm32-unknown-unknown` target); a CI pipeline check confirms that stated code examples in the documentation compile against the current source. Traces to: docs-and-dx.md §readme-and-ci (REQ-DX-008, REQ-DX-009, REQ-DX-010); BAR-FERRIC-022.  
**Depends on:** Conceptual guide and start-to-finish tutorial  
**Verification:** doc_inspection

---

### Performance and Benchmark (one_oh)

**Design areas:** performance-and-benchmark.md, scope-and-requirements.md  
**BAR-FERRIC:** BAR-026, BAR-027, BAR-028  
**Depends on:** Authoring Model, Async/Forms/Events, SSR and Hydration

#### js-framework-benchmark entry and per-operation results

**Acceptance criteria:** A keyed js-framework-benchmark implementation for Ferric is present in the benchmark harness, uses only the public `ferric` API (no framework internals), and covers all standard keyed-category operations: create rows, replace all rows, partial update, select row, swap rows, remove row, and create many rows. The benchmark runs from documented steps on a pinned browser version and pinned dependency versions and produces duration and memory numbers for each operation; the setup steps and pinned conditions are documented so a reviewer can reproduce the run. Traces to: performance-and-benchmark.md §benchmark-entry (REQ-PERF-001, REQ-PERF-002, REQ-PERF-003, REQ-PERF-004, REQ-PERF-005, REQ-FW-012); BAR-FERRIC-026.  
**Depends on:** Client hydration without re-render; Async resources and suspense  
**Verification:** benchmark_run

#### Bundle size and dependency footprint

**Acceptance criteria:** The optimized WASM bundle size for a representative Ferric app (built with `wasm-opt` and `wasm-bindgen` from documented steps) is measured and reported alongside the benchmark results; the framework's dependency set is documented per crate with a one-sentence justification for each addition; no heavy general-purpose runtime is introduced; the bundle-size impact of the dependency set is reflected in the measured numbers. Traces to: performance-and-benchmark.md §bundle-size (REQ-PERF-010, REQ-FW-013); BAR-FERRIC-027.  
**Depends on:** js-framework-benchmark entry and per-operation results  
**Verification:** benchmark_run

#### Peer comparison table and methodology

**Acceptance criteria:** Benchmark results are presented in a table that includes the named Rust/WASM peers (Leptos, Sycamore, Dioxus, Yew) and places JS frameworks (Solid, Vue 3, vanilla JS) as context; all peers are run under the same conditions (same benchmark harness, same browser version, same hardware class); every standard keyed-category operation appears in the peer table; measured numbers are kept distinct from pre-measurement targets; WASM-vs-JS differences in memory and bundle-size accounting are stated so a reader can interpret the comparison. Traces to: performance-and-benchmark.md §peer-comparison (REQ-PERF-006, REQ-PERF-007, REQ-PERF-008, REQ-PERF-009); BAR-FERRIC-028.  
**Depends on:** Bundle size and dependency footprint  
**Verification:** doc_inspection

---

### Release and Distribution (one_oh)

**Design areas:** release-and-distribution.md, scope-and-requirements.md  
**BAR-FERRIC:** BAR-023, BAR-024, BAR-025, BAR-029  
**Depends on:** Docs and DX, Performance and Benchmark

#### Publishing readiness: crates.io metadata, licensing, and workspace versioning

**Acceptance criteria:** All three crates (`ferric`, `ferric-router`, `ferric-http`) publish cleanly to crates.io from `cargo publish` with no git or path dependencies on Ferric required to build a consuming project; each crate's `Cargo.toml` carries a `license` field with an SPDX identifier, a `description`, a `repository` URL, and a `rust-version` field matching the workspace MSRV; intra-workspace dependencies are declared at a semver version, not a path; the crates are publishable in dependency-graph order; `cargo add ferric` resolves published crates at a semver version from a fresh project. Traces to: release-and-distribution.md §publish-readiness (REQ-REL-001, REQ-REL-002, REQ-REL-003, REQ-REL-004, REQ-REL-005, REQ-REL-006, REQ-REL-007, REQ-REL-008, REQ-REL-009, REQ-REL-010, REQ-REL-011, REQ-FW-001); BAR-FERRIC-023.  
**Depends on:** Starter template, reference app, and new-user acceptance path; Peer comparison table and methodology  
**Verification:** publishability

#### Semver contract and cargo semver-checks CI enforcement

**Acceptance criteria:** The public API of each crate is identified; `cargo semver-checks` runs in CI and fails on violations of the semver contract before release; a change that removes or alters an existing public item requires a major version bump; additive changes require at least a minor bump across the workspace; a reviewer can read from the repository what version-level change a given kind of modification requires. Traces to: release-and-distribution.md §semver-contract (REQ-REL-020, REQ-REL-021, REQ-REL-022, REQ-REL-023, REQ-REL-024, REQ-REL-025, REQ-REL-026, REQ-REL-027, REQ-REL-028, REQ-REL-029, REQ-FW-010); BAR-FERRIC-024.  
**Depends on:** Publishing readiness  
**Verification:** automated_test

#### Changelog: Keep a Changelog 1.1.0

**Acceptance criteria:** `CHANGELOG.md` exists at the repository root, follows Keep a Changelog 1.1.0 format (sections: Added, Changed, Deprecated, Removed, Fixed, Security; unreleased section at top), and contains an entry for the 1.0 release naming each user-visible change; every breaking change in the 1.0 release is listed under Changed or Removed. Traces to: release-and-distribution.md §changelog (REQ-REL-030, REQ-REL-031, REQ-REL-032, REQ-REL-033); BAR-FERRIC-025.  
**Depends on:** Publishing readiness  
**Verification:** doc_inspection

#### Versioning and compatibility policy

**Acceptance criteria:** The three crates version as a lockstep unit and are published at the same version number; the MSRV is declared via `rust-version` in the workspace `Cargo.toml`; the `ferric` prelude public surface is frozen for the 1.x line (new items may be added; no existing exported item is removed or altered without a major bump); a written compatibility policy, discoverable from the repository root, states what the 1.x stability promise covers (the three preludes), what it does not cover (private modules, `#[doc(hidden)]` items), and the versioning rule for the multi-crate workspace. Traces to: release-and-distribution.md §compatibility-policy (REQ-REL-040, REQ-REL-041, REQ-REL-042, REQ-REL-043, REQ-REL-044, REQ-REL-045, REQ-REL-046, REQ-REL-047); BAR-FERRIC-029.  
**Depends on:** Publishing readiness  
**Verification:** doc_inspection

---

## Identity Verification

Checked before assembly:

- **J1 — epic ids consistent:** Every epic title referenced in work-items.md resolves to an epic in epics-and-order.md. No orphan work items. **PASSES.**
- **J2 — fold explicit:** epics-and-order.md names issues #5 and #6 in the epic table's "Issue fold" column and in the dedicated §Issues #5 and #6 fold section; work-items.md section headers repeat "extends #5" / "extends #6". The fold table above carries the exact issue numbers. **PASSES.**
- **J3 — BAR coverage:** work-items.md includes an explicit coverage table confirming all 17 `one_oh` BAR-FERRIC-* criteria are covered by ≥1 work-item acceptance criterion. No gaps. **PASSES.**

---

*Release plan version: 1.0 · Sources: epics-and-order.md, work-items.md · Repo target: design/ferric/release-plan.md via SUBTASK_design_pr*
