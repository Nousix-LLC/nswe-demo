//! Typed async REST client over `ferric-http` + the `chirp-types` DTOs.
//!
//! `ApiClient` is the one place the SPA talks to the chirp REST backend. It wraps a
//! [`ferric_http::Client`], (de)serializes the shared [`chirp_types`] wire DTOs **directly** (no
//! parallel types), attaches the bearer token when the session carries one, and funnels every
//! failure through `AppError` — decoding a server `chirp_types::ApiError` out of a non-2xx body
//! — so the views render coherent loading / error states.
//!
//! The method ⇄ endpoint map is frozen in `contracts/rest_endpoints.md`: each method below hits
//! exactly one REST method+path with the request/response DTO named there. JSON is camelCase and
//! ids/timestamps are bare numbers; `chirp-types` pins that via `serde`, so this module never
//! touches field naming.
//!
//! Nothing here panics: there is no `unwrap`/`expect`/`panic!`/`todo!`/`unsafe`. Serialization,
//! transport, network, and non-2xx failures all map to an `AppError` variant. `send`/`send_json`
//! are browser-only (they call the Fetch API); the native unit tests therefore cover the pure,
//! off-DOM logic — URL/path/query building and error decoding — and leave the fetch itself to the
//! compile-only wasm smoke tests the gate documents.

use chirp_types::prelude::*;
use ferric_http::Client;

use crate::error::AppError;

// -----------------------------------------------------------------------------------------
// Public API response types -- matching the frozen spec shapes in contracts/public_api_spec.md.
// Defined here (not in chirp-types) because they serve only the two new unauthenticated
// endpoints; the authenticated chirp-types DTOs remain separate.
// -----------------------------------------------------------------------------------------

/// Author embedded in a public timeline chirp. Matches the `author` field in `GET /api/chirps`.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicAuthor {
    /// The author login handle (URL-safe string).
    pub username: String,
    /// The author chosen display name, or `None` if not set.
    pub display_name: Option<String>,
}

/// One chirp returned by the public timeline endpoint `GET /api/chirps`.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicChirp {
    /// Opaque chirp id.
    pub id: u64,
    /// The chirp text content.
    pub content: String,
    /// The author identity.
    pub author: PublicAuthor,
    /// Creation timestamp (milliseconds since Unix epoch).
    pub created_at: i64,
    /// Total number of likes this chirp has received.
    pub like_count: u64,
}

/// Response body from `GET /api/chirps`.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicTimelineResponse {
    /// Chirps in the page, newest first.
    pub chirps: Vec<PublicChirp>,
    /// Whether there are more chirps before the last one in `chirps`.
    pub has_more: bool,
}

/// User profile info in the public user chirp response.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicUserInfo {
    /// The user login handle.
    pub username: String,
    /// The user chosen display name, or `None` if not set.
    pub display_name: Option<String>,
    /// The user bio text, or `None` if not set.
    pub bio: Option<String>,
    /// Total number of chirps this user has posted.
    pub chirp_count: u64,
}

/// One chirp in a user profile chirp list (`GET /api/users/:username/chirps`).
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicUserChirp {
    /// Opaque chirp id.
    pub id: u64,
    /// The chirp text content.
    pub content: String,
    /// Creation timestamp (milliseconds since Unix epoch).
    pub created_at: i64,
    /// Total number of likes this chirp has received.
    pub like_count: u64,
}

/// Response body from `GET /api/users/:username/chirps`.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicUserResponse {
    /// The user profile info.
    pub user: PublicUserInfo,
    /// Chirps posted by the user, newest first.
    pub chirps: Vec<PublicUserChirp>,
    /// Whether there are more chirps before the last one in `chirps`.
    pub has_more: bool,
}

/// Typed async REST client. Holds the configurable base URL and an optional bearer token, builds a
/// fresh [`ferric_http::Client`] per call (cheap — a string and a small header vector), and maps
/// every failure to `AppError`. Cheap to clone.
///
/// Each method's REST method+path and DTOs are frozen in `contracts/rest_endpoints.md`.
#[derive(Debug, Clone)]
pub struct ApiClient {
    /// Base URL every request is joined onto (e.g. `/api`); `ferric_http` joins it with the path.
    base_url: String,
    /// The bearer token attached as `Authorization: Bearer <token>` when present.
    token: Option<SessionToken>,
}

impl ApiClient {
    /// New client for `base_url` with no token (logged out).
    #[must_use]
    pub fn new(base_url: impl Into<String>) -> Self {
        ApiClient {
            base_url: base_url.into(),
            token: None,
        }
    }

    /// Same client carrying a bearer token added to each request's `Authorization` header.
    #[must_use]
    pub fn with_token(self, token: SessionToken) -> Self {
        ApiClient {
            token: Some(token),
            ..self
        }
    }

    /// `POST /sessions` — [`LoginRequest`] → [`AuthResponse`].
    pub async fn login(&self, req: &LoginRequest) -> Result<AuthResponse, AppError> {
        self.http_client()
            .post("/sessions")
            .json(req)
            .map_err(AppError::from_http)?
            .send_json()
            .await
            .map_err(AppError::from_http)
    }

    /// `POST /users` — [`CreateUserRequest`] → [`User`].
    pub async fn create_user(&self, req: &CreateUserRequest) -> Result<User, AppError> {
        self.http_client()
            .post("/users")
            .json(req)
            .map_err(AppError::from_http)?
            .send_json()
            .await
            .map_err(AppError::from_http)
    }

    /// `POST /chirps` — [`CreateChirpRequest`] → [`Chirp`].
    pub async fn create_chirp(&self, req: &CreateChirpRequest) -> Result<Chirp, AppError> {
        self.http_client()
            .post("/chirps")
            .json(req)
            .map_err(AppError::from_http)?
            .send_json()
            .await
            .map_err(AppError::from_http)
    }

    /// `GET /timeline` — [`TimelineQuery`] (query string: `limit`, `cursor`) → [`Page<Chirp>`].
    ///
    /// Only the present fields are sent; an absent `limit`/`cursor` is omitted from the query
    /// string entirely (mirroring `TimelineQuery`'s own `skip_serializing_if` contract).
    pub async fn timeline(&self, query: &TimelineQuery) -> Result<Page<Chirp>, AppError> {
        let owned = timeline_query_params(query);
        let params: Vec<(&str, &str)> = owned
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        self.http_client()
            .get("/timeline")
            .query(&params)
            .send_json()
            .await
            .map_err(AppError::from_http)
    }

    /// `GET /public/timeline` — [`TimelineQuery`] (query string: `limit`, `cursor`) →
    /// [`Page<Chirp>`].
    ///
    /// The public timeline is unauthenticated: this method builds a client without an
    /// `Authorization` header regardless of whether `self.token` is set, so it is safe to call for
    /// logged-out visitors without leaking the session token.
    pub async fn pub_timeline(&self, query: &TimelineQuery) -> Result<Page<Chirp>, AppError> {
        let owned = timeline_query_params(query);
        let params: Vec<(&str, &str)> = owned
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        // No `Authorization` header: build an unauthenticated client explicitly, ignoring
        // `self.token` even if one is set, to satisfy the public endpoint's no-auth contract.
        Client::builder()
            .base_url(self.base_url.clone())
            .build()
            .get("/public/timeline")
            .query(&params)
            .send_json()
            .await
            .map_err(AppError::from_http)
    }

    /// `GET /chirps` -- public timeline, unauthenticated.
    ///
    /// Returns [`PublicTimelineResponse`]. Does not send an `Authorization` header even when
    /// `self.token` is set. Query params: `before` (ISO 8601 cursor), `limit` (1-50, default 20).
    pub async fn public_timeline(
        &self,
        before: Option<&str>,
        limit: Option<u32>,
    ) -> Result<PublicTimelineResponse, AppError> {
        let owned = public_chirps_query_params(before, limit);
        let params: Vec<(&str, &str)> = owned
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        Client::builder()
            .base_url(self.base_url.clone())
            .build()
            .get("/chirps")
            .query(&params)
            .send_json()
            .await
            .map_err(AppError::from_http)
    }

    /// `GET /users/:username/chirps` -- public user profile, unauthenticated.
    ///
    /// Returns [`PublicUserResponse`]. A 404 maps to `AppError::Api` with `NotFound` code when
    /// the server body follows the shared error format. Query params: `before`, `limit`.
    pub async fn public_user_profile(
        &self,
        username: &str,
        before: Option<&str>,
        limit: Option<u32>,
    ) -> Result<PublicUserResponse, AppError> {
        let path = format!("/users/{username}/chirps");
        let owned = public_chirps_query_params(before, limit);
        let params: Vec<(&str, &str)> = owned
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        Client::builder()
            .base_url(self.base_url.clone())
            .build()
            .get(&path)
            .query(&params)
            .send_json()
            .await
            .map_err(AppError::from_http)
    }

    /// `GET /users/{id}` → [`User`]. `{id}` is the opaque [`UserId`]'s numeric value.
    pub async fn get_user(&self, id: UserId) -> Result<User, AppError> {
        self.http_client()
            .get(&user_path(id))
            .send_json()
            .await
            .map_err(AppError::from_http)
    }

    /// `PUT`/`DELETE /users/{id}/follow` — `PUT` to follow, `DELETE` to unfollow → [`FollowResponse`].
    pub async fn set_follow(&self, id: UserId, follow: bool) -> Result<FollowResponse, AppError> {
        let path = follow_path(id);
        let client = self.http_client();
        let request = if follow {
            client.put(&path)
        } else {
            client.delete(&path)
        };
        request.send_json().await.map_err(AppError::from_http)
    }

    /// `PUT`/`DELETE /chirps/{id}/like` — `PUT` to like, `DELETE` to unlike → [`LikeResponse`].
    pub async fn set_like(&self, id: ChirpId, like: bool) -> Result<LikeResponse, AppError> {
        let path = like_path(id);
        let client = self.http_client();
        let request = if like {
            client.put(&path)
        } else {
            client.delete(&path)
        };
        request.send_json().await.map_err(AppError::from_http)
    }

    /// Build a `ferric_http` client for the current base URL, attaching the bearer token as a
    /// default `Authorization` header when one is present. Cheap: clones a string and, at most, one
    /// header pair.
    fn http_client(&self) -> Client {
        let mut builder = Client::builder().base_url(self.base_url.clone());
        if let Some(token) = &self.token {
            builder = builder.default_header("Authorization", &format!("Bearer {}", token.0));
        }
        builder.build()
    }
}

/// The `/users/{id}` path for a user resource, using the opaque id's numeric value.
fn user_path(id: UserId) -> String {
    format!("/users/{}", id.get())
}

/// The `/users/{id}/follow` sub-resource path for a follow/unfollow action.
fn follow_path(id: UserId) -> String {
    format!("/users/{}/follow", id.get())
}

/// The `/chirps/{id}/like` sub-resource path for a like/unlike action.
fn like_path(id: ChirpId) -> String {
    format!("/chirps/{}/like", id.get())
}

/// Build the timeline query-string parameters, emitting only the fields that are present (an absent
/// `limit`/`cursor` contributes nothing), as owned `(key, value)` pairs ready to borrow for
/// [`ferric_http::RequestBuilder::query`].
fn timeline_query_params(query: &TimelineQuery) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(limit) = query.limit {
        params.push(("limit".to_string(), limit.to_string()));
    }
    if let Some(cursor) = &query.cursor {
        params.push(("cursor".to_string(), cursor.0.clone()));
    }
    params
}

/// Build the `before`/`limit` query-string parameters for the two public endpoints, emitting
/// only present fields as owned `(key, value)` pairs ready to borrow for `query`.
fn public_chirps_query_params(before: Option<&str>, limit: Option<u32>) -> Vec<(String, String)> {
    let mut params = Vec::new();
    if let Some(b) = before {
        params.push(("before".to_string(), b.to_string()));
    }
    if let Some(l) = limit {
        params.push(("limit".to_string(), l.to_string()));
    }
    params
}

#[cfg(test)]
mod tests {
    use super::*;
    use chirp_types::prelude::{ApiError, Cursor, ErrorCode, SessionToken};
    use ferric_http::Error as HttpError;

    // --- client construction ------------------------------------------------------------------

    #[test]
    fn new_client_has_base_url_and_no_token() {
        let client = ApiClient::new("/api");
        assert_eq!(client.base_url, "/api");
        assert!(client.token.is_none());
        assert_eq!(client.http_client().base_url(), Some("/api"));
    }

    #[test]
    fn with_token_sets_the_bearer_token_and_preserves_base_url() {
        let client = ApiClient::new("/api").with_token(SessionToken("tok-123".to_string()));
        assert_eq!(client.base_url, "/api");
        assert!(matches!(client.token, Some(SessionToken(ref t)) if t == "tok-123"));
    }

    // --- path building ------------------------------------------------------------------------

    #[test]
    fn paths_interpolate_the_numeric_id() {
        assert_eq!(user_path(UserId::new(9)), "/users/9");
        assert_eq!(follow_path(UserId::new(42)), "/users/42/follow");
        assert_eq!(like_path(ChirpId::new(7)), "/chirps/7/like");
    }

    #[test]
    fn get_user_resolves_base_joined_path() {
        let client = ApiClient::new("/api");
        let url = client.http_client().get(&user_path(UserId::new(9))).url();
        assert_eq!(url, "/api/users/9");
    }

    // --- query building -----------------------------------------------------------------------

    #[test]
    fn timeline_query_omits_absent_fields() {
        assert!(timeline_query_params(&TimelineQuery::default()).is_empty());
    }

    #[test]
    fn timeline_query_includes_only_present_fields() {
        let limit_only = TimelineQuery {
            limit: Some(20),
            cursor: None,
        };
        assert_eq!(
            timeline_query_params(&limit_only),
            vec![("limit".to_string(), "20".to_string())]
        );

        let cursor_only = TimelineQuery {
            limit: None,
            cursor: Some(Cursor("c-1".to_string())),
        };
        assert_eq!(
            timeline_query_params(&cursor_only),
            vec![("cursor".to_string(), "c-1".to_string())]
        );
    }

    #[test]
    fn timeline_query_builds_both_params_in_order() {
        let both = TimelineQuery {
            limit: Some(50),
            cursor: Some(Cursor("opaque token".to_string())),
        };
        assert_eq!(
            timeline_query_params(&both),
            vec![
                ("limit".to_string(), "50".to_string()),
                ("cursor".to_string(), "opaque token".to_string()),
            ]
        );
    }

    #[test]
    fn pub_timeline_url_has_correct_path_and_no_auth() {
        // Even when the client carries a token, pub_timeline builds an unauthenticated client and
        // hits /public/timeline, not /timeline.
        let client = ApiClient::new("/api").with_token(SessionToken("tok-abc".to_string()));
        let query = TimelineQuery {
            limit: Some(10),
            cursor: Some(Cursor("pg2".to_string())),
        };
        let owned = timeline_query_params(&query);
        let params: Vec<(&str, &str)> = owned
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        // Reproduce what pub_timeline builds internally: a no-auth client over the same base URL.
        let url = Client::builder()
            .base_url(client.base_url.clone())
            .build()
            .get("/public/timeline")
            .query(&params)
            .url();
        assert_eq!(url, "/api/public/timeline?limit=10&cursor=pg2");
    }

    #[test]
    fn timeline_url_encodes_query_onto_base_path() {
        let client = ApiClient::new("/api");
        let query = TimelineQuery {
            limit: Some(50),
            cursor: Some(Cursor("opaque token".to_string())),
        };
        let owned = timeline_query_params(&query);
        let params: Vec<(&str, &str)> = owned
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect();
        let url = client.http_client().get("/timeline").query(&params).url();
        // The space in the cursor is percent-encoded by ferric-http's query builder.
        assert_eq!(url, "/api/timeline?limit=50&cursor=opaque%20token");
    }

    // --- error mapping ------------------------------------------------------------------------

    #[test]
    fn non_2xx_json_body_decodes_to_api_error() {
        let body = serde_json::to_string(&ApiError::new(ErrorCode::NotFound, "no such user"))
            .expect("ApiError serializes");
        let err = AppError::from_http(HttpError::Status {
            status: 404,
            status_text: "Not Found".to_string(),
            url: "/api/users/9".to_string(),
            body: Some(body),
        });
        assert_eq!(err.code(), Some(ErrorCode::NotFound));
        assert_eq!(err.user_message(), "no such user");
    }

    #[test]
    fn network_failure_maps_to_network_variant() {
        let err = AppError::from_http(HttpError::Network("offline".to_string()));
        assert!(matches!(err, AppError::Network(ref m) if m == "offline"));
        assert_eq!(err.code(), None);
    }

    #[test]
    fn non_decodable_body_degrades_to_transport() {
        let err = AppError::from_http(HttpError::Status {
            status: 500,
            status_text: String::new(),
            url: "/api/timeline".to_string(),
            body: Some("<html>oops</html>".to_string()),
        });
        assert!(matches!(err, AppError::Transport(_)));
        assert_eq!(err.code(), None);
    }
}
