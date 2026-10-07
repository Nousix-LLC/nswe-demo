//! `/user/:id` — the user profile view with follow/like actions.
//!
//! Owned by `SUBTASK_profile`; realizes the frozen `profile_view` signature from
//! `contracts/app_api.rs`. It reads the `:id` route parameter, loads the `User` via
//! [`ApiClient::get_user`](crate::api::ApiClient::get_user) (rendering loading / error / not-found /
//! loaded states), and offers **follow / unfollow** (`set_follow`) and **like**
//! (`set_like`) actions whose results update the on-screen counts reactively.
//!
//! # Reactivity model (why state lives in a `thread_local`, not in view-local signals)
//!
//! `ferric` has no per-component state: [`mount`](ferric::dom::mount) wraps the *whole* app render
//! in a single reactive effect, and any [`Signal`](ferric::prelude::Signal) created **during** a
//! render is owned by that effect and disposed when it next re-runs (see `ferric::signal` and
//! `ferric::dom`). A route view is re-invoked on every re-render, so signals it creates cannot
//! survive, and an event handler must only capture signals that are *not* disposed — i.e. runtime
//! root signals created at app setup. The scaffold's [`AppContext::session`](crate::app::AppContext)
//! is exactly such a root signal.
//!
//! This module therefore keeps the profile's mutable state in a module-local [`thread_local`] store
//! of **plain data** (which survives re-renders because it is ordinary Rust, not a reactive node),
//! and drives reactive re-renders through the persistent session signal: `profile_view` reads
//! `session` every render (also the legitimate check for "is the viewer logged in?"), and async
//! completions / click handlers update the plain store and then nudge `session` via
//! `request_render` so `ferric` re-renders the app and this view re-reads the store. Nudging a
//! root signal is panic-free; capturing a per-render signal in a handler would not be.
//!
//! Async work is bridged to the UI exactly as the subtree constraints require: a browser task
//! (`wasm_bindgen_futures::spawn_local`) performs the `fetch`, then writes the store and requests a
//! render on completion. Browser-only calls are isolated behind `#[cfg(target_arch = "wasm32")]`
//! (mirroring `ferric` / `ferric-router`), so this module also compiles and unit-tests natively.

use std::cell::RefCell;

use chirp_types::prelude::{ChirpId, ErrorCode, User, UserId};
use ferric::prelude::{EventKind, Signal, VElement, VNode};
use ferric::vdom::text;
use ferric_router::prelude::Params;

use crate::app::{AppContext, Session};

// ---------------------------------------------------------------------------------------------
// View state — plain data held across re-renders in a module-local store (see the module docs).
// ---------------------------------------------------------------------------------------------

/// The load state of the profile currently being shown.
#[derive(Clone, Default)]
enum Load {
    /// No profile requested yet.
    #[default]
    Idle,
    /// A `get_user` request is in flight.
    Loading,
    /// The user loaded successfully.
    Loaded(User),
    /// The server reported the user does not exist (`NOT_FOUND`).
    NotFound,
    /// The load failed; carries a user-facing message.
    Failed(String),
}

/// The profile view's mutable state. Plain data, so it survives `ferric` re-renders (unlike a
/// signal created during a render). Keyed implicitly by `requested`: when the route id changes the
/// whole state is reset to [`ProfileState::loading`].
#[derive(Clone, Default)]
struct ProfileState {
    /// The id whose profile this state describes.
    requested: Option<UserId>,
    /// Load status for `requested`.
    load: Load,
    /// Whether the viewer follows the target. `None` until a follow action resolves.
    following: Option<bool>,
    /// The target's follower count: seeded from the loaded `User`, then updated by `FollowResponse`.
    follower_count: Option<u64>,
    /// A follow/unfollow request is in flight.
    follow_busy: bool,
    /// The last follow action's error, if any.
    follow_error: Option<String>,
    /// Whether the viewer likes the representative chirp. `None` until a like action resolves.
    like_liked: Option<bool>,
    /// The representative chirp's like count after the last like action.
    like_count: Option<u64>,
    /// A like/unlike request is in flight.
    like_busy: bool,
    /// The last like action's error, if any.
    like_error: Option<String>,
}

impl ProfileState {
    /// Fresh state for a newly requested `id`, in the `Loading` phase.
    fn loading(id: UserId) -> Self {
        ProfileState {
            requested: Some(id),
            load: Load::Loading,
            ..Self::default()
        }
    }
}

thread_local! {
    /// The single, app-lifetime profile-view store (the wasm app is single-threaded).
    static STORE: RefCell<ProfileState> = RefCell::new(ProfileState::default());
}

/// Run `f` with exclusive access to the store, scoping the borrow to the call.
fn with_store<R>(f: impl FnOnce(&mut ProfileState) -> R) -> R {
    STORE.with(|cell| f(&mut cell.borrow_mut()))
}

/// Ask `ferric` to re-render by nudging the persistent session signal.
///
/// `profile_view` reads `session` every render, so `ferric`'s mount effect is subscribed to it;
/// bumping it (with an unchanged value) re-runs the render so the view re-reads the store. `session`
/// is a runtime-root signal (created in [`AppContext::new`](crate::app::AppContext::new) outside any
/// effect), so it is never disposed and this never panics — see the module-level docs.
fn request_render(session: Signal<Option<Session>>) {
    session.update(|_| {});
}

// ---------------------------------------------------------------------------------------------
// Async bridge — a browser task performs the fetch, then writes the store and requests a render.
// ---------------------------------------------------------------------------------------------

/// Spawn a browser task on wasm; a no-op off-wasm (native builds exercise only the pure helpers).
#[cfg(target_arch = "wasm32")]
fn spawn(future: impl std::future::Future<Output = ()> + 'static) {
    wasm_bindgen_futures::spawn_local(future);
}

/// Off-wasm there is no browser executor, so the task is dropped unrun — native builds never drive
/// async I/O (they test the pure, off-DOM logic only). Mirrors how `ferric` isolates browser calls.
#[cfg(not(target_arch = "wasm32"))]
fn spawn(future: impl std::future::Future<Output = ()> + 'static) {
    #[allow(clippy::let_underscore_future)]
    let _ = future;
}

/// Load the user for `id` and fold the outcome into the store (ignored if the view has since moved
/// to another id), then request a render.
async fn load_profile(ctx: AppContext, id: UserId) {
    let result = ctx.client().get_user(id).await;
    with_store(|s| {
        if s.requested != Some(id) {
            return; // navigated away / id changed: this result is stale.
        }
        match result {
            Ok(user) => {
                s.follower_count = Some(user.follower_count);
                s.load = Load::Loaded(user);
            }
            Err(err) => {
                s.load = if err.code() == Some(ErrorCode::NotFound) {
                    Load::NotFound
                } else {
                    Load::Failed(err.user_message())
                };
            }
        }
    });
    request_render(ctx.session);
}

/// Toggle the viewer's follow relationship with `id`, reflecting the returned `FollowResponse`
/// (`following` + `follower_count`) into the store, then request a render.
async fn toggle_follow(ctx: AppContext, id: UserId) {
    let target = next_toggle(with_store(|s| s.following));
    let result = ctx.client().set_follow(id, target).await;
    with_store(|s| {
        s.follow_busy = false;
        match result {
            Ok(resp) => {
                s.following = Some(resp.following);
                s.follower_count = Some(resp.follower_count);
                s.follow_error = None;
            }
            Err(err) => s.follow_error = Some(err.user_message()),
        }
    });
    request_render(ctx.session);
}

/// Toggle the viewer's like of the representative `chirp`, reflecting the returned `LikeResponse`
/// (`liked` + `like_count`) into the store, then request a render.
async fn toggle_like(ctx: AppContext, chirp: ChirpId) {
    let target = next_toggle(with_store(|s| s.like_liked));
    let result = ctx.client().set_like(chirp, target).await;
    with_store(|s| {
        s.like_busy = false;
        match result {
            Ok(resp) => {
                s.like_liked = Some(resp.liked);
                s.like_count = Some(resp.like_count);
                s.like_error = None;
            }
            Err(err) => s.like_error = Some(err.user_message()),
        }
    });
    request_render(ctx.session);
}

// ---------------------------------------------------------------------------------------------
// Pure, off-DOM logic (unit-tested below).
// ---------------------------------------------------------------------------------------------

/// Parse a `/user/:id` route segment into a [`UserId`], or `None` when it is not a `u64`.
fn parse_user_id(raw: &str) -> Option<UserId> {
    raw.parse::<u64>().ok().map(UserId::new)
}

/// The target state of a boolean toggle (follow or like) given its current value; an unknown
/// (`None`) current state is treated as "off", so the first action turns it on.
fn next_toggle(current: Option<bool>) -> bool {
    !current.unwrap_or(false)
}

/// The follow button's label for the given busy / following state.
fn follow_label(busy: bool, following: Option<bool>) -> String {
    if busy {
        "Working…".to_string()
    } else if following == Some(true) {
        "Unfollow".to_string()
    } else {
        "Follow".to_string()
    }
}

/// The like button's label for the given busy / liked state and (optional) known like count.
fn like_label(busy: bool, liked: Option<bool>, count: Option<u64>) -> String {
    let verb = if busy {
        "Working…"
    } else if liked == Some(true) {
        "Unlike"
    } else {
        "Like"
    };
    match count {
        Some(n) => format!("{verb} ({n})"),
        None => verb.to_string(),
    }
}

// ---------------------------------------------------------------------------------------------
// View builders.
// ---------------------------------------------------------------------------------------------

/// `/user/:id` — the profile route. Loads the user and renders loading / error / not-found / loaded
/// states with reactive follow and like controls. Realizes the frozen `app_api.rs` signature.
#[must_use]
pub fn profile_view(ctx: &AppContext, params: &Params) -> VNode {
    // Reactive read: subscribes this render to the session signal (also the auth gate below).
    let viewer_id = ctx
        .session
        .with(|s| s.as_ref().map(|session| session.user.id));

    let Some(id) = params.get("id").and_then(parse_user_id) else {
        return message_view(
            "chirp-profile-invalid",
            "Invalid profile",
            "The user id in the address is not a valid number.",
        );
    };

    // Kick off (or restart, when the id changed) the load exactly once per requested id.
    let needs_load = with_store(|s| {
        if s.requested == Some(id) {
            false
        } else {
            *s = ProfileState::loading(id);
            true
        }
    });
    if needs_load {
        spawn(load_profile(ctx.clone(), id));
    }

    let state = with_store(|s| s.clone());
    match state.load.clone() {
        Load::Idle | Load::Loading => loading_view(id),
        Load::NotFound => message_view(
            "chirp-profile-missing",
            "User not found",
            "No user exists with that id.",
        ),
        Load::Failed(message) => failed_view(ctx, id, &message),
        Load::Loaded(user) => loaded_view(ctx, &user, &state, viewer_id),
    }
}

/// The in-progress placeholder shown while the profile loads.
fn loading_view(id: UserId) -> VNode {
    VElement::new("section")
        .attr("class", "chirp-profile chirp-profile-loading")
        .child(VElement::new("h1").child(text("Profile")))
        .child(VElement::new("p").child(text(format!("Loading {id}…"))))
        .into()
}

/// A generic titled message panel (invalid id / not found).
fn message_view(class: &str, title: &str, body: &str) -> VNode {
    VElement::new("section")
        .attr("class", format!("chirp-profile {class}"))
        .child(VElement::new("h1").child(text(title.to_string())))
        .child(VElement::new("p").child(text(body.to_string())))
        .into()
}

/// The load-failure panel, with a Retry control that re-issues the load.
fn failed_view(ctx: &AppContext, id: UserId, message: &str) -> VNode {
    let ctx_retry = ctx.clone();
    let retry = move || {
        let proceed = with_store(|s| {
            if matches!(s.load, Load::Loading) {
                false
            } else {
                s.load = Load::Loading;
                true
            }
        });
        if proceed {
            request_render(ctx_retry.session);
            spawn(load_profile(ctx_retry.clone(), id));
        }
    };

    VElement::new("section")
        .attr("class", "chirp-profile chirp-profile-error")
        .child(VElement::new("h1").child(text("Couldn't load profile")))
        .child(error_banner(message))
        .child(
            VElement::new("button")
                .attr("type", "button")
                .on(EventKind::Click, retry)
                .child(text("Retry")),
        )
        .into()
}

/// The loaded profile card: identity, counts, and the follow + like controls.
fn loaded_view(
    ctx: &AppContext,
    user: &User,
    state: &ProfileState,
    viewer_id: Option<UserId>,
) -> VNode {
    let follower_count = state.follower_count.unwrap_or(user.follower_count);

    let mut card = VElement::new("section")
        .attr("class", "chirp-profile")
        .child(VElement::new("h1").child(text(user.display_name.clone())))
        .child(
            VElement::new("p")
                .attr("class", "chirp-handle")
                .child(text(format!("@{}", user.username.as_str()))),
        );

    if let Some(bio) = &user.bio {
        card = card.child(
            VElement::new("p")
                .attr("class", "chirp-bio")
                .child(text(bio.clone())),
        );
    }

    card = card.child(
        VElement::new("div")
            .attr("class", "chirp-stats")
            .child(stat("followers", follower_count))
            .child(stat("following", user.following_count)),
    );
    card = card.child(follow_control(ctx, user.id, state, viewer_id));
    card = card.child(like_control(ctx, user.id, state, viewer_id.is_some()));
    card.into()
}

/// A single `<strong>count</strong> label` statistic.
fn stat(label: &str, value: u64) -> VNode {
    VElement::new("span")
        .attr("class", "chirp-stat")
        .child(VElement::new("strong").child(text(value.to_string())))
        .child(text(format!(" {label}")))
        .into()
}

/// The follow / unfollow control, gated on being a logged-in viewer other than the target.
fn follow_control(
    ctx: &AppContext,
    id: UserId,
    state: &ProfileState,
    viewer_id: Option<UserId>,
) -> VNode {
    match viewer_id {
        None => hint("Log in to follow this user."),
        Some(viewer) if viewer == id => note("This is you."),
        Some(_) => {
            let ctx_follow = ctx.clone();
            let on_click = move || {
                let proceed = with_store(|s| {
                    if s.follow_busy {
                        false
                    } else {
                        s.follow_busy = true;
                        s.follow_error = None;
                        true
                    }
                });
                if proceed {
                    request_render(ctx_follow.session);
                    spawn(toggle_follow(ctx_follow.clone(), id));
                }
            };

            let mut button = VElement::new("button")
                .attr("type", "button")
                .attr("class", "chirp-follow-button")
                .on(EventKind::Click, on_click)
                .child(text(follow_label(state.follow_busy, state.following)));
            if state.follow_busy {
                button = button.attr("disabled", "");
            }

            let mut wrap = VElement::new("div")
                .attr("class", "chirp-follow")
                .child(button);
            if let Some(message) = &state.follow_error {
                wrap = wrap.child(error_banner(message));
            }
            wrap.into()
        }
    }
}

/// The like control. The profile endpoint returns no chirps, so this is a **representative** like
/// control (the brief sanctions this): it demonstrates the `set_like` wiring against a representative
/// chirp id derived from the profile, reflecting the returned `LikeResponse` reactively.
fn like_control(
    ctx: &AppContext,
    profile_id: UserId,
    state: &ProfileState,
    logged_in: bool,
) -> VNode {
    if !logged_in {
        return hint("Log in to like chirps.");
    }

    // Representative target: the profile view carries no chirp of its own, so this stands in for
    // "a chirp on this profile" to exercise the like action end to end.
    let chirp = ChirpId::new(profile_id.get());
    let ctx_like = ctx.clone();
    let on_click = move || {
        let proceed = with_store(|s| {
            if s.like_busy {
                false
            } else {
                s.like_busy = true;
                s.like_error = None;
                true
            }
        });
        if proceed {
            request_render(ctx_like.session);
            spawn(toggle_like(ctx_like.clone(), chirp));
        }
    };

    let mut button = VElement::new("button")
        .attr("type", "button")
        .attr("class", "chirp-like-button")
        .on(EventKind::Click, on_click)
        .child(text(like_label(
            state.like_busy,
            state.like_liked,
            state.like_count,
        )));
    if state.like_busy {
        button = button.attr("disabled", "");
    }

    let mut wrap = VElement::new("div")
        .attr("class", "chirp-like")
        .child(button)
        .child(
            VElement::new("p")
                .attr("class", "chirp-like-note")
                .child(text("Representative like control for this profile.")),
        );
    if let Some(message) = &state.like_error {
        wrap = wrap.child(error_banner(message));
    }
    wrap.into()
}

/// A dimmed informational hint (e.g. "log in to follow").
fn hint(message: &str) -> VNode {
    VElement::new("p")
        .attr("class", "chirp-hint")
        .child(text(message.to_string()))
        .into()
}

/// A neutral note (e.g. "this is you").
fn note(message: &str) -> VNode {
    VElement::new("p")
        .attr("class", "chirp-note")
        .child(text(message.to_string()))
        .into()
}

/// An error banner surfacing an [`AppError`](crate::error::AppError) message to the UI.
fn error_banner(message: &str) -> VNode {
    VElement::new("p")
        .attr("class", "chirp-error")
        .attr("role", "alert")
        .child(text(message.to_string()))
        .into()
}

// ---------------------------------------------------------------------------------------------
// Tests — native, no DOM. Exercise the pure, off-DOM logic (id parsing, toggle target, labels).
// ---------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_user_id_accepts_a_numeric_segment() {
        assert_eq!(parse_user_id("42"), Some(UserId::new(42)));
        assert_eq!(parse_user_id("0"), Some(UserId::new(0)));
    }

    #[test]
    fn parse_user_id_rejects_non_numeric_or_empty_segments() {
        assert_eq!(parse_user_id(""), None);
        assert_eq!(parse_user_id("abc"), None);
        assert_eq!(parse_user_id("-1"), None);
        assert_eq!(parse_user_id("4.2"), None);
        assert_eq!(parse_user_id("user-7"), None);
    }

    #[test]
    fn next_toggle_flips_known_state_and_turns_on_from_unknown() {
        assert!(next_toggle(None)); // unknown -> first action turns it on
        assert!(next_toggle(Some(false)));
        assert!(!next_toggle(Some(true)));
    }

    #[test]
    fn follow_label_reflects_busy_and_following_state() {
        assert_eq!(follow_label(true, Some(true)), "Working…");
        assert_eq!(follow_label(true, None), "Working…");
        assert_eq!(follow_label(false, Some(true)), "Unfollow");
        assert_eq!(follow_label(false, Some(false)), "Follow");
        assert_eq!(follow_label(false, None), "Follow");
    }

    #[test]
    fn like_label_reflects_busy_liked_and_count() {
        assert_eq!(like_label(true, Some(false), Some(3)), "Working… (3)");
        assert_eq!(like_label(false, Some(true), Some(10)), "Unlike (10)");
        assert_eq!(like_label(false, Some(false), Some(0)), "Like (0)");
        assert_eq!(like_label(false, None, None), "Like");
        assert_eq!(like_label(false, Some(true), None), "Unlike");
    }

    #[test]
    fn loading_state_resets_for_a_new_id() {
        let state = ProfileState::loading(UserId::new(7));
        assert_eq!(state.requested, Some(UserId::new(7)));
        assert!(matches!(state.load, Load::Loading));
        assert_eq!(state.following, None);
        assert_eq!(state.follower_count, None);
        assert!(!state.follow_busy);
        assert!(!state.like_busy);
    }
}
