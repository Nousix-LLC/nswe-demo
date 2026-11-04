//! `ferric` — a small client-side web-application framework in Rust, compiled to WebAssembly.
//!
//! `ferric` pairs fine-grained reactivity ([`signal`]) with a component/view model
//! ([`component`] + [`vdom`]) and a `wasm-bindgen`/`web-sys` browser runtime ([`dom`]):
//! signals drive effects, components render a [`vdom::VNode`] tree, a keyed [`vdom::diff`]
//! computes the minimal patch, and [`dom::mount`] applies it to the live DOM and re-applies it
//! whenever a read signal changes.
//!
//! This file is the **walking-skeleton composition root**: it fixes the module layout and the
//! frozen public surface (see `contracts/ferric_api.rs`). Each module's body is implemented by
//! its owning work-item; until then the leaf items are `todo!()` placeholders.
//!
//! # Example
//!
//! ```ignore
//! use ferric::prelude::*;
//! // A full counter lives in `crates/ferric/examples/counter.rs`.
//! ```

pub mod component;
pub mod dom;
pub mod signal;
pub mod vdom;

// SSR + hydration modules (additive — frozen modules above are unchanged).
// `ssr` is always compiled (native and wasm32); it must not import web_sys or
// wasm_bindgen.  `hydrate` is WASM-only: it uses web_sys::Element and DOM APIs.
pub mod ssr;
#[cfg(target_arch = "wasm32")]
pub mod hydrate;

// Top-level re-exports: the `view!` macro's generated code references these
// types via `::ferric::VElement`, `::ferric::VNode`, and `::ferric::EventKind`,
// so they must be accessible at the crate root in addition to their canonical
// path `ferric::vdom::*`.  Additive-only — the `ferric::vdom` module and the
// `prelude` are unchanged.
pub use crate::vdom::{EventKind, VElement, VNode};

// SSR + hydration re-exports (additive — not in the frozen prelude block).
// `render_to_string` is always reachable; `hydrate` is wasm32-only.
pub use crate::ssr::render_to_string;
#[cfg(target_arch = "wasm32")]
pub use crate::hydrate::hydrate;

/// The frozen public surface of `ferric`.
///
/// `use ferric::prelude::*;` brings the core API into scope: reactive primitives, the view-tree
/// node and element types, the event kinds, the [`Component`](crate::component::Component) trait,
/// and [`mount`](crate::dom::mount). This set is the `public_api_surface` identity the synthesis
/// gate asserts; additions are additive-only.
pub mod prelude {
    pub use crate::component::{Children, Component, Slot, SlotFn};
    pub use crate::dom::mount;
    pub use crate::signal::{
        batch, create_effect, create_memo, create_signal, expect_context, provide_context,
        use_context, Memo, Signal,
    };
    pub use crate::vdom::{EventKind, VElement, VNode};
    pub use ferric_macros::{component, view};
}
