# Ferric 1.0 — Async resources, events, and forms (concern 4)

**Owns**: `REQ-ASYNC-001..059` (three sub-ranges; 25 requirements registered).
**Specializes**: `REQ-FW-006` — the three capabilities beyond static rendering: async data with
explicit loading/empty/loaded/error states; event handlers beyond `click`/`input` that carry the
triggering event; two-way input binding that keeps model and field in sync.
**Assembled by**: `SUBTASK_afe_synthesis` from three child fragments.
**Repo target**: `design/ferric/async-forms-events.md`.

At the baseline (`Nousix-LLC/nswe-demo` @ `276138f`, `crates/ferric`): the reactive core is
synchronous (no async primitive, no `wasm-bindgen-futures`); `EventKind` is a closed two-variant
enum (`Click`, `Input`); `EventHandler = Rc<dyn Fn()>` is zero-argument (a handler cannot read the
event that triggered it); and there is no two-way binding, no validation affordance, no `submit`
handler. The three capabilities registered here close those gaps — additively, without redefining
any frozen item (`create_signal`, `Signal`, `Memo`, `create_effect`, `batch`, `mount`, `VNode`,
`EventHandler = Rc<dyn Fn()>`, `EventKind::Click`, `EventKind::Input`).

Cross-area references (`REQ-SSR-*`, `REQ-HTTP-*`, `REQ-API-*`) appear in some requirements
read-only; no id outside `REQ-ASYNC-*` is minted here.

---

## 1. Async resources and suspense

### What this capability is, from the app author's side

An author who needs data from a server today hand-rolls a `create_signal` holding
`Option<Result<Vec<T>, E>>`, spawns a future by hand, manages a separate loading flag, and threads
both through the view. The loading flag and the fetch are not connected to the data, so they drift:
a second fetch races the first, an error leaves a stale spinner, "no results" and "still loading"
render the same empty screen.

The async-resource primitive makes the data source and its state **one reactive value**. The author
declares *where the data comes from* (an async fetcher) and *what it depends on* (a reactive
source); the framework owns the rest — running the fetch, tracking whether it is loading / loaded /
empty / failed, re-running it when the source changes, and driving the view to re-render as that
state moves. The author describes each of the four states once, declaratively, and never writes a
loading flag again.

Two surfaces deliver this:

- a **resource** — a reactive handle (like `Signal`) whose value is the current async state; and
- a **suspense boundary** — a view wrapper that shows one fallback while the resources read inside
  it are still loading, so a screen with several pending fetches has one coherent loading state.

#### Illustrative shape (not a mandated signature)

The requirements below state behavior, not API. This sketch gives acceptance criteria that say
`example_build` something concrete to build against; exact signatures are for the implementing code
DAG. `view!`, `create_signal`, and `create_effect` are the author-facing surfaces owned by the
authoring model (`REQ-API-*`) and the frozen reactive core — shown read-only, not redefined.

```rust
let filter: Signal<Filter> = create_signal(Filter::All);

let todos = create_resource(
    move || filter.get(),
    |f| async move { api::fetch_todos(f).await },
);

view! {
    { todos.render()
        .empty_when(|items| items.is_empty())
        .loading(|| view! { <p>"Loading…"</p> })
        .empty(||   view! { <p>"No todos match this filter."</p> })
        .loaded(|items| view! { <ul>{ items.iter().map(todo_row).collect_view() }</ul> })
        .error(|e| view! {
            <p class="err">{ e.to_string() }</p>
            <button on:click=move |_| todos.refetch()>"Retry"</button>
        })
    }
}

suspense(
    || view! { <Spinner/> },
    move || view! { <Profile/> <Feed/> },
)
```

### Requirements (`REQ-ASYNC-001..016`)

Each requirement conforms to `../../contracts/id_and_traceability_schema.yaml`
(`requirement_fields`, `stage`, `release_gating`, `verification_method`, `references`). Statements
are what must be true from an app author's side; acceptance criteria are checkable by someone who
was not in the room.

#### REQ-ASYNC-001 — Async-resource primitive
- **Statement**: An author can declare an asynchronous data source — a fetcher that returns a Rust
  `Future` — and receive a reactive **resource** handle whose current value is that fetch's state.
  The framework drives the fetcher to completion and updates the resource without the author
  spawning the future or managing a loading flag.
- **Rationale**: A data-backed app is the stated target (`REQ-FW-006`, journey JA2). Every peer
  provides this as the core async primitive: Leptos `create_resource`, Solid `createResource`,
  Dioxus `use_resource`. Ferric has none today, so authors hand-build the future-plus-flag pattern
  the framework should own.
- **Acceptance criteria**:
  1. An example declares a resource from a reactive source and an async fetcher and compiles against
     the 1.0 `ferric` surface (`example_build`).
  2. With no manual future-spawning or loading flag in author code, the resource's value becomes
     the fetched data after the fetch resolves, observed by a reactive read (`automated_test`,
     `wasm-bindgen-test`).
- **Stage**: usable_early · **Release gating**: true
- **References**: `REQ-FW-006`; data source abstraction in REQ-ASYNC-014.

#### REQ-ASYNC-002 — Four distinct states, read reactively
- **Statement**: A resource exposes, as a reactive read, exactly which of four states it is in —
  **loading**, **loaded**, **empty**, **error** — so the author can branch on the current state and
  have the branch re-evaluate when the state changes.
- **Rationale**: `REQ-FW-006` names "distinct loading, loaded, empty, and error states (suspense)"
  as the acceptance bar. Solid exposes `resource.loading` / `resource.error`; Leptos exposes
  `Option<T>` plus an error boundary. "Empty" is the state peers leave to the author and apps
  routinely get wrong (an empty list rendered identically to a still-loading one); naming it as a
  first-class state closes that gap.
- **Acceptance criteria**:
  1. A reactive read of the resource's state returns a value distinguishing loading, loaded, empty,
     and error; reading it inside an effect or the view subscribes to changes (`automated_test`).
  2. The four states are documented with the condition that produces each (`doc_inspection`).
- **Stage**: usable_early · **Release gating**: true
- **References**: `REQ-FW-006`; emptiness in REQ-ASYNC-004; error carriage in REQ-ASYNC-005.

#### REQ-ASYNC-003 — Declarative per-resource state render
- **Statement**: An author can map each of a resource's four states to view content once,
  declaratively, and the framework renders the content for the current state and re-renders when
  the state moves — without the author writing conditional show/hide logic or loading flags.
- **Rationale**: The point of suspense is that the author describes states, not transitions. Leptos
  uses `<Suspense>` + pattern-matching `Option<T>`; Solid uses `<Show>` / `<Switch>` over resource
  accessors. Ferric's view is built from `VNode`, so the natural surface is a state-to-`VNode`
  mapping that composes with the existing render path.
- **Acceptance criteria**:
  1. An example renders four different `VNode`s for the four states of one resource, selected by
     state, with no imperative DOM manipulation in author code (`example_build`).
  2. Moving the resource from loading to loaded to error re-renders to the matching content,
     verified by inspecting the mounted DOM across the transitions (`automated_test`,
     `wasm-bindgen-test`).
- **Stage**: usable_early · **Release gating**: true
- **References**: `REQ-API-*` (the `view!`/component surface this render composes with, read-only).

#### REQ-ASYNC-004 — Empty as a first-class, author-defined state
- **Statement**: An author can declare, per resource, when a successfully loaded value counts as
  **empty** (for example, an empty collection), and a loaded-but-empty value renders the empty
  content rather than the loaded content. When the author declares no emptiness condition, a loaded
  value always renders as loaded.
- **Rationale**: `REQ-FW-006` lists empty as distinct from loaded. No peer makes empty first-class
  — Leptos/Solid/Dioxus all leave it to an author `if items.is_empty()` inside the loaded branch,
  which is the check apps forget. Making it a declared predicate means the four-state contract is
  complete and the empty screen is designed, not accidental.
- **Acceptance criteria**:
  1. With an emptiness predicate declared, a fetch resolving to an empty value renders the empty
     content; a non-empty value renders the loaded content (`automated_test`).
  2. With no predicate declared, a resolved value renders as loaded regardless of its contents
     (`automated_test`).
- **Stage**: usable_early · **Release gating**: true
- **References**: REQ-ASYNC-002.

#### REQ-ASYNC-005 — Error state carries the failure and does not break the view
- **Statement**: When the fetcher's `Future` resolves to an error (or the author's fallible fetch
  yields one), the resource enters the error state carrying the error value; the error content can
  read it (for example to show a message); and a failed fetch neither panics nor leaves the view
  wedged in loading.
- **Rationale**: The hand-rolled pattern's worst failure is a fetch error leaving a permanent
  spinner. Leptos routes resource errors to `<ErrorBoundary>`; Solid to `<ErrorBoundary>` and
  `resource.error`. Ferric authors need the error value at the render site and a guarantee the
  framework caught it.
- **Acceptance criteria**:
  1. A fetcher that resolves to an error moves the resource to the error state with the error value
     readable in the error content (`automated_test`).
  2. After an error, the resource is not in the loading state, and a subsequent `refetch`
     (REQ-ASYNC-007) can recover it (`automated_test`).
- **Stage**: usable_early · **Release gating**: true
- **References**: REQ-ASYNC-002, REQ-ASYNC-007.

#### REQ-ASYNC-006 — Re-fetch when a reactive dependency changes
- **Statement**: A resource re-runs its fetcher when a signal or memo read by its reactive
  **source** changes. While a re-fetch is in flight, a later source change supersedes the earlier
  fetch so the resource's resolved value always reflects the most recent source (no out-of-order
  result wins).
- **Rationale**: This is the behavior that makes a resource reactive rather than a one-shot promise.
  Solid re-runs `createResource` when its source signal changes; Leptos re-runs when the source
  closure's tracked signals change; Dioxus `use_resource` restarts when reactive reads in its body
  change. The out-of-order guarantee is the race the hand-rolled flag pattern cannot express.
- **Acceptance criteria**:
  1. Changing a signal the source reads triggers a new fetch with the new source value
     (`automated_test`).
  2. When two source changes produce two fetches that resolve out of order, the resource's final
     value is the one for the later source change (`automated_test`).
- **Stage**: usable_early · **Release gating**: true
- **References**: REQ-ASYNC-008 (reactive composition); REQ-ASYNC-016 (superseded-fetch cleanup).

#### REQ-ASYNC-007 — Imperative refetch
- **Statement**: An author can trigger a re-fetch on demand (for example a Retry button after an
  error, or a manual refresh) without changing the source.
- **Rationale**: Retry-after-error and pull-to-refresh are standard; Solid exposes `refetch`,
  Leptos `resource.refetch()`, Dioxus `resource.restart()`. Without it, the only recovery from an
  error state is a contrived source change.
- **Acceptance criteria**:
  1. Calling the resource's refetch runs the fetcher again with the current source value and moves
     the resource back through loading to its new resolved state (`automated_test`).
- **Stage**: usable_early · **Release gating**: true
- **References**: REQ-ASYNC-005.

#### REQ-ASYNC-008 — Composition with the reactive graph
- **Statement**: A resource's state is a reactive value in the same graph as signals and memos:
  reading it inside `create_effect`, `create_memo`, or a view subscribes the reader to its changes;
  a state change schedules the readers to re-run exactly as a signal write does; and a `batch` that
  spans a resource read and a signal write behaves consistently with the frozen batching semantics.
- **Rationale**: The resource must not be a second, parallel reactivity system. Authors already know
  `.get()` / `.with()` / effects / `batch`; a resource that participates in that graph composes
  with memos (a memo derived from resource state) and effects (an effect that reacts to load
  completion) with no new mental model. This is how Leptos (signals) and Solid (fine-grained graph)
  both do it.
- **Acceptance criteria**:
  1. A memo that reads a resource's state recomputes when the state changes (`automated_test`).
  2. An effect that reads a resource's state re-runs when the state changes, and does not re-run on
     an unrelated signal write (`automated_test`).
- **Stage**: usable_early · **Release gating**: true
- **References**: frozen `signal.rs` surface (`create_memo`, `create_effect`, `batch`) — read-only.

#### REQ-ASYNC-009 — Suspense boundary over multiple pending resources
- **Statement**: An author can wrap a region of the view in a **suspense boundary** with a single
  fallback; the boundary shows the fallback while any resource read within the region is loading,
  and shows the region's content once every such resource has resolved. One screen with several
  concurrent fetches thus has one coherent loading state instead of several independent spinners.
- **Rationale**: This is the "suspense" named in `REQ-FW-006`. Leptos `<Suspense fallback=…>` and
  Solid `<Suspense>` both aggregate pending resource reads in their subtree into one fallback;
  coordinating several fetches by hand is exactly the plumbing the framework should own.
- **Acceptance criteria**:
  1. A boundary containing two resources shows the fallback until both resolve, then shows the
     content (`automated_test`, `wasm-bindgen-test`).
  2. An example composes a boundary over a region with more than one resource and builds against
     the 1.0 surface (`example_build`).
- **Stage**: usable_early · **Release gating**: true
- **References**: REQ-ASYNC-003 (per-resource render is the complement); REQ-ASYNC-010.

#### REQ-ASYNC-010 — Nested suspense boundaries
- **Statement**: Suspense boundaries nest: an inner boundary resolves on its own resources
  independently of the outer boundary, and an outer boundary's fallback depends only on the
  resources read directly in its region (not on those already delegated to a resolved inner
  boundary). An author can thus reveal part of a screen while a slower nested part still loads.
- **Rationale**: Real screens load in stages — shell first, then a slow panel. Solid and Leptos
  both allow nested `<Suspense>` for staged reveal. Without defined nesting, an author cannot show
  anything until the slowest fetch on the page resolves.
- **Acceptance criteria**:
  1. With a fast resource in an outer boundary and a slow resource in a nested inner boundary, the
     outer content (with the inner fallback) appears when the fast resource resolves, and the inner
     content replaces its fallback when the slow resource later resolves (`automated_test`).
- **Stage**: usable_early · **Release gating**: true
- **References**: REQ-ASYNC-009.

#### REQ-ASYNC-011 — Loading-with-previous-value distinguishable from first load
- **Statement**: While a resource re-fetches after having already loaded once, the author can tell
  that a prior loaded value exists and choose to keep showing it (avoiding a flash back to the
  loading state on every refresh), distinct from the first load where no prior value exists.
- **Rationale**: Re-fetching on every keystroke of a search filter should not blank the results
  list each time. Solid's resource keeps its latest value readable while `loading` is true; Leptos
  exposes the prior value during refetch. Without this distinction, REQ-ASYNC-006's re-fetch
  produces a visible flicker on each source change.
- **Acceptance criteria**:
  1. After a first load, a source-change-triggered re-fetch leaves the previously loaded value
     readable while the new fetch is in flight, and the loading indication distinguishes "reloading"
     from "first load" (`automated_test`).
- **Stage**: usable_early · **Release gating**: true
- **References**: REQ-ASYNC-006.

#### REQ-ASYNC-012 — SSR resolution handed to the client (by reference)
- **Statement**: A resource created during a server render resolves its fetcher on the server, and
  its resolved state is made available to the client so that the first client render shows the
  loaded (or empty / error) state rather than loading. The mechanism that serializes the resolved
  state and carries it into hydration is **not defined here** — it is owned by `REQ-SSR-*`
  (`SUBTASK_ssr_hydration`), which this requirement references read-only.
- **Rationale**: SSR exists so the first paint is content, not a spinner; a resource that
  re-entered loading on the client would defeat it. Leptos serializes resource values into the SSR
  payload for exactly this reason. Ferric's async resource must be designed to *participate* in that
  handoff; the carrying mechanism is the SSR child's to build (J5).
- **Acceptance criteria**:
  1. The design states that a resource's resolved server-side state is the input to client hydration
     and cites the `REQ-SSR-*` requirement that carries it, minting no `REQ-SSR-*` id here
     (`doc_inspection`).
- **Stage**: one_oh · **Release gating**: true
- **References**: `REQ-SSR-*` (read-only, `SUBTASK_ssr_hydration`); REQ-ASYNC-013.

#### REQ-ASYNC-013 — No refetch on hydration when SSR provided the state (by reference)
- **Statement**: When a resource's resolved state was produced during SSR and carried to the client,
  the client does not re-run the fetcher on initial hydration; it adopts the carried state. The
  carriage mechanism is `REQ-SSR-*` (read-only); this requirement states only the resource-side
  obligation not to refetch what SSR already resolved.
- **Rationale**: A double fetch (once on the server, again on hydration) wastes a request and can
  flash the UI. This is the client-side half of REQ-ASYNC-012 and the behavior Leptos's serialized
  resources guarantee.
- **Acceptance criteria**:
  1. The design states that a hydrating resource with a carried SSR state performs no initial
     fetch, and references the `REQ-SSR-*` hydration requirement without minting one
     (`doc_inspection`).
- **Stage**: one_oh · **Release gating**: true
- **References**: REQ-ASYNC-012; `REQ-SSR-*` (read-only).

#### REQ-ASYNC-014 — Abstract data source (HTTP client by reference)
- **Statement**: The resource's data source is any async fetcher returning a Rust `Future`; the
  resource does not define or require a particular HTTP or JSON client. A fetcher backed by the
  framework's HTTP client (`REQ-HTTP-*`, read-only), by a third-party client, or by a non-network
  async source (a timer, a local async computation) all satisfy the primitive equally.
- **Rationale**: Coupling the resource to one HTTP client would force that client on apps that use a
  different one and would blur the two concerns. Solid's `createResource` and Dioxus's `use_resource`
  take an arbitrary async function; the fetch client is orthogonal. The HTTP/JSON client is
  `REQ-HTTP-*`'s to design (`SUBTASK_router_and_http`); no `REQ-HTTP-*` id is minted here (J5).
- **Acceptance criteria**:
  1. An example builds a resource over a non-HTTP async source (`example_build`).
  2. The design states the fetcher contract as "returns a `Future`" with no dependency on a
     specific client, referencing `REQ-HTTP-*` read-only (`doc_inspection`).
- **Stage**: usable_early · **Release gating**: true
- **References**: `REQ-HTTP-*` (read-only, `SUBTASK_router_and_http`).

#### REQ-ASYNC-015 — Additive over the frozen reactive core and runtime
- **Statement**: The async-resource primitive is additive over the frozen `ferric` surface. The
  synchronous contract of `create_effect(impl FnMut() + 'static)`, the `Signal`/`Memo` handle API,
  `create_signal` / `create_memo` / `batch`, and the `mount` / `VNode` render path are unchanged;
  the resource, the suspense boundary, and the async executor that drives fetchers are new items,
  not redefinitions of any frozen item.
- **Rationale**: The 1.0 line is drawn against what is real (parent I9, J4). The core is explicitly
  synchronous today and the crate carries no `wasm-bindgen-futures`; introducing async must extend
  the surface, not quietly change the effect contract existing code relies on.
- **Acceptance criteria**:
  1. The frozen prelude items (`create_signal`, `create_memo`, `create_effect`, `batch`, `Signal`,
     `Memo`, `mount`, `VNode`) retain their current signatures and semantics; the async additions
     are new names (`automated_test` that existing reactive-core tests still pass; `doc_inspection`
     of the diff against `276138f`).
- **Stage**: usable_early · **Release gating**: true
- **References**: frozen `signal.rs` / `dom.rs` surfaces (read-only); parent I9.

#### REQ-ASYNC-016 — Superseded and dropped fetches are cleaned up
- **Statement**: When a re-fetch supersedes an in-flight fetch (REQ-ASYNC-006), or when a
  resource's owning reactive scope is disposed, the result of the superseded or orphaned fetch does
  not update the resource's state. A resolved fetch never writes to a resource that has been
  superseded or disposed.
- **Rationale**: This is the correctness guarantee behind REQ-ASYNC-006's out-of-order rule and
  the leak that hand-rolled fetching causes most often (a late response overwriting newer data, or
  a response landing after the component is gone). Fine-grained frameworks (Solid) tie fetch
  lifetime to the reactive owner for exactly this reason.
- **Acceptance criteria**:
  1. A superseded fetch that resolves after its successor does not change the resource's resolved
     value (`automated_test`).
  2. A fetch that resolves after its resource's scope is disposed produces no state update and no
     panic (`automated_test`).
- **Stage**: usable_early · **Release gating**: true
- **References**: REQ-ASYNC-006.

*(REQ-ASYNC-017..019 are unused headroom within this child's assigned range.)*

### Composition

**With reactivity.** The resource participates in the frozen reactive graph (REQ-ASYNC-008,
REQ-ASYNC-015): its state is held in reactive cells, reads subscribe, writes schedule readers. A
memo may derive from resource state; an effect may fire on load completion; `batch` spans both. The
one new runtime capability is an executor that drives a fetcher `Future` on the browser event loop
and writes the result back into the reactive cells — additive, and its exact vehicle (for example
`wasm-bindgen-futures`) is the implementing DAG's choice.

**With SSR (by reference only).** REQ-ASYNC-012 and REQ-ASYNC-013 state the resource-side
obligations and point at `REQ-SSR-*` (`SUBTASK_ssr_hydration`) for the serialization-and-hydration
mechanism. No `REQ-SSR-*` id is minted and no hydration mechanism is designed here (J5).

**With the data source (by reference only).** The fetcher is "any async function returning a
`Future`" (REQ-ASYNC-014). The HTTP/JSON client is `REQ-HTTP-*`'s (`SUBTASK_router_and_http`);
the resource composes with it by reference.

### Peer comparison

| Framework | Async primitive | Re-fetch trigger | Suspense / error surface | Empty state | SSR handling |
|---|---|---|---|---|---|
| **Leptos** | `create_resource(source, fetcher)`; value read as `Option<T>` | source closure's tracked signals change; `.refetch()` | `<Suspense fallback>` aggregates pending reads; `<ErrorBoundary>` for errors | author checks inside loaded branch | serializes resolved resource into SSR payload; hydration adopts it, no refetch |
| **Solid** | `createResource(source, fetcher)` → accessor with `.loading` / `.error`; `{ mutate, refetch }` | source signal changes; `refetch()` | `<Suspense>`, `<SuspenseList>`, `<ErrorBoundary>`; keeps latest value during refetch | author checks inside loaded branch | `<Suspense>` resources resolve during SSR and stream/hydrate |
| **Dioxus** | `use_resource(async closure)` → `Resource<T>` | reactive reads in the closure change; `.restart()` | `SuspenseBoundary`; `.state()` / `.value()` read | author checks inside value branch | server futures resolve during SSR; hydration adopts |
| **Ferric 1.0** | `create_resource(source, fetcher)` → reactive `Resource` (REQ-ASYNC-001) | reactive source change (REQ-ASYNC-006); `refetch` (REQ-ASYNC-007) | per-resource four-state render (REQ-ASYNC-003) and `suspense` boundary (REQ-ASYNC-009–010); error carries value (REQ-ASYNC-005) | **first-class, author-declared `empty_when`** (REQ-ASYNC-004) | resolves on server, adopts carried state, no double-fetch — mechanism via `REQ-SSR-*` (REQ-ASYNC-012–013) |

The resource / re-fetch / suspense / error shape follows Leptos and Solid closely. The one place
this design is more opinionated than all three peers is **empty as a first-class, declared state**
(REQ-ASYNC-004): Leptos, Solid, and Dioxus leave empty to an author-written `if` inside the loaded
branch. `REQ-FW-006` lists empty as distinct, so Ferric designs it in. REQ-ASYNC-012/013 are staged
`one_oh`, matching where SSR+hydration sits on the MVP-vs-1.0 line.

---

## 2. Full DOM event coverage and the payload-carrying handler form

### What this capability is, from the app author's side

At the baseline, `EventKind` is a closed enum of two variants (`Click` and `Input`) and
`EventHandler = Rc<dyn Fn()>` is zero-argument: a handler cannot read the event that triggered it,
so there is no way to get an input's value, a pressed key, or a submit target from inside a
handler. The runtime already receives the `web_sys::Event` and discards it (`move |_event:
web_sys::Event| { handler(); }`). The `vdom.rs` module doc explicitly sanctions an additive
enrichment "to carry a `web_sys::Event` when it needs the event payload."

This capability settles two things:

1. **A handler can read the event that triggered it** — an input's value, the key pressed, the form
   submitted — while the frozen zero-argument handler form keeps working unchanged. This is the
   **seam** the forms section (§3) builds two-way binding on; it is defined once, here.
2. **The range of bindable DOM events reaches beyond `click` and `input`** — submit, keyboard,
   focus/blur, change, and any other named DOM event — additively over the frozen `EventKind`.

Both changes are additive: the frozen `EventKind::Click`, `EventKind::Input`, and
`EventHandler = Rc<dyn Fn()>` remain valid and unchanged in all respects.

### Requirements (`REQ-ASYNC-020..024`)

All five requirements are stage `usable_early` and `release_gating: true` — they specialize
`REQ-FW-006`, which is `usable_early`: an author cannot build a real, form-driven app without
reading events and binding more than two of them.

#### REQ-ASYNC-020 — The payload-carrying handler form (additive over the frozen zero-arg handler)

- **Statement:** An author can bind an event handler that optionally **receives the triggering
  event's payload**, while the frozen zero-argument handler form (`EventHandler = Rc<dyn Fn()>`,
  `VElement::on(kind, || …)`) remains valid and unchanged. Both forms are available on every bound
  event; the author chooses per binding whether the handler needs the payload.
- **Rationale:** The zero-arg form cannot read the event, so an author cannot get an input's value
  or a pressed key from inside a handler — the gap that blocks forms and real event handling. The
  baseline wrapper already receives a `web_sys::Event` and discards it, and the `vdom.rs` module
  doc sanctions enriching the handler form to carry it. Every Rust peer gives handlers their event:
  Leptos passes the typed `web_sys` event to `on:input`/`on:click` closures; Dioxus passes typed
  event data (`Event<MouseData>`, `KeyboardData`, …); Yew's callbacks receive `web_sys` events;
  Sycamore's `on:*` closures receive the `web_sys::Event`. A zero-arg-only framework is alone in
  withholding it.
- **Acceptance criteria:**
  - A1. An app written against the baseline — handlers bound with the zero-argument `.on(kind, ||
    …)` form — compiles and runs unchanged against the 1.0 surface; no previously exported item is
    removed or redefined. *(verification: `automated_test` — a `trybuild`/`cargo test` build of a
    baseline-style view; `example_build`.)*
  - A2. An author can bind a handler that receives the triggering event's payload and reads from it
    within the handler body; a stated example binds such a handler and observes the payload.
    *(verification: `automated_test` — headless `wasm-bindgen-test` dispatches an event and asserts
    the handler saw the payload; `example_build`.)*
  - A3. The zero-argument and payload-carrying forms coexist on the same element and the same event
    type (one element may bind a zero-arg handler for one event and a payload handler for another).
    *(verification: `automated_test`.)*
  - A4. The payload-carrying form is defined in exactly this requirement for the whole concern; the
    forms section and the event-coverage requirements reference it rather than restating it.
    *(verification: `doc_inspection` — REQ-ASYNC-040..059 reference REQ-ASYNC-020/022, not a
    second handler form.)*
- **Stage:** usable_early · **Release gating:** true
- **References:** `REQ-FW-006`, `REQ-FW-009` (frozen surfaces carried forward additively);
  consumed read-only by REQ-ASYNC-022, REQ-ASYNC-023, and the forms register
  REQ-ASYNC-040..059. Sub-contract seam J3.

#### REQ-ASYNC-021 — Full DOM event coverage beyond click/input

- **Statement:** An author can bind the range of DOM events a real app needs — at least `submit`,
  the keyboard events (`keydown`, `keyup`, `keypress`), `focus`/`blur`, and `change`, in addition
  to the baseline `click` and `input` — **and** can bind any other DOM event by its name, without
  the framework having to enumerate it ahead of time. The baseline `EventKind::Click` and
  `EventKind::Input` remain valid and first-class.
- **Rationale:** Today's runtime binds only `click` and `input`; a form cannot submit, a key
  cannot be observed, a field cannot react to blur. `REQ-FW-006` names submit, keyboard,
  focus/blur, and change as the floor. A framework that fixes its event set in a closed enum cannot
  carry the custom and less-common events real apps use, so the common events are first-class for
  discoverability and the long tail is reachable by name — the shape peers converge on (Leptos and
  Yew expose the full typed DOM event set; Dioxus names the common events and types their data).
- **Acceptance criteria:**
  - A1. A stated example binds and fires each of `submit`, `keydown`, `focus`, `blur`, and `change`
    and observes the handler run; the baseline `click` and `input` bindings continue to fire.
    *(verification: `automated_test` — headless `wasm-bindgen-test` dispatches each event type;
    `example_build`.)*
  - A2. An author can bind a DOM event that is not in the first-class set by naming it, and the
    handler fires when that event is dispatched. *(verification: `automated_test`.)*
  - A3. The widening is additive: the baseline `EventKind` variants and the baseline `.on` binding
    are unchanged; no frozen variant is removed or renamed. *(verification: `automated_test` — a
    baseline binding still compiles; `doc_inspection`.)*
  - A4. Each first-class event, bound with the payload-carrying form of REQ-ASYNC-020, delivers the
    event data appropriate to it (a key for keyboard events, the target for change/submit).
    *(verification: `automated_test`; `example_build`.)*
- **Stage:** usable_early · **Release gating:** true
- **References:** `REQ-FW-006`; builds on REQ-ASYNC-020 (payload form) and is read by
  REQ-ASYNC-024 (re-binding). Sub-contract identity J4 (additive over frozen `EventKind`).

#### REQ-ASYNC-022 — Reading the triggering event's data from the payload

- **Statement:** From a payload-carrying handler, an author can read the data the event carries —
  an input or change target's current **value**, the **key** of a keyboard event, and the
  underlying `web_sys` event itself — through the framework, without reaching around it to raw DOM
  lookups.
- **Rationale:** Receiving the payload (REQ-ASYNC-020) is only useful if the common reads are
  ergonomic. The dominant read is "what did the user type," which every peer makes a one-liner:
  Leptos ships `event_target_value(&ev)` and a typed `on:input:target`; Dioxus exposes
  `event.value()` on form events; Yew reads the target via
  `target_dyn_into::<HtmlInputElement>()`. Ferric should give the author the value and the key
  directly, and still expose the raw `web_sys::Event` for anything the helpers do not cover —
  keeping the lean `web-sys` footprint (`REQ-FW-013`) rather than a bespoke event model.
- **Acceptance criteria:**
  - A1. A handler on an `input` or `change` event reads the field's current value through a
    framework accessor and the value matches what was typed. *(verification: `automated_test` —
    `wasm-bindgen-test` sets a field value, dispatches the event, asserts the read value;
    `example_build`.)*
  - A2. A handler on a keyboard event reads which key was pressed through a framework accessor.
    *(verification: `automated_test`.)*
  - A3. The raw `web_sys::Event` is reachable from the payload so an author can handle an event the
    accessors do not cover. *(verification: `automated_test`; `doc_inspection`.)*
  - A4. Reading the payload requires no dependency beyond the framework and its existing
    `wasm-bindgen`/`web-sys`/`js-sys` base. *(verification: `doc_inspection` — the dependency set
    is unchanged; `example_build`.)*
- **Stage:** usable_early · **Release gating:** true
- **References:** `REQ-FW-006`, `REQ-FW-013`; composes on REQ-ASYNC-020; consumed read-only by the
  forms two-way binding register REQ-ASYNC-040..059.

#### REQ-ASYNC-023 — Default-action and propagation control from a handler

- **Statement:** From a handler that receives the payload, an author can suppress the browser's
  default action for the event (for example, stop a `submit` from navigating the page) and can
  stop the event propagating to ancestor handlers.
- **Rationale:** A two-way-bound form is unusable without `preventDefault` on `submit` — otherwise
  the browser performs a full-page form navigation and the single-page app is lost. Propagation
  control is needed wherever nested elements both handle an event. Peers provide both: Dioxus
  exposes `event.prevent_default()` / `stop_propagation()`; Leptos and Yew expose them on the
  `web_sys` event. This is a direct consequence of REQ-ASYNC-020 delivering the event, and is the
  specific capability the forms section's submit flow relies on.
- **Acceptance criteria:**
  - A1. A `submit` handler that suppresses the default action prevents the page navigation; a
    stated example submits a form and the view is not reloaded. *(verification: `automated_test`
    — `wasm-bindgen-test`; `example_build`.)*
  - A2. A handler can stop propagation so an ancestor handler for the same event does not fire.
    *(verification: `automated_test`.)*
  - A3. Default-action and propagation control are reachable only through the payload-carrying
    form; the zero-argument form is unaffected and still valid. *(verification: `doc_inspection`;
    `automated_test`.)*
- **Stage:** usable_early · **Release gating:** true
- **References:** `REQ-FW-006`; composes on REQ-ASYNC-020; relied on by the forms submission flow
  in REQ-ASYNC-040..059.

#### REQ-ASYNC-024 — Re-binding across the keyed diff, stated from the author's side

- **Statement:** An author can rely on these re-binding guarantees across re-renders, for the full
  event surface of REQ-ASYNC-021 and both handler forms of REQ-ASYNC-020:
  1. **A render that does not change which event types are bound on an element keeps that element's
     existing listeners** — the framework does not detach and re-attach on every render. Because
     handlers read their signals live, a retained listener reflects current state; an author
     therefore reads dynamic values from signals (or from the event payload) inside the handler,
     not by capturing them at bind time.
  2. **Adding or removing an event type on an element across a render re-binds that element's
     listeners** to match the new set (for example, conditionally binding `keydown` only while a
     mode is active attaches and detaches the `keydown` listener as the condition flips).
  3. These guarantees hold identically whether an event is a first-class kind or bound by name, and
     whether its handler is zero-argument or payload-carrying.
- **Rationale:** The baseline `diff_events` emits `Patch::SetEvents` only when the set of bound
  `EventKind`s changes between the old and new element; this requirement states that behavior as an
  author-facing contract and extends it over the widened surface, so an author knows when a handler
  swap takes effect and why reading live state inside the handler is the correct pattern. It is
  stated from the author's side — what re-binding they can rely on — not as diff internals.
- **Acceptance criteria:**
  - A1. Re-rendering an element with the same set of bound event types does not re-attach its
    listeners; a handler that reads a signal observes the signal's current value when the event
    fires. *(verification: `automated_test` — `wasm-bindgen-test` re-renders, fires the event,
    asserts current state is read; the existing `vdom.rs` diff tests cover the
    no-`SetEvents`-on-equal-set case with `cargo test`.)*
  - A2. Adding an event type across a render causes that event to fire a handler afterward, and
    removing it causes it to stop firing. *(verification: `automated_test`.)*
  - A3. The guarantees are documented for the author (a guide statement that live-signal reads
    inside handlers are the intended pattern and when re-binding occurs). *(verification:
    `doc_inspection`.)*
- **Stage:** usable_early · **Release gating:** true
- **References:** `REQ-FW-006`; reads REQ-ASYNC-020 and REQ-ASYNC-021; the live-signal-read
  guarantee is the property the forms two-way binding register REQ-ASYNC-040..059 relies on.

*(REQ-ASYNC-025..039 are unused headroom within this child's assigned range.)*

### Seam for forms (§3)

Two-way binding is built on top of this section's capabilities. The forms register
(REQ-ASYNC-040..059) references these ids read-only and does not restate the handler form
(sub-contract identity J3):

| What forms binding needs | Provided by | How forms composes on it |
|---|---|---|
| A handler that receives the event | REQ-ASYNC-020 | The `input`/`change` handler that drives a signal is a payload-carrying handler. |
| Reading the field's current value | REQ-ASYNC-022 | Two-way binding writes the read value into the bound signal on each `input`. |
| `submit` with the default suppressed | REQ-ASYNC-023 | A bound form's submit handler suppresses the page navigation before running validation/submission. |
| Listener stability + live-signal reads | REQ-ASYNC-024 | The bound signal is read live inside the handler, so a retained listener stays correct across renders. |

### Peer comparison

| Framework | Handler receives the event? | Reading an input value | Event coverage | Default/propagation |
|---|---|---|---|---|
| **Leptos** (verified) | Yes — `on:click`/`on:input` closures receive the typed `web_sys` event | `event_target_value(&ev)`; typed `on:input:target` → `ev.target().value()` | Full typed DOM event set via `on:*` | On the `web_sys` event |
| **Dioxus** (verified) | Yes — typed event data (`Event<MouseData>`, `KeyboardData`, `FormData`) | `event.value()` on form events | Named handlers (`onclick`, `oninput`, `onkeydown`, …) with typed data | `event.prevent_default()` / `stop_propagation()` |
| **Yew** | Yes — `Callback` receives `web_sys` events (`MouseEvent`, `InputEvent`, `KeyboardEvent`) | `target_dyn_into::<HtmlInputElement>()` then `.value()` | Full `web_sys` event set | On the `web_sys` event |
| **Sycamore** | Yes — `on:*` closures receive the `web_sys::Event` | Read the target from the event | Full DOM event surface | On the `web_sys` event |
| **Solid** (JS analog) | Yes — native DOM events | `e.currentTarget.value` | Native DOM events | Native event methods |
| **Ferric today** (`276138f`) | **No** — `EventHandler = Rc<dyn Fn()>` is zero-arg; the runtime receives and discards the event | **Not possible** | **`click` + `input` only** | **Not reachable** |

Verified against current sources: Leptos —
[`event_target_value`](https://docs.rs/leptos/latest/leptos/prelude/fn.event_target_value.html) and
the [Forms and Inputs](https://book.leptos.dev/view/05_forms.html) chapter; Dioxus —
[event handlers guide](https://dioxuslabs.com/learn/0.7/essentials/basics/event_handlers/) and
[`dioxus_core::Event`](https://docs.rs/dioxus-core/latest/dioxus_core/struct.Event.html). The Yew,
Sycamore, and Solid rows state each framework's established public model and are directional. The
two verified rows establish the norm: giving a handler its event is settled across the field, and
Ferric is the outlier. REQ-ASYNC-020..024 close that gap additively.

---

## 3. Forms and two-way input binding

### What this capability is, from the app author's side

At the baseline, two-way binding is not expressible at all: `EventHandler = Rc<dyn Fn()>` is
zero-argument, so a handler cannot read the edited field's value — there is no way to get the typed
text back into a signal. The events section (§2) closes that gap. Two-way binding stands on that
work: it is why the forms capability depends on the events capability.

The reactive core already provides the signal-to-field direction: reading a signal in a view
re-renders when it changes, and the keyed diff updates a field's `value` attribute to match. What
was missing was the field-to-signal direction (the handler that reads the typed value and writes it
into the signal) and the binding affordance that wires both directions to one signal handle.

This capability registers:
- **Two-way binding of a signal to an input** — one affordance, one `Signal<T>` handle, both
  directions expressed together.
- **Per-kind coverage** — binding reads and writes the property appropriate to each input kind
  (value for text/textarea, checked for checkbox, selected for select, numeric for number input).
- **Validation ergonomics** — reactive per-field and per-form validity and error messages, built
  from the existing reactive primitives over the bound signals, without a bundled validator DSL.
- **Submission handling** — suppressing the default navigation, reading the model, gating on
  validity.

### Requirements (`REQ-ASYNC-040..043`)

All four requirements are stage `usable_early` and `release_gating: true` — they specialize
`REQ-FW-006` (itself `usable_early`): an author cannot build a real, form-driven app without
binding a field two-way, surfacing validity, and submitting without a page reload. Ids are assigned
monotonically within the assigned range `REQ-ASYNC-040..059` (identity J1).

#### REQ-ASYNC-040 — Two-way binding of a signal to an input, so the model and the field stay in sync

- **Statement:** An author can bind an input to a `Signal<T>` so that editing the field updates the
  signal and writing the signal updates the field, through one binding affordance applied to one
  signal handle. The author does not hand-wire the value-out and change-in edges separately;
  binding expresses both.
- **Rationale:** This is the forms clause of `REQ-FW-006` — "a form binds an input two-way so the
  model and the field stay in sync." At the baseline it was not expressible at all, because the
  zero-argument handler could not read the edited value; the payload-carrying handler
  (REQ-ASYNC-020) and the value read (REQ-ASYNC-022) now make the field→signal edge reachable, and
  the render effect already makes the signal→field edge reachable. Ferric's `create_signal` returns
  a single `Copy` read-write handle, so one signal is both the source and the sink of the binding
  — the author binds one handle, not a read/write pair. Peers converge on a two-way shortcut over
  exactly this pair: Leptos `bind:value`, Svelte `bind:value`, Vue `v-model`.
- **Acceptance criteria:**
  - A1. A stated example binds a text input to a `Signal<String>`; typing in the field changes the
    signal's value, and setting the signal from elsewhere changes what the field displays.
    *(verification: `automated_test` — headless `wasm-bindgen-test` types into the field and
    asserts the signal, then sets the signal and asserts the field value; `example_build`.)*
  - A2. The binding is built on the payload-carrying handler form: the field→signal edge uses the
    handler of REQ-ASYNC-020 and the value read of REQ-ASYNC-022, and this requirement does not
    define a second handler form. *(verification: `doc_inspection` — the binding references
    REQ-ASYNC-020/022, not a new handler form.)*
  - A3. A bound field stays in sync across re-renders without the author re-binding: after an
    unrelated re-render, editing the field still updates the signal and a signal write still updates
    the field. *(verification: `automated_test` — re-render, then exercise both directions; rests
    on the live-read/re-bind guarantee of REQ-ASYNC-024.)*
  - A4. Binding is additive over the frozen surfaces: an app written against the baseline signal and
    event APIs still compiles, and binding adds a new affordance rather than redefining `Signal`,
    `EventKind`, `EventHandler`, or `VElement::on`. *(verification: `automated_test`;
    `doc_inspection`.)*
- **Stage:** usable_early · **Release gating:** true
- **References:** `REQ-FW-006`, `REQ-FW-009`; composes read-only on REQ-ASYNC-020 (payload
  handler), REQ-ASYNC-022 (value read), REQ-ASYNC-024 (live reads across re-render). Sub-contract
  seam J3.

#### REQ-ASYNC-041 — Two-way binding across the common input kinds

- **Statement:** An author can bind two-way across the input kinds a real form needs — at least a
  text input and textarea (to a string signal), a checkbox (to a boolean signal), a radio group
  (to the selected value), a single and multiple `select` (to the selected option or options), and
  a number input (to a numeric signal) — with the binding reading and writing the property
  appropriate to each kind, not a one-size value coercion.
- **Rationale:** "Bind an input two-way" is not one shape: a checkbox's state is `checked`, not
  `value`; a `select` carries a selected option; a multiple select carries several; a number field
  is a numeric model behind a string field. A binding that only handled `<input type=text>.value`
  would not carry a real form. Peers handle the kinds explicitly: Svelte has `bind:value`,
  `bind:checked`, and `bind:group`; Vue's `v-model` adapts to the control and offers
  `.number`/`.trim` modifiers; Leptos binds `value` and `checked`. The payload read of
  REQ-ASYNC-022 exposes the target, so per-kind reads compose on the one seam.
- **Acceptance criteria:**
  - A1. A stated example binds, two-way, each of: a text input and a textarea to a string signal,
    a checkbox to a boolean signal, a radio group to a selected-value signal, a single `select` to
    a selected-option signal, and a number input to a numeric signal; each field edit updates its
    signal and each signal write updates its field. *(verification: `automated_test` —
    `wasm-bindgen-test` per kind; `example_build`.)*
  - A2. A multiple `select` binds to a collection signal so selecting and deselecting options
    updates the collection and setting the collection updates the selection.
    *(verification: `automated_test`.)*
  - A3. A number input's binding surfaces a numeric model (not a raw string) to the author, and a
    non-numeric or empty field is observable to the author rather than silently coerced to a wrong
    number. *(verification: `automated_test`; `doc_inspection` — the documented behavior for an
    unparseable value is stated.)*
  - A4. Each kind's binding reads and writes through the framework's payload accessor and the
    reactive attribute, adding no dependency beyond the existing `wasm-bindgen`/`web-sys`/`js-sys`
    base (`REQ-FW-013`). *(verification: `doc_inspection` — dependency set unchanged; `example_build`.)*
- **Stage:** usable_early · **Release gating:** true
- **References:** `REQ-FW-006`, `REQ-FW-013`; composes on REQ-ASYNC-040 and read-only on
  REQ-ASYNC-022.

#### REQ-ASYNC-042 — Validation ergonomics: reactive per-field and per-form validity and messages

- **Statement:** An author can express per-field and whole-form validation over the bound signals
  and surface the resulting validity and error messages in the view reactively, so that a field
  becoming invalid shows its message and becoming valid clears it without manual refresh wiring.
  The framework provides the reactive substrate (derived values over the bound signals); it does not
  require a bundled validation schema language.
- **Rationale:** A form that cannot tell the user what is wrong is not usable, and validity is a
  function of the model that must update as the model updates — exactly what the reactive core
  (`create_memo` / derived signals) is for. Keeping validation as author-expressed reactive values,
  rather than a bundled validator DSL, matches every peer core framework: Leptos, Solid, and Dioxus
  leave validation to the author's reactive code (or an ecosystem library) and provide the
  reactivity it rests on. Ferric's bar is that validity and messages are expressible, reactive, and
  renderable — not that Ferric ships a validator.
- **Acceptance criteria:**
  - A1. A stated example derives a field's validity from its bound signal and renders an error
    message when the field is invalid; editing the field to a valid value removes the message
    reactively, with no explicit re-render call by the author. *(verification: `automated_test` —
    `wasm-bindgen-test` drives the bound signal across the validity boundary and asserts the
    rendered message appears and clears; `example_build`.)*
  - A2. An author can derive a whole-form validity value from multiple fields' validity, and it
    updates when any contributing field changes. *(verification: `automated_test`.)*
  - A3. Validity and messages are expressed with the framework's existing reactive primitives
    (signals/memos/effects) over the bound signals, requiring no primitive beyond the reactive core
    and no third-party dependency. *(verification: `doc_inspection`; `example_build`.)*
  - A4. The guide states the intended validation pattern for the author — derive validity reactively
    from the bound signals and render messages from derived values — so the approach is
    discoverable rather than implied. *(verification: `doc_inspection`.)*
- **Stage:** usable_early · **Release gating:** true
- **References:** `REQ-FW-006`; composes on REQ-ASYNC-040 (bound signals) and the reactive core
  (`create_signal`/`create_memo`/`create_effect`).

#### REQ-ASYNC-043 — Submission ergonomics: submit without a page reload, read the model, gate on validity

- **Statement:** An author can handle a form's submission so that the browser's default navigation
  is suppressed (the single-page app is not reloaded), the submitted values are read from the bound
  signals (the model), and submission can be refused or deferred while the form is invalid — all
  stated from the author's side.
- **Rationale:** Without suppressing the default `submit` action the browser performs a full-page
  form navigation and the app is lost, so `preventDefault` on submit (REQ-ASYNC-023) is the
  capability a bound form's submit flow depends on. Reading the submitted values from the bound
  signals rather than re-scraping the DOM means the author works against the model they already
  have. Gating on validity (REQ-ASYNC-042) turns a bound, validated form into one that submits
  only acceptable data. Every peer handles submit through the event with `preventDefault` and reads
  the model, not the DOM.
- **Acceptance criteria:**
  - A1. A stated example attaches a `submit` handler that suppresses the default action; submitting
    the form does not reload or navigate the page. *(verification: `automated_test` —
    `wasm-bindgen-test` dispatches `submit` and asserts no navigation; `example_build`.)*
  - A2. The submit handler reads the submitted values from the bound signals, and the values it
    reads match what was entered through the two-way binding. *(verification: `automated_test`.)*
  - A3. A stated example refuses or defers submission while the form's derived validity is false
    and proceeds once it is true, surfacing to the user why submission did not proceed.
    *(verification: `automated_test`; `example_build`.)*
  - A4. The submit flow composes on the payload-carrying handler and its default-action control and
    does not define its own handler form or its own `preventDefault`. *(verification:
    `doc_inspection` — references REQ-ASYNC-023, not a new control surface.)*
- **Stage:** usable_early · **Release gating:** true
- **References:** `REQ-FW-006`; composes read-only on REQ-ASYNC-023 (default-action control) and
  REQ-ASYNC-020 (payload handler); composes on REQ-ASYNC-040 (bound signals) and REQ-ASYNC-042
  (validity gating).

*(REQ-ASYNC-044..059 are unused headroom within this child's assigned range.)*

### How this section composes on the event-payload seam (§2)

| What forms binding uses | Provided by (read-only) | How this section composes on it |
|---|---|---|
| A handler that receives the triggering event | REQ-ASYNC-020 | The `input`/`change` handler that drives a bound signal is a payload-carrying handler. |
| Reading the field's current value / selected state | REQ-ASYNC-022 | The field→signal edge reads the value (or `checked`, or the selection) and writes it into the bound signal. |
| `submit` with the default action suppressed | REQ-ASYNC-023 | The bound form's submit handler suppresses the page navigation before reading the model and running validity gating. |
| Listener stability + live-signal reads across the keyed diff | REQ-ASYNC-024 | The bound signal is read live inside the handler, so a retained listener stays correct across re-renders and the author binds once. |

### Peer comparison

| Framework | Two-way binding affordance | Input-kind coverage | Validation | Submit model |
|---|---|---|---|---|
| **Leptos** (verified) | `bind:value` / `bind:checked` sugar over the signal + event pair | `value`, `checked`; group/select via the typed event target | Author-expressed with reactive signals/memos; no bundled validator | `on:submit` with `ev.prevent_default()`; read the model |
| **Svelte** (verified) | `bind:value`, `bind:checked`, `bind:group` | Text, checkbox, radio group, select (single + `multiple`), number (`bind:value` coerces) | Author or ecosystem (Felte, Superforms); constraint API available | `on:submit\|preventDefault`; read bound state |
| **Vue** (verified) | `v-model` with `.number` / `.trim` / `.lazy` modifiers | Adapts to text, checkbox, radio, select (single + multiple), number | Author or ecosystem (VeeValidate); none in core | `@submit.prevent`; read the reactive model |
| **Dioxus** | No bind shortcut — controlled input (`value` + `oninput` writing a signal) | Author wires per kind via the typed form event | Author-expressed over signals | `onsubmit` with `prevent_default`; read `FormData`/signals |
| **Solid** (JS analog) | No bind by default — controlled input (value + `onInput` setter) | Author wires per kind | Author-expressed over signals/stores | `onSubmit` with `preventDefault`; read the model |
| **Ferric today** (`276138f`) | **None** — and two-way was impossible | **None** | **None** | **None** — `submit` was not even bindable |

Verified against current sources: Leptos — the [Forms and Inputs](https://book.leptos.dev/view/05_forms.html)
chapter; Svelte — the [bindings tutorial](https://svelte.dev/tutorial/svelte/text-inputs) and
`bind:value`/`bind:checked`/`bind:group` docs; Vue — the
[Form Input Bindings](https://vuejs.org/guide/essentials/forms.html) guide (`v-model` and its
modifiers). The Dioxus and Solid rows state each framework's established public model and are
directional. The verified rows establish the pattern: a two-way `bind` is a convenience over the
controlled pair, core frameworks leave validation to the author or ecosystem, and submission runs
through the event with the default suppressed and reads the model. REQ-ASYNC-040..043 specify
exactly that shape for Ferric, composed additively on the payload seam.

---

## Metadata

| Field | Value |
|---|---|
| **Owning task** | `SUBTASK_async_forms_events` |
| **Assembled by** | `SUBTASK_afe_synthesis` |
| **Repo target** | `design/ferric/async-forms-events.md` |
| **Branch** | `work/design-ferric-1_0` |
| **Grounding commit** | `Nousix-LLC/nswe-demo` @ `276138f` (`crates/ferric/src/signal.rs`, `vdom.rs`, `dom.rs`) |
| **Conforms to** | `../../contracts/id_and_traceability_schema.yaml`, `../contracts/_MANIFEST.yaml` |

### Register summary

| Range | Capability | Count | Stage | All release-gating |
|---|---|---|---|---|
| REQ-ASYNC-001..016 | Async resources and suspense | 16 | usable_early (001–011, 014–016); one_oh (012–013) | true |
| REQ-ASYNC-020..024 | Full DOM event coverage + payload handler form | 5 | usable_early | true |
| REQ-ASYNC-040..043 | Forms and two-way input binding | 4 | usable_early | true |
| **Total** | | **25** | | **true** |

Sub-identities J1–J7 all pass (verified by `SUBTASK_afe_synthesis`; see `SYNTHESIS.md` for the
verdict record).

Unused headroom (reserved per `../contracts/_MANIFEST.yaml`): REQ-ASYNC-017..019,
REQ-ASYNC-025..039, REQ-ASYNC-044..059.
