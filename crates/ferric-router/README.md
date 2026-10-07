# ferric-router

A client-side router for the [`ferric`](../ferric) WebAssembly framework.

`ferric-router` adds **declarative routing** on top of `ferric`'s reactive core: a route table with
path parameters (`/user/:id`), a pure path→route matcher, History-API navigation
(`pushState`/`popstate`), a programmatic `navigate(path)` API, a `<Link>`-style navigation helper,
and a **reactive outlet** that re-renders the matched view whenever the path changes. It *composes*
`ferric`'s signals and component model — it does **not** re-implement reactivity or the DOM diff.

It is deliberately lean (`wasm-bindgen`, `web-sys`, `js-sys` — plus `ferric`) and layers cleanly over
the framework: the current path is a single `ferric` signal, so routing is just another reactive
input to your components.

## Architecture

```
   RoutePattern::new("/user/:id").match_path("/user/42")  ─►  Some(Params{ id: "42" })   (matcher.rs)
                              │  pure, browser-free, natively tested
                              ▼
   Router::new().route(pattern, view).fallback(view).mount_history()                      (router.rs)
                              │
                              │  current path = one ferric Signal<String> (thread-local)
       navigate(path) ───────┤   pushState + signal.set    popstate listener ─► signal.set
       current_path() ◄──────┘   reactive read (subscribes the caller)
                              ▼
   Router::outlet()  ─►  impl Component  ─►  reads current_path(), renders the matched route
                              │  mounted with ferric::mount; re-renders on every navigation
                              ▼
   link(to, label)  ─►  VNode (a click-to-navigate element) for declarative in-app navigation (link.rs)
```

1. **Matcher** ([`matcher`](src/matcher.rs)) — `RoutePattern::new` compiles a `/`-separated pattern
   (static segments, `:name` params, and the root `/`); `match_path` matches a concrete path exactly
   over segments and extracts named params into `Params`. Pure and browser-free, so it is covered by
   ordinary native `cargo test`.
2. **Router** ([`router`](src/router.rs)) — a declarative route table (`route`, `fallback`), the
   History-API integration (`mount_history` seeds the path from `window.location` and installs the
   `popstate` listener), programmatic `navigate(path)`, and `current_path()`. The "current path" is a
   single `ferric` `Signal<String>`; `navigate` and `popstate` write it, `current_path` reads it.
3. **Outlet** ([`Router::outlet`](src/router.rs)) — consumes the router into a `ferric` `Component`
   that reads `current_path()` and renders the first matching route's view (or the fallback). Because
   the read happens inside `ferric`'s render effect, every navigation re-renders the outlet via the
   keyed diff — only the routed view changes.
4. **Link** ([`link`](src/link.rs)) — `link(to, label)` returns a `VNode` that calls `navigate(to)`
   on click. See [the `<Link>` and `preventDefault` note](#the-link-helper-and-the-preventdefault-constraint).

## The example app

A full, runnable 2–3 route SPA lives in [`examples/app.rs`](examples/app.rs): routes `/`, `/about`,
and the parameterised `/user/:id`, a nav bar of `link`s, and a button that navigates programmatically.

```rust
use ferric::prelude::*;
use ferric::vdom::text;
use ferric_router::prelude::*;

fn user_view(params: &Params) -> VNode {
    let id = params.get("id").unwrap_or("unknown");
    VElement::new("section")
        .child(VElement::new("h1").child(text(format!("User {id}"))))
        .into()
}

fn main() {
    // A declarative route table; `mount_history` seeds the path and tracks back/forward.
    let router = Router::new()
        .route("/", |_params| text("Home").into())
        .route("/about", |_params| text("About").into())
        .route("/user/:id", user_view)
        .fallback(|_params| text("Not found").into())
        .mount_history();

    let outlet = router.outlet();

    // One root component: a persistent nav bar above the reactive outlet. Reading the outlet here
    // subscribes the mount effect to the current path, so a click re-renders the routed view.
    let app = move || -> VNode {
        VElement::new("div")
            .child(
                VElement::new("nav")
                    .child(link("/", "Home"))
                    .child(link("/about", "About"))
                    .child(link("/user/42", "User 42"))
                    .child(
                        VElement::new("button")
                            .on(EventKind::Click, || navigate("/user/7"))
                            .child(text("Go to user 7")),
                    ),
            )
            .child(outlet.render())
            .into()
    };

    let document = web_sys::window().unwrap().document().unwrap();
    let root = document
        .get_element_by_id("app")
        .or_else(|| document.body().map(Into::into))
        .unwrap();

    mount(root, app);
}
```

Clicking **Home** / **About** / **User 42** calls `navigate(...)`, which pushes a History entry and
sets the current-path signal; the outlet's render effect re-runs, the matcher resolves the new path,
and the keyed diff patches only the changed view. Browser back/forward fire `popstate`, which updates
the same signal — so the UI stays in sync with the address bar.

## The `<Link>` helper and the `preventDefault` constraint

A conventional SPA link is an `<a href="/path">` whose click handler calls `event.preventDefault()`
to suppress the browser's default full-page navigation, then routes on the client. **That approach is
not available here:** `ferric`'s click handler is a zero-argument `Fn()` — it receives no
`web_sys::Event`, so it *cannot* call `preventDefault()`.

Rather than fork `ferric` to pass the event (out of scope for this crate), `link` sidesteps default
navigation entirely by emitting a **non-anchor element** — a focusable, link-styled `<span role="link"
tabindex="0">` with no `href` for the browser to follow — carrying a `ferric` click handler that calls
`navigate(to)`. The user-visible behaviour matches a preventing `<a>`, with no event object required.
The destination is also recorded in a `data-to` attribute (for styling/testing/accessibility). Style
in-app links via the `ferric-router-link` class (or supply your own with the additive `Link` builder:
`Link::new(to, label).class("…").build()`). If a future `ferric` revision exposes the event payload to
handlers, a true `<a href>` + `preventDefault()` variant can be added additively without changing the
frozen `link` signature.

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
  matching, exact-segment matching, param extraction, trailing-slash and doubled-slash normalization)
  and the routing logic (`navigate` → first-match resolution → fallback, and the `link` view shape) —
  none of which need a browser, because the router updates its reactive path off-wasm without touching
  the History API.
- **Browser tests** use `wasm-bindgen-test` behind `#[cfg(all(test, target_arch = "wasm32"))]` and run
  only where a `wasm-bindgen-test-runner` and a headless browser are installed:

  ```bash
  cargo install wasm-bindgen-cli           # provides wasm-bindgen-test-runner
  wasm-pack test --headless --firefox      # or --chrome
  ```

## Notes & trade-offs

- **Composition, not reimplementation.** The current path is a single `ferric` signal and the outlet
  is a plain `ferric` component; routing adds no reactivity or DOM machinery of its own.
- **Native vs. browser.** Every browser-only call (`window.location`, `history.pushState`, the
  `popstate` listener) is behind `#[cfg(target_arch = "wasm32")]`. Off-wasm, `navigate` updates only
  the reactive path and `mount_history` installs no listener, so the whole crate compiles and
  unit-tests natively.
- **First-match-wins.** Routes resolve in registration order; register more specific patterns before
  more general ones.
- **`preventDefault` is unavailable** to click handlers (see above); `link` handles this by design
  with a non-anchor element — it is a known composition constraint, not a defect.
```
