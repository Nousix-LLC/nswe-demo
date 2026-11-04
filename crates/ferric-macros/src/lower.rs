//! Code generator: lowers a [`ViewRoot`] AST into a [`proc_macro2::TokenStream`]
//! that constructs a [`ferric::VNode`] using only the frozen builder API.
//!
//! Every row in §2.12 of `design/ferric/authoring-model.md` is covered here.

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, quote_spanned};

use crate::parser::{
    AttrValue, ChildItem, Element, ForList, HtmlAttr, SlotFillNode, ViewNode, ViewRoot,
};

/// Lower a parsed `view!` root into a `VNode` expression.
pub(crate) fn lower_root(root: &ViewRoot) -> TokenStream2 {
    lower_node(&root.0)
}

fn lower_node(node: &ViewNode) -> TokenStream2 {
    match node {
        ViewNode::Text(lit) => {
            let s = lit.value();
            quote! { ::ferric::vdom::text(#s) }
        }
        ViewNode::Interp { expr, span } => {
            // `{expr}` in child position: dispatch to either VNode (identity),
            // Vec<VNode> (.extend), String / &str (vdom::text), using an inline
            // enum with concrete From impls — no blanket, no coherence issues.
            lower_interp_child(expr, *span)
        }
        ViewNode::Element(el) => lower_element(el),
        ViewNode::For(fl) => lower_for_as_expr(fl),
        ViewNode::SlotFill(sf) => {
            // A slot fill appearing as a root node is an error (it must appear
            // inside a component element).
            let span = sf.span;
            let name = &sf.name;
            quote_spanned! { span =>
                compile_error!(
                    concat!("`<slot:", #name, ">` must be a child of a component element")
                )
            }
        }
    }
}

// ---------------------------------------------------------------------------
// {expr} child dispatch
// ---------------------------------------------------------------------------

/// Lowers `{expr}` in child position to a `VNode`.
///
/// Uses an inline local enum with four concrete `From` impls so type inference
/// routes the value correctly:
/// - `VNode`         → identity
/// - `Vec<VNode>`    → not reachable here (handled by `lower_interp_children`)
/// - `String`        → `vdom::text(s)`
/// - `&str`          → `vdom::text(s)`
///
/// Callers that need to handle `Vec<VNode>` use `lower_interp_children` instead.
fn lower_interp_child(expr: &syn::Expr, span: Span) -> TokenStream2 {
    quote_spanned! { span =>
        {
            #[allow(dead_code)]
            enum __FerricChild {
                Node(::ferric::VNode),
            }
            impl ::std::convert::From<::ferric::VNode> for __FerricChild {
                fn from(v: ::ferric::VNode) -> Self { Self::Node(v) }
            }
            impl ::std::convert::From<::ferric::vdom::VElement> for __FerricChild {
                fn from(e: ::ferric::vdom::VElement) -> Self {
                    Self::Node(::ferric::VNode::Element(e))
                }
            }
            impl ::std::convert::From<String> for __FerricChild {
                fn from(s: String) -> Self { Self::Node(::ferric::vdom::text(s)) }
            }
            impl<'__ferric_a> ::std::convert::From<&'__ferric_a str> for __FerricChild {
                fn from(s: &'__ferric_a str) -> Self { Self::Node(::ferric::vdom::text(s)) }
            }
            match __FerricChild::from(#expr) {
                __FerricChild::Node(n) => n,
            }
        }
    }
}

/// Lowers `{expr}` in child position to a push/extend sequence on `__el`.
///
/// Handles both single-VNode and Vec<VNode> by using a two-variant enum with
/// concrete `From` impls.
fn lower_interp_children_push(expr: &syn::Expr, span: Span) -> TokenStream2 {
    quote_spanned! { span =>
        {
            #[allow(dead_code)]
            enum __FerricChildren {
                One(::ferric::VNode),
                Many(Vec<::ferric::VNode>),
            }
            impl ::std::convert::From<::ferric::VNode> for __FerricChildren {
                fn from(v: ::ferric::VNode) -> Self { Self::One(v) }
            }
            impl ::std::convert::From<::ferric::vdom::VElement> for __FerricChildren {
                fn from(e: ::ferric::vdom::VElement) -> Self {
                    Self::One(::ferric::VNode::Element(e))
                }
            }
            impl ::std::convert::From<Vec<::ferric::VNode>> for __FerricChildren {
                fn from(v: Vec<::ferric::VNode>) -> Self { Self::Many(v) }
            }
            impl ::std::convert::From<String> for __FerricChildren {
                fn from(s: String) -> Self { Self::One(::ferric::vdom::text(s)) }
            }
            impl<'__ferric_a> ::std::convert::From<&'__ferric_a str> for __FerricChildren {
                fn from(s: &'__ferric_a str) -> Self { Self::One(::ferric::vdom::text(s)) }
            }
            match __FerricChildren::from(#expr) {
                __FerricChildren::One(n) => __ferric_el = __ferric_el.child(n),
                __FerricChildren::Many(ns) => __ferric_el = __ferric_el.children(ns),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Element lowering
// ---------------------------------------------------------------------------

fn lower_element(el: &Element) -> TokenStream2 {
    if el.is_component {
        return lower_component(el);
    }

    let tag = &el.tag;

    // Build attribute/event/key calls.
    let mut attr_calls: Vec<TokenStream2> = Vec::new();

    if let Some(key_expr) = &el.key {
        attr_calls.push(quote! { __ferric_el = __ferric_el.key(#key_expr); });
    }

    for attr in &el.attrs {
        attr_calls.push(lower_attr(attr));
    }

    for ev in &el.events {
        attr_calls.push(lower_event(ev));
    }

    // Build children pushes.
    let child_stmts: Vec<TokenStream2> = el.children.iter().map(lower_child_push).collect();

    quote! {
        {
            let mut __ferric_el = ::ferric::VElement::new(#tag);
            #(#attr_calls)*
            #(#child_stmts)*
            ::ferric::VNode::from(__ferric_el)
        }
    }
}

fn lower_attr(attr: &HtmlAttr) -> TokenStream2 {
    let name = &attr.name;
    let val_span = attr.value_span;
    match &attr.value {
        AttrValue::Lit(s) => {
            quote! { __ferric_el = __ferric_el.attr(#name, #s); }
        }
        AttrValue::Expr(expr) => {
            // `quote_spanned!` points any type error to the author's expr.
            quote_spanned! { val_span =>
                __ferric_el = __ferric_el.attr(#name, #expr);
            }
        }
    }
}

fn lower_event(ev: &crate::parser::EventBinding) -> TokenStream2 {
    let kind = match ev.name.as_str() {
        "click" => quote! { ::ferric::EventKind::Click },
        "input" => quote! { ::ferric::EventKind::Input },
        // Unknown event names are caught in the parser; this is unreachable.
        _ => unreachable!("event names validated in parser"),
    };
    let handler = &ev.handler;
    quote! { __ferric_el = __ferric_el.on(#kind, #handler); }
}

fn lower_child_push(item: &ChildItem) -> TokenStream2 {
    match item {
        ChildItem::Single(node) => match node {
            ViewNode::Text(lit) => {
                let s = lit.value();
                quote! { __ferric_el = __ferric_el.child(::ferric::vdom::text(#s)); }
            }
            ViewNode::Interp { expr, span } => {
                // Uses the two-variant dispatch (handles Vec<VNode> too).
                lower_interp_children_push(expr, *span)
            }
            ViewNode::Element(sub_el) => {
                let sub_ts = lower_element(sub_el);
                quote! { __ferric_el = __ferric_el.child(#sub_ts); }
            }
            ViewNode::For(fl) => {
                // <For> produces Vec<VNode> — extend children.
                let for_ts = lower_for_as_vec(fl);
                quote! { __ferric_el = __ferric_el.children(#for_ts); }
            }
            ViewNode::SlotFill(sf) => {
                // A slot fill inside a plain HTML element is an error.
                let span = sf.span;
                let name = &sf.name;
                quote_spanned! { span =>
                    compile_error!(
                        concat!("`<slot:", #name, ">` is only valid inside a component element")
                    );
                }
            }
        },
    }
}

// ---------------------------------------------------------------------------
// Children / slot helpers for component lowering
// ---------------------------------------------------------------------------

/// Lower a `ChildItem` to a statement that pushes/extends a local
/// `Vec<VNode>` variable named `__ferric_v`.  Used when building the
/// `Rc<dyn Fn() -> Vec<VNode>>` thunks passed to component prop setters.
fn lower_child_push_to_vec(item: &ChildItem) -> TokenStream2 {
    match item {
        ChildItem::Single(node) => lower_node_push_to_vec(node),
    }
}

fn lower_node_push_to_vec(node: &ViewNode) -> TokenStream2 {
    match node {
        ViewNode::Text(lit) => {
            let s = lit.value();
            quote! { __ferric_v.push(::ferric::vdom::text(#s)); }
        }
        ViewNode::Interp { expr, span } => lower_interp_push_to_vec(expr, *span),
        ViewNode::Element(sub_el) => {
            let sub_ts = lower_element(sub_el);
            quote! { __ferric_v.push(#sub_ts); }
        }
        ViewNode::For(fl) => {
            let for_ts = lower_for_as_vec(fl);
            quote! { __ferric_v.extend(#for_ts); }
        }
        ViewNode::SlotFill(sf) => {
            let span = sf.span;
            let name = &sf.name;
            quote_spanned! { span =>
                compile_error!(
                    concat!("`<slot:", #name, ">` cannot be nested inside another slot")
                );
            }
        }
    }
}

/// Lowers `{expr}` inside a children/slot thunk to push/extend statements on
/// `__ferric_v`.  Handles `VNode`, `Vec<VNode>`, `String`, and `&str`.
fn lower_interp_push_to_vec(expr: &syn::Expr, span: Span) -> TokenStream2 {
    quote_spanned! { span =>
        {
            #[allow(dead_code)]
            enum __FerricSlotChildren {
                One(::ferric::VNode),
                Many(::std::vec::Vec<::ferric::VNode>),
            }
            impl ::std::convert::From<::ferric::VNode> for __FerricSlotChildren {
                fn from(v: ::ferric::VNode) -> Self { Self::One(v) }
            }
            impl ::std::convert::From<::ferric::VElement> for __FerricSlotChildren {
                fn from(e: ::ferric::VElement) -> Self {
                    Self::One(::ferric::VNode::Element(e))
                }
            }
            impl ::std::convert::From<::std::vec::Vec<::ferric::VNode>> for __FerricSlotChildren {
                fn from(v: ::std::vec::Vec<::ferric::VNode>) -> Self { Self::Many(v) }
            }
            impl ::std::convert::From<::std::string::String> for __FerricSlotChildren {
                fn from(s: ::std::string::String) -> Self {
                    Self::One(::ferric::vdom::text(s))
                }
            }
            impl<'__ferric_a> ::std::convert::From<&'__ferric_a str> for __FerricSlotChildren {
                fn from(s: &'__ferric_a str) -> Self {
                    Self::One(::ferric::vdom::text(s))
                }
            }
            match __FerricSlotChildren::from(#expr) {
                __FerricSlotChildren::One(n) => __ferric_v.push(n),
                __FerricSlotChildren::Many(ns) => __ferric_v.extend(ns),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// <For> lowering
// ---------------------------------------------------------------------------

/// Lowers `<For>` to a `Vec<VNode>` expression (§2.12 row 5 / §2.6).
///
/// ```text
/// items.into_iter().map(|item| {
///     let mut __ferric_node = {body};
///     if let VNode::Element(ref mut __ferric_el_ref) = __ferric_node {
///         __ferric_el_ref.key = Some(key_fn(&item));
///     }
///     __ferric_node
/// }).collect::<Vec<VNode>>()
/// ```
fn lower_for_as_vec(fl: &ForList) -> TokenStream2 {
    let each = &fl.each;
    let key_fn = &fl.key_fn;
    let item = &fl.item;
    let body = lower_node(&fl.body);

    // Hygienic variable names — `mixed_site` keeps them out of user error messages.
    let node_var = format_ident!("__ferric_node", span = Span::mixed_site());
    let el_ref_var = format_ident!("__ferric_el_ref", span = Span::mixed_site());

    quote! {
        (#each).into_iter().map(|#item| {
            let mut #node_var = #body;
            if let ::ferric::VNode::Element(ref mut #el_ref_var) = #node_var {
                #el_ref_var.key = ::std::option::Option::Some((#key_fn)(&#item));
            }
            #node_var
        }).collect::<Vec<::ferric::VNode>>()
    }
}

/// Lowers `<For>` as a `VNode` root expression (wraps in a `VNode::Element` container).
///
/// This path is only taken when `<For>` is used as the SOLE root of a `view!`.
/// That is an unusual pattern; the Vec is wrapped in a no-tag container element.
/// In practice `<For>` is always a child.
fn lower_for_as_expr(fl: &ForList) -> TokenStream2 {
    // `<For>` as a top-level root: wrap the Vec in an invisible container.
    // The spec says the single-root rule applies. A top-level `<For>` is an
    // escape hatch — the resulting `Vec<VNode>` is spliced. We produce it as
    // a compile error here: `<For>` must be a child, not a root.
    let span = fl.span;
    quote_spanned! { span =>
        compile_error!("`<For>` cannot be the root of `view!`; wrap it in an element")
    }
}

// ---------------------------------------------------------------------------
// Component invocation
// ---------------------------------------------------------------------------

/// Lowers a component invocation `<MyComp prop={v}…>children</MyComp>`.
///
/// Uses the generated `MyCompProps` builder:
/// ```rust,ignore
/// let __ferric_comp = MyCompProps::new()
///     .prop(v)
///     .children(Rc::new(move || { … }))
///     .build();
/// Component::render(&__ferric_comp)
/// ```
///
/// If `MyCompProps` is not in scope the ordinary Rust "cannot find struct
/// `MyCompProps` in this scope" error fires, pointing at the tag (§2.9).
fn lower_component(el: &Element) -> TokenStream2 {
    let props_ident = format_ident!("{}Props", el.tag, span = el.open_span);

    // --- prop setter calls from attributes ---
    let mut builder_calls: Vec<TokenStream2> = Vec::new();

    for attr in &el.attrs {
        let prop_name = format_ident!("{}", attr.name, span = attr.value_span);
        let val_span = attr.value_span;
        let call = match &attr.value {
            AttrValue::Lit(s) => quote_spanned! { val_span => .#prop_name(#s) },
            AttrValue::Expr(e) => quote_spanned! { val_span => .#prop_name(#e) },
        };
        builder_calls.push(call);
    }

    // Events are not valid on components; ignore them (they would only cause a
    // type error when attempting to call `.on(…)` on a Props builder).

    // --- Separate children: slot fills vs regular ---
    let mut regular_child_items: Vec<&ChildItem> = Vec::new();
    let mut slot_fills: Vec<&SlotFillNode> = Vec::new();

    for child in &el.children {
        match child {
            ChildItem::Single(ViewNode::SlotFill(sf)) => slot_fills.push(sf),
            other => regular_child_items.push(other),
        }
    }

    // --- build children thunk if there are regular children ---
    if !regular_child_items.is_empty() {
        let child_stmts: Vec<TokenStream2> = regular_child_items
            .iter()
            .map(|c| lower_child_push_to_vec(c))
            .collect();
        builder_calls.push(quote! {
            .children(::std::rc::Rc::new(move || {
                let mut __ferric_v: ::std::vec::Vec<::ferric::VNode> =
                    ::std::vec::Vec::new();
                #(#child_stmts)*
                __ferric_v
            }))
        });
    }

    // --- build slot thunks ---
    for sf in slot_fills {
        let slot_name = format_ident!("{}", sf.name, span = sf.span);
        let slot_stmts: Vec<TokenStream2> =
            sf.children.iter().map(lower_child_push_to_vec).collect();
        builder_calls.push(quote! {
            .#slot_name(::std::rc::Rc::new(move || {
                let mut __ferric_v: ::std::vec::Vec<::ferric::VNode> =
                    ::std::vec::Vec::new();
                #(#slot_stmts)*
                __ferric_v
            }))
        });
    }

    quote! {
        {
            let __ferric_comp = #props_ident::new()
                #(#builder_calls)*
                .build();
            ::ferric::component::Component::render(&__ferric_comp)
        }
    }
}
