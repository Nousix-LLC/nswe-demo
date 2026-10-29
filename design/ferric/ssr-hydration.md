# Ferric — Server-Side Rendering & Hydration

This document designs the one must-have of the Ferric 1.0: **a server-rendered page becomes interactive
by hydration with no client-side re-render.** It owns the `REQ-SSR-*` register. It states the architecture
and its testable bar — what the framework must do and how a stranger checks it — not the implementation.
Each capability is deferred to a later code DAG, one work-item at a time.

**What this settles, and what it does not.** It settles the isomorphic-rendering contract: the shape of the
server render, the client adoption protocol, and the state that crosses between them, together, because the
no-re-render guarantee exists only in their agreement. It does not choose the internal data structures, the
marker byte format, or the `wasm-bindgen` glue; those belong to the implementing work-items. Where a
capability's mechanism is owned by a sibling area — the async-resource primitive (`REQ-ASYNC-*`), the
authoring macro (`REQ-API-*`), the publishing of the crates (`REQ-REL-*`) — this document states the SSR-side
requirement and names that area; cross-references to other registers are read-only (identity I1).

**Grounding.** Observations of "Ferric as it is today" are grounded in `Nousix-LLC/nswe-demo` at `276138f`
on `main`: the `ferric` crate (`signal`, `vdom`, `component`, `dom`), the frozen `ferric`/`ferric-router`/
`ferric-http` prelude surfaces, and the existing Axum server `chirp-server` (`build_router` + `axum::serve`,
in-memory repository, SPA static fallback). Peer-framework comparisons name the frameworks and rest on their
documented public behavior. Conforms to `../contracts/id_and_traceability_schema.yaml` and
`../contracts/_MANIFEST.yaml`.

---

## 1. The gap, and why the seam is already half-built

Today Ferric renders only in the browser. `dom::mount` takes a root element, **empties it**, builds the DOM
for the first render by creating every node from the view tree, and wraps the render in a reactive effect so
later signal changes diff and patch the live DOM. There is no way to serve a page that already contains the
rendered content, and `mount`'s first act — clearing the root and rebuilding it — is precisely the
client-side re-render a 1.0 must avoid. An app's first paint waits on the WASM bundle to download and run.
This is the defining 1.0 gap stated cross-cuttingly as `REQ-FW-004`, and it is the one capability an
evaluating author (persona A1) will reject the framework for.

Two facts about the baseline make the SSR seam cheaper than it looks, and the architecture is built on them:

1. **The view layer is already server-ready.** `vdom` and `component` perform no `web-sys` calls — the module
   docs state this and the crate proves it by exercising the full view tree and the keyed diff with ordinary
   native `cargo test`. A component's `render()` produces a `VNode` tree with no DOM contact. Turning that
   tree into an HTML string is therefore a pure function that compiles and runs **off-wasm**, on the server,
   with no browser and no `web-sys`. The server render needs no new rendering engine — it needs a second
   consumer of the `VNode` tree that the diff already consumes.
2. **The DOM is addressed by a path both sides can share.** The runtime resolves every patch by a `path` —
   the sequence of child indices from the single mounted root node (`resolve` walks `first_child()` then
   `child_nodes().item(i)`). The server emits DOM in that same child order. Hydration can walk the
   server-rendered DOM by the identical addressing to find the node that corresponds to each view node,
   without guessing.

The baseline also fixes the boundary of what can cross the seam. Reactive values live in a thread-local arena
as `Box<dyn Any>`, and event handlers are `Rc<dyn Fn()>` closures. Neither is serializable. The architecture
therefore does **not** attempt to serialize the reactive graph; it carries a declared, typed state snapshot
and rebuilds the graph on the client by re-running the render. That choice is examined against the
resumability alternative in §6.

---

## 2. The architecture: three facets of one mechanism

The guarantee "interactive by hydration with no client-side re-render" is a property of three things agreeing:
what the server emits (§2.1), how the client adopts it (§2.2), and the state that makes the client's first
render reproduce the server's (§2.3). None can be specified without the other two, so they are designed as one
contract.

### 2.1 Render to HTML on the server

Ferric gains a server render path — a function from a component (or its `VNode` tree) to an HTML string —
that lives in the renderable core and builds **off-wasm**. It walks the `VNode` tree the same way the diff
does: an element becomes its start tag with escaped attributes, its children in order, and its end tag; a text
node becomes its escaped text. Event bindings carried on the tree are **dropped** from the HTML — there are no
handlers in server markup; they are wired on the client at hydrate (§2.2). The render runs the component's
`render()` exactly once, against the request's initial state, and performs no effects and no DOM work.

**Text-boundary markers.** HTML collapses adjacent text and does not distinguish an empty text node from none.
A client walking by child index would mis-align on any dynamic or adjacent text. The server render therefore
emits unambiguous boundaries around the nodes the client must re-locate (the approach Leptos and Sycamore both
take with hydration markers). The required property, not the byte format, is the bar: **the node sequence the
client walks by path is exactly the sequence the server produced**, including empty and adjacent text. The
marker scheme is the implementing work-item's to choose; the contract is unambiguous re-location.

**Per-request isolation.** The reactive runtime is thread-local, so a server that renders concurrent requests
must run each render in its own reactive context with no shared signal state — one request's signals must
never be read or written by another. This is a correctness bar, not a nicety: a server that leaks state
between requests serves one user another user's page.

**The Axum integration shape.** The repository's server is Axum (`chirp-server`), and SSR composes into it as
a route handler that returns a complete HTML document: a shell (`<!doctype html>`, `<head>`, a mount container
such as `<div id="app">`) whose container holds the server-rendered markup, the serialized state island (§2.3)
as a `<script type="application/json">`, and the module script that loads the WASM bundle and calls `hydrate`.
Ferric provides the render and the document assembly as a library surface an Axum handler calls; it does not
impose a server framework (an author on another server calls the same render). The named, documented
integration is Axum, wired alongside `chirp-server`'s existing `build_router` so the deployed app serves
server HTML on the document route and the SPA assets as it does today.

### 2.2 Hydration without re-render

The client gains a `hydrate` entry point distinct from `mount`. Where `mount` empties the root and **creates**
every node, `hydrate` **adopts** the server-rendered DOM already under the root:

1. It runs the component's first render to obtain the `VNode` tree — the same render the server ran.
2. It walks the existing DOM by the shared path addressing (§1, fact 2), pairing each view node with the
   server node already present. It creates no element, replaces no node, and removes no node under the
   hydration root. The sole DOM mutation hydration performs is **attaching event listeners** to the adopted
   elements (§2.2, event wiring below).
3. It establishes the reactive effect that retains this first tree as the "previous" tree, so that every
   subsequent signal change diffs against it and patches the live DOM through the existing keyed-patch path —
   the identical update machinery `mount` uses after its first render.

The guarantee is stated as an observable: **between the server markup arriving and hydration completing, no
node under the hydration root is created, replaced, moved, or removed.** After hydrate, interaction works
immediately and later updates patch in place. The no-re-render property holds only if the first client render
is value-identical to the server render — which is the job of §2.3; the two are one requirement seen from two
sides.

**Event wiring on hydrate.** Server markup carries no handlers; hydration binds the handlers the first render's
`VNode` tree carries onto the adopted elements, using the runtime's existing event-binding path (which stashes
and can later detach a listener so a re-bind never double-fires). Today the framework binds `Click` and
`Input`; hydration wires whatever event kinds the view carries, so it extends with the full event set the
async/forms/events area adds (`REQ-ASYNC-*`) without a change here.

### 2.3 State carry and resume

For the client's first render to reproduce the server's markup node-for-node, the state the server rendered
from must reach the client. The server may have rendered from route parameters, from signals it seeded, and —
the case that matters most — from **async resources it resolved** (data fetched on the server so the first
paint shows real content, not a loading state).

The reactive arena is not serializable (§1), so the architecture carries a **declared, typed state snapshot**,
not the graph. The app's serializable state — chiefly resolved resource values and any server-seeded signal
values — is collected on the server into a keyed snapshot, serialized (JSON over the `chirp-types`-style
`serde` DTOs the app already uses), and embedded in the document as the state island. The client deserializes
the island **before** hydration and seeds the matching resources and signals, so the first client render reads
the same values the server read and produces the same tree. Entries are addressed by a **stable key** (a
resource key / hydration id) that is identical on the server and in the client build, so resume is
deterministic rather than positional.

**Resume of async resources.** A resource resolved on the server resumes on the client as **already resolved**:
hydration does not re-issue the server's fetch. The resource primitive itself — its loading/error states and
its `.read()` surface — is owned by `REQ-ASYNC-*`; what SSR owns and requires here is the seam contract: a
resource can be resolved on the server, serialized into the island, and resumed on the client as resolved,
with no second request for data the server already fetched.

---

## 3. Hydration mismatch and recovery

If, while adopting, the client finds a server node that does not correspond to the view node its path expects —
a different tag, different text, a missing or extra node — the server and client renders diverged. This is a
hydration mismatch, and it has two causes worth separating: a bug (non-deterministic render, unstable keys, or
state that failed to cross the seam), or markup altered between render and hydrate.

The architecture requires that mismatches are **detected and surfaced, and that the app recovers to a correct
DOM rather than running on a corrupted one.** Detection compares the expected tag/text at each path against the
adopted node during the walk. On a mismatch, the framework (in a development build) emits a diagnostic that
names the path and the divergence in terms the author can act on — which node, expected-versus-found — rather
than a generic failure; and it recovers by client-rendering the mismatched subtree (the `mount`-style
create/replace path, scoped to that subtree) so the live DOM becomes correct. A mismatch degrades that subtree
to a client render; it never leaves the page silently wrong. This keeps a diagnostics-quality bar (`REQ-FW-014`)
on the one path most likely to confuse an author.

---

## 4. Safety of the serialized boundary

The state island is untrusted-data-shaped markup the server writes into a page, so it is an injection surface,
and the HTML the server emits carries author and data values into markup. The architecture requires that:

- text content and attribute values in server markup are HTML-escaped, so a value containing `<`, `>`, `&`, or
  a quote cannot break out of its text or attribute context;
- the serialized state island is embedded so that its contents cannot terminate the surrounding script or
  element context (a value containing `</script>` or similar cannot close the island and inject markup), and
  the island is parsed as data, not executed.

These are stated as requirements because the state island and server markup are the one place in the SSR path
where app data becomes page markup, and an escaping gap there is an XSS hole in every app built on the
framework.

---

## 5. What is in the SSR bar, and what is deliberately not

**In the 1.0 SSR bar:** server render of the view tree to HTML; the Axum integration; hydration that adopts the
server DOM with no re-render; event wiring on adopt; state carry and resume of server-resolved resources;
first-render determinism; mismatch detection and recovery; safe escaping of markup and the state island. These
are the `REQ-SSR-*` register in §7, all `stage: one_oh`.

**Not in the SSR bar, and why (recorded, not dropped):**

- **Streaming / out-of-order SSR.** Sending the shell and streaming resource-resolved chunks as they complete
  (Leptos's streaming modes, React Suspense streaming) is a throughput refinement on top of the synchronous
  render-then-respond path. A credible 1.0 renders the resolved page and responds; streaming is a post-1.0
  enhancement, not a release blocker. (Recorded here as an SSR-area deferral; it is not one of the
  framework-level out-of-1.0 items in `REQ-FW-015..018`, and no `BAR-FERRIC-*` references it.)
- **Server functions / RPC.** Leptos-style server functions (call a server endpoint as a typed Rust function)
  are an application-data concern served by the existing HTTP/JSON client (`REQ-HTTP-*`) and the resource
  primitive (`REQ-ASYNC-*`); SSR does not need them to render and hydrate.
- **Islands / partial hydration.** Hydrating only marked subtrees and leaving the rest static (Qwik, Astro,
  Leptos islands) is a performance strategy layered on whole-document hydration; the 1.0 hydrates the app, and
  islands are a later optimization.

These SSR-area deferrals are named so the bar is drawn without a hidden cut. They sit outside the `REQ-SSR-*`
gating set below.

---

## 6. The chosen approach, compared to peers by name

Ferric's Rust peers all ship SSR + hydration, and that is the bar this design meets:

- **Leptos** — fine-grained signals, no VDOM; mature SSR with hydration markers, a `leptos_axum` integration,
  server functions, and streaming/islands modes. It is the SSR bar for a Rust signal framework, and this design
  mirrors its marker-based hydration and its Axum integration while leaving server functions and streaming out
  of the 1.0 (§5).
- **Sycamore** — fine-grained signals, no VDOM; `render_to_string` plus marker-based hydration. The closest in
  surface area to what Ferric needs: a string render and an adopt-the-DOM hydrate.
- **Dioxus** — React-like VDOM with an RSX macro; fullstack SSR and hydration that adopts a server-rendered
  tree. The closest to Ferric on the VDOM-adoption mechanic, since Ferric also has a VDOM to walk.
- **Yew** — React-like VDOM; a `ServerRenderer` for SSR with hydration added later. Confirms that VDOM
  hydration is a solved shape in the Rust ecosystem.
- **Solid** (JS analog) — the signal model Ferric's `signal` echoes; signals plus hydration.
- **Qwik** (JS) — **resumability** rather than hydration: it serializes the framework's state and the listener
  references into the HTML and resumes execution lazily, without re-running components on load. The contrast
  below is the one real architectural fork for Ferric.

**Hydration versus resumability — the fork, and the choice.** Resumability's appeal is that it skips the
re-run of component code on load. Its cost is that it must serialize the whole reactive graph and the event
wiring into the HTML and resume execution against it — which requires a runtime designed around that
serialization. Ferric's reactive runtime stores values as `Box<dyn Any>` and handlers as `Rc<dyn Fn()>`
closures (grounded at `276138f`); closures are not serializable, and the arena is not a serialization format.
Making Ferric resumable would mean replacing the reactive foundation, not extending it. **Hydration** — re-run
the render on the client, adopt the server DOM, and seed the carried state so the first render matches — is the
approach that fits the frozen reactive and VDOM foundation additively (new `render_to_string` and `hydrate`
surfaces, `mount` and the preludes unchanged). The trade-off this design accepts is plain: hydration re-runs
component functions once on load, which resumability avoids; in exchange, SSR lands on the existing framework
rather than a rewrite of its core. For a 1.0 whose must-have is a fast first paint and interactivity on adopt,
that is the right trade, and it matches what every Rust peer ships.

---

## 7. The `REQ-SSR-*` register

Each requirement states what must be true from the side of an app author (persona A2) or the author of the
build (A4). Acceptance criteria are written so a reviewer who never met the author can exercise the framework
and reach the same yes/no — a native `cargo test`, a headless `wasm-bindgen-test`, or a stated example build,
per the schema's verification vocabulary. All `REQ-SSR-*` are `stage: one_oh`: SSR is the 1.0 must-have and
nothing in it is an early-usable-floor capability.

### REQ-SSR-001 — Render a component tree to HTML on the server
- **Statement:** Ferric can render a component's view tree to an HTML string on the server, off-wasm, without
  a browser or a DOM.
- **Rationale:** A server-rendered first paint is the whole point of SSR, and the baseline view layer is
  already `web-sys`-free, so a string render is a pure second consumer of the `VNode` tree. Every Rust peer
  ships one (`render_to_string` in Sycamore, the renderer in Leptos/Yew).
- **Acceptance criteria:** A native test (`cargo test`, no wasm, no browser) renders a stated component to an
  HTML string whose elements, attributes, and text match the view tree, with event handlers absent from the
  markup; the render runs the component's `render()` once and touches no DOM.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-FW-004`; JA3.

### REQ-SSR-002 — The renderable core builds and runs off-wasm
- **Statement:** The view and reactive core (the `VNode` tree, components, signals/memos) compiles and runs on
  a native, non-wasm target; only the DOM runtime is browser-bound.
- **Rationale:** The server render (REQ-SSR-001) and its tests must build without `wasm32` and without
  `web-sys`. The baseline already separates `dom.rs` (browser) from `vdom`/`component`/`signal` (native-
  testable); the 1.0 must keep that separation so SSR stays a native capability.
- **Acceptance criteria:** `cargo build` and `cargo test` for the renderable core succeed on the host target
  with no `wasm32-unknown-unknown` target and no browser; a stated server crate depends on the render path and
  builds natively.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-FW-003`; `REQ-FW-004`.

### REQ-SSR-003 — Per-request reactive isolation on the server
- **Statement:** Concurrent server renders do not share reactive state; one request's signals are never read or
  written by another.
- **Rationale:** The reactive runtime is thread-local; a server that renders many requests must isolate each
  render's reactive context or it will serve one user state from another user's render — a correctness and a
  data-safety failure, not a performance detail.
- **Acceptance criteria:** A stated test renders two requests with different seeded state (concurrently, or in
  a way that would expose shared state) and each rendered output reflects only its own request's state; no
  value from one render appears in the other.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-FW-004`.

### REQ-SSR-004 — Hydrate the server-rendered DOM without a client re-render *(the gating must-have)*
- **Statement:** On load, the shipped WASM attaches to the server-rendered DOM and makes it interactive without
  re-rendering it: no node under the hydration root is created, replaced, moved, or removed during hydration;
  the only DOM mutation is event-listener attachment.
- **Rationale:** This is `REQ-FW-004`, the single defining 1.0 capability. A hydration that rebuilds the DOM is
  a client render with extra steps and defeats the server paint. "Interactive by hydration with no client
  re-render" is the release's must-have (feeds release-bar identity I4).
- **Acceptance criteria:** A headless `wasm-bindgen-test` loads a server-rendered DOM for a stated app and calls
  `hydrate`; an observation of the subtree (for example node-identity before/after, or a `MutationObserver`
  over the hydration root) records **zero** structural mutations (no create/replace/move/remove) during
  hydrate; after hydrate a stated interaction (e.g. a click) updates the DOM in place through the patch path.
- **Stage:** one_oh · **Release gating:** true · **the gating must-have (I4)**
- **References:** `REQ-FW-004`, `REQ-FW-008`; JA3.

### REQ-SSR-005 — Event handlers are wired on hydrate
- **Statement:** Hydration binds the event handlers carried on the first render's view tree onto the adopted
  server-rendered elements, so interaction works immediately after hydrate.
- **Rationale:** Server markup carries no handlers; without wiring on adopt, the page looks interactive but
  does nothing until a later render. The runtime's event-binding path already attaches and can detach a
  listener without double-firing; hydration reuses it rather than recreating nodes to get listeners.
- **Acceptance criteria:** A headless `wasm-bindgen-test` hydrates a stated app whose view binds a handler,
  dispatches the corresponding event to the adopted element, and observes the handler's effect (a signal
  change reflected in the DOM); the handler fires without the element having been recreated.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-SSR-004`; `REQ-ASYNC-*` (full event-kind coverage).

### REQ-SSR-006 — Carry server render state across the seam
- **Statement:** The serializable state the server rendered from is embedded in the response and read by the
  client before hydration, keyed stably, so the client can reproduce the server render.
- **Rationale:** The client's first render must match the server markup (REQ-SSR-008), which requires the
  server's state on the client. The reactive arena holds `Box<dyn Any>` and closures and is not serializable,
  so the architecture carries a declared, typed, keyed snapshot (JSON over the app's `serde` DTOs), not the
  graph.
- **Acceptance criteria:** A stated app serializes its server-side state into the response; a test deserializes
  that payload and reconstructs the same keyed values; entries are addressed by a stable key identical on
  server and client, not by position.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-SSR-008`; `REQ-ASYNC-*`.

### REQ-SSR-007 — Resume server-resolved async resources without refetch
- **Statement:** An async resource resolved on the server resumes on the client as already resolved; the client
  does not re-issue the server's fetch for data the server already provided.
- **Rationale:** The highest-value SSR case is data fetched on the server so the first paint shows real
  content. If the client refetches on hydrate, the user sees a flash or a redundant request and the server
  render was wasted. The resource primitive is owned by `REQ-ASYNC-*`; SSR requires the seam: resolve on
  server, serialize, resume as resolved.
- **Acceptance criteria:** A stated app renders with a resource resolved on the server; after hydration the
  client shows the resolved value with no observable loading state for it and issues no client-side request for
  that resource (verifiable by a stated fetch-count assertion or an example build that records requests).
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-SSR-006`; `REQ-ASYNC-*`; JA3.

### REQ-SSR-008 — First client render is value-identical to the server render
- **Statement:** Given the carried state (REQ-SSR-006), the client's first render of an app produces a view
  tree value-identical to the one the server rendered to HTML.
- **Rationale:** The no-re-render guarantee (REQ-SSR-004) holds only if the adopted DOM already matches what the
  client would render; this requirement is that precondition made checkable. It is also what makes a mismatch
  (§3) a detectable bug rather than normal behavior.
- **Acceptance criteria:** A native test renders a stated app to HTML on the server path and renders the same
  app with the same carried state on the client path, and the two view trees (or their serialized HTML) are
  equal; a stated non-deterministic render is shown to produce a mismatch, confirming the check has teeth.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-SSR-004`, `REQ-SSR-006`.

### REQ-SSR-009 — Hydration mismatches are detected and recovered safely
- **Statement:** When the adopted DOM does not match the view node its path expects, the framework detects the
  mismatch, surfaces an actionable diagnostic in a development build, and recovers by client-rendering the
  mismatched subtree so the live DOM becomes correct — never silently wrong.
- **Rationale:** Mismatches happen (non-deterministic renders, unstable keys, altered markup). A framework that
  adopts a diverged DOM blindly runs the whole app on corrupted state; one that only logs leaves the user with
  a broken page. Detection plus bounded recovery keeps the app correct and tells the author where the
  divergence is (a diagnostics-quality bar, `REQ-FW-014`).
- **Acceptance criteria:** A headless `wasm-bindgen-test` hydrates against a deliberately mismatched server DOM;
  the framework emits a diagnostic naming the path and the expected-versus-found divergence, and the resulting
  live DOM matches the client render of that subtree; the rest of the document remains adopted.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-SSR-004`, `REQ-SSR-008`; `REQ-FW-014`.

### REQ-SSR-010 — Server markup and the state island are injection-safe
- **Statement:** Text and attribute values in server markup are HTML-escaped, and the serialized state island is
  embedded so its contents cannot terminate or escape their script/element context and are parsed as data, not
  executed.
- **Rationale:** The server markup and the state island are the one place in the SSR path where app data becomes
  page markup; an escaping gap there is an XSS hole in every app built on Ferric. This is a framework-owned
  safety guarantee, not each app's problem to re-solve.
- **Acceptance criteria:** A native test renders values containing `<`, `>`, `&`, quotes, and a `</script>`
  sequence into text, attributes, and the state island, and confirms each is escaped/encoded so it cannot break
  out of its context; the state island round-trips as data.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-SSR-001`, `REQ-SSR-006`; `REQ-FW-014`.

### REQ-SSR-011 — A documented Axum server-integration path
- **Statement:** Ferric provides a documented way to serve a server-rendered Ferric app from an Axum server: a
  render-and-assemble library surface an Axum handler calls to produce a complete HTML document (mount
  container, the server markup, the state island, and the WASM-bootstrap script), composing with the existing
  `chirp-server` router rather than replacing it.
- **Rationale:** SSR is only real when it reaches a server; Axum is the repository's server and the named
  integration target (persona A4). Peers ship a server integration (Leptos's `leptos_axum`); Ferric's 1.0 must
  let an author wire the render into Axum from documented steps, not folklore.
- **Acceptance criteria:** A stated example (or the reference app) serves a server-rendered document over Axum
  whose first response contains the rendered markup and the state island, and which hydrates in the browser;
  the integration is a library call from an Axum handler, documented, and does not require abandoning
  `chirp-server`'s existing `build_router`/static-serving.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-FW-004`, `REQ-FW-008`; JA3, JA4; `REQ-DX-*` (the getting-started/server docs).

### REQ-SSR-012 — The SSR surface is additive over the frozen foundation
- **Statement:** The SSR capability is added as new public surface — a server render path and a `hydrate` entry
  point — without redefining `mount` or the frozen `ferric` prelude; an app written against the baseline
  surface still compiles and runs client-side.
- **Rationale:** The baseline froze the `ferric` prelude (including `mount`) and the reference app is written
  against it. SSR must extend, not break, that surface (`REQ-FW-009`); `hydrate` is a sibling of `mount`, not a
  change to it.
- **Acceptance criteria:** The baseline prelude items remain present and unchanged; `render_to_string` and
  `hydrate` are additive new items; a stated app using only the baseline `mount` path compiles and runs at 1.0
  with no change. Any surface change is additive or a semver-signalled, changelog-documented break.
- **Stage:** one_oh · **Release gating:** true
- **References:** `REQ-FW-009`; `REQ-REL-*` (semver/compat policy).

### Register summary

| id | stage | gating | must-have | enforced-by (bar area) |
|---|---|---|---|---|
| REQ-SSR-001 server render to HTML | one_oh | true | — | BAR-FERRIC (SSR) |
| REQ-SSR-002 renderable core off-wasm | one_oh | true | — | BAR-FERRIC (SSR) |
| REQ-SSR-003 per-request isolation | one_oh | true | — | BAR-FERRIC (SSR) |
| REQ-SSR-004 hydrate, no re-render | one_oh | true | **yes (I4)** | BAR-FERRIC (SSR) |
| REQ-SSR-005 event wiring on hydrate | one_oh | true | — | BAR-FERRIC (SSR) |
| REQ-SSR-006 state carry | one_oh | true | — | BAR-FERRIC (SSR) |
| REQ-SSR-007 resume resources resolved | one_oh | true | — | BAR-FERRIC (SSR) |
| REQ-SSR-008 first-render determinism | one_oh | true | — | BAR-FERRIC (SSR) |
| REQ-SSR-009 mismatch detect + recover | one_oh | true | — | BAR-FERRIC (SSR) |
| REQ-SSR-010 injection-safe markup | one_oh | true | — | BAR-FERRIC (SSR) |
| REQ-SSR-011 Axum integration | one_oh | true | — | BAR-FERRIC (SSR) |
| REQ-SSR-012 additive surface | one_oh | true | — | BAR-FERRIC (SSR) |

Twelve `REQ-SSR-*`, all `stage: one_oh`, all `release_gating: true`. REQ-SSR-004 is the gating must-have the
release bar marks (identity I4). No `REQ-SSR-*` is `post_one_oh`; the SSR-area deferrals in §5 (streaming,
server functions, islands) are recorded without an id and referenced by no criterion.

---

## 8. Traceability

- **Realizes the framework must-have.** This register is the mechanism behind `REQ-FW-004` (SSR + hydration,
  the 1.0 must-have) and feeds `REQ-FW-008` (the reference app exercises SSR end to end). It answers author
  journey JA3 (render on server → hydrate → carry state) and the A4 production path (JA4).
- **The gating must-have for the release bar.** REQ-SSR-004 is the one the consolidated bar marks as the
  gating 1.0 must-have (release-bar identity I4): *a server-rendered page becomes interactive by hydration with
  no client-side re-render.* The release-bar spoke references REQ-SSR-004 (and the SSR register) from a
  `one_oh` `BAR-FERRIC-*` carrying `must_have: true`.
- **Read-only cross-references (owned elsewhere, identity I1).** `REQ-ASYNC-*` owns the async-resource
  primitive that REQ-SSR-006/007 resume across the seam; `REQ-API-*` owns the authoring macro whose output is
  the `VNode` tree the server renders; `REQ-HTTP-*` owns the client the resources fetch through; `REQ-REL-*`
  owns the semver/compat policy REQ-SSR-012 defers to; `REQ-DX-*` owns the server-integration documentation
  REQ-SSR-011 is written up in. This document mints no id outside `REQ-SSR-*`.

---

## Metadata

- **Owning task:** `SUBTASK_ssr_hydration` (owns the `REQ-SSR-*` register; identity I1).
- **Repository target:** `design/ferric/ssr-hydration.md` (placed by `SUBTASK_design_pr`, identity I8).
- **Conforms to:** `../contracts/id_and_traceability_schema.yaml` (REQ fields, stage vocabulary,
  `release_gating`), `../contracts/_MANIFEST.yaml` (ownership, identity I1; feeds I4).
- **Grounding commit:** `Nousix-LLC/nswe-demo` @ `276138f` (`main`) — `crates/ferric` (`signal`, `vdom`,
  `component`, `dom`), the frozen `ferric`/`ferric-router`/`ferric-http` preludes, and the Axum `chirp-server`.
- **Peer evidence:** Leptos, Sycamore, Dioxus, Yew (Rust); Solid, Qwik (JS, resumability contrast) — compared
  by their documented public behavior.
- **Register summary:** `REQ-SSR-001`–`REQ-SSR-012`, all `stage: one_oh`, all `release_gating: true`;
  `REQ-SSR-004` is the gating must-have (I4). Cross-references to `REQ-FW-*`, `REQ-ASYNC-*`, `REQ-API-*`,
  `REQ-HTTP-*`, `REQ-REL-*`, `REQ-DX-*` are read-only.
</content>
</invoke>
