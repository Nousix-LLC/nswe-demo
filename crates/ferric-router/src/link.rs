//! Declarative in-app navigation helper producing a `ferric` view node.
//!
//! **ferric event constraint:** `ferric`'s click handler is `Fn()` with no event argument, so
//! `preventDefault()` is NOT reachable. [`link`] MUST therefore emit an element the browser will
//! not full-page-navigate on (e.g. a styled non-anchor element, or an `<a>` without a followed
//! href), wired with a click handler that calls [`navigate`](crate::router::navigate). The exact
//! element/markup is owner latitude. **Walking-skeleton stub:** the public surface below is frozen
//! by `contracts/ferric_router_api.rs`; the body is a `todo!()` placeholder filled in by the
//! `view` work-item.

use ferric::vdom::VNode;

/// A declarative in-app navigation link: returns a `ferric` [`VNode`] that, when clicked, calls
/// [`navigate`](crate::router::navigate) to `to` (client-side, no full page reload) and displays
/// `label`. Owners MAY add a richer builder (children, attributes) additively.
pub fn link(to: &str, label: &str) -> VNode {
    let _ = (to, label);
    todo!()
}
