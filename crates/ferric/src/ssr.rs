//! Server-side rendering: render a component closure to an HTML string.
//!
//! This module owns the SSR render path. It compiles and runs off-wasm on a native host;
//! it must never reference `web_sys` or `wasm_bindgen`.
//!
//! # Text-boundary markers
//!
//! The server render wraps every `VNode::Text` node in HTML comment sentinels so the client
//! hydration walk can locate text nodes by index without counting the comment nodes as
//! structural VNode children. Both sides use the constants defined here — never inline
//! literal copies.
//!
//! # Implementation
//!
//! Stub only — the render logic is owned by the `ferric-ssr-r1-server-render` spoke.

use crate::vdom::VNode;

/// HTML comment that opens a text-boundary region around a `VNode::Text` node.
///
/// The server emits this immediately before the escaped text content. The client's
/// hydration walk recognises this sentinel and does not count it when advancing the
/// VNode child index.  Both sides MUST use this exact literal — never define a
/// local copy.
pub const TEXT_OPEN_MARKER: &str = "<!--$t-->";

/// HTML comment that closes a text-boundary region.
///
/// Paired with [`TEXT_OPEN_MARKER`]. The client's hydration walk recognises this
/// sentinel and skips it when counting structural VNode children.  Both sides MUST
/// use this exact literal — never define a local copy.
pub const TEXT_CLOSE_MARKER: &str = "<!--/$t-->";

/// Render a component closure to an HTML string.
///
/// `f` is called once inside a fresh reactive scope; that scope is dropped before
/// this function returns, so no signal subscriptions survive the call.  Text content
/// and attribute values are HTML-escaped; `VNode::Text` nodes are wrapped in
/// [`TEXT_OPEN_MARKER`] / [`TEXT_CLOSE_MARKER`] sentinels.
///
/// # Compile target
///
/// This function compiles and runs on a native (non-wasm) host target.  It must not
/// import `web_sys` or `wasm_bindgen`.
///
/// # Panics
///
/// Panics (via `todo!()`) until the implementation is provided by the
/// `ferric-ssr-r1-server-render` spoke.
pub fn render_to_string<F>(f: F) -> String
where
    F: FnOnce() -> VNode,
{
    todo!()
}
