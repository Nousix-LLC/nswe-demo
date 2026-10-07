//! `/compose` — handle-based login and the validated compose-chirp form.
//!
//! **Scaffold stub.** Owned (initially) by `SUBTASK_scaffold` and **replaced** by
//! `SUBTASK_compose` with the real view: handle-based login that sets `ctx.session`, plus a
//! validated (`ChirpText`) compose form calling `create_chirp`, surfacing validation/API/network
//! errors. The stub renders a non-panicking placeholder so the walking skeleton mounts.

use ferric::vdom::{text, VElement, VNode};

use crate::app::AppContext;

/// `/compose` — login + compose. Realizes the frozen signature; replaced by `SUBTASK_compose`.
#[must_use]
pub fn compose_view(_ctx: &AppContext) -> VNode {
    VElement::new("section")
        .attr("class", "chirp-compose chirp-view-placeholder")
        .child(VElement::new("h1").child(text("Compose")))
        .child(VElement::new("p").child(text("The login + compose view is not implemented yet.")))
        .into()
}
