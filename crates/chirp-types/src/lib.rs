//! `chirp-types` — the shared domain model and client↔server API contract for **chirp**, the
//! Twitter-style application in this engagement.
//!
//! This crate is the **single source of truth for the types that cross the wire**. The
//! [`chirp-server`] and the [`chirp-frontend`] both depend on it, so a type defined here once is
//! the same type on both ends of every request: the server cannot encode a shape the frontend
//! cannot decode, because there is only one definition. The crate is pure data and validation —
//! no I/O, no framework, no async — which is what lets both a native server and a wasm frontend
//! import it unchanged.
//!
//! [`chirp-server`]: https://github.com/Nousix-LLC/nswe-demo
//! [`chirp-frontend`]: https://github.com/Nousix-LLC/nswe-demo
//!
//! # Module map
//!
//! * [`ids`] — opaque, type-safe identifier newtypes ([`UserId`](ids::UserId),
//!   [`ChirpId`](ids::ChirpId)) and the [`Timestamp`](ids::Timestamp) value type.
//! * [`domain`] — the entities ([`User`](domain::User), [`Chirp`](domain::Chirp),
//!   [`Follow`](domain::Follow), [`Like`](domain::Like), [`Timeline`](domain::Timeline)) and the
//!   validated value objects ([`Username`](domain::Username), [`ChirpText`](domain::ChirpText)).
//! * [`api`] — the request/response DTOs, the [`Page`](api::Page) pagination envelope, and the
//!   wire error contract ([`ApiError`](api::ApiError) / [`ErrorCode`](api::ErrorCode)).
//!
//! # The serde field-naming convention (stable contract)
//!
//! Every type derives [`serde::Serialize`] and [`serde::Deserialize`], and every multi-word field
//! is renamed to **camelCase** via `#[serde(rename_all = "camelCase")]`. This rename is
//! deliberate and load-bearing: because the JSON key is pinned by the attribute rather than by the
//! Rust field identifier, a future refactor that renames a Rust field does **not** change the wire
//! format. Two further conventions complete the picture:
//!
//! * **Opaque wrappers are transparent.** Id, [`Cursor`](api::Cursor), and
//!   [`SessionToken`](api::SessionToken) newtypes use `#[serde(transparent)]`, so they serialize
//!   as the bare inner value (a number or string), not as a wrapping object.
//! * **Validated value objects validate on the way in.** [`Username`](domain::Username) and
//!   [`ChirpText`](domain::ChirpText) use `#[serde(try_from = "String")]`, so deserializing an
//!   invalid value *fails* rather than producing an unchecked instance — the same guarantee their
//!   constructors give, extended to the wire.
//! * **Error categories are `SCREAMING_SNAKE_CASE`.** [`ErrorCode`](api::ErrorCode) serializes as
//!   e.g. `"VALIDATION_ERROR"`, the conventional machine-readable REST error-code shape.
//!
//! Timestamps are epoch-milliseconds (UTC) as JSON numbers; see [`ids::Timestamp`].
//!
//! # Example
//!
//! ```
//! use chirp_types::prelude::*;
//!
//! // Build a request the frontend would send and the server would accept.
//! let req = CreateChirpRequest {
//!     text: ChirpText::parse("hello, chirp!").expect("valid body"),
//!     reply_to: None,
//! };
//!
//! // Both sides serialize/deserialize it through the one shared definition.
//! let json = serde_json::to_string(&req).unwrap();
//! let decoded: CreateChirpRequest = serde_json::from_str(&json).unwrap();
//! assert_eq!(decoded, req);
//! ```

pub mod api;
pub mod domain;
pub mod ids;

/// The common import surface: `use chirp_types::prelude::*;`.
///
/// Brings the identifier and timestamp types, the domain entities and value objects, and the core
/// API DTOs into scope. This set is the crate's stable public surface; growth is additive-only.
pub mod prelude {
    pub use crate::api::{
        ApiError, AuthResponse, CreateChirpRequest, CreateUserRequest, Cursor, ErrorCode,
        FieldError, FollowResponse, LikeResponse, LoginRequest, Page, SessionToken, TimelineQuery,
    };
    pub use crate::domain::{
        Chirp, ChirpText, Follow, Like, Timeline, User, Username, ValidationError,
    };
    pub use crate::ids::{ChirpId, Timestamp, UserId};
}
