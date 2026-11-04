//! Fixture: nest children inside a component that does not accept children.
//!
//! `Label` declares only `text: String` — it has no `children: Children`
//! parameter.  When `view!` tries to nest content inside `<Label>`, it
//! generates a `.children(…)` call on `LabelProps`, which has no such
//! method, producing a compile error that names the component.

use ferric::prelude::*;

#[component]
fn Label(text: String) -> VNode {
    ferric::vdom::text(text)
}

fn main() {
    // ERROR: no method `children` found for `LabelProps`
    //        (equivalent: `Label` does not accept children)
    let _ = view! {
        <Label text={"hi".to_string()}>
            <span>"extra"</span>
        </Label>
    };
}
