//! Client-side hydration — adopt a server-rendered DOM subtree without structural mutations,
//! wire event handlers from the VNode tree, and establish a reactive diff/patch effect loop.
//!
//! # Hydration protocol
//!
//! 1. **Adopt pass** (`hydrate` → `adopt_root_vnode` → `adopt_children`):
//!    Walk the existing DOM against the VNode tree returned by `f`.  Comment sentinels
//!    (`TEXT_OPEN_MARKER`, `TEXT_CLOSE_MARKER`) are skipped — they are not VNode children.
//!    Matching elements have their event handlers wired (REQ-SSR-005).  Mismatched nodes emit a
//!    `console.warn` (debug builds only, REQ-SSR-009) and are recovered by client-rendering the
//!    mismatched subtree.  No structural DOM mutation occurs when every node matches (REQ-SSR-004).
//!
//! 2. **Sentinel cleanup** (`strip_sentinels`):
//!    After the adopt walk the TEXT_OPEN_MARKER / TEXT_CLOSE_MARKER Comment nodes are removed so
//!    DOM child indices align with VNode path indices for diff/patch passes.
//!
//! 3. **Reactive effects**:
//!    `create_effect` stores the adopted tree as "previous".  On every subsequent signal-driven
//!    re-render `f` is called again, `vdom::diff` produces the patch list, and
//!    `apply_patches_impl` applies it — identical to `dom::mount`'s reactive-update path.

#![cfg(target_arch = "wasm32")]

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

use crate::signal::create_effect;
// TEXT_OPEN_MARKER / TEXT_CLOSE_MARKER imported from crate::ssr (marker-format-agreement).
use crate::ssr::{TEXT_CLOSE_MARKER, TEXT_OPEN_MARKER};
use crate::vdom::{diff, EventHandler, EventKind, Patch, VNode};

/// Adopt a server-rendered DOM subtree rooted at `root`, wire event handlers, and establish
/// a reactive effect that keeps the DOM up-to-date when signals change.
///
/// `f` must be the same component closure used for the server-side `render_to_string` call
/// that produced the HTML now under `root`.
pub fn hydrate<F>(root: web_sys::Element, f: F)
where
    F: Fn() -> VNode + 'static,
{
    let document = web_sys::window()
        .expect("hydrate: no `window` — requires a browser context")
        .document()
        .expect("hydrate: no `document` on `window`");

    let previous: Rc<RefCell<Option<VNode>>> = Rc::new(RefCell::new(None));

    create_effect(move || {
        // Render first — this is the reactive read that tracks signal subscriptions.
        let next = f();
        let mut slot = previous.borrow_mut();
        match slot.take() {
            None => {
                // First run: adopt the server-rendered DOM (zero structural mutations on match).
                adopt_root_vnode(&document, &root, &next);
                // Strip TEXT_OPEN_MARKER / TEXT_CLOSE_MARKER sentinel Comment nodes so the
                // DOM child indices align with VNode path indices for diff/patch passes.
                strip_sentinels(&root);
            }
            Some(prev) => {
                // Subsequent runs: standard keyed diff/patch update (identical to dom::mount).
                let patches = diff(&prev, &next);
                apply_patches_impl(&document, &root, &patches);
            }
        }
        *slot = Some(next);
    });
}

// ── Adopt pass ────────────────────────────────────────────────────────────────

/// Adopt the top-level VNode against `root.first_child()`.
fn adopt_root_vnode(document: &web_sys::Document, root: &web_sys::Element, vnode: &VNode) {
    match root.first_child() {
        Some(dom_node) => adopt_vnode(document, root, &dom_node, vnode, &[]),
        None => {
            // DOM is empty — mismatch recovery: client-render and append.
            emit_mismatch_warning(&[], &vnode_label(vnode), "absent");
            let new_node = create_node(document, vnode);
            root.append_child(&new_node)
                .expect("adopt_root_vnode: append_child failed in recovery");
        }
    }
}

/// Adopt `vnode` against the existing `dom_node`, wiring events on match and recovering by
/// client-rendering on mismatch.  `parent` is needed for replacement; `path` is for warnings.
fn adopt_vnode(
    document: &web_sys::Document,
    parent: &web_sys::Element,
    dom_node: &web_sys::Node,
    vnode: &VNode,
    path: &[usize],
) {
    match vnode {
        VNode::Element(ve) => {
            match dom_node.dyn_ref::<web_sys::Element>() {
                Some(el) if el.tag_name().eq_ignore_ascii_case(&ve.tag) => {
                    // Tag matches — wire events and recurse into children.
                    if !ve.events.is_empty() {
                        set_events(el, &ve.events);
                    }
                    adopt_children(document, el, &ve.children, path);
                }
                other => {
                    // Tag mismatch or non-element in place of expected element.
                    let found = other
                        .map(|e| format!("<{}>", e.tag_name().to_lowercase()))
                        .unwrap_or_else(|| node_type_label(dom_node));
                    emit_mismatch_warning(path, &format!("<{}>", ve.tag), &found);
                    let new_node = create_node(document, vnode);
                    parent
                        .replace_child(&new_node, dom_node)
                        .expect("adopt_vnode: replace_child failed in element-mismatch recovery");
                }
            }
        }

        VNode::Text(text) => {
            match dom_node.dyn_ref::<web_sys::Text>() {
                Some(t) if t.data() == *text => {
                    // Perfect text match — nothing to do.
                }
                Some(t) => {
                    // Text content mismatch — recover in-place (not a structural mutation).
                    let found = t.data();
                    emit_mismatch_warning(path, &format!("{text:?}"), &format!("{found:?}"));
                    t.set_data(text);
                }
                None => {
                    // Expected a Text node, found something else.
                    let found = node_type_label(dom_node);
                    emit_mismatch_warning(path, &format!("{text:?}"), &found);
                    let new_text = document.create_text_node(text);
                    parent
                        .replace_child(&new_text, dom_node)
                        .expect("adopt_vnode: replace_child failed in text-mismatch recovery");
                }
            }
        }
    }
}

/// Walk `parent_dom.child_nodes()`, skipping TEXT_OPEN/CLOSE sentinel Comment nodes,
/// and adopt each non-sentinel DOM node against the corresponding VNode child.
fn adopt_children(
    document: &web_sys::Document,
    parent_dom: &web_sys::Element,
    vnode_children: &[VNode],
    parent_path: &[usize],
) {
    let dom_children = parent_dom.child_nodes();
    let n_dom = dom_children.length();
    let mut vnode_idx: usize = 0;

    for dom_i in 0..n_dom {
        let dom_node = match dom_children.item(dom_i) {
            Some(n) => n,
            None => break,
        };

        // Skip TEXT_OPEN_MARKER / TEXT_CLOSE_MARKER sentinel Comment nodes — not VNode children.
        if is_sentinel_comment(&dom_node) {
            continue;
        }

        if vnode_idx >= vnode_children.len() {
            // Extra DOM nodes beyond the VNode child list — only warn, do not remove.
            let found = node_type_label(&dom_node);
            let mut child_path = parent_path.to_vec();
            child_path.push(vnode_idx);
            emit_mismatch_warning(&child_path, "absent (extra DOM node)", &found);
            break;
        }

        let vnode = &vnode_children[vnode_idx];
        let mut child_path = parent_path.to_vec();
        child_path.push(vnode_idx);

        adopt_vnode(document, parent_dom, &dom_node, vnode, &child_path);
        vnode_idx += 1;
    }

    // VNode children with no corresponding DOM node — mismatch recovery: append client-rendered.
    for missing_idx in vnode_idx..vnode_children.len() {
        let vnode = &vnode_children[missing_idx];
        let mut child_path = parent_path.to_vec();
        child_path.push(missing_idx);
        emit_mismatch_warning(&child_path, &vnode_label(vnode), "absent");
        let new_node = create_node(document, vnode);
        parent_dom
            .append_child(&new_node)
            .expect("adopt_children: append_child failed in missing-child recovery");
    }
}

// ── Sentinel cleanup ──────────────────────────────────────────────────────────

/// Remove all TEXT_OPEN_MARKER / TEXT_CLOSE_MARKER sentinel Comment nodes from the subtree
/// rooted at `element`, recursively.  After this, DOM child indices match VNode path indices.
fn strip_sentinels(element: &web_sys::Element) {
    // Collect sentinel nodes first — cannot remove during live NodeList iteration.
    let children = element.child_nodes();
    let n = children.length();
    let mut sentinels: Vec<web_sys::Node> = Vec::new();
    for i in 0..n {
        if let Some(node) = children.item(i) {
            if is_sentinel_comment(&node) {
                sentinels.push(node);
            }
        }
    }
    for node in &sentinels {
        let _ = element.remove_child(node);
    }
    // Recurse into child elements (re-read the NodeList after sentinel removals).
    let children = element.child_nodes();
    let n = children.length();
    for i in 0..n {
        if let Some(node) = children.item(i) {
            if let Some(child_el) = node.dyn_ref::<web_sys::Element>() {
                strip_sentinels(child_el);
            }
        }
    }
}

/// Returns `true` if `node` is a Comment node whose data equals a sentinel marker.
///
/// `comment.data()` returns the content between `<!--` and `-->`.  So `TEXT_OPEN_MARKER`
/// (`"<!--$t-->"`) has data `"$t"` and `TEXT_CLOSE_MARKER` (`"<!--/$t-->"`) has data `"/$t"`.
fn is_sentinel_comment(node: &web_sys::Node) -> bool {
    if node.node_type() != web_sys::Node::COMMENT_NODE {
        return false;
    }
    let Some(comment) = node.dyn_ref::<web_sys::Comment>() else {
        return false;
    };
    let data = comment.data();
    // Strip "<!--" prefix (4 bytes) and "-->" suffix (3 bytes) from each marker constant.
    let open_data = TEXT_OPEN_MARKER
        .strip_prefix("<!--")
        .and_then(|s| s.strip_suffix("-->"))
        .unwrap_or("");
    let close_data = TEXT_CLOSE_MARKER
        .strip_prefix("<!--")
        .and_then(|s| s.strip_suffix("-->"))
        .unwrap_or("");
    data == open_data || data == close_data
}

// ── Mismatch diagnostics ──────────────────────────────────────────────────────

/// Emit a `console.warn` in debug builds when a VNode/DOM mismatch is detected (REQ-SSR-009).
#[cfg(debug_assertions)]
fn emit_mismatch_warning(path: &[usize], expected: &str, found: &str) {
    let path_string = path
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    web_sys::console::warn_1(
        &format!(
            "ferric hydration mismatch at [{}]: expected {:?}, found {:?}",
            path_string, expected, found
        )
        .into(),
    );
}

#[cfg(not(debug_assertions))]
#[inline]
fn emit_mismatch_warning(_path: &[usize], _expected: &str, _found: &str) {}

fn vnode_label(vnode: &VNode) -> String {
    match vnode {
        VNode::Element(ve) => format!("<{}>", ve.tag),
        VNode::Text(t) => format!("{t:?}"),
    }
}

fn node_type_label(node: &web_sys::Node) -> String {
    match node.node_type() {
        web_sys::Node::ELEMENT_NODE => node
            .dyn_ref::<web_sys::Element>()
            .map(|e| format!("<{}>", e.tag_name().to_lowercase()))
            .unwrap_or_else(|| "element".to_string()),
        web_sys::Node::TEXT_NODE => "text".to_string(),
        web_sys::Node::COMMENT_NODE => "comment".to_string(),
        _ => "unknown".to_string(),
    }
}

// ── DOM construction (mismatch recovery) ─────────────────────────────────────

/// Build a real DOM node from a VNode.  Used during mismatch recovery.
fn create_node(document: &web_sys::Document, vnode: &VNode) -> web_sys::Node {
    match vnode {
        VNode::Text(content) => document.create_text_node(content).into(),
        VNode::Element(ve) => {
            let dom = document
                .create_element(&ve.tag)
                .expect("create_node: create_element failed");
            for (name, value) in &ve.attrs {
                dom.set_attribute(name, value)
                    .expect("create_node: set_attribute failed");
            }
            for child in &ve.children {
                let child_node = create_node(document, child);
                dom.append_child(&child_node)
                    .expect("create_node: append_child failed");
            }
            if !ve.events.is_empty() {
                set_events(&dom, &ve.events);
            }
            dom.into()
        }
    }
}

// ── Patch application (reactive update path) ─────────────────────────────────

/// Apply a keyed patch list to the live DOM.  Mirrors `dom::apply_patches` exactly.
fn apply_patches_impl(document: &web_sys::Document, root: &web_sys::Element, patches: &[Patch]) {
    for patch in patches {
        match patch {
            Patch::Replace { path, node } => {
                let target = resolve_impl(root, path);
                let parent = target
                    .parent_node()
                    .expect("apply_patches_impl: Replace target has no parent");
                let replacement = create_node(document, node);
                parent
                    .replace_child(&replacement, &target)
                    .expect("apply_patches_impl: replace_child failed");
            }
            Patch::SetText { path, value } => {
                resolve_impl(root, path).set_text_content(Some(value));
            }
            Patch::SetAttr { path, name, value } => {
                resolve_element_impl(root, path)
                    .set_attribute(name, value)
                    .expect("apply_patches_impl: set_attribute failed");
            }
            Patch::RemoveAttr { path, name } => {
                resolve_element_impl(root, path)
                    .remove_attribute(name)
                    .expect("apply_patches_impl: remove_attribute failed");
            }
            Patch::SetEvents { path, events } => {
                set_events(&resolve_element_impl(root, path), events);
            }
            Patch::InsertChild { path, index, node } => {
                let parent = resolve_impl(root, path);
                let inserted = create_node(document, node);
                let reference = parent.child_nodes().item(*index as u32);
                parent
                    .insert_before(&inserted, reference.as_ref())
                    .expect("apply_patches_impl: insert_before failed");
            }
            Patch::RemoveChild { path, index } => {
                let parent = resolve_impl(root, path);
                let child = parent
                    .child_nodes()
                    .item(*index as u32)
                    .expect("apply_patches_impl: RemoveChild index out of range");
                parent
                    .remove_child(&child)
                    .expect("apply_patches_impl: remove_child failed");
            }
            Patch::MoveChild { path, from, to } => {
                let parent = resolve_impl(root, path);
                let node = parent
                    .child_nodes()
                    .item(*from as u32)
                    .expect("apply_patches_impl: MoveChild `from` out of range");
                parent
                    .remove_child(&node)
                    .expect("apply_patches_impl: move remove_child failed");
                let reference = parent.child_nodes().item(*to as u32);
                parent
                    .insert_before(&node, reference.as_ref())
                    .expect("apply_patches_impl: move insert_before failed");
            }
        }
    }
}

/// Walk from the mounted root node to the node addressed by `path`.  Mirrors `dom::resolve`.
fn resolve_impl(root: &web_sys::Element, path: &[usize]) -> web_sys::Node {
    let mut node: web_sys::Node = root
        .first_child()
        .expect("resolve_impl: mount point has no rendered node");
    for &index in path {
        node = node
            .child_nodes()
            .item(index as u32)
            .expect("resolve_impl: missing child at path index");
    }
    node
}

/// Resolve a path to an element.  Mirrors `dom::resolve_element`.
fn resolve_element_impl(root: &web_sys::Element, path: &[usize]) -> web_sys::Element {
    resolve_impl(root, path)
        .dyn_into::<web_sys::Element>()
        .expect("resolve_element_impl: node at path is not an element")
}

// ── Event wiring (mirrors dom::set_events, REQ-SSR-005) ──────────────────────

/// Wire `events` on `element`, detaching any previously stashed listeners first.
/// Uses the same Reflect-based approach as `dom::set_events` with the same stash keys.
fn set_events(element: &web_sys::Element, events: &[(EventKind, EventHandler)]) {
    let target: &JsValue = element.as_ref();

    // Detach anything previously stashed.
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
        let _ = js_sys::Reflect::set(
            target,
            &JsValue::from_str(stash_key(*kind)),
            closure.as_ref(),
        );
        closure.forget();
    }
}

fn event_name(kind: EventKind) -> &'static str {
    match kind {
        EventKind::Click => "click",
        EventKind::Input => "input",
    }
}

fn stash_key(kind: EventKind) -> &'static str {
    match kind {
        EventKind::Click => "__ferric_on_click",
        EventKind::Input => "__ferric_on_input",
    }
}
