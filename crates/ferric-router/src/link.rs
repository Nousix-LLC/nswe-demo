//! Declarative in-app navigation helper producing a `ferric` view node.
//!
//! [`link`] returns a [`VNode`] that navigates client-side — without a full page reload — when the
//! user clicks it, by calling [`navigate`]. It composes with the rest of
//! the router: the click updates the reactive current path, and every mounted
//! [`outlet`](crate::router::Router::outlet) re-renders to show the newly matched route.
//!
//! # Why this is a `<span>`, not an `<a href>` (the `preventDefault` constraint)
//!
//! A conventional SPA link is an `<a href="/path">` whose click handler calls
//! `event.preventDefault()` to stop the browser from performing its default full-page navigation to
//! `href`, then routes client-side instead. **That is not reachable here:** `ferric`'s click handler
//! is a zero-argument `Fn()` (see [`EventKind`] and `ferric`'s `VElement::on`), so the handler never
//! receives a `web_sys::Event` and therefore *cannot* call `preventDefault()`.
//!
//! Rather than work around this by editing `ferric` to pass the event (out of scope for this
//! work-item), [`link`] sidesteps the default-navigation behaviour entirely: it emits a **non-anchor
//! element** (`<span>`) that the browser will never full-page-navigate on, because there is no `href`
//! for it to follow. The element carries a `ferric` [`EventKind::Click`] handler that calls
//! [`navigate`], giving the same user-visible behaviour as a preventing
//! `<a>` without needing an event object. The destination is also recorded in a `data-to` attribute
//! (useful for styling, testing, and accessibility tooling), and `role="link"` + `tabindex="0"`
//! keep the element focusable and announced as a link by assistive technology.
//!
//! If a future `ferric` revision makes the event payload available to click handlers, this helper
//! could additively grow a true `<a href>` variant that calls `preventDefault()`; the frozen
//! [`link`] signature would be unaffected.
//!
//! ```no_run
//! use ferric_router::prelude::*;
//! use ferric::prelude::*;
//!
//! // A nav bar of in-app links. Clicking one routes client-side; no full page reload.
//! let navbar = move || -> VNode {
//!     VElement::new("nav")
//!         .child(link("/", "Home"))
//!         .child(link("/about", "About"))
//!         .child(link("/user/42", "User 42"))
//!         .into()
//! };
//! # let _ = navbar;
//! ```

use ferric::vdom::{text, EventKind, VElement, VNode};

use crate::router::navigate;

/// The CSS class placed on every link element so applications can style in-app links
/// (e.g. `.ferric-router-link { cursor: pointer; color: blue; text-decoration: underline; }`).
pub const LINK_CLASS: &str = "ferric-router-link";

/// A declarative in-app navigation link: returns a `ferric` [`VNode`] that, when clicked, calls
/// [`navigate`] to `to` (client-side, no full page reload) and displays
/// `label`.
///
/// The returned node is a focusable, link-styled `<span>` carrying a [`EventKind::Click`] handler —
/// see the [module documentation](self) for why a non-anchor element is used (the `ferric`
/// `preventDefault` constraint). For additional control over the emitted element (a custom CSS
/// class), use the [`Link`] builder, of which this function is the common-case shorthand.
///
/// ```no_run
/// use ferric_router::prelude::*;
/// use ferric::prelude::*;
///
/// let home: VNode = link("/", "Home");
/// let profile: VNode = link("/user/42", "Alice");
/// # let _ = (home, profile);
/// ```
pub fn link(to: &str, label: &str) -> VNode {
    Link::new(to, label).build()
}

/// A builder for an in-app navigation [`link`], for the cases where the plain [`link`] shorthand is
/// not enough — currently, choosing the CSS class placed on the emitted element.
///
/// This is an **additive** extension of the frozen public surface: the frozen [`link`] function is
/// exactly `Link::new(to, label).build()`. The emitted element and its click-to-[`navigate`] wiring
/// are identical; only the `class` attribute differs.
///
/// ```no_run
/// use ferric_router::link::Link;
/// use ferric::vdom::VNode;
///
/// // A link that also carries an application-specific "active-nav" class.
/// let node: VNode = Link::new("/about", "About")
///     .class("ferric-router-link nav-active")
///     .build();
/// # let _ = node;
/// ```
pub struct Link {
    /// The destination path navigated to on click (e.g. `"/user/42"`).
    to: String,
    /// The visible link text.
    label: String,
    /// The CSS class applied to the emitted element.
    class: String,
}

impl Link {
    /// Start building a link to `to` displaying `label`, with the default [`LINK_CLASS`].
    #[must_use]
    pub fn new(to: &str, label: &str) -> Self {
        Link {
            to: to.to_owned(),
            label: label.to_owned(),
            class: LINK_CLASS.to_owned(),
        }
    }

    /// Override the CSS class placed on the emitted element (replaces the default [`LINK_CLASS`]).
    /// Builder-style (returns `self`).
    #[must_use]
    pub fn class(mut self, class: &str) -> Self {
        self.class = class.to_owned();
        self
    }

    /// Finish building and produce the navigation [`VNode`].
    ///
    /// The result is a `<span role="link" tabindex="0">` carrying the chosen class, a `data-to`
    /// attribute recording the destination, and a [`EventKind::Click`] handler that calls
    /// [`navigate`] to the destination.
    #[must_use]
    pub fn build(self) -> VNode {
        let Link { to, label, class } = self;
        VElement::new("span")
            .attr("role", "link")
            .attr("tabindex", "0")
            .attr("class", class)
            .attr("data-to", to.as_str())
            .on(EventKind::Click, move || navigate(&to))
            .child(text(label))
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferric::vdom::VNode;

    /// Destructure a link `VNode` into its `VElement`, failing the test if it is not an element.
    fn element(node: VNode) -> ferric::vdom::VElement {
        match node {
            VNode::Element(element) => element,
            VNode::Text(_) => panic!("link() must produce an element node, not text"),
        }
    }

    fn attr<'a>(element: &'a ferric::vdom::VElement, name: &str) -> Option<&'a str> {
        element
            .attrs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }

    #[test]
    fn link_emits_a_non_anchor_element() {
        // The whole point of the preventDefault workaround: it must NOT be an <a>, so the browser
        // never performs a default full-page navigation (there is no href to follow).
        let element = element(link("/about", "About"));
        assert_ne!(element.tag, "a", "link must not emit an anchor element");
        assert_eq!(element.tag, "span");
    }

    #[test]
    fn link_shows_its_label_as_text() {
        let element = element(link("/user/42", "Alice"));
        assert!(
            matches!(element.children.as_slice(), [VNode::Text(label)] if label == "Alice"),
            "link should display its label as a single text child"
        );
    }

    #[test]
    fn link_records_destination_and_is_focusable_and_announced() {
        let element = element(link("/user/42", "Alice"));
        assert_eq!(attr(&element, "data-to"), Some("/user/42"));
        assert_eq!(attr(&element, "role"), Some("link"));
        assert_eq!(attr(&element, "tabindex"), Some("0"));
        assert_eq!(attr(&element, "class"), Some(LINK_CLASS));
    }

    #[test]
    fn link_binds_a_single_click_handler() {
        let element = element(link("/", "Home"));
        assert!(
            matches!(element.events.as_slice(), [(EventKind::Click, _)]),
            "link should bind exactly one click handler (and no other event kind)"
        );
    }

    #[test]
    fn builder_allows_a_custom_class_without_changing_wiring() {
        let element = element(Link::new("/about", "About").class("nav-active").build());
        assert_eq!(attr(&element, "class"), Some("nav-active"));
        // The click wiring and destination are unchanged by the additive builder.
        assert_eq!(attr(&element, "data-to"), Some("/about"));
        assert!(matches!(element.events.as_slice(), [(EventKind::Click, _)]));
    }
}
