# ferric

A small client-side web-application framework in Rust, compiled to WebAssembly.

`ferric` pairs **fine-grained reactivity** with a **component/view model** and a **`web-sys` browser
runtime**. You write components as plain Rust closures that read reactive signals and return a view
tree; `ferric` renders that tree to real DOM and, whenever a signal a component read changes,
re-renders it by applying a **minimal keyed diff** to the live DOM — not by rebuilding the subtree.

It is deliberately lean (`wasm-bindgen`, `web-sys`, `js-sys` — no heavy dependencies) and is the
foundation the rest of the engagement builds on.

## Architecture

```
          create_signal / create_memo / create_effect        (signal.rs)
                              │  reads subscribe, writes notify
                              ▼
   Component::render() ─► VNode tree ─► diff(old, new) ─► Vec<Patch>   (component.rs + vdom.rs)
                              │                               │
                              │  wrapped in a reactive effect │  applied in order
                              ▼                               ▼
                         mount(root, component) ──────► live DOM via web-sys   (dom.rs)
```

1. **Signals** ([`signal`]) are a Solid/Leptos-style reactive core: `Signal<T>` and `Memo<T>` are
   cheap `Copy` handles into a thread-local runtime. Reads inside an effect/memo subscribe
   automatically; writes notify subscribers; `batch` coalesces notifications; dropping a reactive
   scope unsubscribes everything it owns (no stale-subscriber leaks).
2. **Components & views** ([`component`] + [`vdom`]): a `Component` renders a `VNode` tree. Any
   `Fn() -> VNode` is a component. `VElement` carries a `key` so sibling lists reconcile by identity.
3. **Keyed diff** ([`vdom::diff`]): compares the previous and next view trees and emits a minimal,
   path-addressed `Patch` list (`SetText`, `SetAttr`, `InsertChild`, `MoveChild`, `RemoveChild`, …).
4. **DOM runtime** ([`dom::mount`]): wraps the render in `create_effect`, so the first render builds
   the DOM and every later render applies the keyed patch to the live DOM. `click` and `input`
   handlers carried on the view tree are bound to real DOM events.

## The counter

A full, runnable version lives in [`examples/counter.rs`](examples/counter.rs):

```rust
use ferric::prelude::*;
use ferric::vdom::text;

fn main() {
    let count = create_signal(0i32);

    // A function component: reading `count` subscribes the render effect.
    let app = move || -> VNode {
        VElement::new("div")
            .child(
                VElement::new("button")
                    .on(EventKind::Click, move || count.update(|n| *n += 1))
                    .child(text("increment")),
            )
            .child(VElement::new("p").child(text(format!("count: {}", count.get()))))
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

Clicking the button calls `count.update(…)`, which notifies the render effect; the effect re-renders,
the diff produces a single `SetText` patch for the `count: N` text node, and the runtime updates just
that node in place.

## Building

```bash
# Host build + the native test suite (reactive core + keyed diff) — no browser required:
cargo build --workspace
cargo test  --workspace

# Compile the framework and the counter example to WebAssembly:
rustup target add wasm32-unknown-unknown   # once
cargo build -p ferric --target wasm32-unknown-unknown
cargo build -p ferric --example counter --target wasm32-unknown-unknown
```

### Running the counter in a browser

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
trunk serve examples/counter.rs   # or: wasm-bindgen the built .wasm and serve the output
```

(`trunk` invokes the example's `main`, which mounts into `#app`.)

## Testing

- **Native tests are the gate.** `cargo test --workspace` exercises the reactive core (signals,
  memos, effects, batching, subscriber-cleanup soundness) and the keyed diff (via a shadow-DOM
  round-trip), none of which need a browser.
- **Browser tests** use `wasm-bindgen-test` and live behind `#[cfg(all(test, target_arch =
  "wasm32"))]`. They run only where a `wasm-bindgen-test-runner` and a headless browser are
  installed:

  ```bash
  cargo install wasm-bindgen-cli           # provides wasm-bindgen-test-runner
  wasm-pack test --headless --firefox      # or --chrome
  ```

## Notes & trade-offs

- **Event-handler lifetime.** Each bound DOM listener is a `wasm-bindgen` `Closure` kept alive for
  the element's lifetime (stashed on the element object and `forget()`-ed). The stash lets a re-bind
  detach the previous listener first, so re-binding never double-fires. This retains a bounded amount
  of memory per event binding — an intentional trade-off for a lean foundation; a future revision may
  use a per-element closure registry that drops closures deterministically.
- **`mount(root, component)`** realizes the frozen API's anticipated `root: web_sys::Element`
  parameter (an additive refinement of the mount surface). Off-wasm it is a no-op so the non-DOM
  layers stay natively testable.
- **Handlers** are `Fn()` (no event payload). A signal-based component reads its state live inside
  the handler, so most handlers need nothing from the event itself.

[`signal`]: src/signal.rs
[`component`]: src/component.rs
[`vdom`]: src/vdom.rs
[`vdom::diff`]: src/vdom.rs
[`dom::mount`]: src/dom.rs
