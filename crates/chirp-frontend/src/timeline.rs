//! `/` — the timeline view.
//!
//! **Scaffold stub.** Owned (initially) by `SUBTASK_scaffold` and **replaced** by
//! `SUBTASK_timeline` with the real view: loading a `Page<Chirp>` via `ctx.client().timeline(..)`,
//! rendering a keyed reactive list with loading/error/empty states and cursor pagination. The stub
//! renders a non-panicking placeholder so the walking skeleton mounts.

use ferric::vdom::{text, VElement, VNode};

use crate::app::AppContext;

/// `/` — timeline. Realizes the frozen signature; replaced by `SUBTASK_timeline`.
#[must_use]
pub fn timeline_view(_ctx: &AppContext) -> VNode {
    VElement::new("section")
        .attr("class", "chirp-timeline chirp-view-placeholder")
        .child(VElement::new("h1").child(text("Timeline")))
        .child(VElement::new("p").child(text("The timeline view is not implemented yet.")))
        .into()
}
