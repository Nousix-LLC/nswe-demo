//! Fixture: omit a required prop on a component invocation.
//!
//! `Greeting` declares `name: String` as a required prop.  Calling
//! `GreetingProps::new().build()` without setting `name` must be a
//! compile-time error naming the missing prop and the component.

use ferric::prelude::*;

#[component]
fn Greeting(name: String) -> VNode {
    ferric::vdom::text(name)
}

fn main() {
    // ERROR: missing required prop `name` on `Greeting`
    let _ = GreetingProps::new().build();
}
