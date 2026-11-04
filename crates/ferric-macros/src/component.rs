//! Implementation of the `#[component]` attribute macro.
//!
//! Transforms a function annotated with `#[component]` into:
//!
//! 1. A hidden impl-struct `__<Name>ComponentImpl` that implements
//!    [`ferric::component::Component`] by running the original function body.
//! 2. A public `<Name>Props` builder struct with one setter per parameter,
//!    respecting `#[prop(default = expr)]` and `#[prop(into)]` modifiers.
//!
//! ## Diagnostic quality — type-state builder
//!
//! Required parameters (plain params without `#[prop(default)]`) use a
//! type-state builder pattern so that omitting a required prop is a **compile-
//! time** error rather than a runtime panic.  Named-slot parameters (`Slot`,
//! `SlotFn<T>`) additionally prevent being filled twice.
//!
//! The generated machinery per required param `name` on component `Greeting`:
//! - Marker types `Greeting_missing_required_prop_name` (unset) / `__GreetingPropNameSet` (set).
//!   The unset type's name is intentionally human-readable so it appears as a helpful message
//!   inside E0599 "doesn't satisfy `Greeting_missing_required_prop_name: __GreetingHasName`".
//! - A sealed trait `__GreetingHasName`.
//! - A PhantomData type param `__NameState` on `GreetingProps`, defaulting to the unset type.
//! - `build()` requires `__NameState: __GreetingHasName`, so it only compiles once set.
//!
//! For named-slot params (`Slot`, `SlotFn<T>`):
//! - Unset: `Page_missing_required_slot_header`; Set: `Page_slot_header_already_filled`.
//! - An additional `__PageHeaderNotYetFilled` trait gates the setter so that a double-fill
//!   also becomes a compile error naming the slot: "doesn't satisfy
//!   `Page_slot_header_already_filled: __PageHeaderNotYetFilled`".
//!
//! **Note:** type-state machinery is only generated for non-generic components.
//! Generic components fall back to the runtime-panic builder (unchanged from
//! the previous implementation).

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse::ParseStream, Attribute, Expr, FnArg, GenericParam, Ident, ItemFn, Token, Type};

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
    /// `true` for `Slot` / `SlotFn<T>` — named-slot params get double-fill
    /// prevention in addition to omission prevention.
    is_named_slot: bool,
}

/// Capitalize the first letter of a string (e.g., "name" → "Name").
fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

// ---------------------------------------------------------------------------
// Slot-type detection
// ---------------------------------------------------------------------------

/// Returns `true` if `ty` (without `Option<..>` wrapping) is `Slot` or
/// `SlotFn<T>`.
fn is_named_slot_type(ty: &Type) -> bool {
    if let Type::Path(tp) = ty {
        if let Some(last) = tp.path.segments.last() {
            return matches!(last.ident.to_string().as_str(), "Slot" | "SlotFn");
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Code generation
// ---------------------------------------------------------------------------

fn expand_inner(func: ItemFn) -> syn::Result<TokenStream2> {
    let fn_name = &func.sig.ident;
    let fn_vis = &func.vis;
    let generics = &func.sig.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let fn_name_str = fn_name.to_string();
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
                let is_named_slot = is_named_slot_type(&ty);
                params.push(PropParam {
                    ident,
                    ty,
                    kind,
                    is_named_slot,
                });
            }
        }
    }

    // --- impl struct fields ---
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

    // --- impl struct and Component impl (unchanged from prior version) ---
    let impl_struct_and_trait = quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        struct #impl_name #generics #where_clause {
            #(#impl_fields,)*
        }

        impl #impl_generics ::ferric::component::Component
            for #impl_name #ty_generics #where_clause
        {
            fn render(&self) -> ::ferric::VNode {
                #(#let_bindings)*
                #body
            }
        }
    };

    // -----------------------------------------------------------------------
    // Decide: use type-state builder (non-generic components) or fallback
    // (generic components).
    // -----------------------------------------------------------------------

    let has_generics = generics
        .params
        .iter()
        .any(|p| matches!(p, GenericParam::Type(_) | GenericParam::Const(_)));

    if has_generics {
        // --- Fallback: runtime-panic builder (original implementation) ---
        let builder_code = build_panic_builder(
            fn_vis,
            &props_name,
            &impl_name,
            &impl_generics,
            &ty_generics,
            where_clause,
            &params,
            &fn_name_str,
        );
        return Ok(quote! {
            #impl_struct_and_trait
            #builder_code
        });
    }

    // -----------------------------------------------------------------------
    // Type-state builder
    // -----------------------------------------------------------------------

    // Collect the required params (those without a default).
    let req_indices: Vec<usize> = params
        .iter()
        .enumerate()
        .filter(|(_, p)| matches!(p.kind, PropKind::Required | PropKind::Into))
        .map(|(i, _)| i)
        .collect();

    let n_req = req_indices.len();

    // If there are no required params, use a simple builder without PhantomData.
    if n_req == 0 {
        let builder_code =
            build_simple_builder(fn_vis, &props_name, &impl_name, &params, &fn_name_str);
        return Ok(quote! {
            #impl_struct_and_trait
            #builder_code
        });
    }

    // -----------------------------------------------------------------------
    // Generate per-required-param marker types and traits.
    // -----------------------------------------------------------------------

    // State type idents for the struct's type params.
    // e.g., req_indices[0] -> param "name" -> struct param "__NameState"
    let state_param_idents: Vec<Ident> = req_indices
        .iter()
        .map(|&i| {
            let p = &params[i];
            let camel = capitalize(&p.ident.to_string());
            format_ident!("__{}State", camel)
        })
        .collect();

    // Unset / Set marker types and "Has" traits per required param.
    let mut state_machinery: Vec<TokenStream2> = Vec::new();

    for &idx in &req_indices {
        let p = &params[idx];
        let name_str = p.ident.to_string();
        let comp_name_str = &fn_name_str;

        // Naming convention:
        //   Required prop unset:  Greeting_missing_required_prop_name
        //   Required prop set:    __GreetingPropNameSet  (internal; shown in unknown-prop errors)
        //   Required slot unset:  Page_missing_required_slot_header
        //   Required slot set:    Page_slot_header_already_filled  (shown in double-fill error)
        //   Has-trait:            __GreetingHasName  (internal)
        //
        // The unset/set marker type names are designed to be human-readable inside E0599 messages:
        //   "doesn't satisfy `Greeting_missing_required_prop_name: __GreetingHasName`"
        //   "doesn't satisfy `Page_slot_header_already_filled: __PageHeaderNotYetFilled`"
        let uname = capitalize(&name_str);
        let (unset_ty, set_ty) = if p.is_named_slot {
            (
                format_ident!("{}_missing_required_slot_{}", comp_name_str, name_str),
                format_ident!("{}_slot_{}_already_filled", comp_name_str, name_str),
            )
        } else {
            (
                format_ident!("{}_missing_required_prop_{}", comp_name_str, name_str),
                format_ident!("__{}Prop{}Set", comp_name_str, uname),
            )
        };
        let has_tr = format_ident!("__{}Has{}", comp_name_str, uname);

        let omit_msg = if p.is_named_slot {
            format!("missing required slot `{name_str}` on `{comp_name_str}`")
        } else {
            format!("missing required prop `{name_str}` on `{comp_name_str}`")
        };
        let omit_note = if p.is_named_slot {
            format!(
                "fill `<slot:{name_str}>…</slot:{name_str}>` inside `<{comp_name_str}>` in `view!`"
            )
        } else {
            format!("call `.{name_str}(value)` on `{props_name}` before calling `.build()`")
        };

        state_machinery.push(quote! {
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            struct #unset_ty;

            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            struct #set_ty;

            #[diagnostic::on_unimplemented(
                message = #omit_msg,
                note    = #omit_note
            )]
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            trait #has_tr {}

            #[allow(non_camel_case_types)]
            impl #has_tr for #set_ty {}
        });

        // Double-fill prevention for named-slot params.
        if p.is_named_slot {
            let not_filled_tr = format_ident!("__{}{}NotYetFilled", comp_name_str, uname);
            let double_msg = format!(
                "slot `{name_str}` on `{comp_name_str}` has already been filled; \
                 each slot can only be filled once"
            );
            state_machinery.push(quote! {
                #[diagnostic::on_unimplemented(
                    message = #double_msg
                )]
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                trait #not_filled_tr {}

                // Only the UNSET state implements this trait; Set does not.
                #[allow(non_camel_case_types)]
                impl #not_filled_tr for #unset_ty {}
            });
        }
    }

    // -----------------------------------------------------------------------
    // Builder struct definition.
    // -----------------------------------------------------------------------

    // Builder struct fields (Option<T> for all params).
    let builder_fields: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            let ty = &p.ty;
            quote! { #name: ::std::option::Option<#ty> }
        })
        .collect();

    // Build the defaults for each state param (Unset).
    let unset_defaults: Vec<TokenStream2> = req_indices
        .iter()
        .map(|&i| {
            let p = &params[i];
            let name_str = p.ident.to_string();
            let unset_ty = if p.is_named_slot {
                format_ident!("{}_missing_required_slot_{}", fn_name_str, name_str)
            } else {
                format_ident!("{}_missing_required_prop_{}", fn_name_str, name_str)
            };
            quote! { #unset_ty }
        })
        .collect();

    let set_tys: Vec<TokenStream2> = req_indices
        .iter()
        .map(|&i| {
            let p = &params[i];
            let name_str = p.ident.to_string();
            let uname = capitalize(&name_str);
            let set_ty = if p.is_named_slot {
                format_ident!("{}_slot_{}_already_filled", fn_name_str, name_str)
            } else {
                format_ident!("__{}Prop{}Set", fn_name_str, uname)
            };
            quote! { #set_ty }
        })
        .collect();

    let has_traits: Vec<TokenStream2> = req_indices
        .iter()
        .map(|&i| {
            let p = &params[i];
            let uname = capitalize(&p.ident.to_string());
            let has_tr = format_ident!("__{}Has{}", fn_name_str, uname);
            quote! { #has_tr }
        })
        .collect();

    let not_filled_traits: Vec<Option<TokenStream2>> = req_indices
        .iter()
        .map(|&i| {
            let p = &params[i];
            if p.is_named_slot {
                let uname = capitalize(&p.ident.to_string());
                let tr = format_ident!("__{}{}NotYetFilled", fn_name_str, uname);
                Some(quote! { #tr })
            } else {
                None
            }
        })
        .collect();

    // The struct's generic parameter list: `<__S0 = Unset0, __S1 = Unset1, ...>`
    let struct_generics_params: Vec<TokenStream2> = state_param_idents
        .iter()
        .zip(unset_defaults.iter())
        .map(|(sp, ud)| quote! { #sp = #ud })
        .collect();

    // The struct's type argument list (for use in `impl<..> PropsName<..>`): `<__S0, __S1, ...>`
    let struct_ty_args: Vec<&Ident> = state_param_idents.iter().collect();

    // For the all-unset constructor, the concrete type: `PropsName<Unset0, Unset1, ...>`
    let all_unset_concrete: Vec<&TokenStream2> = unset_defaults.iter().collect();

    // -----------------------------------------------------------------------
    // Constructor.
    // -----------------------------------------------------------------------

    let field_inits: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            quote! { #name: ::std::option::Option::None }
        })
        .collect();

    let constructor = quote! {
        impl #props_name<#(#all_unset_concrete),*> {
            /// Create an empty props builder (all props unset).
            pub fn new() -> Self {
                Self {
                    #(#field_inits,)*
                    _pd: ::std::marker::PhantomData,
                }
            }
        }
    };

    // -----------------------------------------------------------------------
    // Setters.
    // -----------------------------------------------------------------------
    //
    // All setters live in a single `impl<S0, S1, ...> PropsName<S0, S1, ...>`
    // block, except that named-slot setters are additionally bounded by their
    // "not yet filled" trait (placed in a *separate* impl block so that the
    // bound applies precisely).

    // Start with the "unconstrained" setters block (non-slot required params
    // and all optional params).
    let mut unconstrained_setters: Vec<TokenStream2> = Vec::new();

    // For each param build the field-initialization list to reconstruct the
    // struct in a setter.  For required params, the setter changes one field;
    // all others are carried over from `self`.
    for (param_i, p) in params.iter().enumerate() {
        let name = &p.ident;
        let ty = &p.ty;

        // Position of this param in the required-params list (if it is one).
        let req_pos = req_indices.iter().position(|&ri| ri == param_i);

        // Build the struct initializer for the setter's return value.
        // For every field: either `FieldName: Some(v)` (the one being set),
        // `FieldName: Some(::std::convert::Into::into(v))` (for `Into`),
        // or `FieldName: self.FieldName` (all others).
        let field_assignments: Vec<TokenStream2> = params
            .iter()
            .enumerate()
            .map(|(j, q)| {
                let fname = &q.ident;
                if j == param_i {
                    match &p.kind {
                        PropKind::Into => quote! {
                            #fname: ::std::option::Option::Some(
                                ::std::convert::Into::into(v)
                            )
                        },
                        _ => quote! {
                            #fname: ::std::option::Option::Some(v)
                        },
                    }
                } else {
                    quote! { #fname: self.#fname }
                }
            })
            .collect();

        match &p.kind {
            PropKind::Required | PropKind::Into if !p.is_named_slot => {
                // Non-slot required setter: transitions one state param.
                // Return type: PropsName<S0, ..., Set_i, ..., SN>
                let return_ty_args: Vec<TokenStream2> = state_param_idents
                    .iter()
                    .enumerate()
                    .map(|(j, sp)| {
                        if j == req_pos.unwrap() {
                            // This param: return Set
                            set_tys[j].clone()
                        } else {
                            // Other state params: preserve
                            quote! { #sp }
                        }
                    })
                    .collect();

                let setter = if matches!(p.kind, PropKind::Into) {
                    quote! {
                        /// Set the prop (accepts any `impl Into<T>`).
                        pub fn #name(self, v: impl ::std::convert::Into<#ty>) -> #props_name<#(#return_ty_args),*> {
                            #props_name {
                                #(#field_assignments,)*
                                _pd: ::std::marker::PhantomData,
                            }
                        }
                    }
                } else {
                    quote! {
                        /// Set the prop.
                        pub fn #name(self, v: #ty) -> #props_name<#(#return_ty_args),*> {
                            #props_name {
                                #(#field_assignments,)*
                                _pd: ::std::marker::PhantomData,
                            }
                        }
                    }
                };
                unconstrained_setters.push(setter);
            }

            PropKind::Default(_) => {
                // Optional setter: preserves all state.
                let setter = quote! {
                    /// Set the prop.
                    pub fn #name(self, v: #ty) -> Self {
                        #props_name {
                            #(#field_assignments,)*
                            _pd: self._pd,
                        }
                    }
                };
                unconstrained_setters.push(setter);
            }

            PropKind::Required | PropKind::Into => {
                // Named-slot required setter: handled in a separate impl below.
            }
        }
    }

    let unconstrained_impl = if unconstrained_setters.is_empty() {
        quote! {}
    } else {
        quote! {
            impl<#(#struct_ty_args),*> #props_name<#(#struct_ty_args),*> {
                #(#unconstrained_setters)*
            }
        }
    };

    // Named-slot setters: each in its own bounded impl block.
    let mut slot_setter_impls: Vec<TokenStream2> = Vec::new();

    for (param_i, p) in params.iter().enumerate() {
        if !p.is_named_slot || !matches!(p.kind, PropKind::Required | PropKind::Into) {
            continue;
        }

        let req_pos = req_indices.iter().position(|&ri| ri == param_i).unwrap();
        let name = &p.ident;
        let ty = &p.ty;

        let not_filled_tr = not_filled_traits[req_pos].as_ref().unwrap();

        // Build field assignments for the slot setter.
        let field_assignments: Vec<TokenStream2> = params
            .iter()
            .enumerate()
            .map(|(j, q)| {
                let fname = &q.ident;
                if j == param_i {
                    quote! { #fname: ::std::option::Option::Some(v) }
                } else {
                    quote! { #fname: self.#fname }
                }
            })
            .collect();

        // Return type: PropsName<S0, ..., Set_i, ..., SN>
        let return_ty_args: Vec<TokenStream2> = state_param_idents
            .iter()
            .enumerate()
            .map(|(j, sp)| {
                if j == req_pos {
                    set_tys[j].clone()
                } else {
                    quote! { #sp }
                }
            })
            .collect();

        // The impl requires that the slot's state param implements NotYetFilled.
        // Other state params are unconstrained.
        let impl_generics_slot: Vec<TokenStream2> = state_param_idents
            .iter()
            .enumerate()
            .map(|(j, sp)| {
                if j == req_pos {
                    quote! { #sp: #not_filled_tr }
                } else {
                    quote! { #sp }
                }
            })
            .collect();

        slot_setter_impls.push(quote! {
            impl<#(#impl_generics_slot),*> #props_name<#(#struct_ty_args),*> {
                /// Fill the slot.
                pub fn #name(self, v: #ty) -> #props_name<#(#return_ty_args),*> {
                    #props_name {
                        #(#field_assignments,)*
                        _pd: ::std::marker::PhantomData,
                    }
                }
            }
        });
    }

    // -----------------------------------------------------------------------
    // `build()` — requires all "has" traits.
    // -----------------------------------------------------------------------

    let build_assignments: Vec<TokenStream2> = params
        .iter()
        .enumerate()
        .map(|(param_i, p)| {
            let name = &p.ident;
            let name_str = p.ident.to_string();
            match &p.kind {
                PropKind::Required | PropKind::Into => {
                    // safe because the type-state guarantees the field is set
                    let msg = format!("INTERNAL BUG: required prop `{name_str}` unexpectedly None");
                    let _ = param_i;
                    quote! { #name: self.#name.expect(#msg) }
                }
                PropKind::Default(default_expr) => {
                    quote! { #name: self.#name.unwrap_or_else(|| #default_expr) }
                }
            }
        })
        .collect();

    let build_impl_generics: Vec<TokenStream2> = state_param_idents
        .iter()
        .zip(has_traits.iter())
        .map(|(sp, ht)| quote! { #sp: #ht })
        .collect();

    let build_impl = quote! {
        impl<#(#build_impl_generics),*> #props_name<#(#struct_ty_args),*> {
            /// Consume the builder and produce the component value.
            ///
            /// # Compile error
            ///
            /// Fails to compile if any required prop or required slot was not set.
            pub fn build(self) -> #impl_name {
                #impl_name {
                    #(#build_assignments,)*
                }
            }
        }
    };

    // -----------------------------------------------------------------------
    // Assemble.
    // -----------------------------------------------------------------------

    let pd_tuple: Vec<&Ident> = state_param_idents.iter().collect();

    Ok(quote! {
        #impl_struct_and_trait

        #(#state_machinery)*

        #[doc = "Generated typed props builder for this component."]
        #fn_vis struct #props_name<#(#struct_generics_params),*> {
            #(#builder_fields,)*
            #[doc(hidden)]
            _pd: ::std::marker::PhantomData<(#(#pd_tuple,)*)>,
        }

        #constructor

        #unconstrained_impl

        #(#slot_setter_impls)*

        #build_impl
    })
}

// ---------------------------------------------------------------------------
// Fallback: runtime-panic builder (used for generic components)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn build_panic_builder(
    fn_vis: &syn::Visibility,
    props_name: &Ident,
    impl_name: &Ident,
    impl_generics: &syn::ImplGenerics<'_>,
    ty_generics: &syn::TypeGenerics<'_>,
    where_clause: Option<&syn::WhereClause>,
    params: &[PropParam],
    fn_name_str: &str,
) -> TokenStream2 {
    let builder_fields: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            let ty = &p.ty;
            quote! { #name: ::std::option::Option<#ty> }
        })
        .collect();

    let field_inits: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            quote! { #name: ::std::option::Option::None }
        })
        .collect();

    let setters: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            let ty = &p.ty;
            match &p.kind {
                PropKind::Into => quote! {
                    pub fn #name(mut self, v: impl ::std::convert::Into<#ty>) -> Self {
                        self.#name =
                            ::std::option::Option::Some(::std::convert::Into::into(v));
                        self
                    }
                },
                _ => quote! {
                    pub fn #name(mut self, v: #ty) -> Self {
                        self.#name = ::std::option::Option::Some(v);
                        self
                    }
                },
            }
        })
        .collect();

    let build_assignments: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            let name_str = p.ident.to_string();
            match &p.kind {
                PropKind::Required | PropKind::Into => {
                    let msg = format!("missing required prop `{name_str}` on `{fn_name_str}`");
                    quote! { #name: self.#name.expect(#msg) }
                }
                PropKind::Default(default_expr) => {
                    quote! { #name: self.#name.unwrap_or_else(|| #default_expr) }
                }
            }
        })
        .collect();

    quote! {
        #fn_vis struct #props_name #impl_generics #where_clause {
            #(#builder_fields,)*
        }

        impl #impl_generics #props_name #ty_generics #where_clause {
            pub fn new() -> Self {
                Self { #(#field_inits,)* }
            }

            #(#setters)*

            pub fn build(self) -> #impl_name #ty_generics {
                #impl_name {
                    #(#build_assignments,)*
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Simple builder (zero required params)
// ---------------------------------------------------------------------------

fn build_simple_builder(
    fn_vis: &syn::Visibility,
    props_name: &Ident,
    impl_name: &Ident,
    params: &[PropParam],
    fn_name_str: &str,
) -> TokenStream2 {
    let builder_fields: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            let ty = &p.ty;
            quote! { #name: ::std::option::Option<#ty> }
        })
        .collect();

    let field_inits: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            quote! { #name: ::std::option::Option::None }
        })
        .collect();

    let setters: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            let ty = &p.ty;
            match &p.kind {
                PropKind::Into => quote! {
                    pub fn #name(mut self, v: impl ::std::convert::Into<#ty>) -> Self {
                        self.#name =
                            ::std::option::Option::Some(::std::convert::Into::into(v));
                        self
                    }
                },
                _ => quote! {
                    pub fn #name(mut self, v: #ty) -> Self {
                        self.#name = ::std::option::Option::Some(v);
                        self
                    }
                },
            }
        })
        .collect();

    // All params are optional (default or into-with-default).
    // `Required` without defaults should not appear here (n_req == 0 check).
    let build_assignments: Vec<TokenStream2> = params
        .iter()
        .map(|p| {
            let name = &p.ident;
            let name_str = p.ident.to_string();
            match &p.kind {
                PropKind::Required | PropKind::Into => {
                    // Should not reach here, but fall back to panic.
                    let msg = format!("missing required prop `{name_str}` on `{fn_name_str}`");
                    quote! { #name: self.#name.expect(#msg) }
                }
                PropKind::Default(default_expr) => {
                    quote! { #name: self.#name.unwrap_or_else(|| #default_expr) }
                }
            }
        })
        .collect();

    quote! {
        #fn_vis struct #props_name {
            #(#builder_fields,)*
        }

        impl #props_name {
            pub fn new() -> Self {
                Self { #(#field_inits,)* }
            }

            #(#setters)*

            pub fn build(self) -> #impl_name {
                #impl_name {
                    #(#build_assignments,)*
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: extract PropKind from attributes
// ---------------------------------------------------------------------------

fn extract_prop_kind(attrs: &[Attribute]) -> syn::Result<PropKind> {
    for attr in attrs {
        if attr.path().is_ident("prop") {
            return attr.parse_args_with(|input: ParseStream| -> syn::Result<PropKind> {
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
            });
        }
    }
    Ok(PropKind::Required)
}
