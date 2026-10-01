//! The view tree a component renders, plus the keyed diff.
//!
//! [`diff`] produces a [`Patch`] list that the DOM runtime (`src/dom.rs`) consumes to mutate the
//! real DOM. The view→dom edge is serialized, so the runtime reads the real `Patch` enum; its
//! variants are the owner's design.
//!
//! STUB: the public surface below is frozen by `contracts/ferric_api.rs`; the view tree and the
//! keyed diff are implemented by `SUBTASK_view_and_diff`, which replaces the `todo!()` body here.

use std::rc::Rc;

/// A node in the view tree: an element or a text node.
pub enum VNode {
    /// An element node.
    Element(VElement),
    /// A text node carrying its text content.
    Text(String),
}

/// An element node in the view tree. Fields beyond those named here are owner-designed; `key`
/// MUST exist to support keyed diffing of child lists.
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
pub enum EventKind {
    /// A `click` event.
    Click,
    /// An `input` event.
    Input,
}

/// A boxed event callback. The concrete wrapper type is owner-designed (e.g. the DOM layer may
/// refine it to carry a `web_sys::Event`); the frozen form is a reference-counted closure.
pub type EventHandler = Rc<dyn Fn()>;

/// A single patch operation produced by the keyed diff. Variants are owner-designed (e.g.
/// `ReplaceText`, `SetAttr`, `RemoveAttr`, `InsertChild`, `MoveChild`, `RemoveChild`).
pub enum Patch {}

/// Compute the minimal keyed patch list to turn `old` into `new`. Pure and natively testable.
pub fn diff(old: &VNode, new: &VNode) -> Vec<Patch> {
    let _ = (old, new);
    todo!()
}
