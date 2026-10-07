//! A small `ferric-router` single-page app, built for the browser.
//!
//! It wires the whole router together end-to-end:
//!
//! - a [`Router`] with three routes — `/` (home), `/about`, and the parameterised `/user/:id` —
//!   plus a fallback view for unknown paths;
//! - [`Router::mount_history`] to seed the current path from `window.location` and track browser
//!   back/forward;
//! - a reactive [`Router::outlet`] rendered inside a root component, so navigating re-renders only
//!   the routed view;
//! - a nav bar built from [`link`] for declarative in-app navigation, and a button that calls
//!   [`navigate`] programmatically.
//!
//! Build it for wasm:
//!
//! ```text
//! cargo build -p ferric-router --example app --target wasm32-unknown-unknown
//! ```
//!
//! To run it in a browser, generate the JS bindings with `wasm-bindgen` (or use `trunk`) and load
//! the module from an HTML page that provides a mount point — see `crates/ferric-router/README.md`.
//!
//! [`Router`]: ferric_router::prelude::Router
//! [`link`]: ferric_router::prelude::link
//! [`navigate`]: ferric_router::prelude::navigate

use ferric::prelude::*;
use ferric::vdom::text;
use ferric_router::prelude::*;

/// The home route (`/`).
fn home_view() -> VNode {
    VElement::new("section")
        .child(VElement::new("h1").child(text("Home")))
        .child(VElement::new("p").child(text("Welcome to the ferric-router demo.")))
        .into()
}

/// The about route (`/about`).
fn about_view() -> VNode {
    VElement::new("section")
        .child(VElement::new("h1").child(text("About")))
        .child(VElement::new("p").child(text(
            "This SPA is routed entirely on the client by ferric-router.",
        )))
        .into()
}

/// The parameterised user route (`/user/:id`): reads the `id` param the matcher extracted.
fn user_view(params: &Params) -> VNode {
    let id = params.get("id").unwrap_or("unknown");
    VElement::new("section")
        .child(VElement::new("h1").child(text(format!("User {id}"))))
        .child(VElement::new("p").child(text(format!("Profile page for user id = {id}."))))
        .into()
}

/// The fallback view, rendered when no route matches.
fn not_found_view() -> VNode {
    VElement::new("section")
        .child(VElement::new("h1").child(text("Not found")))
        .child(VElement::new("p").child(text("No route matched this path.")))
        .into()
}

/// The persistent nav bar: declarative in-app links plus one programmatic-navigation button.
fn nav_bar() -> VNode {
    VElement::new("nav")
        .attr("class", "app-nav")
        .child(link("/", "Home"))
        .child(text(" | "))
        .child(link("/about", "About"))
        .child(text(" | "))
        .child(link("/user/42", "User 42"))
        .child(text(" | "))
        // Programmatic navigation: a plain ferric button whose click handler calls `navigate`
        // directly, exactly as `link` does internally.
        .child(
            VElement::new("button")
                .on(EventKind::Click, || navigate("/user/7"))
                .child(text("Go to user 7")),
        )
        .into()
}

fn main() {
    // Build the route table, seed the current path, and install the popstate listener.
    let router = Router::new()
        .route("/", |_params| home_view())
        .route("/about", |_params| about_view())
        .route("/user/:id", user_view)
        .fallback(|_params| not_found_view())
        .mount_history();

    // The reactive outlet renders the matched route and re-renders on navigation.
    let outlet = router.outlet();

    // One root component: a persistent nav bar above the reactive outlet. Calling `outlet.render()`
    // inside this render subscribes the mount effect to the current-path signal, so a click on a
    // `link` (or the programmatic button) re-renders the routed view in place.
    let app = move || -> VNode {
        VElement::new("div")
            .attr("class", "app")
            .child(nav_bar())
            .child(outlet.render())
            .into()
    };

    // Mount into `#app` if the page provides it, otherwise into `<body>` — mirroring the ferric
    // counter example.
    let document = web_sys::window()
        .expect("no browser `window`")
        .document()
        .expect("no `document`");
    let root = document
        .get_element_by_id("app")
        .or_else(|| document.body().map(Into::into))
        .expect("the page has no `#app` element and no `<body>` to mount into");

    mount(root, app);
}
