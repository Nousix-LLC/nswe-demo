//! Request construction: the [`Method`] enum, the fluent [`RequestBuilder`], and the
//! Fetch-API call that turns a built request into a [`Response`].
//!
//! A [`RequestBuilder`] is created by the [`Client`](crate::Client) verb methods
//! ([`get`](crate::Client::get), [`post`](crate::Client::post), …). It carries the
//! client's base URL and default headers and lets the caller layer on query parameters,
//! extra headers, and a JSON (or raw) body before awaiting [`send`](RequestBuilder::send)
//! or [`send_json`](RequestBuilder::send_json).
//!
//! The URL resolution and percent-encoding logic is pure and lives in free functions so it
//! can be unit-tested natively without a browser; only [`send`](RequestBuilder::send)
//! touches `web-sys`.

use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::error::{Error, Result};
use crate::response::Response;

/// An HTTP method supported by the client.
///
/// The client exposes typed verb methods for each of these; [`Method`] is public so the
/// generic [`Client::request`](crate::Client::request) entry point can name one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// HTTP `GET`.
    Get,
    /// HTTP `POST`.
    Post,
    /// HTTP `PUT`.
    Put,
    /// HTTP `DELETE`.
    Delete,
}

impl Method {
    /// The uppercase HTTP token for this method (e.g. `"GET"`), as the Fetch API expects it.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A single HTTP request under construction.
///
/// Build it up with the fluent methods, then `await` [`send`](Self::send) for the raw
/// [`Response`] or [`send_json`](Self::send_json) to go straight to a deserialized value.
///
/// ```no_run
/// # use ferric_http::Client;
/// # use serde::Deserialize;
/// # async fn demo() -> ferric_http::Result<()> {
/// #[derive(Deserialize)]
/// struct User { id: u64, name: String }
///
/// let client = Client::new();
/// let user: User = client
///     .get("https://api.example.com/users/1")
///     .query(&[("expand", "profile")])
///     .header("X-Trace", "abc")
///     .send_json()
///     .await?;
/// # let _ = user;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct RequestBuilder {
    base_url: Option<String>,
    method: Method,
    path: String,
    query: Vec<(String, String)>,
    headers: Vec<(String, String)>,
    body: Option<String>,
}

impl RequestBuilder {
    /// Create a builder from the owning client's configuration. Crate-internal: callers go
    /// through the [`Client`](crate::Client) verb methods.
    pub(crate) fn new(
        base_url: Option<String>,
        default_headers: Vec<(String, String)>,
        method: Method,
        path: &str,
    ) -> Self {
        RequestBuilder {
            base_url,
            method,
            path: path.to_owned(),
            query: Vec::new(),
            headers: default_headers,
            body: None,
        }
    }

    /// Append a request header. May be called multiple times; later values do not replace
    /// earlier ones (the Fetch API combines same-named headers), so call once per header.
    #[must_use]
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_owned(), value.to_owned()));
        self
    }

    /// Append query-string parameters. Keys and values are percent-encoded; calling this
    /// more than once accumulates parameters.
    #[must_use]
    pub fn query(mut self, params: &[(&str, &str)]) -> Self {
        self.query.extend(
            params
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned())),
        );
        self
    }

    /// Serialize `body` to JSON and use it as the request body, defaulting the
    /// `Content-Type` to `application/json` unless one was already set.
    ///
    /// # Errors
    /// Returns [`Error::Serialize`] if `body` cannot be serialized to JSON.
    pub fn json<B: Serialize + ?Sized>(mut self, body: &B) -> Result<Self> {
        let encoded = serde_json::to_string(body).map_err(Error::Serialize)?;
        self.body = Some(encoded);
        if !self.has_header("content-type") {
            self.headers
                .push(("Content-Type".to_owned(), "application/json".to_owned()));
        }
        Ok(self)
    }

    /// Set a raw string request body. The caller is responsible for any `Content-Type`
    /// header; prefer [`json`](Self::json) for JSON payloads.
    #[must_use]
    pub fn body_string(mut self, body: impl Into<String>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// The fully resolved request URL (base joined with path, query appended and encoded).
    ///
    /// Exposed so callers — and tests — can inspect exactly where the request will go
    /// without performing it.
    #[must_use]
    pub fn url(&self) -> String {
        resolve_url(self.base_url.as_deref(), &self.path, &self.query)
    }

    fn has_header(&self, name: &str) -> bool {
        self.headers
            .iter()
            .any(|(n, _)| n.eq_ignore_ascii_case(name))
    }

    /// Perform the request and return the [`Response`] for any completed HTTP exchange.
    ///
    /// A non-`2xx` status is **not** an error here — the [`Response`] is returned so the
    /// caller can inspect it. Use [`Response::error_for_status`] or
    /// [`send_json`](Self::send_json) for status-aware handling.
    ///
    /// # Errors
    /// Returns [`Error::Builder`] if the request cannot be constructed, [`Error::Network`]
    /// if the `fetch` fails before a response arrives, or [`Error::Body`] if the response
    /// body cannot be read.
    ///
    /// Only meaningful in a browser/`wasm32` context: it calls the Fetch API through
    /// `web-sys`, which is not available off-wasm.
    pub async fn send(self) -> Result<Response> {
        let url = self.url();

        let init = web_sys::RequestInit::new();
        init.set_method(self.method.as_str());

        let headers = web_sys::Headers::new()
            .map_err(|e| Error::Builder(format!("could not create headers: {}", js_err(&e))))?;
        for (name, value) in &self.headers {
            headers
                .append(name, value)
                .map_err(|e| Error::Builder(format!("invalid header {name:?}: {}", js_err(&e))))?;
        }
        init.set_headers(headers.as_ref());

        if let Some(body) = &self.body {
            init.set_body(&JsValue::from_str(body));
        }

        let request = web_sys::Request::new_with_str_and_init(&url, &init)
            .map_err(|e| Error::Builder(format!("invalid request for {url}: {}", js_err(&e))))?;

        let window = web_sys::window()
            .ok_or_else(|| Error::Network("no global `window` is available".into()))?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| Error::Network(js_err(&e)))?;
        let web_resp: web_sys::Response = resp_value
            .dyn_into()
            .map_err(|_| Error::Network("fetch did not return a Response".into()))?;

        let status = web_resp.status();
        let status_text = web_resp.status_text();
        let final_url = web_resp.url();
        let body = read_body(&web_resp).await?;
        let headers = collect_headers(&web_resp.headers());

        Ok(Response::from_parts(
            status,
            status_text,
            final_url,
            headers,
            body,
        ))
    }

    /// Perform the request, treat any non-`2xx` status as [`Error::Status`], and
    /// deserialize the response body as JSON into `T`.
    ///
    /// This is the safe-by-default path for a typed JSON API.
    ///
    /// # Errors
    /// Any error [`send`](Self::send) can return, plus [`Error::Status`] for a non-`2xx`
    /// response and [`Error::Deserialize`] if the body is not valid JSON for `T`.
    pub async fn send_json<T: DeserializeOwned>(self) -> Result<T> {
        let response = self.send().await?.error_for_status()?;
        response.json()
    }
}

/// Resolve the final request URL from an optional base, a path, and query parameters.
///
/// If `path` is itself absolute (`http://`/`https://`), it is used as-is and the base is
/// ignored. Otherwise the base and path are joined with exactly one `/` between them. Query
/// parameters are percent-encoded and appended with the correct `?`/`&` separator.
fn resolve_url(base: Option<&str>, path: &str, query: &[(String, String)]) -> String {
    let mut url = if is_absolute(path) {
        path.to_owned()
    } else if let Some(base) = base {
        join_base_and_path(base, path)
    } else {
        path.to_owned()
    };
    append_query(&mut url, query);
    url
}

fn is_absolute(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

fn join_base_and_path(base: &str, path: &str) -> String {
    if path.is_empty() {
        return base.to_owned();
    }
    let base_slash = base.ends_with('/');
    let path_slash = path.starts_with('/');
    match (base_slash, path_slash) {
        (true, true) => format!("{base}{}", &path[1..]),
        (false, false) => format!("{base}/{path}"),
        _ => format!("{base}{path}"),
    }
}

fn append_query(url: &mut String, query: &[(String, String)]) {
    if query.is_empty() {
        return;
    }
    url.push(if url.contains('?') { '&' } else { '?' });
    for (i, (key, value)) in query.iter().enumerate() {
        if i > 0 {
            url.push('&');
        }
        url.push_str(&percent_encode_query(key));
        url.push('=');
        url.push_str(&percent_encode_query(value));
    }
}

/// Percent-encode a query-string component per RFC 3986 (unreserved characters pass
/// through; everything else becomes `%XX` with uppercase hex). Spaces become `%20`.
fn percent_encode_query(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &byte in s.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => {
                out.push('%');
                out.push(to_hex_upper(byte >> 4));
                out.push(to_hex_upper(byte & 0x0f));
            }
        }
    }
    out
}

fn to_hex_upper(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        _ => (b'A' + (nibble - 10)) as char,
    }
}

/// Convert a JS error value into a human-readable string for an [`Error`] message.
fn js_err(value: &JsValue) -> String {
    if let Some(s) = value.as_string() {
        return s;
    }
    if let Ok(message) = js_sys::Reflect::get(value, &JsValue::from_str("message")) {
        if let Some(s) = message.as_string() {
            return s;
        }
    }
    format!("{value:?}")
}

async fn read_body(resp: &web_sys::Response) -> Result<String> {
    let promise = resp.text().map_err(|e| Error::Body(js_err(&e)))?;
    let value = JsFuture::from(promise)
        .await
        .map_err(|e| Error::Body(js_err(&e)))?;
    Ok(value.as_string().unwrap_or_default())
}

fn collect_headers(headers: &web_sys::Headers) -> Vec<(String, String)> {
    let mut out = Vec::new();
    if let Ok(Some(iter)) = js_sys::try_iter(headers.as_ref()) {
        for entry in iter {
            let Ok(entry) = entry else { continue };
            let pair = js_sys::Array::from(&entry);
            let key = pair.get(0).as_string().unwrap_or_default();
            let value = pair.get(1).as_string().unwrap_or_default();
            if !key.is_empty() {
                out.push((key, value));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    #[test]
    fn joins_base_and_path_with_one_slash() {
        assert_eq!(
            resolve_url(Some("https://api.example.com"), "/users", &[]),
            "https://api.example.com/users"
        );
        assert_eq!(
            resolve_url(Some("https://api.example.com/"), "/users", &[]),
            "https://api.example.com/users"
        );
        assert_eq!(
            resolve_url(Some("https://api.example.com"), "users", &[]),
            "https://api.example.com/users"
        );
        assert_eq!(
            resolve_url(Some("https://api.example.com/"), "users", &[]),
            "https://api.example.com/users"
        );
    }

    #[test]
    fn absolute_path_ignores_base() {
        assert_eq!(
            resolve_url(Some("https://api.example.com"), "https://other.test/x", &[]),
            "https://other.test/x"
        );
    }

    #[test]
    fn no_base_uses_path_verbatim() {
        assert_eq!(resolve_url(None, "/relative/path", &[]), "/relative/path");
    }

    #[test]
    fn empty_path_returns_base() {
        assert_eq!(
            resolve_url(Some("https://api.example.com/v1"), "", &[]),
            "https://api.example.com/v1"
        );
    }

    #[test]
    fn appends_query_with_question_mark() {
        assert_eq!(
            resolve_url(Some("https://x.test"), "/s", &q(&[("a", "1"), ("b", "2")])),
            "https://x.test/s?a=1&b=2"
        );
    }

    #[test]
    fn appends_query_with_ampersand_when_url_already_has_query() {
        assert_eq!(
            resolve_url(Some("https://x.test"), "/s?existing=1", &q(&[("a", "2")])),
            "https://x.test/s?existing=1&a=2"
        );
    }

    #[test]
    fn percent_encodes_reserved_characters() {
        assert_eq!(percent_encode_query("a b&c=d/e?f"), "a%20b%26c%3Dd%2Fe%3Ff");
        // Unreserved pass through untouched.
        assert_eq!(percent_encode_query("Aa0-_.~"), "Aa0-_.~");
    }

    #[test]
    fn percent_encodes_utf8_as_bytes() {
        // "é" is U+00E9 → UTF-8 0xC3 0xA9.
        assert_eq!(percent_encode_query("é"), "%C3%A9");
    }

    #[test]
    fn query_values_are_encoded_in_resolved_url() {
        assert_eq!(
            resolve_url(
                Some("https://x.test"),
                "/search",
                &q(&[("q", "a b"), ("tag", "c/d")])
            ),
            "https://x.test/search?q=a%20b&tag=c%2Fd"
        );
    }

    #[test]
    fn builder_accumulates_query_and_headers() {
        let rb = RequestBuilder::new(
            Some("https://api.example.com".into()),
            vec![("Accept".into(), "application/json".into())],
            Method::Get,
            "/users",
        )
        .query(&[("page", "2")])
        .query(&[("limit", "50")])
        .header("X-Trace", "t1");

        assert_eq!(rb.url(), "https://api.example.com/users?page=2&limit=50");
        assert!(rb.has_header("accept"));
        assert!(rb.has_header("x-trace"));
        assert_eq!(rb.method, Method::Get);
    }

    #[test]
    fn json_sets_body_and_default_content_type() {
        use serde::Serialize;
        #[derive(Serialize)]
        struct Payload<'a> {
            name: &'a str,
            active: bool,
        }

        let rb = RequestBuilder::new(None, Vec::new(), Method::Post, "/users")
            .json(&Payload {
                name: "ada",
                active: true,
            })
            .expect("serialization of a plain struct cannot fail");

        assert_eq!(rb.body.as_deref(), Some(r#"{"name":"ada","active":true}"#));
        assert!(rb.has_header("content-type"));
    }

    #[test]
    fn json_does_not_override_explicit_content_type() {
        let rb = RequestBuilder::new(None, Vec::new(), Method::Post, "/x")
            .header("Content-Type", "application/vnd.api+json")
            .json(&serde_json::json!({"a": 1}))
            .unwrap();

        let content_types: Vec<&str> = rb
            .headers
            .iter()
            .filter(|(n, _)| n.eq_ignore_ascii_case("content-type"))
            .map(|(_, v)| v.as_str())
            .collect();
        assert_eq!(content_types, vec!["application/vnd.api+json"]);
    }

    #[test]
    fn method_as_str_matches_http_tokens() {
        assert_eq!(Method::Get.as_str(), "GET");
        assert_eq!(Method::Post.as_str(), "POST");
        assert_eq!(Method::Put.as_str(), "PUT");
        assert_eq!(Method::Delete.as_str(), "DELETE");
        assert_eq!(Method::Delete.to_string(), "DELETE");
    }
}
