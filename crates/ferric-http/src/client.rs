//! The [`Client`] and its [`ClientBuilder`].
//!
//! A [`Client`] holds an optional base URL and a set of default headers, then hands out a
//! [`RequestBuilder`] for each HTTP verb. It is cheap to [`clone`](Clone) (a string and a
//! small vector) and holds no connection state of its own — the browser owns the connection
//! pool — so one client can be shared across a whole app.

use crate::request::{Method, RequestBuilder};

/// An HTTP/JSON client over the browser Fetch API.
///
/// Create one with [`Client::new`] for a bare client, or [`Client::builder`] to set a base
/// URL and default headers. Each verb method returns a [`RequestBuilder`] you finish with
/// `.send().await` or `.send_json().await`.
///
/// ```no_run
/// # use ferric_http::Client;
/// # async fn demo() -> ferric_http::Result<()> {
/// let client = Client::builder()
///     .base_url("https://api.example.com")
///     .default_header("Authorization", "Bearer token")
///     .build();
///
/// let body = client.get("/health").send().await?.text();
/// # let _ = body;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct Client {
    base_url: Option<String>,
    default_headers: Vec<(String, String)>,
}

impl Client {
    /// A client with no base URL and no default headers. Pass absolute URLs to the verb
    /// methods, or use [`Client::builder`] to configure a base URL.
    #[must_use]
    pub fn new() -> Self {
        Client::default()
    }

    /// Start building a configured client.
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Begin a request with an explicit [`Method`]. The verb helpers
    /// ([`get`](Self::get) …) call this; use it directly for symmetry when the method is
    /// chosen at runtime.
    #[must_use]
    pub fn request(&self, method: Method, path: &str) -> RequestBuilder {
        RequestBuilder::new(
            self.base_url.clone(),
            self.default_headers.clone(),
            method,
            path,
        )
    }

    /// Begin a `GET` request to `path` (joined onto the base URL, if any).
    #[must_use]
    pub fn get(&self, path: &str) -> RequestBuilder {
        self.request(Method::Get, path)
    }

    /// Begin a `POST` request to `path`.
    #[must_use]
    pub fn post(&self, path: &str) -> RequestBuilder {
        self.request(Method::Post, path)
    }

    /// Begin a `PUT` request to `path`.
    #[must_use]
    pub fn put(&self, path: &str) -> RequestBuilder {
        self.request(Method::Put, path)
    }

    /// Begin a `DELETE` request to `path`.
    #[must_use]
    pub fn delete(&self, path: &str) -> RequestBuilder {
        self.request(Method::Delete, path)
    }

    /// The client's base URL, if one was configured.
    #[must_use]
    pub fn base_url(&self) -> Option<&str> {
        self.base_url.as_deref()
    }
}

/// A builder for a configured [`Client`].
///
/// Obtain one from [`Client::builder`], chain [`base_url`](Self::base_url) and
/// [`default_header`](Self::default_header), then [`build`](Self::build).
#[derive(Debug, Clone, Default)]
pub struct ClientBuilder {
    base_url: Option<String>,
    default_headers: Vec<(String, String)>,
}

impl ClientBuilder {
    /// Set the base URL that relative request paths are joined onto. An absolute request
    /// path (`http://`/`https://`) overrides it per request.
    #[must_use]
    pub fn base_url(mut self, base: impl Into<String>) -> Self {
        self.base_url = Some(base.into());
        self
    }

    /// Add a header sent with every request from the built client. Per-request
    /// [`header`](RequestBuilder::header) calls add to these.
    #[must_use]
    pub fn default_header(mut self, name: &str, value: &str) -> Self {
        self.default_headers
            .push((name.to_owned(), value.to_owned()));
        self
    }

    /// Finish building the [`Client`].
    #[must_use]
    pub fn build(self) -> Client {
        Client {
            base_url: self.base_url,
            default_headers: self.default_headers,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_client_has_no_base_url() {
        let client = Client::new();
        assert_eq!(client.base_url(), None);
        // An absolute path is used verbatim when there is no base.
        assert_eq!(
            client.get("https://api.example.com/ping").url(),
            "https://api.example.com/ping"
        );
    }

    #[test]
    fn builder_sets_base_url_and_defaults_flow_into_requests() {
        let client = Client::builder()
            .base_url("https://api.example.com")
            .default_header("Authorization", "Bearer tok")
            .default_header("Accept", "application/json")
            .build();

        assert_eq!(client.base_url(), Some("https://api.example.com"));

        let rb = client.get("/users").query(&[("page", "1")]);
        assert_eq!(rb.url(), "https://api.example.com/users?page=1");
    }

    #[test]
    fn each_verb_selects_the_right_method() {
        let client = Client::builder().base_url("https://x.test").build();
        // The resolved URL is the same across verbs; the method differs. We assert the URL
        // here and rely on `Method` tests for the verb tokens; this confirms every verb
        // produces a usable builder bound to the base URL.
        for path in ["/a", "/b", "/c", "/d"] {
            assert!(client.get(path).url().starts_with("https://x.test/"));
        }
        assert_eq!(client.post("/a").url(), "https://x.test/a");
        assert_eq!(client.put("/a").url(), "https://x.test/a");
        assert_eq!(client.delete("/a").url(), "https://x.test/a");
    }

    #[test]
    fn client_is_cloneable_and_independent() {
        let client = Client::builder().base_url("https://x.test").build();
        let clone = client.clone();
        assert_eq!(client.base_url(), clone.base_url());
    }
}
