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

/// The frozen public surface of `ferric`.
///
/// `use ferric::prelude::*;` brings the core API into scope: reactive primitives, the view-tree
/// node and element types, the event kinds, the [`Component`](crate::component::Component) trait,
/// and [`mount`](crate::dom::mount). This set is the `public_api_surface` identity the synthesis
/// gate asserts; additions are additive-only.
pub mod prelude {
    pub use crate::component::Component;
    pub use crate::dom::mount;
    pub use crate::signal::{batch, create_effect, create_memo, create_signal, Memo, Signal};
    pub use crate::vdom::{EventKind, VElement, VNode};
}
