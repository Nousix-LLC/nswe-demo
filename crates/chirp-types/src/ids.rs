//! Opaque identifier and timestamp newtypes shared by the domain model and the API contract.
//!
//! Every entity id is a [newtype](https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html)
//! over `u64` rather than a bare integer, so the compiler rejects accidentally passing a
//! [`ChirpId`] where a [`UserId`] is expected. Ids are **server-assigned and opaque to clients**:
//! a client never mints one, it only echoes back what the server sent. Each id derives
//! `#[serde(transparent)]`, so `UserId(42)` serializes as the bare JSON number `42`, keeping the
//! wire shape identical to a plain integer while preserving type safety in Rust.
//!
//! [`Timestamp`] is milliseconds since the Unix epoch (UTC), serialized as a JSON number. A
//! newtype rather than a crate type (`chrono`/`time`) keeps this contract dependency-free and
//! language-neutral; consumers convert to their preferred date type at the edge.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Generates an opaque, server-assigned identifier newtype over `u64`.
///
/// The macro keeps the three id types (`UserId`, `ChirpId`, …) byte-for-byte consistent in their
/// derives, serde representation, constructor, accessor, `Display`, and `From<u64>` conversion.
macro_rules! opaque_id {
    ($(#[$meta:meta])* $name:ident, $prefix:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(u64);

        impl $name {
            #[doc = concat!("Wraps a raw `u64` as a [`", stringify!($name), "`].")]
            #[must_use]
            pub const fn new(value: u64) -> Self {
                Self(value)
            }

            /// Returns the underlying raw `u64`.
            #[must_use]
            pub const fn get(self) -> u64 {
                self.0
            }
        }

        impl From<u64> for $name {
            fn from(value: u64) -> Self {
                Self(value)
            }
        }

        impl From<$name> for u64 {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}-{}", $prefix, self.0)
            }
        }
    };
}

opaque_id!(
    /// Opaque, server-assigned identifier for a [`User`](crate::domain::User).
    UserId,
    "user"
);

opaque_id!(
    /// Opaque, server-assigned identifier for a [`Chirp`](crate::domain::Chirp).
    ChirpId,
    "chirp"
);

/// A point in time, measured in **milliseconds since the Unix epoch (UTC)**.
///
/// Serialized as a JSON number. Using epoch-milliseconds (rather than an RFC 3339 string or a
/// date-library type) keeps the contract dependency-free, unambiguous about time zone (always
/// UTC), and trivial to compare and order. Consumers convert to their own date type at the edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Builds a timestamp from milliseconds since the Unix epoch (UTC).
    #[must_use]
    pub const fn from_millis(millis: i64) -> Self {
        Self(millis)
    }

    /// Returns the value as milliseconds since the Unix epoch (UTC).
    #[must_use]
    pub const fn as_millis(self) -> i64 {
        self.0
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}ms", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_id_serializes_transparently_as_a_number() {
        let id = UserId::new(42);
        assert_eq!(serde_json::to_string(&id).unwrap(), "42");
        let back: UserId = serde_json::from_str("42").unwrap();
        assert_eq!(back, id);
    }

    #[test]
    fn ids_are_distinct_types_but_share_representation() {
        // Both serialize as bare numbers...
        assert_eq!(serde_json::to_string(&ChirpId::new(7)).unwrap(), "7");
        // ...and round-trip independently.
        let chirp: ChirpId = serde_json::from_str("7").unwrap();
        assert_eq!(chirp.get(), 7);
    }

    #[test]
    fn id_conversions_and_display() {
        let id = UserId::from(3);
        assert_eq!(u64::from(id), 3);
        assert_eq!(id.to_string(), "user-3");
        assert_eq!(ChirpId::new(9).to_string(), "chirp-9");
    }

    #[test]
    fn timestamp_round_trips_as_a_number() {
        let ts = Timestamp::from_millis(1_700_000_000_000);
        assert_eq!(serde_json::to_string(&ts).unwrap(), "1700000000000");
        let back: Timestamp = serde_json::from_str("1700000000000").unwrap();
        assert_eq!(back, ts);
        assert_eq!(back.as_millis(), 1_700_000_000_000);
    }

    #[test]
    fn timestamps_order_chronologically() {
        let earlier = Timestamp::from_millis(1_000);
        let later = Timestamp::from_millis(2_000);
        assert!(earlier < later);
    }
}
