//! The view tree a component renders, plus the keyed diff.
//!
//! A [`VNode`] is an element or a text node; [`VElement`] carries an optional [`Key`] so sibling
//! lists can be reconciled by identity. [`diff`] compares two trees and produces a minimal,
//! ordered [`Patch`] list that the DOM runtime (`src/dom.rs`) applies to the live DOM. Everything
//! in this module is pure and native-testable: it performs no `web-sys` calls, so the full view
//! tree and the diff are exercised by the `#[cfg(test)]` suite below with ordinary `cargo test`.
//!
//! # Patch addressing and application order
//!
//! Each [`Patch`] addresses a node by a `path`: the sequence of child indices from the root of the
//! tree passed to [`diff`] (the empty path is the root itself). The runtime MUST apply the patch
//! list **in order**; [`diff`] emits a parent's structural child edits (remove, then move/insert)
//! before the recursive edits of the retained children, so that every `path` resolves against the
//! node arrangement produced by the patches already applied. Index-based child edits for one parent
//! are emitted so that each index is valid against the evolving child list (removals descend; the
//! positioning pass walks left to right).
//!
//! # Event handlers
//!
//! [`EventHandler`] is an opaque `Rc<dyn Fn()>`; closures cannot be compared, so the diff cannot
//! minimise against handler identity. Retained elements re-bind their handlers (via [`Patch::SetEvents`])
//! only when the **set of bound [`EventKind`]s changes** — a no-op diff of structurally identical
//! trees therefore produces no patches even when handlers are present. Because handlers in a
//! signal-based framework read their signals live rather than capturing values, keeping a retained
//! element's prior closure across a re-render is sound; the DOM layer may additively enrich the
//! handler form (e.g. to carry a `web_sys::Event`) when it needs the event payload.

use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::rc::Rc;

/// A node in the view tree: an element or a text node.
#[derive(Clone, Debug)]
pub enum VNode {
    /// An element node.
    Element(VElement),
    /// A text node carrying its text content.
    Text(String),
}

/// An element node in the view tree. Fields beyond those named here are owner-designed; `key`
/// MUST exist to support keyed diffing of child lists.
#[derive(Clone)]
pub struct VElement {
    /// The element tag name (e.g. `"div"`, `"button"`).
    pub tag: String,
    /// Optional stable identity used when diffing sibling lists.
    pub key: Option<Key>,
    /// Attribute name/value pairs applied to the element.
    pub attrs: Vec<(String, String)>,
    /// Event bindings on this element.
    pub events: Vec<(EventKind, EventHandler)>,
    /// Child nodes, in order.
    pub children: Vec<VNode>,
}

/// Stable identity for keyed list diffing.
pub type Key = String;

/// Events the framework binds (acceptance criteria require at least click + input).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EventKind {
    /// A `click` event.
    Click,
    /// An `input` event.
    Input,
}

/// A boxed event callback. The concrete wrapper type is owner-designed; the frozen form is a
/// reference-counted zero-argument closure. The DOM layer may additively refine it to carry a
/// `web_sys::Event` when it needs the event payload.
pub type EventHandler = Rc<dyn Fn()>;

/// A single patch operation produced by the keyed [`diff`]. Each variant addresses a node by its
/// `path` (child indices from the root; empty = root). Patches are applied in list order.
#[derive(Clone)]
pub enum Patch {
    /// Replace the node at `path` wholesale. Emitted when the node kind changes (element↔text) or
    /// an element's tag differs.
    Replace {
        /// Path to the node being replaced.
        path: Vec<usize>,
        /// The replacement node.
        node: VNode,
    },
    /// Set the text content of the text node at `path`.
    SetText {
        /// Path to the text node.
        path: Vec<usize>,
        /// The new text content.
        value: String,
    },
    /// Add or overwrite an attribute on the element at `path`.
    SetAttr {
        /// Path to the element.
        path: Vec<usize>,
        /// Attribute name.
        name: String,
        /// Attribute value.
        value: String,
    },
    /// Remove an attribute from the element at `path`.
    RemoveAttr {
        /// Path to the element.
        path: Vec<usize>,
        /// Attribute name to remove.
        name: String,
    },
    /// Replace the event bindings on the element at `path`. Emitted when the set of bound
    /// [`EventKind`]s changes (see the module-level note on event handlers).
    SetEvents {
        /// Path to the element.
        path: Vec<usize>,
        /// The element's new event bindings.
        events: Vec<(EventKind, EventHandler)>,
    },
    /// Insert `node` as a new child at position `index` under the element at `path`.
    InsertChild {
        /// Path to the parent element.
        path: Vec<usize>,
        /// Destination index among the parent's children.
        index: usize,
        /// The node to insert.
        node: VNode,
    },
    /// Remove the child at position `index` under the element at `path`.
    RemoveChild {
        /// Path to the parent element.
        path: Vec<usize>,
        /// Index of the child to remove.
        index: usize,
    },
    /// Move a keyed child under the element at `path` from position `from` to position `to`.
    MoveChild {
        /// Path to the parent element.
        path: Vec<usize>,
        /// Current index of the child.
        from: usize,
        /// Destination index of the child.
        to: usize,
    },
}

// ---------------------------------------------------------------------------------------------
// Ergonomic builders (additive to the frozen surface) — enough to express SPA views by hand.
// ---------------------------------------------------------------------------------------------

/// Create a text [`VNode`].
///
/// ```
/// use ferric::prelude::*;
/// let node = ferric::vdom::text("hello");
/// ```
pub fn text(content: impl Into<String>) -> VNode {
    VNode::Text(content.into())
}

impl VElement {
    /// Create an element with the given tag and no key, attributes, events, or children.
    pub fn new(tag: impl Into<String>) -> Self {
        VElement {
            tag: tag.into(),
            key: None,
            attrs: Vec::new(),
            events: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Set the element's stable list-diffing key (builder style).
    #[must_use]
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Add an attribute (builder style).
    #[must_use]
    pub fn attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attrs.push((name.into(), value.into()));
        self
    }

    /// Bind an event handler (builder style).
    #[must_use]
    pub fn on(mut self, kind: EventKind, handler: impl Fn() + 'static) -> Self {
        self.events.push((kind, Rc::new(handler)));
        self
    }

    /// Append a child node (builder style).
    #[must_use]
    pub fn child(mut self, child: impl Into<VNode>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Append many child nodes (builder style).
    #[must_use]
    pub fn children(mut self, children: impl IntoIterator<Item = VNode>) -> Self {
        self.children.extend(children);
        self
    }
}

impl From<VElement> for VNode {
    fn from(element: VElement) -> Self {
        VNode::Element(element)
    }
}

// ---------------------------------------------------------------------------------------------
// Diff
// ---------------------------------------------------------------------------------------------

/// Compute the minimal keyed patch list to turn `old` into `new`. Pure and natively testable.
///
/// The returned patches, applied in order to a tree equal to `old`, produce a tree equal to `new`.
/// Element children are reconciled by [`Key`] when every child on both sides is a keyed element
/// (supporting insert / move / remove), and positionally otherwise.
///
/// ```
/// use ferric::prelude::*;
/// use ferric::vdom::{diff, text, VNode};
///
/// let old: VNode = VElement::new("p").child(text("a")).into();
/// let new: VNode = VElement::new("p").child(text("b")).into();
/// assert_eq!(diff(&old, &old).len(), 0); // no-op
/// assert_eq!(diff(&old, &new).len(), 1); // one text change
/// ```
pub fn diff(old: &VNode, new: &VNode) -> Vec<Patch> {
    let mut patches = Vec::new();
    diff_node(old, new, &[], &mut patches);
    patches
}

fn diff_node(old: &VNode, new: &VNode, path: &[usize], patches: &mut Vec<Patch>) {
    match (old, new) {
        (VNode::Text(a), VNode::Text(b)) => {
            if a != b {
                patches.push(Patch::SetText {
                    path: path.to_vec(),
                    value: b.clone(),
                });
            }
        }
        (VNode::Element(eo), VNode::Element(en)) if eo.tag == en.tag => {
            diff_attrs(eo, en, path, patches);
            diff_events(eo, en, path, patches);
            diff_children(&eo.children, &en.children, path, patches);
        }
        // Kind change (element↔text) or tag change: replace wholesale.
        _ => patches.push(Patch::Replace {
            path: path.to_vec(),
            node: new.clone(),
        }),
    }
}

fn diff_attrs(old: &VElement, new: &VElement, path: &[usize], patches: &mut Vec<Patch>) {
    let old_map: BTreeMap<&str, &str> = old
        .attrs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let new_map: BTreeMap<&str, &str> = new
        .attrs
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    for (name, value) in &new_map {
        if old_map.get(name) != Some(value) {
            patches.push(Patch::SetAttr {
                path: path.to_vec(),
                name: (*name).to_string(),
                value: (*value).to_string(),
            });
        }
    }
    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            patches.push(Patch::RemoveAttr {
                path: path.to_vec(),
                name: (*name).to_string(),
            });
        }
    }
}

fn diff_events(old: &VElement, new: &VElement, path: &[usize], patches: &mut Vec<Patch>) {
    let old_kinds: HashSet<EventKind> = old.events.iter().map(|(k, _)| *k).collect();
    let new_kinds: HashSet<EventKind> = new.events.iter().map(|(k, _)| *k).collect();
    if old_kinds != new_kinds {
        patches.push(Patch::SetEvents {
            path: path.to_vec(),
            events: new.events.clone(),
        });
    }
}

/// A child list is reconciled by key only when every child is a keyed element (an empty list
/// trivially satisfies this). Otherwise a positional diff is used.
fn all_keyed(children: &[VNode]) -> bool {
    children
        .iter()
        .all(|n| matches!(n, VNode::Element(e) if e.key.is_some()))
}

fn diff_children(old: &[VNode], new: &[VNode], path: &[usize], patches: &mut Vec<Patch>) {
    if all_keyed(old) && all_keyed(new) {
        diff_keyed_children(old, new, path, patches);
    } else {
        diff_positional_children(old, new, path, patches);
    }
}

fn diff_positional_children(
    old: &[VNode],
    new: &[VNode],
    path: &[usize],
    patches: &mut Vec<Patch>,
) {
    let min = old.len().min(new.len());
    // Patch the overlapping prefix in place first (structure unchanged, indices stable).
    for i in 0..min {
        diff_node(&old[i], &new[i], &child_path(path, i), patches);
    }
    // Append new trailing children (ascending: each insert targets the growing tail).
    for (i, node) in new.iter().enumerate().skip(min) {
        patches.push(Patch::InsertChild {
            path: path.to_vec(),
            index: i,
            node: node.clone(),
        });
    }
    // Drop old trailing children (descending: higher indices first keeps lower indices valid).
    for i in (min..old.len()).rev() {
        patches.push(Patch::RemoveChild {
            path: path.to_vec(),
            index: i,
        });
    }
}

fn key_of(node: &VNode) -> &Key {
    match node {
        VNode::Element(e) => e
            .key
            .as_ref()
            .expect("diff_keyed_children invoked on a non-keyed child"),
        VNode::Text(_) => panic!("diff_keyed_children invoked on a text child"),
    }
}

fn diff_keyed_children(old: &[VNode], new: &[VNode], path: &[usize], patches: &mut Vec<Patch>) {
    let old_keys: Vec<&Key> = old.iter().map(key_of).collect();
    let new_keys: Vec<&Key> = new.iter().map(key_of).collect();
    let new_key_set: HashSet<&Key> = new_keys.iter().copied().collect();

    // `working` mirrors the child list the runtime holds as patches are applied; it starts as the
    // old key order and is mutated by exactly the structural ops we emit, so the indices we compute
    // stay valid against the evolving list.
    let mut working: Vec<&Key> = old_keys.clone();

    // 1. Remove old children whose key is absent from `new` (descending index).
    for i in (0..working.len()).rev() {
        if !new_key_set.contains(working[i]) {
            patches.push(Patch::RemoveChild {
                path: path.to_vec(),
                index: i,
            });
            working.remove(i);
        }
    }

    // 2. Position pass, left to right: after step j, working[0..=j] == new_keys[0..=j].
    for (j, &k) in new_keys.iter().enumerate() {
        if let Some(cur) = working.iter().position(|&wk| wk == k) {
            if cur != j {
                patches.push(Patch::MoveChild {
                    path: path.to_vec(),
                    from: cur,
                    to: j,
                });
                let item = working.remove(cur);
                working.insert(j, item);
            }
        } else {
            patches.push(Patch::InsertChild {
                path: path.to_vec(),
                index: j,
                node: new[j].clone(),
            });
            working.insert(j, k);
        }
    }

    // 3. Recurse into retained (key-matched) children, now in their final positions.
    for (j, &k) in new_keys.iter().enumerate() {
        if let Some(oi) = old_keys.iter().position(|&ok| ok == k) {
            diff_node(&old[oi], &new[j], &child_path(path, j), patches);
        }
    }
}

fn child_path(path: &[usize], index: usize) -> Vec<usize> {
    let mut p = path.to_vec();
    p.push(index);
    p
}

// ---------------------------------------------------------------------------------------------
// Debug (manual, because event handlers are opaque closures)
// ---------------------------------------------------------------------------------------------

impl fmt::Debug for VElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VElement")
            .field("tag", &self.tag)
            .field("key", &self.key)
            .field("attrs", &self.attrs)
            .field("events", &event_kinds(&self.events))
            .field("children", &self.children)
            .finish()
    }
}

impl fmt::Debug for Patch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Patch::Replace { path, node } => f
                .debug_struct("Replace")
                .field("path", path)
                .field("node", node)
                .finish(),
            Patch::SetText { path, value } => f
                .debug_struct("SetText")
                .field("path", path)
                .field("value", value)
                .finish(),
            Patch::SetAttr { path, name, value } => f
                .debug_struct("SetAttr")
                .field("path", path)
                .field("name", name)
                .field("value", value)
                .finish(),
            Patch::RemoveAttr { path, name } => f
                .debug_struct("RemoveAttr")
                .field("path", path)
                .field("name", name)
                .finish(),
            Patch::SetEvents { path, events } => f
                .debug_struct("SetEvents")
                .field("path", path)
                .field("kinds", &event_kinds(events))
                .finish(),
            Patch::InsertChild { path, index, node } => f
                .debug_struct("InsertChild")
                .field("path", path)
                .field("index", index)
                .field("node", node)
                .finish(),
            Patch::RemoveChild { path, index } => f
                .debug_struct("RemoveChild")
                .field("path", path)
                .field("index", index)
                .finish(),
            Patch::MoveChild { path, from, to } => f
                .debug_struct("MoveChild")
                .field("path", path)
                .field("from", from)
                .field("to", to)
                .finish(),
        }
    }
}

fn event_kinds(events: &[(EventKind, EventHandler)]) -> Vec<EventKind> {
    events.iter().map(|(k, _)| *k).collect()
}

// ---------------------------------------------------------------------------------------------
// Tests — native, no browser required.
//
// Correctness is proven with a "shadow DOM": a structural mirror of the view tree that the patch
// list is applied to. For every (old, new) case, `apply(shadow(old), diff(old, new))` must equal
// `shadow(new)`. Targeted cases additionally assert the patch list is minimal.
// ---------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    // A canonicalised structural mirror of a VNode (closures dropped; attrs/event-kinds canonical).
    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Shadow {
        Text(String),
        Element {
            tag: String,
            key: Option<String>,
            attrs: BTreeMap<String, String>,
            events: BTreeSet<EventKind>,
            children: Vec<Shadow>,
        },
    }

    fn shadow(node: &VNode) -> Shadow {
        match node {
            VNode::Text(s) => Shadow::Text(s.clone()),
            VNode::Element(e) => Shadow::Element {
                tag: e.tag.clone(),
                key: e.key.clone(),
                attrs: e.attrs.iter().cloned().collect(),
                events: e.events.iter().map(|(k, _)| *k).collect(),
                children: e.children.iter().map(shadow).collect(),
            },
        }
    }

    fn node_at<'a>(root: &'a mut Shadow, path: &[usize]) -> &'a mut Shadow {
        let mut cur = root;
        for &i in path {
            match cur {
                Shadow::Element { children, .. } => cur = &mut children[i],
                Shadow::Text(_) => panic!("path descends into a text node"),
            }
        }
        cur
    }

    fn children_at<'a>(root: &'a mut Shadow, path: &[usize]) -> &'a mut Vec<Shadow> {
        match node_at(root, path) {
            Shadow::Element { children, .. } => children,
            Shadow::Text(_) => panic!("parent path is a text node"),
        }
    }

    fn apply(root: &mut Shadow, patches: &[Patch]) {
        for patch in patches {
            match patch {
                Patch::Replace { path, node } => *node_at(root, path) = shadow(node),
                Patch::SetText { path, value } => {
                    if let Shadow::Text(s) = node_at(root, path) {
                        *s = value.clone();
                    } else {
                        panic!("SetText on a non-text node");
                    }
                }
                Patch::SetAttr { path, name, value } => {
                    if let Shadow::Element { attrs, .. } = node_at(root, path) {
                        attrs.insert(name.clone(), value.clone());
                    } else {
                        panic!("SetAttr on a non-element");
                    }
                }
                Patch::RemoveAttr { path, name } => {
                    if let Shadow::Element { attrs, .. } = node_at(root, path) {
                        attrs.remove(name);
                    } else {
                        panic!("RemoveAttr on a non-element");
                    }
                }
                Patch::SetEvents { path, events } => {
                    if let Shadow::Element { events: ev, .. } = node_at(root, path) {
                        *ev = events.iter().map(|(k, _)| *k).collect();
                    } else {
                        panic!("SetEvents on a non-element");
                    }
                }
                Patch::InsertChild { path, index, node } => {
                    children_at(root, path).insert(*index, shadow(node));
                }
                Patch::RemoveChild { path, index } => {
                    children_at(root, path).remove(*index);
                }
                Patch::MoveChild { path, from, to } => {
                    let children = children_at(root, path);
                    let item = children.remove(*from);
                    children.insert(*to, item);
                }
            }
        }
    }

    /// The gold assertion: patching a mirror of `old` with `diff(old, new)` yields a mirror of `new`.
    fn assert_roundtrip(old: &VNode, new: &VNode) -> Vec<Patch> {
        let patches = diff(old, new);
        let mut s = shadow(old);
        apply(&mut s, &patches);
        assert_eq!(s, shadow(new), "patch list did not reconstruct `new`");
        patches
    }

    fn el(tag: &str) -> VElement {
        VElement::new(tag)
    }

    fn keyed(tag: &str, key: &str) -> VElement {
        VElement::new(tag).key(key)
    }

    #[test]
    fn no_op_equal_trees_produce_no_patches() {
        let tree: VNode = el("div")
            .attr("class", "box")
            .child(text("hello"))
            .child(el("span").child(text("world")))
            .into();
        let patches = assert_roundtrip(&tree, &tree.clone());
        assert!(patches.is_empty(), "expected no patches, got {patches:?}");
    }

    #[test]
    fn no_op_is_empty_even_with_event_handlers() {
        // Same set of bound kinds on both sides -> no SetEvents, despite distinct closures.
        let old: VNode = el("button").on(EventKind::Click, || {}).into();
        let new: VNode = el("button").on(EventKind::Click, || {}).into();
        let patches = assert_roundtrip(&old, &new);
        assert!(patches.is_empty(), "expected no patches, got {patches:?}");
    }

    #[test]
    fn text_change_produces_single_settext() {
        let old = text("a");
        let new = text("b");
        let patches = assert_roundtrip(&old, &new);
        assert_eq!(patches.len(), 1);
        assert!(
            matches!(&patches[0], Patch::SetText { path, value } if path.is_empty() && value == "b")
        );
    }

    #[test]
    fn attr_add_change_and_remove() {
        let old: VNode = el("div").attr("id", "x").attr("class", "a").into();
        let new: VNode = el("div").attr("id", "y").attr("role", "main").into();
        let patches = assert_roundtrip(&old, &new);
        // id changed -> SetAttr; role added -> SetAttr; class removed -> RemoveAttr.
        let set = patches
            .iter()
            .filter(|p| matches!(p, Patch::SetAttr { .. }))
            .count();
        let removed = patches
            .iter()
            .filter(|p| matches!(p, Patch::RemoveAttr { .. }))
            .count();
        assert_eq!(set, 2, "id change + role add");
        assert_eq!(removed, 1, "class removed");
    }

    #[test]
    fn tag_change_is_replace() {
        let old: VNode = el("div").child(text("x")).into();
        let new: VNode = el("span").child(text("x")).into();
        let patches = assert_roundtrip(&old, &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::Replace { path, .. } if path.is_empty()));
    }

    #[test]
    fn element_to_text_is_replace() {
        let old: VNode = el("div").into();
        let new = text("now text");
        let patches = assert_roundtrip(&old, &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::Replace { .. }));
    }

    #[test]
    fn event_binding_added_emits_setevents() {
        let old: VNode = el("button").into();
        let new: VNode = el("button").on(EventKind::Click, || {}).into();
        let patches = assert_roundtrip(&old, &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::SetEvents { path, events }
            if path.is_empty() && event_kinds(events) == vec![EventKind::Click]));
    }

    #[test]
    fn event_binding_removed_emits_setevents() {
        let old: VNode = el("input").on(EventKind::Input, || {}).into();
        let new: VNode = el("input").into();
        let patches = assert_roundtrip(&old, &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::SetEvents { events, .. } if events.is_empty()));
    }

    #[test]
    fn keyed_children_insert_in_middle() {
        let old: VNode = el("ul")
            .child(keyed("li", "a"))
            .child(keyed("li", "c"))
            .into();
        let new: VNode = el("ul")
            .child(keyed("li", "a"))
            .child(keyed("li", "b"))
            .child(keyed("li", "c"))
            .into();
        let patches = assert_roundtrip(&old, &new);
        let inserts = patches
            .iter()
            .filter(|p| matches!(p, Patch::InsertChild { .. }))
            .count();
        assert_eq!(inserts, 1, "exactly one child inserted");
        assert!(patches
            .iter()
            .any(|p| matches!(p, Patch::InsertChild { index, .. } if *index == 1)));
    }

    #[test]
    fn keyed_children_remove() {
        let old: VNode = el("ul")
            .child(keyed("li", "a"))
            .child(keyed("li", "b"))
            .child(keyed("li", "c"))
            .into();
        let new: VNode = el("ul")
            .child(keyed("li", "a"))
            .child(keyed("li", "c"))
            .into();
        let patches = assert_roundtrip(&old, &new);
        let removes = patches
            .iter()
            .filter(|p| matches!(p, Patch::RemoveChild { .. }))
            .count();
        assert_eq!(removes, 1, "exactly one child removed");
    }

    #[test]
    fn keyed_children_reorder_emits_moves_not_replaces() {
        let old: VNode = el("ul")
            .child(keyed("li", "a"))
            .child(keyed("li", "b"))
            .child(keyed("li", "c"))
            .into();
        // Reverse order; keyed diff must MOVE, never Replace keyed nodes.
        let new: VNode = el("ul")
            .child(keyed("li", "c"))
            .child(keyed("li", "b"))
            .child(keyed("li", "a"))
            .into();
        let patches = assert_roundtrip(&old, &new);
        assert!(
            patches.iter().any(|p| matches!(p, Patch::MoveChild { .. })),
            "reorder should emit at least one move"
        );
        assert!(
            !patches.iter().any(|p| matches!(p, Patch::Replace { .. })),
            "keyed reorder must not replace nodes"
        );
    }

    #[test]
    fn keyed_children_reorder_preserves_identity_and_patches_content() {
        // Node "a" moves AND its inner text changes: the move keeps identity, recursion patches it.
        let old: VNode = el("ul")
            .child(keyed("li", "a").child(text("A1")))
            .child(keyed("li", "b").child(text("B")))
            .into();
        let new: VNode = el("ul")
            .child(keyed("li", "b").child(text("B")))
            .child(keyed("li", "a").child(text("A2")))
            .into();
        let patches = assert_roundtrip(&old, &new);
        assert!(patches.iter().any(|p| matches!(p, Patch::MoveChild { .. })));
        assert!(patches.iter().any(|p| matches!(p, Patch::SetText { .. })));
    }

    #[test]
    fn unkeyed_children_diff_positionally() {
        let old: VNode = el("div")
            .child(text("one"))
            .child(el("span").child(text("two")))
            .into();
        let new: VNode = el("div")
            .child(text("ONE"))
            .child(el("span").child(text("two")))
            .child(text("three"))
            .into();
        let patches = assert_roundtrip(&old, &new);
        assert!(patches
            .iter()
            .any(|p| matches!(p, Patch::SetText { value, .. } if value == "ONE")));
        assert!(patches
            .iter()
            .any(|p| matches!(p, Patch::InsertChild { index, .. } if *index == 2)));
    }

    #[test]
    fn nested_recursive_diff_reaches_deep_nodes() {
        let old: VNode = el("section")
            .child(el("div").child(el("p").child(text("deep"))))
            .into();
        let new: VNode = el("section")
            .child(el("div").child(el("p").child(text("DEEP"))))
            .into();
        let patches = assert_roundtrip(&old, &new);
        assert_eq!(patches.len(), 1);
        assert!(matches!(&patches[0], Patch::SetText { path, value }
            if path == &vec![0usize, 0, 0] && value == "DEEP"));
    }

    #[test]
    fn keyed_move_insert_and_remove_combined() {
        // a,b,c,d  ->  d,b,e,a   (c removed, e inserted, a & d moved, b kept)
        let old: VNode = el("ul")
            .children(["a", "b", "c", "d"].map(|k| keyed("li", k).into()))
            .into();
        let new: VNode = el("ul")
            .children(["d", "b", "e", "a"].map(|k| keyed("li", k).into()))
            .into();
        let patches = assert_roundtrip(&old, &new);
        assert!(patches
            .iter()
            .any(|p| matches!(p, Patch::RemoveChild { .. })));
        assert!(patches
            .iter()
            .any(|p| matches!(p, Patch::InsertChild { .. })));
        assert!(patches.iter().any(|p| matches!(p, Patch::MoveChild { .. })));
    }

    #[test]
    fn full_list_clear_and_full_list_fill() {
        let full: VNode = el("ul")
            .children(["a", "b", "c"].map(|k| keyed("li", k).into()))
            .into();
        let empty: VNode = el("ul").into();
        // Clear: three removes, reconstructs empty.
        let clear = assert_roundtrip(&full, &empty);
        assert_eq!(
            clear
                .iter()
                .filter(|p| matches!(p, Patch::RemoveChild { .. }))
                .count(),
            3
        );
        // Fill: three inserts, reconstructs full.
        let fill = assert_roundtrip(&empty, &full);
        assert_eq!(
            fill.iter()
                .filter(|p| matches!(p, Patch::InsertChild { .. }))
                .count(),
            3
        );
    }

    #[test]
    fn builders_compose_expected_tree() {
        let node: VNode = el("a")
            .attr("href", "/x")
            .on(EventKind::Click, || {})
            .child(text("link"))
            .into();
        match node {
            VNode::Element(e) => {
                assert_eq!(e.tag, "a");
                assert_eq!(e.attrs, vec![("href".to_string(), "/x".to_string())]);
                assert_eq!(e.events.len(), 1);
                assert_eq!(e.children.len(), 1);
            }
            VNode::Text(_) => panic!("expected element"),
        }
    }
}
