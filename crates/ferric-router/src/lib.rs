//! `ferric-router` — a client-side router for the [`ferric`] WebAssembly framework.
//!
//! The router pairs a declarative route table with path-parameter matching ([`matcher`]), the
//! History-API navigation and reactive routing state ([`router`]), and a declarative in-app
//! navigation link ([`link`]). It composes `ferric`'s signals and component model for
//! reactivity — the outlet re-renders when the current path changes — rather than
//! re-implementing reactivity or the DOM diff.
//!
//! This file is the **walking-skeleton composition root**: it fixes the module layout and the
//! frozen public surface (see `contracts/ferric_router_api.rs`). Each module's body is
//! implemented by its owning work-item; until then the leaf items are `todo!()` placeholders.
//!
//! # Example
//!
//! ```ignore
//! use ferric_router::prelude::*;
//! // A 2–3 route app lives in `crates/ferric-router/examples/app.rs`.
//! ```
//!
//! [`ferric`]: ferric

pub mod link;
pub mod matcher;
pub mod router;

/// The frozen public surface of `ferric-router`.
///
/// `use ferric_router::prelude::*;` brings the router API into scope: the matcher types
/// ([`Params`](crate::matcher::Params), [`RoutePattern`](crate::matcher::RoutePattern)), the
/// routing layer ([`Router`](crate::router::Router), [`navigate`](crate::router::navigate),
/// [`current_path`](crate::router::current_path)), and the navigation [`link`](crate::link::link)
/// helper. This set is the `public_api_surface` identity the synthesis gate asserts; additions
/// are additive-only.
pub mod prelude {
    pub use crate::link::link;
    pub use crate::matcher::{Params, RoutePattern};
    pub use crate::router::{current_path, navigate, Router};
}
