//! Implementation of the `#[component]` attribute macro.
//!
//! Transforms a function annotated with `#[component]` into:
//!
//! 1. A hidden impl-struct `__<Name>ComponentImpl` that implements
//!    [`ferric::component::Component`] by running the original function body.
//! 2. A public `<Name>Props` builder struct with one setter per parameter,
//!    respecting `#[prop(default = expr)]` and `#[prop(into)]` modifiers.
//!
//! The `view!` macro expands `<Name prop=val />` as:
//! ```rust,ignore
//! let __ferric_comp = NameProps::new().prop(val).build();
//! ::ferric::component::Component::render(&__ferric_comp)
//! ```
//!
//! so `NameProps` just needs to be in scope at the call site.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    parse::{ParseStream},
    Attribute, Expr, FnArg, Ident, ItemFn, Token, Type,
};

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub(crate) fn expand_component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func: ItemFn = match syn::parse(item) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error().into(),
    };
    match expand_inner(func) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// ---------------------------------------------------------------------------
// Prop kind
// ---------------------------------------------------------------------------

/// Modifier attached to a `#[component]` function parameter via `#[prop(..)]`.
enum PropKind {
    /// Plain required parameter — no `#[prop]` attribute.
    Required,
    /// `#[prop(default = expr)]` — omitting the prop applies `expr`.
    Default(Expr),
    /// `#[prop(into)]` — the setter accepts `impl Into<FieldType>`.
    Into,
}

struct PropParam {
    ident: Ident,
    ty: Type,
    kind: PropKind,
}

// ---------------------------------------------------------------------------
// Code generation
// ---------------------------------------------------------------------------

fn expand_inner(func: ItemFn) -> syn::Result<TokenStream2> {
    let fn_name = &func.sig.ident;
    let fn_vis = &func.vis;
    let generics = &func.sig.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let body = &func.block;

    // `GreetingProps` / `__GreetingComponentImpl`
    let props_name = format_ident!("{}Props", fn_name);
    let impl_name = format_ident!("__{}ComponentImpl", fn_name);

    // Parse parameters.
    let mut params: Vec<PropParam> = Vec::new();
    for arg in &func.sig.inputs {
        match arg {
            FnArg::Receiver(r) => {
                return Err(syn::Error::new_spanned(
                    r,
                    "#[component] functions cannot have `self`",
                ));
            }
            FnArg::Typed(typed) => {
                let ident = match &*typed.pat {
                    syn::Pat::Ident(pi) => pi.ident.clone(),
                    p => {
                        return Err(syn::Error::new_spanned(
                            p,
                            "#[component] function parameters must be simple identifiers",
                        ));
                    }
                };
                let ty = (*typed.ty).clone();
                let kind = extract_prop_kind(&typed.attrs)?;
                params.push(PropParam { ident, ty, kind });
            }
        }
    }

    // --- impl struct fields (owned values, one per parameter) ---
    let impl_fields: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            let ty = &p.ty;
            quote! { #name: #ty }
        })
        .collect();

    // --- render body: clone each field into a local variable ---
    let let_bindings: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            quote! { let #name = ::std::clone::Clone::clone(&self.#name); }
        })
        .collect();

    // --- builder struct fields (Option<T>, None by default) ---
    let builder_fields: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            let ty = &p.ty;
            quote! { #name: ::std::option::Option<#ty> }
        })
        .collect();

    // --- field initializers for `new()` ---
    let field_inits: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            quote! { #name: ::std::option::Option::None }
        })
        .collect();

    // --- setter methods ---
    let setters: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            let ty = &p.ty;
            match &p.kind {
                PropKind::Into => quote! {
                    /// Set the prop (accepts any `impl Into<T>`).
                    pub fn #name(mut self, v: impl ::std::convert::Into<#ty>) -> Self {
                        self.#name =
                            ::std::option::Option::Some(::std::convert::Into::into(v));
                        self
                    }
                },
                _ => quote! {
                    /// Set the prop.
                    pub fn #name(mut self, v: #ty) -> Self {
                        self.#name = ::std::option::Option::Some(v);
                        self
                    }
                },
            }
        })
        .collect();

    // --- build() field assignments ---
    let fn_name_str = fn_name.to_string();
    let build_assignments: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            let name_str = name.to_string();
            match &p.kind {
                PropKind::Required | PropKind::Into => {
                    let msg = format!(
                        "missing required prop `{name_str}` on `{fn_name_str}`"
                    );
                    quote! { #name: self.#name.expect(#msg) }
                }
                PropKind::Default(default_expr) => {
                    quote! { #name: self.#name.unwrap_or_else(|| #default_expr) }
                }
            }
        })
        .collect();

    Ok(quote! {
        // The hidden impl struct — not part of the public API.
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        struct #impl_name #generics #where_clause {
            #(#impl_fields,)*
        }

        impl #impl_generics ::ferric::component::Component
            for #impl_name #ty_generics #where_clause
        {
            fn render(&self) -> ::ferric::VNode {
                // Unpack every field into a local binding so the original
                // function body can reference them by name.
                #(#let_bindings)*
                // Original function body.
                #body
            }
        }

        /// Generated typed props builder for this component.
        ///
        /// Call `new()` to start the builder, chain one setter per prop,
        /// then call `build()` to get the value that implements
        /// [`ferric::component::Component`].
        #fn_vis struct #props_name #generics #where_clause {
            #(#builder_fields,)*
        }

        impl #impl_generics #props_name #ty_generics #where_clause {
            /// Create an empty props builder (all props unset).
            pub fn new() -> Self {
                Self { #(#field_inits,)* }
            }

            #(#setters)*

            /// Consume the builder and produce the component value.
            ///
            /// # Panics
            ///
            /// Panics at runtime with a descriptive message if any required
            /// prop was not set.
            pub fn build(self) -> #impl_name #ty_generics {
                #impl_name {
                    #(#build_assignments,)*
                }
            }
        }
    })
}

// ---------------------------------------------------------------------------
// Helper: extract PropKind from attributes
// ---------------------------------------------------------------------------

fn extract_prop_kind(attrs: &[Attribute]) -> syn::Result<PropKind> {
    for attr in attrs {
        if attr.path().is_ident("prop") {
            return attr.parse_args_with(
                |input: ParseStream| -> syn::Result<PropKind> {
                    let kw: Ident = input.parse()?;
                    match kw.to_string().as_str() {
                        "default" => {
                            let _eq: Token![=] = input.parse()?;
                            let expr: Expr = input.parse()?;
                            Ok(PropKind::Default(expr))
                        }
                        "into" => Ok(PropKind::Into),
                        "optional" => {
                            // Treat as `#[prop(default = None)]` for `Option<T>` fields.
                            Ok(PropKind::Default(syn::parse_quote!(
                                ::std::option::Option::None
                            )))
                        }
                        other => Err(syn::Error::new(
                            kw.span(),
                            format!(
                                "unknown #[prop] option `{other}`; \
                                 expected `default = <expr>`, `into`, or `optional`"
                            ),
                        )),
                    }
                },
            );
        }
    }
    Ok(PropKind::Required)
}
