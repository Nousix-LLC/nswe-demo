//! A minimal `ferric` counter, built for the browser.
//!
//! It creates a reactive `count` signal and a function component that reads it; `ferric::mount`
// renders the component into the page and re-renders (via the keyed diff) whenever `count` changes.
// Clicking the button increments the signal, and the displayed count updates in place.
//
// Build it for wasm:
//
//     cargo build -p ferric --example counter --target wasm32-unknown-unknown
//
// To actually run it in a browser, generate the JS bindings with `wasm-bindgen` (or use `trunk`)
// and load the module from an HTML page that provides a mount point — see `crates/ferric/README.md`.

use ferric::prelude::*;
use ferric::vdom::text;

fn main() {
    // Reactive state: the current count.
    let count = create_signal(0i32);

    // A function component. Reading `count` inside the render subscribes the mount effect, so each
    // click's `update` re-renders this view and patches the DOM.
    let app = move || -> VNode {
        VElement::new("div")
            .attr("class", "counter")
            .child(
                VElement::new("button")
                    .on(EventKind::Click, move || count.update(|n| *n += 1))
                    .child(text("increment")),
            )
            .child(VElement::new("p").child(text(format!("count: {}", count.get()))))
            .into()
    };

    // Mount into `#app` if the page provides it, otherwise into `<body>`.
    let document = web_sys::window()
        .expect("no browser `window`")
        .document()
        .expect("no `document`");
    let root = document
        .get_element_by_id("app")
        .or_else(|| document.body().map(Into::into))
        .expect("the page has no `#app` element and no `<body>` to mount into");

    mount(root, app);
}
