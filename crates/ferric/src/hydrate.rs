//! Client-side hydration: adopt a server-rendered DOM subtree without re-rendering it.
//!
//! This module is only compiled when `target_arch = "wasm32"` (gated at the
//! `crate::lib` module declaration).
//!
//! `hydrate` adopts the server-rendered DOM under `root` with zero structural
//! mutations — no nodes are created, replaced, moved, or removed during the adopt
//! walk.  Event handlers from the initial view tree are wired onto the adopted
//! elements, and a reactive effect is established so subsequent signal changes drive
//! the existing keyed-patch path exactly as `dom::mount` does after its first render.
//!
//! Text-boundary comment sentinels (`crate::ssr::TEXT_OPEN_MARKER` /
//! `TEXT_CLOSE_MARKER`) are recognised during the walk and are not counted as
//! structural VNode children.
//!
//! # Implementation
//!
//! Stub only — the hydration logic is owned by the `ferric-ssr-r1-client-hydrate`
//! spoke.

use crate::vdom::VNode;

/// Adopt a server-rendered DOM subtree rooted at `root` and make it interactive.
///
/// `f` is called once to obtain the initial `VNode` tree — the same render the
/// server ran.  The existing DOM under `root` is walked and paired with the VNode
/// tree; event handlers are wired onto the adopted elements and a reactive effect
/// is established so later signal writes diff and patch the live DOM through the
/// existing keyed-patch path.
///
/// # Compile target
///
/// Only compiled when `target_arch = "wasm32"`.
///
/// # Panics
///
/// Panics (via `todo!()`) until the implementation is provided by the
/// `ferric-ssr-r1-client-hydrate` spoke.
#[cfg(target_arch = "wasm32")]
pub fn hydrate<F>(root: web_sys::Element, f: F)
where
    F: Fn() -> VNode + 'static,
{
    todo!()
}
