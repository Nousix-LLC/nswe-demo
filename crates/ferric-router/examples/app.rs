//! A `ferric-router` single-page app demonstrating all r1 additions.
//!
//! **Routes**
//!
//! - `/` — home
//! - `/about` — about
//! - `/user/:id` — parameterised user profile
//! - `/settings` — settings layout (nested route)
//!   - `/settings/` — settings index (nested child)
//!   - `/settings/profile` — profile page (nested child)
//!
//! **Features demonstrated**
//!
//! - [`Router::nest`] — nested route with a shared `<section>` layout
//! - [`link`] active-route awareness — the nav bar's `link()` helpers emit `aria-current="page"`
//!   on the active link, enabling CSS active-link styling
//! - [`navigate_replace`] — a "redirect to login" button that replaces the current History entry
//!
//! Build it for wasm:
//!
//! ```text
//! cargo build -p ferric-router --example app --target wasm32-unknown-unknown
//! ```
//!
//! [`Router::nest`]: ferric_router::prelude::Router::nest
//! [`link`]: ferric_router::prelude::link
//! [`navigate_replace`]: ferric_router::prelude::navigate_replace

use ferric::component::Component;
use ferric::prelude::*;
use ferric::vdom::text;
use ferric_router::prelude::*;

// ── Flat routes ───────────────────────────────────────────────────────────────

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

// ── Nested route: /settings ───────────────────────────────────────────────────

/// The settings layout: a shared `<section>` wrapper rendered for every `/settings/*` path.
/// The `child_outlet` renders the matched child route's content inside the layout.
///
/// This is the [`Router::nest`] `layout` closure argument: it receives the prefix params
/// (empty here — `/settings` has no `:param` segments) and a `&dyn Component` for the
/// child outlet, and returns a `VNode`.
fn settings_layout(_params: &Params, child_outlet: &dyn Component) -> VNode {
    VElement::new("section")
        .attr("class", "settings")
        .child(VElement::new("h1").child(text("Settings")))
        .child(
            VElement::new("nav")
                .attr("class", "settings-nav")
                .child(link("/settings/", "Overview"))
                .child(text(" | "))
                .child(link("/settings/profile", "Profile")),
        )
        // The child outlet renders whichever child route matched the remaining path.
        .child(child_outlet.render())
        .into()
}

/// The settings index (`/settings/`): shown when the remaining path after the prefix is `/`.
fn settings_index_view() -> VNode {
    VElement::new("div")
        .child(VElement::new("h2").child(text("Settings Overview")))
        .child(VElement::new("p").child(text("Choose a category on the left.")))
        .into()
}

/// The settings profile child route (`/settings/profile`).
fn settings_profile_view() -> VNode {
    VElement::new("div")
        .child(VElement::new("h2").child(text("Profile Settings")))
        .child(VElement::new("p").child(text("Edit your display name and avatar here.")))
        .into()
}

// ── Nav bar ───────────────────────────────────────────────────────────────────

/// The persistent nav bar.
///
/// Each [`link`] call emits a `<span role="link">` that navigates on click and adds
/// `aria-current="page"` when its destination equals the current path — active-route awareness.
/// The `navigate_replace` button replaces the current History entry instead of pushing a new one,
/// demonstrating a redirect-style navigation (the user cannot press back to reach the page they
/// just left).
fn nav_bar() -> VNode {
    VElement::new("nav")
        .attr("class", "app-nav")
        .child(link("/", "Home"))
        .child(text(" | "))
        .child(link("/about", "About"))
        .child(text(" | "))
        .child(link("/user/42", "User 42"))
        .child(text(" | "))
        .child(link("/settings/", "Settings"))
        .child(text(" | "))
        // navigate_replace: replaces the current entry — the user cannot press back to return here.
        .child(
            VElement::new("button")
                .on(EventKind::Click, || navigate_replace("/"))
                .child(text("Redirect home (replace)")),
        )
        .child(text(" "))
        // Plain navigate: pushes a new entry — the user can press back.
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
        // Nested route: /settings with a shared layout and two child routes.
        .nest("/settings", settings_layout, |child_router| {
            child_router
                .route("/", |_params| settings_index_view())
                .route("/profile", |_params| settings_profile_view())
                .fallback(|_| not_found_view())
        })
        .fallback(|_params| not_found_view())
        .mount_history();

    // The reactive outlet renders the matched route and re-renders on navigation.
    let outlet = router.outlet();

    // One root component: a persistent nav bar above the reactive outlet.
    let app = move || -> VNode {
        VElement::new("div")
            .attr("class", "app")
            .child(nav_bar())
            .child(outlet.render())
            .into()
    };

    // Mount into `#app` if the page provides it, otherwise into `<body>`.
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
