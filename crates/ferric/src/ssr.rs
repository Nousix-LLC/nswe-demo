//! Server-side rendering: render a component closure to an HTML string.
//!
//! This module owns the SSR render path. It compiles and runs off-wasm on a native host;
//! it must never reference `web_sys` or `wasm_bindgen`.
//!
//! # Text-boundary markers
//!
//! The server render wraps every `VNode::Text` node in HTML comment sentinels so the client
//! hydration walk can locate text nodes by index without counting the comment nodes as
//! structural VNode children. Both sides use the constants defined here — never inline
//! literal copies.
//!
//! # Implementation
//!
//! `render_to_string` runs the component closure once inside a fresh
//! a per-request reactive scope, which is disposed before the function returns. This
//! ensures no signal subscriptions from one render leak into subsequent calls or run as
//! effects on the server.

use crate::vdom::VNode;

/// HTML comment that opens a text-boundary region around a `VNode::Text` node.
///
/// The server emits this immediately before the escaped text content. The client's
/// hydration walk recognises this sentinel and does not count it when advancing the
/// VNode child index.  Both sides MUST use this exact literal — never define a
/// local copy.
pub const TEXT_OPEN_MARKER: &str = "<!--$t-->";

/// HTML comment that closes a text-boundary region.
///
/// Paired with [`TEXT_OPEN_MARKER`]. The client's hydration walk recognises this
/// sentinel and skips it when counting structural VNode children.  Both sides MUST
/// use this exact literal — never define a local copy.
pub const TEXT_CLOSE_MARKER: &str = "<!--/$t-->";

/// Returns `true` if `tag` is an HTML void element (self-closing; no end tag).
///
/// Per the SSR contract, void elements are emitted as `<tag ... />` with no children.
fn is_void_element(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

/// HTML-escape `s`, replacing the five injection-risky characters.
///
/// - `<` → `&lt;`
/// - `>` → `&gt;`
/// - `&` → `&amp;`
/// - `"` → `&quot;`
/// - `'` → `&#39;`
///
/// Applied to both text content (inside `VNode::Text`) and attribute values.
fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Recursively render `node` into `out`.
///
/// - `VNode::Element(el)` → `<tag attr="val">children</tag>`, or `<tag ... />` for
///   void elements. Event handlers are dropped: they are wired by `hydrate` on the client.
/// - `VNode::Text(s)` → `TEXT_OPEN_MARKER + escape(s) + TEXT_CLOSE_MARKER`.
fn render_node(node: &VNode, out: &mut String) {
    match node {
        VNode::Text(s) => {
            out.push_str(TEXT_OPEN_MARKER);
            out.push_str(&escape_html(s));
            out.push_str(TEXT_CLOSE_MARKER);
        }
        VNode::Element(el) => {
            out.push('<');
            out.push_str(&el.tag);
            for (name, value) in &el.attrs {
                out.push(' ');
                out.push_str(name);
                out.push_str("=\"");
                out.push_str(&escape_html(value));
                out.push('"');
            }
            // Event handlers are intentionally silent on the server; they are wired
            // at hydration time by the client.
            if is_void_element(&el.tag) {
                // Void elements are self-closing and cannot have children.
                out.push_str(" />");
            } else {
                out.push('>');
                for child in &el.children {
                    render_node(child, out);
                }
                out.push_str("</");
                out.push_str(&el.tag);
                out.push('>');
            }
        }
    }
}

/// Render a component closure to an HTML string.
///
/// `f` is called once inside a fresh reactive scope; that scope is dropped before
/// this function returns, so no signal subscriptions survive the call.  Text content
/// and attribute values are HTML-escaped; `VNode::Text` nodes are wrapped in
/// [`TEXT_OPEN_MARKER`] / [`TEXT_CLOSE_MARKER`] sentinels.
///
/// # Compile target
///
/// This function compiles and runs on a native (non-wasm) host target.  It must not
/// import `web_sys` or `wasm_bindgen`.
pub fn render_to_string<F>(f: F) -> String
where
    F: FnOnce() -> VNode,
{
    // A fresh ReactiveScope owns every reactive node created during the render.
    // When the scope drops at the end of this function it disposes those nodes and
    // removes all their subscriptions, so signal state never leaks between calls.
    let scope = crate::signal::ReactiveScope::new();
    let vnode = scope.run(f);
    let mut out = String::new();
    render_node(&vnode, &mut out);
    out
    // `vnode` then `scope` drop here (reverse declaration order): scope disposes
    // all reactive nodes before the caller resumes, fulfilling the isolation invariant.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vdom::{text, VElement};

    fn el(tag: &str) -> VElement {
        VElement::new(tag)
    }

    // ── Basic element rendering ───────────────────────────────────────────────

    #[test]
    fn renders_empty_element() {
        let html = render_to_string(|| el("div").into());
        assert_eq!(html, "<div></div>");
    }

    #[test]
    fn renders_element_with_attributes() {
        let html = render_to_string(|| el("a").attr("href", "/home").attr("class", "nav").into());
        assert_eq!(html, r#"<a href="/home" class="nav"></a>"#);
    }

    #[test]
    fn renders_element_with_text_child() {
        let html = render_to_string(|| el("p").child(text("hello")).into());
        assert_eq!(
            html,
            format!("<p>{}hello{}</p>", TEXT_OPEN_MARKER, TEXT_CLOSE_MARKER)
        );
    }

    // ── Text escaping ─────────────────────────────────────────────────────────
    //
    // REQ-SSR-002: all five injection-risky characters are escaped in both text
    // content and attribute values.

    #[test]
    fn escapes_all_five_injection_chars_in_text_content() {
        let html = render_to_string(|| text("<>&\"'"));
        assert!(html.contains("&lt;"), "< not escaped: {html}");
        assert!(html.contains("&gt;"), "> not escaped: {html}");
        assert!(html.contains("&amp;"), "& not escaped: {html}");
        assert!(html.contains("&quot;"), "\" not escaped: {html}");
        assert!(html.contains("&#39;"), "' not escaped: {html}");
    }

    #[test]
    fn escapes_all_five_injection_chars_in_attribute_values() {
        let html = render_to_string(|| el("div").attr("data-v", "<>&\"'").into());
        assert!(html.contains("&lt;"), "< not escaped in attr: {html}");
        assert!(html.contains("&gt;"), "> not escaped in attr: {html}");
        assert!(html.contains("&amp;"), "& not escaped in attr: {html}");
        assert!(html.contains("&quot;"), "\" not escaped in attr: {html}");
        assert!(html.contains("&#39;"), "' not escaped in attr: {html}");
    }

    // ── Text-boundary markers ─────────────────────────────────────────────────
    //
    // REQ-SSR-003: VNode::Text nodes are wrapped in TEXT_OPEN_MARKER / TEXT_CLOSE_MARKER.

    #[test]
    fn text_node_wrapped_in_markers() {
        let html = render_to_string(|| text("world"));
        let expected = format!("{}{}{}", TEXT_OPEN_MARKER, "world", TEXT_CLOSE_MARKER);
        assert_eq!(html, expected, "text not wrapped in markers: {html}");
    }

    #[test]
    fn empty_text_node_wrapped_in_markers() {
        let html = render_to_string(|| text(""));
        let expected = format!("{}{}", TEXT_OPEN_MARKER, TEXT_CLOSE_MARKER);
        assert_eq!(html, expected, "empty text not wrapped in markers: {html}");
    }

    // ── Nested elements ───────────────────────────────────────────────────────

    #[test]
    fn renders_nested_children_at_correct_depth() {
        let html = render_to_string(|| {
            el("ul")
                .child(el("li").child(text("one")))
                .child(el("li").child(text("two")))
                .into()
        });
        let one = format!("{}{}{}", TEXT_OPEN_MARKER, "one", TEXT_CLOSE_MARKER);
        let two = format!("{}{}{}", TEXT_OPEN_MARKER, "two", TEXT_CLOSE_MARKER);
        assert_eq!(
            html,
            format!("<ul><li>{one}</li><li>{two}</li></ul>"),
            "nested structure wrong: {html}"
        );
    }

    #[test]
    fn renders_deeply_nested_structure() {
        let html = render_to_string(|| {
            el("section")
                .child(el("div").child(el("p").child(text("deep"))))
                .into()
        });
        let deep = format!("{}{}{}", TEXT_OPEN_MARKER, "deep", TEXT_CLOSE_MARKER);
        assert_eq!(
            html,
            format!("<section><div><p>{deep}</p></div></section>"),
            "deep nesting wrong: {html}"
        );
    }

    // ── Void elements ─────────────────────────────────────────────────────────

    #[test]
    fn void_element_emits_self_closing_tag() {
        let html = render_to_string(|| el("br").into());
        assert_eq!(html, "<br />");
    }

    #[test]
    fn void_element_with_attributes_emits_self_closing() {
        let html = render_to_string(|| el("input").attr("type", "text").into());
        assert_eq!(html, r#"<input type="text" />"#);
    }

    // ── Per-request reactive scope isolation ──────────────────────────────────
    //
    // REQ-SSR-012: each render_to_string call runs in its own ReactiveScope;
    // signal subscriptions must not leak between concurrent calls.

    #[test]
    fn two_renders_produce_independent_outputs() {
        use crate::signal::create_signal;

        let html_a = render_to_string(|| {
            let msg = create_signal("alpha".to_string());
            el("p").child(text(msg.get())).into()
        });

        let html_b = render_to_string(|| {
            let msg = create_signal("beta".to_string());
            el("p").child(text(msg.get())).into()
        });

        let alpha = format!("{}{}{}", TEXT_OPEN_MARKER, "alpha", TEXT_CLOSE_MARKER);
        let beta = format!("{}{}{}", TEXT_OPEN_MARKER, "beta", TEXT_CLOSE_MARKER);

        assert!(
            html_a.contains(&alpha),
            "first render should contain 'alpha', got: {html_a}"
        );
        assert!(
            html_b.contains(&beta),
            "second render should contain 'beta', got: {html_b}"
        );
        // Independence: neither result contains the other's content.
        assert!(
            !html_a.contains(&beta),
            "first render leaked second render content: {html_a}"
        );
        assert!(
            !html_b.contains(&alpha),
            "second render leaked first render content: {html_b}"
        );
    }

    #[test]
    fn signal_created_inside_render_does_not_survive_beyond_call() {
        use crate::signal::create_signal;

        // Render A: signal value = 42.
        let html_a = render_to_string(|| {
            let counter = create_signal(42u32);
            el("span").child(text(counter.get().to_string())).into()
        });

        // Render B: signal value = 99; must not see render A's signal.
        let html_b = render_to_string(|| {
            let counter = create_signal(99u32);
            el("span").child(text(counter.get().to_string())).into()
        });

        let forty_two = format!("{}{}{}", TEXT_OPEN_MARKER, "42", TEXT_CLOSE_MARKER);
        let ninety_nine = format!("{}{}{}", TEXT_OPEN_MARKER, "99", TEXT_CLOSE_MARKER);

        assert!(
            html_a.contains(&forty_two),
            "render A value wrong: {html_a}"
        );
        assert!(
            html_b.contains(&ninety_nine),
            "render B value wrong: {html_b}"
        );
        assert!(
            !html_b.contains("42"),
            "render A signal leaked into render B: {html_b}"
        );
    }
}
