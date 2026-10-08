//! The browser runtime: render a component's view tree into real DOM via `wasm-bindgen` +
//! `web-sys`, re-render through a keyed patch when signals change, and bind events.
//!
//! # How it fits together
//!
//! `ferric`'s layers compose here into a running application:
//!
//! 1. A [`Component`] renders a [`VNode`] tree ([`crate::vdom`]).
//! 2. [`mount`] wraps that render in a reactive effect ([`create_effect`]). Every
//!    [`Signal`](crate::signal::Signal) the render reads is tracked, so a later write re-runs the
//!    render.
//! 3. The first render builds real DOM nodes under the mount point. Each later render diffs the new
//!    view tree against the previous one ([`diff`]) and applies the resulting keyed [`Patch`] list
//!    to the *live* DOM — creating, updating, moving, and removing only the nodes that changed,
//!    rather than rebuilding the subtree.
//! 4. Event handlers carried on the view tree are bound to real DOM events (`click`, `input`).
//!
//! # Patch addressing
//!
//! The view layer addresses every [`Patch`] by a `path` — the sequence of child indices from the
//! root of the rendered tree (the empty path is the mounted root node). The runtime resolves a
//! path by walking `child_nodes()` from the single node it mounted under `root`, and it applies the
//! patch list strictly in order, exactly as the view layer documents.
//!
//! # Event-handler lifetime
//!
//! A DOM listener is a [`wasm_bindgen::closure::Closure`]. Each bound handler's closure is kept
//! alive for the element's lifetime by stashing it on the element object (via `Reflect`) and
//! calling [`Closure::forget`]; the stash also lets a later re-bind detach the previous listener
//! before adding the new one, so re-binding never double-fires. Because handlers in a signal-based
//! framework read their signals live (rather than capturing values), retaining a handler's closure
//! across a re-render is sound. This retains a bounded amount of memory per event binding — an
//! intentional, documented trade-off for a lean foundation runtime; a future revision may move to a
//! per-element closure registry that drops closures deterministically.
//!
//! The runtime calls into the DOM, so it is only meaningful in a browser/wasm context; `web-sys`
//! provides panicking stubs off-wasm, which keeps the crate compiling and the non-DOM layers
//! natively testable without a browser.

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

use crate::component::Component;
use crate::signal::create_effect;
use crate::vdom::{diff, EventHandler, EventKind, Patch, VNode};

/// Mount `component` into the real DOM element `root`, render it, and keep it reactively updated.
///
/// The runtime clears `root`, renders `component` into it, and wraps the render in
/// [`create_effect`] so that any [`Signal`](crate::signal::Signal) read during rendering re-runs
/// the render when it changes. Re-renders are applied as a minimal keyed [`diff`]/[`Patch`] against
/// the live DOM, and the handlers on the view tree are bound to `click` and `input` events.
///
/// `root` is the mount point (for example `document.body()` or an element retrieved by id); it is
/// emptied before the component is rendered into it.
///
/// This realizes the frozen contract's anticipated `root: web_sys::Element` parameter (an additive
/// owner-latitude refinement of the mount surface). It must run in a browser/wasm context: it calls
/// into the DOM through `web-sys`, which panics if invoked off-wasm.
pub fn mount(root: web_sys::Element, component: impl Component + 'static) {
    let document = web_sys::window()
        .expect("mount: no `window` — the runtime requires a browser context")
        .document()
        .expect("mount: no `document` on `window`");

    // Own the mount point: remove anything already there so our single rendered node is always
    // `root.first_child()`, which is what `resolve` walks from.
    while let Some(child) = root.first_child() {
        let _ = root.remove_child(&child);
    }

    // The previously rendered view tree, retained across effect runs so each run can diff against
    // it. `None` until the first render has run.
    let previous: Rc<RefCell<Option<VNode>>> = Rc::new(RefCell::new(None));

    create_effect(move || {
        // Render FIRST (this is the reactive read that subscribes the effect to its signals), then
        // touch `previous` — so no `RefCell` borrow is ever held across a signal read.
        let next = component.render();
        let mut slot = previous.borrow_mut();
        match slot.take() {
            None => {
                let node = create_node(&document, &next);
                root.append_child(&node)
                    .expect("mount: failed to append the rendered root node");
            }
            Some(previous_tree) => {
                let patches = diff(&previous_tree, &next);
                apply_patches(&document, &root, &patches);
            }
        }
        *slot = Some(next);
    });
}

// =====================================================================================
// DOM construction and patch application (these call into `web-sys`)
// =====================================================================================

/// Build a real DOM node from a view node, recursively, binding any events it carries.
fn create_node(document: &web_sys::Document, vnode: &VNode) -> web_sys::Node {
    match vnode {
        VNode::Text(content) => document.create_text_node(content).into(),
        VNode::Element(element) => {
            let dom = document
                .create_element(&element.tag)
                .expect("create_node: create_element failed");
            for (name, value) in &element.attrs {
                dom.set_attribute(name, value)
                    .expect("create_node: set_attribute failed");
            }
            for child in &element.children {
                let child_node = create_node(document, child);
                dom.append_child(&child_node)
                    .expect("create_node: append_child failed");
            }
            if !element.events.is_empty() {
                set_events(&dom, &element.events);
            }
            dom.into()
        }
    }
}

/// Apply a keyed patch list to the live DOM, in order.
fn apply_patches(document: &web_sys::Document, root: &web_sys::Element, patches: &[Patch]) {
    for patch in patches {
        match patch {
            Patch::Replace { path, node } => {
                let target = resolve(root, path);
                let parent = target
                    .parent_node()
                    .expect("apply: Replace target has no parent");
                let replacement = create_node(document, node);
                parent
                    .replace_child(&replacement, &target)
                    .expect("apply: replace_child failed");
            }
            Patch::SetText { path, value } => {
                resolve(root, path).set_text_content(Some(value));
            }
            Patch::SetAttr { path, name, value } => {
                resolve_element(root, path)
                    .set_attribute(name, value)
                    .expect("apply: set_attribute failed");
            }
            Patch::RemoveAttr { path, name } => {
                resolve_element(root, path)
                    .remove_attribute(name)
                    .expect("apply: remove_attribute failed");
            }
            Patch::SetEvents { path, events } => {
                set_events(&resolve_element(root, path), events);
            }
            Patch::InsertChild { path, index, node } => {
                let parent = resolve(root, path);
                let inserted = create_node(document, node);
                let reference = parent.child_nodes().item(*index as u32);
                parent
                    .insert_before(&inserted, reference.as_ref())
                    .expect("apply: insert_before failed");
            }
            Patch::RemoveChild { path, index } => {
                let parent = resolve(root, path);
                let child = parent
                    .child_nodes()
                    .item(*index as u32)
                    .expect("apply: RemoveChild index out of range");
                parent
                    .remove_child(&child)
                    .expect("apply: remove_child failed");
            }
            Patch::MoveChild { path, from, to } => {
                let parent = resolve(root, path);
                let node = parent
                    .child_nodes()
                    .item(*from as u32)
                    .expect("apply: MoveChild `from` out of range");
                parent
                    .remove_child(&node)
                    .expect("apply: move remove_child failed");
                // Re-read the (now shorter) child list and insert before `to`, matching the view
                // layer's remove-then-insert positioning semantics.
                let reference = parent.child_nodes().item(*to as u32);
                parent
                    .insert_before(&node, reference.as_ref())
                    .expect("apply: move insert_before failed");
            }
        }
    }
}

/// Walk from the mounted root node to the node addressed by `path`.
fn resolve(root: &web_sys::Element, path: &[usize]) -> web_sys::Node {
    let mut node: web_sys::Node = root
        .first_child()
        .expect("resolve: the mount point has no rendered node");
    for &index in path {
        node = node
            .child_nodes()
            .item(index as u32)
            .expect("resolve: missing child at path index");
    }
    node
}

/// Resolve a path to an element (patches that carry element-only operations always address an
/// element node).
fn resolve_element(root: &web_sys::Element, path: &[usize]) -> web_sys::Element {
    resolve(root, path)
        .dyn_into::<web_sys::Element>()
        .expect("resolve_element: node at path is not an element")
}

/// Bind `events` on `element`, first detaching any previously bound listeners for the (closed) set
/// of supported kinds so a re-bind never leaves a stale, double-firing listener.
fn set_events(element: &web_sys::Element, events: &[(EventKind, EventHandler)]) {
    let target: &JsValue = element.as_ref();

    // Detach anything we stashed previously.
    for kind in [EventKind::Click, EventKind::Input] {
        let key = JsValue::from_str(stash_key(kind));
        if let Ok(existing) = js_sys::Reflect::get(target, &key) {
            if existing.is_function() {
                let func = existing.unchecked_ref::<js_sys::Function>();
                let _ = element.remove_event_listener_with_callback(event_name(kind), func);
                let _ = js_sys::Reflect::delete_property(
                    element.unchecked_ref::<js_sys::Object>(),
                    &key,
                );
            }
        }
    }

    // Bind the current handlers.
    for (kind, handler) in events {
        let handler = handler.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            handler();
        }) as Box<dyn FnMut(web_sys::Event)>);
        let func = closure.as_ref().unchecked_ref::<js_sys::Function>();
        element
            .add_event_listener_with_callback(event_name(*kind), func)
            .expect("set_events: add_event_listener failed");
        // Keep the closure alive for the element's lifetime and make it retrievable for later
        // detachment.
        let _ = js_sys::Reflect::set(
            target,
            &JsValue::from_str(stash_key(*kind)),
            closure.as_ref(),
        );
        closure.forget();
    }
}

/// The DOM event name bound for an [`EventKind`].
fn event_name(kind: EventKind) -> &'static str {
    match kind {
        EventKind::Click => "click",
        EventKind::Input => "input",
    }
}

/// The element-object property under which a bound listener's closure is stashed, per [`EventKind`].
fn stash_key(kind: EventKind) -> &'static str {
    match kind {
        EventKind::Click => "__ferric_on_click",
        EventKind::Input => "__ferric_on_input",
    }
}

// =====================================================================================
// Native tests — exercise the pure, DOM-free helpers. The render/patch path itself is
// browser-only and is covered by the wasm-bindgen test below where the toolchain allows.
// =====================================================================================

#[cfg(test)]
mod tests {
    use super::{event_name, stash_key};
    use crate::vdom::EventKind;

    #[test]
    fn event_names_map_to_dom_event_types() {
        assert_eq!(event_name(EventKind::Click), "click");
        assert_eq!(event_name(EventKind::Input), "input");
    }

    #[test]
    fn stash_keys_are_distinct_per_kind() {
        assert_ne!(stash_key(EventKind::Click), stash_key(EventKind::Input));
    }
}

// =====================================================================================
// Browser test — compiles only for wasm32 and runs only where a wasm-bindgen test runner and a
// headless browser are available. It is NOT the crate's gate (the native tests are); it proves the
// end-to-end mount/render/patch path in a real DOM when the tooling exists.
// =====================================================================================

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::mount;
    use crate::prelude::*;
    use crate::vdom::text;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn mount_renders_initial_view_and_reacts_to_signal_change() {
        let document = web_sys::window().unwrap().document().unwrap();
        let root = document.create_element("div").unwrap();

        let count = create_signal(0i32);
        let view = move || -> VNode {
            VElement::new("button")
                .on(EventKind::Click, move || count.update(|n| *n += 1))
                .child(text(format!("count: {}", count.get())))
                .into()
        };

        mount(root.clone(), view);
        assert!(root.text_content().unwrap().contains("count: 0"));

        // A signal write re-renders through the keyed patch and updates the live DOM text.
        count.set(5);
        assert!(root.text_content().unwrap().contains("count: 5"));
    }
}
