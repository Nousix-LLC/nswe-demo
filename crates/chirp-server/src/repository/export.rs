//! The serialized shape of a user's data export (the `data_export` capability, REQ-DOM-010).
//!
//! [`export_user_data`](crate::repository::ChirpRepository::export_user_data) returns the export as
//! an **opaque serialized string** — the seam deliberately introduces no new `chirp-types` DTO
//! (see the trait method's docs). This module is the **single source of truth** for what that
//! string contains and how it is ordered, so the two backings (the durable
//! [`PostgresRepository`](crate::repository::postgres::PostgresRepository) and the retained
//! [`InMemoryRepository`](crate::repository::InMemoryRepository)) produce byte-identical exports
//! from the same data rather than each re-deriving the shape. It composes against the canonical
//! `chirp-types` surface and re-declares none of it.
//!
//! The document is a JSON object (camelCase keys) carrying the user's profile, their authored
//! (non-removed) chirps, their follow graph (the ids they follow and the ids that follow them), and
//! the ids of the chirps they like. [`ExportDocument::new`] imposes a **deterministic order** on
//! every collection, so the serialized string is stable across backings and across runs (a property
//! the export tests assert directly).

use serde::{Deserialize, Serialize};

use chirp_types::domain::{Chirp, User};
use chirp_types::ids::{ChirpId, UserId};

use crate::error::ServerError;

/// A user's full data export, serialized to the opaque string
/// [`export_user_data`](crate::repository::ChirpRepository::export_user_data) returns.
///
/// Built with [`ExportDocument::new`], which fixes the ordering of every collection so both backings
/// serialize identical bytes. `Deserialize` is derived so tests can parse an export back and assert
/// its contents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ExportDocument {
    /// The exporting user's public profile.
    pub profile: User,
    /// The user's authored, non-removed chirps, newest first (id descending).
    pub chirps: Vec<Chirp>,
    /// The ids of the accounts the user follows, ascending.
    pub following: Vec<UserId>,
    /// The ids of the accounts that follow the user, ascending.
    pub followers: Vec<UserId>,
    /// The ids of the chirps the user likes, ascending.
    pub likes: Vec<ChirpId>,
}

impl ExportDocument {
    /// Assembles an export document, imposing a deterministic order on every collection so the
    /// serialized form is identical regardless of the order the caller gathered the rows: chirps
    /// newest-first (id descending, matching the timeline), and the id lists ascending.
    pub(crate) fn new(
        profile: User,
        mut chirps: Vec<Chirp>,
        mut following: Vec<UserId>,
        mut followers: Vec<UserId>,
        mut likes: Vec<ChirpId>,
    ) -> Self {
        chirps.sort_by_key(|chirp| std::cmp::Reverse(chirp.id.get()));
        // `UserId` / `ChirpId` derive `Ord` (ascending in their opaque value), so the id lists sort
        // directly — a deterministic order independent of how the caller gathered them.
        following.sort_unstable();
        followers.sort_unstable();
        likes.sort_unstable();
        Self {
            profile,
            chirps,
            following,
            followers,
            likes,
        }
    }

    /// Serializes the document to the opaque JSON string the export capability returns.
    ///
    /// # Errors
    ///
    /// [`ServerError::Internal`] if serialization fails (which, for these in-memory `chirp-types`
    /// values, does not occur in practice) — reported as a failed export, never a partial string.
    pub(crate) fn to_json(&self) -> Result<String, ServerError> {
        serde_json::to_string(self).map_err(|error| {
            ServerError::Internal(format!("failed to serialize the data export: {error}"))
        })
    }
}
