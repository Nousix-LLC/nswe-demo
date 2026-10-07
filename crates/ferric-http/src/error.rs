//! The error type for `ferric-http` and its companion [`Result`] alias.
//!
//! Every fallible operation in this crate returns [`Result<T>`], whose error is the
//! [`Error`] enum. The enum deliberately distinguishes the failure modes a caller is
//! likely to branch on:
//!
//! * [`Error::Builder`] — the request could not even be constructed (bad URL, bad header).
//! * [`Error::Network`] — the `fetch` call itself failed (offline, DNS, CORS, a rejected
//!   promise). No HTTP response was received.
//! * [`Error::Status`] — a response *was* received, but its status was not `2xx`.
//! * [`Error::Serialize`] — a request body could not be serialized to JSON.
//! * [`Error::Deserialize`] — a response body could not be deserialized from JSON.
//! * [`Error::Body`] — the response body could not be read at the JS boundary.
//!
//! The type is hand-implemented (no `thiserror`) to keep the dependency set lean, matching
//! the rest of the `ferric` workspace. `serde_json::Error` is preserved as the
//! [`source`](std::error::Error::source) of the (de)serialization variants so the full
//! causal chain is available to callers.

use std::fmt;

/// A specialized [`std::result::Result`] whose error is this crate's [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// The error type returned by every fallible `ferric-http` operation.
///
/// Match on the variants to react to a specific failure mode; [`Display`](fmt::Display)
/// renders a human-readable, single-line message for logging.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The request could not be built — for example an invalid base URL, or a header
    /// name/value the browser rejected. Carries a human-readable description.
    Builder(String),

    /// The `fetch` call failed before any HTTP response was received: the device is
    /// offline, DNS failed, the request was blocked by CORS, or the underlying promise
    /// rejected. Carries the JS-side description of the failure.
    Network(String),

    /// The server returned a response whose status was outside the `2xx` range.
    ///
    /// The body (when it could be read) is included to aid debugging; a JSON API will
    /// usually place a machine-readable error document there.
    Status {
        /// The HTTP status code (e.g. `404`, `500`).
        status: u16,
        /// The HTTP status text (e.g. `"Not Found"`), when the browser provides it.
        status_text: String,
        /// The final URL the response came from (after any redirects).
        url: String,
        /// The response body, if it could be read.
        body: Option<String>,
    },

    /// A request body could not be serialized to JSON.
    Serialize(serde_json::Error),

    /// A response body could not be deserialized from JSON into the requested type.
    Deserialize(serde_json::Error),

    /// The response body could not be read at the JavaScript boundary (the `text()`
    /// promise rejected, or returned a non-string value).
    Body(String),
}

impl Error {
    /// Returns `true` if this is a [network](Error::Network) failure (no response received).
    #[must_use]
    pub fn is_network(&self) -> bool {
        matches!(self, Error::Network(_))
    }

    /// Returns `true` if this is a non-`2xx` [status](Error::Status) failure.
    #[must_use]
    pub fn is_status(&self) -> bool {
        matches!(self, Error::Status { .. })
    }

    /// The HTTP status code, if this error is a [`Error::Status`].
    ///
    /// Returns `None` for every other variant, so callers can write
    /// `if err.status() == Some(404) { … }` without a full `match`.
    #[must_use]
    pub fn status(&self) -> Option<u16> {
        match self {
            Error::Status { status, .. } => Some(*status),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Builder(msg) => write!(f, "failed to build request: {msg}"),
            Error::Network(msg) => write!(f, "network error: {msg}"),
            Error::Status {
                status,
                status_text,
                url,
                ..
            } => {
                if status_text.is_empty() {
                    write!(f, "HTTP status {status} for {url}")
                } else {
                    write!(f, "HTTP status {status} {status_text} for {url}")
                }
            }
            Error::Serialize(e) => write!(f, "failed to serialize request body: {e}"),
            Error::Deserialize(e) => write!(f, "failed to deserialize response body: {e}"),
            Error::Body(msg) => write!(f, "failed to read response body: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Serialize(e) | Error::Deserialize(e) => Some(e),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_is_single_line_and_descriptive() {
        let e = Error::Network("offline".into());
        assert_eq!(e.to_string(), "network error: offline");

        let e = Error::Builder("empty header name".into());
        assert_eq!(e.to_string(), "failed to build request: empty header name");
    }

    #[test]
    fn status_display_includes_code_and_url() {
        let e = Error::Status {
            status: 404,
            status_text: "Not Found".into(),
            url: "https://api.example.com/users/9".into(),
            body: Some("{\"error\":\"missing\"}".into()),
        };
        assert_eq!(
            e.to_string(),
            "HTTP status 404 Not Found for https://api.example.com/users/9"
        );
    }

    #[test]
    fn status_display_omits_empty_status_text() {
        let e = Error::Status {
            status: 500,
            status_text: String::new(),
            url: "https://api.example.com/x".into(),
            body: None,
        };
        assert_eq!(
            e.to_string(),
            "HTTP status 500 for https://api.example.com/x"
        );
    }

    #[test]
    fn classifiers_and_status_accessor() {
        let net = Error::Network("x".into());
        assert!(net.is_network());
        assert!(!net.is_status());
        assert_eq!(net.status(), None);

        let st = Error::Status {
            status: 503,
            status_text: "Service Unavailable".into(),
            url: "u".into(),
            body: None,
        };
        assert!(st.is_status());
        assert!(!st.is_network());
        assert_eq!(st.status(), Some(503));
    }

    #[test]
    fn deserialize_variant_exposes_serde_source() {
        use std::error::Error as _;
        let serde_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let e = Error::Deserialize(serde_err);
        assert!(e.source().is_some(), "serde_json error must be the source");
        assert!(e
            .to_string()
            .starts_with("failed to deserialize response body"));
    }
}
