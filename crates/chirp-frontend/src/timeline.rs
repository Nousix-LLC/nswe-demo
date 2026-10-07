//! `/` — the timeline view.
//!
//! Realizes the frozen [`timeline_view`] signature (`contracts/app_api.rs`): load a
//! [`Page<Chirp>`] through `ctx.client().timeline(..)`, render a keyed, reactive chirp list, and
//! handle the loading, error, empty, and paginated-content states with cursor-based "load more".
//!
//! # Reactive-state model (why this module holds state the way it does)
//!
//! `ferric` is a *coarse* re-render framework: [`mount`](ferric::mount) wraps the whole render in
//! one reactive effect, and a route view such as this one is **re-invoked from scratch on every
//! re-render**. Critically, a signal created *inside* a running effect is owned by that effect and
//! is **disposed when the effect re-runs** (see the note in `ferric-router`'s `router.rs`: "a
//! `ferric` node created while an effect is the active owner would be disposed when that effect
//! re-runs"). So the intuitive "create local `loading`/`data`/`error` signals inside the view"
//! shape is unsound here — the signals would be torn down and reset on the first re-render, which
//! an async `fetch -> set -> re-render` cycle triggers immediately.
//!
//! The sound equivalent, mirroring how `ferric-router` keeps its current-path state alive, is:
//!
//! * the view's data lives in a module [`thread_local!`] [`TimelineModel`] (plain data — never
//!   disposed, survives every re-render); and
//! * reactivity is driven by a tiny re-render **trigger** [`Signal`] that each render *recreates*
//!   and reads (so the mount effect subscribes to the current one), and that async completions and
//!   event handlers *bump* to request a re-render. Because the trigger is recreated every render,
//!   the handle stored in [`RERENDER`] is always the live one while this route is mounted; a
//!   completion that fires after the user has navigated away is dropped by the [`ROUTE_PATH`]
//!   guard in [`bump`] rather than touching a disposed handle (which would panic).
//!
//! This is a single-instance view (one timeline route), so module-global `thread_local` state is
//! the correct scope — exactly as `ferric-router` scopes its single current-path signal.
//!
//! Nothing here panics: the runtime path surfaces every `ApiClient` failure as an
//! [`AppError`](crate::error::AppError) banner, and the trigger is only ever set while it is live.
//! The live `fetch` path runs only in the browser (via [`spawn_local`](wasm_bindgen_futures::spawn_local));
//! the pure query-building, state-reduction, and view-tree helpers are exercised by the native
//! `#[cfg(test)]` suite below.

use std::cell::{Cell, RefCell};

use chirp_types::prelude::{Chirp, Cursor, Page, TimelineQuery};
use ferric::prelude::*;
use ferric::vdom::text;
use ferric_router::prelude::current_path;

use crate::app::AppContext;
use crate::error::AppError;

/// The path this view is mounted at (see `app.rs` route table). Used by [`bump`] to avoid
/// re-rendering — and thus touching a now-disposed trigger handle — once the user has navigated
/// away while a request was still in flight.
const ROUTE_PATH: &str = "/";

/// The page size requested from the timeline endpoint. The server clamps to its own maximum;
/// pagination continues via the returned [`Page::next_cursor`] regardless of this value.
const PAGE_LIMIT: u32 = 20;

thread_local! {
    /// The timeline's data, independent of the render lifecycle. Plain data (not a `ferric` node),
    /// so it is never disposed and survives every coarse re-render.
    static MODEL: RefCell<TimelineModel> = const { RefCell::new(TimelineModel::new()) };
    /// The current re-render trigger signal. Recreated and re-read on every render so the mount
    /// effect subscribes to the live handle; bumped by async completions / handlers to re-render.
    static RERENDER: RefCell<Option<Signal<u32>>> = const { RefCell::new(None) };
    /// Whether the one-shot initial load has been kicked off yet.
    static STARTED: Cell<bool> = const { Cell::new(false) };
}

/// Which load a request represents: the first page, or a `next_cursor` continuation.
#[derive(Clone, Copy)]
enum LoadKind {
    /// The first page (fresh timeline, or a retry of it).
    Initial,
    /// A `next_cursor` continuation appended to the existing list.
    More,
}

/// The timeline's reactive state: the accumulated chirps plus the loading / error / pagination
/// flags the view branches on. Mutated only through the reducer methods below so the transitions
/// stay in one place and are unit-testable without a DOM.
struct TimelineModel {
    /// Whether the initial page has loaded at least once (distinguishes "first load" from "loaded,
    /// possibly empty").
    loaded: bool,
    /// The chirps accumulated so far, newest first (the endpoint's order, preserved across pages).
    chirps: Vec<Chirp>,
    /// Continuation cursor for the next page, or `None` on the last page.
    next_cursor: Option<Cursor>,
    /// Whether a "load more" continuation is currently in flight.
    loading_more: bool,
    /// The most recent error, if the last request failed. Combined with `loaded` this selects the
    /// full-screen initial-error state vs. the inline load-more-error state.
    error: Option<AppError>,
}

impl TimelineModel {
    /// The empty, pre-load state: nothing loaded, nothing in flight, no error.
    const fn new() -> Self {
        TimelineModel {
            loaded: false,
            chirps: Vec::new(),
            next_cursor: None,
            loading_more: false,
            error: None,
        }
    }

    /// Reset to the initial-loading state (used by the retry action on an initial-load failure).
    fn begin_initial(&mut self) {
        self.loaded = false;
        self.chirps.clear();
        self.next_cursor = None;
        self.loading_more = false;
        self.error = None;
    }

    /// Mark a "load more" continuation as in flight (clears any prior inline error).
    fn begin_more(&mut self) {
        self.loading_more = true;
        self.error = None;
    }

    /// Apply a successful first page: replace the list, adopt its cursor, clear flags/error.
    fn apply_initial(&mut self, page: Page<Chirp>) {
        self.chirps = page.items;
        self.next_cursor = page.next_cursor;
        self.loaded = true;
        self.loading_more = false;
        self.error = None;
    }

    /// Apply a successful continuation page: append its items, adopt its cursor, clear flags/error.
    fn apply_more(&mut self, page: Page<Chirp>) {
        self.chirps.extend(page.items);
        self.next_cursor = page.next_cursor;
        self.loading_more = false;
        self.error = None;
    }

    /// Record a failed request. `loaded` is left unchanged, so a first-load failure shows the
    /// full-screen error while a continuation failure shows an inline error beneath the list.
    fn apply_error(&mut self, err: AppError) {
        self.loading_more = false;
        self.error = Some(err);
    }
}

/// `/` — timeline. Loads a [`Page<Chirp>`] on first display, renders a keyed reactive list with
/// loading / error / empty states, and offers cursor-based "load more".
///
/// See the [module docs](self) for why the view's state lives in a `thread_local` model plus a
/// recreated-each-render trigger signal rather than in signals created inside this function.
#[must_use]
pub fn timeline_view(ctx: &AppContext) -> VNode {
    // Recreate this render's trigger and subscribe the mount effect to it. The previous render's
    // trigger was disposed when the effect re-ran; `RERENDER` now holds the live one.
    let tick = create_signal(0u32);
    RERENDER.with(|cell| *cell.borrow_mut() = Some(tick));
    let _ = tick.get();

    // Kick off the first load exactly once, the first time the timeline is shown.
    STARTED.with(|started| {
        if !started.get() {
            started.set(true);
            request_load(ctx.clone(), LoadKind::Initial);
        }
    });

    // "Load more" / inline-retry fetches the next page; "retry" reloads from scratch. Each handler
    // owns a cheap `AppContext` clone so it is `'static`.
    let more_ctx = ctx.clone();
    let on_load_more = move || trigger_more(more_ctx.clone());
    let retry_ctx = ctx.clone();
    let on_retry = move || reload(retry_ctx.clone());

    MODEL.with(|model| {
        let model = model.borrow();
        view_tree(&model, on_load_more, on_retry)
    })
}

/// Begin a "load more" continuation, unless one is already in flight or there is no next page.
fn trigger_more(ctx: AppContext) {
    let proceed = MODEL.with(|model| {
        let mut model = model.borrow_mut();
        if model.loading_more || model.next_cursor.is_none() {
            return false;
        }
        model.begin_more();
        true
    });
    if proceed {
        bump();
        request_load(ctx, LoadKind::More);
    }
}

/// Reset to the loading state and re-request the first page (the initial-error retry action).
fn reload(ctx: AppContext) {
    MODEL.with(|model| model.borrow_mut().begin_initial());
    bump();
    request_load(ctx, LoadKind::Initial);
}

/// Spawn the async request for `kind`, writing the outcome back into [`MODEL`] and requesting a
/// re-render on completion. The query is built from the current model (the continuation cursor for
/// [`LoadKind::More`]). The future runs on the browser microtask queue; off-wasm it is never polled
/// (native tests drive the pure helpers directly).
fn request_load(ctx: AppContext, kind: LoadKind) {
    let query = MODEL.with(|model| {
        let model = model.borrow();
        match kind {
            LoadKind::Initial => timeline_query(Some(PAGE_LIMIT), None),
            LoadKind::More => timeline_query(Some(PAGE_LIMIT), model.next_cursor.clone()),
        }
    });

    wasm_bindgen_futures::spawn_local(async move {
        let result = ctx.client().timeline(&query).await;
        apply_result(kind, result);
    });
}

/// Fold a completed request's outcome into [`MODEL`] and request a re-render.
fn apply_result(kind: LoadKind, result: Result<Page<Chirp>, AppError>) {
    MODEL.with(|model| {
        let mut model = model.borrow_mut();
        match (kind, result) {
            (LoadKind::Initial, Ok(page)) => model.apply_initial(page),
            (LoadKind::More, Ok(page)) => model.apply_more(page),
            (_, Err(err)) => model.apply_error(err),
        }
    });
    bump();
}

/// Request a re-render by bumping the current trigger signal.
///
/// Guarded by [`ROUTE_PATH`]: if the user has navigated away, the most recent render was another
/// route, so the trigger handle in [`RERENDER`] has been disposed. We skip the bump (the model was
/// still updated, so returning to the timeline renders the latest state) rather than set a disposed
/// signal, which would panic.
fn bump() {
    if current_path().as_str() != ROUTE_PATH {
        return;
    }
    let tick = RERENDER.with(|cell| *cell.borrow());
    if let Some(tick) = tick {
        tick.update(|n| *n = n.wrapping_add(1));
    }
}

/// Build the timeline query from an optional page size and continuation cursor. Absent fields are
/// omitted from the serialized query string by `TimelineQuery`'s serde contract.
fn timeline_query(limit: Option<u32>, cursor: Option<Cursor>) -> TimelineQuery {
    TimelineQuery { limit, cursor }
}

/// Render the whole timeline view tree for `model`. Pure: given the model and the two action
/// handlers it produces the same tree, so it is unit-testable without a DOM.
fn view_tree<FMore, FRetry>(model: &TimelineModel, on_load_more: FMore, on_retry: FRetry) -> VNode
where
    FMore: Fn() + 'static,
    FRetry: Fn() + 'static,
{
    let body: VNode = if !model.loaded {
        match &model.error {
            None => loading_view(),
            Some(err) => VElement::new("div")
                .attr("class", "chirp-timeline-error")
                .child(error_banner(err))
                .child(button("Retry", on_retry))
                .into(),
        }
    } else if model.chirps.is_empty() {
        empty_view()
    } else {
        VElement::new("div")
            .attr("class", "chirp-timeline-loaded")
            .child(chirp_list(&model.chirps))
            .child(footer(model, on_load_more))
            .into()
    };

    VElement::new("section")
        .attr("class", "chirp-timeline")
        .child(VElement::new("h1").child(text("Timeline")))
        .child(body)
        .into()
}

/// The footer beneath a non-empty list: an inline error + retry, a "loading more" indicator, a
/// "load more" button, or an end-of-timeline note — whichever the model selects.
fn footer<FMore>(model: &TimelineModel, on_load_more: FMore) -> VNode
where
    FMore: Fn() + 'static,
{
    if let Some(err) = &model.error {
        VElement::new("div")
            .attr("class", "chirp-load-more-error")
            .child(error_banner(err))
            .child(button("Try again", on_load_more))
            .into()
    } else if model.loading_more {
        VElement::new("p")
            .attr("class", "chirp-loading-more")
            .child(text("Loading more…"))
            .into()
    } else if model.next_cursor.is_some() {
        button("Load more", on_load_more)
    } else {
        VElement::new("p")
            .attr("class", "chirp-end")
            .child(text("You're all caught up."))
            .into()
    }
}

/// The full-screen loading state shown before the first page arrives.
fn loading_view() -> VNode {
    VElement::new("p")
        .attr("class", "chirp-loading")
        .child(text("Loading timeline…"))
        .into()
}

/// The empty state shown when the first page loaded but contained no chirps.
fn empty_view() -> VNode {
    VElement::new("p")
        .attr("class", "chirp-empty")
        .child(text("No chirps yet."))
        .into()
}

/// A user-facing error banner carrying the [`AppError`]'s short message.
fn error_banner(err: &AppError) -> VNode {
    VElement::new("p")
        .attr("class", "chirp-error")
        .attr("role", "alert")
        .child(text(err.user_message()))
        .into()
}

/// The keyed chirp list. Every child is a keyed `<li>` (keyed by [`ChirpId`](chirp_types::prelude::ChirpId)),
/// so `ferric`'s diff reconciles the list by identity across re-renders and pagination.
fn chirp_list(chirps: &[Chirp]) -> VNode {
    VElement::new("ul")
        .attr("class", "chirp-list")
        .children(chirps.iter().map(chirp_item))
        .into()
}

/// One chirp row: author, body text, like count, and timestamp. Keyed by the chirp's id.
fn chirp_item(chirp: &Chirp) -> VNode {
    VElement::new("li")
        .key(chirp.id.get().to_string())
        .attr("class", "chirp")
        .child(
            VElement::new("div")
                .attr("class", "chirp-author")
                .child(text(chirp.author_id.to_string())),
        )
        .child(
            VElement::new("p")
                .attr("class", "chirp-text")
                .child(text(chirp.text.as_str())),
        )
        .child(
            VElement::new("div")
                .attr("class", "chirp-meta")
                .child(
                    VElement::new("span")
                        .attr("class", "chirp-likes")
                        .child(text(format!("{} likes", chirp.like_count))),
                )
                .child(
                    VElement::new("span")
                        .attr("class", "chirp-time")
                        .child(text(format!("{} ms", chirp.created_at.as_millis()))),
                ),
        )
        .into()
}

/// A `<button type="button">` carrying a click handler.
fn button(label: &str, on_click: impl Fn() + 'static) -> VNode {
    VElement::new("button")
        .attr("type", "button")
        .on(EventKind::Click, on_click)
        .child(text(label))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chirp_types::prelude::{ChirpId, ChirpText, Timestamp, UserId, ValidationError};

    fn sample_chirp(id: u64) -> Result<Chirp, ValidationError> {
        Ok(Chirp {
            id: ChirpId::new(id),
            author_id: UserId::new(1),
            text: ChirpText::parse(format!("chirp {id}"))?,
            created_at: Timestamp::from_millis(1_700_000_000_000 + id as i64),
            like_count: id,
            reply_to: None,
        })
    }

    fn page(ids: &[u64], next: Option<&str>) -> Result<Page<Chirp>, ValidationError> {
        let items = ids
            .iter()
            .copied()
            .map(sample_chirp)
            .collect::<Result<_, _>>()?;
        Ok(Page::new(items, next.map(|c| Cursor(c.to_string()))))
    }

    // --- pure helpers: query building -------------------------------------------------------

    #[test]
    fn query_carries_limit_and_cursor() {
        let q = timeline_query(Some(PAGE_LIMIT), Some(Cursor("next".to_string())));
        assert_eq!(q.limit, Some(PAGE_LIMIT));
        assert_eq!(q.cursor, Some(Cursor("next".to_string())));

        let first = timeline_query(Some(PAGE_LIMIT), None);
        assert_eq!(first.cursor, None);
    }

    // --- state reduction --------------------------------------------------------------------

    #[test]
    fn apply_initial_marks_loaded_and_adopts_cursor() -> Result<(), ValidationError> {
        let mut model = TimelineModel::new();
        assert!(!model.loaded);
        model.apply_initial(page(&[3, 2, 1], Some("c3"))?);
        assert!(model.loaded);
        assert_eq!(model.chirps.len(), 3);
        assert_eq!(model.next_cursor, Some(Cursor("c3".to_string())));
        assert!(model.error.is_none());
        Ok(())
    }

    #[test]
    fn apply_more_appends_and_updates_cursor() -> Result<(), ValidationError> {
        let mut model = TimelineModel::new();
        model.apply_initial(page(&[3, 2], Some("c2"))?);
        model.begin_more();
        assert!(model.loading_more);
        model.apply_more(page(&[1], None)?);
        assert_eq!(model.chirps.len(), 3, "continuation appended, not replaced");
        assert_eq!(model.chirps[2].id, ChirpId::new(1));
        assert_eq!(model.next_cursor, None, "reached the last page");
        assert!(!model.loading_more);
        Ok(())
    }

    #[test]
    fn apply_error_on_initial_keeps_unloaded() {
        let mut model = TimelineModel::new();
        model.apply_error(AppError::Network("offline".to_string()));
        assert!(
            !model.loaded,
            "initial failure stays in the first-load state"
        );
        assert!(model.error.is_some());
        assert!(!model.loading_more);
    }

    #[test]
    fn apply_error_after_load_is_inline() -> Result<(), ValidationError> {
        let mut model = TimelineModel::new();
        model.apply_initial(page(&[2, 1], Some("c2"))?);
        model.begin_more();
        model.apply_error(AppError::Network("dropped".to_string()));
        assert!(
            model.loaded,
            "list stays visible under an inline continuation error"
        );
        assert!(model.error.is_some());
        assert!(!model.loading_more);
        Ok(())
    }

    #[test]
    fn begin_initial_resets_everything() -> Result<(), ValidationError> {
        let mut model = TimelineModel::new();
        model.apply_initial(page(&[2, 1], Some("c2"))?);
        model.apply_error(AppError::Transport("x".to_string()));
        model.begin_initial();
        assert!(!model.loaded);
        assert!(model.chirps.is_empty());
        assert_eq!(model.next_cursor, None);
        assert!(model.error.is_none());
        Ok(())
    }

    // --- view tree (pure, DOM-free) ---------------------------------------------------------

    fn find<'a>(node: &'a VNode, class: &str) -> Option<&'a VElement> {
        match node {
            VNode::Element(el) => {
                if el.attrs.iter().any(|(k, v)| k == "class" && v == class) {
                    return Some(el);
                }
                el.children.iter().find_map(|c| find(c, class))
            }
            VNode::Text(_) => None,
        }
    }

    fn text_content(node: &VNode) -> String {
        match node {
            VNode::Text(s) => s.clone(),
            VNode::Element(el) => el.children.iter().map(text_content).collect(),
        }
    }

    fn noop() -> impl Fn() + 'static {
        || {}
    }

    #[test]
    fn loading_model_renders_loading_state() {
        let tree = view_tree(&TimelineModel::new(), noop(), noop());
        assert!(find(&tree, "chirp-loading").is_some());
        assert!(find(&tree, "chirp-list").is_none());
    }

    #[test]
    fn loaded_model_renders_keyed_list_and_load_more() -> Result<(), ValidationError> {
        let mut model = TimelineModel::new();
        model.apply_initial(page(&[3, 2, 1], Some("c3"))?);
        let tree = view_tree(&model, noop(), noop());

        // Keyed children: one <li> per chirp, keyed by id, in order.
        let list = find(&tree, "chirp-list");
        assert!(list.is_some(), "expected a rendered chirp-list");
        if let Some(list) = list {
            let keys: Vec<Option<String>> = list.children.iter().map(element_key).collect();
            assert_eq!(
                keys,
                vec![
                    Some("3".to_string()),
                    Some("2".to_string()),
                    Some("1".to_string())
                ]
            );
        }
        // Last page not yet reached -> a "load more" affordance is present.
        assert!(text_content(&tree).contains("Load more"));
        Ok(())
    }

    #[test]
    fn empty_model_renders_empty_state() -> Result<(), ValidationError> {
        let mut model = TimelineModel::new();
        model.apply_initial(page(&[], None)?);
        let tree = view_tree(&model, noop(), noop());
        assert!(find(&tree, "chirp-empty").is_some());
        assert!(find(&tree, "chirp-list").is_none());
        Ok(())
    }

    #[test]
    fn initial_error_renders_message_and_retry() {
        let mut model = TimelineModel::new();
        model.apply_error(AppError::Api(chirp_types::prelude::ApiError::new(
            chirp_types::prelude::ErrorCode::Internal,
            "boom",
        )));
        let tree = view_tree(&model, noop(), noop());
        let content = text_content(&tree);
        assert!(content.contains("boom"), "surfaces the error message");
        assert!(content.contains("Retry"), "offers a retry");
    }

    #[test]
    fn last_page_renders_end_note_not_load_more() -> Result<(), ValidationError> {
        let mut model = TimelineModel::new();
        model.apply_initial(page(&[1], None)?);
        let tree = view_tree(&model, noop(), noop());
        let content = text_content(&tree);
        assert!(!content.contains("Load more"));
        assert!(content.contains("caught up"));
        Ok(())
    }

    fn element_key(node: &VNode) -> Option<String> {
        match node {
            VNode::Element(el) => el.key.clone(),
            VNode::Text(_) => None,
        }
    }
}
