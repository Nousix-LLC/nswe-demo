//! Procedural macros for the Ferric web framework.
//!
//! This crate exposes the [`view!`] macro that compiles Ferric template
//! syntax into a [`ferric::VNode`] expression tree.
//!
//! [`ferric::VNode`]: https://docs.rs/ferric

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Compiles a Ferric template into a `VNode` expression.
///
/// This is a scaffold stub — the full implementation is provided by
/// `SUBTASK_macro_impl`.  The stub returns an empty [`TokenStream`] so
/// that `cargo build -p ferric-macros` succeeds on its own.
///
/// # Example
///
/// ```ignore
/// use ferric::prelude::*;
/// let node = view! { <div class="hello">{"World"}</div> };
/// ```
#[proc_macro]
pub fn view(_input: TokenStream) -> TokenStream {
    let out: TokenStream2 = quote! {};
    out.into()
}
