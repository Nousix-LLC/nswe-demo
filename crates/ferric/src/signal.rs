//! Fine-grained reactivity (Solid/Leptos-style): [`Signal`], [`Memo`], effects, and batching.
//!
//! A [`Signal`] is a cheap `Copy` handle into a reactive runtime. Reads inside a reactive context
//! (an effect or a memo) subscribe automatically; writes notify subscribers. [`batch`] coalesces
//! notifications, and dropped effects must unsubscribe (the `no_stale_subscriber_leak` identity).
//!
//! STUB: the public surface below is frozen by `contracts/ferric_api.rs`; the reactive runtime
//! itself is implemented by `SUBTASK_reactive_core`, which replaces the `todo!()` bodies here.

use std::marker::PhantomData;

/// A readable + writable reactive value handle. Cheap to `Clone`/`Copy` (it is a key into the
/// reactive runtime, not the value itself).
pub struct Signal<T: 'static> {
    // Owner-designed internal representation (runtime key). The `PhantomData<T>` ties the handle
    // to its value type without storing one; `SUBTASK_reactive_core` replaces this.
    _marker: PhantomData<T>,
}

impl<T: 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for Signal<T> {}

impl<T: 'static> Signal<T> {
    /// Read the value, cloning it out. Subscribes the current reactive observer (if any).
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        todo!()
    }

    /// Read by reference without cloning. Subscribes the current reactive observer (if any).
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let _ = f;
        todo!()
    }

    /// Replace the value and notify subscribers.
    pub fn set(&self, value: T) {
        let _ = value;
        todo!()
    }

    /// Mutate the value in place and notify subscribers.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let _ = f;
        todo!()
    }
}

/// Create a reactive signal with an initial value.
pub fn create_signal<T: 'static>(value: T) -> Signal<T> {
    let _ = value;
    todo!()
}

/// A derived/computed value. Recomputes lazily when its reactive dependencies change; memoizes
/// the result between changes.
pub struct Memo<T: 'static> {
    // Owner-designed internal representation; see `Signal`.
    _marker: PhantomData<T>,
}

impl<T: 'static> Clone for Memo<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for Memo<T> {}

impl<T: 'static> Memo<T> {
    /// Read the memoized value, cloning it out. Subscribes the current reactive observer (if any).
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        todo!()
    }

    /// Read the memoized value by reference. Subscribes the current reactive observer (if any).
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let _ = f;
        todo!()
    }
}

/// Create a memoized derived value from a closure that reads other signals/memos.
pub fn create_memo<T: 'static>(f: impl Fn() -> T + 'static) -> Memo<T> {
    let _ = f;
    todo!()
}

/// Register a reactive side effect. Runs once immediately (tracking its signal reads), then
/// re-runs whenever any tracked dependency changes. The DOM runtime builds re-render on this.
pub fn create_effect(f: impl FnMut() + 'static) {
    let _ = f;
    todo!()
}

/// Run `f`, coalescing all signal notifications into a single flush at the end (batched update).
pub fn batch<R>(f: impl FnOnce() -> R) -> R {
    let _ = f;
    todo!()
}
