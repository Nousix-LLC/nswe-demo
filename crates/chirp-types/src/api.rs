//! The client↔server API contract: request and response DTOs, pagination, and the wire error type.
//!
//! This module is the **single source of truth** for the REST contract. The server implements
//! endpoints that accept these request types and return these response types; the frontend builds
//! the requests and decodes the responses from the very same definitions, so the two sides cannot
//! drift. Where a response simply *is* an entity, the contract reuses the
//! [domain](crate::domain) type directly rather than cloning its shape into a parallel DTO;
//! dedicated DTOs exist only where the wire shape genuinely differs from an entity (client inputs,
//! pagination envelopes, action results, errors).
//!
//! As everywhere in this crate, JSON field names are camelCase and the convention is explicit.
//!
//! ## Endpoint ↔ type map (informative)
//!
//! | Endpoint (illustrative) | Request body | Response body |
//! |-------------------------|--------------|---------------|
//! | `POST /sessions`        | [`LoginRequest`]  | [`AuthResponse`] |
//! | `POST /users`           | [`CreateUserRequest`] | [`User`] |
//! | `POST /chirps`          | [`CreateChirpRequest`] | [`Chirp`](crate::domain::Chirp) |
//! | `GET  /timeline`        | [`TimelineQuery`] (query string) | [`Page`]`<`[`Chirp`](crate::domain::Chirp)`>` |
//! | `PUT/DELETE /users/{id}/follow` | — | [`FollowResponse`] |
//! | `PUT/DELETE /chirps/{id}/like`  | — | [`LikeResponse`] |
//!
//! The URLs above are illustrative; this crate fixes the *types*, not the routes.

use crate::domain::{User, Username};
use crate::ids::{ChirpId, UserId};
use serde::{Deserialize, Serialize};

/// An opaque pagination cursor.
///
/// Clients treat it as a opaque token: read it from a [`Page::next_cursor`] and send it back in
/// the next [`TimelineQuery`]. Only the server interprets its contents. Serialized transparently
/// as a string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Cursor(pub String);

/// An opaque bearer session token returned by authentication.
///
/// The token's internal structure (JWT, random string, …) is a server concern and intentionally
/// not part of this contract. Serialized transparently as a string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionToken(pub String);

/// A single page of a larger collection.
///
/// Generic over the item type so one envelope serves every paginated endpoint. `nextCursor` is
/// `Some` when more items follow (pass it to the next request) and `null` on the last page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    /// The items on this page, in the endpoint's defined order.
    pub items: Vec<T>,
    /// Cursor to fetch the next page, or `None` if this is the last page.
    pub next_cursor: Option<Cursor>,
}

impl<T> Page<T> {
    /// Builds a page from its items and optional continuation cursor.
    #[must_use]
    pub fn new(items: Vec<T>, next_cursor: Option<Cursor>) -> Self {
        Self { items, next_cursor }
    }

    /// Returns `true` when the page carries no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of items on this page.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

/// Request body for authenticating / establishing a session.
///
/// Authentication for this demo is handle-based; real credential exchange (passwords, OAuth) is
/// deliberately outside the type contract. The server issues a [`SessionToken`] in the
/// [`AuthResponse`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    /// The handle to authenticate as.
    pub username: Username,
}

/// Response to a successful [`LoginRequest`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    /// The authenticated user's profile.
    pub user: User,
    /// The bearer token to present on subsequent authenticated requests.
    pub token: SessionToken,
}

/// Request body to register a new user. The server assigns the id, timestamps, and counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    /// The desired unique handle.
    pub username: Username,
    /// The display name to show (free-form).
    pub display_name: String,
    /// Optional initial profile biography.
    pub bio: Option<String>,
}

/// Request body to post a new chirp. The server assigns the id, timestamp, and counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChirpRequest {
    /// The validated body text.
    pub text: crate::domain::ChirpText,
    /// The chirp being replied to, if this is a reply.
    pub reply_to: Option<ChirpId>,
}

/// Query parameters for a timeline request.
///
/// Both fields are optional: omit `cursor` for the first page, and omit `limit` to accept the
/// server's default page size. Absent fields are omitted from the serialized form entirely.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineQuery {
    /// Maximum number of chirps to return (server clamps to its own maximum).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Continuation cursor from a previous [`Page::next_cursor`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
}

/// Result of a follow / unfollow action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowResponse {
    /// The user the action targeted.
    pub followee_id: UserId,
    /// The resulting state: `true` if the caller now follows the target.
    pub following: bool,
    /// The target's follower count after the action.
    pub follower_count: u64,
}

/// Result of a like / unlike action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeResponse {
    /// The chirp the action targeted.
    pub chirp_id: ChirpId,
    /// The resulting state: `true` if the caller now likes the chirp.
    pub liked: bool,
    /// The chirp's like count after the action.
    pub like_count: u64,
}

/// A machine-readable error category.
///
/// Serialized as `SCREAMING_SNAKE_CASE` strings (e.g. `"VALIDATION_ERROR"`), matching the
/// REST-API error-contract convention. The variant list is the stable set of categories a client
/// may branch on; the human-readable detail travels in [`ApiError::message`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// The request body or parameters failed validation.
    ValidationError,
    /// Authentication is required or the supplied token was invalid.
    Unauthorized,
    /// The authenticated caller is not permitted to perform the action.
    Forbidden,
    /// The addressed resource does not exist.
    NotFound,
    /// The request conflicts with the current state (e.g. duplicate handle).
    Conflict,
    /// An unexpected server-side error occurred.
    Internal,
}

/// One field-level problem within a [`ApiError`], for validation failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldError {
    /// The offending field's name (e.g. `"username"`).
    pub field: String,
    /// A human-readable description of the problem with this field.
    pub message: String,
}

/// The wire error contract returned by every endpoint on failure.
///
/// `code` is the machine-readable category a client branches on; `message` is human-readable;
/// `details` carries per-field problems for validation failures and is omitted from the
/// serialized form when empty. This is distinct from
/// [`ValidationError`](crate::domain::ValidationError), which is the library's in-process
/// construction error and never appears on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    /// The machine-readable error category.
    pub code: ErrorCode,
    /// A human-readable explanation suitable for logging or display.
    pub message: String,
    /// Field-level details, present for validation failures.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<FieldError>,
}

impl ApiError {
    /// Builds an error with a code and message and no field details.
    #[must_use]
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: Vec::new(),
        }
    }

    /// Attaches field-level details (builder style), e.g. for a validation error.
    #[must_use]
    pub fn with_details(mut self, details: Vec<FieldError>) -> Self {
        self.details = details;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Chirp, ChirpText};
    use crate::ids::Timestamp;

    #[test]
    fn create_chirp_request_round_trips() {
        let req = CreateChirpRequest {
            text: ChirpText::parse("hello world").unwrap(),
            reply_to: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"replyTo\""));
        let back: CreateChirpRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn timeline_query_omits_absent_optional_fields() {
        let empty = TimelineQuery::default();
        assert_eq!(serde_json::to_string(&empty).unwrap(), "{}");

        let with_limit = TimelineQuery {
            limit: Some(20),
            cursor: None,
        };
        let json = serde_json::to_string(&with_limit).unwrap();
        assert_eq!(json, "{\"limit\":20}");
        let back: TimelineQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(back, with_limit);
    }

    #[test]
    fn page_is_generic_and_round_trips() {
        let page = Page::new(
            vec![Chirp {
                id: ChirpId::new(1),
                author_id: UserId::new(1),
                text: ChirpText::parse("hi").unwrap(),
                created_at: Timestamp::from_millis(1_000),
                like_count: 0,
                reply_to: None,
            }],
            Some(Cursor("opaque-token".to_owned())),
        );
        assert_eq!(page.len(), 1);
        assert!(!page.is_empty());
        let json = serde_json::to_string(&page).unwrap();
        assert!(json.contains("\"nextCursor\""));
        let back: Page<Chirp> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, page);
    }

    #[test]
    fn error_code_serializes_as_screaming_snake_case() {
        assert_eq!(
            serde_json::to_string(&ErrorCode::ValidationError).unwrap(),
            "\"VALIDATION_ERROR\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::NotFound).unwrap(),
            "\"NOT_FOUND\""
        );
        let back: ErrorCode = serde_json::from_str("\"CONFLICT\"").unwrap();
        assert_eq!(back, ErrorCode::Conflict);
    }

    #[test]
    fn api_error_omits_empty_details_but_keeps_them_when_present() {
        let bare = ApiError::new(ErrorCode::NotFound, "no such chirp");
        let json = serde_json::to_string(&bare).unwrap();
        assert!(!json.contains("details"));
        let back: ApiError = serde_json::from_str(&json).unwrap();
        assert_eq!(back, bare);

        let detailed =
            ApiError::new(ErrorCode::ValidationError, "invalid input").with_details(vec![
                FieldError {
                    field: "username".to_owned(),
                    message: "must not be empty".to_owned(),
                },
            ]);
        let json = serde_json::to_string(&detailed).unwrap();
        assert!(json.contains("\"details\""));
        let back: ApiError = serde_json::from_str(&json).unwrap();
        assert_eq!(back, detailed);
    }

    #[test]
    fn follow_and_like_responses_round_trip() {
        let follow = FollowResponse {
            followee_id: UserId::new(2),
            following: true,
            follower_count: 5,
        };
        let back: FollowResponse =
            serde_json::from_str(&serde_json::to_string(&follow).unwrap()).unwrap();
        assert_eq!(back, follow);

        let like = LikeResponse {
            chirp_id: ChirpId::new(3),
            liked: false,
            like_count: 0,
        };
        let back: LikeResponse =
            serde_json::from_str(&serde_json::to_string(&like).unwrap()).unwrap();
        assert_eq!(back, like);
    }
}
