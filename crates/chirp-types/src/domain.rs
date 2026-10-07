//! The domain model: the entities and value objects of the chirp world.
//!
//! Two kinds of type live here:
//!
//! * **Validated value objects** — [`Username`] and [`ChirpText`] wrap a `String` behind a
//!   checking constructor so an invalid value is *unrepresentable*: you cannot build one, and you
//!   cannot `Deserialize` one, without passing validation (both route through the same
//!   [`TryFrom<String>`] via `#[serde(try_from = "String")]`). Construction failures surface as a
//!   [`ValidationError`] rather than a panic, per library error-handling discipline.
//! * **Entities** — [`User`], [`Chirp`], [`Follow`], [`Like`], and the [`Timeline`] aggregate. These
//!   are plain data carriers whose fields are the authoritative on-the-wire shape. Their documented
//!   invariants are enforced by the server when it constructs them; the types record the invariants
//!   so every consumer reads the same contract.
//!
//! All JSON field names are **camelCase** (`#[serde(rename_all = "camelCase")]`), the explicit,
//! stable wire convention described at the crate root.

use crate::ids::{ChirpId, Timestamp, UserId};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// The maximum length of a [`Username`], in characters.
pub const USERNAME_MAX_CHARS: usize = 15;

/// The maximum length of a [`ChirpText`] body, in characters.
pub const CHIRP_TEXT_MAX_CHARS: usize = 280;

/// Why constructing a validated value object failed.
///
/// This is the library's *construction-time* error, distinct from the wire error contract
/// [`ApiError`](crate::api::ApiError): it is what a validating constructor returns in Rust, never
/// a serialized type. A library never panics on bad input — it returns this instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// A required text value was empty after trimming was *not* applied (empty is never valid).
    Empty {
        /// The name of the field that was empty (e.g. `"username"`).
        field: &'static str,
    },
    /// The value was longer than the field's maximum character count.
    TooLong {
        /// The name of the offending field.
        field: &'static str,
        /// The maximum number of characters the field allows.
        max: usize,
        /// The number of characters actually supplied.
        actual: usize,
    },
    /// The value contained a character the field does not permit.
    InvalidCharacter {
        /// The name of the offending field.
        field: &'static str,
        /// The first disallowed character encountered.
        character: char,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::Empty { field } => write!(f, "{field} must not be empty"),
            ValidationError::TooLong { field, max, actual } => {
                write!(
                    f,
                    "{field} is {actual} characters but at most {max} are allowed"
                )
            }
            ValidationError::InvalidCharacter { field, character } => {
                write!(f, "{field} contains a disallowed character: {character:?}")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// A user's handle: 1..={max} characters of ASCII letters, digits, or underscore.
///
/// The value is validated on every path into the type — direct construction *and* deserialization
/// both go through [`Username::try_from`] — so a `Username` in hand is always well-formed. Serialized
/// transparently as its inner string.
///
/// [`try_from`]: Username::try_from
#[doc = concat!("The maximum length is [`USERNAME_MAX_CHARS`] (", stringify!(15), ").")]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Username(String);

impl Username {
    /// Validates and constructs a [`Username`].
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if the handle is empty, longer than
    /// [`USERNAME_MAX_CHARS`], or contains a character other than an ASCII letter, digit, or
    /// underscore.
    pub fn parse(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let count = value.chars().count();
        if count == 0 {
            return Err(ValidationError::Empty { field: "username" });
        }
        if count > USERNAME_MAX_CHARS {
            return Err(ValidationError::TooLong {
                field: "username",
                max: USERNAME_MAX_CHARS,
                actual: count,
            });
        }
        if let Some(bad) = value
            .chars()
            .find(|c| !(c.is_ascii_alphanumeric() || *c == '_'))
        {
            return Err(ValidationError::InvalidCharacter {
                field: "username",
                character: bad,
            });
        }
        Ok(Self(value))
    }

    /// Borrows the handle as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the newtype, returning the owned inner string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl TryFrom<String> for Username {
    type Error = ValidationError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl FromStr for Username {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl From<Username> for String {
    fn from(username: Username) -> Self {
        username.0
    }
}

impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The body of a chirp: 1..={max} characters, measured by Unicode scalar value, not bytes.
///
/// Validated on construction and deserialization alike (see [`ChirpText::try_from`]). Serialized
/// transparently as its inner string.
#[doc = concat!("The maximum length is [`CHIRP_TEXT_MAX_CHARS`] (", stringify!(280), ").")]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ChirpText(String);

impl ChirpText {
    /// Validates and constructs a [`ChirpText`].
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if the body is empty or longer than
    /// [`CHIRP_TEXT_MAX_CHARS`] characters.
    pub fn parse(value: impl Into<String>) -> Result<Self, ValidationError> {
        let value = value.into();
        let count = value.chars().count();
        if count == 0 {
            return Err(ValidationError::Empty { field: "text" });
        }
        if count > CHIRP_TEXT_MAX_CHARS {
            return Err(ValidationError::TooLong {
                field: "text",
                max: CHIRP_TEXT_MAX_CHARS,
                actual: count,
            });
        }
        Ok(Self(value))
    }

    /// Borrows the body as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the newtype, returning the owned inner string.
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl TryFrom<String> for ChirpText {
    type Error = ValidationError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl FromStr for ChirpText {
    type Err = ValidationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl From<ChirpText> for String {
    fn from(text: ChirpText) -> Self {
        text.0
    }
}

impl fmt::Display for ChirpText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A user account and public profile.
///
/// `followerCount` / `followingCount` are server-maintained denormalized counts; clients treat
/// them as read-only. `bio` is absent (`null`) when the user has not set one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// Opaque, server-assigned identifier.
    pub id: UserId,
    /// The user's unique handle.
    pub username: Username,
    /// The user's chosen display name (free-form; not unique).
    pub display_name: String,
    /// Optional free-form profile biography.
    pub bio: Option<String>,
    /// When the account was created.
    pub created_at: Timestamp,
    /// Number of accounts following this user (denormalized, read-only to clients).
    pub follower_count: u64,
    /// Number of accounts this user follows (denormalized, read-only to clients).
    pub following_count: u64,
}

/// A single post ("chirp").
///
/// `replyTo` is `Some` when the chirp is a reply to another chirp, enabling threaded
/// conversations; it is `null` for a top-level chirp. `likeCount` is a server-maintained
/// denormalized count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chirp {
    /// Opaque, server-assigned identifier.
    pub id: ChirpId,
    /// The id of the user who authored the chirp.
    pub author_id: UserId,
    /// The validated body text.
    pub text: ChirpText,
    /// When the chirp was posted.
    pub created_at: Timestamp,
    /// Number of likes (denormalized, read-only to clients).
    pub like_count: u64,
    /// The chirp this one replies to, if any.
    pub reply_to: Option<ChirpId>,
}

/// A directed "follows" edge: `followerId` follows `followeeId`.
///
/// Invariant (enforced by the server): `followerId != followeeId` — a user cannot follow
/// themselves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    /// The user who is following.
    pub follower_id: UserId,
    /// The user being followed.
    pub followee_id: UserId,
    /// When the follow relationship was created.
    pub created_at: Timestamp,
}

/// A "like" edge: `userId` likes `chirpId`.
///
/// Invariant (enforced by the server): at most one `Like` exists for a given
/// (`userId`, `chirpId`) pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Like {
    /// The user who liked the chirp.
    pub user_id: UserId,
    /// The chirp that was liked.
    pub chirp_id: ChirpId,
    /// When the like was recorded.
    pub created_at: Timestamp,
}

/// A materialized timeline: an ordered run of chirps shown to a viewer.
///
/// Ordering invariant: `chirps` is **reverse-chronological** (newest first). This is the domain
/// aggregate; the paginated API representation is [`Page<Chirp>`](crate::api::Page), returned by
/// the timeline query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    /// The chirps, newest first.
    pub chirps: Vec<Chirp>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn username_accepts_valid_handles() {
        assert!(Username::parse("alice_01").is_ok());
        assert_eq!(Username::parse("bob").unwrap().as_str(), "bob");
        // "foo".parse::<Username>() goes through the same validation.
        assert!("foo".parse::<Username>().is_ok());
    }

    #[test]
    fn username_rejects_invalid_handles() {
        assert_eq!(
            Username::parse(""),
            Err(ValidationError::Empty { field: "username" })
        );
        assert!(matches!(
            Username::parse("a".repeat(USERNAME_MAX_CHARS + 1)),
            Err(ValidationError::TooLong {
                field: "username",
                ..
            })
        ));
        assert!(matches!(
            Username::parse("not valid!"),
            Err(ValidationError::InvalidCharacter {
                field: "username",
                character: ' '
            })
        ));
    }

    #[test]
    fn username_validates_on_deserialize() {
        // A valid handle deserializes...
        let ok: Username = serde_json::from_str("\"alice\"").unwrap();
        assert_eq!(ok.as_str(), "alice");
        // ...an invalid one is rejected by serde via the same TryFrom path.
        assert!(serde_json::from_str::<Username>("\"has space\"").is_err());
        assert!(serde_json::from_str::<Username>("\"\"").is_err());
    }

    #[test]
    fn chirp_text_enforces_length_by_chars_not_bytes() {
        // 280 multi-byte chars is valid (length is counted in characters, not bytes).
        let emojis = "🐦".repeat(CHIRP_TEXT_MAX_CHARS);
        assert!(ChirpText::parse(emojis).is_ok());
        // 281 is too long.
        assert!(matches!(
            ChirpText::parse("x".repeat(CHIRP_TEXT_MAX_CHARS + 1)),
            Err(ValidationError::TooLong { field: "text", .. })
        ));
        assert_eq!(
            ChirpText::parse(""),
            Err(ValidationError::Empty { field: "text" })
        );
    }

    #[test]
    fn user_round_trips_with_camel_case_fields() {
        let user = User {
            id: UserId::new(1),
            username: Username::parse("alice").unwrap(),
            display_name: "Alice".to_owned(),
            bio: None,
            created_at: Timestamp::from_millis(1_700_000_000_000),
            follower_count: 10,
            following_count: 3,
        };
        let json = serde_json::to_string(&user).unwrap();
        // Field renaming is explicit and load-bearing.
        assert!(json.contains("\"displayName\""));
        assert!(json.contains("\"createdAt\""));
        assert!(json.contains("\"followerCount\""));
        assert!(!json.contains("\"display_name\""));
        let back: User = serde_json::from_str(&json).unwrap();
        assert_eq!(back, user);
    }

    #[test]
    fn chirp_round_trips_including_optional_reply() {
        let reply = Chirp {
            id: ChirpId::new(100),
            author_id: UserId::new(1),
            text: ChirpText::parse("hello there").unwrap(),
            created_at: Timestamp::from_millis(1_700_000_001_000),
            like_count: 0,
            reply_to: Some(ChirpId::new(99)),
        };
        let back: Chirp = serde_json::from_str(&serde_json::to_string(&reply).unwrap()).unwrap();
        assert_eq!(back, reply);
        assert_eq!(back.reply_to, Some(ChirpId::new(99)));
    }

    #[test]
    fn timeline_preserves_order() {
        let timeline = Timeline {
            chirps: vec![
                Chirp {
                    id: ChirpId::new(2),
                    author_id: UserId::new(1),
                    text: ChirpText::parse("newer").unwrap(),
                    created_at: Timestamp::from_millis(2_000),
                    like_count: 0,
                    reply_to: None,
                },
                Chirp {
                    id: ChirpId::new(1),
                    author_id: UserId::new(1),
                    text: ChirpText::parse("older").unwrap(),
                    created_at: Timestamp::from_millis(1_000),
                    like_count: 0,
                    reply_to: None,
                },
            ],
        };
        let back: Timeline =
            serde_json::from_str(&serde_json::to_string(&timeline).unwrap()).unwrap();
        assert_eq!(back, timeline);
        assert!(back.chirps[0].created_at > back.chirps[1].created_at);
    }
}
