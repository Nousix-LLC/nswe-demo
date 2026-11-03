//! Fixture: fill a slot the component did not declare.
//!
//! `Card` declares only `title: String` and `children: Children`.  When
//! `view!` tries to fill a `<slot:footer>`, it calls `.footer(…)` on
//! `CardProps`, which has no such method — a compile error naming the
//! undeclared slot and the component.

use ferric::prelude::*;

#[component]
fn Card(title: String, children: Children) -> VNode {
    view! {
        <section>
            <h2>{title}</h2>
            <div>{children()}</div>
        </section>
    }
}

fn main() {
    // ERROR: no method `footer` found for `CardProps`
    //        (equivalent: `Card` has no slot `footer`)
    let _ = view! {
        <Card title={"hello".to_string()}>
            <slot:footer>"© 2026"</slot:footer>
        </Card>
    };
}
