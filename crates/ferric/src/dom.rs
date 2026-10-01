//! The browser runtime: render a component's view tree into real DOM via `wasm-bindgen` +
//! `web-sys`, re-render through a keyed patch when signals change, and bind events.
//!
//! STUB: the public surface below is frozen by `contracts/ferric_api.rs`; the runtime is
//! implemented by `SUBTASK_dom_runtime`, which replaces the `todo!()` body here.
//!
//! Note for the implementing layer: the frozen contract carries a commented-out
//! `root: web_sys::Element` parameter on [`mount`] and hints that [`crate::vdom::EventHandler`]
//! may need to carry a `web_sys::Event`. The `web-sys` features those require are already enabled
//! in `Cargo.toml`; realizing them is an owner-latitude refinement / additive contract amendment
//! for this layer (and the view layer), not a scaffold change.

use crate::component::Component;

/// Mount `component` into the DOM, render it, and keep it reactively updated: the runtime wraps
/// rendering in [`crate::signal::create_effect`], so a signal change re-renders and applies the
/// keyed [`crate::vdom::diff`] patch to the live DOM. Event handlers from the view tree are bound
/// to DOM events.
pub fn mount(component: impl Component + 'static) {
    let _ = component;
    todo!()
}
