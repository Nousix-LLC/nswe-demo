//! The [`Response`] type: a completed HTTP response as plain, owned data.
//!
//! [`RequestBuilder::send`](crate::RequestBuilder::send) reads the response body eagerly at
//! the JS boundary and hands back a `Response` that holds the status, headers, final URL,
//! and body as a `String`. Because it owns everything, [`json`](Response::json),
//! [`text`](Response::text), and [`error_for_status`](Response::error_for_status) are
//! synchronous and fully testable off-wasm.

use serde::de::DeserializeOwned;

use crate::error::{Error, Result};

/// A completed HTTP response.
///
/// Obtain one from [`RequestBuilder::send`](crate::RequestBuilder::send). Inspect the
/// [`status`](Self::status)/[`headers`](Self::headers), then consume it with
/// [`json`](Self::json) or [`text`](Self::text) — or short-circuit a non-`2xx` status with
/// [`error_for_status`](Self::error_for_status).
#[derive(Debug, Clone)]
pub struct Response {
    status: u16,
    status_text: String,
    url: String,
    headers: Vec<(String, String)>,
    body: String,
}

impl Response {
    /// Assemble a response from its parts. Crate-internal: produced by the Fetch path in
    /// [`RequestBuilder::send`](crate::RequestBuilder::send).
    pub(crate) fn from_parts(
        status: u16,
        status_text: String,
        url: String,
        headers: Vec<(String, String)>,
        body: String,
    ) -> Self {
        Response {
            status,
            status_text,
            url,
            headers,
            body,
        }
    }

    /// The HTTP status code (e.g. `200`, `404`).
    #[must_use]
    pub fn status(&self) -> u16 {
        self.status
    }

    /// The HTTP status text (e.g. `"OK"`), when the browser provides it.
    #[must_use]
    pub fn status_text(&self) -> &str {
        &self.status_text
    }

    /// The final URL the response came from (after any redirects).
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// `true` if the status is in the `2xx` success range.
    #[must_use]
    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// All response headers, in the order the browser reported them.
    #[must_use]
    pub fn headers(&self) -> &[(String, String)] {
        &self.headers
    }

    /// Look up a header by name, case-insensitively.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    /// The raw response body as a string slice.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Consume the response and return its body as an owned `String`.
    #[must_use]
    pub fn text(self) -> String {
        self.body
    }

    /// Deserialize the response body as JSON into `T`.
    ///
    /// # Errors
    /// Returns [`Error::Deserialize`] if the body is not valid JSON for `T`.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        serde_json::from_str(&self.body).map_err(Error::Deserialize)
    }

    /// Return `self` if the status is `2xx`, otherwise [`Error::Status`] carrying the
    /// status, status text, URL, and body.
    ///
    /// # Errors
    /// Returns [`Error::Status`] for any non-`2xx` status.
    pub fn error_for_status(self) -> Result<Self> {
        if self.ok() {
            Ok(self)
        } else {
            Err(Error::Status {
                status: self.status,
                status_text: self.status_text,
                url: self.url,
                body: Some(self.body),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    fn resp(status: u16, body: &str) -> Response {
        Response::from_parts(
            status,
            "".into(),
            "https://api.example.com/x".into(),
            vec![("Content-Type".into(), "application/json".into())],
            body.into(),
        )
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct User {
        id: u64,
        name: String,
    }

    #[test]
    fn ok_reflects_2xx_range() {
        assert!(resp(200, "").ok());
        assert!(resp(204, "").ok());
        assert!(resp(299, "").ok());
        assert!(!resp(199, "").ok());
        assert!(!resp(300, "").ok());
        assert!(!resp(404, "").ok());
        assert!(!resp(500, "").ok());
    }

    #[test]
    fn header_lookup_is_case_insensitive() {
        let r = resp(200, "");
        assert_eq!(r.header("content-type"), Some("application/json"));
        assert_eq!(r.header("CONTENT-TYPE"), Some("application/json"));
        assert_eq!(r.header("x-missing"), None);
    }

    #[test]
    fn json_round_trips_a_struct() {
        let r = resp(200, r#"{"id":7,"name":"ada"}"#);
        let user: User = r.json().unwrap();
        assert_eq!(
            user,
            User {
                id: 7,
                name: "ada".into()
            }
        );
    }

    #[test]
    fn json_on_malformed_body_is_deserialize_error() {
        let r = resp(200, "this is not json");
        let err = r.json::<User>().unwrap_err();
        assert!(matches!(err, Error::Deserialize(_)));
    }

    #[test]
    fn error_for_status_passes_through_2xx() {
        let r = resp(201, r#"{"id":1,"name":"x"}"#);
        let r = r.error_for_status().expect("201 is a success");
        assert_eq!(r.status(), 201);
    }

    #[test]
    fn error_for_status_maps_non_2xx_to_status_error() {
        let r = resp(404, r#"{"error":"missing"}"#);
        match r.error_for_status() {
            Err(Error::Status {
                status, body, url, ..
            }) => {
                assert_eq!(status, 404);
                assert_eq!(body.as_deref(), Some(r#"{"error":"missing"}"#));
                assert_eq!(url, "https://api.example.com/x");
            }
            other => panic!("expected Status error, got {other:?}"),
        }
    }

    #[test]
    fn text_consumes_and_returns_body() {
        let r = resp(200, "hello");
        assert_eq!(r.text(), "hello");
    }
}
