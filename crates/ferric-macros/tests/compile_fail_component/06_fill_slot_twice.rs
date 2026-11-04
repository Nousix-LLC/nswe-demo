//! Fixture: fill a named slot twice.
//!
//! `Page` declares `header: Slot` as a required slot.  Calling
//! `.header(…)` twice on `PageProps` must be a compile-time error naming
//! the doubly-filled slot and the component.

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
    let fill = || vec![];
    // ERROR: slot `header` on `Page` has already been filled
    let _ = PageProps::new()
        .header(::std::rc::Rc::new(fill))
        .header(::std::rc::Rc::new(fill)) // second fill — compile error
        .build();
}
