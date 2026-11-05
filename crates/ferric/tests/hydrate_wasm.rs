//! Integration tests for `ferric::hydrate` — WASM-only, run under `wasm-pack test --headless`.
//!
//! Covers three acceptance criteria:
//!
//! * REQ-SSR-004 / #224 — adopt walk without structural mutations
//! * REQ-SSR-005 / #224 — event wiring preserved post-hydration
//! * REQ-SSR-006 / #226 — mismatch detection + DOM recovery

#![cfg(target_arch = "wasm32")]

use ferric::signal::create_signal;
use ferric::vdom::VElement;
use ferric::{EventKind, VNode};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// ── Helpers ──────────────────────────────────────────────────────────────────

fn document() -> web_sys::Document {
    web_sys::window()
        .expect("no window — tests must run in a browser context")
        .document()
        .expect("window has no document")
}

/// Create a fresh `<div>` container, attach it to `<body>`, and return it.
/// Must be paired with `cleanup` to avoid leaking between tests.
fn make_container() -> web_sys::Element {
    let doc = document();
    let container = doc
        .create_element("div")
        .expect("create_element(\"div\") failed");
    doc.body()
        .expect("document has no body")
        .append_child(&container)
        .expect("append_child into body failed");
    container
}

/// Remove `el` from the live document so successive tests start with a clean DOM.
fn cleanup(el: &web_sys::Element) {
    if let Some(parent) = el.parent_node() {
        let _ = parent.remove_child(el);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// REQ-SSR-004 / #224 — adopt walk without structural mutations.
///
/// Build a minimal SSR HTML string, inject it into a container, hydrate with
/// a structurally-identical component, and assert that the pre-existing DOM nodes
/// are adopted in-place — node identity is preserved and no `appendChild` /
/// `replaceChild` mutation occurred during the adopt walk.
#[wasm_bindgen_test]
fn hydrate_adopt_walk_no_structural_mutations() {
    let container = make_container();

    // Inject SSR markup: <div><span>hello</span></div>
    container.set_inner_html("<div><span>hello</span></div>");

    // Capture node identities before hydration.
    let before_div = container
        .first_child()
        .expect("container should have a <div> child after set_inner_html");
    let before_span = before_div
        .first_child()
        .expect("<div> should have a <span> child");

    // Hydrate with a structurally-identical component (same tags, same text).
    ferric::hydrate(container.clone(), move || {
        VNode::Element(VElement::new("div").child(VNode::Element(
            VElement::new("span").child(VNode::Text("hello".into())),
        )))
    });

    // The <div> must be the same DOM node — not recreated.
    let after_div = container
        .first_child()
        .expect("container should still have a child after hydration");
    assert!(
        before_div.is_same_node(Some(&after_div)),
        "REQ-SSR-004: <div> was recreated during the adopt walk (structural mutation detected)"
    );

    // The <span> must also be the same DOM node.
    let after_span = after_div
        .first_child()
        .expect("<div> should still have a child after hydration");
    assert!(
        before_span.is_same_node(Some(&after_span)),
        "REQ-SSR-004: <span> was recreated during the adopt walk (structural mutation detected)"
    );

    cleanup(&container);
}

/// REQ-SSR-005 / #224 — event wiring preserved post-hydration.
///
/// After hydration, dispatch a `click` event on the wired button and assert:
/// 1. The DOM text reflects the signal-driven re-render (button label becomes "1").
/// 2. The button's node identity is preserved — the element was patched, not recreated.
#[wasm_bindgen_test]
fn hydrate_event_wiring_post_hydration() {
    let container = make_container();

    // Create a reactive counter; initial value matches the SSR markup ("0").
    let count = create_signal(0_i32);

    // Inject SSR markup matching the component's initial render.
    container.set_inner_html("<button>0</button>");

    // Capture button identity before hydration.
    let before_button = container
        .first_child()
        .expect("container should have a <button> after set_inner_html");

    // Hydrate: the component reads `count` (subscribing the reactive effect) and
    // wires a click handler that increments the counter.
    ferric::hydrate(container.clone(), move || {
        let val = count.get(); // reactive subscription
        VNode::Element(
            VElement::new("button")
                .on(EventKind::Click, move || {
                    count.set(val + 1);
                })
                .child(VNode::Text(val.to_string())),
        )
    });

    // Dispatch a programmatic click — the signal setter fires, the effect re-renders
    // synchronously, and the DOM is patched before `click()` returns.
    container
        .first_child()
        .expect("container should have a <button> before click")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("first child should be an HtmlElement")
        .click();

    // Assert DOM text updated to "1" (signal-driven re-render applied).
    let after_button = container
        .first_child()
        .expect("container should still have a child after click");
    assert_eq!(
        after_button.text_content().as_deref(),
        Some("1"),
        "REQ-SSR-005: button text did not update to \"1\" after click"
    );

    // Assert button node identity preserved (element was patched, not replaced).
    assert!(
        before_button.is_same_node(Some(&after_button)),
        "REQ-SSR-005: button node was recreated after click — element identity not preserved"
    );

    cleanup(&container);
}

/// REQ-SSR-006 / #226 — mismatch detection and DOM recovery.
///
/// Build a server HTML string with a deliberate structural divergence (server renders
/// `<div>`, component returns `<p>`), call `hydrate`, and assert that the live DOM
/// matches the client-rendered component (`<p>`, not `<div>`).
///
/// Note: `emit_mismatch_warning` is `cfg(debug_assertions)`-gated; this test verifies
/// DOM recovery rather than the console output.
#[wasm_bindgen_test]
fn hydrate_mismatch_detection_recovery() {
    let container = make_container();

    // Server HTML diverges from the component: server has <div>, component returns <p>.
    container.set_inner_html("<div>text</div>");

    // Hydrate with a component that returns <p> — a tag mismatch with the server <div>.
    ferric::hydrate(container.clone(), || {
        VNode::Element(VElement::new("p").child(VNode::Text("text".into())))
    });

    // After mismatch recovery the live DOM must reflect the client component: <p>, not <div>.
    let first_child = container
        .first_child()
        .expect("container should have a child after hydration");
    let el = first_child
        .dyn_ref::<web_sys::Element>()
        .expect("first child should be an Element");
    assert_eq!(
        el.tag_name().to_lowercase(),
        "p",
        "REQ-SSR-006: mismatch recovery did not replace server <div> with client <p>"
    );

    cleanup(&container);
}
