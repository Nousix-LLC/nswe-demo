//! `/` -- the public landing view: the unauthenticated home page and public timeline.
//!
//! Loads `GET /api/chirps` on first display, renders a keyed reactive chirp list, and handles
//! loading, empty, error, end-of-list, and paginated states. The empty state explains what Chirp
//! is and offers prominent sign-up and sign-in entry points so logged-out visitors have a path
//! into the authenticated flow.
//!
//! # Reactive-state model
//!
//! Follows the same `thread_local!` + recreated-trigger-signal pattern as `timeline.rs`: the
//! view data lives in a module `thread_local!` `LandingModel` (plain data, never disposed), and
//! reactivity is driven by a re-render trigger `Signal<u32>` that this view recreates and reads
//! each render (subscribing the mount effect), and that async completions / event handlers bump
//! to request a re-render. The `ROUTE_PATH` guard in `bump` avoids touching a disposed trigger
//! after the user has navigated away.
//!
//! # Pagination
//!
//! Uses an ISO 8601 `before` cursor: after each successful page, `before_cursor` is set to the
//! `created_at` timestamp of the last chirp in the page. The next page request passes
//! `?before=<cursor>&limit=20` to fetch chirps created strictly before that timestamp.

use std::cell::{Cell, RefCell};

use ferric::prelude::*;
use ferric::vdom::text;
use ferric_router::prelude::{current_path, link};

use crate::api::{PublicChirp, PublicTimelineResponse};
use crate::app::AppContext;
use crate::error::AppError;

/// The path this view is mounted at. Guards `bump` to avoid touching a disposed trigger after
/// the user has navigated away while a request was in flight.
const ROUTE_PATH: &str = "/";

/// Number of chirps to request per page.
const PAGE_LIMIT: u32 = 20;

// -----------------------------------------------------------------------------------------
// State model
// -----------------------------------------------------------------------------------------

/// Which load a request represents.
#[derive(Clone, Copy)]
enum LoadKind {
    Initial,
    More,
}

/// The landing view's reactive state: accumulated chirps plus loading/error/pagination flags.
/// Mutated only through the reducer methods; transitions stay in one place and are unit-testable
/// without a DOM.
struct LandingModel {
    /// Whether the initial page has loaded at least once.
    loaded: bool,
    /// Accumulated chirps, newest first.
    chirps: Vec<PublicChirp>,
    /// ISO 8601 `created_at` of the last chirp; passed as `before` for the next page.
    before_cursor: Option<String>,
    /// Whether there are more pages (from the last response's `has_more`).
    has_more: bool,
    /// Whether a "load more" continuation is in flight.
    loading_more: bool,
    /// The most recent error, if any. Combined with `loaded` this selects the full-screen
    /// initial-error state vs. the inline load-more-error state.
    error: Option<AppError>,
}

impl LandingModel {
    const fn new() -> Self {
        LandingModel {
            loaded: false,
            chirps: Vec::new(),
            before_cursor: None,
            has_more: false,
            loading_more: false,
            error: None,
        }
    }

    fn begin_initial(&mut self) {
        self.loaded = false;
        self.chirps.clear();
        self.before_cursor = None;
        self.has_more = false;
        self.loading_more = false;
        self.error = None;
    }

    fn begin_more(&mut self) {
        self.loading_more = true;
        self.error = None;
    }

    fn apply_initial(&mut self, resp: PublicTimelineResponse) {
        self.before_cursor = resp.chirps.last().map(|c| c.created_at.clone());
        self.chirps = resp.chirps;
        self.has_more = resp.has_more;
        self.loaded = true;
        self.loading_more = false;
        self.error = None;
    }

    fn apply_more(&mut self, resp: PublicTimelineResponse) {
        // Keep existing cursor if the continuation page is empty (guard against empty more pages).
        if let Some(last) = resp.chirps.last() {
            self.before_cursor = Some(last.created_at.clone());
        }
        self.chirps.extend(resp.chirps);
        self.has_more = resp.has_more;
        self.loading_more = false;
        self.error = None;
    }

    fn apply_error(&mut self, err: AppError) {
        self.loading_more = false;
        self.error = Some(err);
    }
}

// -----------------------------------------------------------------------------------------
// Module-level thread_local state
// -----------------------------------------------------------------------------------------

thread_local! {
    static MODEL: RefCell<LandingModel> = const { RefCell::new(LandingModel::new()) };
    static RERENDER: RefCell<Option<Signal<u32>>> = const { RefCell::new(None) };
    static STARTED: Cell<bool> = const { Cell::new(false) };
}

// -----------------------------------------------------------------------------------------
// Public view entry point
// -----------------------------------------------------------------------------------------

/// `/` -- the public landing view.
///
/// Loads the public timeline on first display, renders a keyed chirp list with loading / empty /
/// error / loaded / end-of-list states, and always shows sign-up and sign-in navigation (AC-4).
#[must_use]
pub fn landing_view(ctx: &AppContext) -> VNode {
    // Recreate this render's trigger and subscribe the mount effect to it.
    let tick = create_signal(0u32);
    RERENDER.with(|cell| *cell.borrow_mut() = Some(tick));
    let _ = tick.get();

    // Kick off the first load exactly once.
    STARTED.with(|started| {
        if !started.get() {
            started.set(true);
            request_load(ctx.clone(), LoadKind::Initial);
        }
    });

    let more_ctx = ctx.clone();
    let on_load_more = move || trigger_more(more_ctx.clone());
    let retry_ctx = ctx.clone();
    let on_retry = move || reload(retry_ctx.clone());

    MODEL.with(|model| {
        let model = model.borrow();
        view_tree(&model, on_load_more, on_retry)
    })
}

// -----------------------------------------------------------------------------------------
// Action helpers
// -----------------------------------------------------------------------------------------

fn trigger_more(ctx: AppContext) {
    let proceed = MODEL.with(|model| {
        let mut m = model.borrow_mut();
        if m.loading_more || !m.has_more {
            return false;
        }
        m.begin_more();
        true
    });
    if proceed {
        bump();
        request_load(ctx, LoadKind::More);
    }
}

fn reload(ctx: AppContext) {
    MODEL.with(|model| model.borrow_mut().begin_initial());
    bump();
    request_load(ctx, LoadKind::Initial);
}

fn request_load(ctx: AppContext, kind: LoadKind) {
    let (before, limit) = MODEL.with(|model| {
        let m = model.borrow();
        match kind {
            LoadKind::Initial => (None, Some(PAGE_LIMIT)),
            LoadKind::More => (m.before_cursor.clone(), Some(PAGE_LIMIT)),
        }
    });

    wasm_bindgen_futures::spawn_local(async move {
        let result = ctx.client().public_timeline(before.as_deref(), limit).await;
        apply_result(kind, result);
    });
}

fn apply_result(kind: LoadKind, result: Result<PublicTimelineResponse, AppError>) {
    MODEL.with(|model| {
        let mut m = model.borrow_mut();
        match (kind, result) {
            (LoadKind::Initial, Ok(resp)) => m.apply_initial(resp),
            (LoadKind::More, Ok(resp)) => m.apply_more(resp),
            (_, Err(err)) => m.apply_error(err),
        }
    });
    bump();
}

/// Request a re-render by bumping the current trigger signal. Guarded by `ROUTE_PATH` to avoid
/// bumping a disposed handle after the user navigates away.
fn bump() {
    if current_path().as_str() != ROUTE_PATH {
        return;
    }
    let tick = RERENDER.with(|cell| *cell.borrow());
    if let Some(tick) = tick {
        tick.update(|n| *n = n.wrapping_add(1));
    }
}

// -----------------------------------------------------------------------------------------
// View tree (pure, unit-testable without a DOM)
// -----------------------------------------------------------------------------------------

fn view_tree<FMore, FRetry>(model: &LandingModel, on_load_more: FMore, on_retry: FRetry) -> VNode
where
    FMore: Fn() + 'static,
    FRetry: Fn() + 'static,
{
    let body: VNode = if !model.loaded {
        match &model.error {
            None => loading_view(),
            Some(err) => VElement::new("div")
                .attr("class", "chirp-landing-error")
                .child(error_banner(&err.user_message()))
                .child(button("Retry", on_retry))
                .into(),
        }
    } else if model.chirps.is_empty() {
        empty_view()
    } else {
        VElement::new("div")
            .attr("class", "chirp-landing-loaded")
            .child(chirp_list(&model.chirps))
            .child(footer(model, on_load_more))
            .into()
    };

    VElement::new("section")
        .attr("class", "chirp-landing")
        .child(header_nav())
        .child(body)
        .into()
}

/// Persistent sign-up / sign-in links shown in every landing-view state (AC-4).
fn header_nav() -> VNode {
    VElement::new("div")
        .attr("class", "chirp-landing-nav")
        .child(link("/signup", "Create account"))
        .child(text(" \u{00b7} "))
        .child(link("/signin", "Sign in"))
        .into()
}

fn loading_view() -> VNode {
    VElement::new("p")
        .attr("class", "chirp-loading")
        .child(text("Loading\u{2026}"))
        .into()
}

/// Empty state: explains what Chirp is and provides sign-up / sign-in entry points (AC-2).
fn empty_view() -> VNode {
    VElement::new("div")
        .attr("class", "chirp-landing-empty")
        .child(VElement::new("h2").child(text("Welcome to Chirp")))
        .child(
            VElement::new("p")
                .attr("class", "chirp-landing-description")
                .child(text(
                    "Chirp is a place to share short thoughts and follow people you find interesting.",
                )),
        )
        .child(
            VElement::new("div")
                .attr("class", "chirp-landing-cta")
                .child(link("/signup", "Create an account"))
                .child(text(" \u{00b7} "))
                .child(link("/signin", "Sign in")),
        )
        .into()
}

fn chirp_list(chirps: &[PublicChirp]) -> VNode {
    VElement::new("ul")
        .attr("class", "chirp-list")
        .children(chirps.iter().map(chirp_item))
        .into()
}

fn chirp_item(chirp: &PublicChirp) -> VNode {
    let profile_path = format!("/@{}", chirp.author.username);
    let display = chirp
        .author
        .display_name
        .as_deref()
        .unwrap_or(&chirp.author.username);
    VElement::new("li")
        .key(chirp.id.clone())
        .attr("class", "chirp")
        .child(
            VElement::new("div")
                .attr("class", "chirp-author")
                .child(link(&profile_path, display)),
        )
        .child(
            VElement::new("p")
                .attr("class", "chirp-text")
                .child(text(chirp.content.as_str())),
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
                        .child(text(chirp.created_at.as_str())),
                ),
        )
        .into()
}

fn footer<FMore>(model: &LandingModel, on_load_more: FMore) -> VNode
where
    FMore: Fn() + 'static,
{
    if let Some(err) = &model.error {
        VElement::new("div")
            .attr("class", "chirp-load-more-error")
            .child(error_banner(&err.user_message()))
            .child(button("Try again", on_load_more))
            .into()
    } else if model.loading_more {
        VElement::new("p")
            .attr("class", "chirp-loading-more")
            .child(text("Loading more\u{2026}"))
            .into()
    } else if model.has_more {
        button("Load more", on_load_more)
    } else {
        VElement::new("p")
            .attr("class", "chirp-end")
            .child(text("You\u{2019}re all caught up."))
            .into()
    }
}

fn error_banner(message: &str) -> VNode {
    VElement::new("p")
        .attr("class", "chirp-error")
        .attr("role", "alert")
        .child(text(message.to_string()))
        .into()
}

fn button(label: &str, on_click: impl Fn() + 'static) -> VNode {
    VElement::new("button")
        .attr("type", "button")
        .on(EventKind::Click, on_click)
        .child(text(label))
        .into()
}

// -----------------------------------------------------------------------------------------
// Tests -- native, no DOM. Exercise pure state-reduction and view-tree logic.
// -----------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::PublicAuthor;

    fn sample_chirp(id: &str, created_at: &str) -> PublicChirp {
        PublicChirp {
            id: id.to_string(),
            content: format!("content of {id}"),
            author: PublicAuthor {
                username: "alice".to_string(),
                display_name: Some("Alice".to_string()),
            },
            created_at: created_at.to_string(),
            like_count: 0,
        }
    }

    fn resp(chirps: Vec<PublicChirp>, has_more: bool) -> PublicTimelineResponse {
        PublicTimelineResponse { chirps, has_more }
    }

    fn noop() -> impl Fn() + 'static {
        || {}
    }

    fn find_class<'a>(node: &'a VNode, class: &str) -> Option<&'a VElement> {
        match node {
            VNode::Element(el) => {
                if el.attrs.iter().any(|(k, v)| k == "class" && v == class) {
                    return Some(el);
                }
                el.children.iter().find_map(|c| find_class(c, class))
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

    // --- state reduction ---

    #[test]
    fn apply_initial_marks_loaded_sets_cursor_and_has_more() {
        let mut m = LandingModel::new();
        let c = sample_chirp("id-1", "2024-01-15T12:00:00Z");
        m.apply_initial(resp(vec![c], true));
        assert!(m.loaded);
        assert_eq!(m.chirps.len(), 1);
        assert_eq!(m.before_cursor.as_deref(), Some("2024-01-15T12:00:00Z"));
        assert!(m.has_more);
        assert!(m.error.is_none());
    }

    #[test]
    fn apply_more_appends_and_updates_cursor() {
        let mut m = LandingModel::new();
        m.apply_initial(resp(vec![sample_chirp("a", "2024-01-15T12:00:00Z")], true));
        m.begin_more();
        m.apply_more(resp(vec![sample_chirp("b", "2024-01-10T08:00:00Z")], false));
        assert_eq!(m.chirps.len(), 2);
        assert_eq!(m.before_cursor.as_deref(), Some("2024-01-10T08:00:00Z"));
        assert!(!m.has_more);
        assert!(!m.loading_more);
    }

    #[test]
    fn apply_error_on_initial_keeps_unloaded() {
        let mut m = LandingModel::new();
        m.apply_error(AppError::Network("offline".to_string()));
        assert!(!m.loaded);
        assert!(m.error.is_some());
    }

    #[test]
    fn apply_error_after_load_keeps_loaded() {
        let mut m = LandingModel::new();
        m.apply_initial(resp(vec![sample_chirp("a", "2024-01-15T12:00:00Z")], true));
        m.begin_more();
        m.apply_error(AppError::Network("dropped".to_string()));
        assert!(m.loaded);
        assert!(m.error.is_some());
        assert!(!m.loading_more);
    }

    #[test]
    fn begin_initial_resets_all_state() {
        let mut m = LandingModel::new();
        m.apply_initial(resp(vec![sample_chirp("a", "2024-01-15T12:00:00Z")], true));
        m.begin_initial();
        assert!(!m.loaded);
        assert!(m.chirps.is_empty());
        assert!(m.before_cursor.is_none());
        assert!(!m.has_more);
        assert!(m.error.is_none());
    }

    // --- view tree ---

    #[test]
    fn unloaded_model_renders_loading_state() {
        let tree = view_tree(&LandingModel::new(), noop(), noop());
        assert!(find_class(&tree, "chirp-loading").is_some());
        assert!(find_class(&tree, "chirp-list").is_none());
    }

    #[test]
    fn every_state_includes_header_nav_links() {
        let tree = view_tree(&LandingModel::new(), noop(), noop());
        let nav = find_class(&tree, "chirp-landing-nav");
        assert!(nav.is_some(), "header nav must be present in loading state");
        let content = text_content(&tree);
        // Both sign-up and sign-in links must be present in every state (AC-4).
        assert!(
            content.contains("Create account") || content.contains("account"),
            "nav must contain a sign-up link, got: {content}"
        );
        assert!(
            content.contains("Sign in"),
            "nav must contain a sign-in link, got: {content}"
        );
    }

    #[test]
    fn empty_model_renders_welcome_statement_and_cta_links() {
        let mut m = LandingModel::new();
        m.apply_initial(resp(vec![], false));
        let tree = view_tree(&m, noop(), noop());
        assert!(find_class(&tree, "chirp-landing-empty").is_some());
        let content = text_content(&tree);
        assert!(
            content.contains("Welcome") || content.contains("Chirp"),
            "empty state must explain what Chirp is: got {content}"
        );
        assert!(
            content.contains("Create an account") || content.contains("account"),
            "empty state must show a sign-up link: got {content}"
        );
        assert!(
            content.contains("Sign in"),
            "empty state must show a sign-in link: got {content}"
        );
    }

    #[test]
    fn loaded_model_renders_chirp_list_and_load_more_button() {
        let mut m = LandingModel::new();
        m.apply_initial(resp(
            vec![
                sample_chirp("id-3", "2024-01-15T12:00:00Z"),
                sample_chirp("id-2", "2024-01-14T12:00:00Z"),
            ],
            true,
        ));
        let tree = view_tree(&m, noop(), noop());
        assert!(
            find_class(&tree, "chirp-list").is_some(),
            "loaded state must render chirp-list"
        );
        let content = text_content(&tree);
        assert!(
            content.contains("Load more"),
            "has_more=true must show Load more button"
        );
    }

    #[test]
    fn last_page_renders_end_note_not_load_more() {
        let mut m = LandingModel::new();
        m.apply_initial(resp(
            vec![sample_chirp("id-1", "2024-01-15T12:00:00Z")],
            false,
        ));
        let tree = view_tree(&m, noop(), noop());
        let content = text_content(&tree);
        assert!(
            !content.contains("Load more"),
            "has_more=false must not show Load more"
        );
        assert!(
            content.contains("caught up") || content.contains("end"),
            "end-of-list state must show end note: got {content}"
        );
    }

    #[test]
    fn initial_error_renders_error_message_and_retry_button() {
        let mut m = LandingModel::new();
        m.apply_error(AppError::Network("network down".to_string()));
        let tree = view_tree(&m, noop(), noop());
        let content = text_content(&tree);
        assert!(
            content.contains("network down") || content.contains("Network"),
            "error state must surface the error message: got {content}"
        );
        assert!(
            content.contains("Retry"),
            "error state must offer a Retry action: got {content}"
        );
    }
}
