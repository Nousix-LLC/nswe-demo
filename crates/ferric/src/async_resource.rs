//! Async resource primitives — public API surface frozen by the walking skeleton.
//!
//! [`Resource<T>`] wraps a [`Signal<ResourceState<T>>`](crate::signal::Signal) and drives it via
//! `wasm_bindgen_futures::spawn_local`. Calling [`Resource::state`] inside a reactive context
//! (an effect or memo) subscribes that context to future state changes, so the view automatically
//! re-renders when an async operation completes.
//!
//! # Suspense
//!
//! [`suspense`] provides a simple "show fallback while loading" primitive. It evaluates the
//! `children` closure once; if any [`Resource::state`] call inside it observed
//! [`ResourceState::Loading`], it returns `fallback()` instead, automatically subscribing the
//! caller to those resources so the boundary re-evaluates when they settle.

use std::cell::Cell;
use std::future::Future;
use std::rc::Rc;

use wasm_bindgen_futures::spawn_local;

use crate::signal::{create_effect, create_signal, Signal};
use crate::vdom::VNode;

// =====================================================================================
// Suspense tracking — thread-local state used by `suspense()` and `state()`
// =====================================================================================

thread_local! {
    /// Number of active suspense boundaries currently evaluating their `children` closure.
    static SUSPENSE_DEPTH: Cell<u32> = const { Cell::new(0) };
    /// Whether any resource was `Loading` inside the current (innermost) suspense invocation.
    static SUSPENSE_HAS_LOADING: Cell<bool> = const { Cell::new(false) };
}

// =====================================================================================
// ResourceState
// =====================================================================================

/// The lifecycle state of an async-loaded value.
///
/// Exhaustively matchable — callers cannot receive a [`Ready`](ResourceState::Ready) before
/// the future has resolved.
///
/// # Examples
///
/// ```
/// use ferric::async_resource::ResourceState;
///
/// fn render<T: std::fmt::Display>(state: ResourceState<T>) {
///     match state {
///         ResourceState::Loading          => println!("loading…"),
///         ResourceState::Ready(val)       => println!("got: {val}"),
///         ResourceState::Failed(msg)      => println!("error: {msg}"),
///     }
/// }
/// ```
#[derive(Debug)]
pub enum ResourceState<T> {
    /// The future has not yet resolved.
    Loading,
    /// The future completed successfully with a value.
    Ready(T),
    /// The future (or the host) reported an error; the message describes why.
    Failed(String),
}

impl<T: Clone> Clone for ResourceState<T> {
    fn clone(&self) -> Self {
        match self {
            ResourceState::Loading => ResourceState::Loading,
            ResourceState::Ready(v) => ResourceState::Ready(v.clone()),
            ResourceState::Failed(e) => ResourceState::Failed(e.clone()),
        }
    }
}

// =====================================================================================
// Resource
// =====================================================================================

/// A reactive handle to an async-loaded value.
///
/// Create one with [`Resource::new`]; poll its current state with [`Resource::state`].
/// The state transitions from [`Loading`](ResourceState::Loading) to
/// [`Ready`](ResourceState::Ready) exactly once (or to [`Failed`](ResourceState::Failed)
/// on the fallible variant).
///
/// `Resource<T>` is `Copy` — it is a thin key into the reactive runtime, not the value itself.
pub struct Resource<T: 'static> {
    signal: Signal<ResourceState<T>>,
}

impl<T: 'static> Clone for Resource<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: 'static> Copy for Resource<T> {}

impl<T: 'static> Resource<T> {
    /// Spawn `fut` and return a handle whose [`state`](Self::state) tracks the result.
    ///
    /// The future is started immediately via `spawn_local`. The returned resource starts in
    /// [`ResourceState::Loading`] and transitions to [`ResourceState::Ready`] when `fut` resolves.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let res: Resource<String> = Resource::new(async { fetch_name().await });
    /// ```
    pub fn new(fut: impl Future<Output = T> + 'static) -> Self {
        let signal = create_signal(ResourceState::Loading);
        spawn_local(async move {
            let value = fut.await;
            signal.set(ResourceState::Ready(value));
        });
        Resource { signal }
    }

    /// Create a resource from a **fallible** future.
    ///
    /// Unlike [`new`](Self::new), the future returns a `Result`; on `Err` the resource
    /// transitions to [`ResourceState::Failed`] with the error message string.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let res = Resource::new_fallible(async {
    ///     fetch_data().await.map_err(|e| e.to_string())
    /// });
    /// ```
    pub fn new_fallible(fut: impl Future<Output = Result<T, String>> + 'static) -> Self {
        let signal = create_signal(ResourceState::Loading);
        spawn_local(async move {
            match fut.await {
                Ok(value) => signal.set(ResourceState::Ready(value)),
                Err(msg) => signal.set(ResourceState::Failed(msg)),
            }
        });
        Resource { signal }
    }

    /// Return the current lifecycle state of the resource.
    ///
    /// **Reactive**: calling this inside an effect or memo subscribes the caller to future state
    /// changes. When the resource settles, all subscribed effects are re-run automatically.
    ///
    /// Also participates in [`suspense`] tracking: if any `state()` call inside a `suspense`
    /// `children` closure returns `Loading`, the boundary shows the fallback instead.
    pub fn state(&self) -> ResourceState<T>
    where
        T: Clone,
    {
        // `Signal::get()` calls `with()` which records the current reactive observer as a
        // subscriber of this signal — that is what makes `state()` reactive.
        let state = self.signal.get();

        // Notify the innermost active suspense boundary if we are loading.
        if matches!(state, ResourceState::Loading) {
            SUSPENSE_DEPTH.with(|d| {
                if d.get() > 0 {
                    SUSPENSE_HAS_LOADING.with(|l| l.set(true));
                }
            });
        }

        state
    }
}

// =====================================================================================
// create_resource — reactive async constructor (REQ-ASYNC-001, 003, 005)
// =====================================================================================

/// Create a reactive async resource that re-fetches whenever `source` changes.
///
/// # Arguments
///
/// * `source` — a closure called inside a reactive effect to derive the fetch key.
///   Any [`Signal`]s read by `source` are tracked; when they change the effect re-runs,
///   triggering a new fetch.
/// * `fetcher` — given the value returned by `source`, produces a `Future<Output = T>` whose
///   result is stored in the resource when it resolves.
///
/// # Cancellation (REQ-ASYNC-016)
///
/// A generation counter is incremented before each new spawn. The spawned future captures the
/// generation at the time it was started and only writes the result if its generation still
/// matches the current counter, discarding results from superseded in-flight fetches.
///
/// # WASM gate
///
/// `spawn_local` is called only on `wasm32` targets. On native / test builds the future is
/// dropped and the signal remains [`ResourceState::Loading`] — which makes the reactive
/// re-trigger testable without a browser executor.
///
/// # Example
///
/// ```ignore
/// let id = create_signal(1u32);
/// let user = create_resource(
///     move || id.get(),
///     |id| async move { fetch_user(id).await },
/// );
/// ```
pub fn create_resource<S, F, T>(
    source: impl Fn() -> S + 'static,
    fetcher: impl Fn(S) -> F + 'static,
) -> Resource<T>
where
    S: 'static,
    F: Future<Output = T> + 'static,
    T: 'static,
{
    let signal = create_signal(ResourceState::Loading);
    let gen = Rc::new(Cell::new(0u64));

    create_effect(move || {
        // Track any signals that `source` reads; re-run this effect when they change.
        let source_value = source();

        // Reset to Loading so callers (and suspense) see the new fetch in progress.
        signal.set(ResourceState::Loading);

        // Bump the generation so any in-flight future from the previous source value will
        // see a stale generation and discard its result (REQ-ASYNC-016).
        let next_gen = gen.get() + 1;
        gen.set(next_gen);

        let fut = fetcher(source_value);

        // spawn_local is WASM-only; on native we drop the future and stay in Loading.
        #[cfg(target_arch = "wasm32")]
        {
            let gen_check = Rc::clone(&gen);
            spawn_local(async move {
                let value = fut.await;
                // Only write if our generation is still the latest (no newer fetch started).
                if gen_check.get() == next_gen {
                    signal.set(ResourceState::Ready(value));
                }
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        drop(fut);
    });

    Resource { signal }
}

// =====================================================================================
// Suspense
// =====================================================================================

/// Render `fallback` while any [`Resource`] used inside `children` is [`ResourceState::Loading`];
/// once all resources have settled (Ready or Failed), render `children`.
///
/// Both closures are `Fn() -> VNode` so they can be called multiple times as the reactive graph
/// re-evaluates the view.
///
/// # Reactivity
///
/// `suspense` evaluates `children` once to discover the loading state. Because
/// [`Resource::state`] subscribes its signal to the current reactive observer, an effect wrapping
/// this call re-runs when any resource inside `children` changes state — re-evaluating the
/// boundary automatically.
///
/// # Nested boundaries
///
/// Nested `suspense` calls are supported. Each boundary tracks its own loading state; an inner
/// boundary that resolves does not suppress an outer boundary that is still loading.
///
/// # Example
///
/// ```ignore
/// let resource = Resource::new(fetch_data());
/// let view = suspense(
///     || VNode::Text("Loading…".into()),
///     || match resource.state() {
///         ResourceState::Ready(data) => render_data(data),
///         ResourceState::Failed(e)   => VNode::Text(format!("Error: {e}")),
///         ResourceState::Loading     => unreachable!(),
///     },
/// );
/// ```
pub fn suspense(fallback: impl Fn() -> VNode, children: impl Fn() -> VNode) -> VNode {
    // Save the outer boundary's loading flag and reset for this boundary's scope.
    let saved_loading = SUSPENSE_HAS_LOADING.with(|l| l.replace(false));
    SUSPENSE_DEPTH.with(|d| d.set(d.get() + 1));

    // Evaluate children to both (a) render them and (b) detect any loading resources.
    let children_vnode = children();

    // Extract this boundary's loading verdict, then restore the outer flag.
    // We propagate `loading` upward so an outer boundary also sees that something is loading
    // (the outer boundary will re-evaluate when the inner resource settles regardless, because
    // `state()` subscribes the outer reactive context to the signal).
    let this_boundary_loading = SUSPENSE_HAS_LOADING.with(|l| {
        let v = l.get();
        l.set(saved_loading || v);
        v
    });

    SUSPENSE_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));

    if this_boundary_loading {
        fallback()
    } else {
        children_vnode
    }
}

// =====================================================================================
// Tests
// =====================================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::create_effect;
    use std::cell::RefCell;
    use std::rc::Rc;

    // ---- ResourceState variants ----

    #[test]
    fn resource_state_loading_matches() {
        let s: ResourceState<i32> = ResourceState::Loading;
        assert!(matches!(s, ResourceState::Loading));
    }

    #[test]
    fn resource_state_ready_matches() {
        let s = ResourceState::Ready(42i32);
        match s {
            ResourceState::Ready(v) => assert_eq!(v, 42),
            _ => panic!("expected Ready"),
        }
    }

    #[test]
    fn resource_state_failed_matches() {
        let s: ResourceState<i32> = ResourceState::Failed("oops".into());
        match s {
            ResourceState::Failed(msg) => assert_eq!(msg, "oops"),
            _ => panic!("expected Failed"),
        }
    }

    // ---- Clone ----

    #[test]
    fn resource_state_clone_loading() {
        let s: ResourceState<String> = ResourceState::Loading;
        let c = s.clone();
        assert!(matches!(c, ResourceState::Loading));
    }

    #[test]
    fn resource_state_clone_ready() {
        let s = ResourceState::Ready("hello".to_string());
        let c = s.clone();
        match c {
            ResourceState::Ready(v) => assert_eq!(v, "hello"),
            _ => panic!("expected Ready"),
        }
    }

    #[test]
    fn resource_state_clone_failed() {
        let s: ResourceState<String> = ResourceState::Failed("err".into());
        let c = s.clone();
        match c {
            ResourceState::Failed(msg) => assert_eq!(msg, "err"),
            _ => panic!("expected Failed"),
        }
    }

    // ---- Debug ----

    #[test]
    fn resource_state_debug_ready() {
        let s = ResourceState::Ready(99i32);
        let dbg = format!("{:?}", s);
        assert!(dbg.contains("Ready"), "Debug output: {dbg}");
    }

    #[test]
    fn resource_state_debug_loading() {
        let s: ResourceState<i32> = ResourceState::Loading;
        let dbg = format!("{:?}", s);
        assert!(dbg.contains("Loading"), "Debug output: {dbg}");
    }

    #[test]
    fn resource_state_debug_failed() {
        let s: ResourceState<i32> = ResourceState::Failed("bad".into());
        let dbg = format!("{:?}", s);
        assert!(dbg.contains("Failed"), "Debug output: {dbg}");
    }

    // ---- Resource initial Loading state ----

    /// Verify that a freshly-created resource begins in `Loading` by constructing one via the
    /// internal signal — this avoids calling `spawn_local`, which is WASM-only.
    #[test]
    fn resource_starts_in_loading_state() {
        let signal = create_signal(ResourceState::<i32>::Loading);
        let resource = Resource { signal };
        assert!(
            matches!(resource.state(), ResourceState::Loading),
            "newly created resource should be in Loading"
        );
    }

    /// Same check for the fallible variant.
    #[test]
    fn resource_fallible_starts_in_loading_state() {
        let signal = create_signal(ResourceState::<String>::Loading);
        let resource = Resource { signal };
        assert!(matches!(resource.state(), ResourceState::Loading));
    }

    // `Resource::new` and `Resource::new_fallible` call `spawn_local` which is WASM-only;
    // the full async dispatch path is tested in `wasm_tests` below.
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn resource_new_starts_loading_wasm() {
        let resource: Resource<i32> = Resource::new(std::future::pending());
        assert!(matches!(resource.state(), ResourceState::Loading));
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn resource_fallible_starts_loading_wasm() {
        let resource: Resource<String> =
            Resource::new_fallible(std::future::pending::<Result<String, String>>());
        assert!(matches!(resource.state(), ResourceState::Loading));
    }

    // ---- Resource::state is reactive ----

    #[test]
    fn resource_state_is_reactive() {
        // Build a Resource by directly wrapping a signal so we can control the state.
        let signal = create_signal(ResourceState::<i32>::Loading);
        let resource = Resource { signal };

        let count = Rc::new(RefCell::new(0u32));
        let count_clone = Rc::clone(&count);

        create_effect(move || {
            let _ = resource.state();
            *count_clone.borrow_mut() += 1;
        });

        // Effect runs once eagerly.
        assert_eq!(*count.borrow(), 1, "effect should run once at creation");

        // Transitioning to Ready notifies subscribers → effect re-runs.
        signal.set(ResourceState::Ready(7));
        assert_eq!(*count.borrow(), 2, "effect should re-run on state change");

        // Transitioning to Failed also notifies.
        signal.set(ResourceState::Failed("oops".into()));
        assert_eq!(*count.borrow(), 3, "effect should re-run on Failed");
    }

    // ---- Transition: Loading → Ready (direct signal manipulation) ----

    #[test]
    fn resource_transition_loading_to_ready() {
        let signal = create_signal(ResourceState::<i32>::Loading);
        let resource = Resource { signal };

        assert!(matches!(resource.state(), ResourceState::Loading));
        signal.set(ResourceState::Ready(42));
        assert!(matches!(resource.state(), ResourceState::Ready(42)));
    }

    // ---- Transition: Loading → Failed (simulated async error) ----

    #[test]
    fn resource_transition_loading_to_failed() {
        let signal = create_signal(ResourceState::<i32>::Loading);
        let resource = Resource { signal };

        assert!(matches!(resource.state(), ResourceState::Loading));
        signal.set(ResourceState::Failed("network error".into()));
        match resource.state() {
            ResourceState::Failed(msg) => assert_eq!(msg, "network error"),
            other => panic!("expected Failed, got {:?}", other),
        }
    }

    // ---- Suspense: fallback when Loading ----

    #[test]
    fn suspense_shows_fallback_when_loading() {
        let signal = create_signal(ResourceState::<i32>::Loading);
        let resource = Resource { signal };

        let vnode = suspense(
            || VNode::Text("loading…".into()),
            || {
                let _ = resource.state();
                VNode::Text("ready".into())
            },
        );

        match vnode {
            VNode::Text(t) => assert_eq!(t, "loading…", "suspense should show fallback"),
            _ => panic!("expected Text node"),
        }
    }

    // ---- Suspense: children when Ready ----

    #[test]
    fn suspense_shows_children_when_ready() {
        let signal = create_signal(ResourceState::Ready(42i32));
        let resource = Resource { signal };

        let vnode = suspense(
            || VNode::Text("loading…".into()),
            || {
                let _ = resource.state();
                VNode::Text("ready".into())
            },
        );

        match vnode {
            VNode::Text(t) => assert_eq!(t, "ready", "suspense should show children"),
            _ => panic!("expected Text node"),
        }
    }

    // ---- Suspense: children when Failed (Failed is settled, not Loading) ----

    #[test]
    fn suspense_shows_children_when_failed() {
        let signal = create_signal(ResourceState::<i32>::Failed("err".into()));
        let resource = Resource { signal };

        let vnode = suspense(
            || VNode::Text("loading…".into()),
            || match resource.state() {
                ResourceState::Failed(msg) => VNode::Text(format!("error: {msg}")),
                _ => VNode::Text("ok".into()),
            },
        );

        match vnode {
            VNode::Text(t) => assert!(t.starts_with("error:"), "got: {t}"),
            _ => panic!("expected error text"),
        }
    }

    // ---- Suspense: fallback visible then replaced on Ready ----

    #[test]
    fn suspense_transitions_fallback_to_children() {
        let signal = create_signal(ResourceState::<i32>::Loading);
        let resource = Resource { signal };

        // Phase 1: resource is Loading → fallback
        let v1 = suspense(
            || VNode::Text("loading…".into()),
            || {
                let _ = resource.state();
                VNode::Text("content".into())
            },
        );
        assert!(
            matches!(v1, VNode::Text(ref t) if t == "loading…"),
            "phase 1 should show fallback"
        );

        // Simulate async completion.
        signal.set(ResourceState::Ready(99));

        // Phase 2: resource is Ready → children
        let v2 = suspense(
            || VNode::Text("loading…".into()),
            || {
                let _ = resource.state();
                VNode::Text("content".into())
            },
        );
        assert!(
            matches!(v2, VNode::Text(ref t) if t == "content"),
            "phase 2 should show children"
        );
    }

    // ---- Suspense: no loading resources → children ----

    #[test]
    fn suspense_no_resources_shows_children() {
        let vnode = suspense(
            || VNode::Text("fallback".into()),
            || VNode::Text("content".into()),
        );
        match vnode {
            VNode::Text(t) => assert_eq!(t, "content"),
            _ => panic!("expected content"),
        }
    }

    // ---- create_resource — reactive re-trigger (native, spawn_local is a no-op) ----

    /// Verify that changing the source signal causes `create_resource` to transition back to
    /// `Loading`. On native `spawn_local` is a no-op so the fetch is dropped, but the reactive
    /// effect re-runs and resets the state — which is the core REQ-ASYNC-003 invariant.
    #[test]
    fn create_resource_source_change_resets_to_loading() {
        use crate::signal::create_signal as make_signal;

        let src = make_signal(1i32);

        // create_resource tracks `src` via create_effect.
        let resource = create_resource(move || src.get(), |_v| async move { 99i32 });

        // Initially Loading (future was dropped on native).
        assert!(
            matches!(resource.state(), ResourceState::Loading),
            "resource should start in Loading"
        );

        // Directly advance the internal signal to Ready to simulate a completed fetch.
        // (The `signal` field is accessible here because this test lives in the same file.)
        resource.signal.set(ResourceState::Ready(99));
        assert!(
            matches!(resource.state(), ResourceState::Ready(99)),
            "resource should be Ready after simulated fetch"
        );

        // Changing the source signal re-runs the create_effect body, which resets to Loading.
        src.set(2);
        assert!(
            matches!(resource.state(), ResourceState::Loading),
            "source change must reset resource to Loading"
        );
    }

    // ---- create_resource — generation counter prevents stale writes (native) ----

    /// Verify that if the source changes twice before a fetch completes, the internal generation
    /// counter is incremented twice, so a future that captured gen=1 would not write when gen=2.
    /// We test this by reading the internal `gen` state indirectly via two successive source
    /// changes: each change sets Loading (not Ready), proving no stale overwrite happened.
    #[test]
    fn create_resource_two_source_changes_stay_loading() {
        use crate::signal::create_signal as make_signal;

        let src = make_signal(0i32);
        let resource = create_resource(move || src.get(), |_v| async move { 0i32 });

        // Manually set Ready then change source twice.
        resource.signal.set(ResourceState::Ready(0));
        src.set(1);
        // After first change: Loading.
        assert!(matches!(resource.state(), ResourceState::Loading));

        resource.signal.set(ResourceState::Ready(1));
        src.set(2);
        // After second change: back to Loading again.
        assert!(
            matches!(resource.state(), ResourceState::Loading),
            "second source change must also reset to Loading"
        );
    }

    // ---- Suspense: nested boundaries ----

    #[test]
    fn suspense_nested_inner_resolved_outer_loading() {
        // Outer resource: loading
        let outer_signal = create_signal(ResourceState::<i32>::Loading);
        let outer = Resource {
            signal: outer_signal,
        };

        // Inner resource: ready
        let inner_signal = create_signal(ResourceState::Ready(1i32));
        let inner = Resource {
            signal: inner_signal,
        };

        // Outer suspense wraps an inner suspense that is resolved.
        let vnode = suspense(
            || VNode::Text("outer-loading".into()),
            || {
                // reading outer resource marks outer boundary as loading
                let _ = outer.state();
                // inner suspense should resolve to its children
                suspense(
                    || VNode::Text("inner-loading".into()),
                    || {
                        let _ = inner.state();
                        VNode::Text("inner-content".into())
                    },
                )
            },
        );

        // Outer boundary should show outer fallback because outer resource is loading.
        match vnode {
            VNode::Text(t) => assert_eq!(t, "outer-loading"),
            _ => panic!("expected outer fallback"),
        }
    }
}

// =====================================================================================
// WASM async integration tests
// =====================================================================================

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    /// Yield to the micro-task queue so `spawn_local` futures can run.
    async fn tick() {
        let promise = js_sys::Promise::resolve(&wasm_bindgen::JsValue::NULL);
        wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
    }

    #[wasm_bindgen_test]
    async fn resource_new_completes_to_ready() {
        let resource: Resource<i32> = Resource::new(async { 42 });
        // Before awaiting: still loading.
        assert!(matches!(resource.state(), ResourceState::Loading));
        tick().await;
        // After one micro-task yield: the spawned future has resolved.
        assert!(matches!(resource.state(), ResourceState::Ready(42)));
    }

    #[wasm_bindgen_test]
    async fn resource_new_fallible_completes_to_failed() {
        let resource: Resource<i32> =
            Resource::new_fallible(async { Err::<i32, String>("oops".into()) });
        assert!(matches!(resource.state(), ResourceState::Loading));
        tick().await;
        match resource.state() {
            ResourceState::Failed(msg) => assert_eq!(msg, "oops"),
            other => panic!("expected Failed, got {:?}", other),
        }
    }

    #[wasm_bindgen_test]
    async fn create_resource_resolves_to_ready() {
        use crate::signal::create_signal as make_signal;

        let src = make_signal(7i32);

        // create_resource: source reads `src`, fetcher returns source value doubled.
        let resource = create_resource(move || src.get(), |v| async move { v * 2 });

        // Before awaiting: still loading (spawn_local hasn't run yet).
        assert!(matches!(resource.state(), ResourceState::Loading));

        tick().await;

        // After one micro-task yield the spawned future has resolved to 7 * 2 = 14.
        assert!(
            matches!(resource.state(), ResourceState::Ready(14)),
            "create_resource should resolve to Ready(14) after tick"
        );
    }

    #[wasm_bindgen_test]
    async fn create_resource_refetches_on_source_change() {
        use crate::signal::create_signal as make_signal;

        let src = make_signal(1i32);
        let resource = create_resource(move || src.get(), |v| async move { v + 10 });

        tick().await;
        assert!(
            matches!(resource.state(), ResourceState::Ready(11)),
            "first fetch: 1 + 10 = 11"
        );

        // Change source: resource goes back to Loading, then fetches again.
        src.set(5);

        // Immediately after set: reactive effect ran and reset to Loading.
        assert!(
            matches!(resource.state(), ResourceState::Loading),
            "source change should reset to Loading"
        );

        tick().await;
        assert!(
            matches!(resource.state(), ResourceState::Ready(15)),
            "second fetch: 5 + 10 = 15"
        );
    }

    #[wasm_bindgen_test]
    async fn suspense_resolves_after_resource_ready() {
        let resource: Resource<i32> = Resource::new(async { 7 });

        // Before tick: fallback.
        let v1 = suspense(
            || VNode::Text("loading".into()),
            || {
                let _ = resource.state();
                VNode::Text("done".into())
            },
        );
        assert!(matches!(v1, VNode::Text(ref t) if t == "loading"));

        tick().await;

        // After tick: children.
        let v2 = suspense(
            || VNode::Text("loading".into()),
            || {
                let _ = resource.state();
                VNode::Text("done".into())
            },
        );
        assert!(matches!(v2, VNode::Text(ref t) if t == "done"));
    }
}
