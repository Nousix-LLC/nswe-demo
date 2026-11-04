//! Procedural macros for the Ferric web framework.
//!
//! This crate exposes the [`view!`] macro, which compiles Ferric template
//! syntax into a `ferric::VNode` builder expression.
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

mod component;
mod lower;
mod parser;

use parser::ViewRoot;

/// Compiles a Ferric template into a `VNode` expression.
///
/// The macro accepts an HTML/JSX-flavoured template and lowers it onto the
/// frozen `ferric::VNode` builder API (§2.12 of the authoring model).
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
/// | `<Name prop={v}/>` | `NameProps::new().prop(v).build()` + `Component::render` |
/// | `<Name>content</Name>` | children thunk passed as `.children(Rc::new(move \|\| …))` |
/// | `<slot:name>…</slot:name>` | named slot fill via `.name(Rc::new(move \|\| …))` |
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

/// Derives a typed props builder for a component function.
///
/// Annotate a `fn Name(prop1: T1, …) -> VNode { … }` to generate a
/// `NameProps` builder whose setter methods accept each prop.
///
/// ## Prop modifiers
///
/// | Attribute | Meaning |
/// |---|---|
/// | _(none)_ | Required prop — omission is a runtime panic via `.build()`. |
/// | `#[prop(default = expr)]` | Optional prop — omitting it applies `expr`. |
/// | `#[prop(into)]` | Setter accepts `impl Into<FieldType>` for ergonomic conversion. |
/// | `#[prop(optional)]` | Shorthand for `#[prop(default = None)]` on `Option<T>` fields. |
///
/// ## Special parameter types
///
/// | Type | Role |
/// |---|---|
/// | `Children` | Deferred child content; filled by nesting in `view!`. |
/// | `Slot` | Named content region; filled by `<slot:name>…</slot:name>` in `view!`. |
/// | `SlotFn<T>` | Parameterized slot (render-prop); passed directly in Rust. |
///
/// ## Example
///
/// ```rust,ignore
/// use ferric::prelude::*;
///
/// #[component]
/// fn Greeting(name: String, #[prop(default = 3)] times: usize) -> VNode {
///     view! { <p>{format!("Hello {name} ×{times}")}</p> }
/// }
///
/// // With both props:
/// let v = GreetingProps::new().name("world".into()).times(5).build();
/// // With default:
/// let v = GreetingProps::new().name("world".into()).build(); // times = 3
/// ```
#[proc_macro_attribute]
pub fn component(attr: TokenStream, item: TokenStream) -> TokenStream {
    component::expand_component(attr, item)
}
