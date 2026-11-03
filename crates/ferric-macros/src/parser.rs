//! Custom recursive-descent parser for the `view!` DSL.
//!
//! Converts a [`proc_macro2::TokenStream`] into a [`ViewRoot`] by walking
//! [`ParseStream`]. All seven §2.9 diagnostic errors are detected here and
//! surfaced as [`syn::Error`] values so [`parse_macro_input!`] emits them as
//! compile errors with the correct spans.

use proc_macro2::Span;
use syn::{
    braced,
    parse::{Parse, ParseStream},
    Expr, Ident, LitStr, Result, Token,
};

// ---------------------------------------------------------------------------
// AST
// ---------------------------------------------------------------------------

/// Root of a parsed `view!` invocation — exactly one top-level node.
pub(crate) struct ViewRoot(pub(crate) ViewNode);

/// A node in the view tree.
pub(crate) enum ViewNode {
    /// An HTML element: `<div …>…</div>` or `<br/>`.
    Element(Element),
    /// A quoted text child: `"hello"`.
    Text(LitStr),
    /// A brace-delimited Rust expression: `{expr}`.
    Interp { expr: Expr, span: Span },
    /// A keyed list: `<For each={…} key={…} let:item>body</For>`.
    For(ForList),
    /// A named slot fill: `<slot:name>…</slot:name>` inside a component element.
    SlotFill(SlotFillNode),
}

/// A parsed HTML element or component invocation.
pub(crate) struct Element {
    /// Span of the opening `<` for diagnostics.
    pub open_span: Span,
    /// Tag name (e.g. `"div"`, `"MyComp"`).
    pub tag: String,
    /// True when the tag starts with an uppercase letter.
    pub is_component: bool,
    /// `key={e}` if present.
    pub key: Option<Expr>,
    /// `name="v"` / `name={e}` attributes.
    pub attrs: Vec<HtmlAttr>,
    /// `on:event={handler}` bindings.
    pub events: Vec<EventBinding>,
    /// Children (empty for self-closing elements).
    pub children: Vec<ChildItem>,
}

/// A child item inside an element body.
pub(crate) enum ChildItem {
    Single(ViewNode),
}

/// A single HTML attribute.
pub(crate) struct HtmlAttr {
    pub name: String,
    pub value: AttrValue,
    /// Span of the value portion (for type-error diagnostics).
    pub value_span: Span,
}

/// Value of an HTML attribute.
pub(crate) enum AttrValue {
    Lit(String),
    Expr(Expr),
}

/// An `on:event={handler}` binding.
pub(crate) struct EventBinding {
    pub name: String,
    pub handler: Expr,
    pub handler_span: Span,
}

/// A `<For each={…} key={…} let:item>body</For>` keyed list.
pub(crate) struct ForList {
    pub span: Span,
    pub each: Expr,
    pub key_fn: Expr,
    pub item: Ident,
    pub body: Box<ViewNode>,
}

/// A `<slot:name>…</slot:name>` named slot fill inside a component element.
pub(crate) struct SlotFillNode {
    /// Span of the opening `<` for diagnostics.
    pub span: Span,
    /// Slot name (the identifier after `:`).
    pub name: String,
    /// Children inside the slot fill.
    pub children: Vec<ChildItem>,
}

// ---------------------------------------------------------------------------
// Parse impl
// ---------------------------------------------------------------------------

impl Parse for ViewRoot {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Err(input.error("view! requires a root element"));
        }
        let node = parse_node(input)?;
        if !input.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "a `view!` must have a single root element; wrap these in one element",
            ));
        }
        Ok(ViewRoot(node))
    }
}

fn parse_node(input: ParseStream) -> Result<ViewNode> {
    if input.peek(LitStr) {
        let lit: LitStr = input.parse()?;
        Ok(ViewNode::Text(lit))
    } else if input.peek(Token![<]) {
        parse_element(input)
    } else if input.peek(syn::token::Brace) {
        let content;
        let brace = braced!(content in input);
        let span = brace.span.join();
        let expr: Expr = content.parse()?;
        Ok(ViewNode::Interp { expr, span })
    } else {
        Err(input.error("expected `<tag>`, a string literal, or `{expr}`"))
    }
}

fn parse_element(input: ParseStream) -> Result<ViewNode> {
    let lt: Token![<] = input.parse()?;
    let open_span = lt.span;

    let tag_ident: Ident = input.parse()?;
    let tag = tag_ident.to_string();

    if tag == "For" {
        return parse_for_body(input, open_span);
    }

    // `<slot:name>…</slot:name>` — a named slot fill inside a component element.
    if tag == "slot" && input.peek(Token![:]) {
        return parse_slot_fill(input, open_span);
    }

    let is_component = tag.chars().next().is_some_and(|c| c.is_uppercase());
    let (key, attrs, events) = parse_attrs(input, &tag, open_span)?;

    // Self-closing `/>` ?
    if input.peek(Token![/]) {
        let _slash: Token![/] = input.parse()?;
        let _gt: Token![>] = input.parse()?;
        return Ok(ViewNode::Element(Element {
            open_span,
            tag,
            is_component,
            key,
            attrs,
            events,
            children: Vec::new(),
        }));
    }

    let _gt: Token![>] = input.parse()?;
    let children = parse_children(input, &tag, open_span)?;

    Ok(ViewNode::Element(Element {
        open_span,
        tag,
        is_component,
        key,
        attrs,
        events,
        children,
    }))
}

/// Parse attributes on an element, stopping before `>` or `/>`.
fn parse_attrs(
    input: ParseStream,
    tag: &str,
    open_span: Span,
) -> Result<(Option<Expr>, Vec<HtmlAttr>, Vec<EventBinding>)> {
    let mut key: Option<Expr> = None;
    let mut attrs: Vec<HtmlAttr> = Vec::new();
    let mut events: Vec<EventBinding> = Vec::new();

    loop {
        if input.peek(Token![>]) || input.peek(Token![/]) {
            break;
        }
        if input.is_empty() {
            return Err(syn::Error::new(
                open_span,
                format!("unclosed element `<{tag}>`"),
            ));
        }

        // Look ahead to categorise the attribute without consuming.
        {
            let ahead = input.fork();
            if let Ok(first) = ahead.parse::<Ident>() {
                let first_name = first.to_string();

                if first_name == "on" && ahead.peek(Token![:]) {
                    // `on:event={handler}`
                    let _on: Ident = input.parse()?;
                    let _colon: Token![:] = input.parse()?;
                    let ev_ident: Ident = input.parse()?;
                    let ev_name = ev_ident.to_string();
                    let ev_span = ev_ident.span();

                    match ev_name.as_str() {
                        "click" | "input" => {}
                        other => {
                            return Err(syn::Error::new(
                                ev_span,
                                format!(
                                    "unknown event `{other}`; bound events are: click, input"
                                ),
                            ));
                        }
                    }

                    let _eq: Token![=] = input.parse()?;
                    let hcontent;
                    let hbrace = braced!(hcontent in input);
                    let handler_span = hbrace.span.join();
                    let handler: Expr = hcontent.parse()?;

                    if !matches!(handler, Expr::Closure(_)) {
                        return Err(syn::Error::new(
                            handler_span,
                            format!("`on:{ev_name}` expects a handler closure"),
                        ));
                    }

                    events.push(EventBinding {
                        name: ev_name,
                        handler,
                        handler_span,
                    });
                    continue;
                }

                if first_name == "key" && ahead.peek(Token![=]) {
                    // `key={expr}` — only treat as VNode key when value is braced.
                    // Fork again from input to look past the `=`.
                    let ahead2 = input.fork();
                    let _ = ahead2.parse::<Ident>(); // consume "key"
                    let _ = ahead2.parse::<Token![=]>(); // consume "="
                    if ahead2.peek(syn::token::Brace) {
                        let _key_id: Ident = input.parse()?;
                        let _eq: Token![=] = input.parse()?;
                        let kcontent;
                        braced!(kcontent in input);
                        let expr: Expr = kcontent.parse()?;
                        key = Some(expr);
                        continue;
                    }
                }
            }
        }

        // Regular attribute: `name="v"` or `name={e}`
        let name_ident: Ident = input.parse()?;
        let name = name_ident.to_string();
        let _eq: Token![=] = input.parse()?;
        let (val, val_span) = parse_attr_value(input)?;
        attrs.push(HtmlAttr {
            name,
            value: val,
            value_span: val_span,
        });
    }

    Ok((key, attrs, events))
}

fn parse_attr_value(input: ParseStream) -> Result<(AttrValue, Span)> {
    if input.peek(LitStr) {
        let lit: LitStr = input.parse()?;
        let span = lit.span();
        Ok((AttrValue::Lit(lit.value()), span))
    } else if input.peek(syn::token::Brace) {
        let content;
        let brace = braced!(content in input);
        let span = brace.span.join();
        let expr: Expr = content.parse()?;
        Ok((AttrValue::Expr(expr), span))
    } else {
        Err(input.error("attribute value must be a string literal or `{expr}`"))
    }
}

fn parse_children(
    input: ParseStream,
    tag: &str,
    open_span: Span,
) -> Result<Vec<ChildItem>> {
    let mut children = Vec::new();

    loop {
        // Check for `</`
        if input.peek(Token![<]) && input.peek2(Token![/]) {
            let _lt: Token![<] = input.parse()?;
            let _sl: Token![/] = input.parse()?;
            let close_id: Ident = input.parse()?;
            let close_name = close_id.to_string();
            let close_span = close_id.span();
            let _gt: Token![>] = input.parse()?;

            if close_name != tag {
                return Err(syn::Error::new(
                    close_span,
                    format!(
                        "closing tag `</{close_name}>` does not match open tag `<{tag}>`"
                    ),
                ));
            }
            break;
        }

        if input.is_empty() {
            return Err(syn::Error::new(
                open_span,
                format!("unclosed element `<{tag}>`"),
            ));
        }

        let node = parse_node(input)?;
        children.push(ChildItem::Single(node));
    }

    Ok(children)
}

// ---------------------------------------------------------------------------
// <slot:name> fill parser
// ---------------------------------------------------------------------------

/// Parse `<slot:name>…</slot:name>` (the `<` and `slot` ident have already been
/// consumed by `parse_element`).
fn parse_slot_fill(input: ParseStream, open_span: Span) -> Result<ViewNode> {
    // We already consumed `<slot`; next is `:name>…</slot:name>`.
    let _colon: Token![:] = input.parse()?;
    let name_ident: Ident = input.parse()?;
    let name = name_ident.to_string();
    let _gt: Token![>] = input.parse()?;

    // Parse children until `</slot:name>`.
    let mut children: Vec<ChildItem> = Vec::new();
    loop {
        // Check for `</slot:name>` closing tag.
        if input.peek(Token![<]) && input.peek2(Token![/]) {
            // Fork to inspect without consuming.
            let ahead = input.fork();
            let _: Token![<] = ahead.parse()?;
            let _: Token![/] = ahead.parse()?;
            if ahead
                .parse::<Ident>()
                .ok()
                .map(|id| id == "slot")
                .unwrap_or(false)
                && ahead.peek(Token![:])
            {
                // Consume the closing tag from `input`.
                let _lt: Token![<] = input.parse()?;
                let _sl: Token![/] = input.parse()?;
                let _slot_id: Ident = input.parse()?; // "slot"
                let _colon2: Token![:] = input.parse()?;
                let close_name: Ident = input.parse()?;
                let _gt2: Token![>] = input.parse()?;
                if close_name != name_ident {
                    return Err(syn::Error::new(
                        close_name.span(),
                        format!(
                            "closing `</slot:{}>` does not match open `<slot:{name}>`",
                            close_name
                        ),
                    ));
                }
                break;
            }
        }

        if input.is_empty() {
            return Err(syn::Error::new(
                open_span,
                format!("unclosed `<slot:{name}>`"),
            ));
        }

        let node = parse_node(input)?;
        children.push(ChildItem::Single(node));
    }

    Ok(ViewNode::SlotFill(SlotFillNode {
        span: open_span,
        name,
        children,
    }))
}

fn parse_for_body(input: ParseStream, span: Span) -> Result<ViewNode> {
    let mut each: Option<Expr> = None;
    let mut key_fn: Option<Expr> = None;
    let mut item_ident: Option<Ident> = None;

    loop {
        if input.peek(Token![>]) {
            break;
        }
        if input.is_empty() {
            return Err(syn::Error::new(span, "unclosed element `<For>`"));
        }

        if input.peek(Token![let]) {
            let _let: Token![let] = input.parse()?;
            let _colon: Token![:] = input.parse()?;
            let ident: Ident = input.parse()?;
            item_ident = Some(ident);
            continue;
        }

        let attr: Ident = input.parse()?;
        match attr.to_string().as_str() {
            "each" => {
                let _eq: Token![=] = input.parse()?;
                let acontent;
                braced!(acontent in input);
                each = Some(acontent.parse()?);
            }
            "key" => {
                let _eq: Token![=] = input.parse()?;
                let kcontent;
                braced!(kcontent in input);
                key_fn = Some(kcontent.parse()?);
            }
            other => {
                return Err(syn::Error::new(
                    attr.span(),
                    format!("unknown `<For>` attribute `{other}`; expected `each`, `key`, or `let:item`"),
                ));
            }
        }
    }

    if key_fn.is_none() {
        return Err(syn::Error::new(
            span,
            "list item needs `key={…}` for keyed reconciliation",
        ));
    }

    let _gt: Token![>] = input.parse()?;
    let body = parse_node(input)?;

    // `</For>`
    if !input.peek(Token![<]) || !input.peek2(Token![/]) {
        return Err(syn::Error::new(span, "unclosed element `<For>`"));
    }
    let _lt: Token![<] = input.parse()?;
    let _sl: Token![/] = input.parse()?;
    let close_id: Ident = input.parse()?;
    let close_name = close_id.to_string();
    let _gt2: Token![>] = input.parse()?;

    if close_name != "For" {
        return Err(syn::Error::new(
            close_id.span(),
            format!(
                "closing tag `</{close_name}>` does not match open tag `<For>`"
            ),
        ));
    }

    Ok(ViewNode::For(ForList {
        span,
        each: each.ok_or_else(|| syn::Error::new(span, "<For> requires `each={…}`"))?,
        key_fn: key_fn.unwrap(),
        item: item_ident
            .ok_or_else(|| syn::Error::new(span, "<For> requires `let:item`"))?,
        body: Box::new(body),
    }))
}
