//! Procedural macros for the Ferric web framework.
//!
//! This crate exposes the [`view!`] macro, which compiles Ferric template
//! syntax into a [`ferric::VNode`] builder expression.
//!
//! # Quick reference
//!
//! ```ignore
//! use ferric::prelude::*;
//!
//! let count = create_signal(0i32);
//! let app = move || view! {
//!     <div class="counter">
//!         <button on:click={move || count.update(|n| *n += 1)}>"increment"</button>
//!         <p>"count: " {count.get().to_string()}</p>
//!     </div>
//! };
//! ```

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod lower;
mod parser;

use parser::ViewRoot;

/// Compiles a Ferric template into a `VNode` expression.
///
/// The macro accepts an HTML/JSX-flavoured template and lowers it onto the
/// frozen [`ferric::VNode`] builder API (§2.12 of the authoring model).
///
/// # Supported constructs
///
/// | Author writes | Lowers to |
/// |---|---|
/// | `<div>…</div>` / `<br/>` | `VElement::new("div").children([…])` |
/// | `"text"` | `vdom::text("text")` |
/// | `{expr}` where `expr: Into<String>` | `vdom::text(expr)` |
/// | `{expr}` where `expr: VNode` | the `VNode` itself |
/// | `{vec}` / `<For …>` | `Vec<VNode>` via `.children(vec)` |
/// | `name="v"` / `name={e}` | `.attr("name", v)` / `.attr("name", e)` |
/// | `key={e}` | `.key(e)` |
/// | `on:click={h}` / `on:input={h}` | `.on(EventKind::Click, h)` |
///
/// # Diagnostics
///
/// Compile errors are emitted for all seven error cases in §2.9:
/// unclosed elements, mismatched close tags, multiple top-level nodes, unknown
/// events, non-closure handlers, missing `<For>` keys, and type mismatches on
/// dynamic attribute values.
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    let root = parse_macro_input!(input as ViewRoot);
    lower::lower_root(&root).into()
}
