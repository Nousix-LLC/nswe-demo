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
//! 4. Event handlers carried on the view tree are bound to real DOM events.
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
/// the render. Re-renders are applied as a minimal keyed [`diff`]/[`Patch`] against the live DOM,
/// and the handlers on the view tree are bound to real DOM events.
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

/// Bind `events` on `element`, first detaching any previously bound listeners so a re-bind never
/// leaves a stale, double-firing listener.
///
/// # Detachment strategy
///
/// For every static (non-`Custom`) [`EventKind`] variant: look up the stash key on the element
/// and, if a previously-leaked closure is found there, remove its listener and delete the property.
///
/// For `Custom` event names: a previous pass may have recorded the bound custom names as a
/// `js_sys::Array` under `"__ferric_custom_events"`. We read that list, remove each named
/// listener, and delete the recorded list before writing the new one.
fn set_events(element: &web_sys::Element, events: &[(EventKind, EventHandler)]) {
    let target: &JsValue = element.as_ref();
    let element_obj = element.unchecked_ref::<js_sys::Object>();

    // --- 1. Detach all static-kind previously-bound listeners ---
    for kind in static_event_kinds() {
        let key_js = JsValue::from_str(&stash_key(&kind));
        if let Ok(existing) = js_sys::Reflect::get(target, &key_js) {
            if existing.is_function() {
                let func = existing.unchecked_ref::<js_sys::Function>();
                let en = event_name(&kind);
                let _ = element.remove_event_listener_with_callback(&en, func);
                let _ = js_sys::Reflect::delete_property(element_obj, &key_js);
            }
        }
    }

    // --- 2. Detach any previously-bound Custom listeners recorded on the element ---
    let custom_list_key = JsValue::from_str("__ferric_custom_events");
    if let Ok(custom_list) = js_sys::Reflect::get(target, &custom_list_key) {
        if let Some(array) = custom_list.dyn_ref::<js_sys::Array>() {
            for i in 0..array.length() {
                if let Some(name) = array.get(i).as_string() {
                    let sk_js = JsValue::from_str(&format!("__ferric_on_custom_{name}"));
                    if let Ok(existing) = js_sys::Reflect::get(target, &sk_js) {
                        if existing.is_function() {
                            let func = existing.unchecked_ref::<js_sys::Function>();
                            let _ = element.remove_event_listener_with_callback(&name, func);
                            let _ = js_sys::Reflect::delete_property(element_obj, &sk_js);
                        }
                    }
                }
            }
            let _ = js_sys::Reflect::delete_property(element_obj, &custom_list_key);
        }
    }

    // --- 3. Bind the current handlers ---
    // Track any Custom event names bound in this pass so we can detach them on the next re-bind.
    let mut custom_names: Vec<String> = Vec::new();

    for (kind, handler) in events {
        let handler = handler.clone();
        let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            handler();
        }) as Box<dyn FnMut(web_sys::Event)>);
        let func = closure.as_ref().unchecked_ref::<js_sys::Function>();
        let en = event_name(kind);
        element
            .add_event_listener_with_callback(&en, func)
            .expect("set_events: add_event_listener failed");
        // Stash the closure under its key so a later re-bind or detach can find it.
        let _ = js_sys::Reflect::set(
            target,
            &JsValue::from_str(&stash_key(kind)),
            closure.as_ref(),
        );
        closure.forget();

        if let EventKind::Custom(name) = kind {
            custom_names.push(name.clone());
        }
    }

    // --- 4. Record any Custom event names bound in this pass for future detachment ---
    if !custom_names.is_empty() {
        let arr = js_sys::Array::new();
        for name in &custom_names {
            arr.push(&JsValue::from_str(name));
        }
        let _ = js_sys::Reflect::set(target, &custom_list_key, &arr);
    }
}

/// Returns all static (non-[`EventKind::Custom`]) variants.
///
/// Used by [`set_events`] to enumerate every possible stash slot during the detachment pass. The
/// array is exhaustive by construction: `static_event_kinds` and `event_name` / `stash_key` must
/// be kept in sync when new variants are added to `EventKind`.
fn static_event_kinds() -> [EventKind; 47] {
    [
        // Mouse
        EventKind::Click,
        EventKind::DblClick,
        EventKind::MouseDown,
        EventKind::MouseUp,
        EventKind::MouseEnter,
        EventKind::MouseLeave,
        EventKind::MouseMove,
        EventKind::MouseOver,
        EventKind::MouseOut,
        EventKind::ContextMenu,
        // Keyboard
        EventKind::KeyDown,
        EventKind::KeyUp,
        EventKind::KeyPress,
        // Input / focus
        EventKind::Input,
        EventKind::Change,
        EventKind::Focus,
        EventKind::Blur,
        // Form
        EventKind::Submit,
        EventKind::Reset,
        // Pointer
        EventKind::PointerDown,
        EventKind::PointerUp,
        EventKind::PointerEnter,
        EventKind::PointerLeave,
        EventKind::PointerMove,
        EventKind::PointerCancel,
        // Scroll / resize
        EventKind::Scroll,
        EventKind::Resize,
        // Drag
        EventKind::DragStart,
        EventKind::Drag,
        EventKind::DragEnd,
        EventKind::DragEnter,
        EventKind::DragLeave,
        EventKind::DragOver,
        EventKind::Drop,
        // Touch
        EventKind::TouchStart,
        EventKind::TouchEnd,
        EventKind::TouchMove,
        EventKind::TouchCancel,
        // Clipboard
        EventKind::Copy,
        EventKind::Cut,
        EventKind::Paste,
        // Animation / transition
        EventKind::AnimationStart,
        EventKind::AnimationEnd,
        EventKind::AnimationIteration,
        EventKind::TransitionEnd,
        // Load
        EventKind::Load,
        EventKind::Error,
    ]
}

/// Returns the DOM event-listener type string for each [`EventKind`].
///
/// The `match` is exhaustive — no wildcard — so the compiler reports an error if a new variant is
/// added to the enum without updating this table.
fn event_name(kind: &EventKind) -> String {
    match kind {
        // Mouse
        EventKind::Click => "click".into(),
        EventKind::DblClick => "dblclick".into(),
        EventKind::MouseDown => "mousedown".into(),
        EventKind::MouseUp => "mouseup".into(),
        EventKind::MouseEnter => "mouseenter".into(),
        EventKind::MouseLeave => "mouseleave".into(),
        EventKind::MouseMove => "mousemove".into(),
        EventKind::MouseOver => "mouseover".into(),
        EventKind::MouseOut => "mouseout".into(),
        EventKind::ContextMenu => "contextmenu".into(),
        // Keyboard
        EventKind::KeyDown => "keydown".into(),
        EventKind::KeyUp => "keyup".into(),
        EventKind::KeyPress => "keypress".into(),
        // Input / focus
        EventKind::Input => "input".into(),
        EventKind::Change => "change".into(),
        EventKind::Focus => "focus".into(),
        EventKind::Blur => "blur".into(),
        // Form
        EventKind::Submit => "submit".into(),
        EventKind::Reset => "reset".into(),
        // Pointer
        EventKind::PointerDown => "pointerdown".into(),
        EventKind::PointerUp => "pointerup".into(),
        EventKind::PointerEnter => "pointerenter".into(),
        EventKind::PointerLeave => "pointerleave".into(),
        EventKind::PointerMove => "pointermove".into(),
        EventKind::PointerCancel => "pointercancel".into(),
        // Scroll / resize
        EventKind::Scroll => "scroll".into(),
        EventKind::Resize => "resize".into(),
        // Drag
        EventKind::DragStart => "dragstart".into(),
        EventKind::Drag => "drag".into(),
        EventKind::DragEnd => "dragend".into(),
        EventKind::DragEnter => "dragenter".into(),
        EventKind::DragLeave => "dragleave".into(),
        EventKind::DragOver => "dragover".into(),
        EventKind::Drop => "drop".into(),
        // Touch
        EventKind::TouchStart => "touchstart".into(),
        EventKind::TouchEnd => "touchend".into(),
        EventKind::TouchMove => "touchmove".into(),
        EventKind::TouchCancel => "touchcancel".into(),
        // Clipboard
        EventKind::Copy => "copy".into(),
        EventKind::Cut => "cut".into(),
        EventKind::Paste => "paste".into(),
        // Animation / transition
        EventKind::AnimationStart => "animationstart".into(),
        EventKind::AnimationEnd => "animationend".into(),
        EventKind::AnimationIteration => "animationiteration".into(),
        EventKind::TransitionEnd => "transitionend".into(),
        // Load
        EventKind::Load => "load".into(),
        EventKind::Error => "error".into(),
        // Custom escape hatch: use the contained string as-is.
        EventKind::Custom(s) => s.clone(),
    }
}

/// The element-object property key under which a bound listener's [`Closure`] is stashed, so a
/// later re-bind can detach the previous listener before adding the new one.
///
/// Keys are prefixed `__ferric_on_` and are distinct per variant (exhaustive `match`; no
/// wildcard). `Custom` variants append `custom_<event_name>` so each distinct custom event gets
/// its own slot and cannot collide with any static key.
fn stash_key(kind: &EventKind) -> String {
    match kind {
        // Mouse
        EventKind::Click => "__ferric_on_click".into(),
        EventKind::DblClick => "__ferric_on_dblclick".into(),
        EventKind::MouseDown => "__ferric_on_mousedown".into(),
        EventKind::MouseUp => "__ferric_on_mouseup".into(),
        EventKind::MouseEnter => "__ferric_on_mouseenter".into(),
        EventKind::MouseLeave => "__ferric_on_mouseleave".into(),
        EventKind::MouseMove => "__ferric_on_mousemove".into(),
        EventKind::MouseOver => "__ferric_on_mouseover".into(),
        EventKind::MouseOut => "__ferric_on_mouseout".into(),
        EventKind::ContextMenu => "__ferric_on_contextmenu".into(),
        // Keyboard
        EventKind::KeyDown => "__ferric_on_keydown".into(),
        EventKind::KeyUp => "__ferric_on_keyup".into(),
        EventKind::KeyPress => "__ferric_on_keypress".into(),
        // Input / focus
        EventKind::Input => "__ferric_on_input".into(),
        EventKind::Change => "__ferric_on_change".into(),
        EventKind::Focus => "__ferric_on_focus".into(),
        EventKind::Blur => "__ferric_on_blur".into(),
        // Form
        EventKind::Submit => "__ferric_on_submit".into(),
        EventKind::Reset => "__ferric_on_reset".into(),
        // Pointer
        EventKind::PointerDown => "__ferric_on_pointerdown".into(),
        EventKind::PointerUp => "__ferric_on_pointerup".into(),
        EventKind::PointerEnter => "__ferric_on_pointerenter".into(),
        EventKind::PointerLeave => "__ferric_on_pointerleave".into(),
        EventKind::PointerMove => "__ferric_on_pointermove".into(),
        EventKind::PointerCancel => "__ferric_on_pointercancel".into(),
        // Scroll / resize
        EventKind::Scroll => "__ferric_on_scroll".into(),
        EventKind::Resize => "__ferric_on_resize".into(),
        // Drag
        EventKind::DragStart => "__ferric_on_dragstart".into(),
        EventKind::Drag => "__ferric_on_drag".into(),
        EventKind::DragEnd => "__ferric_on_dragend".into(),
        EventKind::DragEnter => "__ferric_on_dragenter".into(),
        EventKind::DragLeave => "__ferric_on_dragleave".into(),
        EventKind::DragOver => "__ferric_on_dragover".into(),
        EventKind::Drop => "__ferric_on_drop".into(),
        // Touch
        EventKind::TouchStart => "__ferric_on_touchstart".into(),
        EventKind::TouchEnd => "__ferric_on_touchend".into(),
        EventKind::TouchMove => "__ferric_on_touchmove".into(),
        EventKind::TouchCancel => "__ferric_on_touchcancel".into(),
        // Clipboard
        EventKind::Copy => "__ferric_on_copy".into(),
        EventKind::Cut => "__ferric_on_cut".into(),
        EventKind::Paste => "__ferric_on_paste".into(),
        // Animation / transition
        EventKind::AnimationStart => "__ferric_on_animationstart".into(),
        EventKind::AnimationEnd => "__ferric_on_animationend".into(),
        EventKind::AnimationIteration => "__ferric_on_animationiteration".into(),
        EventKind::TransitionEnd => "__ferric_on_transitionend".into(),
        // Load
        EventKind::Load => "__ferric_on_load".into(),
        EventKind::Error => "__ferric_on_error".into(),
        // Custom: prefix with `custom_` to prevent collisions with any static key.
        // e.g. Custom("click") → "__ferric_on_custom_click" ≠ "__ferric_on_click"
        EventKind::Custom(s) => format!("__ferric_on_custom_{s}"),
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
        // Original two variants preserved exactly.
        assert_eq!(event_name(&EventKind::Click), "click");
        assert_eq!(event_name(&EventKind::Input), "input");
        // Spot-check one variant from each newly added category.
        assert_eq!(event_name(&EventKind::DblClick), "dblclick");
        assert_eq!(event_name(&EventKind::Change), "change");
        assert_eq!(event_name(&EventKind::KeyDown), "keydown");
        assert_eq!(event_name(&EventKind::KeyUp), "keyup");
        assert_eq!(event_name(&EventKind::KeyPress), "keypress");
        assert_eq!(event_name(&EventKind::Focus), "focus");
        assert_eq!(event_name(&EventKind::Blur), "blur");
        assert_eq!(event_name(&EventKind::Submit), "submit");
        assert_eq!(event_name(&EventKind::Reset), "reset");
        assert_eq!(event_name(&EventKind::PointerDown), "pointerdown");
        assert_eq!(event_name(&EventKind::PointerMove), "pointermove");
        assert_eq!(event_name(&EventKind::PointerCancel), "pointercancel");
        assert_eq!(event_name(&EventKind::Scroll), "scroll");
        assert_eq!(event_name(&EventKind::Resize), "resize");
        assert_eq!(event_name(&EventKind::DragStart), "dragstart");
        assert_eq!(event_name(&EventKind::Drag), "drag");
        assert_eq!(event_name(&EventKind::Drop), "drop");
        assert_eq!(event_name(&EventKind::TouchStart), "touchstart");
        assert_eq!(event_name(&EventKind::TouchEnd), "touchend");
        assert_eq!(event_name(&EventKind::Copy), "copy");
        assert_eq!(event_name(&EventKind::Cut), "cut");
        assert_eq!(event_name(&EventKind::Paste), "paste");
        assert_eq!(event_name(&EventKind::AnimationStart), "animationstart");
        assert_eq!(event_name(&EventKind::AnimationEnd), "animationend");
        assert_eq!(
            event_name(&EventKind::AnimationIteration),
            "animationiteration"
        );
        assert_eq!(event_name(&EventKind::TransitionEnd), "transitionend");
        assert_eq!(event_name(&EventKind::Load), "load");
        assert_eq!(event_name(&EventKind::Error), "error");
        // Custom variant uses the contained string directly.
        assert_eq!(
            event_name(&EventKind::Custom("my-event".to_owned())),
            "my-event"
        );
    }

    #[test]
    fn stash_keys_are_distinct_per_kind() {
        // Original assertion preserved.
        assert_ne!(stash_key(&EventKind::Click), stash_key(&EventKind::Input));
        // Different Custom names produce different keys.
        assert_ne!(
            stash_key(&EventKind::Custom("alpha".to_owned())),
            stash_key(&EventKind::Custom("beta".to_owned())),
        );
        // A Custom whose name matches a static event name does NOT collide with that static key.
        assert_ne!(
            stash_key(&EventKind::Custom("click".to_owned())),
            stash_key(&EventKind::Click),
        );
        assert_ne!(
            stash_key(&EventKind::Custom("input".to_owned())),
            stash_key(&EventKind::Input),
        );
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
