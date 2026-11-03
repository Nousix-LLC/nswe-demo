//! Integration tests for the `#[component]` proc-macro (authoring-model.md §3).
//!
//! Each test corresponds to one acceptance-criterion form from the brief.  All
//! tests run on the native host (no WASM required) and use the same structural
//! equality helpers as the view! lowering suite.

use ferric::component::Component;
use ferric::prelude::*;
use ferric::vdom::{text, VElement, VNode};
use ferric_macros::{component, view};
use std::rc::Rc;

// ---------------------------------------------------------------------------
// Structural equality helpers (copied from lowering.rs)
// ---------------------------------------------------------------------------

fn vnode_eq(a: &VNode, b: &VNode) -> bool {
    match (a, b) {
        (VNode::Text(ta), VNode::Text(tb)) => ta == tb,
        (VNode::Element(ea), VNode::Element(eb)) => velem_eq(ea, eb),
        _ => false,
    }
}

fn velem_eq(a: &ferric::vdom::VElement, b: &ferric::vdom::VElement) -> bool {
    a.tag == b.tag
        && a.key == b.key
        && a.attrs == b.attrs
        && a.children.len() == b.children.len()
        && a.children
            .iter()
            .zip(b.children.iter())
            .all(|(ca, cb)| vnode_eq(ca, cb))
}

// ---------------------------------------------------------------------------
// AC 1 — Required props: component with one required prop compiles and renders
// ---------------------------------------------------------------------------

/// A `#[component]` with a required `String` prop.
#[component]
fn Greeter(name: String) -> VNode {
    VElement::new("span").child(text(format!("hello {name}"))).into()
}

#[test]
fn test_required_prop_set_and_renders() {
    let result = view! { <Greeter name={"ferric".to_string()} /> };
    let expected: VNode = VElement::new("span")
        .child(text("hello ferric"))
        .into();
    assert!(
        vnode_eq(&result, &expected),
        "required prop mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ---------------------------------------------------------------------------
// AC 2 — Defaulted prop: `#[prop(default = expr)]`
// ---------------------------------------------------------------------------

/// A `#[component]` with a required prop and a defaulted `usize` prop.
#[component]
fn Greeting(name: String, #[prop(default = 3)] times: usize) -> VNode {
    VElement::new("p")
        .child(text(format!("{name}×{times}")))
        .into()
}

#[test]
fn test_defaulted_prop_explicit_value() {
    let result = view! { <Greeting name={"hi".to_string()} times={5usize} /> };
    let expected: VNode = VElement::new("p").child(text("hi×5")).into();
    assert!(
        vnode_eq(&result, &expected),
        "defaulted prop (explicit) mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

#[test]
fn test_defaulted_prop_uses_default_when_omitted() {
    let result = view! { <Greeting name={"hi".to_string()} /> };
    let expected: VNode = VElement::new("p").child(text("hi×3")).into();
    assert!(
        vnode_eq(&result, &expected),
        "defaulted prop (default) mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ---------------------------------------------------------------------------
// AC 3 — Converting prop: `#[prop(into)]`
// ---------------------------------------------------------------------------

/// A `#[component]` where `src` accepts `impl Into<String>`.
#[component]
fn Avatar(#[prop(into)] src: String) -> VNode {
    VElement::new("img").attr("src", src).into()
}

#[test]
fn test_into_prop_accepts_str_literal() {
    // `"/img.png"` is `&str` — coerces via `Into<String>` through the setter.
    let result = view! { <Avatar src="/img.png" /> };
    let expected: VNode = VElement::new("img").attr("src", "/img.png").into();
    assert!(
        vnode_eq(&result, &expected),
        "#[prop(into)] mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ---------------------------------------------------------------------------
// AC 4 — Children: component with a `children: Children` parameter
// ---------------------------------------------------------------------------

/// A `#[component]` that accepts child content.
#[component]
fn Card(title: String, children: Children) -> VNode {
    let body_nodes = children();
    let mut body = VElement::new("div");
    for n in body_nodes {
        body = body.child(n);
    }
    VNode::from(
        VElement::new("section")
            .child(VNode::from(VElement::new("h2").child(text(title))))
            .child(VNode::from(body)),
    )
}

#[test]
fn test_component_with_children() {
    let result = view! {
        <Card title={"welcome".to_string()}>
            <p>"body text"</p>
        </Card>
    };
    // Structural check: outer section → h2("welcome") + div → p("body text")
    match &result {
        VNode::Element(section) => {
            assert_eq!(section.tag, "section", "outer tag");
            assert_eq!(section.children.len(), 2, "section should have 2 children");
            match &section.children[0] {
                VNode::Element(h2) => {
                    assert_eq!(h2.tag, "h2");
                    assert!(
                        matches!(&h2.children[0], VNode::Text(t) if t == "welcome"),
                        "h2 child should be text 'welcome'"
                    );
                }
                other => panic!("expected h2, got {:?}", other),
            }
            match &section.children[1] {
                VNode::Element(div) => {
                    assert_eq!(div.tag, "div");
                    assert_eq!(div.children.len(), 1, "body div should have 1 child");
                    match &div.children[0] {
                        VNode::Element(p) => assert_eq!(p.tag, "p"),
                        other => panic!("expected p, got {:?}", other),
                    }
                }
                other => panic!("expected div, got {:?}", other),
            }
        }
        other => panic!("expected section element, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// AC 5 — Named slot: component with `header: Slot` and `children: Children`
// ---------------------------------------------------------------------------

/// A `#[component]` with a named slot and default children.
#[component]
fn Page(header: Slot, children: Children) -> VNode {
    let header_nodes = header();
    let body_nodes = children();
    let mut hdr = VElement::new("header");
    for n in header_nodes {
        hdr = hdr.child(n);
    }
    let mut main = VElement::new("main");
    for n in body_nodes {
        main = main.child(n);
    }
    VNode::from(
        VElement::new("article")
            .child(VNode::from(hdr))
            .child(VNode::from(main)),
    )
}

#[test]
fn test_component_with_named_slot() {
    let result = view! {
        <Page>
            <slot:header><h1>"Title"</h1></slot:header>
            <p>"body"</p>
        </Page>
    };

    match &result {
        VNode::Element(article) => {
            assert_eq!(article.tag, "article", "outer tag should be article");
            assert_eq!(article.children.len(), 2, "article should have header + main");
            // First child: <header><h1>Title</h1></header>
            match &article.children[0] {
                VNode::Element(hdr) => {
                    assert_eq!(hdr.tag, "header");
                    assert_eq!(hdr.children.len(), 1);
                    match &hdr.children[0] {
                        VNode::Element(h1) => assert_eq!(h1.tag, "h1"),
                        other => panic!("expected h1, got {:?}", other),
                    }
                }
                other => panic!("expected header element, got {:?}", other),
            }
            // Second child: <main><p>body</p></main>
            match &article.children[1] {
                VNode::Element(main) => {
                    assert_eq!(main.tag, "main");
                    assert_eq!(main.children.len(), 1);
                    match &main.children[0] {
                        VNode::Element(p) => assert_eq!(p.tag, "p"),
                        other => panic!("expected p, got {:?}", other),
                    }
                }
                other => panic!("expected main element, got {:?}", other),
            }
        }
        other => panic!("expected article element, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// AC 6 — SlotFn<T>: type declaration compiles and is accessible
// ---------------------------------------------------------------------------

/// A generic `#[component]` with a `SlotFn<T>` parameter.
/// Full `view!` invocation for `SlotFn<T>` is out of scope; we verify only
/// that the type is declared, the generated props compiles, and direct Rust
/// usage works.
#[component]
fn List<T: Clone + 'static>(items: Vec<T>, row: SlotFn<T>) -> VNode {
    let mut ul = VElement::new("ul");
    for item in items {
        let nodes = row(item);
        let mut li = VElement::new("li");
        for n in nodes {
            li = li.child(n);
        }
        ul = ul.child(VNode::from(li));
    }
    VNode::from(ul)
}

#[test]
fn test_slotfn_type_compiles_and_is_usable() {
    // Construct directly at the Rust level (not via view!).
    let row: SlotFn<String> = Rc::new(|s: String| {
        vec![VElement::new("span").child(text(s)).into()]
    });
    let items = vec!["a".to_string(), "b".to_string()];
    let result = ListProps::new()
        .items(items)
        .row(row)
        .build()
        .render();
    match &result {
        VNode::Element(ul) => {
            assert_eq!(ul.tag, "ul");
            assert_eq!(ul.children.len(), 2, "ul should have 2 li children");
        }
        other => panic!("expected ul, got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// AC 7 — `dyn Component` dynamic dispatch still works
// ---------------------------------------------------------------------------

#[test]
fn test_dyn_component_dispatch_unaffected() {
    // A hand-written `impl Component` struct.
    struct Clock;
    impl Component for Clock {
        fn render(&self) -> VNode {
            VElement::new("time").into()
        }
    }

    // A bare `Fn() -> VNode` (blanket impl).
    let spacer = || -> VNode { VElement::new("hr").into() };

    // A `#[component]`-generated struct.
    #[component]
    fn Badge(label: String) -> VNode {
        VElement::new("span").child(text(label)).into()
    }

    let badge = BadgeProps::new().label("ok".to_string()).build();
    let components: Vec<Box<dyn Component>> = vec![
        Box::new(Clock),
        Box::new(spacer),
        Box::new(badge),
    ];

    let tags: Vec<&str> = components.iter().map(|c| {
        match c.render() {
            VNode::Element(e) => Box::leak(e.tag.into_boxed_str()) as &str,
            VNode::Text(_) => "text",
        }
    }).collect();

    assert_eq!(tags, vec!["time", "hr", "span"],
        "dyn Component dispatch should work for all three component forms");
}

// ---------------------------------------------------------------------------
// AC 8 — Zero-prop component
// ---------------------------------------------------------------------------

/// A `#[component]` with no parameters.
#[component]
fn Empty() -> VNode {
    VElement::new("span").into()
}

#[test]
fn test_zero_prop_component() {
    let result = view! { <Empty /> };
    let expected: VNode = VElement::new("span").into();
    assert!(
        vnode_eq(&result, &expected),
        "zero-prop component mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ---------------------------------------------------------------------------
// AC 9 — `Children` type is in the prelude and callable
// ---------------------------------------------------------------------------

#[test]
fn test_children_type_callable() {
    let thunk: Children = Rc::new(|| {
        vec![VElement::new("li").child(text("item")).into()]
    });
    let nodes = thunk();
    assert_eq!(nodes.len(), 1);
    assert!(matches!(&nodes[0], VNode::Element(e) if e.tag == "li"));
}

// ---------------------------------------------------------------------------
// AC 10 — `Slot` type is distinct and callable
// ---------------------------------------------------------------------------

#[test]
fn test_slot_type_callable() {
    let slot: Slot = Rc::new(|| vec![VElement::new("h1").into()]);
    let nodes = slot();
    assert_eq!(nodes.len(), 1);
    assert!(matches!(&nodes[0], VNode::Element(e) if e.tag == "h1"));
}
