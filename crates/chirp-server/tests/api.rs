//! End-to-end API integration tests for `chirp-server`.
//!
//! These tests exercise the **assembled** server exactly as a client would: they mount the real
//! application via the crate's single composition root [`chirp_server::build_router`] against a
//! fresh in-memory repository and drive it in-process with `tower`'s
//! [`ServiceExt::oneshot`](tower::ServiceExt::oneshot) — **no database, no network bind**. Each
//! request round-trips real JSON: requests are built from the shared `chirp-types` DTOs and
//! responses are decoded back into them, so every test asserts BOTH the HTTP status line AND the
//! decoded wire type, proving the `chirp-types` contract is honored, not merely the status code.
//!
//! The suite is one integration-test crate (one compilation unit) organized into submodules by
//! concern: [`happy_path`] (the acceptance surface end to end), [`errors`] (the contract's
//! `error_mapping` table), and [`pagination`] (cursor + limit behavior). The harness lives at the
//! crate root and is shared by every submodule.
//!
//! Isolation: every test constructs its own [`TestApp`], so no state leaks between tests and the
//! suite is order-independent and parallel-safe.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{header, Request, Response, StatusCode};
use axum::Router;
use chirp_types::api::{
    ApiError, AuthResponse, CreateChirpRequest, CreateUserRequest, ErrorCode, FollowResponse,
    LikeResponse, LoginRequest, Page,
};
use chirp_types::domain::{Chirp, ChirpText, User, Username};
use chirp_types::ids::ChirpId;
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tower::ServiceExt; // for `oneshot`

// ---------------------------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------------------------

/// A mounted application plus the helpers to drive it in-process.
///
/// Holds a cloneable [`Router`] whose shared `AppState` wraps one `InMemoryRepository`. Because the
/// repository lives behind an `Arc` inside the state, cloning the router per request (what
/// `oneshot` requires) preserves the same backing store across a test's requests.
struct TestApp {
    router: Router,
}

/// A decoded HTTP response: the status line plus the raw body bytes, ready to decode.
struct Resp {
    status: StatusCode,
    body: Vec<u8>,
}

impl Resp {
    /// Decodes the body as `T` (a `chirp-types` DTO), failing the test with the raw body on error.
    fn json<T: DeserializeOwned>(&self) -> T {
        serde_json::from_slice(&self.body).unwrap_or_else(|e| {
            panic!(
                "body did not decode as {}: {e}; raw body = {}",
                std::any::type_name::<T>(),
                String::from_utf8_lossy(&self.body)
            )
        })
    }

    /// Decodes the body as the wire error contract [`ApiError`].
    fn api_error(&self) -> ApiError {
        self.json::<ApiError>()
    }
}

impl TestApp {
    /// Builds the real server over a fresh, empty in-memory repository.
    fn new() -> Self {
        Self {
            router: build_app(),
        }
    }

    /// Sends one request through the full transport stack and collects the response.
    async fn send(&self, request: Request<Body>) -> Resp {
        let response: Response<Body> = self
            .router
            .clone()
            .oneshot(request)
            .await
            .expect("router is infallible for a well-formed request");
        let status = response.status();
        let body = response
            .into_body()
            .collect()
            .await
            .expect("response body collects")
            .to_bytes()
            .to_vec();
        Resp { status, body }
    }

    /// `POST`/`PUT`/`DELETE` with an optional JSON body and optional bearer token.
    async fn request<B: Serialize>(
        &self,
        method: &str,
        uri: &str,
        token: Option<&str>,
        json_body: Option<&B>,
    ) -> Resp {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        let body = match json_body {
            Some(value) => {
                builder = builder.header(header::CONTENT_TYPE, "application/json");
                Body::from(serde_json::to_vec(value).expect("request body serializes"))
            }
            None => Body::empty(),
        };
        self.send(builder.body(body).expect("request builds")).await
    }

    /// `GET` with an optional bearer token.
    async fn get(&self, uri: &str, token: Option<&str>) -> Resp {
        let mut builder = Request::builder().method("GET").uri(uri);
        if let Some(token) = token {
            builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        self.send(builder.body(Body::empty()).expect("request builds"))
            .await
    }

    // --- Convenience flows used as fixtures by several tests -------------------------------------

    /// Registers a user via `POST /api/users`, asserting 201, and returns the created profile.
    async fn register(&self, handle: &str) -> User {
        let resp = self
            .request("POST", "/api/users", None, Some(&new_user(handle)))
            .await;
        assert_eq!(
            resp.status,
            StatusCode::CREATED,
            "register({handle}) should be 201"
        );
        resp.json::<User>()
    }

    /// Logs in via `POST /api/sessions`, asserting 200, and returns the auth response (token + user).
    async fn login(&self, handle: &str) -> AuthResponse {
        let body = LoginRequest {
            username: username(handle),
        };
        let resp = self
            .request("POST", "/api/sessions", None, Some(&body))
            .await;
        assert_eq!(resp.status, StatusCode::OK, "login({handle}) should be 200");
        resp.json::<AuthResponse>()
    }

    /// Registers then logs in, returning the profile and a usable bearer token.
    async fn register_and_login(&self, handle: &str) -> (User, String) {
        let user = self.register(handle).await;
        let auth = self.login(handle).await;
        (user, auth.token.0)
    }

    /// Posts a chirp as the authenticated `token`, asserting 201, returning the created chirp.
    async fn post_chirp(&self, token: &str, text: &str) -> Chirp {
        let body = CreateChirpRequest {
            text: ChirpText::parse(text).expect("valid chirp body"),
            reply_to: None,
        };
        let resp = self
            .request("POST", "/api/chirps", Some(token), Some(&body))
            .await;
        assert_eq!(resp.status, StatusCode::CREATED, "post_chirp should be 201");
        resp.json::<Chirp>()
    }
}

/// Builds the application router over a brand-new empty in-memory repository.
fn build_app() -> Router {
    build_router(AppState::new(Arc::new(InMemoryRepository::new())))
}

/// A valid [`Username`] for the given handle (panics on an invalid handle — test author's error).
fn username(handle: &str) -> Username {
    Username::parse(handle).expect("test handle is valid")
}

/// A `CreateUserRequest` with a sensible display name and no bio.
fn new_user(handle: &str) -> CreateUserRequest {
    CreateUserRequest {
        username: username(handle),
        display_name: format!("{handle} display"),
        bio: None,
    }
}

// Pull the crate's public surface into the harness. Kept at the bottom so the `use` list above is
// the wire-type contract and this is the server under test.
use chirp_server::{build_router, AppState, InMemoryRepository};

// ---------------------------------------------------------------------------------------------
// Happy paths — the acceptance surface, end to end.
// ---------------------------------------------------------------------------------------------
mod happy_path {
    use super::*;

    #[tokio::test]
    async fn healthz_reports_liveness() {
        let app = TestApp::new();
        let resp = app.get("/healthz", None).await;
        assert_eq!(resp.status, StatusCode::OK);
        let body: serde_json::Value = resp.json();
        assert_eq!(body["status"], "ok");
        assert_eq!(body["service"], "chirp-server");
        assert!(body["version"].is_string(), "version is reported");
    }

    #[tokio::test]
    async fn create_user_then_fetch_profile_round_trips() {
        let app = TestApp::new();

        // POST /api/users → 201 with the server-assigned, zero-counted profile.
        let created = app.register("alice").await;
        assert_eq!(created.username.as_str(), "alice");
        assert_eq!(created.display_name, "alice display");
        assert_eq!(created.follower_count, 0);
        assert_eq!(created.following_count, 0);

        // GET /api/users/{id} → 200 with the identical profile (full DTO round-trip).
        let resp = app
            .get(&format!("/api/users/{}", created.id.get()), None)
            .await;
        assert_eq!(resp.status, StatusCode::OK);
        assert_eq!(resp.json::<User>(), created);
    }

    #[tokio::test]
    async fn login_issues_a_token_that_authenticates_requests() {
        let app = TestApp::new();
        let created = app.register("bob").await;

        // POST /api/sessions → 200 with a token and the same user profile.
        let auth = app.login("bob").await;
        assert_eq!(auth.user, created);
        assert!(!auth.token.0.is_empty(), "a session token is issued");

        // The issued token actually authenticates an otherwise-401 endpoint.
        let timeline = app.get("/api/timeline", Some(&auth.token.0)).await;
        assert_eq!(timeline.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn post_chirp_appears_on_authors_own_timeline() {
        let app = TestApp::new();
        let (author, token) = app.register_and_login("carol").await;

        let chirp = app.post_chirp(&token, "hello, chirp!").await;
        assert_eq!(chirp.author_id, author.id);
        assert_eq!(chirp.text.as_str(), "hello, chirp!");
        assert_eq!(chirp.like_count, 0);
        assert_eq!(chirp.reply_to, None);

        let resp = app.get("/api/timeline", Some(&token)).await;
        assert_eq!(resp.status, StatusCode::OK);
        let page: Page<Chirp> = resp.json();
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].id, chirp.id);
        assert!(page.next_cursor.is_none());
    }

    #[tokio::test]
    async fn home_timeline_is_scoped_to_self_plus_followees_newest_first() {
        let app = TestApp::new();
        let (_alice, alice_token) = app.register_and_login("alice").await;
        let (bob, bob_token) = app.register_and_login("bob").await;
        let (_carol, carol_token) = app.register_and_login("carol").await;

        let a1 = app.post_chirp(&alice_token, "alice one").await;
        let b1 = app.post_chirp(&bob_token, "bob one").await;
        let _c1 = app.post_chirp(&carol_token, "carol one (unfollowed)").await;
        let a2 = app.post_chirp(&alice_token, "alice two").await;

        // Alice follows bob but not carol.
        let follow = app
            .request::<()>(
                "PUT",
                &format!("/api/users/{}/follow", bob.id.get()),
                Some(&alice_token),
                None,
            )
            .await;
        assert_eq!(follow.status, StatusCode::OK);

        let resp = app.get("/api/timeline", Some(&alice_token)).await;
        assert_eq!(resp.status, StatusCode::OK);
        let page: Page<Chirp> = resp.json();
        let ids: Vec<ChirpId> = page.items.iter().map(|c| c.id).collect();
        // Newest-first: a2, b1, a1 — carol's chirp excluded (not followed).
        assert_eq!(ids, vec![a2.id, b1.id, a1.id]);
    }

    #[tokio::test]
    async fn follow_then_unfollow_moves_the_follower_count() {
        let app = TestApp::new();
        let (_alice, alice_token) = app.register_and_login("alice").await;
        let bob = app.register("bob").await;
        let follow_uri = format!("/api/users/{}/follow", bob.id.get());

        // Follow → following=true, follower_count=1.
        let resp = app
            .request::<()>("PUT", &follow_uri, Some(&alice_token), None)
            .await;
        assert_eq!(resp.status, StatusCode::OK);
        let follow: FollowResponse = resp.json();
        assert_eq!(follow.followee_id, bob.id);
        assert!(follow.following);
        assert_eq!(follow.follower_count, 1);

        // Denormalized count is visible on the fetched profile.
        let bob_profile = app.get(&format!("/api/users/{}", bob.id.get()), None).await;
        assert_eq!(bob_profile.json::<User>().follower_count, 1);

        // Idempotent: following again stays at 1 (PUT semantics).
        let again = app
            .request::<()>("PUT", &follow_uri, Some(&alice_token), None)
            .await;
        assert_eq!(again.json::<FollowResponse>().follower_count, 1);

        // Unfollow → following=false, follower_count=0.
        let resp = app
            .request::<()>("DELETE", &follow_uri, Some(&alice_token), None)
            .await;
        assert_eq!(resp.status, StatusCode::OK);
        let unfollow: FollowResponse = resp.json();
        assert!(!unfollow.following);
        assert_eq!(unfollow.follower_count, 0);
    }

    #[tokio::test]
    async fn like_then_unlike_moves_the_like_count() {
        let app = TestApp::new();
        let (_alice, token) = app.register_and_login("alice").await;
        let chirp = app.post_chirp(&token, "likeable").await;
        let like_uri = format!("/api/chirps/{}/like", chirp.id.get());

        // Like → liked=true, like_count=1.
        let resp = app
            .request::<()>("PUT", &like_uri, Some(&token), None)
            .await;
        assert_eq!(resp.status, StatusCode::OK);
        let like: LikeResponse = resp.json();
        assert_eq!(like.chirp_id, chirp.id);
        assert!(like.liked);
        assert_eq!(like.like_count, 1);

        // Idempotent: liking again stays at 1.
        let again = app
            .request::<()>("PUT", &like_uri, Some(&token), None)
            .await;
        assert_eq!(again.json::<LikeResponse>().like_count, 1);

        // Unlike → liked=false, like_count=0.
        let resp = app
            .request::<()>("DELETE", &like_uri, Some(&token), None)
            .await;
        assert_eq!(resp.status, StatusCode::OK);
        let unlike: LikeResponse = resp.json();
        assert!(!unlike.liked);
        assert_eq!(unlike.like_count, 0);
    }

    #[tokio::test]
    async fn reply_chirp_links_to_its_parent() {
        let app = TestApp::new();
        let (_author, token) = app.register_and_login("alice").await;
        let parent = app.post_chirp(&token, "parent chirp").await;

        let body = CreateChirpRequest {
            text: ChirpText::parse("a reply").expect("valid body"),
            reply_to: Some(parent.id),
        };
        let resp = app
            .request("POST", "/api/chirps", Some(&token), Some(&body))
            .await;
        assert_eq!(resp.status, StatusCode::CREATED);
        assert_eq!(resp.json::<Chirp>().reply_to, Some(parent.id));
    }
}

// ---------------------------------------------------------------------------------------------
// Error paths — asserted against the contract's `error_mapping` table (status + decoded ApiError).
// ---------------------------------------------------------------------------------------------
mod errors {
    use super::*;

    #[tokio::test]
    async fn duplicate_username_is_409_conflict() {
        let app = TestApp::new();
        app.register("alice").await;

        let resp = app
            .request("POST", "/api/users", None, Some(&new_user("alice")))
            .await;
        assert_eq!(resp.status, StatusCode::CONFLICT);
        assert_eq!(resp.api_error().code, ErrorCode::Conflict);
    }

    #[tokio::test]
    async fn unknown_user_id_is_404_not_found() {
        let app = TestApp::new();
        let resp = app.get("/api/users/9999", None).await;
        assert_eq!(resp.status, StatusCode::NOT_FOUND);
        assert_eq!(resp.api_error().code, ErrorCode::NotFound);
    }

    #[tokio::test]
    async fn login_unknown_handle_is_404_not_found() {
        let app = TestApp::new();
        let body = LoginRequest {
            username: username("ghost"),
        };
        let resp = app
            .request("POST", "/api/sessions", None, Some(&body))
            .await;
        assert_eq!(resp.status, StatusCode::NOT_FOUND);
        assert_eq!(resp.api_error().code, ErrorCode::NotFound);
    }

    #[tokio::test]
    async fn missing_token_is_401_unauthorized() {
        let app = TestApp::new();
        // Every authenticated endpoint rejects a token-less request before the handler body runs.
        let resp = app.get("/api/timeline", None).await;
        assert_eq!(resp.status, StatusCode::UNAUTHORIZED);
        assert_eq!(resp.api_error().code, ErrorCode::Unauthorized);
    }

    #[tokio::test]
    async fn invalid_token_is_401_unauthorized() {
        let app = TestApp::new();
        let resp = app.get("/api/timeline", Some("not-a-real-token")).await;
        assert_eq!(resp.status, StatusCode::UNAUTHORIZED);
        assert_eq!(resp.api_error().code, ErrorCode::Unauthorized);
    }

    #[tokio::test]
    async fn self_follow_is_403_forbidden() {
        let app = TestApp::new();
        let (alice, token) = app.register_and_login("alice").await;
        let resp = app
            .request::<()>(
                "PUT",
                &format!("/api/users/{}/follow", alice.id.get()),
                Some(&token),
                None,
            )
            .await;
        assert_eq!(resp.status, StatusCode::FORBIDDEN);
        assert_eq!(resp.api_error().code, ErrorCode::Forbidden);
    }

    #[tokio::test]
    async fn follow_absent_user_is_404_not_found() {
        let app = TestApp::new();
        let (_alice, token) = app.register_and_login("alice").await;
        let resp = app
            .request::<()>("PUT", "/api/users/4242/follow", Some(&token), None)
            .await;
        assert_eq!(resp.status, StatusCode::NOT_FOUND);
        assert_eq!(resp.api_error().code, ErrorCode::NotFound);
    }

    #[tokio::test]
    async fn like_absent_chirp_is_404_not_found() {
        let app = TestApp::new();
        let (_alice, token) = app.register_and_login("alice").await;
        let resp = app
            .request::<()>("PUT", "/api/chirps/4242/like", Some(&token), None)
            .await;
        assert_eq!(resp.status, StatusCode::NOT_FOUND);
        assert_eq!(resp.api_error().code, ErrorCode::NotFound);
    }

    #[tokio::test]
    async fn reply_to_absent_chirp_is_404_not_found() {
        let app = TestApp::new();
        let (_author, token) = app.register_and_login("alice").await;
        let body = CreateChirpRequest {
            text: ChirpText::parse("orphan reply").expect("valid body"),
            reply_to: Some(ChirpId::new(9999)),
        };
        let resp = app
            .request("POST", "/api/chirps", Some(&token), Some(&body))
            .await;
        assert_eq!(resp.status, StatusCode::NOT_FOUND);
        assert_eq!(resp.api_error().code, ErrorCode::NotFound);
    }

    /// A well-formed-JSON body carrying an invalid DTO field value is rejected as the contract's
    /// **400 `VALIDATION_ERROR`** with an [`ApiError`] body — not axum's default plain-text `422`.
    ///
    /// The subtree contract's `error_mapping` maps an `input/DTO validation failure (incl.
    /// chirp-types ValidationError)` to `VALIDATION_ERROR`/400 with field `details`. The
    /// `chirp-types` value objects validate on deserialization (`#[serde(try_from = "String")]`),
    /// so an invalid field value (here a username with a space) fails at the body-extractor's serde
    /// boundary. The [`ValidatedJson`] extractor routes that `JsonRejection` through
    /// `ServerError::InvalidBody`, so the failure surfaces as the contract's `ApiError` with a
    /// `VALIDATION_ERROR` code and a non-empty `details`.
    ///
    /// (Remediates finding F1: the earlier plain `axum::Json` extractor returned `422` plain text,
    /// bypassing the wire error contract. This test previously pinned that non-conforming behavior.)
    #[tokio::test]
    async fn invalid_dto_field_is_rejected_as_400_validation_error() {
        let app = TestApp::new();
        // Valid JSON, but "has space" fails `Username` validation during deserialization.
        let body = serde_json::json!({
            "username": "has space",
            "displayName": "X",
            "bio": null
        });
        let resp = app.request("POST", "/api/users", None, Some(&body)).await;

        // The input is rejected (never created) as the contract's 400 — not axum's default 422.
        assert_eq!(
            resp.status,
            StatusCode::BAD_REQUEST,
            "a well-formed-JSON body with an invalid DTO field must be the contract's 400"
        );
        // And the body IS the `ApiError` wire contract: VALIDATION_ERROR with field details.
        let error = resp.api_error();
        assert_eq!(error.code, ErrorCode::ValidationError);
        assert!(
            !error.details.is_empty(),
            "a validation failure carries at least one FieldError detail"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Pagination — cursor + limit behavior over the home timeline.
// ---------------------------------------------------------------------------------------------
mod pagination {
    use super::*;

    #[tokio::test]
    async fn timeline_paginates_newest_first_with_cursor_and_limit() {
        let app = TestApp::new();
        let (_alice, token) = app.register_and_login("alice").await;

        // Five chirps, ascending ids in post order.
        let mut posted = Vec::new();
        for i in 0..5 {
            posted.push(app.post_chirp(&token, &format!("chirp {i}")).await);
        }

        // Page 1: the two newest (ids 4, 3), with a continuation cursor.
        let resp = app.get("/api/timeline?limit=2", Some(&token)).await;
        assert_eq!(resp.status, StatusCode::OK);
        let page1: Page<Chirp> = resp.json();
        let ids1: Vec<ChirpId> = page1.items.iter().map(|c| c.id).collect();
        assert_eq!(ids1, vec![posted[4].id, posted[3].id]);
        let cursor1 = page1
            .next_cursor
            .clone()
            .expect("more pages remain after page 1");

        // Page 2: strictly older than the cursor (ids 2, 1), with another cursor.
        let resp = app
            .get(
                &format!("/api/timeline?limit=2&cursor={}", cursor1.0),
                Some(&token),
            )
            .await;
        assert_eq!(resp.status, StatusCode::OK);
        let page2: Page<Chirp> = resp.json();
        let ids2: Vec<ChirpId> = page2.items.iter().map(|c| c.id).collect();
        assert_eq!(ids2, vec![posted[2].id, posted[1].id]);
        let cursor2 = page2
            .next_cursor
            .clone()
            .expect("more pages remain after page 2");

        // Page 3: the final chirp (id 0), no further cursor.
        let resp = app
            .get(
                &format!("/api/timeline?limit=2&cursor={}", cursor2.0),
                Some(&token),
            )
            .await;
        assert_eq!(resp.status, StatusCode::OK);
        let page3: Page<Chirp> = resp.json();
        let ids3: Vec<ChirpId> = page3.items.iter().map(|c| c.id).collect();
        assert_eq!(ids3, vec![posted[0].id]);
        assert!(page3.next_cursor.is_none(), "last page has no cursor");

        // The three pages together cover all five chirps exactly once.
        let mut seen: Vec<ChirpId> = ids1;
        seen.extend(ids2);
        seen.extend(ids3);
        assert_eq!(seen.len(), 5);
    }

    #[tokio::test]
    async fn timeline_clamps_an_oversized_limit_without_erroring() {
        let app = TestApp::new();
        let (_alice, token) = app.register_and_login("alice").await;
        for i in 0..3 {
            app.post_chirp(&token, &format!("c{i}")).await;
        }
        // A limit far above the server ceiling is clamped, not rejected: all 3 returned, no cursor.
        let resp = app.get("/api/timeline?limit=100000", Some(&token)).await;
        assert_eq!(resp.status, StatusCode::OK);
        let page: Page<Chirp> = resp.json();
        assert_eq!(page.items.len(), 3);
        assert!(page.next_cursor.is_none());
    }
}
