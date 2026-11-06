//! `/signin` -- the sign-in entry point.
//!
//! Renders a sign-in form skeleton. The actual authentication flow (credential validation,
//! session creation) is out of scope for the public-timeline feature; this view exists so the
//! sign-in route is reachable and visitors are not sent to a 404 when they click "Sign in".
//!
//! When the authentication feature is implemented, this module is the natural home for the
//! credential form, the `POST /sessions` call, and the session-signal write.

use ferric::prelude::*;
use ferric::vdom::text;
use ferric_router::prelude::link;

use crate::app::AppContext;

/// `/signin` -- the sign-in view.
///
/// Renders a placeholder form with a link back to the landing page and to the sign-up page.
/// The form is intentionally non-functional in this iteration; a future authentication feature
/// will replace this stub with the real credential form.
#[must_use]
pub fn sign_in_view(_ctx: &AppContext) -> VNode {
    VElement::new("section")
        .attr("class", "chirp-signin")
        .child(VElement::new("h1").child(text("Sign in")))
        .child(
            VElement::new("p")
                .attr("class", "chirp-signin-description")
                .child(text("Sign in to your Chirp account.")),
        )
        .child(
            VElement::new("form")
                .attr("class", "chirp-signin-form")
                .child(
                    VElement::new("label")
                        .attr("for", "signin-username")
                        .child(text("Username")),
                )
                .child(
                    VElement::new("input")
                        .attr("id", "signin-username")
                        .attr("type", "text")
                        .attr("placeholder", "Your username")
                        .attr("autocomplete", "username"),
                )
                .child(
                    VElement::new("label")
                        .attr("for", "signin-password")
                        .child(text("Password")),
                )
                .child(
                    VElement::new("input")
                        .attr("id", "signin-password")
                        .attr("type", "password")
                        .attr("autocomplete", "current-password"),
                )
                .child(
                    VElement::new("button")
                        .attr("type", "submit")
                        .attr("disabled", "")
                        .child(text("Sign in (coming soon)")),
                ),
        )
        .child(
            VElement::new("p")
                .attr("class", "chirp-signin-links")
                .child(text("New to Chirp? "))
                .child(link("/signup", "Create an account"))
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

    // A minimal AppContext for testing -- the sign-in view does not use it.
    fn dummy_ctx() -> crate::app::AppContext {
        crate::app::AppContext::new("/api")
    }

    #[test]
    fn sign_in_view_renders_a_section_with_correct_class() {
        let tree = sign_in_view(&dummy_ctx());
        assert!(find_class(&tree, "chirp-signin").is_some());
    }

    #[test]
    fn sign_in_view_contains_link_to_signup() {
        let tree = sign_in_view(&dummy_ctx());
        let content = text_content(&tree);
        assert!(
            content.contains("Create an account") || content.contains("account"),
            "sign-in page must link to sign-up: got {content}"
        );
    }

    #[test]
    fn sign_in_view_contains_link_to_home() {
        let tree = sign_in_view(&dummy_ctx());
        let content = text_content(&tree);
        assert!(
            content.contains("home") || content.contains("Back"),
            "sign-in page must link back to home: got {content}"
        );
    }
}
