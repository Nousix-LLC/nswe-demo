//! Fixture: omit a required named slot.
//!
//! `Page` declares `header: Slot` as a required slot.  Calling
//! `PageProps::new().build()` without filling the slot must be a
//! compile-time error naming the missing slot and the component.

use ferric::prelude::*;

#[component]
fn Page(header: Slot) -> VNode {
    view! {
        <article>
            <header>{header()}</header>
        </article>
    }
}

fn main() {
    // ERROR: missing required slot `header` on `Page`
    let _ = PageProps::new().build();
}
