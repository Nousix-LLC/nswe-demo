//! `/@:handle` -- the public user profile view.
//!
//! Accepts a `:handle` route parameter, fetches `GET /api/users/:username/chirps`, and renders
//! six distinct states: loading, not-found (HTTP 404), empty (user exists, no chirps), loaded
//! (user and chirps), end-of-list (last page displayed), and error (recoverable failure).
//!
//! # Reactive-state model
//!
//! Follows the same `thread_local!` + recreated-trigger-signal pattern as `landing.rs` and
//! `timeline.rs`. The view data lives in `ProfileModel` keyed by `handle`: when the route
//! handle changes the model resets to loading and a new fetch is triggered.
//!
//! # Pagination
//!
//! Uses an ISO 8601 `before` cursor from the `created_at` of the last loaded chirp, identical
//! to the landing view pagination strategy.

use std::cell::{Cell, RefCell};

use chirp_types::prelude::ErrorCode;
use ferric::prelude::*;
use ferric::vdom::text;
use ferric_router::prelude::{current_path, link};

use crate::api::{PublicUserChirp, PublicUserInfo, PublicUserResponse};
use crate::app::AppContext;
use crate::error::AppError;

/// Prefix for the profile route. Used by `bump` to guard against touching a disposed trigger
/// after the user navigates away from any `/@...` path.
const ROUTE_PREFIX: &str = "/@";

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

/// The load phase for the profile view.
#[derive(Clone, Default)]
enum LoadPhase {
    /// No data loaded yet (or reset).
    #[default]
    Loading,
    /// The server returned 404 for this handle.
    NotFound,
    /// A non-404 error occurred.
    Error(AppError),
    /// At least one page has loaded.
    Loaded,
}

/// The profile view's reactive state, keyed by `handle`.
struct ProfileModel {
    /// The handle whose profile this state describes.
    handle: Option<String>,
    /// Current load phase.
    phase: LoadPhase,
    /// User info from the first loaded page.
    user_info: Option<PublicUserInfo>,
    /// Accumulated chirps.
    chirps: Vec<PublicUserChirp>,
    /// ISO 8601 `created_at` of the last chirp; passed as `before` for the next page.
    before_cursor: Option<String>,
    /// Whether there are more pages (from the last response's `has_more`).
    has_more: bool,
    /// Whether a "load more" continuation is in flight.
    loading_more: bool,
    /// Inline error after a successful first load (continuation failure).
    continuation_error: Option<AppError>,
}

impl ProfileModel {
    const fn new() -> Self {
        ProfileModel {
            handle: None,
            phase: LoadPhase::Loading,
            user_info: None,
            chirps: Vec::new(),
            before_cursor: None,
            has_more: false,
            loading_more: false,
            continuation_error: None,
        }
    }

    /// Reset for a new handle.
    fn reset_for(&mut self, handle: String) {
        self.handle = Some(handle);
        self.phase = LoadPhase::Loading;
        self.user_info = None;
        self.chirps.clear();
        self.before_cursor = None;
        self.has_more = false;
        self.loading_more = false;
        self.continuation_error = None;
    }

    fn begin_more(&mut self) {
        self.loading_more = true;
        self.continuation_error = None;
    }

    fn apply_initial(&mut self, resp: PublicUserResponse) {
        self.before_cursor = resp.chirps.last().map(|c| c.created_at.clone());
        self.user_info = Some(resp.user);
        self.chirps = resp.chirps;
        self.has_more = resp.has_more;
        self.phase = LoadPhase::Loaded;
        self.loading_more = false;
        self.continuation_error = None;
    }

    fn apply_more(&mut self, resp: PublicUserResponse) {
        if let Some(last) = resp.chirps.last() {
            self.before_cursor = Some(last.created_at.clone());
        }
        self.chirps.extend(resp.chirps);
        self.has_more = resp.has_more;
        self.loading_more = false;
        self.continuation_error = None;
    }

    fn apply_error(&mut self, err: AppError, kind: LoadKind) {
        let is_not_found = matches!(&err, AppError::Api(api) if api.code == ErrorCode::NotFound);
        match kind {
            LoadKind::Initial => {
                self.phase = if is_not_found {
                    LoadPhase::NotFound
                } else {
                    LoadPhase::Error(err)
                };
            }
            LoadKind::More => {
                self.loading_more = false;
                self.continuation_error = Some(err);
            }
        }
    }
}

// -----------------------------------------------------------------------------------------
// Module-level thread_local state
// -----------------------------------------------------------------------------------------

thread_local! {
    static MODEL: RefCell<ProfileModel> = const { RefCell::new(ProfileModel::new()) };
    static RERENDER: RefCell<Option<Signal<u32>>> = const { RefCell::new(None) };
    /// Tracks whether the initial load for the current handle has been started.
    static STARTED: Cell<bool> = const { Cell::new(false) };
}

// -----------------------------------------------------------------------------------------
// Public view entry point
// -----------------------------------------------------------------------------------------

/// `/@:handle` -- the public user profile view.
///
/// Fetches the user's chirp list, renders six distinct states: loading, not-found, empty,
/// loaded, end-of-list, and error.
#[must_use]
pub fn public_profile_view(ctx: &AppContext, handle: String) -> VNode {
    // Recreate this render's trigger and subscribe the mount effect to it.
    let tick = create_signal(0u32);
    RERENDER.with(|cell| *cell.borrow_mut() = Some(tick));
    let _ = tick.get();

    // Reset and re-fetch when the handle changes.
    let needs_load = MODEL.with(|model| {
        let mut m = model.borrow_mut();
        if m.handle.as_deref() != Some(handle.as_str()) {
            m.reset_for(handle.clone());
            STARTED.with(|s| s.set(false));
            true
        } else {
            false
        }
    });

    // Kick off the initial fetch exactly once per handle.
    STARTED.with(|started| {
        if !started.get() || needs_load {
            started.set(true);
            request_load(ctx.clone(), handle.clone(), LoadKind::Initial);
        }
    });

    let more_ctx = ctx.clone();
    let more_handle = handle.clone();
    let on_load_more = move || trigger_more(more_ctx.clone(), more_handle.clone());
    let retry_ctx = ctx.clone();
    let retry_handle = handle.clone();
    let on_retry = move || reload(retry_ctx.clone(), retry_handle.clone());

    MODEL.with(|model| {
        let m = model.borrow();
        view_tree(&m, on_load_more, on_retry)
    })
}

// -----------------------------------------------------------------------------------------
// Action helpers
// -----------------------------------------------------------------------------------------

fn trigger_more(ctx: AppContext, handle: String) {
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
        request_load(ctx, handle, LoadKind::More);
    }
}

fn reload(ctx: AppContext, handle: String) {
    MODEL.with(|model| model.borrow_mut().reset_for(handle.clone()));
    bump();
    request_load(ctx, handle, LoadKind::Initial);
}

fn request_load(ctx: AppContext, handle: String, kind: LoadKind) {
    let (before, limit) = MODEL.with(|model| {
        let m = model.borrow();
        match kind {
            LoadKind::Initial => (None, Some(PAGE_LIMIT)),
            LoadKind::More => (m.before_cursor.clone(), Some(PAGE_LIMIT)),
        }
    });

    wasm_bindgen_futures::spawn_local(async move {
        let result = ctx
            .client()
            .public_user_profile(&handle, before.as_deref(), limit)
            .await;
        // Guard: if the user navigated to a different profile, discard this result.
        let still_current =
            MODEL.with(|model| model.borrow().handle.as_deref() == Some(handle.as_str()));
        if still_current {
            apply_result(result, kind);
        }
    });
}

fn apply_result(result: Result<PublicUserResponse, AppError>, kind: LoadKind) {
    MODEL.with(|model| {
        let mut m = model.borrow_mut();
        match (kind, result) {
            (LoadKind::Initial, Ok(resp)) => m.apply_initial(resp),
            (LoadKind::More, Ok(resp)) => m.apply_more(resp),
            (kind, Err(err)) => m.apply_error(err, kind),
        }
    });
    bump();
}

/// Request a re-render, guarded against bumping a disposed trigger after navigation away.
fn bump() {
    if !current_path().as_str().starts_with(ROUTE_PREFIX) {
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

fn view_tree<FMore, FRetry>(model: &ProfileModel, on_load_more: FMore, on_retry: FRetry) -> VNode
where
    FMore: Fn() + 'static,
    FRetry: Fn() + 'static,
{
    let handle = model.handle.as_deref().unwrap_or("");

    match &model.phase {
        LoadPhase::Loading => loading_view(handle),
        LoadPhase::NotFound => not_found_view(handle),
        LoadPhase::Error(err) => error_view(handle, &err.user_message(), on_retry),
        LoadPhase::Loaded => loaded_view(model, on_load_more),
    }
}

fn loading_view(handle: &str) -> VNode {
    VElement::new("section")
        .attr("class", "chirp-profile chirp-profile-loading")
        .child(VElement::new("h1").child(text(format!("@{handle}"))))
        .child(
            VElement::new("p")
                .attr("class", "chirp-loading")
                .child(text("Loading\u{2026}")),
        )
        .into()
}

fn not_found_view(handle: &str) -> VNode {
    VElement::new("section")
        .attr("class", "chirp-profile chirp-profile-not-found")
        .child(VElement::new("h1").child(text(format!("@{handle}"))))
        .child(VElement::new("p").child(text("This user does not exist.")))
        .child(link("/", "Go to home"))
        .into()
}

fn error_view<FRetry>(handle: &str, message: &str, on_retry: FRetry) -> VNode
where
    FRetry: Fn() + 'static,
{
    VElement::new("section")
        .attr("class", "chirp-profile chirp-profile-error")
        .child(VElement::new("h1").child(text(format!("@{handle}"))))
        .child(error_banner(message))
        .child(
            VElement::new("button")
                .attr("type", "button")
                .on(EventKind::Click, on_retry)
                .child(text("Retry")),
        )
        .into()
}

fn loaded_view<FMore>(model: &ProfileModel, on_load_more: FMore) -> VNode
where
    FMore: Fn() + 'static,
{
    let handle = model.handle.as_deref().unwrap_or("");
    let mut section = VElement::new("section").attr("class", "chirp-profile");

    // User header
    if let Some(info) = &model.user_info {
        let display = info.display_name.as_deref().unwrap_or(&info.username);
        section = section.child(VElement::new("h1").child(text(display.to_string())));
        section = section.child(
            VElement::new("p")
                .attr("class", "chirp-handle")
                .child(text(format!("@{}", info.username))),
        );
        if let Some(bio) = &info.bio {
            section = section.child(
                VElement::new("p")
                    .attr("class", "chirp-bio")
                    .child(text(bio.clone())),
            );
        }
        section = section.child(
            VElement::new("div").attr("class", "chirp-stats").child(
                VElement::new("span")
                    .attr("class", "chirp-stat")
                    .child(VElement::new("strong").child(text(info.chirp_count.to_string())))
                    .child(text(" chirps")),
            ),
        );
    } else {
        section = section.child(VElement::new("h1").child(text(format!("@{handle}"))));
    }

    // Chirp list or empty state
    if model.chirps.is_empty() {
        section = section.child(
            VElement::new("p")
                .attr("class", "chirp-profile-empty")
                .child(text("No chirps yet.")),
        );
    } else {
        section = section.child(chirp_list(&model.chirps));
        section = section.child(profile_footer(model, on_load_more));
    }

    section.into()
}

fn chirp_list(chirps: &[PublicUserChirp]) -> VNode {
    VElement::new("ul")
        .attr("class", "chirp-list")
        .children(chirps.iter().map(profile_chirp_item))
        .into()
}

fn profile_chirp_item(chirp: &PublicUserChirp) -> VNode {
    VElement::new("li")
        .key(chirp.id.clone())
        .attr("class", "chirp")
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

fn profile_footer<FMore>(model: &ProfileModel, on_load_more: FMore) -> VNode
where
    FMore: Fn() + 'static,
{
    if let Some(err) = &model.continuation_error {
        VElement::new("div")
            .attr("class", "chirp-load-more-error")
            .child(error_banner(&err.user_message()))
            .child(
                VElement::new("button")
                    .attr("type", "button")
                    .on(EventKind::Click, on_load_more)
                    .child(text("Try again")),
            )
            .into()
    } else if model.loading_more {
        VElement::new("p")
            .attr("class", "chirp-loading-more")
            .child(text("Loading more\u{2026}"))
            .into()
    } else if model.has_more {
        VElement::new("button")
            .attr("type", "button")
            .on(EventKind::Click, on_load_more)
            .child(text("Load more"))
            .into()
    } else {
        VElement::new("p")
            .attr("class", "chirp-end")
            .child(text("No more chirps."))
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

// -----------------------------------------------------------------------------------------
// Tests -- native, no DOM. Exercise pure state-reduction and view-tree logic.
// -----------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_chirp(id: &str, created_at: &str) -> PublicUserChirp {
        PublicUserChirp {
            id: id.to_string(),
            content: format!("content of {id}"),
            created_at: created_at.to_string(),
            like_count: 0,
        }
    }

    fn user_info() -> PublicUserInfo {
        PublicUserInfo {
            username: "alice".to_string(),
            display_name: Some("Alice".to_string()),
            bio: Some("Just a test user.".to_string()),
            chirp_count: 2,
        }
    }

    fn resp(chirps: Vec<PublicUserChirp>, has_more: bool) -> PublicUserResponse {
        PublicUserResponse {
            user: user_info(),
            chirps,
            has_more,
        }
    }

    fn noop() -> impl Fn() + 'static {
        || {}
    }

    fn find_class<'a>(node: &'a VNode, class: &str) -> Option<&'a VElement> {
        match node {
            VNode::Element(el) => {
                // Check if the element has the CSS class as one of its whitespace-separated tokens.
                if el
                    .attrs
                    .iter()
                    .any(|(k, v)| k == "class" && v.split_whitespace().any(|token| token == class))
                {
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

    fn make_model(handle: &str) -> ProfileModel {
        let mut m = ProfileModel::new();
        m.reset_for(handle.to_string());
        m
    }

    // --- state reduction ---

    #[test]
    fn reset_for_clears_all_prior_state() {
        let mut m = make_model("alice");
        m.apply_initial(resp(
            vec![sample_chirp("c1", "2024-01-15T00:00:00Z")],
            false,
        ));
        m.reset_for("bob".to_string());
        assert_eq!(m.handle.as_deref(), Some("bob"));
        assert!(matches!(m.phase, LoadPhase::Loading));
        assert!(m.chirps.is_empty());
        assert!(m.user_info.is_none());
    }

    #[test]
    fn apply_initial_transitions_to_loaded() {
        let mut m = make_model("alice");
        m.apply_initial(resp(vec![sample_chirp("c1", "2024-01-15T00:00:00Z")], true));
        assert!(matches!(m.phase, LoadPhase::Loaded));
        assert_eq!(m.chirps.len(), 1);
        assert!(m.user_info.is_some());
        assert!(m.has_more);
        assert_eq!(m.before_cursor.as_deref(), Some("2024-01-15T00:00:00Z"));
    }

    #[test]
    fn apply_more_appends_chirps() {
        let mut m = make_model("alice");
        m.apply_initial(resp(vec![sample_chirp("c1", "2024-01-15T00:00:00Z")], true));
        m.begin_more();
        m.apply_more(resp(
            vec![sample_chirp("c2", "2024-01-10T00:00:00Z")],
            false,
        ));
        assert_eq!(m.chirps.len(), 2);
        assert!(!m.has_more);
    }

    #[test]
    fn apply_error_on_initial_not_found_sets_not_found_phase() {
        use chirp_types::prelude::ApiError;
        let mut m = make_model("nobody");
        let not_found = AppError::Api(ApiError::new(ErrorCode::NotFound, "user not found"));
        m.apply_error(not_found, LoadKind::Initial);
        assert!(matches!(m.phase, LoadPhase::NotFound));
    }

    #[test]
    fn apply_error_on_initial_server_error_sets_error_phase() {
        let mut m = make_model("alice");
        m.apply_error(AppError::Network("offline".to_string()), LoadKind::Initial);
        assert!(matches!(m.phase, LoadPhase::Error(_)));
    }

    #[test]
    fn apply_error_on_more_sets_continuation_error() {
        let mut m = make_model("alice");
        m.apply_initial(resp(vec![sample_chirp("c1", "2024-01-15T00:00:00Z")], true));
        m.begin_more();
        m.apply_error(AppError::Network("dropped".to_string()), LoadKind::More);
        assert!(matches!(m.phase, LoadPhase::Loaded));
        assert!(m.continuation_error.is_some());
        assert!(!m.loading_more);
    }

    // --- view tree ---

    #[test]
    fn loading_state_renders_loading_indicator() {
        let m = make_model("alice");
        let tree = view_tree(&m, noop(), noop());
        assert!(find_class(&tree, "chirp-profile-loading").is_some());
        let content = text_content(&tree);
        assert!(
            content.contains("Loading") || content.contains('\u{2026}'),
            "loading state must render a loading indicator: got {content}"
        );
    }

    #[test]
    fn not_found_state_renders_distinct_message() {
        let mut m = make_model("nobody");
        use chirp_types::prelude::ApiError;
        m.apply_error(
            AppError::Api(ApiError::new(ErrorCode::NotFound, "user not found")),
            LoadKind::Initial,
        );
        let tree = view_tree(&m, noop(), noop());
        assert!(find_class(&tree, "chirp-profile-not-found").is_some());
        let content = text_content(&tree);
        assert!(
            content.contains("does not exist") || content.contains("not found"),
            "not-found state must explain user does not exist: got {content}"
        );
    }

    #[test]
    fn error_state_renders_message_and_retry() {
        let mut m = make_model("alice");
        m.apply_error(
            AppError::Network("network down".to_string()),
            LoadKind::Initial,
        );
        let tree = view_tree(&m, noop(), noop());
        assert!(find_class(&tree, "chirp-profile-error").is_some());
        let content = text_content(&tree);
        assert!(
            content.contains("network down") || content.contains("Network"),
            "error state must surface the message: got {content}"
        );
        assert!(content.contains("Retry"), "error state must offer Retry");
    }

    #[test]
    fn empty_loaded_state_renders_empty_message() {
        let mut m = make_model("alice");
        m.apply_initial(resp(vec![], false));
        let tree = view_tree(&m, noop(), noop());
        let content = text_content(&tree);
        assert!(
            content.contains("No chirps"),
            "empty loaded state must say no chirps: got {content}"
        );
        assert!(
            find_class(&tree, "chirp-list").is_none(),
            "no chirp-list in empty state"
        );
    }

    #[test]
    fn loaded_with_chirps_renders_list_and_load_more() {
        let mut m = make_model("alice");
        m.apply_initial(resp(vec![sample_chirp("c1", "2024-01-15T00:00:00Z")], true));
        let tree = view_tree(&m, noop(), noop());
        assert!(find_class(&tree, "chirp-list").is_some());
        let content = text_content(&tree);
        assert!(
            content.contains("Load more"),
            "has_more=true must show Load more"
        );
    }

    #[test]
    fn end_of_list_renders_end_note() {
        let mut m = make_model("alice");
        m.apply_initial(resp(
            vec![sample_chirp("c1", "2024-01-15T00:00:00Z")],
            false,
        ));
        let tree = view_tree(&m, noop(), noop());
        let content = text_content(&tree);
        assert!(
            !content.contains("Load more"),
            "has_more=false must not show Load more"
        );
        assert!(
            content.contains("No more"),
            "end-of-list must render an end note: got {content}"
        );
    }
}
