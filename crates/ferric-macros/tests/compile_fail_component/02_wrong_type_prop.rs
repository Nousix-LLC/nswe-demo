//! Fixture: pass a prop value of the wrong type.
//!
//! `Greeting` declares `name: String`.  Passing an `i32` to the
//! `.name()` setter must be a compile-time type error that names the
//! prop and the expected type.

use ferric::prelude::*;

#[component]
fn Greeting(name: String) -> VNode {
    ferric::vdom::text(name)
}

fn main() {
    // ERROR: prop `name` on `Greeting` expects `String`, got `i32`
    let _ = GreetingProps::new().name(42i32).build();
}
