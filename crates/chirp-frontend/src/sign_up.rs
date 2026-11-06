//! `/signup` -- the sign-up entry point.
//!
//! Renders a sign-up form skeleton. The actual account-creation flow (validation, `POST /users`,
//! session bootstrapping) is out of scope for the public-timeline feature; this view exists so
//! the sign-up route is reachable and visitors are not sent to a 404 when they click
//! "Create an account".
//!
//! When the registration feature is implemented, this module is the natural home for the
//! username / password form, the `POST /users` call, and the session-signal write.

use ferric::prelude::*;
use ferric::vdom::text;
use ferric_router::prelude::link;

use crate::app::AppContext;

/// `/signup` -- the sign-up view.
///
/// Renders a placeholder form with a link back to the landing page and to the sign-in page.
/// The form is intentionally non-functional in this iteration; a future registration feature
/// will replace this stub with the real account-creation form.
#[must_use]
pub fn sign_up_view(_ctx: &AppContext) -> VNode {
    VElement::new("section")
        .attr("class", "chirp-signup")
        .child(VElement::new("h1").child(text("Create an account")))
        .child(
            VElement::new("p")
                .attr("class", "chirp-signup-description")
                .child(text("Join Chirp and start sharing short thoughts.")),
        )
        .child(
            VElement::new("form")
                .attr("class", "chirp-signup-form")
                .child(
                    VElement::new("label")
                        .attr("for", "signup-username")
                        .child(text("Username")),
                )
                .child(
                    VElement::new("input")
                        .attr("id", "signup-username")
                        .attr("type", "text")
                        .attr("placeholder", "Choose a username")
                        .attr("autocomplete", "username"),
                )
                .child(
                    VElement::new("label")
                        .attr("for", "signup-password")
                        .child(text("Password")),
                )
                .child(
                    VElement::new("input")
                        .attr("id", "signup-password")
                        .attr("type", "password")
                        .attr("autocomplete", "new-password"),
                )
                .child(
                    VElement::new("button")
                        .attr("type", "submit")
                        .attr("disabled", "")
                        .child(text("Create account (coming soon)")),
                ),
        )
        .child(
            VElement::new("p")
                .attr("class", "chirp-signup-links")
                .child(text("Already have an account? "))
                .child(link("/signin", "Sign in"))
                .child(text(" \u{00b7} "))
                .child(link("/", "Back to home")),
        )
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find_class<'a>(node: &'a VNode, class: &str) -> Option<&'a VElement> {
        match node {
            VNode::Element(el) => {
                if el.attrs.iter().any(|(k, v)| k == "class" && v == class) {
                    return Some(el);
                }
                el.children.iter().find_map(|c| find_class(c, class))
            }
            VNode::Text(_) => None,
        }
    }

    fn text_content(node: &VNode) -> String {
        match node {
            VNode::Text(s) => s.clone(),
            VNode::Element(el) => el.children.iter().map(text_content).collect(),
        }
    }

    fn dummy_ctx() -> crate::app::AppContext {
        crate::app::AppContext::new("/api")
    }

    #[test]
    fn sign_up_view_renders_a_section_with_correct_class() {
        let tree = sign_up_view(&dummy_ctx());
        assert!(find_class(&tree, "chirp-signup").is_some());
    }

    #[test]
    fn sign_up_view_contains_link_to_signin() {
        let tree = sign_up_view(&dummy_ctx());
        let content = text_content(&tree);
        assert!(
            content.contains("Sign in"),
            "sign-up page must link to sign-in: got {content}"
        );
    }

    #[test]
    fn sign_up_view_contains_link_to_home() {
        let tree = sign_up_view(&dummy_ctx());
        let content = text_content(&tree);
        assert!(
            content.contains("home") || content.contains("Back"),
            "sign-up page must link back to home: got {content}"
        );
    }
}
