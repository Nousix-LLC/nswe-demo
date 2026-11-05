//! Server-side rendering (SSR) integration for `chirp-server`.
//!
//! This module exposes:
//!
//! * [`assemble_document`] — wraps server-rendered markup in a full HTML document shell with
//!   the WASM bootstrap script.  It is a pure function that any handler can call, so handlers
//!   that need to customise the shell (title, additional meta) can compose it at the call site.
//! * [`ssr_handler`] — the top-level `GET /` Axum handler that calls
//!   [`ferric::render_to_string`] with the application root component and returns the assembled
//!   document.
//!
//! # Quick start
//!
//! ```rust,no_run
//! // Build the server-rendered string for any Ferric component:
//! let server_html = ferric::render_to_string(|| {
//!     ferric::VElement::new("h1")
//!         .child(ferric::vdom::text("Hello, Ferric SSR!"))
//!         .into()
//! });
//!
//! // Wrap it in a full HTML document shell:
//! let doc = chirp_server::ssr::assemble_document(&server_html, "/pkg/chirp.js");
//! assert!(doc.contains("<!doctype html>"));
//! assert!(doc.contains(r#"<div id="app">"#));
//! ```
//!
//! # Document shape
//!
//! [`assemble_document`] produces the document shape specified in
//! `design/ferric/ssr-hydration.md §2.1`:
//!
//! ```text
//! <!doctype html>
//! <html lang="en">
//!   <head>…</head>
//!   <body>
//!     <div id="app"><!-- server markup --></div>
//!     <!-- BAR-005 state island placeholder -->
//!     <script type="module" src="/pkg/chirp.js"></script>
//!   </body>
//! </html>
//! ```
//!
//! The `<script type="application/json" id="__state">` state island is deferred to BAR-005
//! (#225); a placeholder comment marks its intended location.
//!
//! # REQ-SSR-011
//!
//! This module satisfies REQ-SSR-011: the first HTTP response for the root path contains
//! server-rendered markup, the integration is a plain library call from an Axum handler,
//! and nothing about `chirp-server`'s existing `build_router` / static-serving structure
//! was changed to accommodate it.

use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

/// Wrap server-rendered markup in a complete HTML document.
///
/// Returns an owned `String` containing the full document, ready to send as an HTTP response
/// body.  The returned document:
///
/// 1. Contains the UTF-8 meta charset declaration so browsers decode the response correctly.
/// 2. Embeds `server_html` inside `<div id="app">` — the hydration mount point the client
///    WASM bundle targets when it calls `ferric::hydrate`.
/// 3. Includes a **placeholder comment** where the BAR-005 state island
///    (`<script type="application/json" id="__state">`) will be inserted once that work item
///    lands.  The comment is structural: downstream spokes MUST anchor their island
///    implementation to this location rather than appending it elsewhere.
/// 4. Loads `wasm_script_path` as an ES module so the WASM bundle initialises and calls
///    `ferric::hydrate` when `#app` already contains server-rendered children (the normal
///    SSR path), or `ferric::dom::mount` when the page has no server markup.
///
/// # Parameters
///
/// * `server_html` — the string returned by [`ferric::render_to_string`].
/// * `wasm_script_path` — URL path to the ES-module entry-point for the WASM bundle (e.g.
///   `"/pkg/chirp.js"`).  This value is inserted verbatim into an HTML `src` attribute; the
///   caller is responsible for ensuring it is a trusted, URL-safe, quote-free string.
///
/// # Panics (debug only)
///
/// Panics in debug builds if `wasm_script_path` contains a double-quote character — this
/// guards against accidental attribute injection.  In release builds the check is elided.
///
/// # Example
///
/// ```rust,no_run
/// let markup = ferric::render_to_string(|| {
///     ferric::VElement::new("h1")
///         .child(ferric::vdom::text("Hello"))
///         .into()
/// });
/// let doc = chirp_server::ssr::assemble_document(&markup, "/pkg/chirp.js");
/// assert!(doc.starts_with("<!doctype html>"));
/// assert!(doc.contains(r#"<div id="app">"#));
/// assert!(doc.contains(r#"src="/pkg/chirp.js""#));
/// ```
pub fn assemble_document(server_html: &str, wasm_script_path: &str) -> String {
    debug_assert!(
        !wasm_script_path.contains('"'),
        "wasm_script_path must not contain '\"': {wasm_script_path:?}"
    );

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>chirp</title>
</head>
<body>
  <div id="app">{server_html}</div>
  <!-- BAR-005 state island: <script type="application/json" id="__state">…</script> -->
  <script type="module" src="{wasm_script_path}"></script>
</body>
</html>
"#
    )
}

/// `GET /` — returns the server-side rendered application document.
///
/// Calls [`ferric::render_to_string`] with the application root component closure, wraps the
/// resulting markup in a full HTML document via [`assemble_document`], and returns it as an
/// HTTP response with status `200 OK` and `Content-Type: text/html; charset=utf-8`.
///
/// # Composability
///
/// The handler is registered with `build_router` as an additive route; no existing route or
/// the SPA fallback is modified.  The explicit `GET /` takes precedence over the
/// `ServeDir` fallback for the root path, so the SSR document replaces the static
/// `index.html` placeholder at `/`.  Every other path continues to be served by the
/// fallback as before.
///
/// # State island (BAR-005)
///
/// Application state serialization for client hydration is tracked as BAR-005 (#225) and is
/// intentionally out of scope here.  The assembled document contains a placeholder comment
/// marking the `<script type="application/json">` island's location; the BAR-005 spoke will
/// replace that comment with real data.  When that work lands, this handler will need access
/// to `AppState` via `State(state): State<AppState>` — the import is left as a commented
/// reminder below.
///
/// # Example — wiring into `build_router`
///
/// ```rust,no_run
/// use axum::routing::get;
/// use axum::Router;
/// use chirp_server::{AppState, ssr::ssr_handler};
///
/// fn extended_router(state: AppState) -> Router {
///     Router::new()
///         .route("/", get(ssr_handler))
///         // … additional routes …
///         .with_state(state)
/// }
/// ```
pub async fn ssr_handler() -> Response {
    // BAR-005 (state island): when that work item lands, add:
    //   State(state): State<AppState>
    // as a parameter here (after re-importing AppState into this module), and pass the
    // serialised state to `assemble_document`.

    // Call `render_to_string` with the application root component closure.
    // The stub implementation in `ferric::ssr` currently panics (via `todo!()`); the
    // `ferric-ssr-r1-server-render` spoke provides the real implementation.  Once that
    // spoke merges onto this branch, this call returns real, hydration-ready markup.
    let server_html = ferric::render_to_string(|| {
        // Placeholder root component: a minimal tree the developer replaces with the
        // real application root once the component tree is wired up.
        ferric::VElement::new("div")
            .attr("id", "ferric-root")
            .child(ferric::vdom::text("Loading\u{2026}"))
            .into()
    });

    let doc = assemble_document(&server_html, "/pkg/chirp.js");

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        doc,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assemble_document_contains_required_structure() {
        let doc = assemble_document("<p>hello</p>", "/pkg/chirp.js");
        assert!(doc.starts_with("<!doctype html>"), "must open with doctype");
        assert!(
            doc.contains(r#"<div id="app"><p>hello</p></div>"#),
            "must embed markup inside mount container"
        );
        assert!(
            doc.contains(r#"src="/pkg/chirp.js""#),
            "must include WASM bootstrap script"
        );
        assert!(
            doc.contains("type=\"module\""),
            "bootstrap must load as ES module"
        );
        assert!(
            doc.contains("BAR-005"),
            "must include the BAR-005 state island placeholder comment"
        );
    }

    #[test]
    fn assemble_document_empty_server_html() {
        // Degenerate case: render_to_string returns an empty string (e.g. empty component).
        let doc = assemble_document("", "/pkg/chirp.js");
        assert!(doc.contains(r#"<div id="app"></div>"#));
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "wasm_script_path must not contain")]
    fn assemble_document_rejects_quoted_script_path() {
        assemble_document("", "/pkg/chirp\".js");
    }
}
