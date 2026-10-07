//! `/user/:id` — the user profile view with follow/like actions.
//!
//! **Scaffold stub.** Owned (initially) by `SUBTASK_scaffold` and **replaced** by
//! `SUBTASK_profile` with the real view: reading `params.get("id")`, loading the `User`, showing
//! profile + counts, and offering follow/unfollow (`set_follow`) and like (`set_like`) actions with
//! reactive count updates. The stub renders a non-panicking placeholder so the walking skeleton
//! mounts.

use ferric::vdom::{text, VElement, VNode};
use ferric_router::prelude::Params;

use crate::app::AppContext;

/// `/user/:id` — profile. Realizes the frozen signature; replaced by `SUBTASK_profile`.
#[must_use]
pub fn profile_view(_ctx: &AppContext, _params: &Params) -> VNode {
    VElement::new("section")
        .attr("class", "chirp-profile chirp-view-placeholder")
        .child(VElement::new("h1").child(text("Profile")))
        .child(VElement::new("p").child(text("The profile view is not implemented yet.")))
        .into()
}
