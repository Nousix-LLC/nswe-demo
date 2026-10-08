//! `/compose` — handle-based login and the validated compose-chirp form.
//!
//! This is the real view behind the frozen [`compose_view`](crate::compose::compose_view)
//! signature (owner: `SUBTASK_compose`). It renders two states off the one reactive
//! [`AppContext::session`](crate::app::AppContext) signal:
//!
//! * **Logged out** — a handle field and a *Log in* button. On submit the handle is validated
//!   with `Username::parse`, then [`ApiClient::login`](crate::api::ApiClient::login) is called;
//!   on success the returned `Session` is written into `ctx.session` (which re-renders every
//!   dependent view), and a server [`AppError`](crate::error::AppError) (e.g. `Unauthorized`) is
//!   surfaced inline.
//! * **Logged in** — a compose field gated on the authed session. On submit the text is validated
//!   with `ChirpText::parse` (a `ValidationError` is shown inline *without* a round-trip), then
//!   [`ApiClient::create_chirp`](crate::api::ApiClient::create_chirp) is called, with reactive
//!   busy/disabled, success, and error feedback.
//!
//! # Reactivity model (why UI state is plain, not a `Signal`)
//!
//! `ferric` renders the whole app from one root effect ([`ferric::dom::mount`]), and it offers a
//! route view no per-component *setup* phase: a [`Signal`](ferric::prelude::Signal) created while
//! the render runs is owned by that effect and disposed the next time it re-runs (see the
//! `counter` example, which creates its state *above* `mount`). A view therefore cannot hold a
//! long-lived local signal of its own. This view keeps its transient UI state (busy / error /
//! success / input drafts) in plain thread-local `ComposeState` and uses the long-lived
//! `session` signal — created once in `AppContext`, outside any effect — as its *render anchor*:
//! `request_rerender` re-notifies it to re-run the render, which re-reads the plain state. The
//! session value is read (to branch logged-in/out) and only ever *changed* by an actual login.
//!
//! # Reading inputs
//!
//! `ferric` event handlers are zero-argument (`Rc<dyn Fn()>`), so a handler cannot pull a value
//! off the event. Instead each field carries a stable `id` and the handler reads its current value
//! straight from the DOM element by that id (`read_field_value`). Inputs are *uncontrolled* (the
//! view never sets their `value` attribute), so a re-render never clobbers what the user is typing.

use std::cell::RefCell;

use ferric::prelude::*;
use ferric::vdom::{text, VElement, VNode};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use chirp_types::prelude::{
    ChirpText, CreateChirpRequest, LoginRequest, Username, ValidationError,
};

use crate::app::{AppContext, Session};

/// DOM `id` of the login handle `<input>`.
const LOGIN_FIELD_ID: &str = "chirp-login-handle";
/// DOM `id` of the compose chirp `<textarea>`.
const COMPOSE_FIELD_ID: &str = "chirp-compose-text";

// =====================================================================================
// Transient UI state (plain — see the module-level "Reactivity model" note)
// =====================================================================================

/// The view's transient UI state. Plain (non-reactive) data held in a thread-local; the render
/// reads a snapshot of it each pass and the `session` signal is the reactive anchor that re-runs
/// the render (see `request_rerender`).
#[derive(Clone, Default)]
struct ComposeState {
    /// The login request is in flight.
    login_busy: bool,
    /// The last login failure, shown inline until the next attempt.
    login_error: Option<String>,
    /// Latest known value of the handle field, mirrored for reactive enable/disable.
    login_handle_draft: String,
    /// The create-chirp request is in flight.
    compose_busy: bool,
    /// The last compose failure (validation or API), shown inline until the next attempt.
    compose_error: Option<String>,
    /// A transient success confirmation after a chirp posts.
    compose_success: Option<String>,
    /// Latest known value of the compose field, mirrored for reactive enable/disable.
    compose_text_draft: String,
}

thread_local! {
    /// The single per-thread instance of the compose view's transient UI state.
    static STATE: RefCell<ComposeState> = RefCell::new(ComposeState::default());
}

/// Take a cloned snapshot of the current UI state (borrow released before returning).
fn snapshot() -> ComposeState {
    STATE.with(|cell| cell.borrow().clone())
}

/// Mutate the UI state in place (borrow released before returning).
fn mutate(f: impl FnOnce(&mut ComposeState)) {
    STATE.with(|cell| f(&mut cell.borrow_mut()));
}

/// Re-render the active view to reflect a change in the plain `ComposeState`.
///
/// See the module-level "Reactivity model" note: the long-lived `session` signal is this view's
/// render anchor. Re-notifying it (without changing its value) re-runs `ferric`'s root render
/// effect, which re-reads the UI snapshot. This is only ever called from an event handler or an
/// async completion — i.e. outside the render effect — so it drives a flush immediately.
fn request_rerender(ctx: &AppContext) {
    ctx.session.update(|_| {});
}

// =====================================================================================
// Pure logic (natively unit-tested, no DOM)
// =====================================================================================

/// Validate a raw handle string into a [`LoginRequest`]. Surrounding whitespace is trimmed (a
/// handle never contains spaces); emptiness and illegal characters surface as a `ValidationError`.
fn validate_handle(raw: &str) -> Result<LoginRequest, ValidationError> {
    Username::parse(raw.trim()).map(|username| LoginRequest { username })
}

/// Validate a raw chirp body into a [`CreateChirpRequest`] via `ChirpText::parse` (empty / too
/// long surface as a `ValidationError`). The body is not trimmed: whitespace is content.
fn validate_chirp(raw: &str) -> Result<CreateChirpRequest, ValidationError> {
    ChirpText::parse(raw).map(|text| CreateChirpRequest {
        text,
        reply_to: None,
    })
}

/// Whether the *Log in* control is actionable: not busy and the handle is non-blank.
fn login_submit_enabled(handle_draft: &str, busy: bool) -> bool {
    !busy && !handle_draft.trim().is_empty()
}

/// Whether the *Chirp* control is actionable: not busy and the body is non-empty (matching
/// [`ChirpText`]'s own emptiness rule, which counts characters and does not trim).
fn compose_submit_enabled(text_draft: &str, busy: bool) -> bool {
    !busy && !text_draft.is_empty()
}

/// A user-facing message for a construction-time `ValidationError`.
fn validation_message(err: &ValidationError) -> String {
    err.to_string()
}

// =====================================================================================
// DOM field access (browser-only; only ever invoked from event handlers)
// =====================================================================================

/// Read the current value of the `<input>`/`<textarea>` with the given `id` from the live DOM,
/// or `""` when it cannot be found (no panic; off-DOM this simply yields an empty string).
fn read_field_value(id: &str) -> String {
    let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(id))
    else {
        return String::new();
    };
    if let Some(input) = element.dyn_ref::<web_sys::HtmlInputElement>() {
        return input.value();
    }
    if let Some(area) = element.dyn_ref::<web_sys::HtmlTextAreaElement>() {
        return area.value();
    }
    String::new()
}

/// Clear the value of the `<input>`/`<textarea>` with the given `id` in the live DOM, if present.
fn clear_field(id: &str) {
    let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(id))
    else {
        return;
    };
    if let Some(input) = element.dyn_ref::<web_sys::HtmlInputElement>() {
        input.set_value("");
    } else if let Some(area) = element.dyn_ref::<web_sys::HtmlTextAreaElement>() {
        area.set_value("");
    }
}

// =====================================================================================
// View fragments
// =====================================================================================

/// A styled, role-annotated inline banner (error or success), or nothing when `message` is `None`.
fn banner(class: &str, role: &str, message: Option<String>) -> Option<VNode> {
    message.map(|message| {
        VElement::new("p")
            .attr("class", class)
            .attr("role", role)
            .child(text(message))
            .into()
    })
}

/// The logged-out login form.
fn render_login(ctx: &AppContext, state: &ComposeState) -> VNode {
    let busy = state.login_busy;
    let enabled = login_submit_enabled(&state.login_handle_draft, busy);

    let input_ctx = ctx.clone();
    let on_input = move || {
        let value = read_field_value(LOGIN_FIELD_ID);
        mutate(|s| {
            s.login_handle_draft = value;
            s.login_error = None;
        });
        request_rerender(&input_ctx);
    };

    let click_ctx = ctx.clone();
    let on_submit = move || submit_login(&click_ctx);

    let mut button = VElement::new("button")
        .attr("type", "button")
        .attr("class", "chirp-submit chirp-login-submit")
        .on(EventKind::Click, on_submit)
        .child(text(if busy { "Logging in…" } else { "Log in" }));
    if !enabled {
        button = button.attr("disabled", "disabled");
    }

    VElement::new("div")
        .attr("class", "chirp-login")
        .child(VElement::new("h1").child(text("Log in to chirp")))
        .child(
            VElement::new("p")
                .attr("class", "chirp-hint")
                .child(text("Enter your handle to start chirping.")),
        )
        .child(
            VElement::new("label")
                .attr("for", LOGIN_FIELD_ID)
                .child(text("Handle")),
        )
        .child(
            VElement::new("input")
                .attr("id", LOGIN_FIELD_ID)
                .attr("name", "handle")
                .attr("type", "text")
                .attr("autocomplete", "username")
                .attr("placeholder", "yourhandle")
                .on(EventKind::Input, on_input),
        )
        .child(button)
        .children(banner("chirp-error", "alert", state.login_error.clone()))
        .into()
}

/// The logged-in compose form, gated on an authed session.
fn render_compose(ctx: &AppContext, state: &ComposeState, username: &str) -> VNode {
    let busy = state.compose_busy;
    let enabled = compose_submit_enabled(&state.compose_text_draft, busy);

    let input_ctx = ctx.clone();
    let on_input = move || {
        let value = read_field_value(COMPOSE_FIELD_ID);
        mutate(|s| {
            s.compose_text_draft = value;
            s.compose_error = None;
            s.compose_success = None;
        });
        request_rerender(&input_ctx);
    };

    let click_ctx = ctx.clone();
    let on_submit = move || submit_chirp(&click_ctx);

    let mut button = VElement::new("button")
        .attr("type", "button")
        .attr("class", "chirp-submit chirp-compose-submit")
        .on(EventKind::Click, on_submit)
        .child(text(if busy { "Chirping…" } else { "Chirp" }));
    if !enabled {
        button = button.attr("disabled", "disabled");
    }

    VElement::new("div")
        .attr("class", "chirp-compose-form")
        .child(VElement::new("h1").child(text("Compose")))
        .child(
            VElement::new("p")
                .attr("class", "chirp-whoami")
                .child(text(format!("Signed in as @{username}"))),
        )
        .child(
            VElement::new("label")
                .attr("for", COMPOSE_FIELD_ID)
                .child(text("What's happening?")),
        )
        .child(
            VElement::new("textarea")
                .attr("id", COMPOSE_FIELD_ID)
                .attr("name", "chirp")
                .attr("rows", "3")
                .attr("maxlength", "280")
                .attr("placeholder", "Say something chirpy…")
                .on(EventKind::Input, on_input),
        )
        .child(button)
        .children(banner("chirp-error", "alert", state.compose_error.clone()))
        .children(banner(
            "chirp-success",
            "status",
            state.compose_success.clone(),
        ))
        .into()
}

// =====================================================================================
// Submit flows (event-handler side: validate, then drive the async call via spawn_local)
// =====================================================================================

/// Validate the handle and, if valid, log in; on success write the `Session` into `ctx.session`.
fn submit_login(ctx: &AppContext) {
    let raw = read_field_value(LOGIN_FIELD_ID);
    let request = match validate_handle(&raw) {
        Ok(request) => request,
        Err(err) => {
            mutate(|s| s.login_error = Some(validation_message(&err)));
            request_rerender(ctx);
            return;
        }
    };

    mutate(|s| {
        s.login_busy = true;
        s.login_error = None;
    });
    request_rerender(ctx);

    let ctx = ctx.clone();
    spawn_local(async move {
        match ctx.client().login(&request).await {
            Ok(auth) => {
                mutate(|s| {
                    s.login_busy = false;
                    s.login_error = None;
                    s.login_handle_draft.clear();
                });
                // Writing the session is itself the reactive signal: it re-renders this view
                // (now logged in) and every other view that reads the session.
                ctx.session.set(Some(Session {
                    user: auth.user,
                    token: auth.token,
                }));
            }
            Err(err) => {
                mutate(|s| {
                    s.login_busy = false;
                    s.login_error = Some(err.user_message());
                });
                request_rerender(&ctx);
            }
        }
    });
}

/// Validate the chirp body and, if valid, post it; reset the field and confirm on success.
fn submit_chirp(ctx: &AppContext) {
    let raw = read_field_value(COMPOSE_FIELD_ID);
    let request = match validate_chirp(&raw) {
        Ok(request) => request,
        Err(err) => {
            mutate(|s| {
                s.compose_error = Some(validation_message(&err));
                s.compose_success = None;
            });
            request_rerender(ctx);
            return;
        }
    };

    mutate(|s| {
        s.compose_busy = true;
        s.compose_error = None;
        s.compose_success = None;
    });
    request_rerender(ctx);

    let ctx = ctx.clone();
    spawn_local(async move {
        match ctx.client().create_chirp(&request).await {
            Ok(chirp) => {
                clear_field(COMPOSE_FIELD_ID);
                mutate(|s| {
                    s.compose_busy = false;
                    s.compose_error = None;
                    s.compose_success = Some(format!("Chirped! ({})", chirp.id));
                    s.compose_text_draft.clear();
                });
                request_rerender(&ctx);
            }
            Err(err) => {
                mutate(|s| {
                    s.compose_busy = false;
                    s.compose_error = Some(err.user_message());
                });
                request_rerender(&ctx);
            }
        }
    });
}

// =====================================================================================
// Frozen entry point
// =====================================================================================

/// `/compose` — login + compose. Renders the login form when logged out and the compose form when
/// a `Session` is present, reading the reactive `session` signal so a login/logout re-renders it.
///
/// Realizes the frozen `contracts/app_api.rs` signature; bound to `/compose` by the scaffold's
/// route table.
#[must_use]
pub fn compose_view(ctx: &AppContext) -> VNode {
    // Reactive read: subscribes the render effect to the session, the view's render anchor.
    let signed_in_as = ctx.session.with(|session| {
        session
            .as_ref()
            .map(|s| s.user.username.as_str().to_string())
    });
    let state = snapshot();

    let body = match signed_in_as {
        Some(username) => render_compose(ctx, &state, &username),
        None => render_login(ctx, &state),
    };

    VElement::new("section")
        .attr("class", "chirp-view chirp-compose")
        .child(body)
        .into()
}

// =====================================================================================
// Native tests — the pure validation/enablement logic, no DOM, no reactive runtime.
// =====================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_handle_parses_and_trims() {
        let request = validate_handle("  alice_01  ").expect("a trimmed, valid handle parses");
        assert_eq!(request.username.as_str(), "alice_01");
    }

    #[test]
    fn blank_handle_is_rejected() {
        assert!(matches!(
            validate_handle("   "),
            Err(ValidationError::Empty { field: "username" })
        ));
    }

    #[test]
    fn handle_with_illegal_character_is_rejected() {
        assert!(matches!(
            validate_handle("has space"),
            Err(ValidationError::InvalidCharacter {
                field: "username",
                ..
            })
        ));
    }

    #[test]
    fn valid_chirp_parses_as_a_top_level_post() {
        let request = validate_chirp("hello, chirp!").expect("a non-empty body parses");
        assert_eq!(request.text.as_str(), "hello, chirp!");
        assert_eq!(request.reply_to, None);
    }

    #[test]
    fn empty_chirp_is_rejected() {
        assert!(matches!(
            validate_chirp(""),
            Err(ValidationError::Empty { field: "text" })
        ));
    }

    #[test]
    fn over_length_chirp_is_rejected() {
        let too_long = "x".repeat(281);
        assert!(matches!(
            validate_chirp(&too_long),
            Err(ValidationError::TooLong { field: "text", .. })
        ));
    }

    #[test]
    fn login_enablement_requires_nonblank_handle_and_not_busy() {
        assert!(login_submit_enabled("alice", false));
        assert!(!login_submit_enabled("alice", true), "disabled while busy");
        assert!(!login_submit_enabled("   ", false), "disabled when blank");
        assert!(!login_submit_enabled("", false), "disabled when empty");
    }

    #[test]
    fn compose_enablement_requires_nonempty_body_and_not_busy() {
        assert!(compose_submit_enabled("hi", false));
        assert!(!compose_submit_enabled("hi", true), "disabled while busy");
        assert!(!compose_submit_enabled("", false), "disabled when empty");
    }

    #[test]
    fn validation_message_is_the_display_form() {
        let err = ValidationError::Empty { field: "text" };
        assert_eq!(validation_message(&err), "text must not be empty");
    }
}
