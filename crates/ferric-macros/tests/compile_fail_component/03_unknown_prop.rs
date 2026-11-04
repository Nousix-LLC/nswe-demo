//! Fixture: pass a prop the component does not declare.
//!
//! `Greeting` only declares `name: String`.  Calling `.color("red")` on
//! `GreetingProps` must be a compile-time error naming the unknown prop
//! and the component.

use ferric::prelude::*;

#[component]
fn Greeting(name: String) -> VNode {
    ferric::vdom::text(name)
}

fn main() {
    // ERROR: `GreetingProps` has no method `color`
    //        (equivalent: `Greeting` has no prop `color`)
    let _ = GreetingProps::new()
        .name("hi".to_string())
        .color("red".to_string())
        .build();
}
