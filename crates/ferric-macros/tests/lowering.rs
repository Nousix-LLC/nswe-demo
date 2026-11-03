//! Conformance tests for the `view!` macro lowering table (authoring-model.md §2.12).
//!
//! Each `#[test]` asserts that `view!{<the form>}` and the equivalent
//! hand-built `VElement`/`vdom::text` builder chain produce structurally equal
//! [`VNode`] values.
//!
//! # Equality approach
//!
//! [`VNode`] and [`VElement`] do not derive [`PartialEq`]: event handlers are
//! `Rc<dyn Fn()>` and closures are not comparable by value.  We therefore use a
//! recursive [`vnode_eq`] helper that
//!
//! * compares [`VNode::Text`] nodes by string content,
//! * compares [`VNode::Element`] nodes by tag, key, attrs, event *kinds* (not
//!   handler identity), and children recursively.
//!
//! All test assertions carry descriptive failure messages produced via [`Debug`]
//! so failures are readable without a debugger.

use ferric::vdom::{text, EventKind, VElement, VNode};
use ferric_macros::view;

// ─── Structural equality helpers ─────────────────────────────────────────────

/// Structurally compare two [`VNode`] trees.
///
/// Event-handler closures are excluded from the comparison — only the
/// [`EventKind`]s bound in the same order are checked.
fn vnode_eq(a: &VNode, b: &VNode) -> bool {
    match (a, b) {
        (VNode::Text(ta), VNode::Text(tb)) => ta == tb,
        (VNode::Element(ea), VNode::Element(eb)) => velem_eq(ea, eb),
        _ => false,
    }
}

fn velem_eq(a: &VElement, b: &VElement) -> bool {
    a.tag == b.tag
        && a.key == b.key
        && a.attrs == b.attrs
        && events_kinds_eq(a, b)
        && a.children.len() == b.children.len()
        && a.children
            .iter()
            .zip(b.children.iter())
            .all(|(ca, cb)| vnode_eq(ca, cb))
}

/// Compare event bindings by [`EventKind`] only (handlers are closures and
/// cannot be compared by value).
fn events_kinds_eq(a: &VElement, b: &VElement) -> bool {
    if a.events.len() != b.events.len() {
        return false;
    }
    a.events
        .iter()
        .zip(b.events.iter())
        .all(|((ak, _), (bk, _))| ak == bk)
}

// ─── §2.12 Row 1a — DOM element open/close ───────────────────────────────────

/// Row 1a: `<div></div>` → `VElement::new("div")`.
#[test]
fn test_dom_element_open_close() {
    let result: VNode = view! { <div></div> };
    let expected: VNode = VElement::new("div").into();
    assert!(
        vnode_eq(&result, &expected),
        "open/close element mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ─── §2.12 Row 1b — Self-closing element ─────────────────────────────────────

/// Row 1b: `<br/>` → `VElement::new("br")`.
#[test]
fn test_dom_element_self_closing() {
    let result: VNode = view! { <br/> };
    let expected: VNode = VElement::new("br").into();
    assert!(
        vnode_eq(&result, &expected),
        "self-closing element mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ─── §2.12 Row 2 — Static text child ─────────────────────────────────────────

/// Row 2: `"text"` as a child → `vdom::text("text")`.
#[test]
fn test_static_text_child() {
    let result: VNode = view! { <p>"hello"</p> };
    let expected: VNode = VElement::new("p").child(text("hello")).into();
    assert!(
        vnode_eq(&result, &expected),
        "static text child mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ─── §2.12 Row 3 — Dynamic text child (Into<String>) ─────────────────────────

/// Row 3: `{expr: Into<String>}` in child position → `vdom::text(expr)`.
#[test]
fn test_dynamic_text_into_string() {
    let greeting = "ferric".to_string();
    let result: VNode = view! { <span>{greeting.clone()}</span> };
    let expected: VNode = VElement::new("span").child(text(greeting)).into();
    assert!(
        vnode_eq(&result, &expected),
        "dynamic text (Into<String>) mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ─── §2.12 Row 4 — Dynamic VNode child ───────────────────────────────────────

/// Row 4: `{expr: VNode}` in child position → the `VNode` itself, spliced.
#[test]
fn test_dynamic_vnode_child() {
    let inner: VNode = VElement::new("span").child(text("inner")).into();
    let result: VNode = view! { <div>{inner.clone()}</div> };
    let expected: VNode = VElement::new("div").child(inner).into();
    assert!(
        vnode_eq(&result, &expected),
        "dynamic VNode child mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ─── §2.12 Row 5a — Vec<VNode> child ─────────────────────────────────────────

/// Row 5a: `{vec: Vec<VNode>}` in child position → `.children(vec)`.
#[test]
fn test_vec_children() {
    let items: Vec<VNode> = vec![
        VElement::new("li").child(text("a")).into(),
        VElement::new("li").child(text("b")).into(),
    ];
    let result: VNode = view! { <ul>{items.clone()}</ul> };
    let expected: VNode = VElement::new("ul").children(items).into();
    assert!(
        vnode_eq(&result, &expected),
        "Vec<VNode> children mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ─── §2.12 Row 5b — <For> list ───────────────────────────────────────────────

/// Row 5b: `<For each={…} key={…} let:item>body</For>` in child position →
/// `Vec<VNode>` via `.children(iter.map(…).collect())`, each node keyed by
/// `key_fn(item)`.
#[test]
fn test_for_list() {
    let names = vec!["alice".to_string(), "bob".to_string()];
    let result: VNode = view! {
        <ul>
            <For each={names.clone()} key={|s: &String| s.clone()} let:item>
                <li>{item.clone()}</li>
            </For>
        </ul>
    };
    // Hand-build the equivalent: each <li> has a key equal to the name.
    let expected: VNode = VElement::new("ul")
        .children(names.iter().map(|s| {
            VElement::new("li")
                .key(s.clone())
                .child(text(s.clone()))
                .into()
        }))
        .into();
    assert!(
        vnode_eq(&result, &expected),
        "<For> list mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ─── §2.12 Row 6a — Static attribute ─────────────────────────────────────────

/// Row 6a: `name="v"` → `.attr("name", "v")`.
#[test]
fn test_static_attribute() {
    let result: VNode = view! { <div class="container"></div> };
    let expected: VNode = VElement::new("div").attr("class", "container").into();
    assert!(
        vnode_eq(&result, &expected),
        "static attribute mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ─── §2.12 Row 6b — Dynamic attribute ────────────────────────────────────────

/// Row 6b: `name={e}` where `e: Into<String>` → `.attr("name", e)`.
#[test]
fn test_dynamic_attribute() {
    let cls = "active".to_string();
    let result: VNode = view! { <div class={cls.clone()}></div> };
    let expected: VNode = VElement::new("div").attr("class", cls).into();
    assert!(
        vnode_eq(&result, &expected),
        "dynamic attribute mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ─── §2.12 Row 7 — key attribute ─────────────────────────────────────────────

/// Row 7: `key={e}` → `.key(e)` (`e: Into<Key>`, `Key = String`).
#[test]
fn test_key_attribute() {
    let result: VNode = view! { <li key={"item-1"}></li> };
    let expected: VNode = VElement::new("li").key("item-1").into();
    assert!(
        vnode_eq(&result, &expected),
        "key attribute mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ─── §2.12 Row 8a — on:click handler ─────────────────────────────────────────

/// Row 8a: `on:click={h}` → `.on(EventKind::Click, h)`.
///
/// Handler closures are not compared by value; this test asserts that the
/// produced element has exactly one binding for [`EventKind::Click`].
#[test]
fn test_on_click_handler() {
    let result: VNode = view! { <button on:click={|| {}}>"click me"</button> };
    let expected: VNode = VElement::new("button")
        .on(EventKind::Click, || {})
        .child(text("click me"))
        .into();
    assert!(
        vnode_eq(&result, &expected),
        "on:click handler mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ─── §2.12 Row 8b — on:input handler ─────────────────────────────────────────

/// Row 8b: `on:input={h}` → `.on(EventKind::Input, h)`.
///
/// Handler closures are not compared by value; this test asserts that the
/// produced element has exactly one binding for [`EventKind::Input`].
#[test]
fn test_on_input_handler() {
    let result: VNode = view! { <input on:input={|| {}}/> };
    let expected: VNode = VElement::new("input").on(EventKind::Input, || {}).into();
    assert!(
        vnode_eq(&result, &expected),
        "on:input handler mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ─── §2.12 Row 10 — Component invocation ─────────────────────────────────────

/// Row 10: `<Capitalized … />` → component model invocation yielding a `VNode`.
///
/// The `view!` macro lowers a capitalised tag to a plain function call, so the
/// component function must be in scope and return a `VNode`.
#[test]
#[allow(non_snake_case)]
fn test_component_invocation() {
    fn Greeting() -> VNode {
        VElement::new("span").child(text("Hello")).into()
    }

    let result: VNode = view! { <Greeting /> };
    let expected: VNode = Greeting();
    assert!(
        vnode_eq(&result, &expected),
        "component invocation mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ─── §2.12 Row 11 — Final .into() → VNode ────────────────────────────────────

/// Row 11: the top-level element of `view!` is converted with `.into()` to a
/// `VNode`; the return type of `view!` is `VNode`, not `VElement`.
#[test]
fn test_view_produces_vnode() {
    let result: VNode = view! { <section></section> };
    assert!(
        matches!(result, VNode::Element(_)),
        "view! must produce a VNode::Element, got a text node"
    );
}

// ─── Nested element (additional Row 1 coverage) ──────────────────────────────

/// Nested elements: `<div><span>"inner"</span></div>` → builder chain with child.
#[test]
fn test_nested_element() {
    let result: VNode = view! { <div><span>"inner"</span></div> };
    // Use VNode::from() explicitly to avoid type-inference ambiguity introduced by
    // the local `impl From<VElement>` that the view! macro generates inside other
    // test functions in this file.
    let inner: VNode = VNode::from(VElement::new("span").child(text("inner")));
    let expected: VNode = VElement::new("div").child(inner).into();
    assert!(
        vnode_eq(&result, &expected),
        "nested element mismatch\n  got:  {:?}\n  want: {:?}",
        result,
        expected
    );
}

// ─── I-LOWER-02: <For> produces all_keyed Vec<VNode> with correct keys ────────

/// **I-LOWER-02**: every node produced by `<For>` is a `VNode::Element` with a
/// non-`None` key equal to `key_fn(item)`.
///
/// This satisfies identity assertion I-LOWER-02 from `contracts/_MANIFEST.yaml`:
/// the produced sibling list is *all_keyed* (every child is a `VNode::Element`
/// with `key` set) and each key equals `key_fn(item)`.
#[test]
fn test_for_list_all_keyed() {
    struct Item {
        id: u32,
        label: &'static str,
    }
    let items = vec![
        Item { id: 1, label: "one" },
        Item { id: 2, label: "two" },
        Item { id: 3, label: "three" },
    ];

    let result: VNode = view! {
        <ul>
            <For each={items} key={|i: &Item| i.id.to_string()} let:item>
                <li>{item.label}</li>
            </For>
        </ul>
    };

    // Extract children from the <ul>.
    let children = match &result {
        VNode::Element(el) => &el.children,
        VNode::Text(_) => panic!("expected VNode::Element for <ul>, got Text"),
    };

    assert_eq!(
        children.len(),
        3,
        "expected 3 children from <For>, got {}",
        children.len()
    );

    let expected_keys = ["1", "2", "3"];
    for (i, (child, expected_key)) in children.iter().zip(expected_keys.iter()).enumerate() {
        match child {
            VNode::Element(el) => {
                assert!(
                    el.key.is_some(),
                    "child[{i}] must have a non-None key (I-LOWER-02)"
                );
                assert_eq!(
                    el.key.as_deref(),
                    Some(*expected_key),
                    "child[{i}] key must equal key_fn(item) = {:?} (I-LOWER-02)",
                    expected_key
                );
            }
            VNode::Text(_) => {
                panic!(
                    "child[{i}] is a text node; <For> must produce keyed VNode::Element \
                     nodes (I-LOWER-02)"
                );
            }
        }
    }
}
