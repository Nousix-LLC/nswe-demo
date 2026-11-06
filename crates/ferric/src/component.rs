//! The component abstraction producing a view tree: the [`Component`] trait plus function
//! components (any `Fn() -> VNode`).
//!
//! A component's [`render`](Component::render) produces a [`VNode`] tree. The DOM runtime invokes
//! `render` inside a reactive effect (`create_effect`), so any [`Signal`](crate::signal::Signal)
//! the render reads is tracked and a later change re-renders the component and patches the DOM.
//! Because function components are just closures, the common case — a view that reads some signals
//! and returns a tree — needs no boilerplate:
//!
//! ```
//! use ferric::prelude::*;
//! use ferric::vdom::text;
//!
//! let count = create_signal(0);
//! // A function component: any `Fn() -> VNode` is a `Component`.
//! let counter = move || -> VNode {
//!     VElement::new("button")
//!         .on(EventKind::Click, move || count.update(|n| *n += 1))
//!         .child(text(format!("count: {}", count.get())))
//!         .into()
//! };
//! // The runtime calls `render`; here we call it directly to show it yields a view tree.
//! let view: VNode = counter.render();
//! assert!(matches!(view, VNode::Element(_)));
//! ```

use crate::vdom::VNode;

/// A component renders itself to a view tree. `render` is re-invoked by the runtime (inside a
/// reactive effect) when the signals it reads change.
pub trait Component {
    /// Render this component to a view-tree node.
    fn render(&self) -> VNode;
}

/// Function components: any `Fn() -> VNode` is a [`Component`].
impl<F> Component for F
where
    F: Fn() -> VNode,
{
    fn render(&self) -> VNode {
        (self)()
    }
}

// ---------------------------------------------------------------------------
// Slot types (additive — nothing existing is redefined)
// ---------------------------------------------------------------------------

/// Deferred child content — a reference-counted thunk producing zero or more [`VNode`]s.
///
/// A `#[component]` function accepts child content by declaring a parameter named
/// `children: Children`. The caller fills it by nesting content inside the component
/// element in `view!`.
///
/// ```
/// use ferric::prelude::*;
///
/// // Children is Rc<dyn Fn() -> Vec<VNode>>; calling it re-computes the nodes.
/// let thunk: Children = std::rc::Rc::new(|| vec![]);
/// let nodes = thunk();
/// assert!(nodes.is_empty());
/// ```
pub type Children = ::std::rc::Rc<dyn Fn() -> Vec<VNode>>;

/// A named content region — same shape as [`Children`] but filled by label in `view!`.
///
/// A `#[component]` function accepts a named slot by declaring a parameter `name: Slot`.
/// The caller fills it via `<slot:name>…</slot:name>` inside the component element.
pub type Slot = ::std::rc::Rc<dyn Fn() -> Vec<VNode>>;

/// A parameterized content region — the render-prop pattern.
///
/// `SlotFn<T>` is the type for parameterized slots where the component passes a value
/// into the caller-provided content. Declare as `name: SlotFn<T>` in a `#[component]`
/// function. Full `view!` invocation for `SlotFn<T>` is a 1.0 feature; use at the
/// Rust level by passing an `Rc<dyn Fn(T) -> Vec<VNode>>` directly.
pub type SlotFn<T> = ::std::rc::Rc<dyn Fn(T) -> Vec<VNode>>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::create_signal;
    use crate::vdom::{text, EventKind, VElement};

    #[test]
    fn function_component_produces_view_tree() {
        let view = (|| -> VNode { VElement::new("div").child(text("hi")).into() }).render();
        match view {
            VNode::Element(e) => {
                assert_eq!(e.tag, "div");
                assert_eq!(e.children.len(), 1);
            }
            VNode::Text(_) => panic!("expected element"),
        }
    }

    #[test]
    fn function_component_reads_reactive_signal() {
        let count = create_signal(41);
        let comp = move || -> VNode { text(format!("n={}", count.get())) };
        assert!(matches!(comp.render(), VNode::Text(s) if s == "n=41"));
        count.set(42);
        // Re-rendering reflects the new signal value (the runtime does this inside an effect).
        assert!(matches!(comp.render(), VNode::Text(s) if s == "n=42"));
    }

    #[test]
    fn trait_object_component_renders() {
        struct Greeting {
            name: String,
        }
        impl Component for Greeting {
            fn render(&self) -> VNode {
                VElement::new("h1")
                    .child(text(format!("Hello, {}", self.name)))
                    .into()
            }
        }
        let g = Greeting {
            name: "ferric".to_string(),
        };
        // Exercise dynamic dispatch too: the trait is object-safe.
        let c: &dyn Component = &g;
        match c.render() {
            VNode::Element(e) => assert_eq!(e.tag, "h1"),
            VNode::Text(_) => panic!("expected element"),
        }
    }

    #[test]
    fn component_with_event_binding() {
        let clicks = create_signal(0);
        let button = move || -> VNode {
            VElement::new("button")
                .on(EventKind::Click, move || clicks.update(|n| *n += 1))
                .child(text("inc"))
                .into()
        };
        // The handler is wired; invoking it drives the signal (the DOM layer binds it to events).
        if let VNode::Element(e) = button.render() {
            assert_eq!(e.events.len(), 1);
            if let crate::vdom::EventHandler::NoArg(f) = &e.events[0].1 {
                f();
            } else {
                panic!("expected NoArg handler");
            }
        } else {
            panic!("expected element");
        }
        assert_eq!(clicks.get(), 1);
    }
}
