//! Fine-grained reactivity (Solid/Leptos-style): [`Signal`], [`Memo`], effects, and batching.
//!
//! A [`Signal`] is a cheap `Copy` handle into a reactive runtime. Reads inside a reactive context
//! (an effect or a memo) subscribe automatically; writes notify subscribers. [`batch`] coalesces
//! notifications into a single flush, and dropping a reactive scope unsubscribes everything it
//! owns (the `no_stale_subscriber_leak` soundness identity).
//!
//! # Design
//!
//! All reactive state lives in one thread-local [`Runtime`] holding a **generational arena** of
//! nodes. `Signal<T>` and `Memo<T>` are `Copy` generational keys into that arena, not the values
//! themselves — cloning a handle is a bitwise copy of a key. Generational keys make the runtime
//! sound against stale handles: once a node is disposed, its slot's generation is bumped, so a
//! leftover key can never alias a freshly-allocated node.
//!
//! * **Dependency tracking** — a reading observer (effect/memo) at the top of the runtime's
//!   observer stack is recorded as a subscriber of every signal/memo it reads. Each time an
//!   observer runs it first clears its previous dependency set, so a conditional read that stops
//!   happening is correctly unsubscribed and subscriber sets never grow unboundedly.
//! * **Ownership & disposal** — every node is owned by the scope (or the running effect/memo) that
//!   created it. Disposing a [`ReactiveScope`], or re-running an effect, disposes the owned nodes
//!   and removes them from every subscriber set they participated in. This is the mechanism behind
//!   the `no_stale_subscriber_leak` identity.
//! * **Values** — each node stores its value behind its own `Rc<RefCell<Box<dyn Any>>>` cell, so a
//!   read or write borrows only that cell (never the whole runtime) while a user closure runs,
//!   avoiding re-entrant runtime-borrow panics.
//! * **Memos** are lazy: a memo performs one eager computation at creation to discover its
//!   dependencies and initial value, then recomputes only when a dependency has changed *and* the
//!   memo is next read.
//! * **Batching** — [`batch`] defers notification flushes to the end of the outermost batch,
//!   coalescing many writes into one re-run of each affected effect.
//!
//! The module is native-testable: nothing here touches the DOM or `web-sys`, so the full reactive
//! graph (signals, computed values, batching, and subscriber cleanup) is exercised by the
//! `#[cfg(test)]` suite below with ordinary `cargo test`.

use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::marker::PhantomData;
use std::rc::Rc;

// A reactive value cell, shared out of the runtime so reads/writes do not hold the runtime borrow
// across user closures.
type ValueCell = Rc<RefCell<Box<dyn Any>>>;
// A reactive computation (an effect body or a memo recompute), stored so it can be cloned out and
// invoked without holding the runtime borrow.
type Computation = Rc<RefCell<Box<dyn FnMut()>>>;

thread_local! {
    static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::default());
}

/// Run `f` with exclusive access to the thread-local reactive runtime.
fn with_rt<R>(f: impl FnOnce(&mut Runtime) -> R) -> R {
    RUNTIME.with(|cell| f(&mut cell.borrow_mut()))
}

// =====================================================================================
// Generational arena
// =====================================================================================

/// A `Copy` generational key identifying a node in the reactive arena.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct NodeId {
    index: usize,
    generation: u32,
}

/// One slot in the arena. `generation` is bumped on removal so a stale [`NodeId`] can never alias a
/// node allocated into the reused slot.
struct Slot {
    generation: u32,
    node: Option<Node>,
}

/// A generational arena of reactive nodes.
#[derive(Default)]
struct Arena {
    slots: Vec<Slot>,
    free: Vec<usize>,
}

impl Arena {
    fn insert(&mut self, node: Node) -> NodeId {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index];
            slot.node = Some(node);
            NodeId {
                index,
                generation: slot.generation,
            }
        } else {
            let index = self.slots.len();
            self.slots.push(Slot {
                generation: 0,
                node: Some(node),
            });
            NodeId {
                index,
                generation: 0,
            }
        }
    }

    fn get(&self, id: NodeId) -> Option<&Node> {
        self.slots
            .get(id.index)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.node.as_ref())
    }

    fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.slots
            .get_mut(id.index)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.node.as_mut())
    }

    fn remove(&mut self, id: NodeId) -> Option<Node> {
        let slot = self.slots.get_mut(id.index)?;
        if slot.generation != id.generation {
            return None;
        }
        let node = slot.node.take();
        if node.is_some() {
            slot.generation = slot.generation.wrapping_add(1);
            self.free.push(id.index);
        }
        node
    }
}

// =====================================================================================
// Reactive nodes
// =====================================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum NodeKind {
    Signal,
    Memo,
    Effect,
    // The ownership-scope node backing `ReactiveScope`. Only constructed through the scope API,
    // which (see `ReactiveScope`) has no production caller yet in this serialized layer pipeline.
    #[allow(dead_code)]
    Scope,
}

/// A node in the reactive graph: a signal, a memo, an effect, or an ownership scope.
struct Node {
    kind: NodeKind,
    /// The value cell, for signals and (once computed) memos.
    value: Option<ValueCell>,
    /// The recompute closure, for effects and memos.
    computation: Option<Computation>,
    /// Observers that read this node's value.
    subscribers: HashSet<NodeId>,
    /// Nodes this node reads (its dependencies); maintained for effects and memos.
    sources: HashSet<NodeId>,
    /// Nodes owned by this node (created while it was the current owner).
    children: Vec<NodeId>,
    /// The scope/computation that owns this node, for disposal.
    owner: Option<NodeId>,
    /// Memo staleness: `true` means the cached value must be recomputed before the next read.
    dirty: bool,
}

impl Node {
    fn blank(kind: NodeKind) -> Self {
        Node {
            kind,
            value: None,
            computation: None,
            subscribers: HashSet::new(),
            sources: HashSet::new(),
            children: Vec::new(),
            owner: None,
            dirty: false,
        }
    }

    fn signal(value: ValueCell) -> Self {
        let mut node = Node::blank(NodeKind::Signal);
        node.value = Some(value);
        node
    }

    fn memo() -> Self {
        let mut node = Node::blank(NodeKind::Memo);
        node.dirty = true;
        node
    }

    fn effect(computation: Computation) -> Self {
        let mut node = Node::blank(NodeKind::Effect);
        node.computation = Some(computation);
        node
    }

    // Constructed only via the `ReactiveScope` API (no production caller yet; see that type).
    #[allow(dead_code)]
    fn scope() -> Self {
        Node::blank(NodeKind::Scope)
    }
}

// =====================================================================================
// Runtime
// =====================================================================================

/// The thread-local reactive runtime: the arena plus the tracking and scheduling state.
#[derive(Default)]
struct Runtime {
    arena: Arena,
    /// Observers currently running, innermost last. The top is the current reactive context.
    observer_stack: Vec<NodeId>,
    /// Owners currently active, innermost last. New nodes attach to the top.
    owner_stack: Vec<NodeId>,
    /// Effects queued to re-run, deduplicated, drained by [`schedule_flush`].
    pending: VecDeque<NodeId>,
    /// Outstanding [`batch`] nesting depth; flushes are deferred while > 0.
    batch_depth: u32,
    /// Guards the flush loop so a write made by a re-running effect does not start a nested flush.
    flushing: bool,
}

impl Runtime {
    /// Insert a node, attaching it to the current owner (if any) for later disposal.
    fn insert_node(&mut self, mut node: Node) -> NodeId {
        let owner = self.owner_stack.last().copied();
        node.owner = owner;
        let id = self.arena.insert(node);
        if let Some(owner) = owner {
            if let Some(parent) = self.arena.get_mut(owner) {
                parent.children.push(id);
            }
        }
        id
    }

    /// Record the current observer (if any) as a subscriber of `id`.
    fn track(&mut self, id: NodeId) {
        if let Some(&observer) = self.observer_stack.last() {
            if observer == id {
                return;
            }
            if let Some(node) = self.arena.get_mut(id) {
                node.subscribers.insert(observer);
            }
            if let Some(obs) = self.arena.get_mut(observer) {
                obs.sources.insert(id);
            }
        }
    }

    /// Clone out the value cell of a signal or computed memo.
    fn value_cell(&self, id: NodeId) -> ValueCell {
        self.arena
            .get(id)
            .and_then(|node| node.value.clone())
            .expect("reactive node has no value cell")
    }

    /// Store a freshly computed value into a node, allocating the cell on first write.
    fn store_value(&mut self, id: NodeId, boxed: Box<dyn Any>) {
        if let Some(node) = self.arena.get_mut(id) {
            match &node.value {
                Some(cell) => *cell.borrow_mut() = boxed,
                None => node.value = Some(Rc::new(RefCell::new(boxed))),
            }
        }
    }

    /// Propagate a change from `id`: mark dependent memos dirty (recursively) and queue dependent
    /// effects for re-running.
    fn mark_dependents(&mut self, id: NodeId) {
        let subscribers: Vec<NodeId> = match self.arena.get(id) {
            Some(node) => node.subscribers.iter().copied().collect(),
            None => return,
        };
        for sub in subscribers {
            let kind = match self.arena.get(sub) {
                Some(node) => node.kind,
                None => continue,
            };
            match kind {
                NodeKind::Memo => {
                    let already_dirty = self.arena.get(sub).map(|n| n.dirty).unwrap_or(true);
                    if !already_dirty {
                        if let Some(node) = self.arena.get_mut(sub) {
                            node.dirty = true;
                        }
                        self.mark_dependents(sub);
                    }
                }
                NodeKind::Effect => {
                    if !self.pending.contains(&sub) {
                        self.pending.push_back(sub);
                    }
                }
                NodeKind::Signal | NodeKind::Scope => {}
            }
        }
    }

    /// Dispose a node and its owned children, detaching it from every subscriber/source set so no
    /// freed node is ever referenced again.
    fn dispose(&mut self, id: NodeId) {
        let children = match self.arena.get_mut(id) {
            Some(node) => std::mem::take(&mut node.children),
            None => return,
        };
        for child in children {
            self.dispose(child);
        }
        let (sources, subscribers) = match self.arena.get_mut(id) {
            Some(node) => (
                std::mem::take(&mut node.sources),
                std::mem::take(&mut node.subscribers),
            ),
            None => return,
        };
        for source in sources {
            if let Some(node) = self.arena.get_mut(source) {
                node.subscribers.remove(&id);
            }
        }
        for subscriber in subscribers {
            if let Some(node) = self.arena.get_mut(subscriber) {
                node.sources.remove(&id);
            }
        }
        self.pending.retain(|pending| *pending != id);
        self.arena.remove(id);
    }
}

// =====================================================================================
// Reactive orchestration (runs user closures outside the runtime borrow)
// =====================================================================================

/// Run a computation (an effect or a memo recompute): clear its previous dependencies and owned
/// children, then execute its closure as the current reactive context so its reads re-subscribe.
fn run_computation(id: NodeId) {
    let started = with_rt(|rt| {
        if rt.arena.get(id).is_none() {
            return false;
        }
        // Dispose nodes created during the previous run (leak-free dynamic graphs).
        let children = rt
            .arena
            .get_mut(id)
            .map(|node| std::mem::take(&mut node.children))
            .unwrap_or_default();
        for child in children {
            rt.dispose(child);
        }
        if rt.arena.get(id).is_none() {
            return false;
        }
        // Drop stale subscriptions; they are rebuilt by the reads this run performs.
        let old_sources = rt
            .arena
            .get_mut(id)
            .map(|node| std::mem::take(&mut node.sources))
            .unwrap_or_default();
        for source in old_sources {
            if let Some(node) = rt.arena.get_mut(source) {
                node.subscribers.remove(&id);
            }
        }
        if let Some(node) = rt.arena.get_mut(id) {
            node.dirty = false;
        }
        rt.observer_stack.push(id);
        rt.owner_stack.push(id);
        true
    });
    if !started {
        return;
    }

    let computation = with_rt(|rt| rt.arena.get(id).and_then(|node| node.computation.clone()));
    if let Some(computation) = computation {
        let mut guard = computation.borrow_mut();
        let run: &mut dyn FnMut() = &mut **guard;
        run();
    }

    with_rt(|rt| {
        rt.observer_stack.pop();
        rt.owner_stack.pop();
    });
}

/// Recompute a memo if its cached value is stale.
fn ensure_current(id: NodeId) {
    let dirty = with_rt(|rt| rt.arena.get(id).map(|node| node.dirty).unwrap_or(false));
    if dirty {
        run_computation(id);
    }
}

/// Record a change at `id` and flush its dependent effects (unless a batch defers the flush).
fn notify(id: NodeId) {
    with_rt(|rt| rt.mark_dependents(id));
    schedule_flush();
}

/// Drain the pending-effect queue, re-running each queued effect exactly once per change. Re-entrant
/// calls (writes made by a re-running effect) are absorbed by the queue rather than nesting flushes.
fn schedule_flush() {
    let acquired = with_rt(|rt| {
        if rt.batch_depth == 0 && !rt.flushing {
            rt.flushing = true;
            true
        } else {
            false
        }
    });
    if !acquired {
        return;
    }
    loop {
        let next = with_rt(|rt| rt.pending.pop_front());
        match next {
            Some(id) => {
                if with_rt(|rt| rt.arena.get(id).is_some()) {
                    run_computation(id);
                }
            }
            None => break,
        }
    }
    with_rt(|rt| rt.flushing = false);
}

/// Decrement the batch depth and flush when the outermost batch ends. Runs on unwind too, so a
/// panicking batch body cannot leave the runtime wedged at a non-zero depth.
struct BatchGuard;

impl Drop for BatchGuard {
    fn drop(&mut self) {
        let ended = with_rt(|rt| {
            rt.batch_depth = rt.batch_depth.saturating_sub(1);
            rt.batch_depth == 0
        });
        if ended {
            schedule_flush();
        }
    }
}

// =====================================================================================
// Reactive scopes (ownership for disposal) — crate-internal
// =====================================================================================

/// An ownership scope (reactive root) for reactive nodes. Effects and memos created while this
/// scope is active are owned by it; dropping the scope disposes them and unsubscribes them from
/// every signal. This is the handle that realizes the brief's "dropping an effect/scope must
/// unsubscribe it" soundness requirement and backs the `no_stale_subscriber_leak` identity.
///
/// It is deliberately crate-internal: the frozen public API (`create_effect`) creates effects in
/// the currently-active scope and never hands back a disposer, so a disposable root is expressed
/// here rather than on the frozen surface. In this serialized layer pipeline it currently has no
/// production caller (a later layer, or an additive public promotion, may consume it); today it is
/// exercised by the soundness tests below, hence the `dead_code` allowance.
#[allow(dead_code)]
pub(crate) struct ReactiveScope {
    id: NodeId,
}

#[allow(dead_code)]
impl ReactiveScope {
    /// Create a new, empty scope rooted under the currently-active scope (if any).
    pub(crate) fn new() -> Self {
        let id = with_rt(|rt| rt.insert_node(Node::scope()));
        ReactiveScope { id }
    }

    /// Run `f` with this scope as the owner of any reactive nodes it creates.
    pub(crate) fn run<R>(&self, f: impl FnOnce() -> R) -> R {
        with_rt(|rt| rt.owner_stack.push(self.id));
        let result = f();
        with_rt(|rt| {
            rt.owner_stack.pop();
        });
        result
    }
}

impl Default for ReactiveScope {
    fn default() -> Self {
        ReactiveScope::new()
    }
}

impl Drop for ReactiveScope {
    fn drop(&mut self) {
        with_rt(|rt| rt.dispose(self.id));
    }
}

// =====================================================================================
// Public API — signals
// =====================================================================================

/// A readable + writable reactive value handle. Cheap to `Clone`/`Copy` (it is a key into the
/// reactive runtime, not the value itself).
pub struct Signal<T: 'static> {
    id: NodeId,
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
        self.with(Clone::clone)
    }

    /// Read by reference without cloning. Subscribes the current reactive observer (if any).
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let cell = with_rt(|rt| {
            rt.track(self.id);
            rt.value_cell(self.id)
        });
        let guard = cell.borrow();
        let value = guard
            .downcast_ref::<T>()
            .expect("signal read with mismatched type");
        f(value)
    }

    /// Replace the value and notify subscribers.
    pub fn set(&self, value: T) {
        let cell = with_rt(|rt| rt.value_cell(self.id));
        *cell.borrow_mut() = Box::new(value);
        notify(self.id);
    }

    /// Mutate the value in place and notify subscribers.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let cell = with_rt(|rt| rt.value_cell(self.id));
        {
            let mut guard = cell.borrow_mut();
            let value = guard
                .downcast_mut::<T>()
                .expect("signal update with mismatched type");
            f(value);
        }
        notify(self.id);
    }
}

/// Create a reactive signal with an initial value.
///
/// ```
/// use ferric::prelude::*;
///
/// let count = create_signal(0);
/// assert_eq!(count.get(), 0);
/// count.set(5);
/// assert_eq!(count.get(), 5);
/// count.update(|n| *n += 1);
/// assert_eq!(count.get(), 6);
/// ```
pub fn create_signal<T: 'static>(value: T) -> Signal<T> {
    let cell: ValueCell = Rc::new(RefCell::new(Box::new(value) as Box<dyn Any>));
    let id = with_rt(|rt| rt.insert_node(Node::signal(cell)));
    Signal {
        id,
        _marker: PhantomData,
    }
}

// =====================================================================================
// Public API — memos
// =====================================================================================

/// A derived/computed value. Recomputes lazily when its reactive dependencies change; memoizes the
/// result between changes so repeated reads do not re-run the computation.
pub struct Memo<T: 'static> {
    id: NodeId,
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
        self.with(Clone::clone)
    }

    /// Read the memoized value by reference. Subscribes the current reactive observer (if any).
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        ensure_current(self.id);
        let cell = with_rt(|rt| {
            rt.track(self.id);
            rt.value_cell(self.id)
        });
        let guard = cell.borrow();
        let value = guard
            .downcast_ref::<T>()
            .expect("memo read with mismatched type");
        f(value)
    }
}

/// Create a memoized derived value from a closure that reads other signals/memos.
///
/// The closure runs once immediately to establish the initial value and its dependency set, then
/// re-runs only when a dependency has changed and the memo is next read.
///
/// ```
/// use ferric::prelude::*;
///
/// let count = create_signal(2);
/// let doubled = create_memo(move || count.get() * 2);
/// assert_eq!(doubled.get(), 4);
/// count.set(10);
/// assert_eq!(doubled.get(), 20);
/// ```
pub fn create_memo<T: 'static>(f: impl Fn() -> T + 'static) -> Memo<T> {
    let id = with_rt(|rt| rt.insert_node(Node::memo()));
    let computation: Computation = Rc::new(RefCell::new(Box::new(move || {
        let value = f();
        with_rt(|rt| rt.store_value(id, Box::new(value) as Box<dyn Any>));
    })));
    with_rt(|rt| {
        if let Some(node) = rt.arena.get_mut(id) {
            node.computation = Some(computation);
        }
    });
    // Eager first computation: discovers dependencies and populates the initial value.
    run_computation(id);
    Memo {
        id,
        _marker: PhantomData,
    }
}

// =====================================================================================
// Public API — effects and batching
// =====================================================================================

/// Register a reactive side effect. Runs once immediately (tracking its signal reads), then
/// re-runs whenever any tracked dependency changes. The DOM runtime builds re-render on this.
///
/// The effect is owned by the currently-active reactive scope; when that scope is disposed the
/// effect is unsubscribed from every signal it read.
pub fn create_effect(f: impl FnMut() + 'static) {
    let computation: Computation = Rc::new(RefCell::new(Box::new(f)));
    let id = with_rt(|rt| rt.insert_node(Node::effect(computation)));
    run_computation(id);
}

/// Run `f`, coalescing all signal notifications into a single flush at the end (batched update).
///
/// Writes performed inside `f` update their signals immediately, but dependent effects do not
/// re-run until the outermost `batch` returns, so each affected effect re-runs at most once.
///
/// ```
/// use ferric::prelude::*;
///
/// let a = create_signal(1);
/// let b = create_signal(1);
/// let sum = create_memo(move || a.get() + b.get());
/// batch(|| {
///     a.set(10);
///     b.set(20);
/// });
/// assert_eq!(sum.get(), 30);
/// ```
pub fn batch<R>(f: impl FnOnce() -> R) -> R {
    with_rt(|rt| rt.batch_depth += 1);
    let _guard = BatchGuard;
    f()
}

// =====================================================================================
// Tests — native, no browser required
// =====================================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// Number of observers currently subscribed to a signal (test-only introspection).
    fn subscriber_count<T: 'static>(signal: Signal<T>) -> usize {
        with_rt(|rt| {
            rt.arena
                .get(signal.id)
                .map(|node| node.subscribers.len())
                .unwrap_or(0)
        })
    }

    /// Number of live nodes in the arena (test-only introspection for leak checks).
    fn live_node_count() -> usize {
        with_rt(|rt| rt.arena.slots.iter().filter(|s| s.node.is_some()).count())
    }

    #[test]
    fn signal_get_set_update() {
        let count = create_signal(0);
        assert_eq!(count.get(), 0);
        count.set(5);
        assert_eq!(count.get(), 5);
        count.update(|n| *n += 3);
        assert_eq!(count.get(), 8);
    }

    #[test]
    fn signal_with_reads_by_reference_without_clone() {
        // A non-Clone payload is still readable through `with`.
        let names = create_signal(vec!["a".to_string(), "b".to_string()]);
        let len = names.with(|v| v.len());
        assert_eq!(len, 2);
        names.update(|v| v.push("c".to_string()));
        assert_eq!(names.with(|v| v.len()), 3);
    }

    #[test]
    fn effect_runs_once_immediately_then_on_change() {
        let source = create_signal(0);
        let runs = Rc::new(Cell::new(0));
        let seen = Rc::new(Cell::new(-1));
        {
            let runs = Rc::clone(&runs);
            let seen = Rc::clone(&seen);
            create_effect(move || {
                runs.set(runs.get() + 1);
                seen.set(source.get());
            });
        }
        assert_eq!(runs.get(), 1, "effect runs once immediately");
        assert_eq!(seen.get(), 0);

        source.set(42);
        assert_eq!(runs.get(), 2, "effect re-runs on dependency change");
        assert_eq!(seen.get(), 42);

        // A write that does not change the observed graph still re-runs the effect once.
        source.set(42);
        assert_eq!(runs.get(), 3);
    }

    #[test]
    fn effect_subscriber_set_does_not_grow_across_reruns() {
        let source = create_signal(0);
        create_effect(move || {
            let _ = source.get();
        });
        assert_eq!(subscriber_count(source), 1);
        for i in 1..=10 {
            source.set(i);
        }
        // Re-diffing dependencies each run keeps the subscriber set at exactly one.
        assert_eq!(subscriber_count(source), 1);
    }

    #[test]
    fn effect_tracks_only_currently_read_dependencies() {
        let toggle = create_signal(true);
        let a = create_signal(1);
        let b = create_signal(2);
        let seen = Rc::new(Cell::new(0));
        {
            let seen = Rc::clone(&seen);
            create_effect(move || {
                let value = if toggle.get() { a.get() } else { b.get() };
                seen.set(value);
            });
        }
        assert_eq!(seen.get(), 1);
        assert_eq!(subscriber_count(a), 1);
        assert_eq!(subscriber_count(b), 0);

        // Switch the branch: the effect should unsubscribe from `a` and subscribe to `b`.
        toggle.set(false);
        assert_eq!(seen.get(), 2);
        assert_eq!(subscriber_count(a), 0, "stale dependency dropped");
        assert_eq!(subscriber_count(b), 1, "new dependency tracked");

        // Writing the now-untracked signal must not re-run the effect.
        a.set(100);
        assert_eq!(seen.get(), 2);
    }

    #[test]
    fn memo_computes_and_memoizes() {
        let count = create_signal(2);
        let compute_runs = Rc::new(Cell::new(0));
        let doubled = {
            let compute_runs = Rc::clone(&compute_runs);
            create_memo(move || {
                compute_runs.set(compute_runs.get() + 1);
                count.get() * 2
            })
        };
        // One eager computation at creation.
        assert_eq!(compute_runs.get(), 1);
        assert_eq!(doubled.get(), 4);
        // Repeated reads do not recompute.
        assert_eq!(doubled.get(), 4);
        assert_eq!(compute_runs.get(), 1, "reads are memoized");

        count.set(10);
        // Lazy: still not recomputed until read.
        assert_eq!(compute_runs.get(), 1, "recompute deferred until read");
        assert_eq!(doubled.get(), 20);
        assert_eq!(compute_runs.get(), 2, "recomputed exactly once on read");
    }

    #[test]
    fn memo_chains_propagate() {
        let count = create_signal(1);
        let doubled = create_memo(move || count.get() * 2);
        let plus_one = create_memo(move || doubled.get() + 1);
        assert_eq!(plus_one.get(), 3);
        count.set(5);
        assert_eq!(plus_one.get(), 11);
    }

    #[test]
    fn effect_reads_memo_and_reruns_on_change() {
        let count = create_signal(1);
        let doubled = create_memo(move || count.get() * 2);
        let seen = Rc::new(Cell::new(0));
        {
            let seen = Rc::clone(&seen);
            create_effect(move || seen.set(doubled.get()));
        }
        assert_eq!(seen.get(), 2);
        count.set(7);
        assert_eq!(
            seen.get(),
            14,
            "effect re-runs when an upstream memo changes"
        );
    }

    #[test]
    fn batch_coalesces_notifications() {
        let a = create_signal(1);
        let b = create_signal(1);
        let runs = Rc::new(Cell::new(0));
        let total = Rc::new(Cell::new(0));
        {
            let runs = Rc::clone(&runs);
            let total = Rc::clone(&total);
            create_effect(move || {
                runs.set(runs.get() + 1);
                total.set(a.get() + b.get());
            });
        }
        assert_eq!(runs.get(), 1);

        batch(|| {
            a.set(10);
            b.set(20);
        });
        // Both writes coalesce into a single re-run.
        assert_eq!(runs.get(), 2, "batch coalesces into one flush");
        assert_eq!(total.get(), 30);
    }

    #[test]
    fn nested_batches_flush_once_at_the_outermost_boundary() {
        let a = create_signal(0);
        let runs = Rc::new(Cell::new(0));
        {
            let runs = Rc::clone(&runs);
            create_effect(move || {
                runs.set(runs.get() + 1);
                let _ = a.get();
            });
        }
        assert_eq!(runs.get(), 1);
        batch(|| {
            a.set(1);
            batch(|| {
                a.set(2);
                a.set(3);
            });
            a.set(4);
        });
        assert_eq!(runs.get(), 2, "only the outermost batch triggers the flush");
        assert_eq!(a.get(), 4);
    }

    #[test]
    fn no_stale_subscriber_leak() {
        // This is the `no_stale_subscriber_leak` soundness identity (contracts/_MANIFEST.yaml).
        let source = create_signal(0);
        assert_eq!(subscriber_count(source), 0);

        let nodes_before = live_node_count();
        {
            let scope = ReactiveScope::new();
            scope.run(|| {
                create_effect(move || {
                    let _ = source.get();
                });
            });
            // The effect subscribed itself to the signal.
            assert_eq!(subscriber_count(source), 1);
        } // scope dropped here -> its effect is disposed

        // The only subscriber is gone: its subscription must have been removed.
        assert_eq!(
            subscriber_count(source),
            0,
            "dropping the owning scope unsubscribed the effect"
        );

        // Setting the signal after its only subscriber is gone must not touch freed state...
        source.set(1);
        source.set(2);
        // ...and must not grow the subscriber set.
        assert_eq!(subscriber_count(source), 0);
        assert_eq!(source.get(), 2);

        // The disposed effect's node was reclaimed (no unbounded node growth).
        assert_eq!(
            live_node_count(),
            nodes_before,
            "the disposed effect and scope nodes were reclaimed"
        );
    }

    #[test]
    fn disposing_scope_also_disposes_owned_memo() {
        let source = create_signal(1);
        assert_eq!(subscriber_count(source), 0);
        {
            let scope = ReactiveScope::new();
            scope.run(|| {
                let derived = create_memo(move || source.get() + 1);
                assert_eq!(derived.get(), 2);
            });
            assert_eq!(subscriber_count(source), 1, "memo subscribed to its source");
        }
        assert_eq!(
            subscriber_count(source),
            0,
            "disposing the scope unsubscribed the memo"
        );
        source.set(100); // must not panic or resurrect the disposed memo
        assert_eq!(subscriber_count(source), 0);
    }

    #[test]
    fn generational_keys_are_aba_safe() {
        // Allocate a signal inside a scope, drop it, then allocate another: the new node reuses the
        // freed slot but with a bumped generation, so the first handle can never alias it.
        let first_id = {
            let scope = ReactiveScope::new();
            scope.run(|| create_signal(1i32).id)
        };
        let second = create_signal(2i32);
        // The reused slot index may match, but the generation must differ.
        if first_id.index == second.id.index {
            assert_ne!(
                first_id.generation, second.id.generation,
                "generation bumped on slot reuse"
            );
        }
        // The stale handle resolves to no live node.
        assert!(with_rt(|rt| rt.arena.get(first_id).is_none()));
    }
}
