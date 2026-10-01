//! The component abstraction producing a view tree: the [`Component`] trait plus function
//! components (any `Fn() -> VNode`).
//!
//! STUB: the public surface below is frozen by `contracts/ferric_api.rs`. The trait and the
//! function-component blanket impl are complete as frozen; richer component machinery (if any) is
//! owned by `SUBTASK_view_and_diff`.

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
