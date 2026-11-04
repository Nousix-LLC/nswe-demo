# ferric-router

A client-side router for the [`ferric`](../ferric) WebAssembly framework.

`ferric-router` adds **declarative routing** on top of `ferric`'s reactive core: a route table with
path parameters (`/user/:id`), a pure path→route matcher, History-API navigation
(`pushState`/`popstate`), programmatic `navigate(path)` / `navigate_replace(path)` APIs, a
`<Link>`-style navigation helper with active-route awareness, **nested routes** with shared layouts,
and a **reactive outlet** that re-renders the matched view whenever the path changes. It *composes*
`ferric`'s signals and component model — it does **not** re-implement reactivity or the DOM diff.

It is deliberately lean (`wasm-bindgen`, `web-sys`, `js-sys` — plus `ferric`) and layers cleanly over
the framework: the current path is a single `ferric` signal, so routing is just another reactive
input to your components.

## Architecture

```
   RoutePattern::new("/user/:id").match_path("/user/42")  ─►  Some(Params{ id: "42" })   (matcher.rs)
   RoutePattern::new("/settings").match_prefix("/settings/profile")
                                              ─►  Some((Params{}, "/profile"))
                              │  pure, browser-free, natively tested
                              ▼
   Router::new()
     .route(pattern, view)        ← flat routes (exact match)
     .nest(prefix, layout, child) ← nested routes (prefix match, tries first)
     .fallback(view)
     .mount_history()                                                                     (router.rs)
                              │
                              │  current path = one ferric Signal<String> (thread-local)
       navigate(path) ───────┤   pushState + signal.set    popstate listener ─► signal.set
       navigate_replace(path) ─┤  replaceState + signal.set
       current_path() ◄──────┘   reactive read (subscribes the caller)
                              ▼
   Router::outlet()  ─►  impl Component  ─►  reads current_path(), renders the matched route
                              │  mounted with ferric::mount; re-renders on every navigation
                              ▼
   link(to, label)  ─►  VNode — click-to-navigate + aria-current="page" on active link  (link.rs)
```

1. **Matcher** ([`matcher`](src/matcher.rs)) — `RoutePattern::new` compiles a `/`-separated pattern
   (static segments, `:name` params, and the root `/`); `match_path` matches a concrete path exactly
   over segments; `match_prefix` matches the pattern's segments against the beginning of a path and
   returns the remaining suffix — used by `Router::nest` for prefix-based nested routing. Pure and
   browser-free, so it is covered by ordinary native `cargo test`.
2. **Router** ([`router`](src/router.rs)) — a declarative route table (`route`, `fallback`, `nest`),
   the History-API integration (`mount_history` seeds the path from `window.location` and installs
   the `popstate` listener), programmatic `navigate(path)` (push) and `navigate_replace(path)`
   (replace), and `current_path()`. The "current path" is a single `ferric` `Signal<String>`;
   `navigate`/`navigate_replace` and `popstate` write it, `current_path` reads it.
3. **Outlet** ([`Router::outlet`](src/router.rs)) — consumes the router into a `ferric` `Component`
   that reads `current_path()` and renders the first matching route's view (or the fallback). Nested
   routes (registered with `nest`) are tried before flat routes. Because the read happens inside
   `ferric`'s render effect, every navigation re-renders the outlet via the keyed diff.
4. **Link** ([`link`](src/link.rs)) — `link(to, label)` returns a `VNode` that calls `navigate(to)`
   on click and adds `aria-current="page"` to the element when `to` matches `current_path()`. See
   [the `<Link>` and `preventDefault` note](#the-link-helper-and-the-preventdefault-constraint).

## `navigate` vs `navigate_replace`

`ferric-router` exposes two programmatic navigation functions:

| Function | History behaviour | Use when… |
|---|---|---|
| `navigate(path)` | Pushes a new entry — the user can press **back** to return | Normal in-app navigation |
| `navigate_replace(path)` | Replaces the current entry — **no back** to the replaced URL | Redirects, login flows, wizard steps where the intermediate URL should not be reachable via back |

```rust
use ferric_router::prelude::*;

// A "redirect home" button that leaves no back-step:
VElement::new("button")
    .on(EventKind::Click, || navigate_replace("/"))
    .child(text("Go home (replace)"))
```

Off-wasm (native unit tests), both functions skip the History API and update only the reactive
path signal — so the routing logic is fully testable without a browser.

## Nested routes with `Router::nest`

`Router::nest` lets you define a **shared layout** rendered for an entire subtree of paths. When
the current path begins with the prefix, the layout closure is called with the prefix params and a
`&dyn Component` for the child outlet, which renders the matched child route.

```rust
use ferric::component::Component;
use ferric::prelude::*;
use ferric::vdom::text;
use ferric_router::prelude::*;

// A shared layout for /settings/*.
fn settings_layout(_params: &Params, child_outlet: &dyn Component) -> VNode {
    VElement::new("section")
        .child(VElement::new("h1").child(text("Settings")))
        .child(
            VElement::new("nav")
                .child(link("/settings/", "Overview"))
                .child(link("/settings/profile", "Profile")),
        )
        .child(child_outlet.render())  // ← the matched child route renders here
        .into()
}

let router = Router::new()
    .route("/", |_| text("Home").into())
    .nest("/settings", settings_layout, |child_router| {
        child_router
            .route("/", |_| text("Settings index").into())   // /settings/
            .route("/profile", |_| text("Profile").into())   // /settings/profile
            .fallback(|_| text("Settings 404").into())
    })
    .fallback(|_| text("Not found").into())
    .mount_history();
```

**Key semantics**:

- Nested routes are tried **before** flat routes, in declaration order.
- The child router matches against the **remaining path** after the prefix (e.g. `/settings/profile`
  → remaining `/profile`; `/settings` or `/settings/` → remaining `/`).
- Each level has its own `.fallback()` — the child router's fallback handles unmatched child paths;
  the app-root router's fallback handles paths that don't match the prefix at all.
- The flat `Router::route` / `.outlet()` behaviour is the degenerate single-level case (unchanged).

## Active-link `aria-current` and styling the active link

Every `link(to, label)` call now emits `aria-current="page"` on the element when `to` equals the
current path. This happens at **build time** — when `Link::build()` / `link()` is called — so it
re-evaluates reactively on every navigation (links are normally built inside a render effect).

```rust
// In a nav bar rendered inside a reactive component:
link("/about", "About")   // → <span … aria-current="page"> when current path is /about
link("/home", "Home")     // → no aria-current when current path is not /home
```

Style the active link with CSS:

```css
/* Target the aria attribute directly */
[aria-current="page"] { font-weight: bold; text-decoration: underline; }

/* Or combine with the ferric-router class */
.ferric-router-link[aria-current="page"] { color: var(--accent); }
```

The inactive case is structurally identical to the frozen surface — no extra attribute, no class
change — so existing styles are unaffected.

## The example app

A full, runnable SPA lives in [`examples/app.rs`](examples/app.rs): routes `/`, `/about`,
`/user/:id`, and a nested `/settings/*` tree with `/settings/` and `/settings/profile` children,
a nav bar of `link`s with active-route awareness, a `navigate_replace` redirect button, and a
plain `navigate` button.

## The `<Link>` helper and the `preventDefault` constraint

A conventional SPA link is an `<a href="/path">` whose click handler calls `event.preventDefault()`
to suppress the browser's default full-page navigation, then routes on the client. **That approach
is not available here:** `ferric`'s click handler is a zero-argument `Fn()` — it receives no
`web_sys::Event`, so it *cannot* call `preventDefault()`.

Rather than fork `ferric` to pass the event (out of scope for this crate), `link` sidesteps default
navigation entirely by emitting a **non-anchor element** — a focusable, link-styled `<span
role="link" tabindex="0">` with no `href` for the browser to follow — carrying a `ferric` click
handler that calls `navigate(to)`. The user-visible behaviour matches a preventing `<a>`, with no
event object required. The destination is recorded in a `data-to` attribute (for
styling/testing/accessibility). Style in-app links via the `ferric-router-link` class (or supply
your own with the additive `Link` builder: `Link::new(to, label).class("…").build()`). If a future
`ferric` revision exposes the event payload to handlers, a true `<a href>` + `preventDefault()`
variant can be added additively without changing the frozen `link` signature.

## Building

```bash
# Host build + the native test suite (matcher + param extraction + navigation logic) — no browser:
cargo build --workspace
cargo test  -p ferric-router

# Compile the crate and the example app to WebAssembly:
rustup target add wasm32-unknown-unknown   # once
cargo build -p ferric-router --target wasm32-unknown-unknown
cargo build -p ferric-router --example app --target wasm32-unknown-unknown
```

### Running the example in a browser

The example compiles to a `.wasm` module; to run it you generate the JS bindings and load them from
an HTML page that provides a mount point. For example, with [`trunk`](https://trunkrs.dev):

```html
<!-- index.html -->
<!doctype html>
<html>
  <body>
    <div id="app"></div>
  </body>
</html>
```

```bash
trunk serve examples/app.rs   # or: wasm-bindgen the built .wasm and serve the output
```

(`trunk` invokes the example's `main`, which mounts into `#app`.)

## Testing

- **Native tests are the gate.** `cargo test -p ferric-router` exercises the matcher (static/param
  matching, exact-segment matching, param extraction, trailing-slash and doubled-slash normalization,
  prefix matching), the routing logic (`navigate`/`navigate_replace` → first-match resolution →
  fallback, nested route layout and child resolution, per-level fallback, `link` active-route
  awareness) — none of which need a browser.
- **Browser tests** use `wasm-bindgen-test` behind `#[cfg(all(test, target_arch = "wasm32"))]`:

  ```bash
  cargo install wasm-bindgen-cli           # provides wasm-bindgen-test-runner
  wasm-pack test --headless --firefox      # or --chrome
  ```

## Notes & trade-offs

- **Composition, not reimplementation.** The current path is a single `ferric` signal and the outlet
  is a plain `ferric` component; routing adds no reactivity or DOM machinery of its own.
- **Native vs. browser.** Every browser-only call (`window.location`, `history.pushState`, the
  `popstate` listener) is behind `#[cfg(target_arch = "wasm32")]`. Off-wasm, `navigate` and
  `navigate_replace` update only the reactive path, `mount_history` installs no listener — so the
  whole crate compiles and unit-tests natively.
- **First-match-wins.** Routes resolve in registration order within each level (nested or flat).
  Nested routes are always tried before flat routes.
- **`preventDefault` is unavailable** to click handlers (see above); `link` handles this by design
  with a non-anchor element — it is a known composition constraint, not a defect.
