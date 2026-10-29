# Ferric 1.0 — Router and HTTP/JSON client (`REQ-RT-*` · `REQ-HTTP-*`)

*Composed by `SUBTASK_synthesis` from `SUBTASK_router/router.md` (owns `REQ-RT-*`) and
`SUBTASK_http/http.md` (owns `REQ-HTTP-*`). Conforms to
`../contracts/id_and_traceability_schema.yaml` and the sub-contract
`contracts/_MANIFEST.yaml`. Repo target: `design/ferric/router-and-http.md`.*

---

## Overview

The router and the HTTP/JSON client are the two surfaces a Ferric app author reaches for when
connecting a single-page application to the outside world. The router maps URL paths to views and
re-renders the active view on navigation (client-side, no page reload); the HTTP client issues typed,
JSON-encoded requests to a backend and returns — or errors on — the response. Together they cover the
cross-cutting `REQ-FW-007` surface: client routing with path parameters, nested routes, and in-app
links, plus a typed round-trip that treats a non-2xx status as an error.

Each capability builds on a crate the walking skeleton already shipped — `ferric-router` (board issue
#5, now closed) and `ferric-http` (board issue #6, now closed) — and each design is additive over that
frozen surface. The two designs are composed here into one document so their combined coverage of
`REQ-FW-007` can be read and verified as a unit, and so the release plan's backlog edits the two
existing board issues rather than duplicating them.

---

# Part 1: Client-Side Router (`REQ-RT-*`)

*Owns the `REQ-RT-*` register. Conforms to `../contracts/id_and_traceability_schema.yaml` and the
sub-contract `contracts/_MANIFEST.yaml` (`doc_layout`, identities SI1–SI6).*

## Overview

*(section: `overview`)*

This is the design for Ferric 1.0's **client-side router** — the capability an app author (persona A2,
"I'm building a real app on it") uses to move a single-page application between views, and that the
evaluating author (A1) expects any credible framework to provide. It specializes the framework-level
routing requirement `REQ-FW-007` (client routing with route parameters, nested routes, and in-app
links, with the view re-rendering on path change) into a testable `REQ-RT-*` register, and it is held
to the additive-over-frozen-surface rule `REQ-FW-009` (identity SI2 / parent I9).

The router exists today as a landed crate, not a blank page. Board issue #5 shipped
`crates/ferric-router` as a working walking skeleton: a declarative route table with path parameters,
History-API navigation, a reactive current path, a reactive outlet, and an in-app link — all composing
`ferric`'s signals and component model rather than re-implementing reactivity or the DOM diff. Its
`prelude` surface is frozen. The 1.0 design therefore does two things: it states the bar the
already-shipped capabilities must keep meeting, and it adds, **additively**, the two capabilities the
baseline does not yet have — **nested routes** (with a nested outlet) and **replace** semantics for
History navigation — plus one credible-1.0 affordance, **active-route awareness** for links.

**Who it is for.** The app author (A2) building a multi-view, data-backed SPA (the repository's
`chirp-frontend` is the first real one). They declare routes, drop one outlet into their shell, link
between views, and read the current route — and they expect the router, the HTTP client, and the view
framework to compose without mismatched assumptions (journey JA2, "Navigate" and "Compose the
pieces").

**Non-goals (stated so the cut is explicit, not silent).**

- **Route-level data loaders are not a router concern.** Peer routers bolt data loading onto routes;
  Ferric keeps the router navigation-only and composes data loading from the async surface
  (`REQ-ASYNC-*`, read-only cross-reference): an author fetches inside a route's view with a
  resource/suspense, so loading and error states live in one place rather than being split between the
  router and the async layer. This keeps the router's surface small and its single responsibility —
  resolve the path to a view and re-render on change — intact.
- **Query/search-string parameters** are a recorded post-1.0 deferral (`REQ-RT-009`), not a 1.0
  capability.
- **Typed/enum routes** (the Dioxus model) are deliberately not adopted; the router stays
  string-pattern based, matching the frozen `RoutePattern`. The rationale is in the peer comparison
  below.

---

## Relationship to the frozen `ferric-router` surface

*(section: `relationship_to_frozen`)*

The baseline `ferric-router` `prelude` is frozen (verified by reading `crates/ferric-router/src/` at
`276138f` and `contracts/ferric_router_api.rs`). The 1.0 design is additive over it: every item below
is carried forward with its signature unchanged, and the 1.0 additions are new items that do not remove
or redefine any of them (identity SI2 / parent I9).

### What exists and is carried forward unchanged

| Frozen prelude item | What it is | 1.0 disposition |
|---|---|---|
| `matcher::Params` (`get`, `is_empty`, `len`, and the additive `iter`) | Named path parameters extracted from a matched route (stringly-typed `get(name) -> Option<&str>`). | Carried forward unchanged. The bar it meets is `REQ-RT-001`. |
| `matcher::RoutePattern` (`new`, `match_path`) | A compiled `/`-separated pattern with `:name` segments; exact-over-segments matching; trailing slash insignificant; root is the empty segment list. | Carried forward unchanged (`REQ-RT-001`). |
| `router::Router` (`new`, `route`, `fallback`, `mount_history`, `outlet`) | A flat, first-match-wins route table consumed into a reactive outlet component. | Carried forward unchanged; the flat table becomes the degenerate (single-level) case of nested routing (`REQ-RT-005`). |
| `router::navigate(&str)` | Programmatic navigation: `pushState` + update the reactive current path. | Carried forward unchanged — it remains the **push** path (`REQ-RT-003`). |
| `router::current_path() -> String` | Reads the routed path reactively (subscribes the calling reactive context). | Carried forward unchanged (`REQ-RT-002`, `REQ-RT-003`). |
| `link::link(to, label) -> VNode` (plus the additive `Link` builder and `LINK_CLASS`) | A declarative in-app link emitting a focusable, `role="link"` non-anchor element wired to `navigate` (ferric's `Fn()` click handler cannot `preventDefault`, so an `<a href>` is intentionally avoided). | Carried forward unchanged (`REQ-RT-004`). |

### What the 1.0 adds (every addition flagged additive)

| Addition | Requirement | Why it is additive, not a redefinition |
|---|---|---|
| **Nested routes + a nested outlet** — parent routes render a layout containing a child outlet; the matched child view renders into it. | `REQ-RT-005` | The frozen flat `Router::route`/`outlet` keep working as the single-level case; nested routing is a new declaration form and a new child-outlet item layered on top of the frozen `RoutePattern`/`current_path`. No frozen signature changes. |
| **Replace-navigation** — a `replaceState` counterpart to the push-only `navigate`. | `REQ-RT-003` | `navigate(&str)` keeps its exact signature and push semantics; replace is a new additive API (its exact shape is left to implementation), not an in-place change to `navigate`. |
| **Active-route awareness for links** — a link reflects whether its target is the active route (for styling / `aria-current`). | `REQ-RT-007` | The frozen `link(to, label)` is unchanged; active-state is an additive extension of the `Link` builder. |
| **Per-level fallback for nested routes** — a nested level may declare its own no-match view. | `REQ-RT-006` | The frozen flat `Router::fallback` behavior is preserved; per-level fallback is defined only for the new nested form. |

Any change to a frozen item that could not be expressed additively would require a semver-signalled
break documented in the changelog (`REQ-FW-010`, release area, read-only here). This design introduces
none.

---

## Requirements register (`REQ-RT-*`)

*(section: `requirements_register`. Each requirement: id, statement (what, not how), rationale (peer
evidence), acceptance criteria (stranger-checkable, each naming a `verification_method` from
`../contracts/id_and_traceability_schema.yaml`: `automated_test` / `example_build` /
`doc_inspection`), stage, release gating, references. Identities SI1/SI6.)*

### REQ-RT-001 — Declarative route table with path parameters

- **Statement:** An author can declare a table of routes — each a `/`-separated path pattern with named
  `:param` segments — mapped to a view, and the router resolves the current path to the first matching
  route and hands that view the named parameters it extracted.
- **Rationale:** Path parameters are table stakes for any multi-view app; both peers provide them
  (`leptos_router` reactive `use_params_map()` over `:id`; Dioxus typed route fields). The baseline
  froze `RoutePattern`/`Params`/`Router::route`, so 1.0 states the bar they must keep meeting rather
  than reinventing them.
- **Acceptance criteria:**
  - A native `cargo test` resolves the pattern `/user/:id` against `/user/42` to that route's view and
    `Params::get("id")` returns `"42"`; a path with an extra or missing segment (`/user/42/extra`,
    `/user`) does not match. *(verification_method: automated_test)*
  - A stated multi-route example builds to WASM and, navigated to a parameterized path, renders the
    matched view showing the bound parameter. *(verification_method: example_build)*
- **Stage:** usable_early · **Release gating:** true
- **Relationship to frozen:** frozen (`RoutePattern`, `Params`, `Router::route`) carried forward
  unchanged.
- **References:** `REQ-FW-007` (read-only).

### REQ-RT-002 — A reactive outlet that re-renders on path change

- **Statement:** An author mounts one router outlet that renders the currently matched route's view and
  re-renders to the newly matched view whenever the current path changes, without the author wiring any
  subscription by hand.
- **Rationale:** Re-rendering the view on navigation is the defining job of a client router
  (`REQ-FW-007`: "the view re-rendering on path change"). `leptos_router` renders the matched node
  reactively; Ferric composes its own signal and render-effect for the same effect. The baseline froze
  `Router::outlet`/`current_path`.
- **Acceptance criteria:**
  - A native `cargo test` shows that after `navigate("/a")` then `navigate("/b")` the outlet renders
    the `/b` route's view (the outlet reads `current_path` reactively, so the render effect re-runs on
    the path write). *(verification_method: automated_test)*
  - In a stated example, activating an in-app link changes the displayed view with no full-page reload.
    *(verification_method: example_build)*
- **Stage:** usable_early · **Release gating:** true
- **Relationship to frozen:** frozen (`Router::outlet`, `current_path`) carried forward unchanged.
- **References:** `REQ-FW-007`, `REQ-RT-003`, `REQ-RT-004` (read-only).

### REQ-RT-003 — History-API navigation: push, replace, back/forward, reactive current path

- **Statement:** An author can navigate programmatically so as to either **push** a new history entry
  (the default) or **replace** the current one; browser back/forward are reflected in the routed view;
  and the current path is readable as reactive state.
- **Rationale:** The push/replace distinction is a standard router affordance (`leptos_router`'s
  `use_navigate` options; Dioxus's `Navigator`): a redirect after a form submit, or a step in a wizard,
  needs replace so the back button does not return the user to a stale or intermediate URL. The baseline
  froze `navigate` (push), `current_path`, and the `popstate` sync; replace is not yet expressible, so
  the 1.0 adds it **additively**.
- **Acceptance criteria:**
  - A native `cargo test` shows `navigate("/x")` updates `current_path()` to `/x`.
    *(verification_method: automated_test)*
  - A `wasm-bindgen-test` in a headless browser shows that push navigation grows the history length and
    back (`popstate`) restores the previous path into `current_path`, while replace navigation changes
    `current_path` without growing history length. *(verification_method: automated_test)*
  - The guide documents push versus replace and when an author should choose each.
    *(verification_method: doc_inspection)*
- **Stage:** usable_early · **Release gating:** true
- **Relationship to frozen:** **additive** — `navigate` (push) and `current_path` carried forward with
  unchanged signatures; replace-navigation is a new additive API (its exact shape left to
  implementation), not an in-place change to `navigate`.
- **References:** `REQ-FW-007` (read-only).

### REQ-RT-004 — In-app links that navigate client-side

- **Statement:** An author can place a declarative in-app link that, when activated, navigates
  client-side (no full page reload) to its target, is keyboard-focusable, and is announced as a link
  by assistive technology.
- **Rationale:** Links are how a user moves through an SPA, and being focusable and announced is part
  of a link an author can ship, not an extra. The baseline froze `link(to, label)` and the additive
  `Link` builder; because ferric's click handler is a zero-argument `Fn()` that cannot call
  `preventDefault()`, the link is a focusable `role="link"` non-anchor element wired to `navigate`
  rather than an `<a href>` — a deliberate composition choice, recorded so a reader does not read the
  missing `<a>` as a defect.
- **Acceptance criteria:**
  - A native `cargo test` shows `link("/about", "About")` yields a non-anchor, focusable
    (`tabindex="0"`), `role="link"` element that carries exactly one click handler and records its
    destination. *(verification_method: automated_test)*
  - In a stated example, clicking a `link` changes the view client-side with no reload.
    *(verification_method: example_build)*
  - The guide states why the link is a non-anchor element (the `preventDefault` constraint) and lists
    its accessibility attributes. *(verification_method: doc_inspection)*
- **Stage:** usable_early · **Release gating:** true
- **Relationship to frozen:** frozen (`link`, `Link`, `LINK_CLASS`) carried forward unchanged.
- **References:** `REQ-FW-007`, `REQ-RT-002` (read-only).

### REQ-RT-005 — Nested routes with a nested outlet

- **Statement:** An author can compose routes hierarchically: a parent route renders a layout that
  contains a nested outlet, and the matched child route's view renders into that outlet — so a shared
  shell (navigation bar, section frame) is written once and only the child view swaps on navigation.
- **Rationale:** Nested routing is named directly in `REQ-FW-007`'s early-usable routing floor, and it
  is the one routing capability the baseline flat first-match table does not provide. Both peers center
  on it: `leptos_router` renders the matched child through an `<Outlet/>` (omit the outlet and the
  child does not show); Dioxus declares nesting with `#[nest]`/`#[layout]` and renders children through
  an `Outlet`. Without nested routing every view re-declares the whole shell, which does not scale past
  a toy.
- **Acceptance criteria:**
  - A native `cargo test` over a two-level configuration where `/users` renders a layout and
    `/users/:id` renders a child view into the parent's nested outlet shows that navigating `/users/7`
    renders the parent layout with the child view bound to `id = "7"`, and navigating `/users` renders
    the layout with its index/empty child. *(verification_method: automated_test)*
  - A stated example shows a persistent parent layout (for example a sidebar) with child content
    swapping on in-app link navigation, the parent layout not being torn down and rebuilt between child
    routes. *(verification_method: example_build)*
  - The guide documents declaring nested routes and placing the nested outlet.
    *(verification_method: doc_inspection)*
- **Stage:** usable_early · **Release gating:** true
- **Relationship to frozen:** **additive** — builds on the frozen `RoutePattern`/`Router`/`current_path`;
  the nested-route declaration and the child outlet are new items. The frozen flat
  `Router::route`/`outlet` remain valid as the single-level case.
- **References:** `REQ-FW-007`, `REQ-RT-001`, `REQ-RT-002` (read-only).

### REQ-RT-006 — No-match (fallback) handling at each routing level

- **Statement:** An author can declare a fallback (no-match / "404") view; when no route matches at a
  level, that level's fallback renders with empty parameters, and a nested level may declare its own
  fallback distinct from the app root's.
- **Rationale:** Every real app needs a defined answer for an unmatched URL. The baseline froze
  `Router::fallback` for the flat table (no match with no fallback renders an empty node); nested
  routing needs a per-level answer so that a miss within a section (`/users/<unknown>`) can render the
  section's own not-found rather than always falling through to the app root.
- **Acceptance criteria:**
  - A native `cargo test` shows that with a fallback declared, an unmatched path renders the fallback
    view invoked with empty parameters; with no fallback declared, an empty node renders (the baseline
    behavior is preserved). *(verification_method: automated_test)*
  - A native `cargo test` shows a nested level with its own fallback renders that fallback for an
    unmatched child while the parent layout still renders. *(verification_method: automated_test)*
  - The guide documents fallback resolution, including at nested levels.
    *(verification_method: doc_inspection)*
- **Stage:** usable_early · **Release gating:** true
- **Relationship to frozen:** frozen `Router::fallback` behavior carried forward; **additive** per-level
  fallback defined for the new nested form.
- **References:** `REQ-RT-005` (read-only).

### REQ-RT-007 — Active-route awareness for links

- **Statement:** An author can render a link that reflects whether its target is the currently active
  route, so active navigation can be styled and announced (for example via an active CSS class and
  `aria-current`) without the author hand-wiring a comparison against `current_path`.
- **Rationale:** Marking the active link is a standard router affordance — `leptos_router`'s `<A/>`
  sets `aria-current="page"` on the active link — and apps otherwise re-derive it from `current_path`
  at every nav item. An app can ship without it (so it is not on the early-usable floor), but a
  credible 1.0 router provides it.
- **Acceptance criteria:**
  - A native `cargo test` shows that when the current path matches a link's target the emitted element
    carries the active marker (an active class and/or `aria-current="page"`), and when it does not
    match the marker is absent. *(verification_method: automated_test)*
  - A stated navigation example visibly marks the active link and updates the marker on navigation.
    *(verification_method: example_build)*
  - The guide shows how to style the active link. *(verification_method: doc_inspection)*
- **Stage:** one_oh · **Release gating:** true
- **Relationship to frozen:** **additive** — extends the `Link` builder with active-state over the
  frozen `link`/`Link`; `link(to, label)` is unchanged.
- **References:** `REQ-RT-004`, `REQ-RT-002` (read-only).

### REQ-RT-008 — Composition over the frozen surface, additive-only

- **Statement:** The 1.0 router composes `ferric`'s signals and component model (it does not
  re-implement reactivity or the DOM diff) and extends the baseline `ferric-router` prelude
  additively: every baseline prelude item (`Params`, `RoutePattern`, `Router`, `navigate`,
  `current_path`, `link`) remains usable with an unchanged signature, and an app written against the
  baseline prelude still compiles and runs at 1.0.
- **Rationale:** This specializes `REQ-FW-009` and realizes identity SI2 (parent I9). The walking
  skeleton is a dependable base only while its evolution is additive; the shipped example and the
  future `chirp-frontend` depend on the frozen surface.
- **Acceptance criteria:**
  - A native `cargo test` / `example_build` shows the baseline 2–3 route example (built with
    `Router::new().route(...).fallback(...).mount_history().outlet()`, `navigate`, `current_path`, and
    `link`) compiles and its tests pass unchanged against the 1.0 crate.
    *(verification_method: automated_test)*
  - A reviewer confirms every 1.0 addition to the prelude is recorded as additive in the changelog and
    no baseline prelude item is removed or has its signature changed without a semver-signalled,
    changelog-documented break. *(verification_method: doc_inspection)*
- **Stage:** usable_early · **Release gating:** true
- **Relationship to frozen:** this requirement *is* the additive identity (SI2) stated as a testable
  condition.
- **References:** `REQ-FW-009`, `REQ-HTTP-*` (sibling spoke, read-only).

### REQ-RT-009 — Query / search-string parameters as reactive state (deferred)

- **Statement (deferred):** A router-owned reactive accessor for URL query/search parameters
  (`?q=...`) is **not** part of 1.0.
- **Reason:** The baseline matcher is path-only, and `REQ-FW-007` and the author journeys do not
  require router-owned query state for a credible 1.0. An app that needs the query string can read
  `window.location.search` or layer its own parsing until this is picked up. Recorded so the cut is
  explicit, not silent; `leptos_router`'s `use_query_map` is the peer precedent should this be
  designed post-1.0.
- **Verification:** none (post-1.0 item; referenced by no `BAR-FERRIC-*`, per trace rule "no
  out-of-1.0 item is enforced by any BAR").
- **Stage:** post_one_oh · **Release gating:** false
- **References:** `REQ-ASYNC-*` (read-only; query state, if added, would compose with the async
  surface rather than own data loading).

### Register summary

| REQ-RT | Capability | Stage | Release gating | Frozen / additive |
|---|---|---|---|---|
| 001 | Route table with path parameters | usable_early | true | frozen |
| 002 | Reactive outlet re-renders on path change | usable_early | true | frozen |
| 003 | History navigation: push, **replace**, back/forward, reactive path | usable_early | true | additive (replace) |
| 004 | In-app client-side links (focusable, announced) | usable_early | true | frozen |
| 005 | **Nested routes** with a nested outlet | usable_early | true | additive |
| 006 | No-match / fallback at each level | usable_early | true | frozen + additive (nested) |
| 007 | Active-route awareness for links | one_oh | true | additive |
| 008 | Additive composition over the frozen surface | usable_early | true | identity (SI2) |
| 009 | Query/search params (deferred) | post_one_oh | false | — |

`REQ-RT-001`–`REQ-RT-008` are 1.0 requirements (`release_gating: true`); `REQ-RT-009` is a recorded
post-1.0 deferral. Every gating requirement carries at least one acceptance criterion a stranger could
check with a `verification_method` from the parent schema (SI6). This register writes only `REQ-RT-*`
ids (SI1); all `REQ-FW-*`, `REQ-ASYNC-*`, and `REQ-HTTP-*` references are read-only.

---

## Peer comparison

*(section: `peer_comparison`. Named comparison of routing models, per the sub-contract. Peer behavior
is drawn from each project's current public documentation.)*

Ferric's routing model is a **string-pattern route table of closures** (`Router::new().route("/user/:id",
|p| …)`) with stringly-typed parameters (`Params::get("id") -> Option<&str>`), first-match-wins
resolution, a single reactive current path, and one outlet component. The two Rust peers an evaluating
author (A1) weighs take different stances; the 1.0 nested-route design follows the one closest to
Ferric's existing shape.

### `leptos_router`

- **Nested routes / outlet:** a URL matches a nested tree of `<Route>`s; the matched child renders
  through an `<Outlet/>` placed in the parent's view (omit the `<Outlet/>` and the child is not
  shown). This is the model Ferric's `REQ-RT-005` follows — a parent layout with a nested outlet into
  which the matched child renders.
- **Path parameters:** `:id` dynamic segments read **reactively** with `use_params_map()` /
  `use_params` inside the matched component.
- **Links:** an `<A/>` component that resolves relative nested routes and sets `aria-current="page"`
  on the link to the active page — the precedent for Ferric's `REQ-RT-007` active-route awareness.
- **Query:** reactive `use_query_map()` (the precedent cited by the deferred `REQ-RT-009`).
- **Closest to Ferric on:** the signal-reactive model and the outlet-based nested-routing shape.
  Ferric's current parameters are read by value (`Params::get`) from the view builder's `&Params`
  argument rather than through a reactive hook; the 1.0 keeps that shape (the matched view is already
  re-run reactively by the outlet), which is the main model difference to note for an author coming
  from Leptos.

### Dioxus router

- **Nested routes / layouts:** declared on an enum via `#[derive(Routable)]` with `#[route]`,
  `#[nest]` / `#[end_nest]`, `#[layout]` / `#[end_layout]`, `#[redirect]`, `#[child]`; a `#[layout]`
  component wraps child routes, which render through an `Outlet`.
- **Type-safe routing:** routes are **enum variants** and parameters are **typed struct fields**; the
  macro turns the enum into a parser/renderer. Links are typed — `Link { to: Route }` — so a
  destination that does not exist is a compile error, and programmatic navigation goes through a
  `Navigator` (`use_navigator()`).
- **Closest to Ferric on:** the outlet-based nested rendering (Ferric's `REQ-RT-005` matches the
  `Outlet` concept). It is **further from Ferric** on the authoring model: Dioxus's typed-enum routes
  and compile-checked links are a macro-heavy approach that would pull routing into the
  authoring-macro design (`REQ-API-*`), and they presuppose a route-enum abstraction the frozen
  `RoutePattern` does not have.

### The design choice this comparison settles

Ferric 1.0 keeps the **string-pattern, closure-table** model (additive over the frozen
`RoutePattern`/`Router`) and takes **nested routing via a nested outlet** from the leptos/Dioxus
`Outlet` model. It does **not** adopt Dioxus-style typed-enum routes: compile-time-checked
destinations are attractive, but they require a route-enum and macro that the frozen surface and this
release's scope do not include, and retrofitting them would redefine the frozen `Router`/`RoutePattern`
rather than extend them — which `REQ-FW-009` / SI2 forbid. Typed routes are therefore left as a
possible future direction, not a 1.0 capability. The cost of this choice is concrete: Ferric's links
and route references stay string-based, so a mistyped path is a runtime no-match (handled by
`REQ-RT-006`), not a compile error.

---

## Board fold — issue #5

*(section: `board_fold`. How board issue #5 is superseded/extended by this design — identity SI3,
feeds parent I7. So the release plan and backlog filing edit #5 rather than duplicate it.)*

**Issue #5 — "ferric-router: client-side router (History API)" (epic #1, milestone "M1: Framework
(ferric)") is CLOSED.** It was delivered as the walking-skeleton crate `crates/ferric-router`: a
declarative route table with path parameters, a path→route matcher with named-parameter extraction,
History-API navigation (`pushState`/`popstate`), programmatic `navigate`, an in-app link, and a
reactive outlet, unit-tested with fmt/clippy clean and shipped as a PR into `main`. Its scope is
realized and should be treated as the **delivered baseline**, not reopened.

**The fold:** this 1.0 design **extends** #5; it does not duplicate or supersede it. The capabilities
#5 shipped map to `REQ-RT-001`, `REQ-RT-002`, `REQ-RT-004`, `REQ-RT-006`, and the push/back-forward
half of `REQ-RT-003` — stated here as the bar the baseline must keep meeting (carried forward
unchanged). What this design adds on top of #5 is new, unshipped work: **nested routes with a nested
outlet** (`REQ-RT-005`), **replace-navigation** (the additive half of `REQ-RT-003`), **active-route
awareness** (`REQ-RT-007`), and **per-level nested fallback** (the additive half of `REQ-RT-006`).

**Instruction to the release plan and backlog (read-only cross-references; the filing itself is owned
by `SUBTASK_release_plan` / `SUBTASK_file_backlog`):**

- Do **not** file a new issue that re-states #5's shipped scope. #5 is closed and its baseline stands.
- File the 1.0 router work as **new work-item(s) under epic #1** that **reference #5 as the superseded
  baseline** and cover only the additive capabilities above (`REQ-RT-005`, the replace half of
  `REQ-RT-003`, `REQ-RT-007`, the nested-fallback half of `REQ-RT-006`), each tracing to its
  `REQ-RT-*` id and to this design section.
- Per the id schema's epic rule, an epic that carries this work records `supersedes: #5`
  (extend/supersede, never duplicate — identity I7), signaling that #5 is folded in as the baseline
  the new work builds on.

---

# Part 2: HTTP/JSON Client (`REQ-HTTP-*`)

*Owns the `REQ-HTTP-*` register. Conforms to `../contracts/id_and_traceability_schema.yaml` and the
sub-contract `contracts/_MANIFEST.yaml`. Grounded in `Nousix-LLC/nswe-demo` at `276138f`. This is
design — what the client must do and the bar it is held to — not how the fetch path is coded.*

## Overview

*(section: `overview`)*

Ferric's HTTP/JSON client is the capability an application author (persona A2) reaches for when a
Ferric single-page app needs to read from and write to a JSON/REST backend — the `chirp-frontend`
talking to `chirp-server` is the first real case. The author configures a client once (a base URL and
the headers every request carries, such as `Authorization`), then for each call picks a verb, a path,
query parameters and a typed request body, and `await`s a typed response. Request and response bodies
are ordinary `serde` types, so the author works in their own structs rather than in JSON strings. Every
failure — a request that could not be built, a network call that never reached the server, a non-2xx
status, or a body that would not (de)serialize — arrives as one matchable `Error`, so the calling code
can branch on *why* a call failed.

The evaluating author (persona A1) weighs this client against `reqwest` and `gloo-net` when deciding
whether Ferric can carry a real data-backed app. The client is built on the browser Fetch API and keeps
a lean dependency set so the WASM bundle stays small; it does not depend on the `ferric` view crate, so
it is usable on its own and composes with the view framework and the async/resource layer
(`REQ-ASYNC-*`) rather than assuming them. This client realizes the HTTP half of the cross-cutting
`REQ-FW-007` (client routing and an HTTP/JSON client) and is one of the capabilities the `usable_early`
floor depends on: an author cannot build a real app without being able to fetch and post typed data with
real error states.

The companion crate `ferric-http` already exists at the baseline with a frozen public surface and an
implemented, natively-tested core. This design ratifies that surface as the 1.0 baseline, states the
testable bar the client is held to, and names the small additions a credible 1.0 adds on top — each
addition flagged additive so no existing item is silently redefined (identity SI2 / parent I9).

---

## Relationship to the frozen `ferric-http` surface

*(section: `relationship_to_frozen`; realizes SI2 / parent I9 — every change below is additive)*

### What exists at the baseline (`276138f`), read from the crate

`ferric-http`'s public surface is its `prelude` plus the crate-root re-exports. Confirmed by reading
`crates/ferric-http/src/{lib,client,request,response,error}.rs` — not the summary alone. The frozen
exported items are:

| Exported item | What it is at the baseline |
|---|---|
| `Client` | Holds an optional base URL + default headers; `Clone`, `Default`, holds no connection state. Verb entries `get`/`post`/`put`/`delete(path)` and a generic `request(Method, path)`; each returns a `RequestBuilder`. `new()`, `builder()`, `base_url()`. |
| `ClientBuilder` | `base_url(impl Into<String>)`, `default_header(name, value)`, `build()`. |
| `Method` | Enum `Get`/`Post`/`Put`/`Delete`; `as_str()`, `Display`. **Not** `#[non_exhaustive]` at the baseline. |
| `RequestBuilder` | Fluent `header`, `query(&[(&str,&str)])`, `json<B: Serialize>(&B) -> Result<Self>`, `body_string`, `url() -> String`; terminals `send() -> Result<Response>` and `send_json<T: DeserializeOwned>() -> Result<T>`. |
| `Response` | Owned data: `status`, `status_text`, `url`, `ok`, `headers`, case-insensitive `header(name)`, `body`, `text`, `json<T>()`, `error_for_status()` — all synchronous. |
| `Error` | `#[non_exhaustive]` enum: `Builder(String)`, `Network(String)`, `Status { status, status_text, url, body }`, `Serialize(serde_json::Error)`, `Deserialize(serde_json::Error)`, `Body(String)`; `is_network()`, `is_status()`, `status() -> Option<u16>`, `Display`, `std::error::Error` with `source`. |
| `Result<T>` | `std::result::Result<T, Error>`. |

The pure logic (URL base/path joining, RFC-3986 query percent-encoding, the `Response` accessors, serde
round-trips, and error mapping/`Display`) is implemented and covered by native `cargo test`; the Fetch
call in `RequestBuilder::send` is implemented against `web-sys` and exercised by a browser-gated
`wasm-bindgen-test` smoke test. The surface is a real, usable baseline, not a `todo!()` shell. Earlier
framing of the companion crates as "`todo!()` skeletons" is correct for some leaves elsewhere but not
for `ferric-http`'s core, which this design treats as the ratified `usable_early` client.

### What the 1.0 adds, each additive

The 1.0 keeps every item above usable and unchanged (`REQ-HTTP-013` below), and adds only:

1. **Broader method coverage (`REQ-HTTP-009`).** Add `Method::Patch` and a `Client::patch(path)` verb
   entry. Because `Method` is not `#[non_exhaustive]` at the baseline, adding a variant would break a
   downstream exhaustive `match`; the additive-safe move is to mark `Method` `#[non_exhaustive]` as
   part of 1.0, then add `Patch` under it. The baseline variants and `as_str()`/`Display` are
   unchanged; the `#[non_exhaustive]` attribute is a new, semver-recorded surface fact, not a
   redefinition of an existing variant.
2. **Per-request timeout / cancellation (`REQ-HTTP-010`).** Add a `RequestBuilder::timeout(..)` (and/or
   an abort handle) and a new matchable `Error` variant for a timed-out/aborted request. The `Error`
   enum is already `#[non_exhaustive]` at the baseline, so a new variant is additive by construction.
   No existing variant changes.

No existing exported item is removed or redefined. If a future need forces a true break, it is
signalled by semver and recorded in the changelog (`REQ-REL-*`, read-only here), never applied
silently — this is the acceptance condition of `REQ-FW-009` restated for this crate.

---

## Requirements register (`REQ-HTTP-*`)

*(section: `requirements_register`. Each item: statement (what, not how), rationale (peer evidence
where it bears), acceptance criteria checkable by a stranger via a `verification_method` from the
parent schema — `automated_test` / `example_build` / `doc_inspection` — a `stage`, and
`release_gating`. Cross-references to other registers are read-only. SI1: only `REQ-HTTP-*` ids are
defined here.)*

### A. Request building

#### REQ-HTTP-001 — One configured, shareable client
- **Statement:** An author configures a single `Client` with an optional base URL and a set of default
  headers and shares it across the whole app; the client is cheap to clone and carries no connection
  state of its own.
- **Rationale:** A SPA against one backend sets the base URL and the auth header once, not on every
  call. `reqwest::Client` is the shareable, clone-cheap connection holder authors expect; `gloo-net`
  has no client object at all, so base URL and headers are repeated per request — a gap this client
  closes. Ferric's client owns no connection (the browser's Fetch layer does), so sharing is just
  cloning a string and a small vector.
- **Acceptance criteria:** A native `cargo test` builds a client with `ClientBuilder::base_url(..)` and
  two `default_header(..)` calls and asserts (a) the resolved URL of an issued request includes the
  base URL and (b) the default headers are present on the produced `RequestBuilder`; a second test
  asserts a cloned client resolves identically to its source.
- **verification_method:** automated_test · **stage:** usable_early · **release_gating:** true
- **References:** REQ-FW-007, REQ-FW-009. *Satisfied by the frozen surface; carried forward additively.*

#### REQ-HTTP-002 — Fluent per-request construction
- **Statement:** For each request an author selects a method (a verb helper, or `Method` via the
  generic `request` entry), a path, appends query parameters, adds headers, and optionally sets a body,
  through a chainable `RequestBuilder` — without assembling a URL or header string by hand.
- **Rationale:** Builder ergonomics are the shared idiom of `reqwest` and `gloo-net`; hand-built URLs
  and header lists are where missing encoding and duplicated `?` separators hide. Keeping construction
  fluent and inspectable moves those bugs into tests rather than into runtime 400s.
- **Acceptance criteria:** A native `cargo test` asserts that two successive `.query(..)` calls
  accumulate and produce the exact resolved URL, that `.header(..)` values are observable on the
  builder, and that `.url()` returns the fully resolved URL *without* performing the request.
- **verification_method:** automated_test · **stage:** usable_early · **release_gating:** true
- **References:** REQ-FW-007. *Satisfied by the frozen surface; carried forward additively.*

#### REQ-HTTP-003 — Correct URL resolution and RFC-3986 query encoding, pure and native-testable
- **Statement:** Base and path join with exactly one `/` separator; an absolute request path
  (`http(s)://…`) overrides the base; query keys and values are percent-encoded per RFC 3986; and this
  resolution logic is pure, so it is testable without a browser.
- **Rationale:** URL correctness is a frequent, silent source of backend errors, and a WASM client that
  needs a headless browser to test its URL building is hard to trust. Making resolution a pure function
  the native test suite covers is how the client earns that trust cheaply.
- **Acceptance criteria:** `cargo test -p ferric-http` passes resolution cases covering base+`/path`
  and base+`path`, both trailing-slash variants, absolute-path override of the base,
  empty-path-returns-base, the `?`-vs-`&` separator choice when the path already carries a query, and
  `%XX` uppercase encoding including a multi-byte UTF-8 value (`é` → `%C3%A9`).
- **verification_method:** automated_test · **stage:** usable_early · **release_gating:** true
- **References:** REQ-FW-007. *Satisfied by the frozen surface; carried forward additively.*

### B. Typed JSON fetch

#### REQ-HTTP-004 — Typed JSON request body with a defaulted Content-Type
- **Statement:** An author sets a request body by passing any `serde::Serialize` value; the client
  serializes it to JSON and defaults `Content-Type: application/json` unless the author set a
  Content-Type explicitly.
- **Rationale:** Typed request bodies are the reason to use a JSON client rather than raw Fetch;
  `reqwest`'s `.json()` is the model. Defaulting Content-Type without clobbering an author-set value
  removes a common cause of a backend 415.
- **Acceptance criteria:** A native test asserts `.json(&T)` encodes the expected JSON string and adds
  exactly one `Content-Type: application/json`; a second test with a prior explicit `Content-Type`
  asserts that value is preserved and not duplicated; a serialization failure is returned as
  `Error::Serialize` (a `Result`), never a panic.
- **verification_method:** automated_test · **stage:** usable_early · **release_gating:** true
- **References:** REQ-FW-007. *Satisfied by the frozen surface; carried forward additively.*

#### REQ-HTTP-005 — Safe-by-default typed round-trip, with a raw escape hatch
- **Statement:** `send_json::<T>()` performs the request and returns a deserialized `T`, treating a
  non-2xx status as an error; a lower-level `send()` returns the raw `Response` so an author who wants
  to read a non-2xx body, or stream their own handling, can.
- **Rationale:** The dominant case — typed in, typed out, error on failure — should be the shortest
  path, with an explicit escape hatch for the rest. This two-path design is what separates a considered
  client from `gloo-net`'s thinner single surface, where status handling is left to the caller every
  time.
- **Acceptance criteria:** A headless `wasm-bindgen-test` performs a typed round-trip against an
  echo/stub endpoint and asserts the deserialized value; it then asserts `send()` on a 404 returns a
  `Response` without erroring while `send_json()` on the same request yields `Error::Status`. Where no
  browser runner is available, the native suite covers the `Response` → `error_for_status` → `json`
  composition that `send_json` is defined as, so the behavior is gated by `cargo test` even without a
  browser.
- **verification_method:** automated_test · **stage:** usable_early · **release_gating:** true
- **References:** REQ-FW-007. *Satisfied by the frozen surface; carried forward additively.*

#### REQ-HTTP-006 — Response as owned, synchronously-inspectable data
- **Statement:** A completed `Response` exposes the status code, status text, final URL (after
  redirects), headers with case-insensitive lookup, and the body as owned data, with synchronous
  `json::<T>()`, `text()`, `ok()`, and `error_for_status()`.
- **Rationale:** Reading the body eagerly and owning it makes every accessor synchronous and
  native-testable; `reqwest`'s response bodies are async, and Ferric trades that for testability and a
  simpler surface suited to the browser, where the body is already in memory after the Fetch resolves.
- **Acceptance criteria:** Native tests assert `ok()` across the 2xx boundary (199/200/204/299/300), a
  case-insensitive header lookup, a `json()` round-trip plus `Error::Deserialize` on a malformed body,
  and `error_for_status()` mapping a 404 to `Error::Status` carrying the code, URL, and body.
- **verification_method:** automated_test · **stage:** usable_early · **release_gating:** true
- **References:** REQ-FW-007. *Satisfied by the frozen surface; carried forward additively.*

### C. Error model

#### REQ-HTTP-007 — One matchable error type over every failure mode
- **Statement:** Every fallible operation returns `Result<T, Error>`, where `Error` distinguishes
  request-build failure, network failure (no response received), a non-2xx status (carrying code,
  status text, final URL, and body), request-body serialization failure, response-body deserialization
  failure, and body-read failure; no fallible path panics.
- **Rationale:** An app branches on *why* a call failed — retry a network blip, show a 404 to the
  user, log a 500 — so the failure modes must be distinct and matchable. A single opaque error forces
  string-matching on messages. `reqwest`'s `Error` with its `is_*` predicates and `status()` accessor
  is the model this follows, kept lean (no `thiserror`) to hold the dependency footprint.
- **Acceptance criteria:** A native test constructs each variant and asserts a single-line `Display`,
  the `is_network()` / `is_status()` / `status()` classifiers, and that the serialize/deserialize
  variants expose their `serde_json::Error` through `std::error::Error::source`. A `doc_inspection` of
  the public surface confirms every fallible method returns `Result` (no `unwrap`/`expect`/panic on
  the fallible path).
- **verification_method:** automated_test · **stage:** usable_early · **release_gating:** true
- **References:** REQ-FW-007.

#### REQ-HTTP-008 — Non-2xx is an error on the safe path
- **Statement:** On the typed path a non-2xx response is surfaced as `Error::Status` carrying the
  status code, status text, final URL, and body — not returned as though it were a successful value.
- **Rationale:** Deserializing a 500's error body as if it were the expected `T`, or returning it as
  success, is the classic JSON-client bug. The `REQ-FW-007` acceptance bar names "handles a non-2xx
  status as an error" explicitly, and this requirement is where that bar is met for the client half.
- **Acceptance criteria:** A test asserts `send_json()` and `error_for_status()` on a 404 and on a 500
  each yield `Error::Status` with the correct code and the response body preserved, while `send()` on
  the same requests returns the `Response` unchanged for the caller to inspect.
- **verification_method:** automated_test · **stage:** usable_early · **release_gating:** true
- **References:** REQ-FW-007.

### D. Additions a credible 1.0 adds (flagged additive — SI2)

#### REQ-HTTP-009 — Broader HTTP method coverage, added additively
- **Statement:** Beyond GET/POST/PUT/DELETE the client can issue PATCH, via a `Method::Patch` variant
  and a `Client::patch(path)` verb entry; the `Method` enum is carried forward so that adding methods
  does not break exhaustive matches in downstream code.
- **Rationale:** PATCH is standard REST for a partial update, and a backend like `chirp-server` uses
  it; a 1.0 client that cannot PATCH pushes authors to the generic `request` entry or a workaround.
  Adding a variant to a plain enum is a breaking change, so the additive-safe path is to mark `Method`
  `#[non_exhaustive]` as part of 1.0 and add `Patch` under it — a surface change recorded by semver,
  not a redefinition of an existing variant.
- **Acceptance criteria:** `Method::Patch` exists, `client.patch(path)` issues a request whose method
  token is `PATCH`, and `Method` is `#[non_exhaustive]`; a `trybuild`/build check confirms code
  written against the baseline variants still compiles; a `doc_inspection` confirms the changelog
  records the `#[non_exhaustive]` addition as the semver-signalled surface change.
- **verification_method:** automated_test · **stage:** one_oh · **release_gating:** true
- **References:** REQ-FW-009; REQ-REL-* (semver/changelog, read-only). *Additive over the frozen
  `Method`.*

#### REQ-HTTP-010 — Per-request timeout and cancellation
- **Statement:** An author can bound a request with a timeout and/or cancel it through an abort handle,
  so a hung or slow backend does not hang the app; a timed-out or aborted request surfaces as a
  distinct, matchable `Error`.
- **Rationale:** A real app cannot wait forever on a request. The Fetch API provides `AbortController`
  / `AbortSignal` for exactly this, and `reqwest` exposes `.timeout()`; the frozen surface has neither
  today. The addition is a `RequestBuilder::timeout(..)` (and/or an abort handle) plus a new `Error`
  variant — and because `Error` is already `#[non_exhaustive]`, the new variant is additive by
  construction. It uses `web-sys`'s existing `AbortController`, so it adds no new crate dependency
  (`REQ-HTTP-012`).
- **Acceptance criteria:** A headless `wasm-bindgen-test` issues a request against a never-responding
  stub with a short timeout and asserts the call resolves to the timeout/abort error within the bound
  rather than hanging; the new error variant is matchable and documented. (On a native build, the
  timeout plumbing that does not touch Fetch is unit-tested where it is pure.)
- **verification_method:** automated_test · **stage:** one_oh · **release_gating:** true
- **References:** REQ-FW-009, REQ-FW-013. *Additive: new `RequestBuilder::timeout`/abort handle + a
  new `#[non_exhaustive]` `Error` variant.*

### E. Composition and footprint

#### REQ-HTTP-011 — Composes with the view framework and async layer; no view dependency
- **Statement:** The client is usable from Ferric's async resource/suspense layer — a shared `Client`
  driven inside an async loader that renders loading, loaded, empty, and error states — and does not
  depend on the `ferric` view crate, so it is usable on its own and composes without conflicting
  assumptions.
- **Rationale:** `REQ-FW-008` requires the pieces to compose into one real app; the HTTP client is
  consumed by the async/resource layer (`REQ-ASYNC-*`), which means it must be `Clone`/shareable and
  view-agnostic. The crate already declares no `ferric` dependency, which keeps it reusable and
  testable on its own.
- **Acceptance criteria:** `cargo tree -p ferric-http` shows no dependency on the `ferric` view crate;
  a stated example (or the `chirp-frontend` reference app) drives a `send_json()` from inside an async
  resource and renders distinct loading/loaded/error states from the result.
- **verification_method:** example_build · **stage:** usable_early · **release_gating:** true
- **References:** REQ-FW-007, REQ-FW-008; REQ-ASYNC-* (resource/suspense, read-only).

#### REQ-HTTP-012 — Lean, documented dependency footprint
- **Statement:** The client's dependency set stays limited to the WASM-binding essentials (`web-sys`,
  `js-sys`, `wasm-bindgen`, `wasm-bindgen-futures`) plus `serde` and `serde_json`, with no heavy
  general-purpose HTTP runtime; the set and its bundle-size impact are documented.
- **Rationale:** In WASM every dependency is downloaded bytes before the app runs, and `REQ-FW-013`
  names leanness as a differentiator Ferric competes on. A full-stack native client such as `reqwest`
  pulls a large `hyper`/`tokio` dependency tree ill-suited to a lean WASM bundle; a Fetch-based client
  exists precisely to avoid that. The `serde`/`serde_json`/`wasm-bindgen-futures` additions are the
  narrowly-scoped, per-crate-justified exceptions `REQ-FW-013` anticipates.
- **Acceptance criteria:** A reviewer confirms the crate's `Cargo.toml` dependency set matches the
  stated list, and a check confirms no transitively-heavy runtime (for example `tokio` or `hyper`) is
  pulled on the `wasm32-unknown-unknown` target; the footprint is recorded and its bundle-size
  contribution is reflected in the performance numbers (`REQ-PERF-*`, read-only).
- **verification_method:** doc_inspection · **stage:** one_oh · **release_gating:** true
- **References:** REQ-FW-013; REQ-PERF-* (bundle-size measurement, read-only).

#### REQ-HTTP-013 — Frozen surface carried forward additively
- **Statement:** The `ferric-http` public `prelude` present at the baseline stays usable at 1.0; every
  change is additive (new items only), and no previously exported item is removed or redefined without
  a semver-signalled, changelog-recorded break.
- **Rationale:** `chirp-frontend` and any example are written against the baseline `prelude`; silently
  redefining it during the 1.0 work would break consumers. This is `REQ-FW-009` restated for this crate
  and the condition identity SI2 asserts over this spoke.
- **Acceptance criteria:** A program written against the baseline `prelude` (`Client`, `ClientBuilder`,
  `Error`, `Method`, `RequestBuilder`, `Response`, `Result`) still compiles against the 1.0 crate; the
  only surface differences a `doc_inspection` finds are additions (the `Method::Patch` variant and
  `#[non_exhaustive]`, the timeout/abort method, the new `Error` variant), each recorded in the
  changelog.
- **verification_method:** automated_test · **stage:** usable_early · **release_gating:** true
- **References:** REQ-FW-009; REQ-REL-* (compatibility policy, read-only).

### Register summary

| Facet | Requirements | Stage |
|---|---|---|
| Request building | REQ-HTTP-001, 002, 003 | usable_early |
| Typed JSON fetch | REQ-HTTP-004, 005, 006 | usable_early |
| Error model | REQ-HTTP-007, 008 | usable_early |
| 1.0 additions (flagged additive) | REQ-HTTP-009, 010 | one_oh |
| Composition & footprint | REQ-HTTP-011 (usable_early), 012 (one_oh), 013 (usable_early) | mixed |

All `REQ-HTTP-*` are `release_gating: true` (none is a post-1.0 deferral). Every item carries ≥1
acceptance criterion checkable by a stranger via a parent-schema `verification_method` (SI6).
REQ-HTTP-001–008 and 013 are met by the frozen surface the baseline already landed; REQ-HTTP-009 and
010 are the additive 1.0 work.

---

## Peer comparison

*(section: `peer_comparison` — named comparison against `reqwest` and `gloo-net`, on typed bodies,
builder ergonomics, error handling, and WASM Fetch fit)*

| Dimension | `reqwest` | `gloo-net` | Ferric `ferric-http` |
|---|---|---|---|
| Transport | Native `hyper`/`tokio` stack; a separate `wasm` backend wraps Fetch | Browser Fetch only | Browser Fetch only (`web-sys`) |
| Client object | `Client` holds connection pool + config, clone-cheap, shareable | No client object — base URL/headers repeated per call | `Client` holds base URL + default headers, clone-cheap, no connection state |
| Typed bodies | `.json()` request and response via `serde` | `.json()` request and response via `serde` | `.json(&T)` request, `send_json::<T>()` response via `serde` |
| Safe-by-default status | `error_for_status()` is opt-in; `send` returns any status | Caller checks status each time | `send_json()` treats non-2xx as `Error::Status`; `send()` is the raw escape hatch |
| Error model | Rich `Error` with `is_*` predicates and `status()` | Thinner; a single error over Fetch/serde failures | One `#[non_exhaustive]` `Error` splitting builder / network / status / (de)serialize / body, with `is_*` + `status()` |
| Timeout / cancel | `.timeout()`, abort | Via raw `AbortController` | Added at 1.0 (`REQ-HTTP-010`); not at baseline |
| WASM bundle weight | Heavier — the fuller stack is more than a lean WASM target wants | Light | Light (binding essentials + `serde`) |

Where Ferric sits: it takes `gloo-net`'s leanness and Fetch basis and adds the client-level ergonomics
an author expects from `reqwest` — a configured, shareable `Client` with base URL and default headers,
a typed round-trip that errors on non-2xx by default, and a structured error — without `reqwest`'s
dependency weight. Against `reqwest` it is deliberately smaller: no connection pooling to own (the
browser owns it), synchronous owned-body responses instead of async bodies, and (until `REQ-HTTP-010`
lands) no timeout. Against `gloo-net` it adds the pieces a real app otherwise re-implements by hand:
client configuration, the safe-by-default status path, and a matchable error taxonomy. The one
capability both peers have that the baseline lacks — a request timeout — is the `REQ-HTTP-010`
addition, so the 1.0 client is not behind either peer on the dimensions an app author weighs.

---

## Board fold — issue #6 (`ferric-http`)

*(section: `board_fold`; realizes SI3, feeds parent I7 — record the fold, do not duplicate)*

**Issue #6 — "ferric-http: fetch-based HTTP/JSON client"** (epic #1, milestone M1, labels `framework`
/ `type:feature`) is **CLOSED**. Its scope was "a small async HTTP client over the Fetch API: typed
JSON get/post/put/delete with serde, error handling, request/response helpers; tested; fmt/clippy
clean; delivered as a PR into `main`." The PR landed the `ferric-http` crate at the frozen `prelude`
surface documented above, with the pure logic implemented and natively tested and a browser smoke test
for the Fetch path.

**The fold: #6 is extended, not duplicated.** #6 delivered the `usable_early` core of this client —
it is the scaffold that `REQ-HTTP-001`–`REQ-HTTP-008` and `REQ-HTTP-013` describe and that those
criteria are already met by. The 1.0 design does not re-file that work; it builds on it. The additive
1.0 work is `REQ-HTTP-009` (PATCH + `#[non_exhaustive] Method`) and `REQ-HTTP-010`
(timeout/cancellation), plus ratifying the testable bar across the whole register.

Because #6 is closed, the release plan should record the fold as follows (the plan is owned by
`SUBTASK_release_plan`; backlog edits by `SUBTASK_file_backlog` — both read-only to this spoke):

- The 1.0 HTTP epic/work-items **supersede/extend #6** and cite it as the scaffold they complete, with
  their acceptance criteria tracing to `REQ-HTTP-009` / `REQ-HTTP-010` and this design section (per
  the epic `supersedes` field and identity I7).
- `file_backlog` should **link the new work-items to the closed #6** (reference it in the epic's
  `supersedes`) rather than opening a duplicate "HTTP client" issue — the capability already has a
  home on the board; the 1.0 work is the additive completion of it.

---

## Metadata

- **Composed by:** `SUBTASK_synthesis` from `SUBTASK_router/router.md` (owns `REQ-RT-*`) and
  `SUBTASK_http/http.md` (owns `REQ-HTTP-*`).
- **Repo target:** `design/ferric/router-and-http.md` on branch `work/design-ferric-1_0`
  (per `../contracts/design_artifact_map.yaml`, owner `SUBTASK_router_and_http`; placed by
  `SUBTASK_design_pr`).
- **Conforms to:** `../contracts/id_and_traceability_schema.yaml` (REQ fields, stage vocabulary,
  `verification_method`, `release_gating`); `contracts/_MANIFEST.yaml` (ownership, `doc_layout`,
  identities SI1–SI6).
- **Identities asserted (SI1–SI6):** all PASS — see `SYNTHESIS.md` at this workspace for verdicts.
- **Grounding:**
  - Router: frozen surface read from `crates/ferric-router/src/` and `contracts/ferric_router_api.rs`
    at `276138f`; board issue #5 state read from the issue tracker (CLOSED). Peer behavior
    (`leptos_router`, Dioxus router) from each project's current public documentation.
  - HTTP: `crates/ferric-http/src/*` read directly at `276138f`; board issue #6 and epic #1 read from
    the NSWE Delivery board (CLOSED).
- **Registers:** `REQ-RT-001`–`REQ-RT-009` (001–008 release-gating 1.0; 009 recorded post-1.0
  deferral) · `REQ-HTTP-001`–`REQ-HTTP-013` (all release-gating true).
